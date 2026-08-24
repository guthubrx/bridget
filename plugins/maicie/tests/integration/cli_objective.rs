use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, DecisionCoordination, EtatDecision, EtatObjectif, TypeDecision};
use maicie::store::{MaicieStore, StoreError};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

#[test]
fn commandes_objectif_rendent_et_persistent_les_decisions_explicites() {
    let fixture = Fixture::new();
    let objective_id = fixture.seed();

    let status = run(&fixture, &["status", &objective_id, "--json"]);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status_json["kind"], "status");
    assert_eq!(
        status_json["coordination"][0]["objective"]["id"],
        objective_id
    );
    assert_eq!(status_json["transport_snapshot"]["state"], "unknown");
    assert_eq!(status_json["stream_state"], "unavailable");

    let add = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "add-participant",
            "sentry",
            "--json",
        ],
    );
    assert!(add.status.success());
    let add_json: Value = serde_json::from_slice(&add.stdout).unwrap();
    assert_eq!(add_json["kind"], "decision");
    assert_eq!(add_json["decision"]["kind"], "ajouter_participant");

    let remove = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "remove-participant",
            "sentry",
            "--reason",
            "périmètre terminé",
            "--json",
        ],
    );
    assert!(remove.status.success());

    let summary = run(
        &fixture,
        &["objective", &objective_id, "summarize", "--json"],
    );
    assert!(summary.status.success());
    let summary_json: Value = serde_json::from_slice(&summary.stdout).unwrap();
    assert_eq!(summary_json["kind"], "summary");
    assert_eq!(
        summary_json["coordination"]["decisions"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let close = run(
        &fixture,
        &[
            "objective",
            &objective_id,
            "close",
            "--reason",
            "validation humaine",
            "--json",
        ],
    );
    assert!(close.status.success());
    let close_json: Value = serde_json::from_slice(&close.stdout).unwrap();
    assert_eq!(close_json["decision"]["kind"], "cloturer");

    let final_status = run(&fixture, &["status", &objective_id, "--json"]);
    assert!(final_status.status.success());
    let final_json: Value = serde_json::from_slice(&final_status.stdout).unwrap();
    assert_eq!(final_json["coordination"][0]["objective"]["etat"], "clos");
    assert_eq!(
        final_json["coordination"][0]["decisions"]
            .as_array()
            .unwrap()
            .len(),
        3
    );

    for command in [
        vec![
            "objective",
            &objective_id,
            "add-participant",
            "sentry",
            "--json",
        ],
        vec![
            "objective",
            &objective_id,
            "remove-participant",
            "sentry",
            "--reason",
            "trop tard",
            "--json",
        ],
        vec![
            "objective",
            &objective_id,
            "close",
            "--reason",
            "doublon",
            "--json",
        ],
    ] {
        let rejected = run(&fixture, &command);
        assert!(!rejected.status.success());
        assert!(String::from_utf8_lossy(&rejected.stderr).contains("objectif déjà clos"));
    }
    // La synthèse est une lecture factuelle : elle reste disponible après la
    // clôture, sans créer de décision supplémentaire.
    assert!(
        run(
            &fixture,
            &["objective", &objective_id, "summarize", "--json"]
        )
        .status
        .success()
    );
}

#[test]
fn decision_obsolete_nefface_pas_l_issue_terminale_reinjectee() {
    let fixture = Fixture::new();
    let objective_id = Uuid::parse_str(&fixture.seed()).unwrap();
    let snapshot = MaicieStore::open(&fixture.database)
        .unwrap()
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .pop()
        .unwrap()
        .objective;

    // Réinjection de la course C3 : une issue terminale est durablement
    // constatée après la projection CLI, juste avant sa décision de clôture.
    // Sans `WHERE id AND state`, le close obsolète écraserait cet état.
    let mut stale_close = snapshot.clone();
    stale_close.clore(200).unwrap();
    let mut terminal_objective = snapshot;
    terminal_objective
        .transition(EtatObjectif::AEvaluer, 150)
        .unwrap();
    let connection = rusqlite::Connection::open(&fixture.database).unwrap();
    connection
        .execute(
            "UPDATE objectives SET state = 'a_evaluer', payload_json = ?1 WHERE id = ?2",
            rusqlite::params![
                serde_json::to_vec(&terminal_objective).unwrap(),
                objective_id.to_string(),
            ],
        )
        .unwrap();
    drop(connection);

    let stale_decision = DecisionCoordination {
        id: Uuid::new_v4(),
        objectif_id: objective_id,
        kind: TypeDecision::Cloturer,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Appliquee,
        motif: "clôture devenue obsolète".to_string(),
    };
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    assert!(matches!(
        store.apply_objective_decision(
            &stale_decision,
            Some(&stale_close),
            EtatObjectif::EnCoordination,
        ),
        Err(StoreError::Conflict("objectif modifié concurremment"))
    ));

    let after = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(after.objective.etat, EtatObjectif::AEvaluer);
    assert!(after.decisions.is_empty());
}

fn run(fixture: &Fixture, tail: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
    command.args(tail);
    command.args(["--config", fixture.config.to_str().unwrap()]);
    command.output().unwrap()
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-cli-objective-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": "/tmp/mc-objective.sock",
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": []
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            database,
            config,
        }
    }

    fn seed(&self) -> String {
        let mut store = MaicieStore::open(&self.database).unwrap();
        let candidates = vec![DelegationCandidate {
            name: "prospective".to_string(),
            tags: vec![],
            available: true,
            dnd: false,
        }];
        let request = DelegateRequest {
            goal: "objectif de test",
            explicit_target: Some("prospective"),
            required_tags: &[],
            duration: ClasseDuree::Normale,
            reply: true,
            constat_id: None,
            suite: maicie::domain::SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            idempotency_key: "cli-objective-seed",
            now: 100,
            retry_until: 150,
            dedup_retained_until: 200,
            max_frame_bytes: 256 * 1024,
        };
        let result = delegate(
            &mut store,
            DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            "maicie",
            &candidates,
            &request,
        )
        .unwrap();
        let DelegateResult::Created(created) = result else {
            panic!("délégation attendue");
        };
        created.objective_id.to_string()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
