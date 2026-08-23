use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[test]
fn profile_propose_puis_approve_expose_le_consentement_local_et_l_outbox() {
    let fixture = Fixture::new();
    let objective_id = Uuid::new_v4().to_string();

    let proposed = fixture.run(&[
        "profile",
        "propose",
        &objective_id,
        "review-hostile",
        "--definition",
        fixture.definition.to_str().unwrap(),
        "--context-scope",
        "objective:review",
        "--cwd",
        "/tmp/maicie-profile-cli",
        "--persistent",
        "--reason",
        "profil absent compatible",
        "--json",
    ]);
    assert!(
        proposed.status.success(),
        "{}",
        String::from_utf8_lossy(&proposed.stderr)
    );
    let proposed: Value = serde_json::from_slice(&proposed.stdout).unwrap();
    assert_eq!(proposed["kind"], "proposed");
    assert_eq!(proposed["screen"]["command"], "claude-code-acp");
    assert_eq!(proposed["screen"]["args"], json!(["--model", "claude-fable-5"]));
    let approval_id = proposed["approval_id"].as_str().unwrap().to_string();
    assert!(MaicieStore::open(&fixture.database)
        .unwrap()
        .pending_activation_outboxes()
        .unwrap()
        .is_empty());

    let refused = fixture.run(&[
        "profile",
        "approve",
        &approval_id,
        "--definition",
        fixture.definition.to_str().unwrap(),
        "--json",
    ]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--confirm est obligatoire"));

    let approved = fixture.run(&[
        "profile",
        "approve",
        &approval_id,
        "--definition",
        fixture.definition.to_str().unwrap(),
        "--confirm",
        "--json",
    ]);
    assert!(
        approved.status.success(),
        "{}",
        String::from_utf8_lossy(&approved.stderr)
    );
    let approved: Value = serde_json::from_slice(&approved.stdout).unwrap();
    assert_eq!(approved["kind"], "approved");
    assert_eq!(approved["actor"], "local_human");
    assert_eq!(approved["screen"]["args"], json!(["--model", "claude-fable-5"]));
    let pending = MaicieStore::open(&fixture.database)
        .unwrap()
        .pending_activation_outboxes()
        .unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].approval.id.to_string(), approval_id);
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    config: PathBuf,
    definition: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        // Le socket de Bridget reste sous la borne Unix de 103 octets.
        let root = PathBuf::from("/tmp").join(format!("mcp-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        let definition = root.join("resolved-definition.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": root.join("bridget.sock"),
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": [{
                    "id": "review-hostile",
                    "agent_name": "cxbridget",
                    "display_name": "Revue hostile",
                    "agent_type": "claude",
                    "model": "claude-fable-5",
                    "effort": "raisonnement-renforce",
                    "tags": ["review", "security"],
                    "personality_ref": "profiles/cxbridget.md",
                    "tools": ["bridget_send"],
                    "spawn_order_ref": "agents/claude-review"
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &definition,
            serde_json::to_vec(&json!({
                "command": "claude-code-acp",
                "args": ["--model", "claude-fable-5"],
                "protocol": "acp",
                "forbidden_env": ["ANTHROPIC_API_KEY"],
                "pass_env": ["HOME"],
                "permissions": "allow",
                "queue_capacity": 32,
                "notify_timeout_secs": 60,
                "mcp": {"interactive": "none", "acp_session": true},
                "digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            database,
            config,
            definition,
        }
    }

    fn run(&self, tail: &[&str]) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
        command.args(tail);
        command.args(["--config", self.config.to_str().unwrap()]);
        command.output().unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
