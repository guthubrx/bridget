//! Purge bornée des copies orphelines dans le namespace autorisé et alerte disque.
//! Aucun inventaire des worktrees de l'hôte.
//!
//! Discipline alignée sur `purge_orphan_mcp_configs` : best-effort, log
//! nominatif, jamais d'effacement sous un processus vivant (fail-closed si la
//! sonde propriétaire est incertaine).
//!
//! Chemin critique (bisect fable2, 2026-08-24) :
//! 1. filtre d'âge **avant** toute sonde (Article XVIII) ;
//! 2. hors boot synchrone — thread détaché après readiness du daemon ;
//! 3. borne par passage (`MAX_ENTRIES_PER_PASS`) ;
//! 4. **un** `lsof` global par passage, jamais un fork par entrée.

use log::{info, warn};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Âge minimal avant qu'une copie `/tmp/bridget-*` orpheline soit éligible.
pub const ORPHAN_MIN_AGE_SECS: u64 = 24 * 60 * 60;

/// Seuil d'alerte : en dessous, WARN visible (boot, reaper, who/status).
pub const DISK_WARN_FREE_BYTES: u64 = 20 * 1024 * 1024 * 1024;

/// Borne d'entrées **âgées** traitées par passage (le reste est journalisé).
pub const MAX_ENTRIES_PER_PASS: usize = 128;

/// Préfixe des copies de travail à considérer sous `/tmp`.
pub const BRIDGET_TMP_PREFIX: &str = "bridget-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeDecision {
    Delete,
    Spare { reason: &'static str },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeReport {
    pub deleted: Vec<PathBuf>,
    pub spared: Vec<(PathBuf, &'static str)>,
    pub errors: Vec<(PathBuf, String)>,
    /// Entrées âgées non traitées faute de borne (prochain passage).
    pub deferred: usize,
}

/// Décision pure : **âge d'abord** (Article XVIII), puis propriétaires.
///
/// `owner_pids = None` → propriétaire inconnu → épargne (fail-closed).
/// `owner_pids = Some([])` → orphelin confirmé.
/// `owner_pids = Some([..])` → vivant → épargne.
pub fn decide_purge(age_secs: u64, owner_pids: Option<&[u32]>, min_age_secs: u64) -> PurgeDecision {
    // Critère gratuit avant toute sonde coûteuse.
    if age_secs < min_age_secs {
        return PurgeDecision::Spare {
            reason: "copie_du_jour",
        };
    }
    match owner_pids {
        None => PurgeDecision::Spare {
            reason: "proprietaire_inconnu",
        },
        Some(pids) if !pids.is_empty() => PurgeDecision::Spare {
            reason: "processus_vivant",
        },
        Some(_) => PurgeDecision::Delete,
    }
}

/// Message d'alerte si l'espace libre du volume est sous le seuil.
pub fn disk_pressure_warning(free_bytes: u64, threshold_bytes: u64) -> Option<String> {
    if free_bytes >= threshold_bytes {
        return None;
    }
    let free_gi = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let threshold_gi = threshold_bytes / (1024 * 1024 * 1024);
    Some(format!(
        "WARN disque: {free_gi:.1} Gi libres (< {threshold_gi} Gi) — saturation récurrente des copies de travail"
    ))
}

/// Espace libre disponible pour un utilisateur non-root sur le volume de `path`.
pub fn free_bytes_for(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: pointeur CString valide le temps de l'appel ; struct zeroed.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        // Casts volontaires : largeur de f_bavail/f_frsize varie selon l'OS.
        #[allow(clippy::unnecessary_cast)]
        {
            Some(stat.f_bavail as u64 * stat.f_frsize as u64)
        }
    }
}

/// WARN nominatif si le volume de `path` est sous le seuil.
pub fn warn_if_disk_low(path: &Path) {
    let Some(free) = free_bytes_for(path) else {
        return;
    };
    if let Some(message) = disk_pressure_warning(free, DISK_WARN_FREE_BYTES) {
        warn!("{message}");
    }
}

/// Texte à afficher dans who/status (aucune panique auto).
pub fn disk_warning_for_display(path: &Path) -> Option<String> {
    let free = free_bytes_for(path)?;
    disk_pressure_warning(free, DISK_WARN_FREE_BYTES)
}

