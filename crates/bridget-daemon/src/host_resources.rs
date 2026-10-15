//! Relevé hôte168 passif : primitives natives, sans thread, processus ni benchmark.
use bridget_transport::protocol::HostResourceSnapshotV1;
#[cfg(target_os = "linux")]
use std::io::{BufRead, Read};

#[derive(Default)]
pub struct HostResourceCollector {
    previous_cpu: Option<(u64, u64)>,
    latest: Option<HostResourceSnapshotV1>,
    #[cfg(test)]
    collections: usize,
}

impl HostResourceCollector {
    /// O(1) : fichiers Linux bornés ou appels natifs Mac constants. Une seule
    /// collecte sert tout le lot ; la capacité compilation est déclarative.
    pub fn collect(&mut self, now: i64, compilation_slots: Option<u8>) -> HostResourceSnapshotV1 {
        self.observe(
            now,
            compilation_slots,
            read_cpu(),
            read_available_memory_percent(),
            read_normalized_load_percent(),
        )
    }

    /// Changement d'époque : aucune moyenne CPU ni valeur de l'ancien lien
    /// n'est proposée comme nouvel échantillon. Le prochain CPU reste inconnu.
    pub fn reset(&mut self) {
        self.previous_cpu = None;
        self.latest = None;
    }

    /// Lecture du cache hôte, O(1). Aucun fait périmé ou futur n'est rendu.
    /// Le maître conserve son propre contrôle received_at/époque à la réception.
    pub fn fresh_snapshot(&self, now: i64, max_age_secs: u64) -> Option<&HostResourceSnapshotV1> {
        let snapshot = self.latest.as_ref()?;
        if now <= 0
            || snapshot.source_observed_at <= 0
            || snapshot.received_at <= 0
            || !(1..=300).contains(&max_age_secs)
            || snapshot.source_observed_at > now
            || snapshot.received_at > now
            || now.saturating_sub(snapshot.source_observed_at) > max_age_secs as i64
            || now.saturating_sub(snapshot.received_at) > max_age_secs as i64
        {
            return None;
        }
        Some(snapshot)
    }

    /// Oracle de chaîne uniquement : mêmes validations et cache que collect.
    #[cfg(test)]
    pub(crate) fn observe_for_test(
        &mut self,
        now: i64,
        compilation_slots: Option<u8>,
        cpu: Option<(u64, u64)>,
        available_memory_percent: Option<u8>,
        normalized_load_percent: Option<u8>,
    ) -> HostResourceSnapshotV1 {
        self.observe(
            now,
            compilation_slots,
            cpu,
            available_memory_percent,
            normalized_load_percent,
        )
    }

    fn observe(
        &mut self,
        now: i64,
        compilation_slots: Option<u8>,
        cpu: Option<(u64, u64)>,
        available_memory_percent: Option<u8>,
        normalized_load_percent: Option<u8>,
    ) -> HostResourceSnapshotV1 {
        #[cfg(test)]
        {
            self.collections += 1;
        }
        if self
            .latest
            .as_ref()
            .is_some_and(|previous| now < previous.source_observed_at)
        {
            self.previous_cpu = None;
        }
        let cpu_percent = match (self.previous_cpu, cpu) {
            (Some(previous), Some(current)) => cpu_percent(previous, current),
            _ => None,
        };
        // Une lecture ratée coupe la série ; la suivante ne fabrique aucun delta.
        self.previous_cpu = cpu;
        let snapshot = HostResourceSnapshotV1 {
            source_observed_at: now,
            received_at: now,
            cpu_percent,
            available_memory_percent: available_memory_percent.filter(|value| *value <= 100),
            normalized_load_percent: normalized_load_percent.filter(|value| *value <= 100),
            compilation_slots: compilation_slots
                .filter(|value| *value <= 10)
                .map(u16::from),
        };
        self.latest = Some(snapshot.clone());
        snapshot
    }
}

#[cfg(target_os = "linux")]
fn read_cpu() -> Option<(u64, u64)> {
    let mut line = String::new();
    std::io::BufReader::new(std::fs::File::open("/proc/stat").ok()?)
        .take(4096)
        .read_line(&mut line)
        .ok()?;
    parse_linux_cpu(&line)
}

