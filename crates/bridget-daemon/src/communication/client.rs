//! Client Unix partagé : une seule inscription auxiliaire et un seul budget CLI/MCP.
//! Aucune logique de présentation ni accès client au stockage du daemon.

use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

const DAEMON_BUDGET: Duration = Duration::from_secs(10);

pub(crate) fn rename_display_name(
    identity: &str,
    instance_id: &str,
    socket: &Path,
    name: &str,
) -> Result<DaemonToWrapper, ClientError> {
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let response = connection.send_then_wait(&WrapperToDaemon::DisplayNameSet {
        request: bridget_transport::protocol::DisplayNameRequest {
            version: 1,
            display_name: name.into(),
        },
    })?;
    match &response {
        DaemonToWrapper::DisplayNameResult {
            outcome: bridget_transport::protocol::DisplayNameOutcome::Applied { agent_id, .. },
        } if agent_id == identity => Ok(response),
        DaemonToWrapper::DisplayNameResult {
            outcome: bridget_transport::protocol::DisplayNameOutcome::Rejected { .. },
        } => Ok(response),
        _ => unexpected_response(response),
    }
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

pub(crate) struct DaemonConnection {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
    deadline: Instant,
}

impl DaemonConnection {
    pub(crate) fn connect(socket: &Path) -> Result<Self, ClientError> {
        let deadline = Instant::now() + DAEMON_BUDGET;
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
            writer: BufWriter::new(stream),
            reader: BufReader::new(reader_stream),
            deadline,
        })
    }

    pub(crate) fn exchange(
        &mut self,
        command: &WrapperToDaemon,
    ) -> Result<DaemonToWrapper, ClientError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ClientError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ClientError::Technical {
            code: "daemon_unreachable",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer
            .flush()
            .map_err(|error| ClientError::Technical {
                code: "daemon_unreachable",
                message: format!("flush daemon impossible : {error}"),
            })?;
        self.read_response("daemon_unreachable")
    }

    pub(crate) fn send_then_wait(
        &mut self,
        command: &WrapperToDaemon,
    ) -> Result<DaemonToWrapper, ClientError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ClientError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ClientError::Technical {
            code: "outcome_unknown",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer
            .flush()
            .map_err(|error| ClientError::Technical {
                code: "outcome_unknown",
                message: format!("flush daemon impossible : {error}"),
            })?;
        self.read_response("outcome_unknown")
    }

    fn read_response(
        &mut self,
        failure_code: &'static str,
    ) -> Result<DaemonToWrapper, ClientError> {
        self.apply_remaining_timeout(failure_code)?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|error| ClientError::Technical {
                code: failure_code,
                message: format!("réponse daemon indisponible : {error}"),
            })?;
        if line.is_empty() {
            return Err(ClientError::Technical {
                code: failure_code,
                message: "daemon Bridget a fermé la connexion sans réponse".to_string(),
            });
        }
        decode(line.trim_end()).map_err(|error| ClientError::Technical {
            code: failure_code,
            message: format!("réponse daemon invalide : {error}"),
        })
    }

    fn apply_remaining_timeout(&self, code: &'static str) -> Result<(), ClientError> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| ClientError::Technical {
                code,
                message: "budget total de 10 s dépassé".to_string(),
            })?;
        self.writer
            .get_ref()
            .set_write_timeout(Some(remaining))
            .map_err(|error| ClientError::Technical {
                code,
                message: format!("impossible de borner l'écriture daemon : {error}"),
            })?;
        self.reader
            .get_ref()
            .set_read_timeout(Some(remaining))
            .map_err(|error| ClientError::Technical {
                code,
                message: format!("impossible de borner la lecture daemon : {error}"),
            })
    }
}

pub(crate) fn connect_nonblocking(socket: &Path, deadline: Instant) -> io::Result<UnixStream> {
    if deadline <= Instant::now() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "budget connexion dépassé",
        ));
    }
    let path = socket.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "socket Unix trop long",
        ));
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        address.sun_len = (std::mem::size_of::<libc::sa_family_t>() + path.len() + 1) as u8;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            path.as_ptr().cast(),
            address.sun_path.as_mut_ptr(),
            path.len(),
        );
    }
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let close = |fd: libc::c_int| unsafe { libc::close(fd) };
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        close(fd);
        return Err(io::Error::last_os_error());
    }
    let result = unsafe {
        libc::connect(
            fd,
            (&address as *const libc::sockaddr_un).cast(),
            std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            close(fd);
            return Err(error);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "budget connexion dépassé"))?;
        let timeout = remaining.as_millis().min(i32::MAX as u128) as libc::c_int;
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };
        if unsafe { libc::poll(&mut pollfd, 1, timeout) } <= 0 {
            close(fd);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "connexion daemon expirée",
            ));
        }
        let mut so_error: libc::c_int = 0;
        let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut so_error as *mut libc::c_int).cast(),
                &mut length,
            )
        } < 0
            || so_error != 0
        {
            close(fd);
            return Err(io::Error::from_raw_os_error(so_error));
        }
    }
    let stream = unsafe { UnixStream::from_raw_fd(fd) };
    stream.set_nonblocking(false)?;
    Ok(stream)
}

pub(crate) fn registered_connection(
    identity: &str,
    instance_id: &str,
    socket: &Path,
) -> Result<DaemonConnection, ClientError> {
    let mut connection = DaemonConnection::connect(socket)?;
    let registration = WrapperToDaemon::Register {
        agent_type: "mcp".to_string(),
        identity_version: 2,
        agent_id: identity.to_string(),
        host: None,
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: Some(instance_id.to_string()),
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    };
    match connection.exchange(&registration)? {
        DaemonToWrapper::Registered { .. } => Ok(connection),
        other => unexpected_response(other),
    }
}

pub(crate) fn unexpected_response<T>(response: DaemonToWrapper) -> Result<T, ClientError> {
    Err(ClientError::Technical {
        code: "daemon_protocol",
        message: format!("réponse daemon inattendue : {response:?}"),
    })
}
