//! Client Unix partagé : inscription auxiliaire commune et budget borné par opération.
//! Aucune logique de présentation ni accès client au stockage du daemon.

#[cfg(test)]
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, CONTROL_STATE_CONTRACT_VERSION, ClientCapability, ConnectionRole,
    ControlEventFrame, ControlStateFrame, RuntimeSource, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{self, BufReader, Write};
use std::os::fd::AsRawFd;
#[cfg(test)]
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

#[path = "../lineage_client.rs"]
mod lineage;
pub(crate) use lineage::human_lineage;

pub(crate) const DAEMON_BUDGET: Duration = Duration::from_secs(10);

/// Lecture locale publique du catalogue168. Cette connexion ne porte aucune
/// identité d'agent : le daemon vérifie le rôle Client, la négociation et le
/// UID Unix observé avant toute projection.
pub(crate) fn routing_catalogue(socket: &Path) -> Result<serde_json::Value, ClientError> {
    routing_catalogue_observed(socket, false, false, None)
}

pub(crate) fn routing_catalogue_observed(
    socket: &Path,
    observed: bool,
    refresh: bool,
    machine: Option<String>,
) -> Result<serde_json::Value, ClientError> {
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        _ => {
            return Err(ClientError::Technical {
                code: "capability_unavailable",
                message: "le daemon refuse le rôle client du catalogue".into(),
            });
        }
    }
    let capabilities = match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: crate::communication::issuer_scope("routing-catalogue"),
        capabilities: if observed {
            vec![
                ClientCapability::RoutingCatalogueV1,
                ClientCapability::ProviderCatalogueV1,
            ]
        } else {
            vec![ClientCapability::RoutingCatalogueV1]
        },
    })? {
        DaemonToWrapper::ClientWelcome {
            version,
            capabilities,
            ..
        } if version == CLIENT_CONTRACT_VERSION => capabilities,
        _ => Vec::new(),
    };
    if !capabilities.contains(&ClientCapability::RoutingCatalogueV1)
        || observed && !capabilities.contains(&ClientCapability::ProviderCatalogueV1)
    {
        return Err(ClientError::Technical {
            code: "capability_unavailable",
            message: "le daemon ne négocie pas routing_catalogue_v1".into(),
        });
    }
    match connection.exchange(&WrapperToDaemon::NativeDelegation {
        request: if observed {
            bridget_transport::protocol::NativeDelegationRequest::ProviderCatalogueV1 {
                refresh,
                machine_id: machine,
            }
        } else {
            bridget_transport::protocol::NativeDelegationRequest::RoutingCatalogueV1 {}
        },
    })? {
        DaemonToWrapper::NativeDelegationResult { result } => Ok(result),
        other => unexpected_response(other),
    }
}

/// Sonde sans identité auxiliaire : budget client commun, avant toute trame151.
/// Le premier octet du relais SSH observé arrive parfois après3secondes.
pub(crate) fn daemon_supports_wait(socket: &Path) -> Result<bool, ClientError> {
    let mut connection = DaemonConnection::connect_until(socket, Instant::now() + DAEMON_BUDGET)?;
    match connection.exchange(&WrapperToDaemon::DaemonIdentityRequest)? {
        DaemonToWrapper::DaemonIdentityReport { features, .. } => {
            Ok(features.iter().any(|feature| feature == "wait_v1"))
        }
        other => unexpected_response(other),
    }
}

pub(crate) fn deserialize_project_reason<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let reason = <String as serde::Deserialize>::deserialize(deserializer)?;
    crate::communication::validate_cross_project_reason(Some(&reason))
        .map_err(serde::de::Error::custom)
}

/// Session 138 : négociation obligatoire avant un champ ou une portée nouvelle.
/// Aucun repli vers l'annuaire global ni motif ignoré par un ancien serveur.
pub(crate) fn negotiate_projects(
    connection: &mut DaemonConnection,
    issuer_scope: &str,
) -> Result<(), ClientError> {
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: issuer_scope.into(),
        capabilities: vec![ClientCapability::CommunicationProjectsV1],
    })? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::CommunicationProjectsV1) =>
        {
            Ok(())
        }
        _ => Err(ClientError::Technical {
            code: "communication_projects_unsupported",
            message: "le daemon ne négocie pas la portée projet ; aucun envoi ni repli global"
                .into(),
        }),
    }
}

/// Contexte du client de fond, sans fabriquer un agent et sans emprunter T3.
pub(crate) fn announce_client_project(
    connection: &mut DaemonConnection,
    root: &Path,
) -> Result<(), ClientError> {
    match connection.exchange(&WrapperToDaemon::CommunicationProjectFact {
        root: root.to_string_lossy().into_owned(),
        source: bridget_transport::protocol::CommunicationProjectSource::Git,
        host: crate::wrapper::host_name(),
        worktree_root: None,
    })? {
        DaemonToWrapper::ProjectContextResult { .. } => Ok(()),
        other => unexpected_response(other),
    }
}

/// Annuaire de communication : l'auxiliaire hérite du parent ; seul un client
/// autonome fournit son propre contexte. ListAgents de diagnostic reste séparé.
pub(crate) fn directory_request(
    identity: Option<(&str, &str)>,
    socket: &Path,
    scope: bridget_transport::protocol::CommunicationDirectoryScope,
    project_root: Option<&Path>,
) -> Result<DaemonToWrapper, ClientError> {
    let mut connection = DaemonConnection::connect(socket)?;
    let issuer = match identity {
        Some((identity, instance)) => {
            authenticate_auxiliary(&mut connection, identity, instance, socket)?;
            crate::communication::issuer_scope(instance)
        }
        None => {
            match connection.exchange(&WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            })? {
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                } => {}
                other => return unexpected_response(other),
            }
            crate::communication::issuer_scope("public-communication-directory")
        }
    };
    negotiate_projects(&mut connection, &issuer)?;
    if identity.is_none()
        && let Some(root) = project_root
    {
        announce_client_project(&mut connection, root)?;
    }
    match connection.exchange(&WrapperToDaemon::DirectoryScoped { scope })? {
        result @ DaemonToWrapper::CommunicationDirectory { .. } => Ok(result),
        other => unexpected_response(other),
    }
}

