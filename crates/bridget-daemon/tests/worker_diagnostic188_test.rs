#![cfg(target_os = "linux")]
//! Vérifie l'entrée CLI réelle, sans service ni fournisseur.
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("bridget188-cli-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&p).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        Self(p)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn diagnostic(root: &Path, namespace: &Path) -> Output {
    diagnostic_with_path(root, namespace, "/usr/bin:/bin")
}
fn diagnostic_with_path(root: &Path, namespace: &Path, path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args([
            "worker",
            "diagnose",
            "--machine-id",
            "cartae",
            "--worker-id",
            "worker-cartae",
            "--provisioning-revision",
            "1",
        ])
        .arg("--credential-file")
        .arg(root.join("missing-credential"))
        .arg("--state-root")
        .arg(root.join("state"))
        .arg("--registry-file")
        .arg(root.join("registry"))
        .arg("--master-socket")
        .arg(root.join("socket"))
        .args([
            "--command-id",
            "command188",
            "--task-id",
            "task188",
            "--instance-id",
            "instance188",
            "--generation",
            "1",
        ])
        .env("PATH", path)
        .env("BRIDGET_HOME", namespace)
        .env_remove("BRIDGET_SOCKET")
        .env_remove("BRIDGET_AGENT_ID_FILE")
        .env_remove("BRIDGET_AGENT_INSTANCE_ID")
        .output()
        .unwrap()
}
#[test]
fn spec188_cli_invalid_namespace_closes_before_worker_inputs() {
    let f = Fixture::new();
    let r = diagnostic(&f.0, Path::new("relative-invalid-namespace"));
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    assert!(String::from_utf8_lossy(&r.stderr).contains("namespace_configuration_invalid"));
    assert!(!f.0.join("state").exists());
}
#[test]
fn spec188_cli_invalid_delegated_marker_also_closes() {
    let f = Fixture::new();
    let markers = f.0.join("delegated-pids");
    fs::create_dir(&markers).unwrap();
    fs::set_permissions(&markers, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        markers.join(std::process::id().to_string()),
        b"invalid marker",
    )
    .unwrap();
    let r = diagnostic(&f.0, &f.0);
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    assert!(String::from_utf8_lossy(&r.stderr).contains("delegated_mcp_only"));
    assert!(!f.0.join("state").exists());
}
#[test]
fn spec188_cli_unbound_owner_is_read_only_and_gets_structured_refusal() {
    let f = Fixture::new();
    let r = diagnostic(&f.0, &f.0);
    assert!(!r.status.success());
    assert!(r.stdout.is_empty());
    let v: serde_json::Value = serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(v["code"], "diagnostic_worker_binding_unavailable");
    assert_eq!(v["stage"], "worker_binding");
    assert!(v.get("cause").is_some());
    assert_eq!(v["environment"], "diagnostic_process");
    assert_eq!(v["authoritative"], false);
    assert_eq!(v["command"]["task_id"], "task188");
    assert!(!v["recovery"].as_str().unwrap().is_empty());
    assert!(!f.0.join("state").exists());
}

struct LiveWorker {
    child: std::process::Child,
    fixture: Option<Fixture>,
}
impl LiveWorker {
    fn root(&self) -> &Path {
        &self.fixture.as_ref().unwrap().0
    }
    fn new() -> Self {
        use sha2::Digest;
        let f = Fixture::new();
        fs::create_dir(f.0.join("state")).unwrap();
        fs::set_permissions(f.0.join("state"), fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(f.0.join("missing-credential"), b"worker188-fixture-secret").unwrap();
        fs::set_permissions(
            f.0.join("missing-credential"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        let registry = serde_json::json!({"agents":{"custom188":{"command":"/bin/echo","args":[],"protocol":"acp","private_fixture_key188":"fixture-secret-label"}},
            "machines":{"cartae":{"enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":format!("{:x}",sha2::Sha256::digest(b"worker188-fixture-secret")),"allowed_agent_types":[],"mappings":{}}}});
        fs::write(f.0.join("registry"), serde_json::to_vec(&registry).unwrap()).unwrap();
        fs::set_permissions(f.0.join("registry"), fs::Permissions::from_mode(0o600)).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args([
                "worker",
                "--machine-id",
                "cartae",
                "--worker-id",
                "worker-cartae",
                "--provisioning-revision",
                "1",
            ])
            .arg("--credential-file")
            .arg(f.0.join("missing-credential"))
            .arg("--state-root")
            .arg(f.0.join("state"))
            .arg("--registry-file")
            .arg(f.0.join("registry"))
            .arg("--master-socket")
            .arg(f.0.join("socket"))
            .env_clear()
            .env("HOME", &f.0)
            .env("PATH", "/usr/bin:/bin")
            .env("BRIDGET_HOME", &f.0)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let mut live = Self {
            child,
            fixture: Some(f),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            assert!(
                live.child.try_wait().unwrap().is_none(),
                "worker fixture ne démarre pas"
            );
            if live.root().join("state/journals").is_dir()
                && fs::read_to_string("/proc/locks")
                    .unwrap()
                    .contains(&format!(" {} ", live.child.id()))
            {
                return live;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!("verrou worker fixture absent");
    }
}
impl Drop for LiveWorker {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        let pid = self.child.id().to_string();
        let identified = Command::new("ps")
            .args(["-p", &pid, "-o", "comm="])
            .output()
            .ok()
            .is_some_and(|o| {
                o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "bridget"
            });
        if identified {
            unsafe {
                libc::kill(self.child.id() as i32, libc::SIGTERM);
            };
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
        if self.child.try_wait().ok().flatten().is_none() {
            if let Some(f) = self.fixture.take() {
                eprintln!("fixture conservée : {}", f.0.display());
                std::mem::forget(f);
            }
        }
    }
}
#[test]
fn spec188_cli_cannot_hide_worker_lineage_with_an_empty_namespace() {
    let live = LiveWorker::new();
    let root = live.root();
    let markers = root.join("delegated-pids");
    fs::create_dir(&markers).unwrap();
    fs::set_permissions(&markers, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        markers.join(std::process::id().to_string()),
        b"invalid delegated marker",
    )
    .unwrap();
    let empty = root.join("empty");
    fs::create_dir(&empty).unwrap();
    fs::set_permissions(&empty, fs::Permissions::from_mode(0o700)).unwrap();
    let result = diagnostic(root, &empty);
    let v: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(
        v["code"],
        "delegated_mcp_only",
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(v["stage"], "caller");
    assert!(result.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("private_fixture_key188"));
}
#[test]
fn spec188_cli_quiet_registry_emits_only_closed_failure_json() {
    let live = LiveWorker::new();
    let root = live.root();
    let result = diagnostic(root, root);
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let v: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(v["code"], "diagnostic_receipt_unavailable");
    assert_eq!(v["stage"], "receipt");
    assert_eq!(v["environment"], "diagnostic_process");
    assert!(v.get("cause").is_some());
    assert!(!String::from_utf8_lossy(&result.stderr).contains("fixture-secret"));
}

#[test]
fn spec188_cli_different_tool_path_refuses_before_business_recheck() {
    let live = LiveWorker::new();
    let result = diagnostic_with_path(live.root(), live.root(), "/bin:/usr/bin");
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    let v: serde_json::Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(v["code"], "diagnostic_tool_environment_mismatch");
    assert_eq!(v["stage"], "tool_environment");
    assert!(v["recovery"].as_str().unwrap().contains("PATH"));
    assert_eq!(v["command"]["task_id"], "task188");
}
