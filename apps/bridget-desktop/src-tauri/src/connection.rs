//! Connexions Bridget Desktop : endpoint versionné, tunnel et relais prouvé.

use crate::profile::{ConnectionErrorCategory, ConnectionSession, ConnectionState, RelayEndpoint};
use crate::ssh::{OwnedTunnel, discovery_invocation};
use serde::Deserialize;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const ENDPOINT_VERSION: u8 = 1;
pub const LOCAL_ENDPOINT_PROGRAM: &str = "bridget";
pub const LOCAL_ENDPOINT_ARGS: [&str; 3] = ["ui", "endpoint", "--json"];
const RELAY_READY_TIMEOUT: Duration = Duration::from_secs(4);
const RELAY_RETRY_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndpointDocument {
    version: u8,
    port: u16,
    token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ConnectionStatus {
    pub profile_id: String,
    pub state: ConnectionState,
    pub category: Option<ConnectionErrorCategory>,
}

impl ConnectionStatus {
    pub fn from_session(session: &ConnectionSession) -> Self {
        Self {
            profile_id: session.profile_id.clone(),
            state: session.state.clone(),
            category: session
                .last_error
                .as_ref()
                .map(|error| error.category.clone()),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct InvalidTransition {
    from: ConnectionState,
    to: ConnectionState,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Transition de connexion interdite : {:?} vers {:?}.",
            self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}

pub fn transition(
    session: &mut ConnectionSession,
    next: ConnectionState,
) -> Result<ConnectionStatus, InvalidTransition> {
    let allowed = matches!(
        (&session.state, &next),
        (
            ConnectionState::Disconnected,
            ConnectionState::ConnectingSsh
                | ConnectionState::AwaitingHostApproval
                | ConnectionState::CheckingRelay
                | ConnectionState::Closed
        ) | (
            ConnectionState::ConnectingSsh,
            ConnectionState::AwaitingHostApproval
                | ConnectionState::OpeningTunnel
                | ConnectionState::Reconnecting
                | ConnectionState::Failed
                | ConnectionState::Closed
        ) | (
            ConnectionState::AwaitingHostApproval,
            ConnectionState::ConnectingSsh | ConnectionState::Closed
        ) | (
            ConnectionState::OpeningTunnel,
            ConnectionState::CheckingRelay
                | ConnectionState::Reconnecting
                | ConnectionState::Failed
                | ConnectionState::Closed
        ) | (
            ConnectionState::CheckingRelay,
            ConnectionState::Connected
                | ConnectionState::Reconnecting
                | ConnectionState::Failed
                | ConnectionState::Closed
        ) | (
            ConnectionState::Connected,
            ConnectionState::Reconnecting | ConnectionState::Disconnected | ConnectionState::Closed
        ) | (
            ConnectionState::Reconnecting,
            ConnectionState::ConnectingSsh
                | ConnectionState::CheckingRelay
                | ConnectionState::Failed
                | ConnectionState::Closed
        ) | (
            ConnectionState::Failed,
            ConnectionState::Reconnecting | ConnectionState::Disconnected | ConnectionState::Closed
        )
    );
    if !allowed {
        return Err(InvalidTransition {
            from: session.state.clone(),
            to: next,
        });
    }
    if next == ConnectionState::Connected && session.endpoint().is_none() {
        return Err(InvalidTransition {
            from: session.state.clone(),
            to: next,
        });
    }
    session.state = next;
    Ok(ConnectionStatus::from_session(session))
}

#[derive(Debug)]
pub enum ConnectionError {
    EndpointVersion,
    EndpointInvalid,
    SshUnavailable,
    TunnelUnavailable,
    RelayUnavailable,
    LocalUnavailable,
    HostApprovalRequired,
}

impl ConnectionError {
    pub fn category(&self) -> ConnectionErrorCategory {
        match self {
            Self::EndpointVersion => ConnectionErrorCategory::UnsupportedEndpointVersion,
            Self::EndpointInvalid => ConnectionErrorCategory::InvalidEndpoint,
            Self::SshUnavailable => ConnectionErrorCategory::SshUnavailable,
            Self::TunnelUnavailable => ConnectionErrorCategory::Tunnel,
            Self::RelayUnavailable => ConnectionErrorCategory::RelayUnavailable,
            Self::LocalUnavailable => ConnectionErrorCategory::RelayUnavailable,
            Self::HostApprovalRequired => ConnectionErrorCategory::HostIdentity,
        }
    }
}

impl std::fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndpointVersion => {
                formatter.write_str("Le relais utilise une version d'endpoint non prise en charge.")
            }
            Self::EndpointInvalid => {
                formatter.write_str("Le relais a renvoyé un endpoint invalide.")
            }
            Self::SshUnavailable => {
                formatter.write_str("La connexion SSH n'a pas pu être établie.")
            }
            Self::LocalUnavailable => {
                formatter.write_str("Le relais Bridget de cet ordinateur est indisponible.")
            }
            Self::TunnelUnavailable => formatter.write_str("Le tunnel SSH n'a pas pu être ouvert."),
            Self::RelayUnavailable => formatter
                .write_str("Le relais Bridget ne répond pas encore à travers la connexion."),
            Self::HostApprovalRequired => formatter
                .write_str("L'identité SSH du serveur doit être approuvée avant la connexion."),
        }
    }
}

impl std::error::Error for ConnectionError {}

pub trait RemoteTransport {
    fn discover_endpoint(
        &mut self,
        profile: &crate::profile::ConnectionProfile,
    ) -> Result<RelayEndpoint, ConnectionError>;
    fn open_tunnel(
        &mut self,
        profile: &crate::profile::ConnectionProfile,
        remote_port: u16,
    ) -> Result<u16, ConnectionError>;
    fn close(&mut self);
}

pub trait RelayProbe {
    fn check(&mut self, local_port: u16, endpoint: &RelayEndpoint) -> Result<(), ConnectionError>;
}

pub struct SshRemoteTransport {
    known_hosts: std::path::PathBuf,
    tunnel: Option<OwnedTunnel>,
}

impl SshRemoteTransport {
    pub fn new(known_hosts: impl Into<std::path::PathBuf>) -> Self {
        Self {
            known_hosts: known_hosts.into(),
            tunnel: None,
        }
    }

    /// Port local réellement possédé par ce transport, après ouverture du tunnel.
    pub fn local_port(&self) -> Option<u16> {
        self.tunnel.as_ref().map(|tunnel| tunnel.local_port)
    }

    /// Ne sonde que l'enfant SSH possédé par cette connexion.
    pub fn is_running(&mut self) -> Result<bool, ConnectionError> {
        match self.tunnel.as_mut() {
            Some(tunnel) => tunnel
                .is_running()
                .map_err(|_| ConnectionError::TunnelUnavailable),
            None => Ok(false),
        }
    }
}

impl RemoteTransport for SshRemoteTransport {
    fn discover_endpoint(
        &mut self,
        profile: &crate::profile::ConnectionProfile,
    ) -> Result<RelayEndpoint, ConnectionError> {
        let invocation = discovery_invocation(profile, &self.known_hosts)
            .map_err(|_| ConnectionError::SshUnavailable)?;
        let output = invocation
            .spawn()
            .and_then(|child| child.wait_with_output())
            .map_err(|_| ConnectionError::SshUnavailable)?;
        if !output.status.success() {
            return Err(ConnectionError::SshUnavailable);
        }
        parse_endpoint_document(&output.stdout)
    }

    fn open_tunnel(
        &mut self,
        profile: &crate::profile::ConnectionProfile,
        remote_port: u16,
    ) -> Result<u16, ConnectionError> {
        let tunnel = OwnedTunnel::open(profile, &self.known_hosts, remote_port)
            .map_err(|_| ConnectionError::TunnelUnavailable)?;
        let local_port = tunnel.local_port;
        self.tunnel = Some(tunnel);
        Ok(local_port)
    }

    fn close(&mut self) {
        if let Some(tunnel) = self.tunnel.take() {
            let _ = tunnel.close();
        }
    }
}

impl Drop for SshRemoteTransport {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct HttpRelayProbe;

impl RelayProbe for HttpRelayProbe {
    fn check(&mut self, local_port: u16, endpoint: &RelayEndpoint) -> Result<(), ConnectionError> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, local_port));
        let deadline = Instant::now() + RELAY_READY_TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ConnectionError::RelayUnavailable);
            }
            let attempt_timeout = remaining.min(Duration::from_millis(250));
            if check_relay_once(address, endpoint, attempt_timeout).is_ok() {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ConnectionError::RelayUnavailable);
            }
            std::thread::sleep(RELAY_RETRY_INTERVAL.min(remaining));
        }
    }
}