pub(crate) fn observation_request(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::ObservationRequest,
) -> Result<serde_json::Value, ClientError> {
    if let bridget_transport::protocol::ObservationRequest::Missed { limit, after_seq } = &request
        && (limit.is_some_and(|value| !(1..=64).contains(&value))
            || after_seq.is_some_and(|value| value > i64::MAX as u64))
    {
        return Err(ClientError::InvalidParams(
            "missed : limit 1..64 et curseur de séquence borné requis".into(),
        ));
    }
    let mut connection = registered_connection(identity, instance_id, socket)?;
    match connection.send_then_wait(&WrapperToDaemon::ObservationRequest { request })? {
        DaemonToWrapper::ObservationResult { result } => Ok(result),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "observation_rejected",
            message: reason,
        }),
        other => unexpected_response(other),
    }
}

/// Session 102 : opération de fil, même connexion attestée que les observations.
pub(crate) fn thread_request(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::ThreadRequest,
) -> Result<bridget_transport::protocol::ThreadResult, ClientError> {
    let mut connection = registered_connection(identity, instance_id, socket)?;
    if matches!(
        request.request,
        bridget_transport::protocol::ThreadAction::AddMembers { .. }
    ) {
        let mut required = vec![ClientCapability::ThreadMembersV1];
        if request.cross_project_reason.is_some() {
            required.push(ClientCapability::CommunicationProjectsV1);
        }
        let accepted = match connection.exchange(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope(instance_id),
            capabilities: required.clone(),
        })? {
            DaemonToWrapper::ClientWelcome { capabilities, .. } => {
                required.iter().all(|cap| capabilities.contains(cap))
            }
            _ => false,
        };
        if !accepted {
            return Err(ClientError::Technical {
                code: "thread_members_unsupported",
                message: "le daemon ne négocie pas l'ajout de membres ; aucune mutation ni repli"
                    .into(),
            });
        }
    } else if request.cross_project_reason.is_some() {
        negotiate_projects(
            &mut connection,
            &crate::communication::issuer_scope(instance_id),
        )?;
    }
    match connection.send_then_wait(&WrapperToDaemon::ThreadRequest { request })? {
        DaemonToWrapper::ThreadResult { result } => Ok(result),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "thread_rejected",
            message: reason,
        }),
        other => unexpected_response(other),
    }
}

/// Lecture humaine : connexion Client nue, sans autostart ni credential.
pub(crate) fn human_thread_view(
    socket: &Path,
    request: bridget_transport::protocol::HumanThreadViewV1,
) -> Result<bridget_transport::protocol::HumanThreadViewResult, ClientError> {
    use bridget_transport::protocol::{
        HUMAN_THREAD_VIEW_VERSION, HumanThreadViewError as E, HumanThreadViewResult as R,
    };
    let capability = request.request.required_capability();
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        _ => return Ok(R::error(E::UnsupportedVersion)),
    }
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: crate::communication::issuer_scope("human-thread-view"),
        capabilities: vec![capability],
    })? {
        DaemonToWrapper::ClientWelcome {
            version,
            capabilities,
            ..
        } if version == CLIENT_CONTRACT_VERSION && capabilities.as_slice() == [capability] => {}
        _ => return Ok(R::error(E::UnsupportedVersion)),
    }
    match connection.exchange(&WrapperToDaemon::HumanThreadViewV1 { request })? {
        DaemonToWrapper::HumanThreadViewResult { result }
            if result.version == HUMAN_THREAD_VIEW_VERSION =>
        {
            Ok(result)
        }
        DaemonToWrapper::HumanThreadViewResult { .. } => Ok(R::error(E::UnsupportedVersion)),
        other => unexpected_response(other),
    }
}

/// Le handshake est borné à6s. Après ready, seul un fragment en cours a un
/// délai ; l'attente sans données reste ouverte et annulable sans polling métier.
pub(crate) fn human_thread_watch(
    socket: &Path,
    request: bridget_transport::protocol::HumanThreadWatchV1,
    mut output: impl FnMut(&bridget_transport::protocol::HumanThreadWatchEvent) -> io::Result<()>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<(), ClientError> {
    use bridget_transport::protocol::{
        HUMAN_THREAD_WATCH_MAX_BYTES, HumanThreadViewError as R, HumanThreadWatchEvent as E,
    };
    let mut connection =
        DaemonConnection::connect_until(socket, Instant::now() + Duration::from_secs(6))?;
    connection.max_response_bytes = HUMAN_THREAD_WATCH_MAX_BYTES;
    let handshake = (|| {
        if !matches!(
            connection.exchange(&WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            })?,
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            }
        ) {
            return Ok(false);
        }
        Ok(matches!(connection.exchange(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("human-thread-watch"),
            capabilities: vec![ClientCapability::HumanThreadWatchV1],
        })?, DaemonToWrapper::ClientWelcome {version, capabilities,..}
            if version == CLIENT_CONTRACT_VERSION && capabilities.as_slice() == [ClientCapability::HumanThreadWatchV1]))
    })();
    match handshake {
        Ok(true) => {}
        Ok(false) => {
            output(&E::error(R::UnsupportedVersion)).map_err(watch_output_error)?;
            return Ok(());
        }
        Err(error) => return Err(error),
    }
    let first = match connection.exchange(&WrapperToDaemon::HumanThreadWatchV1 { request })? {
        DaemonToWrapper::HumanThreadWatchEvent { event } => event,
        _ => {
            output(&E::error(R::UnsupportedVersion)).map_err(watch_output_error)?;
            return Ok(());
        }
    };
    let generation = match &first {
        E::Ready { generation, .. } => generation.clone(),
        E::Error { .. } => {
            output(&first).map_err(watch_output_error)?;
            return Ok(());
        }
        _ => return Err(watch_protocol_error()),
    };
    output(&first).map_err(watch_output_error)?;
    connection
        .reader
        .get_ref()
        .set_read_timeout(None)
        .map_err(watch_output_error)?;
    let mut seq = 0;
    loop {
        if cancelled() {
            connection.poison();
            return Ok(());
        }
        let mut poll = libc::pollfd {
            fd: connection.reader.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        if connection.reader.buffer().is_empty() {
            // SAFETY: `poll` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            let ready = unsafe { libc::poll(&mut poll, 1, 100) };
            if ready == 0 {
                continue;
            }
            if ready < 0 {
                if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(watch_protocol_error());
            }
        }
        let frame = bridget_transport::jsonl::read_unix_line(
            &mut connection.reader,
            HUMAN_THREAD_WATCH_MAX_BYTES,
            bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(6)),
        )
        .map_err(|_| watch_protocol_error())?;
        let Some(frame) = frame else {
            return Err(watch_protocol_error());
        };
        let response: DaemonToWrapper =
            serde_json::from_slice(&frame).map_err(|_| watch_protocol_error())?;
        let DaemonToWrapper::HumanThreadWatchEvent { event } = response else {
            return Err(watch_protocol_error());
        };
        match &event {
            E::Changed {
                generation: current,
                seq: next,
                ..
            }
            | E::Resync {
                generation: current,
                seq: next,
                ..
            } if current == &generation && *next > seq => seq = *next,
            E::Error { .. } => {
                output(&event).map_err(watch_output_error)?;
                return Ok(());
            }
            _ => return Err(watch_protocol_error()),
        }
        output(&event).map_err(watch_output_error)?;
    }
}

