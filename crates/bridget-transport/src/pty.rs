//! Transport pseudo-terminal — remet un message en l'écrivant dans le maître
//! d'un PTY possédé par le wrapper, comme une saisie humaine collée.
//!
//! Le wrapper relaie déjà la frappe de l'humain vers ce même maître ; une
//! remise Bridget n'est qu'une écriture de plus, encadrée en collage
//! (`ESC[200~ … ESC[201~`) pour que les sauts de ligne du corps n'entraînent
//! pas de soumission anticipée, puis soumise par un retour chariot. Aucune
//! lecture de l'écran : le succès de l'écriture est l'observable, comme pour
//! la voie tmux.

use crate::tmux::validate_injection_content;
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, envelope::wrap_envelope};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Début et fin de collage encadré, tels que les envoie un terminal.
pub const BRACKETED_PASTE_START: &[u8] = b"\x1b[200~";
pub const BRACKETED_PASTE_END: &[u8] = b"\x1b[201~";

/// Délai laissé à l'interface pour digérer le collage avant le retour
/// chariot. tmux sonde l'écran jusqu'à 1,5 s ; sans écran lisible ici, un
/// délai fixe et court suffit à séparer les deux écritures.
const DIGESTION_DELAY: Duration = Duration::from_millis(150);

/// Borne d'attente d'un maître non accueillant (enfant stoppé, file pleine).
/// Au-delà, la remise est un échec nommé, jamais un blocage du wrapper.
const WRITE_DEADLINE: Duration = Duration::from_secs(5);

/// Taille des tranches écrites entre deux sondages d'écriture possible.
const WRITE_CHUNK: usize = 256;

/// Transport PTY : écrit dans le maître d'un pseudo-terminal.
pub struct PtyTransport {
    master: Arc<Mutex<OwnedFd>>,
    agent_pid: u32,
    connection_id: String,
}

impl PtyTransport {
    /// Duplique le descripteur maître : le transport possède sa copie et ne
    /// ferme jamais celle du relais.
    pub fn from_master(master: RawFd, agent_pid: u32) -> Result<Self, TransportError> {
        // Copie fermée à l'exec : le fournisseur ne doit jamais hériter du
        // maître de son propre terminal (audit 097, SEC-005).
        let duplicate = unsafe { libc::fcntl(master, libc::F_DUPFD_CLOEXEC, 0) };
        if duplicate < 0 {
            return Err(TransportError::Io(format!(
                "dup du maître PTY: {}",
                std::io::Error::last_os_error()
            )));
        }
        let owned = unsafe { OwnedFd::from_raw_fd(duplicate) };
        // Non bloquant : `poll` peut annoncer un maître accueillant alors que
        // la file du terminal se remplit en cours d'écriture ; sans ce drapeau
        // l'écriture bloquerait le wrapper malgré l'échéance. Le drapeau est
        // partagé avec la description ouverte du relais, qui le tolère.
        let flags = unsafe { libc::fcntl(duplicate, libc::F_GETFL) };
        if flags < 0
            || unsafe { libc::fcntl(duplicate, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return Err(TransportError::Io(format!(
                "O_NONBLOCK sur le maître PTY: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(Self {
            master: Arc::new(Mutex::new(owned)),
            agent_pid,
            connection_id: format!("pty-{agent_pid}"),
        })
    }

    /// Écrit tout `bytes`, par tranches, en sondant l'accueil du maître avant
    /// chaque tranche. O(n) sur la taille du message.
    fn write_all_bounded(fd: RawFd, bytes: &[u8], deadline: Instant) -> Result<(), TransportError> {
        let mut offset = 0;
        while offset < bytes.len() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(TransportError::DeliveryFailed(
                    "maître PTY non accueillant : délai d'écriture dépassé".to_string(),
                ));
            }
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLOUT,
                revents: 0,
            };
            let timeout = remaining.as_millis().min(i32::MAX as u128) as i32;
            let ready = unsafe { libc::poll(&mut poll, 1, timeout) };
            if ready < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(TransportError::Io(format!("poll maître PTY: {error}")));
            }
            if ready == 0 {
                continue;
            }
            if poll.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                return Err(TransportError::AgentDead);
            }
            let end = (offset + WRITE_CHUNK).min(bytes.len());
            let chunk = &bytes[offset..end];
            let written =
                unsafe { libc::write(fd, chunk.as_ptr() as *const libc::c_void, chunk.len()) };
            if written < 0 {
                let error = std::io::Error::last_os_error();
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                ) {
                    continue;
                }
                return Err(TransportError::Io(format!("écriture maître PTY: {error}")));
            }
            offset += written as usize;
        }
        Ok(())
    }
}