#[cfg(any(target_os = "linux", test))]
fn parse_linux_cpu(line: &str) -> Option<(u64, u64)> {
    let mut fields = line.split_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }
    // Les deux colonnes guest sont déjà comprises dans user/nice. Seules les
    // huit premières colonnes entrent dans le total : pas de double comptage.
    let values = fields
        .take(8)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() < 4 {
        return None;
    }
    let total = values
        .iter()
        .try_fold(0_u64, |sum, value| sum.checked_add(*value))?;
    let idle = values[3].checked_add(values.get(4).copied().unwrap_or(0))?;
    (idle <= total).then_some((total, idle))
}

fn cpu_percent(previous: (u64, u64), current: (u64, u64)) -> Option<u8> {
    let total = current.0.checked_sub(previous.0)?;
    let idle = current.1.checked_sub(previous.1)?;
    if total == 0 || idle > total {
        return None;
    }
    Some((((total - idle) as u128 * 100) / total as u128) as u8)
}

fn available_percent(total: u64, available: u64) -> Option<u8> {
    if total == 0 || available > total {
        return None;
    }
    Some(((available as u128 * 100) / total as u128) as u8)
}

#[cfg(target_os = "linux")]
fn read_available_memory_percent() -> Option<u8> {
    let mut text = String::new();
    std::fs::File::open("/proc/meminfo")
        .ok()?
        .take(65536)
        .read_to_string(&mut text)
        .ok()?;
    available_percent(
        meminfo(&text, "MemTotal:")?,
        meminfo(&text, "MemAvailable:")?,
    )
}