fn check_relay_once(
    address: SocketAddr,
    endpoint: &RelayEndpoint,
    timeout: Duration,
) -> Result<(), ConnectionError> {
    let mut stream = TcpStream::connect_timeout(&address, timeout)
        .map_err(|_| ConnectionError::RelayUnavailable)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| ConnectionError::RelayUnavailable)?;
    let request = format!(
        "GET /?token={} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
        percent_encode(endpoint.token())
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|_| ConnectionError::RelayUnavailable)?;
    let mut response = [0_u8; 96];
    let size = stream
        .read(&mut response)
        .map_err(|_| ConnectionError::RelayUnavailable)?;
    if !response[..size].starts_with(b"HTTP/1.1 200") {
        return Err(ConnectionError::RelayUnavailable);
    }
    Ok(())
}

/// `connected` est atteint uniquement après la réponse HTTP du relais.
pub fn connect_remote<T: RemoteTransport, P: RelayProbe>(
    profile: &crate::profile::ConnectionProfile,
    transport: &mut T,
    probe: &mut P,
) -> Result<ConnectionSession, ConnectionError> {
    let mut session = ConnectionSession::disconnected(profile.id());
    if matches!(
        profile,
        crate::profile::ConnectionProfile::Ssh {
            host_fingerprint: None,
            ..
        }
    ) {
        let _ = transition(&mut session, ConnectionState::AwaitingHostApproval);
        return Err(ConnectionError::HostApprovalRequired);
    }
    let _ = transition(&mut session, ConnectionState::ConnectingSsh);
    let endpoint = transport.discover_endpoint(profile)?;
    let _ = transition(&mut session, ConnectionState::OpeningTunnel);
    let local_port = transport.open_tunnel(profile, endpoint.port)?;
    let _ = transition(&mut session, ConnectionState::CheckingRelay);
    if let Err(error) = probe.check(local_port, &endpoint) {
        transport.close();
        return Err(error);
    }
    session.set_endpoint(endpoint);
    let _ = transition(&mut session, ConnectionState::Connected);
    Ok(session)
}