impl Transport for PtyTransport {
    fn deliver(&mut self, msg: &BridgetMessage) -> Result<(), TransportError> {
        if !self.is_alive() {
            return Err(TransportError::AgentDead);
        }
        let envelope = wrap_envelope(msg);
        validate_injection_content(&envelope)?;

        let master = self
            .master
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let fd = master.as_raw_fd();
        let deadline = Instant::now() + WRITE_DEADLINE;
        Self::write_all_bounded(fd, BRACKETED_PASTE_START, deadline)?;
        Self::write_all_bounded(fd, envelope.as_bytes(), deadline)?;
        Self::write_all_bounded(fd, BRACKETED_PASTE_END, deadline)?;
        std::thread::sleep(DIGESTION_DELAY);
        if !self.is_alive() {
            return Err(TransportError::AgentDead);
        }
        Self::write_all_bounded(fd, b"\r", Instant::now() + WRITE_DEADLINE)
    }

    fn is_alive(&self) -> bool {
        unsafe { libc::kill(self.agent_pid as i32, 0) == 0 }
    }

    fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    struct Pty {
        master: OwnedFd,
        slave: OwnedFd,
    }

    fn open_pty() -> Pty {
        let (mut master, mut slave) = (-1, -1);
        let rc = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(rc, 0, "openpty: {}", std::io::Error::last_os_error());
        // Esclave en mode brut : ce que le maître écrit arrive octet pour
        // octet, sans écho ni traduction de ligne.
        let mut attributes = std::mem::MaybeUninit::<libc::termios>::uninit();
        assert_eq!(
            unsafe { libc::tcgetattr(slave, attributes.as_mut_ptr()) },
            0
        );
        let mut raw = unsafe { attributes.assume_init() };
        unsafe { libc::cfmakeraw(&mut raw) };
        assert_eq!(unsafe { libc::tcsetattr(slave, libc::TCSANOW, &raw) }, 0);
        unsafe {
            Pty {
                master: OwnedFd::from_raw_fd(master),
                slave: OwnedFd::from_raw_fd(slave),
            }
        }
    }