/// Purge production : âge d'abord, un lsof global, borne par passage.
pub fn purge_orphan_bridget_tmp(tmp_dir: &Path) -> PurgeReport {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entries = match scan_bridget_tmp(tmp_dir, now) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                "ramasse-copies: lecture {} impossible: {error}",
                tmp_dir.display()
            );
            return PurgeReport::default();
        }
    };
    let (young, aged) = partition_by_age(entries, ORPHAN_MIN_AGE_SECS);
    let mut report = PurgeReport::default();
    for entry in young {
        report.spared.push((entry.path, "copie_du_jour"));
    }
    let deferred = aged.len().saturating_sub(MAX_ENTRIES_PER_PASS);
    report.deferred = deferred;
    if deferred > 0 {
        warn!(
            "ramasse-copies: {deferred} entrée(s) âgée(s) reportée(s) (borne {MAX_ENTRIES_PER_PASS})"
        );
    }
    let batch: Vec<BridgetTmpEntry> = aged.into_iter().take(MAX_ENTRIES_PER_PASS).collect();
    let paths: Vec<PathBuf> = batch.iter().map(|e| e.path.clone()).collect();
    let index = GlobalOwnerIndex::build_once(&paths);
    apply_purge_batch(&batch, ORPHAN_MIN_AGE_SECS, &index, &mut report);
    report
}

/// Cœur testable : le probe n'est appelé **qu'après** le filtre d'âge.
pub fn purge_orphan_bridget_tmp_at<P: OwnerProbe>(
    tmp_dir: &Path,
    now_unix: u64,
    min_age_secs: u64,
    max_per_pass: usize,
    probe: &P,
) -> PurgeReport {
    let mut report = PurgeReport::default();
    let entries = match scan_bridget_tmp(tmp_dir, now_unix) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                "ramasse-copies: lecture {} impossible: {error}",
                tmp_dir.display()
            );
            return report;
        }
    };
    let (young, aged) = partition_by_age(entries, min_age_secs);
    for entry in young {
        report.spared.push((entry.path, "copie_du_jour"));
    }
    report.deferred = aged.len().saturating_sub(max_per_pass);
    if report.deferred > 0 {
        warn!(
            "ramasse-copies: {} entrée(s) âgée(s) reportée(s) (borne {max_per_pass})",
            report.deferred
        );
    }
    let batch: Vec<BridgetTmpEntry> = aged.into_iter().take(max_per_pass).collect();
    apply_purge_batch(&batch, min_age_secs, probe, &mut report);
    report
}

fn partition_by_age(
    entries: Vec<BridgetTmpEntry>,
    min_age_secs: u64,
) -> (Vec<BridgetTmpEntry>, Vec<BridgetTmpEntry>) {
    let mut young = Vec::new();
    let mut aged = Vec::new();
    for entry in entries {
        if entry.age_secs < min_age_secs {
            young.push(entry);
        } else {
            aged.push(entry);
        }
    }
    (young, aged)
}