/// Une perte de tunnel ne devient jamais un état connecté artificiel. Le
/// Desktop conserve l'état `Reconnecting` afin de pouvoir recréer son tunnel
/// sans laisser le panneau attaché à un port local mort.
pub fn begin_tunnel_reconnect(session: &mut ConnectionSession) -> ConnectionStatus {
    if session.state != ConnectionState::Connected {
        return ConnectionStatus::from_session(session);
    }
    let _ = transition(session, ConnectionState::Reconnecting);
    session.retry_count = session.retry_count.saturating_add(1);
    session.last_error = Some(crate::profile::RedactedConnectionError {
        category: crate::profile::ConnectionErrorCategory::Tunnel,
        message: "Le tunnel SSH s'est arrêté. Reconnexion automatique en cours.".into(),
    });
    ConnectionStatus::from_session(session)
}

pub fn parse_endpoint_document(body: &[u8]) -> Result<RelayEndpoint, ConnectionError> {
    let document: EndpointDocument =
        serde_json::from_slice(body).map_err(|_| ConnectionError::EndpointInvalid)?;
    if document.version != ENDPOINT_VERSION {
        return Err(ConnectionError::EndpointVersion);
    }
    RelayEndpoint::new(document.port, document.token).map_err(|_| ConnectionError::EndpointInvalid)
}

/// Découverte locale éphémère : aucun profil ni jeton n'est persisté.
///
/// La commande et ses arguments sont constants afin qu'une WebView ne puisse
/// jamais piloter une exécution locale. L'absence de relais est un état attendu
/// de l'interface, pas une erreur fatale de Desktop.
pub fn discover_local_endpoint() -> Result<RelayEndpoint, ConnectionError> {
    let mut command = Command::new(LOCAL_ENDPOINT_PROGRAM);
    if let Some(path) = local_endpoint_path(
        std::env::var_os("HOME").as_deref().map(Path::new),
        std::env::var_os("PATH"),
    ) {
        command.env("PATH", path);
    }
    let output = command
        .args(LOCAL_ENDPOINT_ARGS)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map_err(|_| ConnectionError::LocalUnavailable)?;
    if !output.status.success() {
        return Err(ConnectionError::LocalUnavailable);
    }
    parse_endpoint_document(&output.stdout)
}

