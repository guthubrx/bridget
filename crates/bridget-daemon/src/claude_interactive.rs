//! Session Claude Code interactive sans tmux.
//!
//! Le wrapper possède un pseudo-terminal : il y lance le fournisseur avec ce
//! PTY pour terminal de contrôle, met le terminal de l'humain en mode brut,
//! relaie frappe, affichage et taille de fenêtre dans les deux sens, puis
//! restaure le terminal à la sortie. Les messages Bridget passent par ce même
//! maître (`bridget_transport::PtyTransport`). Aucun octet relayé n'est
//! interprété ni journalisé ici. Complexité : relais O(octets), une tranche
//! de tampon par sens.

use std::io::{self, IsTerminal};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Protocole publié dans la présence : nomme le canal réel, jamais tmux.
pub(crate) const PROTOCOL: &str = "claude_pty";

/// Période de la boucle de signaux (taille de fenêtre, arrêt demandé).
const SIGNAL_TICK: Duration = Duration::from_millis(50);

/// Tranche de relais entre les deux terminaux.
const RELAY_CHUNK: usize = 16 * 1024;

pub(crate) fn check_terminal() -> Result<(), String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(
            "Claude interactif exige stdin et stdout sur un terminal ; utilisez bridget spawn claude pour un agent détaché"
                .into(),
        );
    }
    Ok(())
}

fn os_error(context: &str) -> String {
    format!("{context}: {}", io::Error::last_os_error())
}

/// Terminal de l'humain en mode brut complet, restauré exactement une fois :
/// à la fin normale, sur signal et sur panique (`Drop`).
struct RawTerminal {
    fd: RawFd,
    original: libc::termios,
    restored: bool,
}

impl RawTerminal {
    fn enable(fd: RawFd) -> Result<Self, String> {
        let mut original = MaybeUninit::<libc::termios>::uninit();
        if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
            return Err(os_error("lecture termios du terminal"));
        }
        let original = unsafe { original.assume_init() };
        let mut raw = original;
        unsafe { libc::cfmakeraw(&mut raw) };
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            return Err(os_error("mode brut du terminal"));
        }
        Ok(Self {
            fd,
            original,
            restored: false,
        })
    }

    fn restore(&mut self) {
        if self.restored {
            return;
        }
        self.restored = true;
        unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &self.original) };
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        self.restore();
    }
}

fn set_window_size(fd: RawFd, rows: u16, cols: u16) {
    let size = libc::winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &size) };
}

fn open_pty() -> Result<(OwnedFd, OwnedFd), String> {
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
    if rc != 0 {
        return Err(os_error("ouverture du pseudo-terminal"));
    }
    // Ni le maître ni l'esclave d'origine ne traversent l'exec : l'enfant ne
    // reçoit que les copies posées sur son stdio (audit 097, SEC-005).
    for fd in [master, slave] {
        if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
            return Err(os_error("FD_CLOEXEC sur le pseudo-terminal"));
        }
    }
    Ok(unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) })
}

/// Copie `from` vers `to` jusqu'à EOF ou erreur ; retourne à l'arrêt demandé.
/// Attend qu'un descripteur soit prêt : le maître PTY devient non bloquant
/// dès que le transport de remise est ouvert (même description ouverte).
fn wait_ready(fd: RawFd, events: libc::c_short) {
    let mut poll = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    unsafe { libc::poll(&mut poll, 1, SIGNAL_TICK.as_millis() as libc::c_int) };
}

fn relay(from: RawFd, to: RawFd, stopping: Arc<AtomicBool>) {
    let mut buffer = vec![0u8; RELAY_CHUNK];
    loop {
        let read =
            unsafe { libc::read(from, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len()) };
        if read < 0 {
            let error = io::Error::last_os_error();
            if stopping.load(Ordering::SeqCst) {
                return;
            }
            match error.kind() {
                io::ErrorKind::Interrupted => continue,
                io::ErrorKind::WouldBlock => {
                    wait_ready(from, libc::POLLIN);
                    continue;
                }
                _ => return,
            }
        }
        if read == 0 {
            return;
        }
        let mut offset = 0usize;
        while offset < read as usize {
            let written = unsafe {
                libc::write(
                    to,
                    buffer[offset..].as_ptr() as *const libc::c_void,
                    read as usize - offset,
                )
            };
            if written < 0 {
                match io::Error::last_os_error().kind() {
                    io::ErrorKind::Interrupted => continue,
                    io::ErrorKind::WouldBlock => {
                        wait_ready(to, libc::POLLOUT);
                        continue;
                    }
                    _ => return,
                }
            }
            offset += written as usize;
        }
    }
}

struct Signals(Vec<signal_hook::SigId>);

impl Drop for Signals {
    fn drop(&mut self) {
        for id in self.0.drain(..) {
            signal_hook::low_level::unregister(id);
        }
    }
}

/// Le fournisseur, son pseudo-terminal et le terminal de l'humain.
pub(crate) struct PtySession {
    master: OwnedFd,
    child: Child,
    terminal: RawTerminal,
    stopping: Arc<AtomicBool>,
    output_relay: Option<JoinHandle<()>>,
    signal_loop: Option<JoinHandle<()>>,
    _signals: Signals,
}