fn watch_protocol_error() -> ClientError {
    ClientError::Technical {
        code: "daemon_protocol",
        message: "flux de suivi interrompu ou invalide".into(),
    }
}

fn watch_output_error(_: io::Error) -> ClientError {
    ClientError::Technical {
        code: "watch_output_closed",
        message: "sortie de suivi fermée".into(),
    }
}

/// Session 104 : recherche bornée ; un daemon antérieur répond par une
/// erreur de protocole explicite, jamais par un repli vers le ledger global.
pub(crate) fn ledger_search(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::LedgerSearchRequest,
) -> Result<bridget_transport::protocol::LedgerSearchOutcomeV1, ClientError> {
    let mut connection = registered_connection(identity, instance_id, socket)?;
    match connection.send_then_wait(&WrapperToDaemon::LedgerSearch { request })? {
        DaemonToWrapper::LedgerSearchResult { outcome } => Ok(outcome),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "daemon_protocol",
            message: reason,
        }),
        other => unexpected_response(other),
    }
}

/// Session 104 : relecture exacte d'un message par `(id, target)`.
pub(crate) fn ledger_read(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::LedgerReadRequest,
) -> Result<bridget_transport::protocol::LedgerReadOutcomeV1, ClientError> {
    let mut connection = registered_connection(identity, instance_id, socket)?;
    match connection.send_then_wait(&WrapperToDaemon::LedgerRead { request })? {
        DaemonToWrapper::LedgerReadResult { outcome } => Ok(outcome),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "daemon_protocol",
            message: reason,
        }),
        other => unexpected_response(other),
    }
}

pub(crate) fn cancel_request(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    id: &str,
    reason: Option<String>,
) -> Result<DaemonToWrapper, ClientError> {
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let response = connection.send_then_wait(&WrapperToDaemon::CancelRequest {
        id: id.into(),
        sender: identity.into(),
        reason,
    })?;
    match &response {
        DaemonToWrapper::RequestCancelled { id: actual, .. }
        | DaemonToWrapper::Nack { id: actual, .. }
            if actual == id =>
        {
            Ok(response)
        }
        _ => unexpected_response(response),
    }
}

/// Borne mémoire du client, délimiteur LF inclus ; distincte de la borne
/// guichet (64 Kio) et des fragments attach (256 Kio). Une réponse plus grande
/// est une erreur explicite, jamais tronquée ni partiellement présentée.
pub(crate) use bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES as MAX_DAEMON_RESPONSE_BYTES;

pub(crate) fn rename_display_name(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    name: &str,
) -> Result<DaemonToWrapper, ClientError> {
    let expected_name = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let response = connection.send_then_wait(&WrapperToDaemon::DisplayNameSet {
        request: bridget_transport::protocol::DisplayNameRequest {
            version: 1,
            display_name: name.into(),
        },
    })?;
    match &response {
        DaemonToWrapper::DisplayNameResult {
            outcome:
                bridget_transport::protocol::DisplayNameOutcome::Applied {
                    agent_id,
                    display_name,
                    ..
                },
        } if agent_id == identity && display_name == &expected_name => Ok(response),
        DaemonToWrapper::DisplayNameResult {
            outcome: bridget_transport::protocol::DisplayNameOutcome::Rejected { .. },
        } => Ok(response),
        _ => unexpected_response(response),
    }
}

pub(crate) const DND_DEFAULT_SECS: u64 = 60 * 60;
pub(crate) const DND_MAX_SECS: u64 = 7 * 24 * 60 * 60;

pub(crate) fn parse_dnd_duration_secs(value: &str) -> Result<u64, ClientError> {
    let value = value.trim();
    let (digits, multiplier) = match value.chars().last() {
        Some('s') => (&value[..value.len() - 1], 1_u64),
        Some('m') => (&value[..value.len() - 1], 60),
        Some('h') => (&value[..value.len() - 1], 60 * 60),
        Some(last) if last.is_ascii_digit() => (value, 60),
        _ => {
            return Err(ClientError::InvalidParams(
                "durée attendue sous la forme 90s, 30m ou 2h".to_string(),
            ));
        }
    };
    let amount = digits.parse::<u64>().map_err(|_| {
        ClientError::InvalidParams("durée attendue sous la forme 90s, 30m ou 2h".to_string())
    })?;
    let seconds = amount
        .checked_mul(multiplier)
        .ok_or_else(|| ClientError::InvalidParams("durée supérieure à 7 jours".to_string()))?;
    if !(1..=DND_MAX_SECS).contains(&seconds) {
        return Err(ClientError::InvalidParams(
            "durée comprise entre 1 seconde et 7 jours requise".to_string(),
        ));
    }
    Ok(seconds)
}

fn self_mutation(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    command: WrapperToDaemon,
    expected_id: &str,
) -> Result<DaemonToWrapper, ClientError> {
    self_mutation_until(
        identity,
        instance_id,
        socket,
        command,
        expected_id,
        Instant::now() + DAEMON_BUDGET,
    )
}

fn self_mutation_until(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    command: WrapperToDaemon,
    expected_id: &str,
    deadline: Instant,
) -> Result<DaemonToWrapper, ClientError> {
    let mut connection = registered_connection_until(identity, instance_id, socket, deadline)?;
    let response = connection.send_then_wait(&command)?;
    match &response {
        DaemonToWrapper::Ack { id, .. } | DaemonToWrapper::Nack { id, .. } if id == expected_id => {
            Ok(response)
        }
        _ => unexpected_response(response),
    }
}