/// Reconstruit le PATH du processus enfant sans dépendre du shell ayant lancé
/// Desktop. Les installations Bridget officielles placent le binaire dans
/// .local/bin, qui n est pas toujours propagé par une application graphique.
/// Le chemin reste local, déterministe et ne vient jamais d une WebView.
fn local_endpoint_path(home: Option<&Path>, inherited_path: Option<OsString>) -> Option<OsString> {
    let mut paths = Vec::new();
    if let Some(home) = home {
        paths.push(home.join(".local/bin"));
    }
    paths.extend(
        inherited_path
            .as_deref()
            .into_iter()
            .flat_map(std::env::split_paths),
    );
    (!paths.is_empty())
        .then(|| std::env::join_paths(paths).ok())
        .flatten()
}

pub fn relay_url(local_port: u16, endpoint: &RelayEndpoint) -> String {
    relay_url_for_client(local_port, endpoint, None)
}

/// Le client Desktop injecte son identité stable au relais. Le port local reste
/// éphémère et n'est jamais utilisé comme identité de préférences.
pub fn relay_url_for_client(
    local_port: u16,
    endpoint: &RelayEndpoint,
    client_id: Option<&str>,
) -> String {
    let client_query = client_id
        .filter(|value| !value.is_empty())
        .map(|value| format!("&client_id={}", percent_encode(value)))
        .unwrap_or_default();
    format!(
        "http://127.0.0.1:{local_port}/?token={}{}",
        percent_encode(endpoint.token()),
        client_query
    )
}

pub fn desktop_relay_url(local_port: u16, endpoint: &RelayEndpoint, client_id: &str) -> String {
    format!(
        "{}&native_attention=1",
        relay_url_for_client(local_port, endpoint, Some(client_id))
    )
}

/// URL de conversation pour un enfant WebView : le jeton reste dans Rust et ne
/// remonte jamais dans un payload IPC de la page Desktop.
pub fn desktop_panel_url(
    local_port: u16,
    endpoint: &RelayEndpoint,
    client_id: &str,
    agent_name: Option<&str>,
    project_id: Option<&str>,
    desktop_action: Option<&str>,
    desktop_source: Option<&str>,
) -> String {
    let mut url = format!(
        "{}&desktop_shell=1",
        desktop_relay_url(local_port, endpoint, client_id)
    );
    if let Some(agent_name) = agent_name.filter(|value| !value.is_empty()) {
        url.push_str("&agent=");
        url.push_str(&percent_encode(agent_name));
    }
    if let Some(project_id) = project_id.filter(|value| !value.is_empty()) {
        url.push_str("&project_id=");
        url.push_str(&percent_encode(project_id));
    }
    if let Some(action) = desktop_action {
        let action = match action {
            "create_project" | "import_project" => action,
            "settings" => {
                url.push_str("&view=settings");
                return url;
            }
            "usage" => {
                url.push_str("&view=usage");
                return url;
            }
            _ => return url,
        };
        url.push_str("&desktop_action=");
        url.push_str(action);
    }
    if let Some(source) = desktop_source.filter(|value| !value.is_empty()) {
        url.push_str("&desktop_source=");
        url.push_str(&percent_encode(source));
    }
    url
}

pub fn fleet_snapshot_path(endpoint: &RelayEndpoint) -> String {
    format!("/v1/snapshot?token={}", percent_encode(endpoint.token()))
}

pub fn fleet_projects_path(endpoint: &RelayEndpoint) -> String {
    format!("/v1/projects?token={}", percent_encode(endpoint.token()))
}

pub fn attention_request_path(endpoint: &RelayEndpoint, client_id: &str) -> String {
    format!(
        "/v1/attention?token={}&client_id={}",
        percent_encode(endpoint.token()),
        percent_encode(client_id)
    )
}