impl PtySession {
    /// Ouvre le PTY à la taille du terminal courant, lance `command` dedans
    /// comme chef de session avec ce PTY pour terminal de contrôle, puis
    /// démarre les relais. Un échec avant le retour ne laisse ni enfant ni
    /// terminal modifié.
    pub(crate) fn spawn(mut command: Command) -> Result<Self, String> {
        let stdin = libc::STDIN_FILENO;
        let stdout = libc::STDOUT_FILENO;
        let (master, slave) = open_pty()?;
        if let Some((columns, rows)) = crate::attach::terminal_geometry(stdout) {
            set_window_size(slave.as_raw_fd(), rows as u16, columns as u16);
        }
        let slave_fd = slave.as_raw_fd();
        let stdio = |fd: RawFd| -> Result<Stdio, String> {
            let duplicate = unsafe { libc::dup(fd) };
            if duplicate < 0 {
                return Err(os_error("dup de l'esclave PTY"));
            }
            Ok(unsafe { Stdio::from_raw_fd(duplicate) })
        };
        command
            .stdin(stdio(slave_fd)?)
            .stdout(stdio(slave_fd)?)
            .stderr(stdio(slave_fd)?);
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::ioctl(libc::STDIN_FILENO, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn().map_err(|error| {
            format!("lancement du fournisseur dans le pseudo-terminal: {error}")
        })?;
        drop(slave);
        let terminal = RawTerminal::enable(stdin)?;

        let stopping = Arc::new(AtomicBool::new(false));
        let master_fd = master.as_raw_fd();
        let input_stop = stopping.clone();
        let child_pid = child.id() as libc::pid_t;
        thread::Builder::new()
            .name("claude-pty-input".into())
            .spawn(move || {
                relay(stdin, master_fd, input_stop.clone());
                // Le terminal de l'humain s'est fermé : le fournisseur reçoit
                // le même signal qu'avec une fenêtre refermée.
                if !input_stop.load(Ordering::SeqCst) {
                    unsafe { libc::kill(child_pid, libc::SIGHUP) };
                }
            })
            .map_err(|error| format!("relais de saisie: {error}"))?;
        let output_stop = stopping.clone();
        let output_relay = thread::Builder::new()
            .name("claude-pty-output".into())
            .spawn(move || relay(master_fd, stdout, output_stop))
            .map_err(|error| format!("relais d'affichage: {error}"))?;

        let resized = Arc::new(AtomicBool::new(false));
        let terminate = Arc::new(AtomicBool::new(false));
        let hangup = Arc::new(AtomicBool::new(false));
        let mut ids = Vec::new();
        for (signal, flag) in [
            (libc::SIGWINCH, &resized),
            (libc::SIGTERM, &terminate),
            (libc::SIGINT, &terminate),
            (libc::SIGHUP, &hangup),
        ] {
            let id = signal_hook::flag::register(signal, flag.clone())
                .map_err(|error| format!("signal {signal}: {error}"))?;
            ids.push(id);
        }
        let signal_stop = stopping.clone();
        let signal_loop = thread::Builder::new()
            .name("claude-pty-signals".into())
            .spawn(move || {
                while !signal_stop.load(Ordering::SeqCst) {
                    if resized.swap(false, Ordering::SeqCst)
                        && let Some((columns, rows)) = crate::attach::terminal_geometry(stdout)
                    {
                        set_window_size(master_fd, rows as u16, columns as u16);
                    }
                    if terminate.swap(false, Ordering::SeqCst) {
                        unsafe { libc::kill(child_pid, libc::SIGTERM) };
                    }
                    if hangup.swap(false, Ordering::SeqCst) {
                        unsafe { libc::kill(child_pid, libc::SIGHUP) };
                    }
                    thread::sleep(SIGNAL_TICK);
                }
            })
            .map_err(|error| format!("boucle de signaux: {error}"))?;

        Ok(Self {
            master,
            child,
            terminal,
            stopping,
            output_relay: Some(output_relay),
            signal_loop: Some(signal_loop),
            _signals: Signals(ids),
        })
    }

    pub(crate) fn child_id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn master_fd(&self) -> RawFd {
        self.master.as_raw_fd()
    }

    /// Attend la fin du fournisseur, vide l'affichage restant, restaure le
    /// terminal et retourne le code de sortie à relayer.
    pub(crate) fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait();
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(relay) = self.output_relay.take() {
            let _ = relay.join();
        }
        if let Some(signals) = self.signal_loop.take() {
            let _ = signals.join();
        }
        self.terminal.restore();
        status
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        self.terminal.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_protocole_nomme_le_canal_reel() {
        assert_eq!(PROTOCOL, "claude_pty");
        assert_ne!(PROTOCOL, "tmux");
    }

    #[test]
    fn check_terminal_refuse_un_pipe() {
        // Sous cargo test, stdin n'est pas un terminal : le refus est nommé.
        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            return;
        }
        let error = check_terminal().unwrap_err();
        assert!(error.contains("terminal"), "{error}");
        assert!(error.contains("bridget spawn claude"), "{error}");
    }

    #[test]
    fn le_pty_ouvert_ne_traverse_pas_l_exec() {
        let (master, slave) = open_pty().unwrap();
        for fd in [master.as_raw_fd(), slave.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            assert!(
                flags >= 0 && flags & libc::FD_CLOEXEC != 0,
                "FD_CLOEXEC attendu : {flags}"
            );
        }
    }

    #[test]
    fn le_pty_ouvert_porte_la_taille_demandee() {
        let (master, slave) = open_pty().unwrap();
        set_window_size(slave.as_raw_fd(), 41, 99);
        assert_eq!(
            crate::attach::terminal_geometry(master.as_raw_fd()),
            Some((99, 41))
        );
    }
}
