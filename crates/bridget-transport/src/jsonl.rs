//! Frontière JSONL Unix partagée. LF fait partie de la borne ; aucune donnée
//! partielle n'est un message. L'attente entre deux messages n'est pas le délai
//! de lecture d'une trame : un wrapper peut rester légitimement inactif.

use std::io::{self, BufRead, BufReader};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::{Duration, Instant};

pub const MAX_DAEMON_FRAME_BYTES: usize = 16 * 1024 * 1024;

pub fn validate_unix_socket_path(socket: &Path) -> io::Result<()> {
    let path = socket.as_os_str().as_bytes();
    let address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path.is_empty() || path.contains(&0) || path.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "chemin socket Unix vide, NUL ou trop long",
        ));
    }
    Ok(())
}

/// Connexion Unix sous échéance absolue, partagée par les clients et pilotes.
pub fn connect_nonblocking(socket: &Path, deadline: Instant) -> io::Result<UnixStream> {
    if deadline <= Instant::now() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "budget connexion dépassé",
        ));
    }
    let path = socket.as_os_str().as_bytes();
    validate_unix_socket_path(socket)?;
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
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
    // Propriétaire dès l'ouverture : toute sortie par ? ferme le descripteur,
    // y compris l'expiration entre connect et poll (ancien chemin de fuite).
    let owned = unsafe { OwnedFd::from_raw_fd(fd) };
    let fd = owned.as_raw_fd();
    if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
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
            return Err(error);
        }
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|d| !d.is_zero())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::TimedOut, "budget connexion dépassé")
                })?;
            let timeout = remaining.as_millis().max(1).min(i32::MAX as u128) as libc::c_int;
            let ready = unsafe { libc::poll(&mut pollfd, 1, timeout) };
            if ready > 0 {
                break;
            }
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
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
        {
            return Err(io::Error::last_os_error());
        }
        if so_error != 0 {
            return Err(io::Error::from_raw_os_error(so_error));
        }
    }
    if Instant::now() >= deadline {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "budget connexion dépassé",
        ));
    }
    let stream = UnixStream::from(owned);
    stream.set_nonblocking(false)?;
    Ok(stream)
}

#[derive(Clone, Copy)]
pub enum LineDeadline {
    Absolute(Instant),
    /// Aucune échéance d'inactivité ; le budget commence au premier octet.
    AfterFirstByte(Duration),
}

/// O(taille), allocation au plus `maximum` octets. Retourne le LF intact pour
/// permettre aux sous-protocoles plus stricts de vérifier LEURS propres bornes.
pub fn read_unix_line(
    reader: &mut BufReader<UnixStream>,
    maximum: usize,
    policy: LineDeadline,
) -> io::Result<Option<Vec<u8>>> {
    if maximum == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "borne JSONL nulle",
        ));
    }
    let mut deadline = match policy {
        LineDeadline::Absolute(value) => Some(value),
        LineDeadline::AfterFirstByte(_) => None,
    };
    let mut frame = Vec::new();
    loop {
        let timeout = deadline
            .map(|value| {
                value
                    .checked_duration_since(Instant::now())
                    .filter(|d| !d.is_zero())
                    .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "budget JSONL dépassé"))
            })
            .transpose()?;
        // Un SEUL lecteur possède la socket. poll avant chaque remplissage
        // borne le read sans setsockopt : macOS refuse SO_RCVTIMEO avec EINVAL
        // après fermeture du pair, même lorsque des bytes restent à drainer.
        if reader.buffer().is_empty() {
            let millis = timeout.map_or(-1, |d| {
                d.as_nanos().div_ceil(1_000_000).min(i32::MAX as u128) as i32
            });
            let mut fd = libc::pollfd {
                fd: reader.get_ref().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ready = unsafe { libc::poll(&mut fd, 1, millis) };
            if ready == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "budget JSONL dépassé",
                ));
            }
            if ready < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
        }
        let bytes = match reader.fill_buf() {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if bytes.is_empty() {
            return if frame.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "trame sans LF",
                ))
            };
        }
        if deadline.is_none()
            && let LineDeadline::AfterFirstByte(budget) = policy
        {
            deadline = Some(Instant::now() + budget);
        }
        let end = bytes.iter().position(|b| *b == b'\n').map(|i| i + 1);
        let count = end.unwrap_or(bytes.len());
        if count > maximum.saturating_sub(frame.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "trame trop grande, LF inclus",
            ));
        }
        let needed = frame.len() + count;
        if needed > frame.capacity() {
            let capacity = frame
                .capacity()
                .max(8192)
                .saturating_mul(2)
                .min(maximum)
                .max(needed);
            frame.reserve_exact(capacity - frame.len());
        }
        frame.extend_from_slice(&bytes[..count]);
        reader.consume(count);
        if deadline.is_some_and(|value| Instant::now() >= value) {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "budget JSONL dépassé",
            ));
        }
        if end.is_some() {
            return Ok(Some(frame));
        }
        if frame.len() == maximum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "borne atteinte sans LF",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn borne_lf_inclus_et_deux_trames_preservees() {
        for size in [7, 8, 9] {
            let (local, mut peer) = UnixStream::pair().unwrap();
            let mut bytes = vec![b'x'; size - 1];
            bytes.push(b'\n');
            peer.write_all(&bytes).unwrap();
            peer.write_all(b"ok\n").unwrap();
            let mut reader = BufReader::new(local);
            let result = read_unix_line(
                &mut reader,
                8,
                LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)),
            );
            if size <= 8 {
                assert_eq!(result.unwrap(), Some(bytes));
                assert_eq!(
                    read_unix_line(
                        &mut reader,
                        8,
                        LineDeadline::AfterFirstByte(Duration::from_secs(1))
                    )
                    .unwrap(),
                    Some(b"ok\n".to_vec())
                );
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
            }
        }
    }

    #[test]
    fn eof_propre_distinct_d_une_trame_partielle() {
        for body in [b"".as_slice(), b"partiel"] {
            let (local, mut peer) = UnixStream::pair().unwrap();
            peer.write_all(body).unwrap();
            drop(peer);
            let result = read_unix_line(
                &mut BufReader::new(local),
                1024,
                LineDeadline::Absolute(Instant::now() + Duration::from_secs(1)),
            );
            if body.is_empty() {
                assert_eq!(result.unwrap(), None);
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
            }
        }
    }
}
