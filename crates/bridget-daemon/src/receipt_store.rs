//! Reçus durables de remise idempotente côté wrapper.

use bridget_transport::fsutil::{create_private_dir, write_private_file_atomic};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    Io(io::Error),
}

impl std::fmt::Display for ReceiptError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInstanceId => write!(formatter, "instance_id invalide"),
            Self::InvalidDeliveryId => write!(formatter, "delivery_id invalide"),
            Self::InvalidExpiry => write!(formatter, "expires_at invalide"),
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
}

impl ReceiptStore {
    /// Ouvre l'état sous `<state_home>/bridget/receipts/<instance_id>/`.
    pub fn open(
        state_home: &Path,
        instance_id: &str,
        quota: ReceiptQuota,
    ) -> Result<Self, ReceiptError> {
        validate_token(instance_id).map_err(|_| ReceiptError::InvalidInstanceId)?;
        let directory = state_home.join("bridget").join("receipts").join(instance_id);
        let existed_before_open = directory.exists();
        create_private_dir(&directory)?;
        let store = Self { directory, quota };
        if existed_before_open {
            if store.read_max_expires_at().is_err() {
                write_private_file_atomic(&store.quarantine_path(), i64::MAX.to_string().as_bytes())?;
            }
        } else {
            store.write_max_expires_at(0)?;
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
                if self.would_exceed_quota(ReceiptEntry::Seen { expires_at }.serialize().len() as u64)? {
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
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(ReceiptDecision::Indeterminate),
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
            Ok(bytes) => Ok(parse_timestamp(&bytes).map(|until| now < until).unwrap_or(true)),
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
        Ok(entries >= self.quota.max_entries || bytes.saturating_add(next_entry_bytes) > self.quota.max_bytes)
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
            Ok(bytes) => parse_timestamp(&bytes).map_err(|_| ReceiptError::Io(io::Error::other("horizon corrompu"))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(0),
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
    std::str::from_utf8(bytes).map_err(|_| ())?.trim().parse().map_err(|_| ())
}

fn validate_token(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_DELIVERY_ID_LEN
        || !value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

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
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(store.acknowledge("delivery_1", NOW).unwrap(), ReceiptDecision::Acked);
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Acked);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn seen_without_ack_is_indeterminate_and_never_requests_a_second_injection() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Indeterminate);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn quota_saturation_is_a_terminal_refusal_before_a_seen_write() {
        let (store, root) = test_store(ReceiptQuota { max_entries: 1, max_bytes: 1024, horizon_max_secs: 60 });
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(store.receive("delivery_2", NOW + 60, NOW).unwrap(), ReceiptDecision::RejectedQuota);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn a_corrupted_acked_receipt_quarantines_without_reinjection() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(store.acknowledge("delivery_1", NOW).unwrap(), ReceiptDecision::Acked);
        fs::write(store.entry_path("delivery_1"), b"broken").unwrap();
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Indeterminate);
        assert!(store.quarantine_path().exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn disappearance_after_initialization_is_fail_closed() {
        let (store, root) = test_store(ReceiptQuota::default());
        fs::remove_dir_all(&store.directory).unwrap();
        assert_eq!(store.receive("delivery_1", NOW + 60, NOW).unwrap(), ReceiptDecision::Indeterminate);
        assert!(!store.directory.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn private_state_permissions_and_compaction_are_enforced() {
        let (store, root) = test_store(ReceiptQuota::default());
        assert_eq!(store.receive("delivery_1", NOW + 1, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(store.receive("delivery_2", NOW + 60, NOW).unwrap(), ReceiptDecision::Inject);
        assert_eq!(fs::metadata(&store.directory).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(store.entry_path("delivery_1")).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(store.purge_expired(NOW + 1).unwrap(), 1);
        assert_eq!(store.read_max_expires_at().unwrap(), NOW + 60);
        let _ = fs::remove_dir_all(root);
    }
}