pub(crate) fn set_dnd(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    duration_secs: Option<u64>,
) -> Result<DaemonToWrapper, ClientError> {
    let until_secs = duration_secs
        .map(|seconds| {
            if !(1..=DND_MAX_SECS).contains(&seconds) {
                return Err(ClientError::InvalidParams(
                    "durée comprise entre 1 seconde et 7 jours requise".to_string(),
                ));
            }
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| ClientError::Technical {
                    code: "clock_unavailable",
                    message: "horloge système antérieure à l'époque Unix".to_string(),
                })?
                .as_secs();
            now.checked_add(seconds)
                .ok_or_else(|| ClientError::InvalidParams("échéance DND hors plage".to_string()))
        })
        .transpose()?;
    self_mutation(
        identity,
        instance_id,
        socket,
        WrapperToDaemon::Availability {
            agent: identity.to_string(),
            until_secs,
        },
        "availability",
    )
}

fn domain_state_path(socket: &Path, identity: &str) -> Result<std::path::PathBuf, ClientError> {
    let root = socket.parent().ok_or_else(|| ClientError::Technical {
        code: "domain_persistence_failed",
        message: "domaine appliqué en mémoire, mais racine de persistance absente".to_string(),
    })?;
    Ok(root.join("agent-domains").join(identity))
}

pub(crate) struct DomainLock {
    _file: std::fs::File,
    identity: String,
    state_path: std::path::PathBuf,
    deadline: Instant,
}

#[cfg(test)]
type DomainLockContentionObserver = std::sync::Arc<dyn Fn(&str) + Send + Sync>;

#[cfg(test)]
static DOMAIN_LOCK_CONTENTION_OBSERVER: std::sync::Mutex<Option<DomainLockContentionObserver>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
static DOMAIN_TEST_OBSERVER_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) fn lock_domain_test_observers() -> std::sync::MutexGuard<'static, ()> {
    DOMAIN_TEST_OBSERVER_MUTEX.lock().recover_poison()
}

#[cfg(test)]
pub(crate) fn replace_domain_lock_contention_observer(
    observer: Option<DomainLockContentionObserver>,
) {
    *DOMAIN_LOCK_CONTENTION_OBSERVER.lock().recover_poison() = observer;
}

fn acquire_domain_lock_until(
    socket: &Path,
    identity: &str,
    deadline: Instant,
) -> Result<DomainLock, ClientError> {
    let state_path = domain_state_path(socket, identity)?;
    let lock_path = state_path
        .parent()
        .ok_or_else(|| ClientError::Technical {
            code: "domain_lock_failed",
            message: "répertoire de verrou domaine absent".to_string(),
        })?
        .join(format!(".{identity}.lock"));
    let file = bridget_transport::fsutil::open_private_file(&lock_path).map_err(|error| {
        ClientError::Technical {
            code: "domain_lock_failed",
            message: format!("verrou domaine indisponible : {error}"),
        }
    })?;
    loop {
        // SAFETY: les pointeurs passés à flock désignent des variables locales valides pendant l'appel.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return Ok(DomainLock {
                _file: file,
                identity: identity.to_string(),
                state_path,
                deadline,
            });
        }
        let error = io::Error::last_os_error();
        if !matches!(
            error.raw_os_error(),
            Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN
        ) {
            return Err(ClientError::Technical {
                code: "domain_lock_failed",
                message: format!("verrou domaine impossible : {error}"),
            });
        }
        #[cfg(test)]
        if let Some(observer) = DOMAIN_LOCK_CONTENTION_OBSERVER.lock().unwrap().clone() {
            observer(identity);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| ClientError::Technical {
                code: "domain_lock_timeout",
                message: "verrou domaine indisponible avant la fin du budget".to_string(),
            })?;
        std::thread::sleep(remaining.min(Duration::from_millis(10)));
    }
}

pub(crate) fn acquire_domain_lock(
    socket: &Path,
    identity: &str,
    budget: Duration,
) -> Result<DomainLock, ClientError> {
    acquire_domain_lock_until(socket, identity, Instant::now() + budget)
}

pub(crate) fn read_domain_override(lock: &DomainLock) -> Result<Option<String>, ClientError> {
    match std::fs::read_to_string(&lock.state_path) {
        Ok(domain) => {
            let domain = domain.trim().to_string();
            bridget_core::router::validate_technical_label(&domain)
                .map_err(ClientError::InvalidParams)?;
            Ok(Some(domain))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ClientError::Technical {
            code: "domain_persistence_failed",
            message: format!("lecture du domaine persistant impossible : {error}"),
        }),
    }
}

