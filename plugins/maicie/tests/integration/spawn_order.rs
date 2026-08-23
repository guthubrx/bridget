use maicie::app::{
    LocalProfileApproval, ProfileActivationProposalRequest, approve_profile_activation,
    propose_profile_activation,
};
use maicie::reconcile::{
    ActivationReconcileAction, ActivationReconcilePhase, ReconcileError,
    reconcile_activation_startup_at, reconcile_activation_startup_at_observed,
};
use maicie::store::MaicieStore;
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread;
use uuid::Uuid;

const DIGEST: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn replay_exact_accepted_consomme_l_approbation_apres_l_issue_durable() {
    let fixture = Fixture::new();
    let (mut store, bytes) = approved(&fixture);
    let listener = UnixListener::bind(&fixture.socket).unwrap();
    let expected = bytes.clone();
    let server = thread::spawn(move || accepted_server(listener, expected, DIGEST));

    let report = reconcile_activation_startup_at(&mut store, &fixture.socket, 20).unwrap();
    assert_eq!(
        report.actions,
        vec![ActivationReconcileAction::IssueTerminale {
            command_id: command_id(&bytes),
        }]
    );
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
    server.join().unwrap();
    assert_eq!(approval_state(&fixture.database), "consumed");
}

#[test]
fn digest_divergent_est_refuse_terminalement_sans_reapprobation() {
    let fixture = Fixture::new();
    let (mut store, bytes) = approved(&fixture);
    let listener = UnixListener::bind(&fixture.socket).unwrap();
    let expected = bytes.clone();
    let server = thread::spawn(move || {
        accepted_server(
            listener,
            expected,
            "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210",
        )
    });

    let report = reconcile_activation_startup_at(&mut store, &fixture.socket, 20).unwrap();
    assert_eq!(
        report.actions,
        vec![ActivationReconcileAction::DefinitionDivergente {
            command_id: command_id(&bytes),
        }]
    );
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
    server.join().unwrap();
    assert_eq!(approval_state(&fixture.database), "consumed");
}

#[test]
fn hors_horizon_refuse_sans_ouvrir_de_socket_ni_nouvelle_approbation() {
    let fixture = Fixture::new();
    let (mut store, bytes) = approved(&fixture);
    let report = reconcile_activation_startup_at(&mut store, &fixture.socket, 81).unwrap();
    assert_eq!(
        report.actions,
        vec![ActivationReconcileAction::IssueTerminale {
            command_id: command_id(&bytes),
        }]
    );
    assert!(store.pending_activation_outboxes().unwrap().is_empty());
    assert_eq!(approval_state(&fixture.database), "consumed");
}

#[test]
fn crash_apres_issue_avant_commit_conserve_l_activation_a_rejouer() {
    let fixture = Fixture::new();
    let (mut store, bytes) = approved(&fixture);
    let listener = UnixListener::bind(&fixture.socket).unwrap();
    let expected = bytes.clone();
    let server = thread::spawn(move || accepted_server(listener, expected, DIGEST));

    let error =
        reconcile_activation_startup_at_observed(&mut store, &fixture.socket, 20, |phase| {
            match phase {
                ActivationReconcilePhase::AfterIssueBeforeStoreCommit => {
                    Err(ReconcileError::InvalidSnapshot("crash test"))
                }
                ActivationReconcilePhase::BeforeSocket => Ok(()),
            }
        })
        .unwrap_err();
    assert!(matches!(
        error,
        ReconcileError::InvalidSnapshot("crash test")
    ));
    assert_eq!(store.pending_activation_outboxes().unwrap().len(), 1);
    server.join().unwrap();
}

fn approved(fixture: &Fixture) -> (MaicieStore, Vec<u8>) {
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let proposal = propose_profile_activation(
        &mut store,
        &ProfileActivationProposalRequest {
            objective_id: Uuid::new_v4(),
            profile_id: "claude-review",
            agent_type: "claude",
            profile_hash: &[7; 32],
            resolved_definition_digest: DIGEST,
            context_scope: "objective:test",
            cwd: "/tmp",
            persistent: true,
            now: 10,
            spawn_deadline_at: 60,
            approval_expires_at: 100,
            retry_until: 80,
            dedup_retained_until: 100,
            reason: "test",
        },
    )
    .unwrap();
    let bytes = proposal.spawn_order_bytes.clone();
    approve_profile_activation(
        &mut store,
        &proposal,
        &LocalProfileApproval {
            approval_id: proposal.approval.id,
            now: 11,
            profile_hash: &[7; 32],
            resolved_definition_digest: DIGEST,
        },
    )
    .unwrap();
    (store, bytes)
}

fn accepted_server(listener: UnixListener, expected: Vec<u8>, digest: &str) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"wrapper"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"wrapper"}));
    let mut bytes = Vec::new();
    reader.read_until(b'\n', &mut bytes).unwrap();
    assert_eq!(&bytes[..bytes.len() - 1], expected.as_slice());
    let command_id = command_id(&expected).to_string();
    write_json(
        &mut writer,
        json!({"type":"SpawnAccepted","command_id":command_id,"name":"claude-review","definition":{"digest":digest}}),
    );
}

fn command_id(bytes: &[u8]) -> Uuid {
    let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    Uuid::parse_str(value["command_id"].as_str().unwrap()).unwrap()
}

fn approval_state(database: &PathBuf) -> String {
    Connection::open(database)
        .unwrap()
        .query_row("SELECT state FROM activation_approvals", [], |row| {
            row.get(0)
        })
        .unwrap()
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().unwrap()),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim_end()).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).unwrap();
    writer.flush().unwrap();
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(format!(
            "/tmp/mso-{}",
            &Uuid::new_v4().simple().to_string()[..12]
        ));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            database: root.join("maicie.sqlite3"),
            socket: root.join("bridget.sock"),
            root,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
