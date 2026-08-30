//! Connexions Bridget Desktop : endpoint versionné, tunnel et relais prouvé.

use crate::profile::{ConnectionErrorCategory, ConnectionSession, ConnectionState, RelayEndpoint};
use crate::ssh::{OwnedTunnel, discovery_invocation};
use serde::Deserialize;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

const ENDPOINT_VERSION: u8 = 1;
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

/// Une perte de tunnel ne devient jamais un état connecté artificiel. La
/// transition `Reconnecting` est volontairement unique : l'interface propose
/// ensuite une relance explicite au lieu de boucler silencieusement.
pub fn mark_tunnel_lost(session: &mut ConnectionSession) -> ConnectionStatus {
    if session.state != ConnectionState::Connected {
        return ConnectionStatus::from_session(session);
    }
    let _ = transition(session, ConnectionState::Reconnecting);
    session.retry_count = session.retry_count.saturating_add(1);
    session.last_error = Some(crate::profile::RedactedConnectionError {
        category: crate::profile::ConnectionErrorCategory::Tunnel,
        message: "Le tunnel SSH s'est arrêté. Relancez explicitement ce serveur.".into(),
    });
    let _ = transition(session, ConnectionState::Failed);
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

pub fn relay_url(local_port: u16, endpoint: &RelayEndpoint) -> String {
    format!(
        "http://127.0.0.1:{local_port}/?token={}",
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
        ConnectionError, HttpRelayProbe, RelayProbe, RemoteTransport, connect_remote,
        mark_tunnel_lost, parse_endpoint_document, transition,
    };
    use crate::profile::{
        ConnectionProfile, ConnectionSession, ConnectionState, ProfileCapability, RelayEndpoint,
        SshIdentityRef,
    };
    use std::cell::Cell;
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
    fn perte_de_tunnel_declenche_une_seule_reconnexion_puis_un_echec_honnete() {
        let mut session = ConnectionSession::disconnected("remote");
        session.set_endpoint(RelayEndpoint::new(17888, "fixture-token".into()).unwrap());
        transition(&mut session, ConnectionState::ConnectingSsh).unwrap();
        transition(&mut session, ConnectionState::OpeningTunnel).unwrap();
        transition(&mut session, ConnectionState::CheckingRelay).unwrap();
        transition(&mut session, ConnectionState::Connected).unwrap();
        let status = mark_tunnel_lost(&mut session);
        assert_eq!(status.state, ConnectionState::Failed);
        assert_eq!(
            status.category,
            Some(crate::profile::ConnectionErrorCategory::Tunnel)
        );
        assert_eq!(session.retry_count, 1);
        assert_eq!(
            mark_tunnel_lost(&mut session).state,
            ConnectionState::Failed
        );
        assert_eq!(session.retry_count, 1);
    }
}
