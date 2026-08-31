//! Reçus durables de remise idempotente côté wrapper.

use bridget_transport::fsutil::{create_private_dir, open_private_file, write_private_file_atomic};
use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_DELIVERY_ID_LEN: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiptQuota {
    pub max_entries: usize,
    pub max_bytes: u64,
    pub horizon_max_secs: i64,
}

impl Default for ReceiptQuota {
    fn default() -> Self {
        Self {
            max_entries: 4096,
            max_bytes: 16 * 1024 * 1024,
            horizon_max_secs: 7 * 24 * 60 * 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptDecision {
    Inject,
    Acked,
    Indeterminate,
    RejectedQuota,
}

#[derive(Debug)]
pub enum ReceiptError {
    InvalidInstanceId,
    InvalidDeliveryId,
    InvalidExpiry,
    InstanceAlreadyOpen,
    Io(io::Error),
}

impl std::fmt::Display for ReceiptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstanceId => write!(formatter, "instance_id invalide"),
            Self::InvalidDeliveryId => write!(formatter, "delivery_id invalide"),
            Self::InvalidExpiry => write!(formatter, "expires_at invalide"),
            Self::InstanceAlreadyOpen => {
                write!(formatter, "store de reçus déjà ouvert pour cette instance")
            }
            Self::Io(error) => write!(formatter, "reçu durable impossible: {error}"),
        }
    }
}

impl std::error::Error for ReceiptError {}

impl From<io::Error> for ReceiptError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct ReceiptStore {
    directory: PathBuf,
    quota: ReceiptQuota,
    _handle_lock: fs::File,
    transition_lock: Mutex<()>,
}

impl ReceiptStore {
    /// Ouvre l'état sous `<state_home>/bridget/receipts/<instance_id>/`.
    pub fn open(
        state_home: &Path,
        instance_id: &str,
        quota: ReceiptQuota,
    ) -> Result<Self, ReceiptError> {
        validate_token(instance_id).map_err(|_| ReceiptError::InvalidInstanceId)?;
        let receipts_directory = state_home.join("bridget").join("receipts");
        create_private_dir(&receipts_directory)?;
        let directory = receipts_directory.join(instance_id);
        let enrollment_path = receipts_directory.join(format!(".instance-{instance_id}.enrolled"));
        let handle_lock = acquire_instance_lock(&receipts_directory, instance_id)?;
        let enrollment = read_enrollment(&enrollment_path)?;
        let directory_exists = directory.exists();
        let store = Self {
            directory,
            quota,
            _handle_lock: handle_lock,
            transition_lock: Mutex::new(()),
        };

        match (enrollment, directory_exists) {
            (Enrollment::Missing, false) => {
                // Cet enregistrement est écrit avant le premier état de l'instance :
                // une disparition ultérieure du répertoire ne peut jamais ressembler
                // à une première initialisation.
                write_private_file_atomic(&enrollment_path, b"v1\n")?;
                create_private_dir(&store.directory)?;
                store.write_max_expires_at(0)?;
            }
            (Enrollment::Valid, true) => {
                if store.read_max_expires_at().is_err() {
                    store.quarantine_permanently()?;
                }
            }
            (Enrollment::Valid, false) | (Enrollment::Missing, true) | (Enrollment::Corrupt, _) => {
                if enrollment == Enrollment::Missing {
                    write_private_file_atomic(&enrollment_path, b"v1\n")?;
                }
                create_private_dir(&store.directory)?;
                store.quarantine_permanently()?;
            }
        }
        Ok(store)
    }