fn persist_domain(socket: &Path, identity: &str, domain: Option<&str>) -> Result<(), ClientError> {
    let path = domain_state_path(socket, identity)?;
    let result = match domain {
        Some(domain) => {
            bridget_transport::fsutil::write_private_file_atomic(&path, domain.as_bytes())
        }
        None => match std::fs::remove_file(&path) {
            Ok(()) => path
                .parent()
                .map(std::fs::File::open)
                .transpose()
                .and_then(|directory| directory.map_or(Ok(()), |file| file.sync_all())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        },
    };
    result.map_err(|error| ClientError::Technical {
        code: "domain_persistence_failed",
        message: format!("domaine appliqué en mémoire, mais persistance non confirmée : {error}"),
    })
}

#[cfg(test)]
type DomainAfterAckObserver = std::sync::Arc<dyn Fn(&str) + Send + Sync>;

#[cfg(test)]
static DOMAIN_AFTER_ACK_OBSERVER: std::sync::Mutex<Option<DomainAfterAckObserver>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
pub(crate) fn replace_domain_after_ack_observer(observer: Option<DomainAfterAckObserver>) {
    *DOMAIN_AFTER_ACK_OBSERVER.lock().recover_poison() = observer;
}

#[cfg(test)]
fn observe_domain_after_ack(identity: &str) {
    let observer = DOMAIN_AFTER_ACK_OBSERVER.lock().unwrap().clone();
    if let Some(observer) = observer {
        observer(identity);
    }
}

pub(crate) fn set_domain(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    domain: Option<String>,
) -> Result<DaemonToWrapper, ClientError> {
    if let Some(domain) = domain.as_deref() {
        bridget_core::router::validate_technical_label(domain)
            .map_err(ClientError::InvalidParams)?;
    }
    let deadline = Instant::now() + DAEMON_BUDGET;
    let lock = acquire_domain_lock_until(socket, identity, deadline)?;
    let response = set_domain_with_lock(&lock, identity, instance_id, socket, domain.clone())?;
    if matches!(&response, DaemonToWrapper::Ack { .. }) {
        #[cfg(test)]
        observe_domain_after_ack(identity);
        persist_domain(socket, identity, domain.as_deref())?;
    }
    Ok(response)
}

pub(crate) fn set_domain_with_lock(
    lock: &DomainLock,
    identity: &str,
    instance_id: &str,
    socket: &Path,
    domain: Option<String>,
) -> Result<DaemonToWrapper, ClientError> {
    if lock.identity != identity {
        return Err(ClientError::InvalidParams(
            "verrou domaine d'une autre identité".to_string(),
        ));
    }
    if let Some(domain) = domain.as_deref() {
        bridget_core::router::validate_technical_label(domain)
            .map_err(ClientError::InvalidParams)?;
    }
    self_mutation_until(
        identity,
        instance_id,
        socket,
        WrapperToDaemon::Domain {
            agent: identity.to_string(),
            domain,
        },
        "domain",
        lock.deadline,
    )
}

pub(crate) fn declare_runtime(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    model: String,
    effort: Option<String>,
) -> Result<DaemonToWrapper, ClientError> {
    for (label, value) in [
        ("model", Some(model.as_str())),
        ("effort", effort.as_deref()),
    ] {
        if let Some(value) = value
            && (value.trim().is_empty()
                || value.chars().count() > 100
                || value.chars().any(bridget_core::is_disallowed_control))
        {
            return Err(ClientError::InvalidParams(format!(
                "{label} runtime invalide"
            )));
        }
    }
    self_mutation(
        identity,
        instance_id,
        socket,
        WrapperToDaemon::Runtime {
            agent: identity.to_string(),
            model,
            effort,
            source: RuntimeSource::Declared,
        },
        "runtime",
    )
}

pub(crate) fn read_control_status(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    history_limit: u32,
) -> Result<(ControlStateFrame, u32, Vec<ControlEventFrame>), ClientError> {
    if history_limit > 50 {
        return Err(ClientError::InvalidParams(
            "history_limit doit être compris entre 0 et 50".to_string(),
        ));
    }
    // La portée Client reste une lecture, mais elle n'invente pas son instance :
    // l'inscription auxiliaire doit d'abord l'attester auprès du même daemon.
    drop(registered_connection(identity, instance_id, socket)?);
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        response => return unexpected_response(response),
    }
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: crate::communication::issuer_scope(instance_id),
        capabilities: vec![ClientCapability::Lookup],
    })? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities == vec![ClientCapability::Lookup] => {}
        response => return unexpected_response(response),
    }
    let (state, inbox_open_count) =
        match connection.exchange(&WrapperToDaemon::ControlStateRead {
            version: CONTROL_STATE_CONTRACT_VERSION,
        })? {
            DaemonToWrapper::ControlState {
                state,
                inbox_open_count,
            } => (state, inbox_open_count),
            DaemonToWrapper::ControlStateRejected { reason } => {
                return Err(ClientError::Technical {
                    code: "control_status_unavailable",
                    message: format!("état de contrôle indisponible : {reason:?}"),
                });
            }
            response => return unexpected_response(response),
        };
    let history = if history_limit == 0 {
        Vec::new()
    } else {
        match connection.exchange(&WrapperToDaemon::ControlHistory {
            version: CONTROL_STATE_CONTRACT_VERSION,
            limit: history_limit,
        })? {
            DaemonToWrapper::ControlHistory { events } => events,
            DaemonToWrapper::ControlStateRejected { reason } => {
                return Err(ClientError::Technical {
                    code: "control_status_unavailable",
                    message: format!("historique de contrôle indisponible : {reason:?}"),
                });
            }
            response => return unexpected_response(response),
        }
    };
    Ok((state, inbox_open_count, history))
}

/// La portée vient de l'inscription auxiliaire attestée, jamais des arguments.
pub(crate) fn read_artifact(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::ArtifactReadRequest,
) -> Result<DaemonToWrapper, ClientError> {
    use bridget_transport::protocol::{
        ARTIFACT_READ_VERSION, MAX_ARTIFACT_READ_BYTES, MAX_ARTIFACT_REF_BYTES,
    };
    if request.version != ARTIFACT_READ_VERSION
        || request.limit == 0
        || request.limit > MAX_ARTIFACT_READ_BYTES
        || request.artifact_ref.is_empty()
        || request.artifact_ref.len() > MAX_ARTIFACT_REF_BYTES
        || request.version_ref.is_empty()
        || request.version_ref.len() > MAX_ARTIFACT_REF_BYTES
    {
        return Err(ClientError::InvalidParams(
            "lecture de contenu : version, références ou borne invalides".into(),
        ));
    }
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let response = connection.exchange(&WrapperToDaemon::ArtifactRead {
        request: request.clone(),
    })?;
    match &response {
        DaemonToWrapper::ArtifactReadResult {
            version,
            artifact_ref,
            version_ref,
            ..
        } if *version == ARTIFACT_READ_VERSION
            && *artifact_ref == request.artifact_ref
            && *version_ref == request.version_ref =>
        {
            Ok(response)
        }
        _ => unexpected_response(response),
    }
}

