use bridget_daemon::managed_process::process_birth;
use bridget_daemon::mcp_identity::write_marker;
use bridget_transport::greffe_authorization::{
    GreffeAuthorizationGate, GreffeAuthorizationRefusal, GreffeDepositAuthorization,
    GreffeMutationAction,
};
use bridget_transport::greffe_policy_refresh::{MarkerInventory, PolicyRegenerationReport};
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "bridget-greffe-policy-refresh-real-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run(arguments: &[&Path]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget-greffe-policy-refresh"));
    for argument in arguments {
        command.arg(argument);
    }
    command.output().unwrap()
}

#[test]
fn vrai_binaire_regenere_puis_la_garde_accepte_une_mutation_durable() {
    let fixture = Fixture::new();
    let markers = fixture.path("agent-pids");
    let names = fixture.path("names");
    fs::create_dir(&markers).unwrap();
    fs::create_dir(&names).unwrap();
    let name_file = names.join("current");
    fs::write(&name_file, "agent-redemarre").unwrap();
    let pid = std::process::id();
    let birth = process_birth(pid).unwrap();
    write_marker(&markers, pid, birth, "instance-nouvelle", &name_file).unwrap();

    let inventory_path = fixture.path("inventory.json");
    let scan = run(&[
        Path::new("scan"),
        Path::new("--markers"),
        &markers,
        Path::new("--output"),
        &inventory_path,
    ]);
    assert_eq!(
        scan.status.code(),
        Some(0),
        "scan réel échoué : {}",
        String::from_utf8_lossy(&scan.stderr)
    );
    assert_eq!(
        fs::metadata(&inventory_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let inventory: MarkerInventory =
        serde_json::from_slice(&fs::read(&inventory_path).unwrap()).unwrap();
    assert_eq!(inventory.live.len(), 1);
    assert_eq!(inventory.live[0].principal, "agent-redemarre");
    assert_eq!(inventory.live[0].instance_id, "instance-nouvelle");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let policy_path = fixture.path("policy.json");
    let policy = json!({
        "version": 1,
        "generation": 11,
        "attestation_key": KEY,
        "principals": [{
            "principal": "agent-redemarre",
            "marker_source": inventory.source,
            "actions": ["delegate"],
            "instances": [{
                "instance_id": "instance-ancienne",
                "expires_at": now + 600,
                "revoked": false
            }]
        }]
    });
    fs::write(&policy_path, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
    fs::set_permissions(&policy_path, fs::Permissions::from_mode(0o600)).unwrap();
    let original = fs::read(&policy_path).unwrap();

    let preview = run(&[
        Path::new("refresh"),
        Path::new("--policy"),
        &policy_path,
        Path::new("--inventory"),
        &inventory_path,
    ]);
    assert_eq!(
        preview.status.code(),
        Some(0),
        "prévisualisation échouée : {}",
        String::from_utf8_lossy(&preview.stderr)
    );
    let preview_report: PolicyRegenerationReport = serde_json::from_slice(&preview.stdout).unwrap();
    assert!(!preview_report.applied);
    assert_eq!(preview_report.previous_generation, 11);
    assert_eq!(preview_report.next_generation, 12);
    assert_eq!(fs::read(&policy_path).unwrap(), original);

    let apply = run(&[
        Path::new("refresh"),
        Path::new("--policy"),
        &policy_path,
        Path::new("--inventory"),
        &inventory_path,
        Path::new("--apply"),
    ]);
    assert_eq!(
        apply.status.code(),
        Some(0),
        "application échouée : {}",
        String::from_utf8_lossy(&apply.stderr)
    );
    let applied_report: PolicyRegenerationReport = serde_json::from_slice(&apply.stdout).unwrap();
    assert!(applied_report.applied);
    assert_eq!(applied_report.refreshed_principals, ["agent-redemarre"]);
    let rewritten = fs::read_to_string(&policy_path).unwrap();
    assert!(rewritten.contains("instance-nouvelle"));
    assert!(!rewritten.contains("instance-ancienne"));

    let audit_path = fixture.path("audit.jsonl");
    let gate = GreffeAuthorizationGate::new(&policy_path, &audit_path);
    let durable = fixture.path("durable-effect");
    fs::write(&durable, b"unchanged").unwrap();
    let old = gate.authorize_deposit_then(
        GreffeDepositAuthorization {
            canonical_name: Some("agent-redemarre"),
            canonical_instance_id: Some("instance-ancienne"),
            declared_from: Some("agent-redemarre"),
            action: GreffeMutationAction::Delegate,
            issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
            request_id: "request-old-instance",
            request_issued_at: now - 1,
            canonical_request: br#"{"operation":"delegate","value":"old"}"#,
            observed_at: now,
        },
        |_| fs::write(&durable, b"mutated-by-old").unwrap(),
    );
    assert_eq!(
        old.unwrap_err(),
        GreffeAuthorizationRefusal::InstanceUnknown
    );
    assert_eq!(fs::read(&durable).unwrap(), b"unchanged");

    gate.authorize_deposit_then(
        GreffeDepositAuthorization {
            canonical_name: Some("agent-redemarre"),
            canonical_instance_id: Some("instance-nouvelle"),
            declared_from: Some("agent-redemarre"),
            action: GreffeMutationAction::Delegate,
            issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
            request_id: "request-new-instance",
            request_issued_at: now - 1,
            canonical_request: br#"{"operation":"delegate","value":"new"}"#,
            observed_at: now,
        },
        |_| fs::write(&durable, b"mutated-by-new").unwrap(),
    )
    .unwrap();
    assert_eq!(fs::read(&durable).unwrap(), b"mutated-by-new");
}
