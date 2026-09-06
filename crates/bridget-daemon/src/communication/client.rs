//! Client Unix partagé : une seule inscription auxiliaire et un seul budget CLI/MCP.
//! Aucune logique de présentation ni accès client au stockage du daemon.

use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{self, BufReader, Write};
#[cfg(test)]
use std::os::fd::AsRawFd;
#[cfg(test)]
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

const DAEMON_BUDGET: Duration = Duration::from_secs(10);

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

    fn read_response(
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

pub(crate) fn unexpected_response<T>(_response: DaemonToWrapper) -> Result<T, ClientError> {
    Err(ClientError::Technical {
        code: "daemon_protocol",
        message: "réponse daemon inattendue pour cette opération".into(),
    })
}

#[cfg(test)]
mod security_tests {
    use super::*;
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
}
