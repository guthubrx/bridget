//! Cadrage WebSocket sur Unix du Codex local. Aucun RPC ni événement métier
//! n'est reconstruit ici : une trame texte reste ses octets d'origine.
use crate::jsonl::{MAX_DAEMON_FRAME_BYTES, connect_nonblocking};
use std::io::{self, Write};
use std::net::Shutdown;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::{Error, HandshakeError, Message, WebSocket, client::client_with_config};

type Shared = Arc<Mutex<WebSocket<UnixStream>>>;
const IO_BUDGET: Duration = Duration::from_secs(3);

/// Vérifié AVANT lancement : le serveur ne doit exposer aucun contrôle dans
/// un répertoire partagé, ni écraser un chemin/symlink préexistant.
pub(crate) fn validate_private_path(path: &Path) -> io::Result<()> {
    crate::jsonl::validate_unix_socket_path(path)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "socket sans répertoire privé")
        })?;
    let metadata = std::fs::symlink_metadata(parent)?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "répertoire de socket Codex non privé (0700, propriétaire courant requis)",
        ));
    }
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "chemin de socket Codex déjà occupé",
        )),
    }
}

pub(crate) struct SocketLines {
    shared: Shared,
    ended: bool,
}

pub(crate) struct SocketWriter {
    shared: Shared,
    bytes: Vec<u8>,
}

fn config(maximum: usize) -> WebSocketConfig {
    WebSocketConfig::default()
        .write_buffer_size(0)
        .max_write_buffer_size(maximum + 1024)
        .max_message_size(Some(maximum))
        .max_frame_size(Some(maximum))
}

fn error(error: Error) -> io::Error {
    match error {
        Error::Io(error) => error,
        other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
    }
}

/// L'échéance englobe connect ET l'Upgrade HTTP, même si le pair envoie
/// l'en-tête au goutte-à-goutte. Pas de connect bloquant ni de budget renouvelé.
pub(crate) fn connect(path: &Path, deadline: Instant) -> io::Result<(SocketLines, SocketWriter)> {
    let stream = connect_nonblocking(path, deadline)?;
    stream.set_nonblocking(true)?;
    let fd = stream.as_raw_fd();
    let mut result = client_with_config(
        "ws://localhost/",
        stream,
        Some(config(MAX_DAEMON_FRAME_BYTES)),
    );
    loop {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "négociation Codex expirée",
            ));
        }
        match result {
            Ok((socket, _)) => return Ok(split(socket)),
            Err(HandshakeError::Failure(err)) => return Err(error(err)),
            Err(HandshakeError::Interrupted(handshake)) => {
                // L'Upgrade est minuscule ; attendre IN ou une courte tranche
                // évite de boucler sur POLLOUT pendant l'attente de sa réponse.
                poll(fd, libc::POLLIN, deadline)?;
                result = handshake.handshake();
            }
        }
    }
}

fn split(socket: WebSocket<UnixStream>) -> (SocketLines, SocketWriter) {
    let shared = Arc::new(Mutex::new(socket));
    (
        SocketLines {
            shared: shared.clone(),
            ended: false,
        },
        SocketWriter {
            shared,
            bytes: Vec::new(),
        },
    )
}

fn poll(fd: RawFd, events: i16, deadline: Instant) -> io::Result<()> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .filter(|value| !value.is_zero())
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "budget socket Codex dépassé"))?;
    let millis = remaining.as_millis().clamp(1, 25) as i32;
    let mut descriptor = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    if unsafe { libc::poll(&mut descriptor, 1, millis) } < 0 {
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    Ok(())
}

impl Iterator for SocketLines {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.ended {
            // Socket NON bloquante : le verrou n'est jamais gardé pendant poll.
            // Le writer peut donc envoyer un RPC pendant une lecture inactive.
            let (result, fd) = {
                let mut socket = self.shared.lock().unwrap_or_else(|e| e.into_inner());
                (socket.read(), socket.get_ref().as_raw_fd())
            };
            match result {
                Ok(Message::Text(text)) => return Some(Ok(text.to_string())),
                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                Ok(Message::Close(_)) | Err(Error::ConnectionClosed | Error::AlreadyClosed) => {
                    self.ended = true;
                }
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {
                    if let Err(err) = poll(fd, libc::POLLIN, Instant::now() + IO_BUDGET) {
                        self.ended = true;
                        return Some(Err(err));
                    }
                }
                Err(err) => {
                    self.ended = true;
                    return Some(Err(error(err)));
                }
                _ => {
                    self.ended = true;
                    return Some(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Codex doit émettre une trame texte",
                    )));
                }
            }
        }
        None
    }
}

