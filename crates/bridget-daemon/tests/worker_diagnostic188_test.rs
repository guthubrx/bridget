//! Vérifie l'entrée CLI réelle, sans service ni fournisseur.
use std::{fs,path::{Path,PathBuf},process::{Command,Output}};
use std::os::unix::fs::PermissionsExt;
struct Fixture(PathBuf);
impl Fixture {
    fn new()->Self {
        let p=std::env::temp_dir().join(format!("bridget188-cli-{}",uuid::Uuid::new_v4()));
        fs::create_dir(&p).unwrap();fs::set_permissions(&p,fs::Permissions::from_mode(0o700)).unwrap();Self(p)
    }
}
impl Drop for Fixture {fn drop(&mut self){let _=fs::remove_dir_all(&self.0);}}
fn diagnostic(root:&Path,namespace:&Path)->Output {
    Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["worker","diagnose","--machine-id","cartae","--worker-id","worker-cartae","--provisioning-revision","1"])
        .arg("--credential-file").arg(root.join("missing-credential"))
        .arg("--state-root").arg(root.join("state"))
        .arg("--registry-file").arg(root.join("registry"))
        .arg("--master-socket").arg(root.join("socket"))
        .args(["--command-id","command188","--task-id","task188","--instance-id","instance188","--generation","1"])
        .env("BRIDGET_HOME",namespace).env_remove("BRIDGET_SOCKET")
        .env_remove("BRIDGET_AGENT_ID_FILE").env_remove("BRIDGET_AGENT_INSTANCE_ID")
        .output().unwrap()
}
#[test]
fn spec188_cli_invalid_namespace_closes_before_worker_inputs() {
    let f=Fixture::new();let r=diagnostic(&f.0,Path::new("relative-invalid-namespace"));
    assert!(!r.status.success());assert!(r.stdout.is_empty());
    assert!(String::from_utf8_lossy(&r.stderr).contains("namespace_configuration_invalid"));
    assert!(!f.0.join("state").exists());
}
#[test]
fn spec188_cli_invalid_delegated_marker_also_closes() {
    let f=Fixture::new();let markers=f.0.join("delegated-pids");fs::create_dir(&markers).unwrap();
    fs::set_permissions(&markers,fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(markers.join(std::process::id().to_string()),b"invalid marker").unwrap();
    let r=diagnostic(&f.0,&f.0);
    assert!(!r.status.success());assert!(r.stdout.is_empty());
    assert!(String::from_utf8_lossy(&r.stderr).contains("delegated_mcp_only"));
    assert!(!f.0.join("state").exists());
}
#[test]
fn spec188_cli_unbound_owner_is_read_only_and_gets_structured_refusal() {
    let f=Fixture::new();let r=diagnostic(&f.0,&f.0);
    assert!(!r.status.success());assert!(r.stdout.is_empty());
    let v:serde_json::Value=serde_json::from_slice(&r.stderr).unwrap();
    assert_eq!(v["code"],"diagnostic_context_unavailable");
    assert_eq!(v["authoritative"],false);assert_eq!(v["command"]["task_id"],"task188");
    assert!(!v["recovery"].as_str().unwrap().is_empty());
    assert!(!f.0.join("state").exists());
}