/// Frontière158B : identité auxiliaire pour MCP ; rôle client pour CLI local.
/// Les actions administratives gardent la négociation de contrôle existante.
/// Coût O(B), B octets sérialisés et lus ; nombre constant d'échanges.
/// Cette borne ne décrit ni la latence réseau ni le temps de réponse du daemon.
pub(crate) fn wake_request(
    identity: Option<(&str, &str)>,
    socket: &Path,
    request: bridget_transport::protocol::WakeRequest,
) -> Result<bridget_transport::protocol::WakeResult, ClientError> {
    use bridget_transport::protocol::{WAKE_CONTRACT_VERSION, WakeAction};
    if request.version != WAKE_CONTRACT_VERSION {
        return Err(ClientError::InvalidParams(
            "version de réveil invalide".into(),
        ));
    }
    if matches!(
        &request.request,
        WakeAction::AdminList {} | WakeAction::AdminCancel { .. }
    ) {
        return Err(ClientError::InvalidParams(
            "action de contrôle réservée au chemin CLI humain".into(),
        ));
    }
    let mut connection = if let Some((agent, instance)) = identity {
        registered_connection(agent, instance, socket)?
    } else if matches!(&request.request, WakeAction::CliOnce { .. }) {
        let mut connection = DaemonConnection::connect(socket)?;
        match connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        })? {
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client,
            } => {}
            other => return unexpected_response(other),
        }
        match connection.exchange(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("bridget-wake-cli"),
            capabilities: Vec::new(),
        })? {
            DaemonToWrapper::ClientWelcome { .. } => {}
            other => return unexpected_response(other),
        }
        connection
    } else {
        return Err(ClientError::Technical {
            code: "identity_unavailable",
            message: "identité auxiliaire requise pour cette action de réveil".into(),
        });
    };
    let mutation = !matches!(&request.request, WakeAction::List {});
    let command = WrapperToDaemon::WakeRequest { request };
    let response = if mutation {
        connection.send_then_wait(&command)
    } else {
        connection.exchange(&command)
    }
    .map_err(|error| match error {
        ClientError::Technical {
            code: "outcome_unknown",
            message,
        } if mutation => ClientError::Technical {
            code: "outcome_unknown",
            message: format!(
                "{message} Relisez la liste des réveils avant toute nouvelle création ou annulation ; conservez l’identifiant et les paramètres d’origine."
            ),
        },
        other => other,
    })?;
    match response {
        DaemonToWrapper::WakeResult { result } if result.version == WAKE_CONTRACT_VERSION => {
            Ok(result)
        }
        other => unexpected_response(other),
    }
}

#[derive(Debug)]
pub(crate) enum ClientError {
    InvalidParams(String),
    Technical { code: &'static str, message: String },
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidParams(message) => write!(f, "invalid_params: {message}"),
            Self::Technical { code, message } => write!(f, "{code}: {message}"),
        }
    }
}

pub(crate) struct DaemonConnection {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
    deadline: Instant,
    poisoned: bool,
    max_response_bytes: usize,
}

impl DaemonConnection {
    pub(crate) fn read_stream(&self) -> &UnixStream {
        self.reader.get_ref()
    }

