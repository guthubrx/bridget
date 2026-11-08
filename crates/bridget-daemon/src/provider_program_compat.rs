//! Compatibilité du programme, distincte des modèles, comptes et permissions.
//! Le contrôle physique est appelé uniquement par le collecteur supervisé220.
//! Le préflight et le wrapper ne font que lire une preuve de l'image courante.
use crate::lifecycle::SourceEnvironment;
use bridget_transport::protocol::{ProviderObservation, ProviderOperation};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const CONTRACT: &str = "bridget-codex-program-v1";
fn contract_id() -> String {
    format!("{CONTRACT}-{}", env!("BRIDGET_PROGRAM_CONTRACT_DIGEST"))
}
thread_local! {static SCHEMA_UNVERIFIED:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
fn unknown_schema() -> bool {
    SCHEMA_UNVERIFIED.set(true);
    false
}
struct TemporaryProof(PathBuf);
impl Drop for TemporaryProof {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

const MAX_SCHEMA: u64 = 4 * 1024 * 1024;
#[derive(Clone, PartialEq, Eq)]
struct ImageStamp {
    dev: u64,
    ino: u64,
    bytes: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}
fn stamp(m: &fs::Metadata) -> ImageStamp {
    ImageStamp {
        dev: m.dev(),
        ino: m.ino(),
        bytes: m.len(),
        modified: (m.mtime(), m.mtime_nsec()),
        changed: (m.ctime(), m.ctime_nsec()),
    }
}
type DigestCache = VecDeque<(PathBuf, ImageStamp, String)>;
static DIGESTS: OnceLock<Mutex<DigestCache>> = OnceLock::new();

/// Lecture par blocs ; le cache borné n'est réutilisé que pour le même inode,
/// taille, mtime ET ctime. Le lien est résolu à nouveau à chaque contrôle.
pub(crate) fn program_digest(path: &Path) -> Result<String, &'static str> {
    let mut f = File::open(path).map_err(|_| "program_unreadable")?;
    let metadata = f.metadata().map_err(|_| "program_unreadable")?;
    if !metadata.is_file() || metadata.len() > 512 * 1024 * 1024 {
        return Err("program_unreadable");
    }
    let before = stamp(&metadata);
    let cache = DIGESTS.get_or_init(|| Mutex::new(VecDeque::new()));
    if let Some(d) = cache
        .lock()
        .map_err(|_| "program_check_unavailable")?
        .iter()
        .find(|(p, m, _)| p == path && *m == before)
        .map(|(_, _, d)| d.clone())
    {
        if fs::metadata(path).is_ok_and(|m| stamp(&m) == before) {
            return Ok(d);
        }
        return Err("program_changed_during_check");
    }
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buffer).map_err(|_| "program_unreadable")?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    if f.metadata().map(|m| stamp(&m)).ok() != Some(before.clone())
        || fs::metadata(path).map(|m| stamp(&m)).ok() != Some(before.clone())
    {
        return Err("program_changed_during_check");
    }
    let digest = format!("{:x}", hash.finalize());
    let mut entries = cache.lock().map_err(|_| "program_check_unavailable")?;
    entries.retain(|(p, _, _)| p != path);
    if entries.len() >= 32 {
        entries.pop_front();
    }
    entries.push_back((path.into(), before, digest.clone()));
    Ok(digest)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UpgradeRefusal {
    Pending,
    Incompatible,
    Unavailable,
}
thread_local! {static FAILURE:std::cell::Cell<Option<UpgradeRefusal>>=const{std::cell::Cell::new(None)};}
pub(crate) fn record_failure(code: &str) {
    FAILURE.set(Some(match code {
        "program_upgrade_check_pending" => UpgradeRefusal::Pending,
        "program_protocol_incompatible" => UpgradeRefusal::Incompatible,
        _ => UpgradeRefusal::Unavailable,
    }));
}
pub(crate) fn capture<T>(work: impl FnOnce() -> T) -> (T, Option<UpgradeRefusal>) {
    struct Restore(Option<UpgradeRefusal>);
    impl Drop for Restore {
        fn drop(&mut self) {
            FAILURE.set(self.0);
        }
    }
    let _restore = Restore(FAILURE.replace(None));
    let result = work();
    (result, FAILURE.get())
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CheckedContract {
    version: u8,
    contract: String,
    program: String,
    digest: String,
    binary_version: String,
    observed_at: i64,
    compatible: bool,
    verified: bool,
    operations: Vec<ProviderOperation>,
}
fn root(source: &SourceEnvironment) -> Result<PathBuf, &'static str> {
    let home = PathBuf::from(source.get("HOME").ok_or("program_check_unavailable")?);
    if !home.is_absolute() {
        return Err("program_check_unavailable");
    }
    Ok(home.join(".cache/bridget-program-compat"))
}
fn proof_path(source: &SourceEnvironment, program: &str) -> Result<PathBuf, &'static str> {
    Ok(root(source)?.join(format!("{:x}.json", Sha256::digest(program.as_bytes()))))
}
fn private(m: &fs::Metadata) -> bool {
    m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0 && !m.file_type().is_symlink()
}
/// Au-delà de ce délai, un contrôle de compatibilité qui échoue à chaque
/// essai n'est plus présenté comme en attente.
const CHECK_FAILURE_GRACE_SECS: i64 = 600;
const CHECK_FAILURE_EXCERPT_BYTES: usize = 512;

