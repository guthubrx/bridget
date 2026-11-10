//! Autorisations d'appareil privées ; aucun état métier ni secret brut persistant.
use bridget_transport::protocol::CommunicationProject;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

const PAIRING_TTL: i64 = 300;
const DEVICE_TTL: i64 = 30 * 86_400;
const ROTATION_REPLAY_TTL: i64 = 600;
const MAX_DEVICES: i64 = 16;
const MAX_RETAINED: i64 = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Operation {
    Read,
    Message,
    TaskRead,
    TaskCreate,
    TaskContinue,
    TaskCancel,
    TaskRespond,
    DevicesManage,
    SpecialistsManage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub all_machines: bool,
    pub machines: Vec<String>,
    pub all_projects: bool,
    pub projects: Vec<CommunicationProject>,
    pub operations: Vec<Operation>,
}

impl Scope {
    /// L'ancien --all accordait exactement ces sept opérations globales.
    /// Conserver son administration complète après ajout des spécialistes,
    /// sans promouvoir un appareil partiel ou un seul droit DevicesManage.
    fn legacy_global_administrator(&self) -> bool {
        const LEGACY: [Operation; 7] = [Operation::Read, Operation::Message,
            Operation::TaskRead, Operation::TaskContinue, Operation::TaskCancel,
            Operation::TaskRespond, Operation::DevicesManage];
        self.all_machines && self.all_projects
            && self.machines.is_empty() && self.projects.is_empty()
            && self.operations.len() == LEGACY.len()
            && LEGACY.iter().all(|op| self.operations.contains(op))
    }

    /// Les historiques ne portent pas encore de partition fine attestée.
    pub fn permits_federation(&self, op: Operation) -> bool {
        self.all_machines && self.all_projects && self.operations.contains(&op)
    }
    /// O(M+P+O), listes bornées : 128 machines/projets et 9 opérations.
    pub fn permits(
        &self,
        op: Operation,
        machine: &str,
        project: Option<&CommunicationProject>,
    ) -> bool {
        self.operations.contains(&op)
            && (self.all_machines || self.machines.iter().any(|m| m == machine))
            && (self.all_projects || project.is_some_and(|p| self.projects.contains(p)))
    }

