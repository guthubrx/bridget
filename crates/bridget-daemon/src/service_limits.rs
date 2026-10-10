//! Budget de descripteurs des deux services, sans modifier la limite dure.

pub(crate) fn ensure_open_file_limit() -> Result<(), String> {
    const BUDGET: libc::rlim_t = 4096;
    let read = || -> Result<libc::rlimit, String> {
        let mut limits = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) } != 0 {
            return Err(format!(
                "getrlimit(RLIMIT_NOFILE): {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(limits)
    };
    let mut limits = read()?;
    let hard = limits.rlim_max;
    let target = limits.rlim_cur.max(BUDGET.min(hard));
    if limits.rlim_cur < target {
        limits.rlim_cur = target;
        if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limits) } != 0 {
            return Err(format!(
                "setrlimit(RLIMIT_NOFILE, soft={target}, hard={hard}): {}",
                std::io::Error::last_os_error()
            ));
        }
    }
    let actual = read()?;
    log::info!(
        "service RLIMIT_NOFILE: soft={}, hard={}, budget={BUDGET}",
        actual.rlim_cur,
        actual.rlim_max
    );
    if actual.rlim_max < BUDGET {
        log::warn!(
            "service RLIMIT_NOFILE: limite dure restrictive {} < {BUDGET}",
            actual.rlim_max
        );
    }
    if actual.rlim_cur < target || actual.rlim_max != hard {
        return Err(format!(
            "RLIMIT_NOFILE incohérent après réglage: soft={}, hard={}, attendu soft>={target}, hard={hard}",
            actual.rlim_cur, actual.rlim_max
        ));
    }
    Ok(())
}
