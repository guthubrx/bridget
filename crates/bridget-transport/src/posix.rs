//! Wrappers POSIX sûrs et bornés pour centraliser les appels système récurrents
//! (`fcntl`, `poll`, `flock`, `pipe`) sans disperser de blocs `unsafe` redondants.

use std::io;
use std::os::unix::io::RawFd;

/// Active ou désactive `FD_CLOEXEC` sur un descripteur de fichier ouvert.
pub fn set_cloexec(fd: RawFd, enabled: bool) -> io::Result<()> {
    // SAFETY: `fcntl(F_GETFD/F_SETFD)` opère uniquement sur des drapeaux entiers sans pointeur.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let next = if enabled {
        flags | libc::FD_CLOEXEC
    } else {
        flags & !libc::FD_CLOEXEC
    };
    // SAFETY: `fcntl(F_SETFD)` opère uniquement sur des entiers sans déréférencement de pointeur.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, next) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Active ou désactive `O_NONBLOCK` sur un descripteur ouvert.
pub fn set_nonblocking(fd: RawFd, nonblocking: bool) -> io::Result<()> {
    // SAFETY: `fcntl(F_GETFL)` ne déréférence aucun pointeur.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    let next = if nonblocking {
        flags | libc::O_NONBLOCK
    } else {
        flags & !libc::O_NONBLOCK
    };
    // SAFETY: `fcntl(F_SETFL)` ne déréférence aucun pointeur.
    if unsafe { libc::fcntl(fd, libc::F_SETFL, next) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Pose un verrou exclusif non bloquant (`LOCK_EX | LOCK_NB`) sur un descripteur ouvert.
pub fn flock_exclusive_nonblocking(fd: RawFd) -> io::Result<bool> {
    // SAFETY: `flock(2)` prend uniquement deux entiers (descripteur et masque d'opération).
    if unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0 {
        Ok(true)
    } else {
        let err = io::Error::last_os_error();
        if err.kind() == io::ErrorKind::WouldBlock {
            Ok(false)
        } else {
            Err(err)
        }
    }
}

/// Attend des événements sur une tranche de `libc::pollfd` avec délai en millisecondes.
pub fn poll_fds(fds: &mut [libc::pollfd], timeout_ms: libc::c_int) -> io::Result<usize> {
    // SAFETY: `fds.as_mut_ptr()` est un tableau contigu valide de `fds.len()` `pollfd`
    // pour toute la durée de l'appel `poll(2)`.
    let rc = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms) };
    if rc < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(rc as usize)
    }
}

/// Ouvre un tube anonyme configuré en `FD_CLOEXEC | O_NONBLOCK` sur ses deux extrémités.
pub fn pipe_cloexec_nonblocking() -> io::Result<(RawFd, RawFd)> {
    let mut fds = [-1i32; 2];
    // SAFETY: `fds` est un tableau mutable de 2 `c_int` valide sur la pile.
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    for fd in fds {
        if let Err(err) = set_cloexec(fd, true).and_then(|()| set_nonblocking(fd, true)) {
            // SAFETY: les deux descripteurs appartiennent à cette fonction avant tout retour.
            unsafe {
                libc::close(fds[0]);
                libc::close(fds[1]);
            }
            return Err(err);
        }
    }
    Ok((fds[0], fds[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_and_poll_work_without_blocking() {
        let (r, w) = pipe_cloexec_nonblocking().unwrap();
        let mut pfd = [libc::pollfd {
            fd: r,
            events: libc::POLLIN,
            revents: 0,
        }];
        assert_eq!(poll_fds(&mut pfd, 0).unwrap(), 0);
        let byte = 1u8;
        // SAFETY: `w` est un descripteur de pipe ouvert appartenant à ce test et `&byte` est valide.
        let written = unsafe { libc::write(w, (&byte as *const u8).cast(), 1) };
        assert_eq!(written, 1);
        assert_eq!(poll_fds(&mut pfd, 50).unwrap(), 1);
        // SAFETY: `r` et `w` appartiennent exclusivement à ce test et ne sont plus utilisés après.
        unsafe {
            libc::close(r);
            libc::close(w);
        }
    }
}
