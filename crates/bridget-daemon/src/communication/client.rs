//! Client Unix partagé : une seule inscription auxiliaire et un seul budget CLI/MCP.
//! Aucune logique de présentation ni accès client au stockage du daemon.

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

pub(crate) const DAEMON_BUDGET: Duration = Duration::from_secs(10);

pub(crate) fn observation_request(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    request: bridget_transport::protocol::ObservationRequest,
) -> Result<serde_json::Value, ClientError> {
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
    match connection.send_then_wait(&WrapperToDaemon::ThreadRequest { request })? {
        DaemonToWrapper::ThreadResult { result } => Ok(result),
        DaemonToWrapper::Nack { reason, .. } => Err(ClientError::Technical {
            code: "thread_rejected",
            message: reason,
        }),
        other => unexpected_response(other),
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
        DaemonToWrapper::Ack { id } | DaemonToWrapper::Nack { id, .. } if id == expected_id => {
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
    DOMAIN_TEST_OBSERVER_MUTEX
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

#[cfg(test)]
pub(crate) fn replace_domain_lock_contention_observer(
    observer: Option<DomainLockContentionObserver>,
) {
    *DOMAIN_LOCK_CONTENTION_OBSERVER
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = observer;
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
    *DOMAIN_AFTER_ACK_OBSERVER
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = observer;
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
}

impl DaemonConnection {
    pub(crate) fn connect(socket: &Path) -> Result<Self, ClientError> {
        let deadline = Instant::now() + DAEMON_BUDGET;
        Self::connect_until(socket, deadline)
    }

    pub(crate) fn connect_until(socket: &Path, deadline: Instant) -> Result<Self, ClientError> {
        let stream =
            connect_nonblocking(socket, deadline).map_err(|error| ClientError::Technical {
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
        })
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
        self.read_response(after_write)
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
            let line = read_bounded_response(&mut self.reader, self.deadline).map_err(|error| {
                ClientError::Technical {
                    code: failure_code,
                    message: format!("réponse daemon indisponible : {error}"),
                }
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
                message: "budget total de 10 s dépassé".to_string(),
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

fn registered_connection_until(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    deadline: Instant,
) -> Result<DaemonConnection, ClientError> {
    let mut connection = DaemonConnection::connect_until(socket, deadline)?;
    authenticate_auxiliary(&mut connection, identity, instance_id, socket)?;
    Ok(connection)
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
mod security_tests {
    use super::*;
    use std::io::BufRead;
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;

    fn pair(budget: Duration) -> (DaemonConnection, UnixStream) {
        let (stream, peer) = UnixStream::pair().unwrap();
        (
            DaemonConnection {
                reader: BufReader::new(stream.try_clone().unwrap()),
                writer: stream,
                deadline: Instant::now() + budget,
                poisoned: false,
            },
            peer,
        )
    }

    fn registered_server(
        path: &Path,
        response: DaemonToWrapper,
    ) -> (thread::JoinHandle<()>, mpsc::Receiver<WrapperToDaemon>) {
        let listener = UnixListener::bind(path).unwrap();
        crate::mcp_identity::mock_private_identity(
            path,
            "89000000-0000-4000-8000-000000000194",
            "instance-094",
        );
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = decode(line.trim()).unwrap()
            else {
                panic!("Register attendu");
            };
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            seen_tx.send(decode(line.trim()).unwrap()).unwrap();
            writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            stream.flush().unwrap();
        });
        (server, seen_rx)
    }

    #[test]
    fn spec094_rename_confirme_uuid_et_nom_normalise_sans_accepter_une_substitution() {
        use bridget_transport::protocol::DisplayNameOutcome;

        let identity = "89000000-0000-4000-8000-000000000194";
        for (returned_name, accepted) in [("Agent Vingt Trois", true), ("Autre agent", false)] {
            let path = crate::mcp_identity::mock_socket("rename");
            let response = DaemonToWrapper::DisplayNameResult {
                outcome: DisplayNameOutcome::Applied {
                    agent_id: identity.to_string(),
                    display_name: returned_name.to_string(),
                    revision: 2,
                },
            };
            let (server, seen) = registered_server(&path, response);
            let result =
                rename_display_name(identity, "instance-094", &path, "Agent   Vingt Trois");
            assert_eq!(result.is_ok(), accepted, "réponse={returned_name}");
            assert!(matches!(
                seen.recv().unwrap(),
                WrapperToDaemon::DisplayNameSet { .. }
            ));
            server.join().unwrap();
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn spec094_domaine_distingue_ack_memoire_et_echec_de_persistance() {
        let root =
            std::env::temp_dir().join(format!("b94-domain-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let (server, seen) = registered_server(
            &path,
            DaemonToWrapper::Ack {
                id: "domain".to_string(),
            },
        );
        let identity = "89000000-0000-4000-8000-000000000194";
        // Le répertoire des domaines doit rester accessible au préflight du
        // verrou. Seule la cible finale est rendue impropre à un remplacement,
        // afin de prouver l'échec durable APRES l'ACK mémoire.
        std::fs::create_dir_all(root.join("agent-domains").join(identity)).unwrap();
        let result = set_domain(
            identity,
            "instance-094",
            &path,
            Some("documentation".to_string()),
        );
        assert!(matches!(
            result,
            Err(ClientError::Technical {
                code: "domain_persistence_failed",
                ref message,
            }) if message.contains("appliqué en mémoire")
                && message.contains("persistance non confirmée")
        ));
        assert!(matches!(
            seen.recv_timeout(Duration::from_secs(2)).unwrap(),
            WrapperToDaemon::Domain {
                domain: Some(domain), ..
            } if domain == "documentation"
        ));
        server.join().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_deux_domaines_concurrents_gardent_le_meme_ordre_en_memoire_et_sur_disque() {
        let _observer_guard = lock_domain_test_observers();
        let root = std::env::temp_dir().join(format!(
            "b94-domain-order-{}",
            uuid::Uuid::new_v4().simple()
        ));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000294",
            "instance-094",
        );
        let (seen_tx, seen_rx) = mpsc::channel();
        let memory_domain = Arc::new(std::sync::Mutex::new(None));
        let memory_domain_for_server = Arc::clone(&memory_domain);
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } =
                    decode(line.trim()).unwrap()
                else {
                    panic!("Register attendu");
                };
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        credential: None,
                        agent_id
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::Domain {
                    domain: Some(domain),
                    ..
                } = decode(line.trim()).unwrap()
                else {
                    panic!("Domain attendu");
                };
                *memory_domain_for_server.lock().unwrap() = Some(domain.clone());
                seen_tx.send(domain).unwrap();
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Ack {
                        id: "domain".to_string(),
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
            }
        });

        let identity = "89000000-0000-4000-8000-000000000294";
        let first_observed = Arc::new(AtomicBool::new(false));
        let first_observed_for_hook = Arc::clone(&first_observed);
        let (blocked_tx, blocked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (contended_tx, contended_rx) = mpsc::channel();
        let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
        let release_rx_for_hook = Arc::clone(&release_rx);
        let identity_for_hook = identity.to_string();
        replace_domain_after_ack_observer(Some(Arc::new(move |observed_identity| {
            if observed_identity == identity_for_hook
                && !first_observed_for_hook.swap(true, Ordering::SeqCst)
            {
                blocked_tx.send(()).unwrap();
                release_rx_for_hook.lock().unwrap().recv().unwrap();
            }
        })));
        let contention_reported = Arc::new(AtomicBool::new(false));
        let contention_reported_for_hook = Arc::clone(&contention_reported);
        let identity_for_contention = identity.to_string();
        replace_domain_lock_contention_observer(Some(Arc::new(move |observed_identity| {
            if observed_identity == identity_for_contention
                && !contention_reported_for_hook.swap(true, Ordering::SeqCst)
            {
                contended_tx.send(()).unwrap();
            }
        })));

        let first_path = path.clone();
        let first = thread::spawn(move || {
            set_domain(
                identity,
                "instance-094",
                &first_path,
                Some("premier".to_string()),
            )
        });
        assert_eq!(seen_rx.recv().unwrap(), "premier");
        blocked_rx.recv().unwrap();
        let second_path = path.clone();
        let second = thread::spawn(move || {
            set_domain(
                identity,
                "instance-094",
                &second_path,
                Some("second".to_string()),
            )
        });
        contended_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("le second appel doit rencontrer le verrou réellement tenu");
        let second_reached_daemon_while_first_blocked = seen_rx.try_recv().is_ok();
        release_tx.send(()).unwrap();
        assert!(first.join().unwrap().is_ok());
        assert!(second.join().unwrap().is_ok());
        assert_eq!(
            seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "second"
        );
        server.join().unwrap();
        replace_domain_after_ack_observer(None);
        replace_domain_lock_contention_observer(None);

        assert!(
            !second_reached_daemon_while_first_blocked,
            "le second domaine a dépassé le premier avant sa persistance"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("agent-domains").join(identity)).unwrap(),
            "second"
        );
        assert_eq!(memory_domain.lock().unwrap().as_deref(), Some("second"));
        let _ = std::fs::remove_file(path);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_control_status_negocie_lookup_borne_a_l_instance_sans_mutation() {
        use bridget_transport::protocol::ControlStateRefusal;

        let path = crate::mcp_identity::mock_socket("control");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000194",
            "instance-094",
        );
        let (hello_tx, hello_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut registration, _) = listener.accept().unwrap();
            let mut registration_reader = BufReader::new(registration.try_clone().unwrap());
            let mut line = String::new();
            registration_reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = decode(line.trim()).unwrap()
            else {
                panic!("Register attendu avant la lecture Client");
            };
            writeln!(
                registration,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id
                })
                .unwrap()
            )
            .unwrap();
            registration.flush().unwrap();
            drop(registration_reader);
            drop(registration);

            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let hello = decode(line.trim()).unwrap();
            hello_tx.send(hello).unwrap();
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-094".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::Lookup],
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::ControlStateRead { .. }
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::ControlStateRejected {
                    reason: ControlStateRefusal::StoreUnavailable,
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
        });
        assert!(
            read_control_status(
                "89000000-0000-4000-8000-000000000194",
                "instance-094",
                &path,
                0,
            )
            .is_err()
        );
        match hello_rx.recv().unwrap() {
            WrapperToDaemon::ClientHello {
                issuer_scope,
                capabilities,
                ..
            } => {
                assert_eq!(
                    issuer_scope,
                    crate::communication::issuer_scope("instance-094")
                );
                assert_eq!(capabilities, vec![ClientCapability::Lookup]);
                assert!(!capabilities.contains(&ClientCapability::ControlStateV1));
            }
            other => panic!("ClientHello attendu, reçu {other:?}"),
        }
        server.join().unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn spec094_control_status_stoppe_si_l_attestation_auxiliaire_est_perimee() {
        let path = crate::mcp_identity::mock_socket("control-stale");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000194",
            "instance-perimee",
        );
        let (check_tx, check_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary {
                    instance_id, ..
                } if instance_id == "instance-perimee"
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::Nack {
                    id: "register".to_string(),
                    reason: "instance remplacée".to_string(),
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            check_rx.recv().unwrap();
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(
                    listener.accept(),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock
                ),
                "une connexion Client a contourné l'attestation refusée"
            );
        });
        assert!(
            read_control_status(
                "89000000-0000-4000-8000-000000000194",
                "instance-perimee",
                &path,
                0,
            )
            .is_err()
        );
        check_tx.send(()).unwrap();
        server.join().unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn une_reponse_sans_lf_ne_devient_pas_un_accuse_valide() {
        let (mut connection, mut peer) = pair(Duration::from_secs(2));
        peer.write_all(b"{\"type\":\"Ack\",\"id\":\"ok\"}").unwrap();
        peer.shutdown(std::net::Shutdown::Write).unwrap();
        assert!(
            matches!(
                connection.read_response("outcome_unknown"),
                Err(ClientError::Technical {
                    code: "outcome_unknown",
                    ..
                })
            ),
            "EOF partiel accepté"
        );
    }

    #[test]
    fn le_goutte_a_goutte_ne_renouvelle_pas_le_budget_global() {
        let (mut connection, mut peer) = pair(Duration::from_millis(150));
        let writer = thread::spawn(move || {
            for byte in b"{\"type\":\"Ack\",\"id\":\"ok\"}\n" {
                if peer.write_all(&[*byte]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(25));
            }
        });
        let started = Instant::now();
        let result = connection.read_response("outcome_unknown");
        let elapsed = started.elapsed();
        drop(connection);
        writer.join().unwrap();
        assert!(
            matches!(
                result,
                Err(ClientError::Technical {
                    code: "outcome_unknown",
                    ..
                })
            ),
            "réponse après échéance acceptée"
        );
        assert!(
            elapsed < Duration::from_millis(450),
            "budget renouvelé : {elapsed:?}"
        );
    }

    #[test]
    fn la_borne_de_reponse_compte_le_lf_et_ne_restitue_pas_le_contenu() {
        const LIMIT: usize = 16 * 1024 * 1024;
        for excess in [0, 1] {
            let (mut connection, mut peer) = pair(Duration::from_secs(5));
            let writer = thread::spawn(move || {
                let suffix = b"{\"type\":\"Ack\",\"id\":\"ok\"}\n";
                let mut bytes = vec![b' '; LIMIT + excess - suffix.len()];
                bytes.extend_from_slice(suffix);
                let _ = peer.write_all(&bytes);
            });
            let result = connection.read_response("outcome_unknown");
            drop(connection);
            writer.join().unwrap();
            assert_eq!(
                result.is_ok(),
                excess == 0,
                "borne LF incluse, dépassement={excess}, réponse={result:?}"
            );
        }
    }

    #[test]
    fn erreur_de_couture_ne_recopie_pas_un_secret_dans_son_diagnostic() {
        let result = unexpected_response::<()>(DaemonToWrapper::Nack {
            id: "id".into(),
            reason: "CANARI_SECRET_089".into(),
        });
        assert!(!format!("{result:?}").contains("CANARI_SECRET_089"));
    }

    #[test]
    fn erreur_empoisonne_la_connexion_meme_si_une_reponse_valide_suit() {
        let (mut connection, mut peer) = pair(Duration::from_secs(2));
        peer.write_all(b"invalide\n{\"type\":\"Ack\",\"id\":\"tardif\"}\n")
            .unwrap();
        assert!(connection.read_response("outcome_unknown").is_err());
        // Le second message est déjà dans le buffer : sans la garde poisoned,
        // cette lecture le prendrait pour l'accusé d'une nouvelle opération.
        assert!(!connection.reader.buffer().is_empty());
        assert!(connection.read_response("outcome_unknown").is_err());
        assert!(connection.exchange(&WrapperToDaemon::ListAgents).is_err());
    }

    #[test]
    fn trois_phases_lentes_consument_une_seule_echeance() {
        use std::io::BufRead;
        let (mut connection, mut peer) = pair(Duration::from_millis(1000));
        let peer_reader = peer.try_clone().unwrap();
        let (seen_tx, seen_rx) = std::sync::mpsc::channel();
        let writer = thread::spawn(move || {
            peer_reader
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(peer_reader);
            for _ in 0..3 {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                seen_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(400)); // lenteur du pair, pas synchronisation du test
                if peer
                    .write_all(b"{\"type\":\"Ack\",\"id\":\"phase\"}\n")
                    .is_err()
                {
                    break;
                }
            }
        });
        let started = Instant::now();
        let results = (0..3)
            .map(|_| connection.exchange(&WrapperToDaemon::ListAgents))
            .collect::<Vec<_>>();
        let elapsed = started.elapsed();
        drop(connection);
        writer.join().unwrap();
        assert_eq!(
            seen_rx.try_iter().count(),
            3,
            "les trois phases doivent être réellement tentées"
        );
        assert!(results[0].is_ok() && results[1].is_ok(), "{results:?}");
        assert!(
            results[2].is_err(),
            "renouveler le budget à chaque échange accepterait les trois réponses"
        );
        assert!(elapsed < Duration::from_millis(2000), "{elapsed:?}");
    }

    #[test]
    fn backlog_reel_plein_est_refuse_sans_attendre_un_accept() {
        use std::os::unix::net::UnixListener;
        let path = std::path::PathBuf::from("/tmp")
            .join(format!("b89-connect-{}", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 1) }, 0);
        // Une connexion dans la file, personne n'appelle accept pendant la sonde.
        let mut fillers = Vec::new();
        let mut refusal = None;
        for _ in 0..64 {
            let started = Instant::now();
            match connect_nonblocking(&path, started + Duration::from_millis(150)) {
                Ok(stream) => fillers.push(stream),
                Err(error) => {
                    refusal = Some((error, started.elapsed()));
                    break;
                }
            }
        }
        assert!(!fillers.is_empty());
        let (error, elapsed) = refusal.expect("borne réelle de la file atteinte sans accept");
        // Observé macOS : ECONNREFUSED sur file AF_UNIX pleine ; Linux peut
        // rendre EAGAIN. Ne pas prétendre avoir traversé EINPROGRESS/poll
        // lorsque le noyau a refusé immédiatement la connexion.
        assert!(
            matches!(
                error.kind(),
                io::ErrorKind::TimedOut
                    | io::ErrorKind::WouldBlock
                    | io::ErrorKind::ConnectionRefused
            ),
            "{error}"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "connect a ignoré la deadline : {elapsed:?}"
        );
        assert_eq!(
            unsafe { libc::fcntl(fillers[0].as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
            libc::FD_CLOEXEC
        );
        drop(fillers);
        drop(listener);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn budget_expire_et_chemin_nul_refuses_avant_toute_connexion() {
        use std::os::unix::ffi::OsStringExt;
        use std::os::unix::net::UnixListener;
        let path = std::path::PathBuf::from("/tmp")
            .join(format!("b89-no-connect-{}", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        assert_eq!(
            connect_nonblocking(&path, Instant::now() - Duration::from_millis(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        let mut nul = path.as_os_str().as_bytes().to_vec();
        nul.extend_from_slice(b"\0suffixe");
        assert_eq!(
            connect_nonblocking(
                Path::new(&std::ffi::OsString::from_vec(nul)),
                Instant::now() + Duration::from_secs(1)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
        );
        drop(listener);
        std::fs::remove_file(path).unwrap();
    }

    /// Session 104 : un daemon antérieur ne connaît pas `ledger_search`. Le
    /// client rend une erreur technique explicite (Nack → `daemon_protocol`,
    /// coupure → `outcome_unknown`) et ne se replie jamais sur la projection
    /// globale ni sur un fichier local.
    #[test]
    fn spec104_ancien_daemon_erreur_honnete_sans_repli() {
        let root =
            std::env::temp_dir().join(format!("b104-old-daemon-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
        );
        let server = thread::spawn(move || {
            for turn in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } =
                    decode(line.trim()).unwrap()
                else {
                    panic!("RegisterAuxiliary attendu");
                };
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        credential: None,
                        agent_id
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(line.contains("\"ledger_search\""), "{line}");
                if turn == 0 {
                    // Daemon ancien qui répond par un refus générique.
                    writeln!(
                        stream,
                        "{}",
                        encode(&DaemonToWrapper::Nack {
                            id: "ledger".into(),
                            reason: "commande inconnue".into()
                        })
                        .unwrap()
                    )
                    .unwrap();
                    stream.flush().unwrap();
                } // turn 1 : coupure sèche, aucune réponse.
            }
        });
        let request = bridget_transport::protocol::LedgerSearchRequest {
            source: bridget_transport::protocol::LedgerSearchSource::Messages,
            query: "x".into(),
            author: None,
            peer: None,
            since: None,
            until: None,
            limit: None,
            cursor: None,
            thread_id: None,
        };
        let nack = ledger_search(
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
            &path,
            request.clone(),
        );
        assert!(
            matches!(
                &nack,
                Err(ClientError::Technical {
                    code: "daemon_protocol",
                    ..
                })
            ),
            "{nack:?}"
        );
        let cut = ledger_search(
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
            &path,
            request,
        );
        assert!(
            matches!(&cut, Err(ClientError::Technical { .. })),
            "coupure = erreur technique, jamais une page : {cut:?}"
        );
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }
}