fn failure_path(source: &SourceEnvironment, program: &str) -> Result<PathBuf, &'static str> {
    Ok(root(source)?.join(format!("{:x}.failed.json", Sha256::digest(program.as_bytes()))))
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Extrait borné et imprimable de la sortie d'erreur d'un contrôle.
pub(crate) fn failure_excerpt(raw: &[u8]) -> String {
    raw.iter()
        .take(CHECK_FAILURE_EXCERPT_BYTES)
        .map(|b| if b.is_ascii_graphic() || *b == b' ' { *b as char } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn read_failure(source: &SourceEnvironment, program: &str) -> Option<Value> {
    let path = failure_path(source, program).ok()?;
    let f = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path).ok()?;
    let m = f.metadata().ok()?;
    if !m.is_file() || !private(&m) || m.len() > 16 * 1024 {
        return None;
    }
    serde_json::from_reader(f.take(16 * 1024 + 1)).ok()
}

fn check_failure_persistent(source: &SourceEnvironment, program: &str, digest: &str) -> bool {
    read_failure(source, program).is_some_and(|v| {
        v["digest"] == digest
            && v["first_failed_at"]
                .as_i64()
                .is_some_and(|first| now_secs().saturating_sub(first) >= CHECK_FAILURE_GRACE_SECS)
    })
}

/// E3.1 (244) : un échec de qualification garde sa cause et un extrait de sa
/// sortie d'erreur, au lieu d'un refus muet. Écriture privée et atomique.
pub(crate) fn record_check_failure(
    source: &SourceEnvironment,
    program: &str,
    digest: &str,
    cause: &str,
    stderr: &[u8],
) {
    let excerpt = failure_excerpt(stderr);
    eprintln!("bridget_program_check_failed244 digest={digest} cause={cause} stderr={excerpt:?}");
    let Ok(root) = root(source) else { return };
    if fs::symlink_metadata(&root).is_err() {
        use std::os::unix::fs::DirBuilderExt;
        if root.parent().is_none_or(|p| !p.exists())
            || fs::DirBuilder::new().mode(0o700).create(&root).is_err()
        {
            return;
        }
    }
    if !fs::symlink_metadata(&root).is_ok_and(|m| m.is_dir() && private(&m)) {
        return;
    }
    let now = now_secs();
    let first = read_failure(source, program)
        .filter(|v| v["digest"] == digest)
        .and_then(|v| v["first_failed_at"].as_i64())
        .unwrap_or(now);
    let value = json!({"version":1,"program":program,"digest":digest,"cause":cause,
        "first_failed_at":first,"last_failed_at":now,"stderr":excerpt});
    let Ok(target) = failure_path(source, program) else { return };
    let temp = root.join(format!(".{}.failed.tmp", uuid::Uuid::new_v4()));
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&temp)
        .and_then(|mut f| f.write_all(value.to_string().as_bytes()).and_then(|_| f.sync_all()));
    if written.is_err() || fs::rename(&temp, &target).is_err() {
        let _ = fs::remove_file(&temp);
    }
}

