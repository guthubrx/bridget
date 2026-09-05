//! Frontière JSONL Unix partagée. LF fait partie de la borne ; aucune donnée
//! partielle n'est un message. L'attente entre deux messages n'est pas le délai
//! de lecture d'une trame : un wrapper peut rester légitimement inactif.

use std::io::{self, BufRead, BufReader};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

pub const MAX_DAEMON_FRAME_BYTES: usize = 16 * 1024 * 1024;

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