    fn read_available(fd: RawFd, wait: Duration) -> Vec<u8> {
        let mut collected = Vec::new();
        let deadline = Instant::now() + wait;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut poll, 1, remaining.as_millis() as i32) };
            if ready <= 0 {
                break;
            }
            let mut buffer = [0u8; 1024];
            let mut file = unsafe { std::fs::File::from_raw_fd(libc::dup(fd)) };
            match file.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => collected.extend_from_slice(&buffer[..n]),
            }
            if remaining.is_zero() {
                break;
            }
        }
        collected
    }

    fn message(body: &str) -> BridgetMessage {
        BridgetMessage::new("expediteur", "destinataire", body)
    }

    #[test]
    fn remise_ecrit_le_collage_encadre_puis_le_retour_chariot() {
        let pty = open_pty();
        let mut transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        let msg = message("Première ligne\nDeuxième ligne");
        transport.deliver(&msg).unwrap();
        let received = read_available(pty.slave.as_raw_fd(), Duration::from_millis(600));
        let mut expected = Vec::new();
        expected.extend_from_slice(BRACKETED_PASTE_START);
        expected.extend_from_slice(wrap_envelope(&msg).as_bytes());
        expected.extend_from_slice(BRACKETED_PASTE_END);
        expected.push(b'\r');
        assert_eq!(
            received, expected,
            "octets exacts : collage encadré puis CR"
        );
        assert_eq!(
            transport.connection_id(),
            format!("pty-{}", std::process::id())
        );
    }

    #[test]
    fn le_retour_chariot_n_arrive_qu_apres_le_delai_de_digestion() {
        let pty = open_pty();
        let mut transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        let slave = pty.slave.as_raw_fd();
        let handle = std::thread::spawn(move || {
            // Lecture courte : le collage est là, le CR pas encore.
            read_available(slave, Duration::from_millis(60))
        });
        transport.deliver(&message("corps")).unwrap();
        let early = handle.join().unwrap();
        assert!(early.ends_with(BRACKETED_PASTE_END), "{early:?}");
        let late = read_available(pty.slave.as_raw_fd(), Duration::from_millis(300));
        assert_eq!(late, b"\r");
    }

    #[test]
    fn corps_avec_sequence_d_echappement_n_ecrit_rien() {
        let pty = open_pty();
        let mut transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        let error = transport
            .deliver(&message("hostile \x1b]52;c;secret\x07"))
            .unwrap_err();
        assert!(
            matches!(error, TransportError::DeliveryFailed(_)),
            "{error:?}"
        );
        let received = read_available(pty.slave.as_raw_fd(), Duration::from_millis(100));
        assert!(
            received.is_empty(),
            "aucun octet ne doit partir : {received:?}"
        );
    }

    #[test]
    fn enfant_termine_rend_agent_dead_sans_ecriture() {
        let pty = open_pty();
        let mut child = std::process::Command::new("true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        let mut transport = PtyTransport::from_master(pty.master.as_raw_fd(), pid).unwrap();
        assert!(!transport.is_alive());
        assert!(matches!(
            transport.deliver(&message("corps")),
            Err(TransportError::AgentDead)
        ));
        let received = read_available(pty.slave.as_raw_fd(), Duration::from_millis(100));
        assert!(received.is_empty());
    }

    #[test]
    fn maitre_non_accueillant_echoue_dans_le_delai_sans_bloquer() {
        // Personne ne lit l'esclave : la file du PTY se remplit, l'écriture
        // doit rendre un échec nommé avant l'échéance, jamais bloquer le wrapper.
        let pty = open_pty();
        let transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        let master = transport.master.lock().unwrap().as_raw_fd();
        let payload = vec![b'x'; 1 << 20];
        let started = Instant::now();
        let deadline = started + Duration::from_millis(300);
        let error = PtyTransport::write_all_bounded(master, &payload, deadline).unwrap_err();
        assert!(
            matches!(error, TransportError::DeliveryFailed(ref detail) if detail.contains("délai")),
            "{error:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "l'échec doit suivre l'échéance"
        );
    }

    #[test]
    fn la_copie_du_maitre_est_fermee_a_l_exec() {
        let pty = open_pty();
        let transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        let fd = transport.master.lock().unwrap().as_raw_fd();
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert!(
            flags >= 0 && flags & libc::FD_CLOEXEC != 0,
            "FD_CLOEXEC attendu : {flags}"
        );
    }

    #[test]
    fn le_transport_possede_sa_copie_du_maitre() {
        let pty = open_pty();
        let transport =
            PtyTransport::from_master(pty.master.as_raw_fd(), std::process::id()).unwrap();
        drop(pty.master);
        // Le maître original fermé, la copie du transport reste écrivable.
        let fd = transport.master.lock().unwrap().as_raw_fd();
        PtyTransport::write_all_bounded(fd, b"x", Instant::now() + Duration::from_secs(1))
            .expect("copie du maître écrivable");
        assert_eq!(
            read_available(pty.slave.as_raw_fd(), Duration::from_millis(100)),
            b"x"
        );
    }
}