fn apply_purge_batch<P: OwnerProbe>(
    batch: &[BridgetTmpEntry],
    min_age_secs: u64,
    probe: &P,
    report: &mut PurgeReport,
) {
    for entry in batch {
        // Âge déjà filtré — la sonde ne court que sur le lot âgé borné.
        let owners = probe.owner_pids(&entry.path);
        match decide_purge(entry.age_secs, owners.as_deref(), min_age_secs) {
            PurgeDecision::Spare { reason } => {
                report.spared.push((entry.path.clone(), reason));
            }
            PurgeDecision::Delete => {
                let path = entry.path.clone();
                let remove_result = if entry.is_dir {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };
                match remove_result {
                    Ok(()) => {
                        info!("ramasse-copies: orphelin âgé supprimé {}", path.display());
                        report.deleted.push(path);
                    }
                    Err(error) => {
                        warn!(
                            "ramasse-copies: suppression impossible {}: {error}",
                            path.display()
                        );
                        report.errors.push((path, error.to_string()));
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct BridgetTmpEntry {
    path: PathBuf,
    age_secs: u64,
    is_dir: bool,
}

fn scan_bridget_tmp(tmp_dir: &Path, now_unix: u64) -> io::Result<Vec<BridgetTmpEntry>> {
    let entries = match fs::read_dir(tmp_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(BRIDGET_TMP_PREFIX) {
            continue;
        }
        // Socket / pid de production : ne pas toucher au socket daemon courant.
        if name == "bridget.sock" || name.ends_with(".sock") || name.ends_with(".pid") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let age = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| now_unix.saturating_sub(d.as_secs()))
            .unwrap_or(0);
        out.push(BridgetTmpEntry {
            path: entry.path(),
            age_secs: age,
            is_dir: meta.is_dir(),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Sonde des PIDs qui détiennent encore une copie.
pub trait OwnerProbe {
    /// `None` = incertitude (épargner). `Some(pids)` = liste connue (vide = orphelin).
    fn owner_pids(&self, path: &Path) -> Option<Vec<u32>>;
}

/// Index propriétaire construit **une fois** par passage (un lsof + un ps).
#[derive(Debug, Default)]
pub struct GlobalOwnerIndex {
    by_path: HashMap<PathBuf, Vec<u32>>,
    cmdline_hits: HashMap<PathBuf, Vec<u32>>,
    lsof_reliable: bool,
}

impl GlobalOwnerIndex {
    /// Un seul `lsof` pour tout le lot + un seul `ps` — jamais N forks.
    pub fn build_once(paths: &[PathBuf]) -> Self {
        if paths.is_empty() {
            return Self {
                lsof_reliable: true,
                ..Self::default()
            };
        }
        let (by_path, lsof_reliable) = match lsof_index_once(paths) {
            Ok(map) => (map, true),
            Err(error) => {
                warn!("ramasse-copies: lsof global en échec: {error}");
                (HashMap::new(), false)
            }
        };
        let cmdline_hits = cmdline_index_once(paths).unwrap_or_default();
        Self {
            by_path,
            cmdline_hits,
            lsof_reliable,
        }
    }
}

impl OwnerProbe for GlobalOwnerIndex {
    fn owner_pids(&self, path: &Path) -> Option<Vec<u32>> {
        let mut pids = Vec::new();

        if let Some(name_pid) = pid_suffix_from_name(path)
            && process_alive(name_pid)
        {
            pids.push(name_pid);
        }

        if let Some(found) = self.by_path.get(path) {
            for pid in found {
                if !pids.contains(pid) {
                    pids.push(*pid);
                }
            }
        } else {
            // Fichiers ouverts sous un répertoire candidat.
            for (open_path, open_pids) in &self.by_path {
                if open_path.starts_with(path) {
                    for pid in open_pids {
                        if !pids.contains(pid) {
                            pids.push(*pid);
                        }
                    }
                }
            }
        }

        if let Some(found) = self.cmdline_hits.get(path) {
            for pid in found {
                if !pids.contains(pid) {
                    pids.push(*pid);
                }
            }
        }

        if !self.lsof_reliable && pids.is_empty() {
            return None;
        }
        Some(pids)
    }
}

fn lsof_index_once(paths: &[PathBuf]) -> io::Result<HashMap<PathBuf, Vec<u32>>> {
    let mut command = Command::new("lsof");
    // -F pn : un enregistrement machine (pid puis noms) pour TOUT le lot.
    command.args(["-F", "pn", "--"]);
    for path in paths {
        command.arg(path);
    }
    let output = command.output()?;
    // exit 1 (rien d'ouvert) est nominal — on parse toujours stdout.
    Ok(parse_lsof_fn(&output.stdout))
}

fn parse_lsof_fn(stdout: &[u8]) -> HashMap<PathBuf, Vec<u32>> {
    let mut map: HashMap<PathBuf, Vec<u32>> = HashMap::new();
    let mut current_pid: Option<u32> = None;
    for line in String::from_utf8_lossy(stdout).lines() {
        if let Some(pid_s) = line.strip_prefix('p') {
            current_pid = pid_s.trim().parse().ok();
            continue;
        }
        let Some(name) = line.strip_prefix('n') else {
            continue;
        };
        let Some(pid) = current_pid else {
            continue;
        };
        let path = PathBuf::from(name);
        let entry = map.entry(path).or_default();
        if !entry.contains(&pid) {
            entry.push(pid);
        }
    }
    map
}

fn cmdline_index_once(paths: &[PathBuf]) -> io::Result<HashMap<PathBuf, Vec<u32>>> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("ps a échoué"));
    }
    let needles: Vec<(PathBuf, String)> = paths
        .iter()
        .map(|p| (p.clone(), p.to_string_lossy().into_owned()))
        .collect();
    let mut map: HashMap<PathBuf, Vec<u32>> = HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        let Some((pid_s, command)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Ok(pid) = pid_s.trim().parse::<u32>() else {
            continue;
        };
        for (path, needle) in &needles {
            if command.contains(needle.as_str()) {
                let entry = map.entry(path.clone()).or_default();
                if !entry.contains(&pid) {
                    entry.push(pid);
                }
            }
        }
    }
    Ok(map)
}

fn pid_suffix_from_name(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let tail = name.rsplit('-').next()?;
    let pid: u32 = tail.parse().ok()?;
    // Éviter les faux positifs (années, compteurs courts).
    if pid >= 100 { Some(pid) } else { None }
}

fn process_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    struct MapProbe {
        map: HashMap<PathBuf, Option<Vec<u32>>>,
    }

    impl OwnerProbe for MapProbe {
        fn owner_pids(&self, path: &Path) -> Option<Vec<u32>> {
            self.map.get(path).cloned().unwrap_or(Some(vec![]))
        }
    }

    struct CountingSlowProbe {
        calls: AtomicUsize,
    }

    impl OwnerProbe for CountingSlowProbe {
        fn owner_pids(&self, _path: &Path) -> Option<Vec<u32>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            // 50 ms × 1000 = 50 s si le filtre d'âge est sauté — le test exige <5 s.
            std::thread::sleep(Duration::from_millis(50));
            Some(vec![])
        }
    }

    fn set_mtime(path: &Path, unix_secs: i64) {
        use std::io::Write;
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        let _ = (&file).write_all(b"x");
        let times = libc::timeval {
            tv_sec: unix_secs,
            tv_usec: 0,
        };
        let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        // SAFETY: chemin CString valide ; tableau de 2 timeval (atime, mtime).
        unsafe {
            let tv = [times, times];
            assert_eq!(libc::utimes(c_path.as_ptr(), tv.as_ptr()), 0);
        }
    }

    #[test]
    fn orpheline_agee_est_ramassee() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS + 1, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Delete
        );
    }

    #[test]
    fn copie_du_jour_est_epargnee() {
        assert_eq!(
            decide_purge(60, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "copie_du_jour"
            }
        );
    }

    #[test]
    fn processus_vivant_est_epargne() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, Some(&[4242]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "processus_vivant"
            }
        );
    }

    #[test]
    fn proprietaire_inconnu_fail_closed() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, None, ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "proprietaire_inconnu"
            }
        );
    }

    #[test]
    fn seuil_franchi_produit_warn() {
        let message = disk_pressure_warning(5 * 1024 * 1024 * 1024, DISK_WARN_FREE_BYTES).unwrap();
        assert!(message.contains("WARN disque"));
        assert!(message.contains("< 20 Gi"));
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES, DISK_WARN_FREE_BYTES).is_none());
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES + 1, DISK_WARN_FREE_BYTES).is_none());
    }

    #[test]
    fn purge_reelle_respecte_age_et_vivant() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let aged = root.join("bridget-aged-orphan");
        let today = root.join("bridget-today");
        let live = root.join("bridget-live-owner");
        let now = 1_700_000_000i64;
        set_mtime(&aged, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);
        set_mtime(&today, now - 120);
        set_mtime(&live, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);

        let mut map = HashMap::new();
        map.insert(aged.clone(), Some(vec![]));
        map.insert(today.clone(), Some(vec![]));
        map.insert(live.clone(), Some(vec![7]));
        let probe = MapProbe { map };

        let report =
            purge_orphan_bridget_tmp_at(&root, now as u64, ORPHAN_MIN_AGE_SECS, usize::MAX, &probe);

        assert!(!aged.exists(), "orpheline âgée doit être ramassée");
        assert!(today.exists(), "copie du jour doit être épargnée");
        assert!(live.exists(), "copie d'un vivant doit être épargnée");
        assert_eq!(report.deleted, vec![aged]);
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &today && *r == "copie_du_jour")
        );
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &live && *r == "processus_vivant")
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn boot_avec_1000_jeunes_reste_sous_5s_sans_sonde() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-boot-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let now = 1_700_000_000u64;
        for i in 0..1000 {
            let path = root.join(format!("bridget-jeune-{i}"));
            set_mtime(&path, (now as i64) - 30);
        }
        let probe = CountingSlowProbe {
            calls: AtomicUsize::new(0),
        };
        let started = Instant::now();
        let report = purge_orphan_bridget_tmp_at(
            &root,
            now,
            ORPHAN_MIN_AGE_SECS,
            MAX_ENTRIES_PER_PASS,
            &probe,
        );
        let elapsed = started.elapsed();
        assert_eq!(
            probe.calls.load(Ordering::SeqCst),
            0,
            "aucune sonde sur des copies du jour"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "boot simulé trop lent: {elapsed:?}"
        );
        assert!(report.deleted.is_empty());
        assert_eq!(report.spared.len(), 1000);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn borne_reporte_le_surplus_age() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-borne-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let now = 1_700_000_000i64;
        for i in 0..5 {
            set_mtime(
                &root.join(format!("bridget-age-{i}")),
                now - (ORPHAN_MIN_AGE_SECS as i64) - 60,
            );
        }
        let probe = MapProbe {
            map: HashMap::new(),
        };
        let report = purge_orphan_bridget_tmp_at(&root, now as u64, ORPHAN_MIN_AGE_SECS, 2, &probe);
        assert_eq!(report.deleted.len(), 2);
        assert_eq!(report.deferred, 3);
        let _ = fs::remove_dir_all(&root);
    }
}