impl Write for SocketWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > MAX_DAEMON_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trame Codex trop grande",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.bytes.is_empty() {
            return Ok(());
        }
        let mut bytes = std::mem::take(&mut self.bytes);
        // Seul le LF de cadrage du writer JSONL est retiré, pas les espaces
        // ni les LF à l'intérieur d'un objet JSON mis en forme.
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
        }
        let text =
            String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let deadline = Instant::now() + IO_BUDGET;
        let mut socket = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        match socket.send(Message::text(text)) {
            Ok(()) => return Ok(()),
            // tungstenite conserve les bytes en attente : NE PAS renvoyer le
            // message. Seul flush est rejoué, jusqu'à la même échéance.
            Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
            Err(err) => return Err(error(err)),
        }
        loop {
            poll(socket.get_ref().as_raw_fd(), libc::POLLOUT, deadline)?;
            match socket.flush() {
                Ok(()) => return Ok(()),
                Err(Error::Io(err)) if err.kind() == io::ErrorKind::WouldBlock => {}
                Err(err) => return Err(error(err)),
            }
        }
    }
}

impl Drop for SocketWriter {
    fn drop(&mut self) {
        let socket = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        let _ = socket.get_ref().shutdown(Shutdown::Both);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::DirBuilderExt;
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc;
    use std::thread;
    use tungstenite::protocol::Role;

    #[test]
    fn socket_privee_refuse_parent_public_lien_meme_pendant_et_chemin_long() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let root = Path::new("/tmp").join(format!("b90-private-{}", std::process::id()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let socket = root.join("s");
        assert!(validate_private_path(&socket).is_ok());
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            validate_private_path(&socket).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        symlink(root.join("absent"), &socket).unwrap();
        assert_eq!(
            validate_private_path(&socket).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            validate_private_path(&root.join("x".repeat(120)))
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        std::fs::remove_file(socket).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn upgrade_reel_et_budget_du_pair_silencieux() {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let root = Path::new("/tmp").join(format!(
            "b90ws-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        // /tmp court : le TMPDIR macOS peut dépasser SUN_LEN.
        let path = root.join("s");
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut ws = tungstenite::accept(stream).unwrap();
            ws.send(Message::text("{ \"event\": 1 }")).unwrap();
        });
        let (mut reader, writer) = connect(&path, Instant::now() + Duration::from_secs(2)).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), "{ \"event\": 1 }");
        drop(writer);
        server.join().unwrap();
        std::fs::remove_file(&path).unwrap();
        let listener = UnixListener::bind(&path).unwrap();
        let (release, held) = mpsc::channel();
        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            held.recv_timeout(Duration::from_secs(3)).unwrap();
        });
        let deadline = Instant::now() + Duration::from_millis(100);
        assert!(
            matches!(connect(&path, deadline), Err(err) if err.kind() == io::ErrorKind::TimedOut)
        );
        release.send(()).unwrap();
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    fn pair(maximum: usize) -> (SocketLines, SocketWriter, WebSocket<UnixStream>) {
        let (left, right) = UnixStream::pair().unwrap();
        left.set_nonblocking(true).unwrap();
        right
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let (reader, writer) = split(WebSocket::from_raw_socket(
            left,
            Role::Client,
            Some(config(maximum)),
        ));
        (
            reader,
            writer,
            WebSocket::from_raw_socket(right, Role::Server, None),
        )
    }

    #[test]
    fn lecture_inactive_ne_bloque_pas_le_rpc_et_raw_reste_exact() {
        let (mut reader, mut writer, mut server) = pair(4096);
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            tx.send(reader.next().unwrap().unwrap()).unwrap();
        });
        let rpc = "{  \"id\":1,\"method\":\"ping\" }";
        writeln!(writer, "{rpc}").unwrap();
        writer.flush().unwrap();
        assert_eq!(server.read().unwrap().into_text().unwrap(), rpc);
        let raw = "{ \"method\": \"future/event\",\n \"params\": {\"future\":42}  }";
        server.send(Message::text(raw)).unwrap();
        // Mutation : ré-sérialiser le JSON fait échouer cet oracle d'octets.
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap().as_bytes(),
            raw.as_bytes()
        );
        worker.join().unwrap();
    }

    #[test]
    fn borne_et_binaire_sont_refuses_sans_troncature() {
        let (mut reader, _writer, mut server) = pair(64);
        server.send(Message::text("x".repeat(65))).unwrap();
        assert_eq!(
            reader.next().unwrap().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(reader.next().is_none());
        let (mut reader, _writer, mut server) = pair(64);
        server.send(Message::Binary(vec![1, 2, 3].into())).unwrap();
        assert_eq!(
            reader.next().unwrap().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn fermeture_writer_reveille_la_lecture_sans_attendre_un_evenement() {
        let (mut reader, writer, _server) = pair(64);
        let (tx, rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            tx.send(reader.next()).unwrap();
        });
        drop(writer);
        assert!(rx.recv_timeout(Duration::from_secs(2)).is_ok());
        worker.join().unwrap();
    }
}
