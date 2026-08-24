use maicie::app::{
    DelegateRequest, DelegateResult, DelegationCandidate, GuichetError, GuichetProcessResult,
    delegate, process_guichet_claim, record_guichet_lifecycle_event,
};
use maicie::bridget_client::{GuichetClaim, GuichetLifecycleEvent};
use maicie::config::DurationClasses;
use maicie::domain::guichet::{RequeteGuichet, parse_claim};
use maicie::domain::{ClasseDuree, EtatDelegation, EtatObjectif};
use maicie::store::{GuichetCommitPhase, MaicieStore};
use rusqlite::{Connection, ErrorCode};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-guichet-graft-{label}-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    }
}

fn seed(database: &Path) -> maicie::app::DelegationCreated {
    let mut store = MaicieStore::open(database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec!["rust".to_string()],
        available: true,
        dnd: false,
    }];
    let request = DelegateRequest {
        goal: "produire le rapport",
        explicit_target: Some("prospective"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: true,
        constat_id: None,
        idempotency_key: "guichet-seed",
        now: 900,
        retry_until: 1_100,
        dedup_retained_until: 1_200,
        max_frame_bytes: 256 * 1024,
    };
    let DelegateResult::Created(created) =
        delegate(&mut store, durations(), "maicie", &candidates, &request).unwrap()
    else {
        panic!("délégation attendue")
    };
    created
}

fn delivery_claim(request_id: &str, created: &maicie::app::DelegationCreated) -> GuichetClaim {
    let bytes = format!(
        "{{\"type\":\"service_request\",\"v\":1,\"issuer_scope\":\"scope-0123456789abcdef0123456789abcdef\",\"request_id\":\"{request_id}\",\"issued_at\":1000,\"from\":\"prospective\",\"to\":\"maicie\",\"operation\":\"delivery_report\",\"payload\":{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\",\"delivery_hash\":\"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\",\"in_reply_to\":\"{}\"}}}}",
        created.objective_id, created.delegation_id, created.message_id
    )
    .into_bytes();
    GuichetClaim {
        issuer_scope: "scope-0123456789abcdef0123456789abcdef".to_string(),
        request_id: request_id.to_string(),
        canonical_request: bytes,
        claimed_at: 1_000,
        claim_generation: 1,
        claim_token: "claim-0123456789abcdef0123456789abcdef".to_string(),
        claim_lease_expires_at: 1_100,
        expires_at: 1_200,
    }
}

fn lifecycle(
    claim: &GuichetClaim,
    created: &maicie::app::DelegationCreated,
    response_message_id: &str,
    state: &str,
) -> GuichetLifecycleEvent {
    let correlated = state == "answered";
    GuichetLifecycleEvent {
        issuer_scope: claim.issuer_scope.clone(),
        event_id: format!("event-{state}"),
        request_id: claim.request_id.clone(),
        state: state.to_string(),
        observed_at: 1_005,
        in_reply_to: correlated.then(|| created.message_id.to_string()),
        response_message_id: correlated.then(|| response_message_id.to_string()),
    }
}

fn assert_single_graft(database: &Path, objective_id: Uuid) {
    let store = MaicieStore::open(database).unwrap();
    let snapshots = store.objective_snapshots(Some(objective_id)).unwrap();
    assert_eq!(snapshots.len(), 1);
    let snapshot = &snapshots[0];
    assert_eq!(snapshot.objective.etat, EtatObjectif::AEvaluer);
    assert_ne!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::AEvaluer);
    assert_eq!(snapshot.decisions.len(), 1);
    drop(store);
    let connection = Connection::open(database).unwrap();
    for (table, expected) in [
        ("guichet_receptions", 1_i64),
        ("guichet_correlations", 1),
        ("coordination_decisions", 1),
    ] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, expected, "cardinalité inattendue dans {table}");
    }
}

