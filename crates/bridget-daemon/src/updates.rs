//! Préférence humaine et fenêtre d'activation exclusive. Aucun téléchargement sous State.
use bridget_transport::protocol::{
    MobileAdministrationRequest as A, MobileAdministrationResult as R, UpdateRequest as Q,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::os::unix::fs::OpenOptionsExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn yes() -> bool {
    true
}
static FILE_LOCK: Mutex<()> = Mutex::new(());
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    #[serde(default = "yes")]
    automatic: bool,
    #[serde(default)]
    hosts: BTreeMap<String, Value>,
    #[serde(default)]
    check_generation: u64,
    #[serde(default)]
    install_target: Option<String>,
    #[serde(default)]
    granted_devices: BTreeSet<String>,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            automatic: true,
            hosts: BTreeMap::new(),
            check_generation: 0,
            install_target: None,
            granted_devices: BTreeSet::new(),
        }
    }
}
fn read<T: serde::de::DeserializeOwned>(path: &Path) -> io::Result<Option<T>> {
    crate::environment::validate_state_file(path, false).map_err(io::Error::other)?;
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut bytes = Vec::new();
    file.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return Err(io::Error::other("update_state_too_large"));
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(io::Error::other)
}
fn write<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    crate::environment::validate_state_file(path, false).map_err(io::Error::other)?;
    let temp = path.with_file_name(format!(".update-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        f.write_all(&serde_json::to_vec(value)?)?;
        f.sync_all()?;
        fs::rename(&temp, path)?;
        fs::File::open(
            path.parent()
                .ok_or_else(|| io::Error::other("update_root_missing"))?,
        )?
        .sync_all()
    })();
    let _ = fs::remove_file(temp);
    result
}
/// The grant is separate from legacy device scopes so rollback can still read them.
pub(crate) fn grant_device(root: &Path, device_id: &str) -> Result<bool, &'static str> {
    let devices = crate::mobile::store::Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.devices())
        .map_err(|e| e.code())?;
    let current = now();
    let active = |id: &str| {
        devices
            .iter()
            .any(|d| d.device_id == id && d.revoked_at.is_none() && d.expires_at > current)
    };
    if !active(device_id) {
        return Ok(false);
    }
    let _lock = FILE_LOCK.lock().map_err(|_| "update_state_unavailable")?;
    let path = root.join("updates.json");
    let mut p: Policy = read(&path)
        .map_err(|_| "update_state_unavailable")?
        .unwrap_or_default();
    p.granted_devices.retain(|id| active(id));
    if p.granted_devices.len() >= 256 && !p.granted_devices.contains(device_id) {
        return Err("update_state_unavailable");
    }
    p.granted_devices.insert(device_id.into());
    write(&path, &p).map_err(|_| "update_state_unavailable")?;
    Ok(true)
}
pub(crate) fn device_granted(root: &Path, device_id: &str) -> Result<bool, &'static str> {
    let _lock = FILE_LOCK.lock().map_err(|_| "update_state_unavailable")?;
    let p: Policy = read(&root.join("updates.json"))
        .map_err(|_| "update_state_unavailable")?
        .unwrap_or_default();
    Ok(p.granted_devices.contains(device_id))
}
pub(crate) fn policy(root: &Path, request: &Q) -> Result<Value, &'static str> {
    let _lock = FILE_LOCK.lock().map_err(|_| "update_state_unavailable")?;
    let path = root.join("updates.json");
    let mut p: Policy = read(&path)
        .map_err(|_| "update_state_unavailable")?
        .unwrap_or_default();
    match request {
        Q::Status => {}
        Q::Configure { automatic } => {
            p.automatic = *automatic;
            if !automatic {
                p.install_target = None;
            }
        }
        Q::Check => {
            p.check_generation = p
                .check_generation
                .checked_add(1)
                .ok_or("update_state_unavailable")?
        }
        Q::Install { version: target } => {
            if version(target).is_none() {
                return Err("invalid_request");
            }
            p.install_target = Some(target.clone());
            p.check_generation = p
                .check_generation
                .checked_add(1)
                .ok_or("update_state_unavailable")?;
        }
        Q::Report {
            host,
            installed,
            available,
            stage,
            client_compatibility,
        } => {
            if host.is_empty()
                || host.len() > 128
                || !host
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                || version(installed).is_none()
                || available.as_ref().is_some_and(|v| version(v).is_none())
                || (!p.hosts.contains_key(host) && p.hosts.len() >= 32)
            {
                return Err("invalid_request");
            }
            if client_compatibility.as_ref().is_some_and(|v| !crate::client_compatibility::valid_report(v,host)) {
                return Err("invalid_request");
            }
            let mut report=json!({"installed":installed,"available":available,"stage":stage,"observed_at":now()});
            if let Some(clients)=client_compatibility {report["client_compatibility"]=clients.clone();}
            p.hosts.insert(host.clone(),report);
        }
        _ => return Err("invalid_request"),
    }
    if !matches!(request, Q::Status) {
        write(&path, &p).map_err(|_| "update_state_unavailable")?;
    }
    Ok(
        json!({"status":"ok","automatic":p.automatic,"hosts":p.hosts,"check_generation":p.check_generation,"install_target":p.install_target,"source":"github_stable","core_version":env!("CARGO_PKG_VERSION"),"observed_at":now()}),
    )
}
pub(crate) fn version(value: &str) -> Option<[u16; 3]> {
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut v = [0; 3];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty()
            || p.len() > 3
            || !p.bytes().all(|b| b.is_ascii_digit())
            || p.len() > 1 && p.starts_with('0')
        {
            return None;
        }
        v[i] = p.parse().ok()?;
    }
    Some(v)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lease {
    token: String,
    expires_at: i64,
}
#[derive(Default)]
struct Inner {
    active: usize,
    lease: Option<Lease>,
}
pub(crate) struct Gate {
    inner: Mutex<Inner>,
    path: PathBuf,
}
pub(crate) struct Activity(Arc<Gate>);
impl Drop for Activity {
    fn drop(&mut self) {
        let mut i = self.0.inner.lock().unwrap_or_else(|e| e.into_inner());
        i.active = i.active.saturating_sub(1);
    }
}
impl Gate {
    pub(crate) fn open(root: &Path) -> io::Result<Arc<Self>> {
        let path = root.join("update-activation.json");
        let lease: Option<Lease> = read(&path)?;
        if lease
            .as_ref()
            .is_some_and(|l| uuid::Uuid::parse_str(&l.token).is_err() || l.expires_at > now() + 180)
        {
            return Err(io::Error::other("update_lease_invalid"));
        }
        Ok(Arc::new(Self {
            inner: Mutex::new(Inner { active: 0, lease }),
            path,
        }))
    }
    pub(crate) fn enter(self: &Arc<Self>) -> Option<Activity> {
        let mut i = self.inner.lock().ok()?;
        if i.lease.as_ref().is_some_and(|l| l.expires_at > now()) {
            return None;
        }
        i.active += 1;
        Some(Activity(self.clone()))
    }
    pub(crate) fn prepare(&self, token: &str) -> Result<Value, &'static str> {
        if uuid::Uuid::parse_str(token).is_err() {
            return Err("invalid_request");
        }
        let mut i = self.inner.lock().map_err(|_| "update_state_unavailable")?;
        if i.active != 0 {
            return Err("update_operations_active");
        }
        if let Some(l) = &i.lease {
            if l.expires_at > now() {
                return if l.token == token {
                    Ok(json!({"status":"ok","expires_at":l.expires_at}))
                } else {
                    Err("update_activation_active")
                };
            }
        }
        let lease = Lease {
            token: token.into(),
            expires_at: now() + 180,
        };
        write(&self.path, &lease).map_err(|_| "update_state_unavailable")?;
        let result = json!({"status":"ok","expires_at":lease.expires_at});
        i.lease = Some(lease);
        Ok(result)
    }
    pub(crate) fn finish(&self, token: &str) -> Result<Value, &'static str> {
        let mut i = self.inner.lock().map_err(|_| "update_state_unavailable")?;
        if !i.lease.as_ref().is_some_and(|l| l.token == token) {
            return Err("update_lease_mismatch");
        }
        match fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(_) => return Err("update_state_unavailable"),
        }
        i.lease = None;
        Ok(json!({"status":"ok"}))
    }
}
pub(crate) fn run(args: &[String]) {
    if args == ["watch"] || args == ["once"] {
        use std::os::unix::process::CommandExt;
        let error = std::process::Command::new("python3")
            .arg("-c")
            .arg(include_str!("../../../scripts/update.py"))
            .env(
                "BRIDGET_UPDATE_WATCH",
                if args == ["watch"] { "1" } else { "0" },
            )
            .exec();
        eprintln!("update_runtime_unavailable: {error}");
        std::process::exit(1);
    }

    use std::io::IsTerminal;
    if matches!(
        args.first().map(String::as_str),
        Some("grant" | "auto" | "install")
    ) && !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal())
    {
        eprintln!(
            "human_terminal_required: utilisez les paramètres de l’application ou un terminal humain"
        );
        std::process::exit(1);
    }
    let result = (|| -> Result<R, &'static str> {
        let request = match args
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["status"] => A::Updates { request: Q::Status },
            ["check"] => A::Updates { request: Q::Check },
            ["install", version] => A::Updates {
                request: Q::Install {
                    version: (*version).into(),
                },
            },
            ["auto", v @ ("on" | "off")] => A::Updates {
                request: Q::Configure {
                    automatic: *v == "on",
                },
            },
            ["prepare", token] => A::Updates {
                request: Q::Prepare {
                    token: (*token).into(),
                },
            },
            ["finish", token] => A::Updates {
                request: Q::Finish {
                    token: (*token).into(),
                },
            },
            ["report", path] => {
                let q: Q = read(Path::new(path))
                    .map_err(|_| "update_report_invalid")?
                    .ok_or("update_report_invalid")?;
                if !matches!(q, Q::Report { .. }) {
                    return Err("invalid_request");
                }
                A::Updates { request: q }
            }
            ["grant", device_id] => A::GrantUpdates {
                device_id: (*device_id).into(),
            },
            _ => {
                return Err(
                    "usage: bridget updates status | auto on/off | prepare TOKEN | finish TOKEN | report FILE | grant DEVICE",
                );
            }
        };
        let ns = crate::environment::Namespace::from_environment()
            .map_err(|_| "namespace_unavailable")?;
        crate::mobile::admin_connection(&ns.socket, request)
    })();
    match result {
        Ok(R::Updates { value }) => println!("{value}"),
        Ok(R::Error { code }) => {
            eprintln!("{}", json!({"status":"refused","code":code}));
            std::process::exit(1)
        }
        Err(code) => {
            eprintln!("{}", json!({"status":"refused","code":code}));
            std::process::exit(1)
        }
        Ok(other) => println!("{}", serde_json::to_string(&other).unwrap()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        let r = std::env::temp_dir().join(format!("bridget179-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&r).unwrap();
        r
    }
    #[test]
    fn updates179_barrier_survives_restart_and_excludes_concurrent_work() {
        let r = root();
        let g = Gate::open(&r).unwrap();
        let a = g.enter().unwrap();
        let token = uuid::Uuid::new_v4().to_string();
        assert_eq!(g.prepare(&token).unwrap_err(), "update_operations_active");
        drop(a);
        g.prepare(&token).unwrap();
        assert!(g.enter().is_none());
        let after = Gate::open(&r).unwrap();
        assert!(after.enter().is_none());
        assert_eq!(after.finish("wrong").unwrap_err(), "update_lease_mismatch");
        after.finish(&token).unwrap();
        assert!(after.enter().is_some());
        fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn updates179_default_on_and_explicit_off_persist() {
        let r = root();
        assert_eq!(policy(&r, &Q::Status).unwrap()["automatic"], true);
        policy(&r, &Q::Configure { automatic: false }).unwrap();
        assert_eq!(policy(&r, &Q::Status).unwrap()["automatic"], false);
        fs::write(r.join("updates.json"), "invalid").unwrap();
        assert!(policy(&r, &Q::Status).is_err());
        fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn updates179_versions_do_not_sort_build_identifiers() {
        assert!(version("0.10.0") > version("0.9.99"));
        for bad in ["0.5.0-dev", "r10e", "01.0.0", "1.2", "1.2.3.4", "1000.0.0"] {
            assert!(version(bad).is_none());
        }
    }
    #[test]
    fn updates179_manual_target_is_exact_and_disabling_cancels_it() {
        let r = root();
        policy(&r, &Q::Configure { automatic: false }).unwrap();
        let value = policy(
            &r,
            &Q::Install {
                version: "0.6.0".into(),
            },
        )
        .unwrap();
        assert_eq!(value["automatic"], false);
        assert_eq!(value["install_target"], "0.6.0");
        assert!(
            policy(
                &r,
                &Q::Install {
                    version: "main".into()
                }
            )
            .is_err()
        );
        let value = policy(&r, &Q::Configure { automatic: false }).unwrap();
        assert!(value["install_target"].is_null());
        fs::remove_dir_all(r).unwrap();
    }
}