    /// Flux déjà négocié : aucune échéance de tour, seulement un fragment borné.
    pub(crate) fn read_stream_response(
        &mut self,
        max_bytes: usize,
    ) -> Result<DaemonToWrapper, ClientError> {
        self.reader
            .get_ref()
            .set_read_timeout(None)
            .map_err(watch_output_error)?;
        let line = bridget_transport::jsonl::read_unix_line(
            &mut self.reader,
            max_bytes,
            bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(6)),
        )
        .map_err(watch_output_error)?
        .ok_or_else(watch_protocol_error)?;
        serde_json::from_slice::<DaemonToWrapper>(&line).map_err(|_| watch_protocol_error())
    }

    pub(crate) fn connect(socket: &Path) -> Result<Self, ClientError> {
        let deadline = Instant::now() + DAEMON_BUDGET;
        Self::connect_until(socket, deadline)
    }

    pub(crate) fn connect_until(socket: &Path, deadline: Instant) -> Result<Self, ClientError> {
        let contextual = crate::mcp::check_tool_call(None)?;
        let connected = if contextual {
            let mut registered = false;
            let mut checkpoint = |stream: &UnixStream| {
                crate::mcp::check_tool_call(if registered { None } else { Some(stream) }).map_err(
                    |_| io::Error::new(io::ErrorKind::Interrupted, "appel MCP interrompu"),
                )?;
                registered = true;
                Ok(())
            };
            bridget_transport::jsonl::connect_nonblocking_with(
                socket,
                deadline,
                Some(&mut checkpoint),
            )
        } else {
            connect_nonblocking(socket, deadline)
        };
        crate::mcp::check_tool_call(None)?;
        let stream = connected.map_err(|error| ClientError::Technical {
            code: "daemon_unreachable",
            message: format!("daemon Bridget injoignable : {error}"),
        })?;
        let reader_stream = stream.try_clone().map_err(|error| ClientError::Technical {
            code: "daemon_unreachable",
            message: format!("impossible de dupliquer le socket daemon : {error}"),
        })?;
        Ok(Self {
            writer: stream,
            reader: BufReader::new(reader_stream),
            deadline,
            poisoned: false,
            max_response_bytes: MAX_DAEMON_RESPONSE_BYTES,
        })
    }

    /// Après le handshake borné, seul l'échange d'attente reçoit son budget long.
    pub(crate) fn set_deadline(&mut self, deadline: Instant) {
        self.deadline = deadline;
    }

    pub(crate) fn exchange(
        &mut self,
        command: &WrapperToDaemon,
    ) -> Result<DaemonToWrapper, ClientError> {
        self.transmit(command, "daemon_unreachable")
    }

    pub(crate) fn send_then_wait(
        &mut self,
        command: &WrapperToDaemon,
    ) -> Result<DaemonToWrapper, ClientError> {
        let response = self.transmit(command, "outcome_unknown")?;
        if let WrapperToDaemon::SendIdempotent { message_id, .. } = command
            && !matches!(&response, DaemonToWrapper::IdempotencyResult {
                operation_kind, idempotency_key, ..
            } if operation_kind == "send" && idempotency_key == message_id)
        {
            // Un JSON décodable n'est pas une preuve de remise. Valider la
            // corrélation au même endroit pour CLI et MCP, sans exposer le
            // contenu d'une réponse étrangère dans le diagnostic.
            self.poison();
            return Err(ClientError::Technical {
                code: "outcome_unknown",
                message: "aucun accusé valide corrélé à cet envoi".into(),
            });
        }
        Ok(response)
    }

    fn transmit(
        &mut self,
        command: &WrapperToDaemon,
        after_write: &'static str,
    ) -> Result<DaemonToWrapper, ClientError> {
        let started = Instant::now();
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ClientError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        if json.len().saturating_add(1) > MAX_DAEMON_RESPONSE_BYTES {
            return Err(ClientError::InvalidParams(
                "requête supérieure à 16 Mio, LF inclus ; aucun envoi".into(),
            ));
        }
        let mut frame = json.into_bytes();
        frame.push(b'\n');
        // Sans BufWriter/write_all : le temps restant est réappliqué avant
        // CHAQUE appel système, pas seulement avant la première écriture.
        let mut offset = 0;
        let mut attempted = false;
        let written = (|| {
            while offset < frame.len() {
                self.apply_remaining_timeout(if !attempted {
                    "daemon_unreachable"
                } else {
                    after_write
                })?;
                attempted = true;
                match self.writer.write(&frame[offset..]) {
                    Ok(0) => {
                        return Err(ClientError::Technical {
                            code: after_write,
                            message: "écriture daemon interrompue".into(),
                        });
                    }
                    Ok(size) => offset += size,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => {
                        return Err(ClientError::Technical {
                            code: after_write,
                            message: format!("écriture daemon impossible : {error}"),
                        });
                    }
                }
            }
            Ok(())
        })();
        if let Err(error) = written {
            self.poison();
            return Err(error);
        }
        let response = self.read_response(after_write);
        if let Err(ClientError::Technical { code, .. }) = &response {
            // Libellés fermés : ni corps, ni credential, ni nom fourni par l'appelant.
            let phase = match command {
                WrapperToDaemon::RegisterAuxiliary { .. } => "auxiliary_registration",
                WrapperToDaemon::DaemonIdentityRequest => "daemon_identity",
                WrapperToDaemon::NativeDelegation { .. }
                | WrapperToDaemon::NativeDelegationT3 { .. } => "native_operation",
                _ => "operation_response",
            };
            eprintln!(
                "bridget_client_exchange_failed phase={phase} code={code} elapsed_ms={}",
                started.elapsed().as_millis()
            );
        }
        response
    }

    pub(crate) fn read_response(
        &mut self,
        failure_code: &'static str,
    ) -> Result<DaemonToWrapper, ClientError> {
        let result = (|| {
            if self.poisoned || Instant::now() >= self.deadline {
                return Err(ClientError::Technical {
                    code: failure_code,
                    message: "connexion fermée ou budget total dépassé".into(),
                });
            }
            let frame = if self.max_response_bytes == MAX_DAEMON_RESPONSE_BYTES {
                read_bounded_response(&mut self.reader, self.deadline)
            } else {
                bridget_transport::jsonl::read_unix_line(
                    &mut self.reader,
                    self.max_response_bytes,
                    bridget_transport::jsonl::LineDeadline::Absolute(self.deadline),
                )
                .and_then(|frame| {
                    frame.ok_or_else(|| {
                        io::Error::new(io::ErrorKind::UnexpectedEof, "daemon fermé sans réponse")
                    })
                })
                .and_then(|frame| {
                    String::from_utf8(frame).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "réponse non UTF-8")
                    })
                })
            };
            let line = frame.map_err(|error| ClientError::Technical {
                code: failure_code,
                message: format!("réponse daemon indisponible : {error}"),
            })?;
            let response = decode(&line).map_err(|_| ClientError::Technical {
                code: failure_code,
                message: "réponse daemon invalide".into(),
            })?;
            if Instant::now() >= self.deadline {
                return Err(ClientError::Technical {
                    code: failure_code,
                    message: "budget total dépassé pendant le décodage".into(),
                });
            }
            if matches!(response, DaemonToWrapper::UpdateInProgress) {
                return Err(ClientError::Technical {
                    code: "update_in_progress",
                    message: "Une mise à jour Bridget est en cours. Cet appel n’a pas été exécuté. Attendez le retour du lien, puis relisez le sort de la demande avec les mêmes identifiants et paramètres ; ne créez pas de mission de remplacement.".into(),
                });
            }
            if matches!(response, DaemonToWrapper::Disconnect) {
                // Une fermeture annoncée a le même sort incertain qu'un EOF.
                // Conserver le code de l'appelant, notamment outcome_unknown
                // après une mutation ; ne pas inventer un accusé ou un refus.
                return Err(ClientError::Technical {
                    code: failure_code,
                    message: "Le démon Bridget a fermé la connexion. La commande peut avoir été traitée ; cette coupure ne prouve ni son échec ni son annulation. Attendez le retour du lien, puis vérifiez le sort de l’opération avant toute nouvelle action. Conservez les identifiants et les paramètres d’origine.".into(),
                });
            }
            // Une réponse complète reste valide si le pair ferme ensuite.
            // Aucun setsockopt après lecture : macOS peut le refuser sur EOF.
            Ok(response)
        })();
        if result.is_err() {
            self.poison();
        }
        result
    }

    fn poison(&mut self) {
        self.poisoned = true;
        let _ = self.writer.shutdown(std::net::Shutdown::Both);
    }

    fn apply_remaining_timeout(&self, code: &'static str) -> Result<(), ClientError> {
        if self.poisoned {
            return Err(ClientError::Technical {
                code,
                message: "connexion fermée après erreur ; ouvrir une nouvelle connexion".into(),
            });
        }
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| ClientError::Technical {
                code,
                message: "budget total de la connexion dépassé".to_string(),
            })?;
        self.writer
            .set_write_timeout(Some(remaining))
            .map_err(|error| ClientError::Technical {
                code,
                message: format!("impossible de borner l'écriture daemon : {error}"),
            })?;
        Ok(())
    }
}

/// Lecture bornée LF inclus. Chaque attente poll consomme le RESTE du budget,
/// sans reconfigurer une socket dont le pair a éventuellement déjà fermé.
pub(crate) fn read_bounded_response(
    reader: &mut BufReader<UnixStream>,
    deadline: Instant,
) -> io::Result<String> {
    let frame = bridget_transport::jsonl::read_unix_line(
        reader,
        MAX_DAEMON_RESPONSE_BYTES,
        bridget_transport::jsonl::LineDeadline::Absolute(deadline),
    )?
    .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "daemon fermé sans réponse"))?;
    String::from_utf8(frame)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "réponse non UTF-8"))
}

pub(crate) use bridget_transport::jsonl::connect_nonblocking;

pub(crate) fn registered_connection(
    identity: &str,
    instance_id: &str,
    socket: &Path,
) -> Result<DaemonConnection, ClientError> {
    registered_connection_until(
        identity,
        instance_id,
        socket,
        Instant::now() + DAEMON_BUDGET,
    )
}