    fn validate(&self) -> Result<(), StoreError> {
        if self.operations.contains(&Operation::TaskCreate)
            || self.machines.len() > 128
            || self.projects.len() > 128
            || self.operations.is_empty()
            || self.operations.len() > 9
            || (self.all_machines && !self.machines.is_empty())
            || (self.all_projects && !self.projects.is_empty())
            || self.machines.iter().any(|m| !bounded_label(m, 256))
            || self.projects.iter().any(|p| {
                !bounded_label(&p.host, 256)
                    || !Path::new(&p.root).is_absolute()
                    || p.root.len() > 4096
                    || p.root.chars().any(char::is_control)
                    || Path::new(&p.root)
                        .components()
                        .any(|c| matches!(c, std::path::Component::ParentDir))
            })
        {
            return Err(StoreError::InvalidRequest);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Device {
    pub device_id: String,
    pub label: String,
    pub scope: Scope,
    pub created_at: i64,
    pub expires_at: i64,
    pub revoked_at: Option<i64>,
}

// Introduction de SpecialistsManage, commit dadca5d0 (1 novembre 2026).
pub(crate) const SPECIALISTS_MANAGEMENT_INTRODUCED_AT: i64 = 1_793_532_131;

impl Device {
    /// Compatibilité accordée aux seules associations administratives complètes
    /// antérieures aux spécialistes. La portée stockée reste inchangée : cette
    /// administration effective est une extension explicite de compatibilité.
    /// Les contrôles d'expiration/révocation appartiennent aux frontières appelantes.
    pub(crate) fn permits_specialists_management(&self) -> bool {
        self.scope.all_machines
            && (self.scope.operations.contains(&Operation::SpecialistsManage)
                || ((1..SPECIALISTS_MANAGEMENT_INTRODUCED_AT).contains(&self.created_at)
                    && self.scope.legacy_global_administrator()))
    }
}

// Le QR est une autorisation temporaire : pas de Debug/Serialize implicite.
pub(crate) struct Pairing {
    pub pairing_id: String,
    pub credential: String,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StoreError {
    InvalidRequest,
    PairingUnavailable,
    AuthorizationRequired,
    DeviceRevoked,
    CredentialExpired,
    EnvelopeMismatch,
    DeviceLimit,
    StorageUnavailable,
}

impl StoreError {
    pub fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::PairingUnavailable => "pairing_unavailable",
            Self::AuthorizationRequired => "authorization_required",
            Self::DeviceRevoked => "device_revoked",
            Self::CredentialExpired => "credential_expired",
            Self::EnvelopeMismatch => "envelope_mismatch",
            Self::DeviceLimit => "device_limit",
            Self::StorageUnavailable => "storage_unavailable",
        }
    }
}

pub(crate) struct Store {
    conn: Connection,
}

fn storage(_: rusqlite::Error) -> StoreError {
    StoreError::StorageUnavailable
}

fn bounded_label(value: &str, limit: usize) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= limit
        && !value.chars().any(char::is_control)
}

fn valid_id(value: &str) -> bool {
    uuid::Uuid::parse_str(value)
        .is_ok_and(|id| id.to_string() == value && id.get_version() == Some(uuid::Version::Random))
}

fn valid_key(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub(super) fn random_key() -> String {
    // Deux UUID v4 générés par l'OS : 244 bits aléatoires, sans générateur maison.
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn hash_key(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn row_device(row: &rusqlite::Row<'_>) -> rusqlite::Result<Device> {
    let scope: String = row.get(2)?;
    let scope = serde_json::from_str(&scope).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(error))
    })?;
    Ok(Device {
        device_id: row.get(0)?,
        label: row.get(1)?,
        scope,
        created_at: row.get(3)?,
        expires_at: row.get(4)?,
        revoked_at: row.get(5)?,
    })
}

fn active(device: Device, now: i64) -> Result<Device, StoreError> {
    if device.revoked_at.is_some() {
        Err(StoreError::DeviceRevoked)
    } else if device.expires_at <= now {
        Err(StoreError::CredentialExpired)
    } else {
        device.scope.validate()?;
        Ok(device)
    }
}

impl Store {
    /// O(profondeur du chemin), trois fichiers SQLite privés ; aucun parcours de namespace.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        use std::os::unix::fs::OpenOptionsExt;
        let parent = path.parent().ok_or(StoreError::InvalidRequest)?;
        crate::environment::validate_private_directory_if_present(parent)
            .map_err(|_| StoreError::StorageUnavailable)?;
        crate::environment::ensure_private_directory(parent)
            .map_err(|_| StoreError::StorageUnavailable)?;
        crate::environment::validate_state_file(path, false)
            .map_err(|_| StoreError::StorageUnavailable)?;
        for suffix in ["-wal", "-shm"] {
            let sidecar = std::path::PathBuf::from(format!("{}{}", path.display(), suffix));
            crate::environment::validate_state_file(&sidecar, false)
                .map_err(|_| StoreError::StorageUnavailable)?;
        }
        std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .map_err(|_| StoreError::StorageUnavailable)?;
        Self::from_connection(Connection::open(path).map_err(storage)?)
    }