    /// Marque la remise comme vue avant toute injection.
    pub fn receive(
        &self,
        delivery_id: &str,
        expires_at: i64,
        now: i64,
    ) -> Result<ReceiptDecision, ReceiptError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        validate_token(delivery_id).map_err(|_| ReceiptError::InvalidDeliveryId)?;
        if expires_at <= now {
            return Err(ReceiptError::InvalidExpiry);
        }
        if !self.directory_is_healthy() {
            return Ok(ReceiptDecision::Indeterminate);
        }
        if self.is_quarantined(now)? {
            return Ok(ReceiptDecision::Indeterminate);
        }
        let path = self.entry_path(delivery_id);
        match fs::read(&path) {
            Ok(bytes) => match parse_entry(&bytes) {
                Ok(ReceiptEntry::Acked { .. }) => Ok(ReceiptDecision::Acked),
                Ok(ReceiptEntry::Seen { .. }) => Ok(ReceiptDecision::Indeterminate),
                Err(_) => {
                    self.quarantine(expires_at, now)?;
                    Ok(ReceiptDecision::Indeterminate)
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if self.would_exceed_quota(
                    ReceiptEntry::Acked { expires_at }.serialize().len() as u64
                )? {
                    return Ok(ReceiptDecision::RejectedQuota);
                }
                self.persist_entry(&path, ReceiptEntry::Seen { expires_at })?;
                self.write_max_expires_at(self.read_max_expires_at()?.max(expires_at))?;
                Ok(ReceiptDecision::Inject)
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Après l'observable `PromptDispatched`, rend l'accusé rejouable.
    pub fn acknowledge(
        &self,
        delivery_id: &str,
        now: i64,
    ) -> Result<ReceiptDecision, ReceiptError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        validate_token(delivery_id).map_err(|_| ReceiptError::InvalidDeliveryId)?;
        if self.is_quarantined(now)? || !self.directory_is_healthy() {
            return Ok(ReceiptDecision::Indeterminate);
        }
        let path = self.entry_path(delivery_id);
        let entry = match fs::read(&path) {
            Ok(bytes) => match parse_entry(&bytes) {
                Ok(entry) => entry,
                Err(_) => {
                    self.quarantine(now.saturating_add(self.quota.horizon_max_secs), now)?;
                    return Ok(ReceiptDecision::Indeterminate);
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(ReceiptDecision::Indeterminate);
            }
            Err(error) => return Err(error.into()),
        };
        match entry {
            ReceiptEntry::Acked { .. } => Ok(ReceiptDecision::Acked),
            ReceiptEntry::Seen { expires_at } => {
                self.persist_entry(&path, ReceiptEntry::Acked { expires_at })?;
                Ok(ReceiptDecision::Acked)
            }
        }
    }

    pub fn purge_expired(&self, now: i64) -> Result<usize, ReceiptError> {
        let _transition = self
            .transition_lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        // La purge est atomique par entrée : le sentinel conserve un horizon
        // conservateur entre deux suppressions, donc un crash ne peut pas
        // transformer un reçu disparu en nouvelle injection.
        if !self.directory_is_healthy() {
            return Ok(0);
        }
        let mut removed = 0;
        let mut max_expires_at = 0;
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let path = entry.path();
            if path == self.quarantine_path() || path == self.max_expires_path() {
                continue;
            }
            let bytes = fs::read(&path)?;
            let receipt = match parse_entry(&bytes) {
                Ok(receipt) => receipt,
                Err(_) => {
                    self.quarantine(now.saturating_add(self.quota.horizon_max_secs), now)?;
                    return Ok(removed);
                }
            };
            if receipt.expires_at() <= now {
                fs::remove_file(path)?;
                removed += 1;
            } else {
                max_expires_at = max_expires_at.max(receipt.expires_at());
            }
        }
        self.write_max_expires_at(max_expires_at)?;
        Ok(removed)
    }

    fn directory_is_healthy(&self) -> bool {
        fs::metadata(&self.directory)
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false)
            && self.read_max_expires_at().is_ok()
    }

    fn is_quarantined(&self, now: i64) -> Result<bool, ReceiptError> {
        match fs::read(self.quarantine_path()) {
            Ok(bytes) => Ok(parse_timestamp(&bytes)
                .map(|until| now < until)
                .unwrap_or(true)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    fn quarantine(&self, expires_at: i64, now: i64) -> Result<(), ReceiptError> {
        let until = self
            .read_max_expires_at()
            .unwrap_or(0)
            .max(expires_at)
            .max(now.saturating_add(self.quota.horizon_max_secs));
        write_private_file_atomic(&self.quarantine_path(), until.to_string().as_bytes())?;
        Ok(())
    }

    fn quarantine_permanently(&self) -> Result<(), ReceiptError> {
        write_private_file_atomic(&self.quarantine_path(), i64::MAX.to_string().as_bytes())?;
        Ok(())
    }

    fn would_exceed_quota(&self, next_entry_bytes: u64) -> Result<bool, ReceiptError> {
        let mut entries = 0usize;
        let mut bytes = 0u64;
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let path = entry.path();
            if path == self.quarantine_path() || path == self.max_expires_path() {
                continue;
            }
            entries += 1;
            bytes = bytes.saturating_add(entry.metadata()?.len());
        }
        Ok(entries >= self.quota.max_entries
            || bytes.saturating_add(next_entry_bytes) > self.quota.max_bytes)
    }

    fn entry_path(&self, delivery_id: &str) -> PathBuf {
        self.directory.join(format!("receipt-{delivery_id}"))
    }

    fn max_expires_path(&self) -> PathBuf {
        self.directory.join("max_expires_at")
    }

    fn quarantine_path(&self) -> PathBuf {
        self.directory.join("quarantine_until")
    }

    fn read_max_expires_at(&self) -> Result<i64, ReceiptError> {
        match fs::read(self.max_expires_path()) {
            Ok(bytes) => parse_timestamp(&bytes)
                .map_err(|_| ReceiptError::Io(io::Error::other("horizon corrompu"))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Err(ReceiptError::Io(
                io::Error::new(io::ErrorKind::NotFound, "sentinel max_expires_at absent"),
            )),
            Err(error) => Err(error.into()),
        }
    }

    fn write_max_expires_at(&self, expires_at: i64) -> Result<(), ReceiptError> {
        write_private_file_atomic(&self.max_expires_path(), expires_at.to_string().as_bytes())?;
        Ok(())
    }

    fn persist_entry(&self, path: &Path, entry: ReceiptEntry) -> Result<(), ReceiptError> {
        write_private_file_atomic(path, entry.serialize().as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Enrollment {
    Missing,
    Valid,
    Corrupt,
}

fn read_enrollment(path: &Path) -> Result<Enrollment, ReceiptError> {
    match fs::read(path) {
        Ok(bytes) if bytes == b"v1\n" => Ok(Enrollment::Valid),
        Ok(_) => Ok(Enrollment::Corrupt),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Enrollment::Missing),
        Err(error) => Err(error.into()),
    }
}

fn acquire_instance_lock(
    receipts_directory: &Path,
    instance_id: &str,
) -> Result<fs::File, ReceiptError> {
    let lock_path = receipts_directory.join(format!(".instance-{instance_id}.lock"));
    let lock = open_private_file(&lock_path)?;
    // Une instance de session est servie par un seul handle. Le verrou est au
    // niveau parent afin que supprimer le répertoire de reçus ne l'efface pas.
    let result = unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result != 0 {
        let error = io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN)
        {
            return Err(ReceiptError::InstanceAlreadyOpen);
        }
        return Err(error.into());
    }
    Ok(lock)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiptEntry {
    Seen { expires_at: i64 },
    Acked { expires_at: i64 },
}

impl ReceiptEntry {
    fn expires_at(self) -> i64 {
        match self {
            Self::Seen { expires_at } | Self::Acked { expires_at } => expires_at,
        }
    }

    fn serialize(self) -> String {
        match self {
            Self::Seen { expires_at } => format!("seen\n{expires_at}\n"),
            Self::Acked { expires_at } => format!("acked\n{expires_at}\n"),
        }
    }
}

fn parse_entry(bytes: &[u8]) -> Result<ReceiptEntry, ()> {
    let value = std::str::from_utf8(bytes).map_err(|_| ())?;
    let mut lines = value.lines();
    let state = lines.next().ok_or(())?;
    let expires_at = lines.next().ok_or(())?.parse().map_err(|_| ())?;
    if lines.next().is_some() {
        return Err(());
    }
    match state {
        "seen" => Ok(ReceiptEntry::Seen { expires_at }),
        "acked" => Ok(ReceiptEntry::Acked { expires_at }),
        _ => Err(()),
    }
}

fn parse_timestamp(bytes: &[u8]) -> Result<i64, ()> {
    std::str::from_utf8(bytes)
        .map_err(|_| ())?
        .trim()
        .parse()
        .map_err(|_| ())
}

fn validate_token(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_DELIVERY_ID_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    };
    use std::thread;

    const NOW: i64 = 1_000_000;
    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn test_store(quota: ReceiptQuota) -> (ReceiptStore, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "bridget-receipts-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        let store = ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", quota).unwrap();
        (store, root)
    }

    #[test]
    fn seen_then_acked_replays_the_ack_without_another_injection() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.acknowledge("delivery_1", NOW).unwrap(),
            ReceiptDecision::Acked
        );
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Acked
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seen_without_ack_is_indeterminate_and_never_requests_a_second_injection() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn quota_saturation_is_a_terminal_refusal_before_a_seen_write() {
        let (store, root) = test_store(ReceiptQuota {
            max_entries: 1,
            max_bytes: 1024,
            horizon_max_secs: 60,
        });
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.receive("delivery_2", NOW + 60, NOW).unwrap(),
            ReceiptDecision::RejectedQuota
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_corrupted_acked_receipt_quarantines_without_reinjection() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.acknowledge("delivery_1", NOW).unwrap(),
            ReceiptDecision::Acked
        );
        fs::write(store.entry_path("delivery_1"), b"broken").unwrap();
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert!(store.quarantine_path().exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn disappearance_after_initialization_is_fail_closed() {
        let (store, root) = test_store(ReceiptQuota::default());
        fs::remove_dir_all(&store.directory).unwrap();
        assert_eq!(
            store.receive("delivery_1", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert!(!store.directory.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn deleting_an_enrolled_instance_directory_remains_fail_closed_after_reopen() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.acknowledge("delivery_old", NOW).unwrap(),
            ReceiptDecision::Acked
        );
        let directory = store.directory.clone();
        drop(store);

        fs::remove_dir_all(directory).unwrap();
        let reopened =
            ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default())
                .unwrap();
        assert_eq!(
            reopened.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert_eq!(
            reopened.receive("delivery_new", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_sentinel_or_recreated_directory_is_fail_closed_after_initialization() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        fs::remove_file(store.max_expires_path()).unwrap();
        assert_eq!(
            store.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert_eq!(
            store.receive("delivery_new", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        drop(store);
        let reopened =
            ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default())
                .unwrap();
        assert_eq!(
            reopened.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert_eq!(
            reopened.receive("delivery_new", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );

        let directory = reopened.directory.clone();
        drop(reopened);
        fs::remove_dir_all(&directory).unwrap();
        create_private_dir(&directory).unwrap();
        let recreated =
            ReceiptStore::open(&root, "instance_012_aaaaaaaaaaaa", ReceiptQuota::default())
                .unwrap();
        assert_eq!(
            recreated.receive("delivery_old", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        assert_eq!(
            recreated.receive("delivery_new", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Indeterminate
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_delivery_reserves_once_and_serializes_quota() {
        let (store, root) = test_store(ReceiptQuota {
            max_entries: 1,
            max_bytes: 1024,
            horizon_max_secs: 60,
        });
        let store = Arc::new(store);
        let barrier = Arc::new(Barrier::new(3));
        let mut same_delivery = Vec::new();
        for _ in 0..2 {
            let store = store.clone();
            let barrier = barrier.clone();
            same_delivery.push(thread::spawn(move || {
                barrier.wait();
                store.receive("delivery_same", NOW + 60, NOW).unwrap()
            }));
        }
        barrier.wait();
        let decisions: Vec<_> = same_delivery
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            decisions
                .iter()
                .filter(|decision| **decision == ReceiptDecision::Inject)
                .count(),
            1
        );
        assert_eq!(
            decisions
                .iter()
                .filter(|decision| **decision == ReceiptDecision::Indeterminate)
                .count(),
            1
        );

        let quota_store = Arc::new(
            ReceiptStore::open(
                &root,
                "instance_012_bbbbbbbbbbbb",
                ReceiptQuota {
                    max_entries: 1,
                    max_bytes: 1024,
                    horizon_max_secs: 60,
                },
            )
            .unwrap(),
        );
        let quota_barrier = Arc::new(Barrier::new(3));
        let mut distinct_delivery = Vec::new();
        for delivery_id in ["delivery_other_1", "delivery_other_2"] {
            let store = quota_store.clone();
            let barrier = quota_barrier.clone();
            distinct_delivery.push(thread::spawn(move || {
                barrier.wait();
                store.receive(delivery_id, NOW + 60, NOW).unwrap()
            }));
        }
        quota_barrier.wait();
        let decisions: Vec<_> = distinct_delivery
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            decisions
                .iter()
                .filter(|decision| **decision == ReceiptDecision::Inject)
                .count(),
            1
        );
        assert_eq!(
            decisions
                .iter()
                .filter(|decision| **decision == ReceiptDecision::RejectedQuota)
                .count(),
            1
        );
        drop(store);
        drop(quota_store);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn only_one_handle_can_open_an_instance_at_once() {
        let root = std::env::temp_dir().join(format!(
            "bridget-receipts-handle-{}-{}",
            std::process::id(),
            NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for _ in 0..2 {
            let root = root.clone();
            let barrier = barrier.clone();
            handles.push(thread::spawn(move || {
                let opened =
                    ReceiptStore::open(&root, "instance_012_cccccccccccc", ReceiptQuota::default());
                barrier.wait();
                let accepted = opened.is_ok();
                drop(opened);
                accepted
            }));
        }
        barrier.wait();
        let opened_count = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|opened| *opened)
            .count();
        assert_eq!(opened_count, 1);
        match ReceiptStore::open(&root, "instance_012_cccccccccccc", ReceiptQuota::default()) {
            Ok(store) => drop(store),
            Err(error) => panic!("le verrou doit être libéré après les deux handles: {error}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn quota_reserves_the_final_acked_size_at_its_exact_boundary() {
        let expires_at = NOW + 60;
        let final_size = ReceiptEntry::Acked { expires_at }.serialize().len() as u64;
        let (store, root) = test_store(ReceiptQuota {
            max_entries: 2,
            max_bytes: final_size,
            horizon_max_secs: 60,
        });
        assert_eq!(
            store.receive("delivery_1", expires_at, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.acknowledge("delivery_1", NOW).unwrap(),
            ReceiptDecision::Acked
        );
        assert_eq!(
            store.receive("delivery_2", expires_at, NOW).unwrap(),
            ReceiptDecision::RejectedQuota
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn private_state_permissions_and_compaction_are_enforced() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(
            store.receive("delivery_1", NOW + 1, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            store.receive("delivery_2", NOW + 60, NOW).unwrap(),
            ReceiptDecision::Inject
        );
        assert_eq!(
            fs::metadata(&store.directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(store.entry_path("delivery_1"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(store.purge_expired(NOW + 1).unwrap(), 1);
        assert_eq!(store.read_max_expires_at().unwrap(), NOW + 60);
        let _ = fs::remove_dir_all(root);
    }
}