/// Une qualification réussie efface la fiche d'échec de ce programme.
pub(crate) fn clear_check_failure(source: &SourceEnvironment, program: &str) {
    if let Ok(path) = failure_path(source, program) {
        let _ = fs::remove_file(path);
    }
}

fn read_proof(
    source: &SourceEnvironment,
    program: &str,
    digest: &str,
) -> Result<CheckedContract, &'static str> {
    let root = root(source)?;
    let m = fs::symlink_metadata(&root).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "program_upgrade_check_pending"
        } else {
            "program_check_unavailable"
        }
    })?;
    if !m.is_dir() || !private(&m) {
        return Err("program_check_unavailable");
    }
    let p = proof_path(source, program)?;
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(p)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                // E3.2 (244) : un contrôle qui échoue durablement n'est plus
                // « en attente » ; il devient un échec nommé et réessayé.
                if check_failure_persistent(source, program, digest) {
                    "program_check_unavailable"
                } else {
                    "program_upgrade_check_pending"
                }
            } else {
                "program_check_unavailable"
            }
        })?;
    let m = f.metadata().map_err(|_| "program_check_unavailable")?;
    if !m.is_file() || !private(&m) || m.len() > 16 * 1024 {
        return Err("program_check_unavailable");
    }
    let value: CheckedContract =
        serde_json::from_reader(f.take(16 * 1024 + 1)).map_err(|_| "program_check_unavailable")?;
    if value.version != 1
        || value.contract != contract_id()
        || value.program != program
        || value.digest != digest
        || value.observed_at < 0
        || value.operations.len() > 5
    {
        return Err("program_upgrade_check_pending");
    }
    if !value.verified {
        return Err("program_check_unavailable");
    }
    if value.compatible {
        Ok(value)
    } else {
        Err("program_protocol_incompatible")
    }
}

pub(crate) fn validate_upgrade(
    program: &str,
    source: &SourceEnvironment,
    digest: &str,
) -> Result<(), &'static str> {
    read_proof(source, program, digest).map(|_| ())
}
/// L'observation de la définition reste historique. Aucune capacité n'est
/// ajoutée : intersection de ses opérations avec le contrat réellement relu.
pub(crate) fn runtime_observation(
    def: &crate::registry::AgentDefinition,
    source: &SourceEnvironment,
) -> Result<Option<ProviderObservation>, &'static str> {
    let Some(old) = def.capabilities.observed.as_ref() else {
        return Ok(None);
    };
    let digest = program_digest(Path::new(&def.command))?;
    if digest == old.binary_digest {
        return Ok(Some(old.clone()));
    }
    if def.protocol != "codex_app_server" {
        return Err("program_protocol_unverified");
    }
    let checked = read_proof(source, &def.command, &digest)?;
    let mut value = old.clone();
    value.binary_digest = digest;
    value.binary_version = checked.binary_version;
    value.contract_version = format!("{}:{}", checked.contract, &value.binary_digest);
    value
        .operations
        .retain(|op| checked.operations.contains(op));
    Ok(Some(value))
}