#[test]
fn rapport_puis_answered_rejoue_les_memes_octets_sans_seconde_decision() {
    let root = root("report-first");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = delivery_claim("request-report-first", &created);
    let mut store = MaicieStore::open(&database).unwrap();
    let first = process_guichet_claim(&mut store, &claim, "response-report-first", 1_010).unwrap();
    assert!(!first.replayed);
    assert!(String::from_utf8_lossy(&first.reply_bytes).contains("\"outcome\":\"accepted\""));
    record_guichet_lifecycle_event(
        &mut store,
        &lifecycle(&claim, &created, "response-report-first", "answered"),
    )
    .unwrap();
    let replay = process_guichet_claim(&mut store, &claim, "ignored-on-replay", 1_020).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);
    let mut reclaimed = claim.clone();
    reclaimed.claim_generation = 2;
    reclaimed.claim_token = "claim-abcdef0123456789abcdef0123456789".to_string();
    let regenerated =
        process_guichet_claim(&mut store, &reclaimed, "ignored-on-reclaim", 1_025).unwrap();
    assert!(regenerated.replayed);
    assert_ne!(regenerated.reply_bytes, first.reply_bytes);
    assert!(String::from_utf8_lossy(&regenerated.reply_bytes).contains("\"claim_generation\":2"));
    assert!(
        String::from_utf8_lossy(&regenerated.reply_bytes)
            .contains("claim-abcdef0123456789abcdef0123456789")
    );
    let mut divergent = claim.clone();
    divergent.canonical_request = String::from_utf8(divergent.canonical_request)
        .unwrap()
        .replace(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .into_bytes();
    assert_eq!(
        process_guichet_claim(&mut store, &divergent, "response-divergent", 1_030),
        Err(GuichetError::EnvelopeMismatch)
    );
    drop(store);
    assert_single_graft(&database, created.objective_id);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn answered_puis_rapport_et_course_concurrente_convergent_vers_un_recu() {
    let root = root("event-first");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = delivery_claim("request-event-first", &created);
    let response_id = "response-event-first";
    let report_database = database.clone();
    let report_claim = claim.clone();
    let (transaction_open_tx, transaction_open_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let report_thread = thread::spawn(move || {
        let mut store = MaicieStore::open(report_database).unwrap();
        let canonical = parse_claim(&report_claim).unwrap();
        let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
            panic!("rapport attendu")
        };
        store
            .graft_delivery_report_observed(
                &report_claim,
                &canonical,
                report,
                response_id,
                1_010,
                |phase| {
                    if phase == GuichetCommitPhase::AfterDecisionInsert {
                        transaction_open_tx.send(()).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                    }
                    Ok(())
                },
            )
            .unwrap()
    });
    transaction_open_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    let contender = Connection::open(&database).unwrap();
    contender.busy_timeout(Duration::ZERO).unwrap();
    let contention = contender.execute(
        "INSERT INTO guichet_lifecycle_events(
             issuer_scope, event_id, request_id, state, observed_at,
             in_reply_to, response_message_id, payload_json
         ) VALUES ('contention-scope', 'contention-event', 'contention-request',
                   'answered', 1005, 'contention-message', 'contention-response', X'00')",
        [],
    );
    assert!(matches!(
        contention,
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == ErrorCode::DatabaseBusy
    ));
    drop(contender);
    // Mutation discriminante : retirer la transaction IMMEDIATE de la greffe,
    // ou placer la barrière hors transaction, rendrait l'INSERT concurrent
    // possible et ferait échouer l'assertion SQLITE_BUSY ci-dessus.
    release_tx.send(()).unwrap();
    let result = report_thread.join().unwrap();
    assert!(
        String::from_utf8_lossy(&result.reception.reply_bytes).contains("\"outcome\":\"accepted\"")
    );

    let event = lifecycle(&claim, &created, response_id, "answered");
    let mut event_store = MaicieStore::open(&database).unwrap();
    record_guichet_lifecycle_event(&mut event_store, &event).unwrap();
    drop(event_store);
    assert_single_graft(&database, created.objective_id);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn timeout_gagne_mais_le_rapport_et_son_hash_restent_greffes() {
    let root = root("timeout");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = delivery_claim("request-timeout", &created);
    let mut store = MaicieStore::open(&database).unwrap();
    record_guichet_lifecycle_event(
        &mut store,
        &lifecycle(&claim, &created, "unused", "timed_out"),
    )
    .unwrap();
    let result = process_guichet_claim(&mut store, &claim, "response-timeout", 1_010).unwrap();
    let reply = String::from_utf8(result.reply_bytes).unwrap();
    assert!(reply.contains("\"outcome\":\"request_already_terminal\""));
    assert!(reply.contains("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
    drop(store);
    assert_single_graft(&database, created.objective_id);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn depot_sur_objectif_clos_est_refuse_sans_tuer_maicie_ni_reouvrir() {
    let root = root("closed-objective");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let mut store = MaicieStore::open(&database).unwrap();
    store
        .close_objective(created.objective_id, "clôture manuelle", 1_005)
        .unwrap();
    let claim = delivery_claim("request-after-close", &created);
    let first = process_guichet_claim(&mut store, &claim, "response-after-close", 1_010).unwrap();
    assert!(!first.replayed);
    let reply = String::from_utf8(first.reply_bytes.clone()).unwrap();
    assert!(reply.contains("\"outcome\":\"request_already_terminal\""));

    let replay = process_guichet_claim(&mut store, &claim, "ignored-on-replay", 1_020).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);

    let snapshots = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap();
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].objective.etat, EtatObjectif::Clos);
    assert_eq!(snapshots[0].delegations[0].etat, EtatDelegation::Creee);
    assert!(
        snapshots[0]
            .decisions
            .iter()
            .any(|decision| decision.motif.contains("état métier incompatible"))
    );
    drop(store);

    let connection = Connection::open(&database).unwrap();
    let receptions: i64 = connection
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(receptions, 1);
    let outcome: String = connection
        .query_row(
            "SELECT outcome FROM guichet_receptions WHERE request_id = 'request-after-close'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(outcome, "request_already_terminal");
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_v6_vers_v7_preserve_les_agregats_et_ajoute_les_recus() {
    let root = root("migration-v6");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let connection = Connection::open(&database).unwrap();
    connection
        .execute("DROP TABLE guichet_correlations", [])
        .unwrap();
    connection
        .execute("DROP TABLE guichet_lifecycle_events", [])
        .unwrap();
    connection
        .execute("DROP TABLE guichet_receptions", [])
        .unwrap();
    connection.pragma_update(None, "user_version", 6).unwrap();
    connection
        .execute("DELETE FROM schema_migrations WHERE version = 7", [])
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 7);
    assert_eq!(
        store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()
            .len(),
        1
    );
    let claim = delivery_claim("request-migration-v7", &created);
    let first = process_guichet_claim(&mut store, &claim, "response-migration-v7", 1_010).unwrap();
    drop(store);

    // Une seconde ouverture d'une base déjà v7 est la vraie preuve
    // d'idempotence : la migration ne doit ni recréer, ni vider les tables.
    let mut reopened = MaicieStore::open(&database).unwrap();
    assert_eq!(reopened.schema_version().unwrap(), 7);
    let replay = process_guichet_claim(&mut reopened, &claim, "ignored", 1_020).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.reply_bytes, first.reply_bytes);
    drop(reopened);

    let connection = Connection::open(&database).unwrap();
    let tables: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'\n\
             AND name IN ('guichet_receptions','guichet_lifecycle_events','guichet_correlations')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tables, 3);
    let receptions: i64 = connection
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .unwrap();
    let correlations: i64 = connection
        .query_row("SELECT COUNT(*) FROM guichet_correlations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!((receptions, correlations), (1, 1));
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn faute_apres_decision_annule_decision_et_transition_dans_la_meme_transaction() {
    let root = root("atomic-decision-transition");
    let database = root.join("maicie.sqlite3");
    let created = seed(&database);
    let claim = delivery_claim("request-atomic-decision", &created);
    let canonical = parse_claim(&claim).unwrap();
    let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
        panic!("rapport attendu")
    };
    let mut store = MaicieStore::open(&database).unwrap();
    let result = store.graft_delivery_report_observed(
        &claim,
        &canonical,
        report,
        "response-atomic-decision",
        1_010,
        |phase| {
            if phase == GuichetCommitPhase::AfterDecisionInsert {
                return Err(maicie::store::StoreError::Conflict(
                    "faute injectée après décision",
                ));
            }
            Ok(())
        },
    );
    assert!(result.is_err());
    drop(store);

    let store = MaicieStore::open(&database).unwrap();
    let snapshot = store
        .objective_snapshots(Some(created.objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(snapshot.objective.etat, EtatObjectif::EnCoordination);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::Creee);
    assert!(snapshot.decisions.is_empty());
    drop(store);

    let connection = Connection::open(&database).unwrap();
    for table in [
        "coordination_decisions",
        "guichet_receptions",
        "guichet_correlations",
    ] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        // Mutation discriminante : si la transition ou la décision sort de
        // la transaction IMMEDIATE, cette cardinalité ou l'état ci-dessus
        // devient non nul malgré la faute et le test échoue.
        assert_eq!(count, 0, "écriture partielle visible dans {table}");
    }
    drop(connection);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn crash_worker() {
    let Ok(database) = std::env::var("MAICIE_GUICHET_CRASH_DB") else {
        return;
    };
    let request_id = std::env::var("MAICIE_GUICHET_CRASH_REQUEST").unwrap();
    let objective_id =
        Uuid::parse_str(&std::env::var("MAICIE_GUICHET_OBJECTIVE").unwrap()).unwrap();
    let delegation_id =
        Uuid::parse_str(&std::env::var("MAICIE_GUICHET_DELEGATION").unwrap()).unwrap();
    let message_id = Uuid::parse_str(&std::env::var("MAICIE_GUICHET_MESSAGE").unwrap()).unwrap();
    let phase = std::env::var("MAICIE_GUICHET_CRASH_PHASE").unwrap();
    let marker = PathBuf::from(std::env::var("MAICIE_GUICHET_CRASH_MARKER").unwrap());
    let created = maicie::app::DelegationCreated {
        objective_id,
        delegation_id,
        message_id,
        participant: "prospective".to_string(),
        duration: ClasseDuree::Normale,
        timeout_secs: 60,
        deadline_contractuelle: 960,
        replayed: false,
    };
    let claim = delivery_claim(&request_id, &created);
    let canonical = parse_claim(&claim).unwrap();
    let RequeteGuichet::DeliveryReport(report) = &canonical.request else {
        panic!("rapport attendu")
    };
    let mut store = MaicieStore::open(database).unwrap();
    let _ = store.graft_delivery_report_observed(
        &claim,
        &canonical,
        report,
        "response-crash",
        1_010,
        |observed| {
            let selected = matches!(
                (phase.as_str(), observed),
                ("before", GuichetCommitPhase::BeforeCommit)
                    | ("after", GuichetCommitPhase::AfterCommit)
            );
            if selected {
                fs::write(&marker, b"ready").unwrap();
                let mut byte = [0_u8; 1];
                std::io::stdin().read_exact(&mut byte).unwrap();
            }
            Ok(())
        },
    );
}

#[test]
fn crash_reel_avant_et_apres_commit_discrimine_l_atomicite() {
    for phase in ["before", "after"] {
        let root = root(phase);
        let database = root.join("maicie.sqlite3");
        let marker = root.join("barrier");
        let created = seed(&database);
        let request_id = format!("request-crash-{phase}");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("guichet_greffe::crash_worker")
            .arg("--nocapture")
            .env("MAICIE_GUICHET_CRASH_DB", &database)
            .env("MAICIE_GUICHET_CRASH_REQUEST", &request_id)
            .env("MAICIE_GUICHET_OBJECTIVE", created.objective_id.to_string())
            .env(
                "MAICIE_GUICHET_DELEGATION",
                created.delegation_id.to_string(),
            )
            .env("MAICIE_GUICHET_MESSAGE", created.message_id.to_string())
            .env("MAICIE_GUICHET_CRASH_PHASE", phase)
            .env("MAICIE_GUICHET_CRASH_MARKER", &marker)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !marker.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists(), "barrière de crash {phase} non atteinte");
        let process = Command::new("ps")
            .args(["-p", &child.id().to_string(), "-o", "command="])
            .output()
            .unwrap();
        let command = String::from_utf8_lossy(&process.stdout);
        assert!(command.contains("guichet_greffe_integration"));
        assert!(!command.contains("Firefox"));
        unsafe {
            libc::kill(child.id() as i32, libc::SIGTERM);
        }
        child.wait().unwrap();

        let mut store = MaicieStore::open(&database).unwrap();
        let snapshot = store
            .objective_snapshots(Some(created.objective_id))
            .unwrap()
            .remove(0);
        if phase == "before" {
            assert_eq!(snapshot.objective.etat, EtatObjectif::EnCoordination);
            assert!(snapshot.decisions.is_empty());
        } else {
            assert_eq!(snapshot.objective.etat, EtatObjectif::AEvaluer);
            assert_eq!(snapshot.decisions.len(), 1);
            let replay = process_guichet_claim(
                &mut store,
                &delivery_claim(&request_id, &created),
                "ignored",
                1_020,
            )
            .unwrap();
            assert!(replay.replayed);
        }
        drop(store);
        let connection = Connection::open(&database).unwrap();
        let receipts: i64 = connection
            .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(receipts, i64::from(phase == "after"));
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}

#[allow(dead_code)]
fn _assert_result_shape(result: &GuichetProcessResult) {
    assert!(!result.request_id.is_empty());
}