pub fn attention_state_request_path(endpoint: &RelayEndpoint) -> String {
    format!(
        "/v1/attention/state?token={}",
        percent_encode(endpoint.token())
    )
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectionError, HttpRelayProbe, LOCAL_ENDPOINT_ARGS, LOCAL_ENDPOINT_PROGRAM, RelayProbe,
        RemoteTransport, attention_request_path, attention_state_request_path,
        begin_tunnel_reconnect, connect_remote, desktop_panel_url, desktop_relay_url,
        fleet_projects_path, fleet_snapshot_path, local_endpoint_path, parse_endpoint_document,
        transition,
    };
    use crate::profile::{
        ConnectionProfile, ConnectionSession, ConnectionState, ProfileCapability, RelayEndpoint,
        SshIdentityRef,
    };
    use std::cell::Cell;
    use std::ffi::OsString;
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, TcpListener};
    use std::thread;
    use std::time::Duration;

    fn ssh_profile(approved: bool) -> ConnectionProfile {
        ConnectionProfile::Ssh {
            id: "remote".into(),
            label: "Remote".into(),
            host: "cartae.app".into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::Agent,
            host_fingerprint: approved.then(|| "SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
            capabilities: vec![ProfileCapability::Ui],
        }
    }
    struct FakeTransport {
        endpoint: RelayEndpoint,
        opened: Cell<bool>,
        closed: Cell<bool>,
    }
    impl RemoteTransport for FakeTransport {
        fn discover_endpoint(
            &mut self,
            _: &ConnectionProfile,
        ) -> Result<RelayEndpoint, ConnectionError> {
            Ok(self.endpoint.clone())
        }
        fn open_tunnel(&mut self, _: &ConnectionProfile, _: u16) -> Result<u16, ConnectionError> {
            self.opened.set(true);
            Ok(39001)
        }
        fn close(&mut self) {
            self.closed.set(true);
        }
    }
    struct FakeProbe {
        result: Result<(), ConnectionError>,
    }
    impl RelayProbe for FakeProbe {
        fn check(&mut self, _: u16, _: &RelayEndpoint) -> Result<(), ConnectionError> {
            match &self.result {
                Ok(()) => Ok(()),
                Err(_) => Err(ConnectionError::RelayUnavailable),
            }
        }
    }
    fn transport() -> FakeTransport {
        FakeTransport {
            endpoint: RelayEndpoint::new(17888, "fixture-token".into()).unwrap(),
            opened: Cell::new(false),
            closed: Cell::new(false),
        }
    }

    #[test]
    fn endpoint_rejette_version_champ_inconnu_port_et_erreur_sans_jeton() {
        assert!(matches!(
            parse_endpoint_document(br#"{"version":2,"port":17888,"token":"fixture-token"}"#),
            Err(ConnectionError::EndpointVersion)
        ));
        assert!(matches!(
            parse_endpoint_document(br#"{"version":1,"port":0,"token":"fixture-token"}"#),
            Err(ConnectionError::EndpointInvalid)
        ));
        assert!(matches!(
            parse_endpoint_document(
                br#"{"version":1,"port":17888,"token":"fixture-token","other":true}"#
            ),
            Err(ConnectionError::EndpointInvalid)
        ));
        assert!(
            !ConnectionError::EndpointInvalid
                .to_string()
                .contains("fixture-token")
        );
    }

    #[test]
    fn ssh_vivant_ne_devient_pas_connected_sans_reponse_relais() {
        let mut transport = transport();
        let mut probe = FakeProbe {
            result: Err(ConnectionError::RelayUnavailable),
        };
        assert!(matches!(
            connect_remote(&ssh_profile(true), &mut transport, &mut probe),
            Err(ConnectionError::RelayUnavailable)
        ));
        assert!(transport.opened.get());
        assert!(transport.closed.get());
    }

    #[test]
    fn sonde_attend_un_relais_qui_devient_disponible() {
        let reservation = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("port de test");
        let port = reservation.local_addr().expect("adresse").port();
        drop(reservation);
        let server = thread::spawn(move || {
            thread::sleep(Duration::from_millis(75));
            let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).expect("relais tardif");
            let (mut stream, _) = listener.accept().expect("connexion sonde");
            let mut request = [0_u8; 512];
            let _ = stream.read(&mut request).expect("requête");
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("réponse");
        });
        let endpoint = RelayEndpoint::new(17888, "fixture-token".into()).expect("endpoint");
        let mut probe = HttpRelayProbe;
        assert!(probe.check(port, &endpoint).is_ok());
        server.join().expect("serveur terminé");
    }

    #[test]
    fn url_desktop_garde_le_client_stable_quand_le_port_change() {
        let endpoint = RelayEndpoint::new(17888, "fixture token".into()).expect("endpoint");
        let client_id = "11111111-1111-4111-8111-111111111111";
        let first = desktop_relay_url(39001, &endpoint, client_id);
        let second = desktop_relay_url(39002, &endpoint, client_id);
        assert!(first.contains("127.0.0.1:39001"));
        assert!(second.contains("127.0.0.1:39002"));
        assert!(first.contains(client_id));
        assert!(second.contains(client_id));
        assert!(first.ends_with("native_attention=1"));
        assert_eq!(
            attention_request_path(&endpoint, client_id),
            format!("/v1/attention?token=fixture%20token&client_id={client_id}")
        );
        assert_eq!(
            attention_state_request_path(&endpoint),
            "/v1/attention/state?token=fixture%20token"
        );
        assert_eq!(LOCAL_ENDPOINT_PROGRAM, "bridget");
        assert_eq!(LOCAL_ENDPOINT_ARGS, ["ui", "endpoint", "--json"]);
        let panel_url = desktop_panel_url(
            39002,
            &endpoint,
            client_id,
            Some("coordinateur / projet"),
            Some("projet bleu"),
            Some("create_project"),
            Some("Serveur de test"),
        );
        assert!(panel_url.contains("desktop_shell=1"));
        assert!(panel_url.contains("agent=coordinateur%20%2F%20projet"));
        assert!(panel_url.contains("project_id=projet%20bleu"));
        assert!(panel_url.contains("desktop_action=create_project"));
        assert!(panel_url.contains("desktop_source=Serveur%20de%20test"));
        let settings_url = desktop_panel_url(
            39002,
            &endpoint,
            client_id,
            None,
            None,
            Some("settings"),
            None,
        );
        assert!(settings_url.contains("desktop_shell=1"));
        assert!(settings_url.contains("view=settings"));
        assert!(!settings_url.contains("desktop_action="));
        let usage_url =
            desktop_panel_url(39002, &endpoint, client_id, None, None, Some("usage"), None);
        assert!(usage_url.contains("view=usage"));
        assert_eq!(
            fleet_snapshot_path(&endpoint),
            "/v1/snapshot?token=fixture%20token"
        );
        assert_eq!(
            fleet_projects_path(&endpoint),
            "/v1/projects?token=fixture%20token"
        );
    }

    #[test]
    fn decouverte_locale_retablit_le_repertoire_d_installation_utilisateur() {
        let path = local_endpoint_path(
            Some(std::path::Path::new("/home/fixture")),
            Some(OsString::from("/usr/local/bin:/usr/bin")),
        )
        .expect("PATH local");
        let paths = std::env::split_paths(&path).collect::<Vec<_>>();
        assert_eq!(
            paths[0],
            std::path::PathBuf::from("/home/fixture/.local/bin")
        );
        assert_eq!(paths[1], std::path::PathBuf::from("/usr/local/bin"));
        assert_eq!(paths[2], std::path::PathBuf::from("/usr/bin"));
    }

    #[test]
    fn hote_non_approuve_n_est_pas_contacte() {
        let mut transport = transport();
        let mut probe = FakeProbe { result: Ok(()) };
        assert!(matches!(
            connect_remote(&ssh_profile(false), &mut transport, &mut probe),
            Err(ConnectionError::HostApprovalRequired)
        ));
        assert!(!transport.opened.get());
    }

    #[test]
    fn machine_d_etats_refuse_connected_sans_relais_et_toute_transition_apres_fermeture() {
        let mut session = crate::profile::ConnectionSession::disconnected("remote");
        assert!(transition(&mut session, ConnectionState::Connected).is_err());
        transition(&mut session, ConnectionState::ConnectingSsh).unwrap();
        transition(&mut session, ConnectionState::OpeningTunnel).unwrap();
        transition(&mut session, ConnectionState::CheckingRelay).unwrap();
        assert!(transition(&mut session, ConnectionState::Connected).is_err());
        transition(&mut session, ConnectionState::Closed).unwrap();
        assert!(transition(&mut session, ConnectionState::Reconnecting).is_err());
    }

    #[test]
    fn perte_de_tunnel_garde_un_etat_de_reconnexion_automatique() {
        let mut session = ConnectionSession::disconnected("remote");
        session.set_endpoint(RelayEndpoint::new(17888, "fixture-token".into()).unwrap());
        transition(&mut session, ConnectionState::ConnectingSsh).unwrap();
        transition(&mut session, ConnectionState::OpeningTunnel).unwrap();
        transition(&mut session, ConnectionState::CheckingRelay).unwrap();
        transition(&mut session, ConnectionState::Connected).unwrap();
        let status = begin_tunnel_reconnect(&mut session);
        assert_eq!(status.state, ConnectionState::Reconnecting);
        assert_eq!(
            status.category,
            Some(crate::profile::ConnectionErrorCategory::Tunnel)
        );
        assert_eq!(session.retry_count, 1);
        assert_eq!(
            begin_tunnel_reconnect(&mut session).state,
            ConnectionState::Reconnecting
        );
        assert_eq!(session.retry_count, 1);
    }
}