impl CheckedContract {
    pub(crate) fn is_compatible(&self) -> bool {
        self.compatible && self.verified
    }
    pub(crate) fn needed(
        def: &crate::registry::AgentDefinition,
        source: &SourceEnvironment,
    ) -> Result<Option<String>, &'static str> {
        let Some(old) = def.capabilities.observed.as_ref() else {
            return Ok(None);
        };
        let digest = program_digest(Path::new(&def.command))?;
        if digest == old.binary_digest {
            return Ok(None);
        }
        match read_proof(source, &def.command, &digest) {
            Ok(_) => return Ok(None),
            Err("program_protocol_incompatible") => return Err("program_protocol_incompatible"),
            Err(_) => {}
        }
        Ok(Some(digest))
    }
    pub(crate) fn from_schemas(
        directory: &Path,
        program: &str,
        digest: &str,
    ) -> Result<Self, &'static str> {
        let mut compatible = true;
        let mut verified = true;
        let mut operations = Vec::new();
        for (method, sample) in bridget_transport::codex_app_server::program_contract_requests() {
            let (schema, operation) = match method {
                "initialize" => ("v1/InitializeParams.json", None),
                "thread/start" => ("v2/ThreadStartParams.json", None),
                "turn/start" => ("v2/TurnStartParams.json", None),
                "turn/interrupt" => (
                    "v2/TurnInterruptParams.json",
                    Some(ProviderOperation::Interrupt),
                ),
                "turn/steer" => ("v2/TurnSteerParams.json", Some(ProviderOperation::Steer)),
                "thread/resume" => (
                    "v2/ThreadResumeParams.json",
                    Some(ProviderOperation::Resume),
                ),
                "thread/fork" => ("v2/ThreadForkParams.json", Some(ProviderOperation::Fork)),
                _ => unreachable!(),
            };
            let checked =
                schema_accepts_file(&directory.join(schema), &sample).and_then(|accepts| {
                    if !accepts {
                        Ok(false)
                    } else {
                        schema_accepts_file(
                            &directory.join("ClientRequest.json"),
                            &json!({"id":1,"method":method,"params":sample}),
                        )
                    }
                });
            if let Some(operation) = operation {
                match checked {
                    Ok(true) => operations.push(operation),
                    Ok(false) => {}
                    Err(_) => {
                        verified = false;
                        compatible = false;
                    }
                }
            } else {
                match checked {
                    Ok(value) => compatible &= value,
                    Err(_) => {
                        verified = false;
                        compatible = false;
                    }
                }
            }
            #[cfg(test)]
            eprintln!("compat method {method} checked={checked:?}");
        }
        Ok(Self {
            version: 1,
            contract: contract_id(),
            program: program.into(),
            digest: digest.into(),
            binary_version: "not_reported".into(),
            observed_at: 0,
            compatible,
            verified,
            operations,
        })
    }
    pub(crate) fn publish(
        mut self,
        source: &SourceEnvironment,
        initialize: &Value,
    ) -> Result<(), &'static str> {
        if program_digest(Path::new(&self.program))? != self.digest {
            return Err("program_changed_during_check");
        }
        for field in ["userAgent", "codexHome", "platformFamily", "platformOs"] {
            if initialize.get(field).and_then(Value::as_str).is_none() {
                return Err("program_protocol_incompatible");
            }
        }
        if let Some(v) = initialize
            .get("userAgent")
            .and_then(Value::as_str)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.strip_prefix("codex_cli_rs/"))
        {
            if v.len() <= 64
                && v.split('.').count() == 3
                && v.bytes().all(|b| b.is_ascii_digit() || b == b'.')
            {
                self.binary_version = v.into();
            }
        }
        self.observed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let root = root(source)?;
        let parent = root.parent().ok_or("program_check_unavailable")?;
        if !parent.exists() {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(parent)
                .map_err(|_| "program_check_unavailable")?;
        }
        match fs::symlink_metadata(&root) {
            Ok(m) if m.is_dir() && private(&m) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&root).map_err(|_| "program_check_unavailable")?;
                fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                    .map_err(|_| "program_check_unavailable")?;
            }
            _ => return Err("program_check_unavailable"),
        }
        let p = proof_path(source, &self.program)?;
        let temp = root.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
        let _temporary = TemporaryProof(temp.clone());
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp)
            .map_err(|_| "program_check_unavailable")?;
        let data = serde_json::to_vec(&self).map_err(|_| "program_check_unavailable")?;
        f.write_all(&data)
            .and_then(|_| f.sync_all())
            .map_err(|_| "program_check_unavailable")?;
        if program_digest(Path::new(&self.program))? != self.digest {
            let _ = fs::remove_file(&temp);
            return Err("program_changed_during_check");
        }
        fs::rename(temp, p)
            .and_then(|_| File::open(root)?.sync_all())
            .map_err(|_| "program_check_unavailable")
    }
}