    fn from_connection(conn: Connection) -> Result<Self, StoreError> {
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(storage)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(storage)?;
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(storage)?;
        if version > 2 {
            return Err(StoreError::StorageUnavailable);
        }
        if version == 2 {
            return Ok(Self { conn });
        }
        if version == 1 {
            conn.execute_batch("BEGIN IMMEDIATE; CREATE TABLE IF NOT EXISTS mobile_notifications(device_id TEXT PRIMARY KEY REFERENCES mobile_devices(device_id) ON DELETE CASCADE, token TEXT NOT NULL UNIQUE); PRAGMA user_version=2; COMMIT;").map_err(storage)?;
            return Ok(Self { conn });
        }
        conn.execute_batch(
            "BEGIN IMMEDIATE;
             CREATE TABLE IF NOT EXISTS mobile_devices(
               device_id TEXT PRIMARY KEY, label TEXT NOT NULL, scope TEXT NOT NULL,
               key_hash TEXT NOT NULL UNIQUE CHECK(length(key_hash)=64),
               created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL, revoked_at INTEGER
             ) STRICT;
             CREATE TABLE IF NOT EXISTS mobile_pairings(
               pairing_id TEXT PRIMARY KEY, credential_hash TEXT NOT NULL UNIQUE CHECK(length(credential_hash)=64),
               scope TEXT NOT NULL, expires_at INTEGER NOT NULL,
               attempt_id TEXT, binding_hash TEXT, device_id TEXT REFERENCES mobile_devices(device_id) ON DELETE CASCADE
             ) STRICT;
             CREATE TABLE IF NOT EXISTS mobile_rotations(
               from_hash TEXT PRIMARY KEY CHECK(length(from_hash)=64),
               to_hash TEXT NOT NULL CHECK(length(to_hash)=64), request_id TEXT NOT NULL,
               device_id TEXT NOT NULL REFERENCES mobile_devices(device_id) ON DELETE CASCADE,
               replay_until INTEGER NOT NULL
             ) STRICT;
             CREATE INDEX IF NOT EXISTS mobile_pairings_expiry ON mobile_pairings(expires_at);
             CREATE INDEX IF NOT EXISTS mobile_devices_expiry ON mobile_devices(expires_at);
             CREATE TABLE IF NOT EXISTS mobile_notifications(device_id TEXT PRIMARY KEY REFERENCES mobile_devices(device_id) ON DELETE CASCADE, token TEXT NOT NULL UNIQUE);
             PRAGMA user_version=2;
             COMMIT;"
        ).map_err(storage)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub(super) fn memory() -> Self {
        Self::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    /// O(D), D<=16 ; ce jeton de réception ne porte aucune autorité Bridget.
    pub fn set_notifications(
        &self,
        device: &Device,
        token: Option<&str>,
        now: i64,
    ) -> Result<(), StoreError> {
        let current = self.conn.query_row("SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices WHERE device_id=?1", [&device.device_id], row_device).map_err(storage)?;
        let current = active(current, now)?;
        if !current.scope.permits_federation(Operation::Read)
            || !current.scope.permits_federation(Operation::TaskRead)
        {
            return Err(StoreError::InvalidRequest);
        }
        match token {
            None => {
                self.conn
                    .execute(
                        "DELETE FROM mobile_notifications WHERE device_id=?1",
                        [&device.device_id],
                    )
                    .map_err(storage)?;
            }
            Some(token) => {
                if !(20..=4096).contains(&token.len())
                    || !token
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-:.".contains(&b))
                {
                    return Err(StoreError::InvalidRequest);
                }
                self.conn.execute("DELETE FROM mobile_notifications WHERE device_id IN (SELECT device_id FROM mobile_devices WHERE revoked_at IS NOT NULL OR expires_at<=?1)", [now]).map_err(storage)?;
                // Un même téléphone peut être réassocié sans changer de jeton FCM.
                // REPLACE déplace seulement la sonnette, jamais les droits de l’ancien appareil.
                self.conn.execute("INSERT OR REPLACE INTO mobile_notifications(device_id,token) VALUES (?1,?2)", params![device.device_id,token]).map_err(storage)?;
            }
        }
        Ok(())
    }

    /// O(D), D<=16 ; relecture des droits avant chaque envoi, jamais de corps métier.
    pub fn notification_targets(&self, now: i64) -> Result<Vec<(String, String)>, StoreError> {
        let mut stmt=self.conn.prepare("SELECT d.device_id,d.label,d.scope,d.created_at,d.expires_at,d.revoked_at,n.token FROM mobile_devices d JOIN mobile_notifications n USING(device_id) WHERE d.revoked_at IS NULL AND d.expires_at>?1 LIMIT 16").map_err(storage)?;
        let rows = stmt
            .query_map([now], |row| {
                Ok((row_device(row)?, row.get::<_, String>(6)?))
            })
            .map_err(storage)?;
        let mut targets = Vec::new();
        for row in rows {
            let (d, t) = row.map_err(storage)?;
            if d.scope.permits_federation(Operation::Read)
                && d.scope.permits_federation(Operation::TaskRead)
            {
                targets.push((d.device_id, t));
            }
        }
        Ok(targets)
    }

    pub fn remove_notification_token(
        &self,
        device_id: &str,
        token: &str,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "DELETE FROM mobile_notifications WHERE device_id=?1 AND token=?2",
                params![device_id, token],
            )
            .map_err(storage)?;
        Ok(())
    }

    /// O(D+P), D/P<=256 ; suppression uniquement d'autorisations déjà expirées.
    pub fn issue_pairing(&mut self, scope: &Scope, now: i64) -> Result<Pairing, StoreError> {
        scope.validate()?;
        let expires_at = now
            .checked_add(PAIRING_TTL)
            .filter(|_| now >= 0)
            .ok_or(StoreError::InvalidRequest)?;
        let encoded = serde_json::to_string(scope).map_err(|_| StoreError::InvalidRequest)?;
        if encoded.len() > 16 * 1024 {
            return Err(StoreError::InvalidRequest);
        }
        let grant = Pairing {
            pairing_id: uuid::Uuid::new_v4().to_string(),
            credential: random_key(),
            expires_at,
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        tx.execute("DELETE FROM mobile_rotations WHERE replay_until<=?1", [now])
            .map_err(storage)?;
        tx.execute(
            "DELETE FROM mobile_pairings WHERE expires_at<=?1 AND device_id IS NULL",
            [now],
        )
        .map_err(storage)?;
        tx.execute(
            "DELETE FROM mobile_devices WHERE expires_at<=?1",
            [now.saturating_sub(DEVICE_TTL)],
        )
        .map_err(storage)?;
        let devices: i64 = tx
            .query_row("SELECT count(*) FROM mobile_devices", [], |r| r.get(0))
            .map_err(storage)?;
        let pending: i64 = tx
            .query_row("SELECT count(*) FROM mobile_pairings", [], |r| r.get(0))
            .map_err(storage)?;
        if devices >= MAX_RETAINED || pending >= MAX_RETAINED {
            return Err(StoreError::DeviceLimit);
        }
        tx.execute("INSERT INTO mobile_pairings(pairing_id,credential_hash,scope,expires_at) VALUES(?1,?2,?3,?4)",
            params![grant.pairing_id,hash_key(&grant.credential),encoded,expires_at]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(grant)
    }

    /// O(log P+log D), plus count borné à 256 ; consommation et appareil atomiques.
    pub fn pair(
        &mut self,
        pairing: &str,
        attempt: &str,
        key: &str,
        label: &str,
        now: i64,
    ) -> Result<Device, StoreError> {
        if !valid_key(pairing)
            || !valid_key(key)
            || !valid_id(attempt)
            || !bounded_label(label, 128)
            || now < 0
        {
            return Err(StoreError::InvalidRequest);
        }
        let fingerprint = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(attempt, hash_key(key), label))
                    .map_err(|_| StoreError::InvalidRequest)?
            )
        );
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let row = tx.query_row(
            "SELECT scope,expires_at,binding_hash,device_id FROM mobile_pairings WHERE credential_hash=?1",
            [hash_key(pairing)],
            |r| Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?))
        ).optional().map_err(storage)?.ok_or(StoreError::PairingUnavailable)?;
        if let Some(id) = row.3 {
            if row.2.as_deref() != Some(&fingerprint) {
                return Err(StoreError::PairingUnavailable);
            }
            let device = tx.query_row(
                "SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices WHERE device_id=?1 AND key_hash=?2",
                params![id,hash_key(key)],row_device
            ).optional().map_err(storage)?.ok_or(StoreError::PairingUnavailable)?;
            return active(device, now);
        }
        if row.1 <= now {
            return Err(StoreError::PairingUnavailable);
        }
        let scope: Scope =
            serde_json::from_str(&row.0).map_err(|_| StoreError::StorageUnavailable)?;
        scope.validate()?;
        let count: i64 = tx
            .query_row(
                "SELECT count(*) FROM mobile_devices WHERE revoked_at IS NULL AND expires_at>?1",
                [now],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if count >= MAX_DEVICES {
            return Err(StoreError::DeviceLimit);
        }
        let device = Device {
            device_id: uuid::Uuid::new_v4().to_string(),
            label: label.to_owned(),
            scope,
            created_at: now,
            expires_at: now
                .checked_add(DEVICE_TTL)
                .ok_or(StoreError::InvalidRequest)?,
            revoked_at: None,
        };
        tx.execute(
            "INSERT INTO mobile_devices(device_id,label,scope,key_hash,created_at,expires_at) VALUES(?1,?2,?3,?4,?5,?6)",
            params![device.device_id,device.label,row.0,hash_key(key),now,device.expires_at]
        ).map_err(storage)?;
        tx.execute(
            "UPDATE mobile_pairings SET attempt_id=?1,binding_hash=?2,device_id=?3 WHERE credential_hash=?4",
            params![attempt,fingerprint,device.device_id,hash_key(pairing)]
        ).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(device)
    }

    /// O(log D), D<=256 ; la révocation est relue à chaque accès.
    pub fn authenticate(&self, key: &str, now: i64) -> Result<Device, StoreError> {
        if !valid_key(key) || now < 0 {
            return Err(StoreError::AuthorizationRequired);
        }
        let device = self.conn.query_row(
            "SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices WHERE key_hash=?1",
            [hash_key(key)], row_device
        ).optional().map_err(storage)?.ok_or(StoreError::AuthorizationRequired)?;
        active(device, now)
    }

    /// O(log D + log R), bornes existantes ; sujet de débit, jamais autorité d’opération.
    pub fn renewal_subject(&self, key: &str, now: i64) -> Result<Device, StoreError> {
        match self.authenticate(key, now) {
            Ok(device) => return Ok(device),
            Err(StoreError::AuthorizationRequired) => {},
            Err(error) => return Err(error),
        }
        if !valid_key(key) || now < 0 { return Err(StoreError::AuthorizationRequired); }
        let device=self.conn.query_row(
            "SELECT d.device_id,d.label,d.scope,d.created_at,d.expires_at,d.revoked_at FROM mobile_rotations r JOIN mobile_devices d ON d.device_id=r.device_id AND d.key_hash=r.to_hash WHERE r.from_hash=?1 AND r.replay_until>?2",
            params![hash_key(key),now],row_device
        ).optional().map_err(storage)?.ok_or(StoreError::AuthorizationRequired)?;
        active(device,now)
    }

    /// O(D), D<=256 ; aucun hash de clé n'est projeté.
    pub fn devices(&self) -> Result<Vec<Device>, StoreError> {
        self.conn.prepare(
            "SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices ORDER BY created_at,device_id"
        ).map_err(storage)?.query_map([],row_device).map_err(storage)?
            .collect::<Result<Vec<_>,_>>().map_err(storage)
    }

    /// O(log D), D<=256 ; idempotente, irréversible par ce contrat.
    pub fn revoke(&mut self, id: &str, now: i64) -> Result<bool, StoreError> {
        if !valid_id(id) || now < 0 {
            return Err(StoreError::InvalidRequest);
        }
        self.conn
            .execute(
                "UPDATE mobile_devices SET revoked_at=coalesce(revoked_at,?1) WHERE device_id=?2",
                params![now, id],
            )
            .map(|n| n == 1)
            .map_err(storage)
    }

    /// O(log D+R), D<=256 et R<=1024 ; le rejeu ne réactive jamais l'ancienne clé.
    pub fn renew(
        &mut self,
        old: &str,
        new: &str,
        request: &str,
        now: i64,
    ) -> Result<Device, StoreError> {
        if !valid_key(old) || !valid_key(new) || old == new || !valid_id(request) || now < 0 {
            return Err(StoreError::InvalidRequest);
        }
        let old_hash = hash_key(old);
        let new_hash = hash_key(new);
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let replay = tx.query_row(
            "SELECT to_hash,request_id,device_id,replay_until FROM mobile_rotations WHERE from_hash=?1",
            [&old_hash], |r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?))
        ).optional().map_err(storage)?;
        if let Some((to, prior, device_id, until)) = replay {
            if until <= now {
                return Err(StoreError::AuthorizationRequired);
            }
            if to != new_hash || prior != request {
                return Err(StoreError::EnvelopeMismatch);
            }
            let device = tx.query_row(
                "SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices WHERE device_id=?1 AND key_hash=?2",
                params![device_id,new_hash],row_device
            ).optional().map_err(storage)?.ok_or(StoreError::AuthorizationRequired)?;
            return active(device, now);
        }
        let mut device = tx.query_row(
            "SELECT device_id,label,scope,created_at,expires_at,revoked_at FROM mobile_devices WHERE key_hash=?1",
            [&old_hash],row_device
        ).optional().map_err(storage)?.ok_or(StoreError::AuthorizationRequired)?;
        device = active(device, now)?;
        tx.execute("DELETE FROM mobile_rotations WHERE replay_until<=?1", [now])
            .map_err(storage)?;
        let count: i64 = tx
            .query_row("SELECT count(*) FROM mobile_rotations", [], |r| r.get(0))
            .map_err(storage)?;
        if count >= 1024 {
            return Err(StoreError::DeviceLimit);
        }
        device.expires_at = now
            .checked_add(DEVICE_TTL)
            .ok_or(StoreError::InvalidRequest)?;
        tx.execute(
            "UPDATE mobile_devices SET key_hash=?1,expires_at=?2 WHERE device_id=?3",
            params![new_hash, device.expires_at, device.device_id],
        )
        .map_err(storage)?;
        tx.execute("INSERT INTO mobile_rotations(from_hash,to_hash,request_id,device_id,replay_until) VALUES(?1,?2,?3,?4,?5)",
            params![old_hash,new_hash,request,device.device_id,now.checked_add(ROTATION_REPLAY_TTL).ok_or(StoreError::InvalidRequest)?]
        ).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(device)
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;