#[cfg(any(target_os = "linux", test))]
fn meminfo(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        line.strip_prefix(key)?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

#[cfg(target_os = "macos")]
struct HostPort(libc::mach_port_t);

#[cfg(target_os = "macos")]
impl HostPort {
    fn acquire() -> Option<Self> {
        let port = unsafe { libc::mach_host_self() };
        (port != 0).then_some(Self(port))
    }
}

#[cfg(target_os = "macos")]
impl Drop for HostPort {
    fn drop(&mut self) {
        // libc fournit les statistiques Mach mais pas ce décrément de droit.
        // Chaque mach_host_self rend un send-right qui doit être libéré.
        unsafe extern "C" {
            fn mach_port_deallocate(
                task: libc::mach_port_t,
                name: libc::mach_port_t,
            ) -> libc::kern_return_t;
        }
        #[allow(deprecated)]
        unsafe {
            let _ = mach_port_deallocate(libc::mach_task_self(), self.0);
        }
    }
}

#[cfg(target_os = "macos")]
fn read_cpu() -> Option<(u64, u64)> {
    let host = HostPort::acquire()?;
    let mut info = std::mem::MaybeUninit::<libc::host_cpu_load_info_data_t>::zeroed();
    let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
    let result = unsafe {
        libc::host_statistics(
            host.0,
            libc::HOST_CPU_LOAD_INFO,
            info.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if result != libc::KERN_SUCCESS || count < libc::HOST_CPU_LOAD_INFO_COUNT {
        return None;
    }
    let ticks = unsafe { info.assume_init() }.cpu_ticks;
    let total = ticks.iter().map(|value| u64::from(*value)).sum();
    Some((total, u64::from(ticks[libc::CPU_STATE_IDLE as usize])))
}

#[cfg(target_os = "macos")]
fn read_available_memory_percent() -> Option<u8> {
    let mut total: u64 = 0;
    let mut size = std::mem::size_of::<u64>();
    let result = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&mut total as *mut u64).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 || size != std::mem::size_of::<u64>() {
        return None;
    }
    let host = HostPort::acquire()?;
    let mut info = std::mem::MaybeUninit::<libc::vm_statistics64_data_t>::zeroed();
    let mut count = libc::HOST_VM_INFO64_COUNT;
    let result = unsafe {
        libc::host_statistics64(
            host.0,
            libc::HOST_VM_INFO64,
            info.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if result != libc::KERN_SUCCESS || count < 3 {
        return None;
    }
    let info = unsafe { info.assume_init() };
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return None;
    }
    // RAM immédiatement libre ou réutilisable (pages inactives). Les pages
    // spéculatives sont déjà incluses dans free_count ; pas de double compte.
    let pages = u64::from(info.free_count).checked_add(u64::from(info.inactive_count))?;
    available_percent(total, pages.checked_mul(page_size as u64)?)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_cpu() -> Option<(u64, u64)> {
    None
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_available_memory_percent() -> Option<u8> {
    None
}

fn normalize_load(load: f64, cpus: i64) -> Option<u8> {
    if !load.is_finite() || load < 0.0 || cpus <= 0 {
        return None;
    }
    Some((load * 100.0 / cpus as f64).clamp(0.0, 100.0) as u8)
}
fn read_normalized_load_percent() -> Option<u8> {
    let mut loads = [0.0_f64; 1];
    if unsafe { libc::getloadavg(loads.as_mut_ptr(), 1) } != 1 {
        return None;
    }
    normalize_load(
        loads[0],
        unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) } as i64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec168_cpu_initial_delta_inconnu_apres_erreur_et_reset_epoch() {
        let mut collector = HostResourceCollector::default();
        let first = collector.observe(100, Some(4), Some((100, 20)), Some(60), Some(20));
        assert_eq!(first.cpu_percent, None);
        assert_eq!(collector.collections, 1);
        assert_eq!(collector.fresh_snapshot(100, 30), Some(&first));
        assert!(collector.fresh_snapshot(131, 30).is_none());
        assert!(collector.fresh_snapshot(99, 30).is_none());
        assert!(collector.fresh_snapshot(100, 0).is_none());
        assert_eq!(first.compilation_slots, Some(4));
        let next = collector.observe(101, None, Some((200, 40)), None, None);
        assert_eq!(next.cpu_percent, Some(80));
        assert_eq!(
            collector.collections, 2,
            "une collecte par lot, pas par option"
        );
        assert_eq!(collector.latest.as_ref(), Some(&next));
        let failed = collector.observe(102, Some(11), None, Some(101), Some(101));
        assert_eq!(failed.cpu_percent, None);
        assert_eq!(failed.available_memory_percent, None);
        assert_eq!(failed.normalized_load_percent, None);
        assert_eq!(failed.compilation_slots, None);
        assert_eq!(
            collector
                .observe(103, None, Some((300, 50)), None, None)
                .cpu_percent,
            None
        );
        collector.reset();
        assert!(collector.latest.is_none());
        assert_eq!(
            collector
                .observe(104, Some(0), Some((400, 60)), None, None)
                .cpu_percent,
            None
        );
        assert_eq!(
            collector
                .observe(90, None, Some((500, 70)), None, None)
                .cpu_percent,
            None,
            "horloge reculée"
        );
        assert_eq!(cpu_percent((100, 30), (100, 30)), None);
        assert_eq!(cpu_percent((100, 30), (90, 20)), None);
        assert_eq!(cpu_percent((100, 30), (110, 50)), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn spec168_darwin_cpu_et_ram_utilisent_les_primitives_natives() {
        assert!(
            read_cpu().is_some(),
            "host_statistics CPU disponible sur Mac"
        );
        assert!(
            read_available_memory_percent().is_some(),
            "sysctl + host_statistics64 disponibles sur Mac"
        );
    }

    #[test]
    fn spec168_linux_guest_ram_load_et_bornes_native() {
        assert_eq!(
            parse_linux_cpu("cpu 10 20 30 40 5 6 7 8 90 80"),
            Some((126, 45))
        );
        assert_eq!(parse_linux_cpu("cpu0 1 2 3 4"), None);
        assert_eq!(parse_linux_cpu("cpu 1 2 invalid 4"), None);
        assert_eq!(parse_linux_cpu("cpu 18446744073709551615 1 2 3"), None);
        assert_eq!(
            meminfo("MemTotal: 100 kB\nMemAvailable: 25 kB\n", "MemAvailable:"),
            Some(25)
        );
        assert_eq!(available_percent(100, 25), Some(25));
        assert_eq!(available_percent(0, 0), None);
        assert_eq!(available_percent(100, 101), None);
        assert_eq!(available_percent(u64::MAX, u64::MAX), Some(100));
        assert_eq!(normalize_load(2.0, 4), Some(50));
        assert_eq!(normalize_load(8.0, 4), Some(100));
        assert_eq!(normalize_load(f64::NAN, 4), None);
        assert_eq!(normalize_load(-1.0, 4), None);
        assert_eq!(normalize_load(1.0, 0), None);
    }
}