fn schema_accepts_file(path: &Path, value: &Value) -> Result<bool, &'static str> {
    let f = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "program_schema_unavailable")?;
    let m = f.metadata().map_err(|_| "program_schema_unavailable")?;
    if !m.is_file() || m.len() > MAX_SCHEMA {
        return Err("program_schema_unavailable");
    }
    let schema: Value = serde_json::from_reader(f.take(MAX_SCHEMA + 1))
        .map_err(|_| "program_schema_unavailable")?;
    let previous = SCHEMA_UNVERIFIED.replace(false);
    let result = accepts(&schema, &schema, value, 0);
    let unknown = SCHEMA_UNVERIFIED.replace(previous);
    if unknown {
        Err("program_schema_unverified")
    } else {
        Ok(result)
    }
}
/// Sous-ensemble fermé nécessaire aux requêtes natives. Un vocabulaire nouveau
/// contraignant n'est jamais traité comme une preuve positive.
fn accepts(root: &Value, s: &Value, v: &Value, depth: u8) -> bool {
    if depth > 32 {
        return unknown_schema();
    }
    if let Some(b) = s.as_bool() {
        return b;
    }
    let Some(o) = s.as_object() else {
        return unknown_schema();
    };
    const KNOWN: &[&str] = &[
        "$schema",
        "$ref",
        "$id",
        "definitions",
        "$defs",
        "title",
        "description",
        "default",
        "deprecated",
        "type",
        "enum",
        "const",
        "anyOf",
        "oneOf",
        "allOf",
        "properties",
        "required",
        "additionalProperties",
        "items",
        "minItems",
        "maxItems",
        "minLength",
        "maxLength",
        "minimum",
        "maximum",
        "format",
        "examples",
        "readOnly",
        "writeOnly",
    ];
    if o.keys().any(|k| !KNOWN.contains(&k.as_str())) {
        return unknown_schema();
    }
    for key in ["anyOf", "oneOf", "allOf", "enum", "required"] {
        if s.get(key).is_some_and(|v| !v.is_array()) {
            return unknown_schema();
        }
    }
    for key in ["properties", "definitions", "$defs"] {
        if s.get(key).is_some_and(|v| !v.is_object()) {
            return unknown_schema();
        }
    }
    for key in ["$ref", "format"] {
        if s.get(key).is_some_and(|v| !v.is_string()) {
            return unknown_schema();
        }
    }
    for key in ["minItems", "maxItems", "minLength", "maxLength"] {
        if s.get(key).is_some_and(|v| v.as_u64().is_none()) {
            return unknown_schema();
        }
    }
    for key in ["minimum", "maximum"] {
        if s.get(key).is_some_and(|v| !v.is_number()) {
            return unknown_schema();
        }
    }
    if let Some(r) = s.get("$ref").and_then(Value::as_str) {
        let Some(r) = r.strip_prefix('#').and_then(|p| root.pointer(p)) else {
            return unknown_schema();
        };
        if !accepts(root, r, v, depth + 1) {
            return false;
        }
    }
    if let Some(values) = s.get("enum").and_then(Value::as_array) {
        if !values.contains(v) {
            return false;
        }
    }
    if s.get("const").is_some_and(|x| x != v) {
        return false;
    }
    for key in ["anyOf", "oneOf", "allOf"] {
        if let Some(a) = s.get(key).and_then(Value::as_array) {
            let n = a.iter().filter(|s| accepts(root, s, v, depth + 1)).count();
            if match key {
                "anyOf" => n == 0,
                "oneOf" => n != 1,
                _ => n != a.len(),
            } {
                return false;
            }
        }
    }
    if let Some(t) = s.get("type") {
        let matched = |t: &str| match t {
            "null" => v.is_null(),
            "string" => v.is_string(),
            "object" => v.is_object(),
            "array" => v.is_array(),
            "boolean" => v.is_boolean(),
            "integer" => v.is_i64() || v.is_u64(),
            "number" => v.is_number(),
            _ => unknown_schema(),
        };
        if !t.as_str().is_some_and(matched)
            && !t
                .as_array()
                .is_some_and(|a| a.iter().filter_map(Value::as_str).any(matched))
        {
            return false;
        }
    }
    if let Some(value) = v.as_object() {
        if s.get("required")
            .and_then(Value::as_array)
            .is_some_and(|a| {
                a.iter()
                    .any(|k| k.as_str().is_none_or(|k| !value.contains_key(k)))
            })
        {
            return false;
        }
        for (k, v) in value {
            if let Some(p) = s.get("properties").and_then(|p| p.get(k)) {
                if !accepts(root, p, v, depth + 1) {
                    return false;
                }
            } else if let Some(additional) = s.get("additionalProperties") {
                if !accepts(root, additional, v, depth + 1) {
                    return false;
                }
            }
        }
    }
    if let Some(a) = v.as_array() {
        if s.get("minItems")
            .and_then(Value::as_u64)
            .is_some_and(|n| a.len() < n as usize)
            || s.get("maxItems")
                .and_then(Value::as_u64)
                .is_some_and(|n| a.len() > n as usize)
        {
            return false;
        }
        if let Some(i) = s.get("items") {
            if a.iter().any(|v| !accepts(root, i, v, depth + 1)) {
                return false;
            }
        }
    }
    if let Some(text) = v.as_str() {
        let n = text.chars().count();
        if s.get("minLength")
            .and_then(Value::as_u64)
            .is_some_and(|m| n < m as usize)
            || s.get("maxLength")
                .and_then(Value::as_u64)
                .is_some_and(|m| n > m as usize)
        {
            return false;
        }
        if let Some(format) = s.get("format").and_then(Value::as_str) {
            if format != "uuid" {
                return unknown_schema();
            }
            if uuid::Uuid::parse_str(text).is_err() {
                return false;
            }
        }
    }
    if let Some(n) = v.as_f64() {
        if s.get("minimum")
            .and_then(Value::as_f64)
            .is_some_and(|m| n < m)
            || s.get("maximum")
                .and_then(Value::as_f64)
                .is_some_and(|m| n > m)
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (PathBuf, SourceEnvironment) {
        let p = std::env::temp_dir().join(format!("bpc-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir(&p).unwrap();
        fs::create_dir(p.join(".cache")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let source = SourceEnvironment::from([("HOME".into(), p.as_os_str().into())]);
        (p, source)
    }
    fn baseline(program: &Path) -> crate::registry::AgentDefinition {
        let r=crate::registry::AgentRegistry::from_json(&json!({"agents":{"fixture":{"command":program,"protocol":"codex_app_server","args":["app-server"],"capabilities":{"execution_paths":["codex_app_server"],"observed":{"binary_path":program,"binary_version":"old","binary_digest":"0".repeat(64),"contract_version":"historical","operations":["steer","interrupt"]}}}}}).to_string(),"/tmp/compat-registry.json").unwrap();
        r.get("fixture").unwrap().clone()
    }
    fn checked(program: &Path) -> CheckedContract {
        CheckedContract {
            version: 1,
            contract: contract_id(),
            program: program.to_string_lossy().into(),
            digest: program_digest(program).unwrap(),
            binary_version: "not_reported".into(),
            observed_at: 0,
            compatible: true,
            verified: true,
            operations: vec![ProviderOperation::Steer, ProviderOperation::Resume],
        }
    }
    fn hello() -> Value {
        json!({"userAgent":"codex_cli_rs/0.166.0 (fixture)","codexHome":"private","platformFamily":"unix","platformOs":"linux"})
    }
    #[test]
    fn spec221_program_compat_changed_image_requalified_without_profile_mutation() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program-a").unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        let def = baseline(&program);
        let original = def.clone();
        assert_eq!(
            validate_upgrade(&def.command, &source, &program_digest(&program).unwrap()),
            Err("program_upgrade_check_pending")
        );
        checked(&program).publish(&source, &hello()).unwrap();
        crate::lifecycle::launch_preflight(
            &crate::registry::AgentRegistry::from_json(
                &json!({"agents":{"fixture":def}}).to_string(),
                "/tmp/compat-registry.json",
            )
            .unwrap(),
            "fixture",
            &source,
        )
        .unwrap();
        let runtime = runtime_observation(&original, &source).unwrap().unwrap();
        assert_eq!(runtime.binary_version, "0.166.0");
        assert_eq!(runtime.operations, vec![ProviderOperation::Steer]);
        assert_eq!(
            original
                .capabilities
                .observed
                .as_ref()
                .unwrap()
                .binary_digest,
            "0".repeat(64)
        );
        fs::write(&program, b"program-b").unwrap();
        assert_eq!(
            runtime_observation(&original, &source).unwrap_err(),
            "program_upgrade_check_pending"
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec244_echec_durable_de_qualification_devient_un_refus_nomme() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program-244").unwrap();
        let name = program.to_str().unwrap();
        let digest = program_digest(&program).unwrap();
        // Échec récent : encore en attente, mais cause et extrait gardés.
        record_check_failure(&source, name, &digest, "program_unavailable",
            b"bridget socket trop longue\n\x1b[31m");
        assert_eq!(validate_upgrade(name, &source, &digest), Err("program_upgrade_check_pending"));
        let fiche = read_failure(&source, name).unwrap();
        assert_eq!(fiche["cause"], "program_unavailable");
        assert_eq!(fiche["stderr"], "bridget socket trop longue [31m");
        // Échec durable : refus nommé, plus jamais « en attente » sans fin.
        let path = failure_path(&source, name).unwrap();
        let mut old = fiche.clone();
        old["first_failed_at"] = json!(now_secs() - CHECK_FAILURE_GRACE_SECS - 1);
        fs::write(&path, old.to_string()).unwrap();
        assert_eq!(validate_upgrade(name, &source, &digest), Err("program_check_unavailable"));
        // Un nouvel échec garde la première date ; une autre image repart de zéro.
        record_check_failure(&source, name, &digest, "timed_out", b"");
        assert_eq!(read_failure(&source, name).unwrap()["first_failed_at"], old["first_failed_at"]);
        assert_eq!(validate_upgrade(name, &source, "autre"), Err("program_upgrade_check_pending"));
        // Une qualification réussie efface la fiche.
        clear_check_failure(&source, name);
        assert!(read_failure(&source, name).is_none());
    }

    #[test]
    fn spec221_program_compat_current_image_keeps_historical_operations() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let mut def = baseline(&program);
        def.capabilities.observed.as_mut().unwrap().binary_digest =
            program_digest(&program).unwrap();
        assert_eq!(
            runtime_observation(&def, &source).unwrap(),
            def.capabilities.observed
        );
        assert!(CheckedContract::needed(&def, &source).unwrap().is_none());
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_private_and_current_proof_only() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let digest = program_digest(&program).unwrap();
        checked(&program).publish(&source, &hello()).unwrap();
        let p = proof_path(&source, program.to_str().unwrap()).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        let saved = p.with_extension("saved");
        fs::rename(&p, &saved).unwrap();
        std::os::unix::fs::symlink(&saved, &p).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_unknown_schema_is_not_positive() {
        assert!(accepts(
            &json!({}),
            &json!({"type":"object","required":["x"],"properties":{"x":{"type":"string","enum":["ok"]}},"additionalProperties":false}),
            &json!({"x":"ok"}),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"object","required":["new_required"]}),
            &json!({"x":"ok"}),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"string","pattern":".*"}),
            &json!("value"),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"object","additionalProperties":{"type":"integer"}}),
            &json!({"x":"text"}),
            0
        ));
    }
    #[test]
    fn spec221_program_compat_incompatible_is_cached_without_rechecking() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let mut c = checked(&program);
        c.compatible = false;
        c.publish(&source, &hello()).unwrap();
        let def = baseline(&program);
        assert!(matches!(
            CheckedContract::needed(&def, &source),
            Err("program_protocol_incompatible")
        ));
        assert_eq!(
            runtime_observation(&def, &source).unwrap_err(),
            "program_protocol_incompatible"
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_capture_is_scoped_and_not_shared() {
        let ((), cause) = capture(|| record_failure("program_upgrade_check_pending"));
        assert_eq!(cause, Some(UpgradeRefusal::Pending));
        assert_eq!(capture(|| ()).1, None);
        capture(|| {
            record_failure("program_protocol_incompatible");
            assert_eq!(
                std::thread::spawn(|| capture(|| ()).1).join().unwrap(),
                None
            );
            assert_eq!(
                capture(|| record_failure("unknown")).1,
                Some(UpgradeRefusal::Unavailable)
            );
            assert_eq!(FAILURE.get(), Some(UpgradeRefusal::Incompatible));
        });
        assert_eq!(FAILURE.get(), None);
    }
    #[test]
    fn spec221_program_compat_unknown_dialect_is_unavailable_not_incompatible() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let schema = home.join("unknown.json");
        fs::write(&schema, json!({"type":"string","pattern":".*"}).to_string()).unwrap();
        assert_eq!(
            schema_accepts_file(&schema, &json!("text")),
            Err("program_schema_unverified")
        );
        let mut value = checked(&program);
        value.compatible = false;
        value.verified = false;
        value.publish(&source, &hello()).unwrap();
        assert_eq!(
            validate_upgrade(
                program.to_str().unwrap(),
                &source,
                &program_digest(&program).unwrap()
            ),
            Err("program_check_unavailable")
        );
        assert!(
            CheckedContract::needed(&baseline(&program), &source)
                .unwrap()
                .is_some()
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_cache_parent_and_atomic_failure_leave_no_temporary() {
        let (home, source) = fixture();
        fs::remove_dir(home.join(".cache")).unwrap();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        checked(&program).publish(&source, &hello()).unwrap();
        let path = proof_path(&source, program.to_str().unwrap()).unwrap();
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert_eq!(
            checked(&program).publish(&source, &hello()),
            Err("program_check_unavailable")
        );
        assert!(
            fs::read_dir(root(&source).unwrap()).unwrap().all(|p| !p
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_checker_revision_invalidates_proof() {
        let mut hash = Sha256::new();
        for (name, bytes) in [
            (
                "crates/bridget-daemon/src/provider_program_compat.rs",
                include_bytes!("provider_program_compat.rs").as_slice(),
            ),
            (
                "crates/bridget-daemon/src/provider_catalogue220.rs",
                include_bytes!("provider_catalogue220.rs").as_slice(),
            ),
            (
                "crates/bridget-transport/src/codex_app_server.rs",
                include_bytes!("../../bridget-transport/src/codex_app_server.rs").as_slice(),
            ),
        ] {
            hash.update(name.as_bytes());
            hash.update(bytes);
        }
        hash.update(env!("CARGO_PKG_VERSION").as_bytes());
        assert_eq!(
            format!("{:x}", hash.finalize()),
            env!("BRIDGET_PROGRAM_CONTRACT_DIGEST")
        );
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let value = checked(&program);
        value.publish(&source, &hello()).unwrap();
        let path = proof_path(&source, program.to_str().unwrap()).unwrap();
        let mut previous: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        previous["contract"] = json!("old-checker");
        fs::write(&path, previous.to_string()).unwrap();
        assert_eq!(
            validate_upgrade(
                program.to_str().unwrap(),
                &source,
                &program_digest(&program).unwrap()
            ),
            Err("program_upgrade_check_pending")
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_optional_unknown_schema_and_unreadable_parent_are_unavailable() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let schemas = home.join("schemas");
        fs::create_dir_all(schemas.join("v1")).unwrap();
        fs::create_dir_all(schemas.join("v2")).unwrap();
        for name in [
            "ClientRequest.json",
            "v1/InitializeParams.json",
            "v2/ThreadStartParams.json",
            "v2/TurnStartParams.json",
            "v2/TurnInterruptParams.json",
            "v2/TurnSteerParams.json",
            "v2/ThreadResumeParams.json",
            "v2/ThreadForkParams.json",
        ] {
            fs::write(schemas.join(name), "{}").unwrap();
        }
        fs::write(
            schemas.join("v2/TurnSteerParams.json"),
            r#"{"unsupportedKeyword":true}"#,
        )
        .unwrap();
        let digest = program_digest(&program).unwrap();
        let value =
            CheckedContract::from_schemas(&schemas, program.to_str().unwrap(), &digest).unwrap();
        assert!(!value.verified);
        assert!(!value.is_compatible());
        value.publish(&source, &hello()).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        // Real filesystem refusal; tests normally run as the unprivileged worker user.
        if unsafe { libc::geteuid() } != 0 {
            let parent = home.join(".cache");
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap();
            let result = validate_upgrade(program.to_str().unwrap(), &source, &digest);
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(result, Err("program_check_unavailable"));
        }
        fs::remove_dir_all(home).unwrap();
    }
}