pub(crate) fn registered_connection_until(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    deadline: Instant,
) -> Result<DaemonConnection, ClientError> {
    // L'inscription n'émet aucune commande métier. Reprendre une coupure
    // précoce une fois, dans l'échéance du caller, sans rejouer sa mutation.
    // Un délai entièrement consommé ou un refus d'identité reste terminal.
    let mut retried = false;
    loop {
        let result = (|| {
            let mut connection = DaemonConnection::connect_until(socket, deadline)?;
            authenticate_auxiliary(&mut connection, identity, instance_id, socket)?;
            Ok(connection)
        })();
        match result {
            Err(ClientError::Technical {
                code: "daemon_unreachable",
                ..
            }) if !retried && Instant::now() < deadline => retried = true,
            Ok(connection) => {
                if retried {
                    eprintln!("bridget_registration_recovered attempts=2");
                }
                return Ok(connection);
            }
            other => return other,
        }
    }
}

pub(crate) fn authenticate_auxiliary(
    connection: &mut DaemonConnection,
    identity: &str,
    instance_id: &str,
    socket: &Path,
) -> Result<(), ClientError> {
    let registration = crate::mcp_identity::auxiliary_registration(identity, instance_id, socket)
        .map_err(|message| ClientError::Technical {
        code: "auxiliary_credential_required",
        message,
    })?;
    match connection.exchange(&registration)? {
        DaemonToWrapper::Registered { agent_id, .. } if agent_id == identity => Ok(()),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "auxiliary_identity_unproven",
            message: reason,
        }),
        other => unexpected_response(other),
    }
}

pub(crate) fn unexpected_response<T>(_response: DaemonToWrapper) -> Result<T, ClientError> {
    Err(ClientError::Technical {
        code: "daemon_protocol",
        message: "réponse daemon inattendue pour cette opération".into(),
    })
}

#[cfg(test)]
#[path = "client_security_tests.rs"]
mod security_tests;

/// Contrat181 : négociation explicite, preuve d’agent ou principal humain local séparés.
pub(crate) fn user_input_request(
    socket: &Path,
    identity: Option<(&str, &str)>,
    request: serde_json::Value,
) -> Result<serde_json::Value, ClientError> {
    let wait = if request["action"] == "wait" {
        request["wait_timeout"].as_u64().unwrap_or(3600).min(3600)
    } else {
        0
    };
    let mut connection =
        DaemonConnection::connect_until(socket, Instant::now() + Duration::from_secs(wait + 10))?;
    let issuer = if let Some((name, instance)) = identity {
        authenticate_auxiliary(&mut connection, name, instance, socket)?;
        crate::communication::issuer_scope(instance)
    } else {
        match connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        })? {
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client,
            } => {}
            other => return unexpected_response(other),
        }
        crate::communication::issuer_scope("bridget-control-cli")
    };
    let mut capabilities = vec![ClientCapability::UserInputV1];
    if identity.is_none() {
        capabilities.push(ClientCapability::ControlStateV1);
    }
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: issuer,
        capabilities,
    })? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::UserInputV1) => {}
        _ => {
            return Err(ClientError::Technical {
                code: "capability_unavailable",
                message: "le daemon ne négocie pas user_input_v1".into(),
            });
        }
    }
    match connection.send_then_wait(&WrapperToDaemon::UserInput { request })? {
        DaemonToWrapper::UserInputResult { result } => Ok(result),
        other => unexpected_response(other),
    }
}

pub(crate) fn human_agent_tracking(
    socket: &Path,
    request: bridget_transport::agent_tracking::AgentTrackingRequest,
) -> Result<serde_json::Value, ClientError> {
    let mut connection = DaemonConnection::connect(socket)?;
    if !matches!(
        connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        })?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ) {
        return Err(ClientError::Technical {
            code: "human_principal_required",
            message: "Le rôle client humain est requis.".into(),
        });
    }
    let capability = ClientCapability::AgentTrackingV1;
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
        capabilities: vec![capability],
    })? {
        DaemonToWrapper::ClientWelcome {
            version,
            capabilities,
            ..
        } if version == CLIENT_CONTRACT_VERSION && capabilities.contains(&capability) => {}
        _ => {
            return Err(ClientError::Technical {
                code: "agent_tracking_unavailable",
                message:
                    "Le client et le service doivent charger agents dormants avant cette action."
                        .into(),
            });
        }
    }
    match connection.exchange(&WrapperToDaemon::AgentTracking { request })? {
        DaemonToWrapper::AgentTrackingResult { result } => Ok(result),
        other => unexpected_response(other),
    }
}

pub(crate) fn human_specialists(
    socket: &Path,
    request: bridget_transport::specialists::SpecialistRequest,
) -> Result<serde_json::Value, ClientError> {
    let mut connection = DaemonConnection::connect(socket)?;
    if !matches!(
        connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        })?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ) {
        return Err(ClientError::Technical {
            code: "human_principal_required",
            message: "Le rôle client humain est requis.".into(),
        });
    }
    let capability = ClientCapability::SpecialistsV1;
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
        capabilities: vec![capability],
    })? {
        DaemonToWrapper::ClientWelcome {
            version,
            capabilities,
            ..
        } if version == CLIENT_CONTRACT_VERSION && capabilities.contains(&capability) => {}
        _ => {
            return Err(ClientError::Technical {
                code: "specialists_capability_required",
                message: "Faire charger le daemon et le client specialists_v1.".into(),
            });
        }
    }
    match connection.exchange(&WrapperToDaemon::Specialists { request })? {
        DaemonToWrapper::NativeDelegationResult { result } => Ok(result),
        other => unexpected_response(other),
    }
}

#[cfg(test)]
mod tracking230_federated_wait_tests {
    use super::*;
    use std::io::{BufRead, Write};
    #[test]
    fn tracking230_capacites_attente_acceptent_amorcage_relais_dans_budget_client() {
        let path =
            std::env::temp_dir().join(format!("bg230-wait-{}.sock", uuid::Uuid::new_v4().simple()));
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            assert!(matches!(
                bridget_transport::protocol::decode::<WrapperToDaemon>(&line).unwrap(),
                WrapperToDaemon::DaemonIdentityRequest
            ));
            std::thread::sleep(Duration::from_millis(3200));
            writeln!(
                stream,
                "{}",
                bridget_transport::protocol::encode(&DaemonToWrapper::DaemonIdentityReport {
                    host: "fixture".into(),
                    db_path: "/fixture/db".into(),
                    instance_id: "fixture".into(),
                    features: vec!["wait_v1".into()]
                })
                .unwrap()
            )
            .unwrap();
        });
        assert!(daemon_supports_wait(&path).unwrap());
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
