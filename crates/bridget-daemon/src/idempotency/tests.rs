use super::*;
use std::sync::{Arc, Barrier, Mutex, Once};
use std::thread;

const NOW: i64 = 1_000_000;
const HORIZON: i64 = 3600;

/// Un seul logger global (contrainte `log`) ; les tests qui lisent la trace
/// prennent `TEST_LOG_LOCK` pour ne pas se marcher dessus.
static TEST_LOG_LOCK: Mutex<()> = Mutex::new(());
static TEST_WARN_LOGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct TempDbGuard(std::path::PathBuf);

impl TempDbGuard {
    fn new(name_prefix: &str) -> Self {
        Self(std::env::temp_dir().join(format!("{name_prefix}-{}.db", uuid::Uuid::new_v4())))
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDbGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn init_test_warn_logger() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        struct CapturingLogger;
        impl log::Log for CapturingLogger {
            fn enabled(&self, metadata: &log::Metadata) -> bool {
                metadata.level() <= log::Level::Warn
            }
            fn log(&self, record: &log::Record) {
                if record.level() <= log::Level::Warn {
                    TEST_WARN_LOGS
                        .lock()
                        .expect("TEST_WARN_LOGS")
                        .push(record.args().to_string());
                }
            }
            fn flush(&self) {}
        }
        static LOGGER: CapturingLogger = CapturingLogger;
        log::set_logger(&LOGGER).expect("logger de test installable");
        log::set_max_level(log::LevelFilter::Warn);
    });
}

fn commencer_capture_trace_sans_enveloppe() {
    init_test_warn_logger();
    let _guard = TEST_LOG_LOCK.lock().expect("TEST_LOG_LOCK");
    TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clear();
}

fn traces_sans_enveloppe() -> Vec<String> {
    TEST_WARN_LOGS
        .lock()
        .expect("TEST_WARN_LOGS")
        .iter()
        .filter(|line| line.contains("corrélation in_reply_to impossible"))
        .cloned()
        .collect()
}

fn key() -> IdempotencyKey {
    IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-1").unwrap()
}

fn spawn_key() -> IdempotencyKey {
    IdempotencyKey::new(
        "supervisor_009_aaaaaaaaaaaa",
        OperationKind::Spawn,
        "command-1",
    )
    .unwrap()
}

fn reserve(store: &IdempotencyStore, bytes: &[u8]) -> Reservation {
    store.reserve(&key(), bytes, NOW, HORIZON, NOW, 30).unwrap()
}

#[test]
fn native151_envoi_resume_fige_migration_idempotence_preserve_ancienne_demande() {
    let path = TempDbGuard::new("bridget-151-idempotence");
    {
        let legacy = Connection::open(path.path()).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE tracked_requests (
                    id TEXT PRIMARY KEY, sender TEXT NOT NULL, target TEXT NOT NULL,
                    state TEXT NOT NULL, created_at INTEGER NOT NULL,
                    deadline_at INTEGER NOT NULL, escalation_level INTEGER NOT NULL DEFAULT 0,
                    cancel_reason TEXT, completed_at INTEGER
                 );
                 INSERT INTO tracked_requests VALUES
                    ('ancienne', 'parent', 'enfant', 'answered', 1, 2, 0, NULL, 3);",
            )
            .unwrap();
    }
    for _ in 0..2 {
        let store = IdempotencyStore::open(path.path()).unwrap();
        let snapshot: (String, Option<String>) = store
            .conn
            .query_row(
                "SELECT state, reply_summary FROM tracked_requests WHERE id='ancienne'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(snapshot, ("answered".into(), None));
    }
}

#[test]
fn native151_envoi_resume_fige_deux_scopes_resistent_au_remplacement_du_ledger() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for (scope, request, responder, body, delivery) in [
        (
            "012_scope_aaaaaaaaaaaa",
            "demande-a",
            "enfant-a",
            "première réponse",
            "remise-a",
        ),
        (
            "012_scope_bbbbbbbbbbbb",
            "demande-b",
            "enfant-b",
            "seconde réponse",
            "remise-b",
        ),
    ] {
        store
            .conn
            .execute(
                "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at)
                 VALUES (?1, 'parent', ?2, 'open', ?3, ?4)",
                params![request, responder, NOW, NOW + HORIZON],
            )
            .unwrap();
        let key = IdempotencyKey::new(scope, OperationKind::Send, "reponse-partagee").unwrap();
        assert!(matches!(
            store
                .reserve(&key, body.as_bytes(), NOW, HORIZON, NOW, 30)
                .unwrap(),
            Reservation::Prepared { .. }
        ));
        let mut message = bridget_core::BridgetMessage::new(responder, "parent", body);
        message.id = "reponse-partagee".into();
        message.in_reply_to = Some(request.into());
        store
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: delivery.into(),
                    recipient_instance_id: "instance-parent".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: serde_json::to_vec(&message).unwrap(),
                },
            )
            .unwrap();
        assert_eq!(
            store
                .acknowledge_send_delivery(delivery, "instance-parent", 1)
                .unwrap(),
            Some(request.into())
        );
        assert_eq!(
            store
                .acknowledge_send_delivery(delivery, "instance-parent", 1)
                .unwrap(),
            None
        );
    }
    let ledger_body: String = store
        .conn
        .query_row(
            "SELECT body FROM ledger WHERE id='reponse-partagee' AND target='parent'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(ledger_body, "seconde réponse");
    for (request, responder, body) in [
        ("demande-a", "enfant-a", "première réponse"),
        ("demande-b", "enfant-b", "seconde réponse"),
    ] {
        let encoded: String = store
            .conn
            .query_row(
                "SELECT reply_summary FROM tracked_requests WHERE id=?1",
                [request],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&encoded).unwrap(),
            serde_json::json!({"id":"reponse-partagee", "from":responder, "summary":body, "summary_truncated":false})
        );
    }
}

#[test]
fn native151_envoi_resume_fige_frontieres_utf8_et_taille_du_corps() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for (index, (body, expected, truncated)) in [
        (String::new(), String::new(), false),
        ("a".repeat(1023), "a".repeat(1023), false),
        ("a".repeat(1024), "a".repeat(1024), false),
        ("a".repeat(1025), "a".repeat(1024), true),
        (format!("{}€", "a".repeat(1023)), "a".repeat(1023), true),
        (
            format!("{}€", "a".repeat(1021)),
            format!("{}€", "a".repeat(1021)),
            false,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let request = format!("demande-{index}");
        store
            .conn
            .execute(
                "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at)
             VALUES (?1, 'parent', 'enfant', 'open', ?2, ?3)",
                params![request, NOW, NOW + HORIZON],
            )
            .unwrap();
        let id = format!("reponse-{index}");
        let key = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, &id).unwrap();
        assert!(matches!(
            store
                .reserve(&key, body.as_bytes(), NOW, HORIZON, NOW, 30)
                .unwrap(),
            Reservation::Prepared { .. }
        ));
        let mut message = bridget_core::BridgetMessage::new("enfant", "parent", body);
        message.id = id.clone();
        message.in_reply_to = Some(request.clone());
        store
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: id.clone(),
                    recipient_instance_id: "instance-parent".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: serde_json::to_vec(&message).unwrap(),
                },
            )
            .unwrap();
        store
            .acknowledge_send_delivery(&id, "instance-parent", 1)
            .unwrap();
        let encoded: String = store
            .conn
            .query_row(
                "SELECT reply_summary FROM tracked_requests WHERE id=?1",
                [&request],
                |row| row.get(0),
            )
            .unwrap();
        let reply: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(reply["summary"], expected);
        assert_eq!(reply["summary_truncated"], truncated);
        assert!(reply["summary"].as_str().unwrap().len() <= 1024);
    }
}

#[test]
fn native151_envoi_resume_fige_et_accuse_sont_atomiques() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    store
        .conn
        .execute_batch(
            "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at)
             VALUES ('demande-atomique', 'parent', 'enfant', 'open', 1, 2);
             CREATE TRIGGER refuser_resume BEFORE UPDATE OF reply_summary ON tracked_requests
             BEGIN SELECT RAISE(ABORT, 'faute injectée sur le résumé'); END;",
        )
        .unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"reponse"),
        Reservation::Prepared { .. }
    ));
    let mut message = bridget_core::BridgetMessage::new("enfant", "parent", "réponse atomique");
    message.id = key.idempotency_key.clone();
    message.in_reply_to = Some("demande-atomique".into());
    store
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "remise-atomique".into(),
                recipient_instance_id: "instance-parent".into(),
                delivery_generation: 1,
                expires_at: NOW + HORIZON,
                message_bytes: serde_json::to_vec(&message).unwrap(),
            },
        )
        .unwrap();
    assert!(
        store
            .acknowledge_send_delivery("remise-atomique", "instance-parent", 1)
            .is_err()
    );
    assert_eq!(delivery_phase(&store, "remise-atomique"), "dispatching");
    assert!(matches!(
        store.lookup(&key, NOW).unwrap(),
        LookupResult::OutcomeUnknown { .. }
    ));
    let state: String = store
        .conn
        .query_row(
            "SELECT state FROM tracked_requests WHERE id='demande-atomique'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(state, "open");
    store
        .conn
        .execute_batch("DROP TRIGGER refuser_resume")
        .unwrap();
    assert_eq!(
        store
            .acknowledge_send_delivery("remise-atomique", "instance-parent", 1)
            .unwrap(),
        Some("demande-atomique".into())
    );
}

#[test]
fn migration_v3_ajoute_la_definition_resolue_aux_sagas_existantes() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-definition-migration-{}.db",
        uuid::Uuid::new_v4()
    ));
    {
        let legacy = Connection::open(&path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );",
            )
            .unwrap();
    }
    let store = IdempotencyStore::open(&path).unwrap();
    let columns = store
        .conn
        .prepare("PRAGMA table_info(spawn_commands)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(
        columns
            .iter()
            .any(|column| column == "resolved_definition_json")
    );
    drop(store);
    std::fs::remove_file(path).unwrap();
}

/// ORACLE — migration v4 : une base pré-orphaned DOIT élargir le CHECK.
/// Meurt si l'on retire le bloc v4 alors que le CREATE fresh porte déjà
/// `orphaned` (les oracles de phase restent verts, la montée reste muette).
///
/// Nature de la charge : trois migrations sur quatre avaient déjà leur
/// témoin dans ce fichier — la v4 était la seule orpheline. Ce n'est pas
/// une pratique à instaurer, c'est une pratique à ne pas rompre. Montage :
/// DDL **v3** à la main (CHECK à trois phases), pas une base neuve.
#[test]
fn migration_v4_elargit_le_check_pour_accepter_orphaned() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-v4-orphan-{}.db",
        uuid::Uuid::new_v4()
    ));
    {
        let legacy = Connection::open(&path).unwrap();
        legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        idempotency_key TEXT NOT NULL,
                        canonical_bytes BLOB NOT NULL,
                        state TEXT NOT NULL,
                        public_result_kind TEXT,
                        public_result_category TEXT,
                        public_result_reason TEXT,
                        issued_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
                     );
                     CREATE TABLE send_deliveries (
                        delivery_id TEXT PRIMARY KEY,
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                        idempotency_key TEXT NOT NULL,
                        recipient_instance_id TEXT NOT NULL,
                        delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                        phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate')),
                        expires_at INTEGER NOT NULL,
                        message_bytes BLOB,
                        FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                            REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                            ON DELETE CASCADE
                     );
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'legacy-key', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );
                     INSERT INTO send_deliveries VALUES (
                        'delivery-legacy', '012_scope_aaaaaaaaaaaa', 'send', 'legacy-key',
                        'instance-1', 1, 'dispatching', 1003600, X'7b7d'
                     );",
                )
                .unwrap();
        // `X'7b7d'` (= `{}`) suffit ici : cet oracle ne teste que le CHECK
        // via UPDATE brut. Un chemin réel (`orphan_dispatching_for_instance`)
        // exige un vrai `BridgetMessage` sérialisé — voir
        // `chemin_reel_sur_base_migree_depuis_v3`.
        // Sur le CHECK pré-v4, orphaned est refusé.
        let refused = legacy.execute(
            "UPDATE send_deliveries SET phase = 'orphaned' WHERE delivery_id = 'delivery-legacy'",
            [],
        );
        assert!(
            refused.is_err(),
            "précondition : le CHECK pré-v4 doit refuser orphaned"
        );
    }

    let store = IdempotencyStore::open(&path).unwrap();
    let has_v4: bool = store
        .conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 4)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(has_v4, "la migration v4 doit être consignées");
    store
        .conn
        .execute(
            "UPDATE send_deliveries SET phase = 'orphaned' WHERE delivery_id = 'delivery-legacy'",
            [],
        )
        .expect("après v4, orphaned doit passer le CHECK");
    let phase: String = store
        .conn
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-legacy'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(phase, "orphaned");
    drop(store);
    std::fs::remove_file(path).unwrap();
}

/// ORACLE — chemin de production sur base réellement migrée v3→v4.
/// Complète `migration_v4_elargit…` (CHECK par UPDATE brut) : ici
/// `orphan_dispatching_for_instance` désérialise `message_bytes`, écrit
/// `orphan_emitter_notices`, et le lookup rend `Orphaned`.
#[test]
fn chemin_reel_sur_base_migree_depuis_v3() {
    let path =
        std::env::temp_dir().join(format!("bridget-chemin-reel-{}.db", uuid::Uuid::new_v4()));
    let message = bridget_core::BridgetMessage::new("emetteur-x", "cible-y", "mandat perdu");
    let message_id = message.id.clone();
    let bytes = serde_json::to_vec(&message).unwrap();
    let key =
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "cle-reelle").unwrap();

    {
        let legacy = Connection::open(&path).unwrap();
        legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        idempotency_key TEXT NOT NULL,
                        canonical_bytes BLOB NOT NULL,
                        state TEXT NOT NULL,
                        public_result_kind TEXT,
                        public_result_category TEXT,
                        public_result_reason TEXT,
                        issued_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
                     );
                     CREATE TABLE send_deliveries (
                        delivery_id TEXT PRIMARY KEY,
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                        idempotency_key TEXT NOT NULL,
                        recipient_instance_id TEXT NOT NULL,
                        delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                        phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate')),
                        expires_at INTEGER NOT NULL,
                        message_bytes BLOB,
                        FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                            REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                            ON DELETE CASCADE
                     );
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'cle-reelle', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );",
                )
                .unwrap();
        legacy
            .execute(
                "INSERT INTO send_deliveries VALUES (
                        'delivery-reel', '012_scope_aaaaaaaaaaaa', 'send', 'cle-reelle',
                        'instance-1', 1, 'dispatching', 1003600, ?1)",
                params![bytes],
            )
            .unwrap();
    }

    let mut store = IdempotencyStore::open(&path).expect("migration v4");
    let notices = store
        .orphan_dispatching_for_instance("instance-1", "destinataire purge")
        .expect("chemin réel sur base migrée");

    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].delivery_id, "delivery-reel");
    assert_eq!(notices[0].message_id, message_id);
    assert_eq!(notices[0].sender, "emetteur-x");
    assert_eq!(notices[0].target, "cible-y");

    let phase: String = store
        .conn
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-reel'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(phase, "orphaned");

    let notices_en_base: i64 = store
        .conn
        .query_row("SELECT COUNT(*) FROM orphan_emitter_notices", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(notices_en_base, 1);

    assert_eq!(
        store.lookup(&key, 1_000_000).unwrap(),
        LookupResult::Orphaned {
            expires_at: 1_003_600,
            reason: "destinataire purge".to_string(),
        }
    );

    drop(store);
    std::fs::remove_file(path).unwrap();
}

/// ORACLE — migration v4 face à des enfants sans parent (FK OFF hors daemon).
/// Meurt si l'INSERT SELECT échoue au démarrage : flotte bloquée.
/// Meurt aussi si l'écart est silencieux (mutant : DELETE sans `warn!`).
/// Nature : prouve le *mécanisme* (précaution). Occurrence nulle mesurée
/// sur la copie prod du jour — ne pas lire cet oracle comme un incident.
#[test]
fn migration_v4_nettoie_les_enfants_sans_parent_sans_bloquer_le_daemon() {
    init_test_warn_logger();
    let _log_guard = TEST_LOG_LOCK.lock().expect("TEST_LOG_LOCK");
    TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clear();

    let db = TempDbGuard::new("bridget-idempotency-v4-fk-orphan");
    {
        let legacy = Connection::open(db.path()).unwrap();
        // Comme le CLI SQLite : FK OFF par défaut → orphelin de schéma possible.
        legacy.pragma_update(None, "foreign_keys", false).unwrap();
        legacy
                .execute_batch(
                    "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                     INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                     CREATE TABLE idempotency_records (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        idempotency_key TEXT NOT NULL,
                        canonical_bytes BLOB NOT NULL,
                        state TEXT NOT NULL,
                        public_result_kind TEXT,
                        public_result_category TEXT,
                        public_result_reason TEXT,
                        issued_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
                     );
                     CREATE TABLE send_deliveries (
                        delivery_id TEXT PRIMARY KEY,
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL CHECK (operation_kind = 'send'),
                        idempotency_key TEXT NOT NULL,
                        recipient_instance_id TEXT NOT NULL,
                        delivery_generation INTEGER NOT NULL CHECK (delivery_generation > 0),
                        phase TEXT NOT NULL CHECK (phase IN ('dispatching', 'acked', 'indeterminate')),
                        expires_at INTEGER NOT NULL,
                        message_bytes BLOB,
                        FOREIGN KEY (issuer_scope, operation_kind, idempotency_key)
                            REFERENCES idempotency_records(issuer_scope, operation_kind, idempotency_key)
                            ON DELETE CASCADE
                     );
                     CREATE TABLE spawn_commands (
                        issuer_scope TEXT NOT NULL,
                        operation_kind TEXT NOT NULL,
                        command_id TEXT NOT NULL,
                        name TEXT NOT NULL,
                        generation INTEGER NOT NULL,
                        persistent INTEGER NOT NULL,
                        state TEXT NOT NULL,
                        instance_id TEXT,
                        deadline_at INTEGER NOT NULL,
                        expires_at INTEGER NOT NULL,
                        issue_kind TEXT,
                        issue_category TEXT,
                        issue_reason TEXT,
                        resolved_definition_json TEXT,
                        PRIMARY KEY (issuer_scope, operation_kind, command_id)
                     );
                     -- Parent valide + enfant valide.
                     INSERT INTO idempotency_records VALUES (
                        '012_scope_aaaaaaaaaaaa', 'send', 'kept-key', X'00',
                        'dispatching', NULL, NULL, NULL, 1000000, 1003600
                     );
                     INSERT INTO send_deliveries VALUES (
                        'delivery-kept', '012_scope_aaaaaaaaaaaa', 'send', 'kept-key',
                        'instance-1', 1, 'dispatching', 1003600, X'7b7d'
                     );
                     -- Enfant SANS parent (impossible sous le daemon, possible hors daemon).
                     INSERT INTO send_deliveries VALUES (
                        'delivery-sans-parent', '012_scope_bbbbbbbbbbbb', 'send', 'ghost-key',
                        'instance-ghost', 1, 'dispatching', 1003600, X'7b7d'
                     );",
                )
                .unwrap();
    }

    // Ne doit PAS paniquer / échouer : c'est le démarrage du daemon.
    let store = IdempotencyStore::open(db.path())
        .expect("v4 ne doit pas bloquer le démarrage sur un enfant sans parent");
    let has_v4: bool = store
        .conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 4)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(has_v4);
    let kept: usize = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = 'delivery-kept'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let ghost: usize = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = 'delivery-sans-parent'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(kept, 1, "l'enfant valide survit");
    assert_eq!(ghost, 0, "l'enfant sans parent est nettoyé, pas bloquant");

    let logs = TEST_WARN_LOGS.lock().expect("TEST_WARN_LOGS").clone();
    let dit = logs.iter().any(|line| {
        line.contains("migration v4")
            && line.contains("écarté")
            && line.contains("delivery-sans-parent")
    });
    assert!(
        dit,
        "l'écart doit être DIT (warn! compte + delivery_id) ; logs={logs:?}"
    );

    drop(store);
}

#[test]
fn creation_spawn_et_saga_sont_atomiques_sous_la_meme_fk() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = spawn_key();
    assert!(
        store
            .reserve_spawn(
                &key,
                b"canon-spawn",
                NOW,
                HORIZON,
                NOW,
                30,
                "codex-1",
                u64::MAX,
                true,
                NOW + 60,
                "instance-1",
            )
            .is_err()
    );
    assert_eq!(store.record_count().unwrap(), 0);
    assert!(matches!(
        store
            .reserve_spawn(
                &key,
                b"canon-spawn",
                NOW,
                HORIZON,
                NOW,
                30,
                "codex-1",
                1,
                true,
                NOW + 60,
                "instance-1",
            )
            .unwrap(),
        SpawnReservation::Requested(_)
    ));
    assert_eq!(store.spawn_commands(&key.issuer_scope).unwrap().len(), 1);
}

#[test]
fn echec_de_finalisation_socle_annule_la_finalisation_saga() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = spawn_key();
    store
        .reserve_spawn(
            &key,
            b"canon-spawn",
            NOW,
            HORIZON,
            NOW,
            30,
            "codex-1",
            1,
            true,
            NOW + 60,
            "instance-1",
        )
        .unwrap();
    store
        .advance_spawn(
            &key,
            1,
            SpawnCommandState::Requested,
            SpawnCommandState::Reserved,
        )
        .unwrap();
    store
        .conn
        .execute(
            "UPDATE idempotency_records SET state = 'terminal',
                    public_result_kind = 'accepted'
                 WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
                   AND idempotency_key = ?2",
            params![key.issuer_scope, key.idempotency_key],
        )
        .unwrap();
    assert!(
        store
            .finish_spawn(
                &key,
                1,
                &SpawnCommandIssue::Failed {
                    category: "startup_failed".to_string(),
                    reason: "fixture".to_string(),
                },
            )
            .is_err()
    );
    assert_eq!(
        store.spawn_commands(&key.issuer_scope).unwrap()[0].state,
        SpawnCommandState::Reserved
    );
}

#[test]
fn reserve_then_replay_uses_the_same_record() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Prepared {
            expires_at: NOW + HORIZON
        }
    );
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        })
    );
}

#[test]
fn a_single_byte_difference_is_an_envelope_mismatch() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    assert_eq!(reserve(&store, b"canOn"), Reservation::EnvelopeMismatch);
}

#[test]
fn terminal_result_is_replayed_without_mutation() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    store
        .transition(&key, RecordState::Prepared, RecordState::Dispatching)
        .unwrap();
    store
        .finalize(
            &key,
            PublicResult::Rejected {
                category: "dnd".to_string(),
                reason: "occupé".to_string(),
            },
        )
        .unwrap();
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::Rejected {
            category: "dnd".to_string(),
            reason: "occupé".to_string(),
            expires_at: NOW + HORIZON,
        })
    );
}

#[test]
fn first_send_outside_its_horizon_is_expired() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert_eq!(
        store
            .reserve(&key(), b"canon", NOW - HORIZON - 1, HORIZON, NOW, 30)
            .unwrap(),
        Reservation::IdempotencyExpired
    );
    assert_eq!(
        store.lookup(&key(), NOW).unwrap(),
        LookupResult::IdempotencyExpired
    );
}

#[test]
fn transitions_are_monotone() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    assert!(matches!(
        store.transition(&key, RecordState::Prepared, RecordState::Terminal),
        Err(IdempotencyError::InvalidTransition { .. })
    ));
    store
        .transition(&key, RecordState::Prepared, RecordState::Dispatching)
        .unwrap();
    store
        .finalize(
            &key,
            PublicResult::Accepted {
                expires_at: NOW + HORIZON,
            },
        )
        .unwrap();
    assert!(matches!(
        store.transition(&key, RecordState::Terminal, RecordState::Dispatching),
        Err(IdempotencyError::InvalidTransition { .. })
    ));
}

#[test]
fn purge_uses_each_record_expiry_not_a_new_configuration() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    assert_eq!(store.purge_expired(NOW + HORIZON - 1).unwrap(), 0);
    assert!(matches!(
        store.lookup(&key(), NOW + HORIZON - 1).unwrap(),
        LookupResult::OutcomeUnknown { .. }
    ));
    assert_eq!(store.purge_expired(NOW + HORIZON).unwrap(), 1);
    assert_eq!(
        store.lookup(&key(), NOW + HORIZON).unwrap(),
        LookupResult::IdempotencyExpired
    );
}

#[test]
fn retry_keeps_the_original_horizon_after_a_configuration_drop() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    assert_eq!(
        store
            .reserve(&key(), b"canon", NOW, 10, NOW + 11, 30)
            .unwrap(),
        Reservation::Replayed(LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        })
    );
}

#[test]
fn dispatch_and_delivery_are_persisted_in_one_transaction() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-1".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery));
    assert_eq!(
        store.lookup(&key, NOW).unwrap(),
        LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        }
    );
}

#[test]
fn lien_causal_de_livraison_preserve_le_rejeu_exact() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-causale".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    let expected = DeliveryExecutionLink {
        delivery_id: delivery.delivery_id.clone(),
        submission_id: "submission-1".to_string(),
        execution_id: "execution-1".to_string(),
    };
    store.link_send_delivery(&expected).unwrap();
    assert_eq!(
        store
            .delivery_execution_link(&delivery.delivery_id)
            .unwrap(),
        Some(expected)
    );
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        })
    );
    assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery));
    assert!(matches!(
        store.link_send_delivery(&DeliveryExecutionLink {
            delivery_id: "delivery-causale".to_string(),
            submission_id: "submission-2".to_string(),
            execution_id: "execution-1".to_string(),
        }),
        Err(IdempotencyError::InvalidDelivery)
    ));
}

#[test]
fn spec_079_remise_et_lien_execution_partagent_la_transaction() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let primary_key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-079".to_string(),
        recipient_instance_id: "instance-079".to_string(),
        delivery_generation: 79,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-079", "agent-079"),
    };
    let link = DeliveryExecutionLink {
        delivery_id: delivery.delivery_id.clone(),
        submission_id: "message-079".to_string(),
        execution_id: "execution-message-079".to_string(),
    };
    store
        .begin_send_delivery_with_execution(&primary_key, &delivery, None, &link)
        .unwrap();
    assert_eq!(store.send_delivery(&primary_key).unwrap(), Some(delivery));
    assert_eq!(
        store.delivery_execution_link("delivery-079").unwrap(),
        Some(link)
    );

    let mut refused = IdempotencyStore::open_in_memory().unwrap();
    let refused_key = key();
    assert!(matches!(
        reserve(&refused, b"canon"),
        Reservation::Prepared { .. }
    ));
    let refused_delivery = SendDelivery {
        delivery_id: "delivery-refusee".to_string(),
        recipient_instance_id: "instance-079".to_string(),
        delivery_generation: 80,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-079", "agent-079"),
    };
    let mismatched = DeliveryExecutionLink {
        delivery_id: "autre-delivery".to_string(),
        submission_id: "message-079".to_string(),
        execution_id: "execution-message-079".to_string(),
    };
    assert!(matches!(
        refused.begin_send_delivery_with_execution(
            &refused_key,
            &refused_delivery,
            None,
            &mismatched
        ),
        Err(IdempotencyError::InvalidDelivery)
    ));
    assert_eq!(delivery_row_count(&refused, "delivery-refusee"), 0);
}

#[test]
fn spec_079_remise_dispatching_bloque_une_nouvelle_continuation() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let primary_key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-recovery-079".to_string(),
        recipient_instance_id: "instance-079".to_string(),
        delivery_generation: 81,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-079", "agent-079"),
    };
    let link = DeliveryExecutionLink {
        delivery_id: delivery.delivery_id.clone(),
        submission_id: "message-079".to_string(),
        execution_id: "execution-recovery-079".to_string(),
    };
    store
        .begin_send_delivery_with_execution(&primary_key, &delivery, None, &link)
        .unwrap();

    assert_eq!(
        store
            .dispatching_delivery_for_execution("execution-recovery-079", "instance-079", NOW)
            .unwrap(),
        Some(delivery.clone())
    );
    assert_eq!(
        store
            .dispatching_delivery_for_execution(
                "execution-recovery-079",
                "instance-apres-redemarrage",
                NOW,
            )
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .dispatching_delivery_for_execution_any_instance("execution-recovery-079", NOW,)
            .unwrap(),
        Some(delivery.clone())
    );
    store
        .acknowledge_send_delivery(
            &delivery.delivery_id,
            &delivery.recipient_instance_id,
            delivery.delivery_generation,
        )
        .unwrap();
    assert_eq!(
        store
            .dispatching_delivery_for_execution("execution-recovery-079", "instance-079", NOW,)
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .dispatching_delivery_for_execution_any_instance("execution-recovery-079", NOW,)
            .unwrap(),
        None
    );
}

#[test]
fn retry_en_vol_rejoue_unknown_puis_accepted_apres_accuse() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-retry".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 9,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        })
    );
    store
        .acknowledge_send_delivery("delivery-retry", "instance-1", 9)
        .unwrap();
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::Accepted {
            expires_at: NOW + HORIZON
        })
    );
}

fn sample_message_bytes(id: &str, to: &str) -> Vec<u8> {
    let mut message = bridget_core::BridgetMessage::new("sender-peer", to, "corps collège");
    message.id = id.to_string();
    serde_json::to_vec(&message).expect("message sérialisable")
}

fn ledger_rows_for(store: &IdempotencyStore, id: &str) -> usize {
    store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM ledger WHERE id = ?1",
            params![id],
            |row| row.get::<_, usize>(0),
        )
        .unwrap()
}

fn delivery_phase(store: &IdempotencyStore, delivery_id: &str) -> String {
    store
        .conn
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id = ?1",
            params![delivery_id],
            |row| row.get(0),
        )
        .unwrap()
}

fn delivery_row_count(store: &IdempotencyStore, delivery_id: &str) -> usize {
    store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id = ?1",
            params![delivery_id],
            |row| row.get::<_, usize>(0),
        )
        .unwrap()
}

/// Oracle : enveloppe illisible → erreur explicite, aucune remise créée.
/// Meurt si un `.ok()` silencieux réapparaît et laisse passer une remise
/// `dispatching` sans ligne au ledger.
#[test]
fn begin_send_refuse_enveloppe_illisible_sans_creer_remise() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let err = store
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-corrupt".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 1,
                expires_at: NOW + HORIZON,
                message_bytes: b"pas-du-json-message".to_vec(),
            },
        )
        .expect_err("enveloppe illisible doit remonter");
    assert!(
        matches!(err, IdempotencyError::CorruptRecord(_)),
        "attendu CorruptRecord, obtenu {err:?}"
    );
    assert_eq!(delivery_row_count(&store, "delivery-corrupt"), 0);
    assert_eq!(ledger_rows_for(&store, "message-1"), 0);
    assert_eq!(store.send_delivery(&key).unwrap(), None);
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT state FROM idempotency_records WHERE idempotency_key = 'message-1'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "prepared",
        "parse avant transaction : pas de demi-écriture dispatching"
    );
}

/// Oracle (ii) : un message vers un destinataire encore non accusé est
/// VISIBLE au ledger ; la phase reste `dispatching` (accusé distinct).
#[test]
fn ledger_visible_pendant_dispatching_avant_accuse() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-visible".to_string(),
        recipient_instance_id: "instance-busy".to_string(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("msg-visible-busy", "peer-busy"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    assert_eq!(delivery_phase(&store, "delivery-visible"), "dispatching");
    assert_eq!(
        ledger_rows_for(&store, "msg-visible-busy"),
        1,
        "le fait d'émission doit être au ledger avant DeliverAcked"
    );
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        }),
        "outcome_unknown reste nominal tant que l'accusé manque"
    );
    store
        .acknowledge_send_delivery("delivery-visible", "instance-busy", 3)
        .unwrap();
    assert_eq!(delivery_phase(&store, "delivery-visible"), "acked");
    assert_eq!(
        ledger_rows_for(&store, "msg-visible-busy"),
        1,
        "l'accusé ne doit pas dupliquer la ligne d'émission"
    );
    assert_eq!(
        reserve(&store, b"canon"),
        Reservation::Replayed(LookupResult::Accepted {
            expires_at: NOW + HORIZON
        })
    );
}

/// Oracle (iii) : les remises d'une instance morte sont reprises sur la
/// nouvelle instance (réassignation), sans toucher aux phases terminales.
#[test]
fn remises_dispatching_migrees_au_changement_d_instance() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-orphan".to_string(),
        recipient_instance_id: "instance-morte".to_string(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("msg-orphan", "peer-respawn"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    assert_eq!(
        store
            .reassign_dispatching_deliveries("instance-morte", "instance-vivante", NOW)
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .dispatching_deliveries_for_instance("instance-morte", NOW)
            .unwrap()
            .len(),
        0
    );
    let revived = store
        .dispatching_deliveries_for_instance("instance-vivante", NOW)
        .unwrap();
    assert_eq!(revived.len(), 1);
    assert_eq!(revived[0].delivery_id, "delivery-orphan");
    assert_eq!(revived[0].recipient_instance_id, "instance-vivante");
    // Ack sur la nouvelle instance doit aboutir.
    store
        .acknowledge_send_delivery("delivery-orphan", "instance-vivante", 7)
        .unwrap();
    assert_eq!(delivery_phase(&store, "delivery-orphan"), "acked");
    assert_eq!(
        store
            .reassign_dispatching_deliveries("instance-vivante", "autre", NOW)
            .unwrap(),
        0,
        "une remise déjà acked ne migre pas"
    );
}

/// ORACLE — purge de présence ⇒ phase `orphaned`, lookup ≠ outcome_unknown.
#[test]
fn purge_orpheline_les_remises_dispatching_sans_outcome_unknown() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-purge".to_string(),
        recipient_instance_id: "instance-purgée".to_string(),
        delivery_generation: 2,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("msg-purge", "relec-zombie"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    let notices = store
        .orphan_dispatching_for_instance(
            "instance-purgée",
            "destinataire purgé — présence absente ; remise orpheline",
        )
        .unwrap();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].delivery_id, "delivery-purge");
    assert_eq!(notices[0].target, "relec-zombie");
    assert_eq!(delivery_phase(&store, "delivery-purge"), "orphaned");
    assert!(
        store
            .dispatching_deliveries_for_instance("instance-purgée", NOW)
            .unwrap()
            .is_empty(),
        "plus aucune remise en vol après orphelinage"
    );
    assert_eq!(
        store.lookup(&key, NOW).unwrap(),
        LookupResult::Orphaned {
            expires_at: NOW + HORIZON,
            reason: "destinataire purgé — présence absente ; remise orpheline".to_string(),
        },
        "le rejeu doit dire orphelin, jamais outcome_unknown"
    );
    assert_eq!(
        store.orphaned_delivery_id(&key).unwrap().as_deref(),
        Some("delivery-purge")
    );
    assert!(
        store
            .orphan_dispatching_for_instance("instance-purgée", "rejeu")
            .unwrap()
            .is_empty(),
        "orphelinage absorbant"
    );
}

#[test]
fn purge_expired_removes_its_delivery_through_the_foreign_key() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    store
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-expired".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 2,
                expires_at: NOW + HORIZON,
                message_bytes: sample_message_bytes("message-1", "peer-1"),
            },
        )
        .unwrap();
    assert_eq!(store.purge_expired(NOW + HORIZON).unwrap(), 1);
    assert_eq!(store.send_delivery(&key).unwrap(), None);
}

#[test]
fn failed_delivery_insert_rolls_back_the_dispatch_transition() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let first = key();
    let second =
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message-2").unwrap();
    assert!(matches!(
        reserve(&store, b"first"),
        Reservation::Prepared { .. }
    ));
    store
        .begin_send_delivery(
            &first,
            &SendDelivery {
                delivery_id: "same-delivery".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 3,
                expires_at: NOW + HORIZON,
                message_bytes: sample_message_bytes("message-1", "peer-1"),
            },
        )
        .unwrap();
    assert!(matches!(
        store
            .reserve(&second, b"second", NOW, HORIZON, NOW, 30)
            .unwrap(),
        Reservation::Prepared { .. }
    ));
    assert!(
        store
            .begin_send_delivery(
                &second,
                &SendDelivery {
                    delivery_id: "same-delivery".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 4,
                    expires_at: NOW + HORIZON,
                    message_bytes: sample_message_bytes("message-2", "peer-1"),
                },
            )
            .is_err()
    );
    assert_eq!(
        store.lookup(&second, NOW).unwrap(),
        LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        }
    );
    assert_eq!(store.send_delivery(&second).unwrap(), None);
}

#[test]
fn failed_reply_tracking_rolls_back_then_a_retry_after_restart_prepares_both() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-reply-fault-{}-{}.db",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let key = key();
    let delivery = SendDelivery {
        delivery_id: "delivery-reply".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 5,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "agent-2"),
    };
    let reply = ReplyTracking {
        request_id: "request-reply".to_string(),
        sender: "guichet".to_string(),
        target: "agent-2".to_string(),
        created_at: NOW,
        deadline_at: NOW + 60,
    };
    {
        let mut store = IdempotencyStore::open(&path).unwrap();
        assert!(matches!(
            reserve(&store, b"canon"),
            Reservation::Prepared { .. }
        ));
        // Injection de faute : la clé primaire déjà présente force l'INSERT
        // de suivi à échouer après l'INSERT de remise, dans la transaction.
        store
                .conn
                .execute(
                    "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at, escalation_level)
                     VALUES ('request-reply', 'old', 'target', 'open', 1, 2, 0)",
                    [],
                )
                .unwrap();
        assert!(
            store
                .begin_send_delivery_with_reply(&key, &delivery, &reply)
                .is_err()
        );
        assert_eq!(store.send_delivery(&key).unwrap(), None);
        assert_eq!(
            store.lookup(&key, NOW).unwrap(),
            LookupResult::OutcomeUnknown {
                expires_at: NOW + HORIZON
            }
        );
    }
    let mut reopened = IdempotencyStore::open(&path).unwrap();
    assert!(matches!(
        reserve(&reopened, b"canon"),
        Reservation::Replayed(LookupResult::OutcomeUnknown { .. })
    ));
    assert_eq!(
        reopened.prepared_expiry(&key, NOW).unwrap(),
        Some(NOW + HORIZON)
    );
    reopened
        .conn
        .execute(
            "DELETE FROM tracked_requests WHERE id = 'request-reply'",
            [],
        )
        .unwrap();
    reopened
        .begin_send_delivery_with_reply(&key, &delivery, &reply)
        .unwrap();
    assert_eq!(reopened.send_delivery(&key).unwrap(), Some(delivery));
    assert_eq!(
        reopened
            .conn
            .query_row(
                "SELECT sender || ':' || target FROM tracked_requests WHERE id = 'request-reply'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "guichet:agent-2"
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reject_prepared_publishes_only_the_terminal_refusal() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    store
        .reject_prepared(&key, "routing", "cible absente")
        .unwrap();
    assert_eq!(
        store.lookup(&key, NOW).unwrap(),
        LookupResult::Rejected {
            category: "routing".to_string(),
            reason: "cible absente".to_string(),
            expires_at: NOW + HORIZON,
        }
    );
}

#[test]
fn invalid_horizons_are_refused_for_a_first_reservation() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        store.reserve(&key(), b"zero", NOW, 0, NOW, 30),
        Err(IdempotencyError::InvalidHorizon)
    ));
    assert!(matches!(
        store.reserve(&key(), b"negative", NOW, -1, NOW, 30),
        Err(IdempotencyError::InvalidHorizon)
    ));
}

#[test]
fn migration_v1_classe_les_remises_sans_payload_sans_bloquer_les_valides() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-v1-{}-{}.db",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE idempotency_records (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                state TEXT NOT NULL,
                public_result_kind TEXT,
                public_result_category TEXT,
                public_result_reason TEXT,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
            );
            CREATE TABLE send_deliveries (
                delivery_id TEXT PRIMARY KEY,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                recipient_instance_id TEXT NOT NULL,
                delivery_generation INTEGER NOT NULL,
                phase TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                message_bytes BLOB
            );",
    )
    .unwrap();
    for (key, delivery) in [("legacy", "delivery-legacy"), ("valid", "delivery-valid")] {
        conn.execute(
                "INSERT INTO idempotency_records VALUES (?1, 'send', ?2, X'00', 'dispatching', NULL, NULL, NULL, ?3, ?4)",
                params!["012_scope_aaaaaaaaaaaa", key, NOW, NOW + HORIZON],
            )
            .unwrap();
        conn.execute(
                "INSERT INTO send_deliveries VALUES (?1, ?2, 'send', ?3, 'instance-1', 1, 'dispatching', ?4, ?5)",
                params![
                    delivery,
                    "012_scope_aaaaaaaaaaaa",
                    key,
                    NOW + HORIZON,
                    (key == "valid").then(|| b"payload".to_vec()),
                ],
            )
            .unwrap();
    }
    drop(conn);

    let mut store = IdempotencyStore::open(&path).unwrap();
    let deliveries = store
        .dispatching_deliveries_for_instance("instance-1", NOW)
        .unwrap();
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].delivery_id, "delivery-valid");
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-legacy'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "indeterminate"
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM idempotency_schema_migrations WHERE version = 2",
                [],
                |row| row.get::<_, u32>(0),
            )
            .unwrap(),
        1
    );
    std::fs::remove_file(path).unwrap();
}

/// ORACLE C1 — chemin RUNTIME, enveloppe PRÉSENTE.
///
/// C'est le cas de l'unique occurrence réelle relevée en production : une
/// remise que le daemon n'a pas pu redéployer (échec de reprise) ou que le
/// wrapper a déclarée indéterminée passe en quarantaine. Son enveloppe est
/// intacte, sa ligne est là — et c'est précisément ce qui la rendait
/// crédible : `send_delivery` la remontait, l'appelant lisait « en vol,
/// rejouez », rc=0, pendant les 7 jours de l'horizon. Or la quarantaine est
/// un état ABSORBANT : plus aucune transition n'en sort, ce message ne sera
/// jamais accusé.
///
/// Cet oracle tue le mutant exact qui retire la garde de phase : avec une
/// enveloppe présente, l'écart porte sur le sens métier et non sur le
/// décodage SQLite.
#[test]
fn une_remise_en_quarantaine_ne_remonte_plus_comme_une_remise_en_vol() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = key();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-quarantaine".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 4,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key, &delivery).unwrap();
    // Avant la quarantaine, la remise est bien en vol : sans ce constat,
    // l'oracle passerait aussi pour une clé qui n'a jamais rien déposé.
    assert_eq!(store.send_delivery(&key).unwrap(), Some(delivery.clone()));

    store
        .mark_delivery_indeterminate("delivery-quarantaine", "instance-1", 4)
        .unwrap();

    assert_eq!(
        store
            .send_delivery_mutant_sans_filtre_de_phase(&key)
            .unwrap(),
        Some(delivery),
        "contrôle positif du mutant : sans garde de phase, une enveloppe \
             présente ferait passer la quarantaine pour une remise en vol"
    );
    assert_eq!(
        store.send_delivery(&key).unwrap(),
        None,
        "une remise en quarantaine doit cesser d'attester un dépôt : \
             sans cela, l'appelant lit « en vol, rejouez » sur un message \
             que plus rien n'accusera jamais"
    );
    // L'enveloppe reste en base — la ligne n'est pas supprimée, elle est
    // seulement écartée de ce que « remise en vol » désigne.
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-quarantaine'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "indeterminate"
    );
}

/// ORACLE C1 — chemin MIGRATION, enveloppe ABSENTE.
///
/// La seconde porte vers la quarantaine, et la plus large : la migration v2
/// écarte d'un coup toutes les remises v1 dépourvues d'enveloppe. Une
/// montée de version pouvait donc fabriquer en masse des remises
/// définitivement inaccusables qui continuaient à s'annoncer comme des
/// dépôts réussis.
///
/// Cet oracle garde la projection nullable : même sous le mutant exact qui
/// retire la garde de phase, l'absence d'enveloppe reste non relivrable et
/// ne fuit jamais en `InvalidColumnType`. L'oracle runtime voisin, dont
/// l'enveloppe est présente, est celui qui tue ce mutant sur le sens métier.
#[test]
fn une_remise_mise_en_quarantaine_par_la_migration_n_atteste_plus_un_depot() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-quarantaine-migration-{}-{}.db",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE idempotency_records (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                state TEXT NOT NULL,
                public_result_kind TEXT,
                public_result_category TEXT,
                public_result_reason TEXT,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
            );
            CREATE TABLE send_deliveries (
                delivery_id TEXT PRIMARY KEY,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                recipient_instance_id TEXT NOT NULL,
                delivery_generation INTEGER NOT NULL,
                phase TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                message_bytes BLOB
            );",
    )
    .unwrap();
    conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', 'message-1', X'00', 'dispatching', NULL, NULL, NULL, ?2, ?3)",
            params!["012_scope_aaaaaaaaaaaa", NOW, NOW + HORIZON],
        )
        .unwrap();
    conn.execute(
            "INSERT INTO send_deliveries VALUES ('delivery-sans-enveloppe', ?1, 'send', 'message-1', 'instance-1', 1, 'dispatching', ?2, NULL)",
            params!["012_scope_aaaaaaaaaaaa", NOW + HORIZON],
        )
        .unwrap();
    drop(conn);

    // L'ouverture applique la migration, qui met la remise en quarantaine.
    let store = IdempotencyStore::open(&path).unwrap();
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-sans-enveloppe'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "indeterminate",
        "prémisse de l'oracle : la migration doit bien avoir mis en quarantaine"
    );

    assert_eq!(
        store
            .send_delivery_mutant_sans_filtre_de_phase(&key())
            .unwrap(),
        None,
        "le mutant de phase ne doit pas réintroduire l'échec de décodage : \
             sans enveloppe, la remise reste non relivrable"
    );
    assert_eq!(
        store.send_delivery(&key()).unwrap(),
        None,
        "une remise mise en quarantaine par la migration ne doit pas \
             attester un dépôt : une montée de version en fabriquerait en masse"
    );
    std::fs::remove_file(path).unwrap();
}

// ========== ORACLES JURY (relecteur fable2 du lot parent) ==========
// PROPRIÉTÉ SOUS TEST — celle que le lot déclare tenir :
// « toute lecture de send_deliveries est totale sur le schéma hérité ;
//   si message_bytes vaut NULL, aucun Sqlite(InvalidColumnType) ne fuit. »
// Le lot parent la tient sur send_delivery. Ces deux oracles couvrent les
// DEUX AUTRES chemins de lecture de message_bytes restés ouverts. Base
// héritée où la quarantaine ne rejoue pas (v2 déjà marquée).

fn base_heritee_dispatching_sans_enveloppe() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bridget-jury-lecture-totale-{}-{}.db",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE idempotency_records (
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                canonical_bytes BLOB NOT NULL,
                state TEXT NOT NULL,
                public_result_kind TEXT,
                public_result_category TEXT,
                public_result_reason TEXT,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                PRIMARY KEY (issuer_scope, operation_kind, idempotency_key)
            );
            CREATE TABLE send_deliveries (
                delivery_id TEXT PRIMARY KEY,
                issuer_scope TEXT NOT NULL,
                operation_kind TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                recipient_instance_id TEXT NOT NULL,
                delivery_generation INTEGER NOT NULL,
                phase TEXT NOT NULL,
                expires_at INTEGER NOT NULL,
                message_bytes BLOB
            );
            CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
            INSERT INTO idempotency_schema_migrations(version) VALUES (2);
            INSERT INTO idempotency_schema_migrations(version) VALUES (3);",
    )
    .unwrap();
    conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', 'message-1', X'00', 'dispatching', NULL, NULL, NULL, ?2, ?3)",
            params!["012_scope_aaaaaaaaaaaa", NOW, NOW + HORIZON],
        )
        .unwrap();
    conn.execute(
            "INSERT INTO send_deliveries VALUES ('delivery-encore-en-vol', ?1, 'send', 'message-1', 'instance-1', 1, 'dispatching', ?2, NULL)",
            params!["012_scope_aaaaaaaaaaaa", NOW + HORIZON],
        )
        .unwrap();
    drop(conn);
    path
}

fn ajouter_remise_valide_heritee(path: &Path, key: &str, delivery_id: &str, generation: u64) {
    let conn = Connection::open(path).unwrap();
    conn.execute(
            "INSERT INTO idempotency_records VALUES (?1, 'send', ?2, X'00', 'dispatching', NULL, NULL, NULL, ?3, ?4)",
            params!["012_scope_aaaaaaaaaaaa", key, NOW, NOW + HORIZON],
        )
        .unwrap();
    conn.execute(
            "INSERT INTO send_deliveries VALUES (?1, ?2, 'send', ?3, 'instance-1', ?4, 'dispatching', ?5, ?6)",
            params![
                delivery_id,
                "012_scope_aaaaaaaaaaaa",
                key,
                generation,
                NOW + HORIZON,
                sample_message_bytes(key, "instance-1"),
            ],
        )
        .unwrap();
}

/// Chemin de REPRISE (daemon.rs:2293). `collect::<Result<Vec<_>>>` : une
/// seule ligne sans enveloppe fait échouer la reprise ENTIÈRE de
/// l'instance, y compris ses remises légitimes.
#[test]
fn jury_lecture_totale_chemin_reprise() {
    let path = base_heritee_dispatching_sans_enveloppe();
    let mut store = IdempotencyStore::open(&path).unwrap();
    let lu = store.dispatching_deliveries_for_instance("instance-1", NOW);
    let _ = std::fs::remove_file(&path);
    assert!(
        lu.is_ok(),
        "lecture non totale sur le chemin de reprise : {:?}",
        lu.err()
    );
}

/// Chemin d'ACCUSÉ. La lecture stricte de message_bytes précède même la
/// garde de phase : aucune phase ne protège de la fuite.
#[test]
fn jury_lecture_totale_chemin_accuse() {
    let path = base_heritee_dispatching_sans_enveloppe();
    let mut store = IdempotencyStore::open(&path).unwrap();
    let lu = store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1);
    let _ = std::fs::remove_file(&path);
    assert!(
        !matches!(
            lu,
            Err(IdempotencyError::Sqlite(
                rusqlite::Error::InvalidColumnType(..)
            ))
        ),
        "lecture non totale sur le chemin d'accusé : {:?}",
        lu.err()
    );
}

#[test]
fn reprise_reclasse_sans_enveloppe_en_indeterminate() {
    let path = base_heritee_dispatching_sans_enveloppe();
    let mut store = IdempotencyStore::open(&path).unwrap();

    let deliveries = store
        .dispatching_deliveries_for_instance("instance-1", NOW)
        .expect("la reprise doit lire totalement le schéma hérité");

    assert_eq!(deliveries, Vec::<SendDelivery>::new());
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-encore-en-vol'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "indeterminate",
        "écarter une enveloppe absente sans reclasser laisserait un état absorbant silencieux"
    );
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn reprise_isole_deux_remises_valides_d_une_sans_enveloppe() {
    let path = base_heritee_dispatching_sans_enveloppe();
    ajouter_remise_valide_heritee(&path, "message-valid-a", "delivery-valid-a", 2);
    ajouter_remise_valide_heritee(&path, "message-valid-b", "delivery-valid-b", 3);
    let mut store = IdempotencyStore::open(&path).unwrap();

    let ids = store
        .dispatching_deliveries_for_instance("instance-1", NOW)
        .expect("une remise illisible ne doit pas faire échouer toute l'instance")
        .into_iter()
        .map(|delivery| delivery.delivery_id)
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        vec![
            "delivery-valid-a".to_string(),
            "delivery-valid-b".to_string(),
        ]
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-encore-en-vol'",
                [],
                |row| row.get::<_, String>(0),
            )
            .unwrap(),
        "indeterminate",
        "rendre les valides ne suffit pas si la remise écartée reste dispatching"
    );
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn accuse_sans_enveloppe_respecte_les_trois_verdicts_metier() {
    commencer_capture_trace_sans_enveloppe();

    let path_dispatching = base_heritee_dispatching_sans_enveloppe();
    let mut store = IdempotencyStore::open(&path_dispatching).unwrap();
    let acknowledgement =
        store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1);
    let durable_state = store
        .conn
        .query_row(
            "SELECT d.phase, r.state, r.public_result_kind
                 FROM send_deliveries d
                 JOIN idempotency_records r
                   ON r.issuer_scope = d.issuer_scope
                  AND r.operation_kind = d.operation_kind
                  AND r.idempotency_key = d.idempotency_key
                 WHERE d.delivery_id = 'delivery-encore-en-vol'",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .unwrap();
    let traces = traces_sans_enveloppe();
    assert!(
        matches!(acknowledgement, Ok(None))
            && durable_state
                == (
                    "acked".to_string(),
                    "terminal".to_string(),
                    Some("accepted".to_string()),
                )
            && traces.iter().any(|trace| {
                trace.contains("delivery-encore-en-vol")
                    && trace.contains("corrélation in_reply_to impossible")
            }),
        "l'accusé et sa trace doivent survivre à l'enveloppe locale absente: résultat={acknowledgement:?}, état={durable_state:?}, traces={traces:?}"
    );
    drop(store);
    std::fs::remove_file(path_dispatching).unwrap();

    let path_acked = base_heritee_dispatching_sans_enveloppe();
    let conn = Connection::open(&path_acked).unwrap();
    conn.execute(
        "UPDATE send_deliveries SET phase = 'acked' WHERE delivery_id = 'delivery-encore-en-vol'",
        [],
    )
    .unwrap();
    conn.execute(
            "UPDATE idempotency_records SET state = 'terminal', public_result_kind = 'accepted' WHERE idempotency_key = 'message-1'",
            [],
        )
        .unwrap();
    drop(conn);
    let mut store = IdempotencyStore::open(&path_acked).unwrap();
    assert_eq!(
        store
            .acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1)
            .expect("un accusé déjà enregistré reste idempotent"),
        None
    );
    drop(store);
    std::fs::remove_file(path_acked).unwrap();

    let path_indeterminate = base_heritee_dispatching_sans_enveloppe();
    let conn = Connection::open(&path_indeterminate).unwrap();
    conn.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate' WHERE delivery_id = 'delivery-encore-en-vol'",
            [],
        )
        .unwrap();
    drop(conn);
    let mut store = IdempotencyStore::open(&path_indeterminate).unwrap();
    assert!(matches!(
        store.acknowledge_send_delivery("delivery-encore-en-vol", "instance-1", 1),
        Err(IdempotencyError::InvalidDelivery)
    ));
    assert_eq!(
        delivery_phase(&store, "delivery-encore-en-vol"),
        "indeterminate"
    );
    drop(store);
    std::fs::remove_file(path_indeterminate).unwrap();
}

#[test]
fn issuer_scope_requires_a_base64url_sized_opaque_value() {
    assert!(IdempotencyKey::new("scope-too-short", OperationKind::Send, "message").is_err());
    assert!(IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "message").is_ok());
    assert!(
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa!", OperationKind::Send, "message").is_err()
    );
}

#[test]
fn migration_v6_ajoute_les_references_projet_apres_une_base_deja_en_v5() {
    let path = std::env::temp_dir().join(format!(
        "bridget-idempotency-v6-{}.db",
        uuid::Uuid::new_v4()
    ));
    let legacy = Connection::open(&path).unwrap();
    legacy
        .execute_batch(
            "CREATE TABLE idempotency_schema_migrations (version INTEGER PRIMARY KEY);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (2);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (3);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (4);
                 INSERT INTO idempotency_schema_migrations(version) VALUES (5);
                 CREATE TABLE agent_links (
                    link_id TEXT PRIMARY KEY,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL UNIQUE,
                    parent_execution_id TEXT,
                    objective_id TEXT,
                    delegation_id TEXT,
                    role TEXT NOT NULL,
                    agent_path TEXT NOT NULL,
                    state TEXT NOT NULL,
                    created_at INTEGER NOT NULL,
                    closed_at INTEGER,
                    revision INTEGER NOT NULL
                 );
                 CREATE TABLE agent_link_events (
                    cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id TEXT NOT NULL UNIQUE,
                    link_id TEXT NOT NULL,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL,
                    state TEXT NOT NULL,
                    observed_at INTEGER NOT NULL
                 );
                 CREATE TABLE delegated_runtime_events (
                    cursor INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id TEXT NOT NULL UNIQUE,
                    link_id TEXT NOT NULL,
                    parent_instance_id TEXT NOT NULL,
                    child_instance_id TEXT NOT NULL,
                    child_execution_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    code TEXT NOT NULL,
                    reference TEXT NOT NULL,
                    observed_at INTEGER NOT NULL,
                    acknowledged_at INTEGER
                 );",
        )
        .unwrap();
    drop(legacy);

    let store = IdempotencyStore::open(&path).unwrap();
    for table in [
        "agent_links",
        "agent_link_events",
        "delegated_runtime_events",
    ] {
        let has_project_id: bool = store
                .conn
                .query_row(
                    &format!(
                        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name = 'project_id')"
                    ),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
        assert!(has_project_id, "{table} migre project_id");
    }
    let has_v6: bool = store
        .conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM idempotency_schema_migrations WHERE version = 6)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(has_v6);
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn two_concurrent_reservations_have_one_winner() {
    let db_path =
        std::env::temp_dir().join(format!("bridget-idempotency-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&db_path);
    let barrier = Arc::new(Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let path = db_path.clone();
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            let store = IdempotencyStore::open(&path).unwrap();
            barrier.wait();
            reserve(&store, b"canon")
        }));
    }
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Reservation::Prepared { .. }))
            .count(),
        1
    );
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Reservation::Replayed(_)))
            .count(),
        1
    );
    let _ = std::fs::remove_file(db_path);
}
#[test]
fn spec_068_faits_runtime_delegues_restent_ordonnes_et_accuses() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let project = ProjectReference {
        project_id: "project-068".to_string(),
        binding_generation: 4,
    };
    store
        .create_agent_link(&AgentLinkRecord {
            link_id: "link-068".to_string(),
            parent_instance_id: "instance-parent".to_string(),
            child_instance_id: "instance-child".to_string(),
            parent_execution_id: Some("execution-parent".to_string()),
            objective_id: Some("objective-068".to_string()),
            delegation_id: Some("delegation-068".to_string()),
            project: Some(project.clone()),
            role: "worker".to_string(),
            agent_path: "parent/child".to_string(),
            state: AgentLinkState::Reserved,
            created_at: NOW,
            closed_at: None,
            revision: 0,
        })
        .unwrap();

    assert_eq!(
        store
            .agent_link_events_after("instance-parent", None)
            .unwrap()
            .as_slice(),
        &[AgentLinkEvent {
            cursor: 1,
            event_id: "link-068:0".to_string(),
            link_id: "link-068".to_string(),
            parent_instance_id: "instance-parent".to_string(),
            child_instance_id: "instance-child".to_string(),
            state: AgentLinkState::Reserved,
            observed_at: NOW,
            project: Some(project.clone()),
        }]
    );

    let warning_reference = format!("sha256:{}", "a".repeat(64));
    let failed_reference = format!("sha256:{}", "b".repeat(64));

    let warning = store
        .record_delegated_runtime_event(DelegatedRuntimeEventInput {
            child_instance_id: "instance-child".to_string(),
            child_execution_id: "execution-child".to_string(),
            kind: DelegatedRuntimeEventKind::Warning,
            code: "unsupported_provider_request".to_string(),
            reference: warning_reference.clone(),
            observed_at: NOW + 1,
        })
        .unwrap();
    let failed = store
        .record_delegated_runtime_event(DelegatedRuntimeEventInput {
            child_instance_id: "instance-child".to_string(),
            child_execution_id: "execution-child".to_string(),
            kind: DelegatedRuntimeEventKind::Failed,
            code: "provider_failed".to_string(),
            reference: failed_reference.clone(),
            observed_at: NOW + 2,
        })
        .unwrap();
    assert_eq!(warning.project, Some(project.clone()));
    assert_eq!(failed.project, Some(project.clone()));
    assert_eq!(
        store
            .record_delegated_runtime_event(DelegatedRuntimeEventInput {
                child_instance_id: "instance-child".to_string(),
                child_execution_id: "execution-child".to_string(),
                kind: DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".to_string(),
                reference: warning_reference,
                observed_at: NOW + 99,
            })
            .unwrap()
            .event_id,
        warning.event_id
    );
    let pending = store
        .delegated_runtime_events_for_parent("instance-parent")
        .unwrap();
    assert!(
        pending
            .iter()
            .all(|event| event.project == Some(project.clone()))
    );
    assert_eq!(
        pending
            .iter()
            .map(|event| event.event_id.as_str())
            .collect::<Vec<_>>(),
        vec![warning.event_id.as_str(), failed.event_id.as_str()]
    );
    assert!(matches!(
        store.acknowledge_delegated_runtime_event(&warning.event_id, "other-parent"),
        Err(IdempotencyError::InvalidDelegatedRuntimeEvent)
    ));
    assert!(
        store
            .acknowledge_delegated_runtime_event(&warning.event_id, "instance-parent")
            .unwrap()
    );
    assert!(
        !store
            .acknowledge_delegated_runtime_event(&warning.event_id, "instance-parent")
            .unwrap()
    );
    assert_eq!(
        store
            .delegated_runtime_events_for_parent("instance-parent")
            .unwrap()
            .iter()
            .map(|event| event.event_id.as_str())
            .collect::<Vec<_>>(),
        vec![failed.event_id.as_str()]
    );
}

/// Session 116 : une remise restée « en vol » après l'expiration de sa saga
/// passe en sort inconnu, et plus aucune remise de l'instance n'attend.
#[test]
fn spec116_remise_expiree_encore_en_vol_devient_sort_inconnu() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-116".to_string(),
        recipient_instance_id: "instance-disparue".to_string(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key(), &delivery).unwrap();
    let phase = |store: &IdempotencyStore| -> String {
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id = 'delivery-116'",
                [],
                |row| row.get(0),
            )
            .unwrap()
    };
    assert_eq!(
        store.settle_expired_dispatching(NOW + HORIZON - 1).unwrap(),
        0,
        "une saga encore valide n'est pas touchée"
    );
    assert_eq!(phase(&store), "dispatching");
    assert_eq!(store.settle_expired_dispatching(NOW + HORIZON).unwrap(), 1);
    assert_eq!(
        phase(&store),
        "indeterminate",
        "plus rien n'est affiché en vol"
    );
}

/// Session 116 : la purge vise les seuls envois, après une marge. Le
/// lancement d'un équipier géré, supprimé en cascade avec son enregistrement,
/// n'est jamais emporté.
#[test]
fn spec116_purge_des_envois_epargne_les_lancements_et_respecte_la_marge() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "delivery-116-purge".to_string(),
        recipient_instance_id: "instance-1".to_string(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer-1"),
    };
    store.begin_send_delivery(&key(), &delivery).unwrap();
    assert!(matches!(
        store
            .reserve(&spawn_key(), b"spawn", NOW, HORIZON, NOW, 30)
            .unwrap(),
        Reservation::Prepared { .. }
    ));
    assert_eq!(store.record_count().unwrap(), 2);
    let grace = 30 * 24 * 3600;
    assert_eq!(
        store
            .purge_expired_sends(NOW + HORIZON + grace - 1, grace)
            .unwrap(),
        0,
        "pendant la marge, un rejeu tardif reçoit encore « expiré »"
    );
    assert_eq!(
        store
            .purge_expired_sends(NOW + HORIZON + grace, grace)
            .unwrap(),
        1
    );
    assert_eq!(store.record_count().unwrap(), 1, "le lancement reste");
    assert_eq!(
        store.send_delivery(&key()).unwrap(),
        None,
        "la remise suit en cascade"
    );
}

/// Session 150 : `send_delivery_phase` rend la phase brute, y compris celles
/// que `send_delivery` masque, et `None` pour une clé sans remise.
#[test]
fn native150_purge_phase_de_remise_cinq_cas() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let delivery = |id: &str, instance: &str, generation: u64| SendDelivery {
        delivery_id: id.to_string(),
        recipient_instance_id: instance.to_string(),
        delivery_generation: generation,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes(id, "peer-1"),
    };
    let key_for =
        |id: &str| IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, id).unwrap();
    let begin = |store: &mut IdempotencyStore, id: &str, instance: &str, generation: u64| {
        let key = key_for(id);
        store
            .reserve(&key, id.as_bytes(), NOW, HORIZON, NOW, 30)
            .unwrap();
        store
            .begin_send_delivery(&key, &delivery(id, instance, generation))
            .unwrap();
        key
    };
    // Absente : aucune ligne pour la clé.
    assert_eq!(
        store.send_delivery_phase(&key_for("absente")).unwrap(),
        None
    );
    // dispatching : remise déposée, non accusée.
    let en_vol = begin(&mut store, "en-vol", "instance-1", 1);
    assert_eq!(
        store.send_delivery_phase(&en_vol).unwrap().as_deref(),
        Some("dispatching")
    );
    // acked.
    let accusee = begin(&mut store, "accusee", "instance-2", 2);
    store
        .acknowledge_send_delivery("accusee", "instance-2", 2)
        .unwrap();
    assert_eq!(
        store.send_delivery_phase(&accusee).unwrap().as_deref(),
        Some("acked")
    );
    // indeterminate : masquée par `send_delivery`, visible ici.
    let quarantaine = begin(&mut store, "quarantaine", "instance-3", 3);
    store
        .mark_delivery_indeterminate("quarantaine", "instance-3", 3)
        .unwrap();
    assert_eq!(store.send_delivery(&quarantaine).unwrap(), None);
    assert_eq!(
        store.send_delivery_phase(&quarantaine).unwrap().as_deref(),
        Some("indeterminate")
    );
    // orphaned.
    let orpheline = begin(&mut store, "orpheline", "instance-4", 4);
    store
        .orphan_dispatching_for_instance("instance-4", "destinataire purgé")
        .unwrap();
    assert_eq!(
        store.send_delivery_phase(&orpheline).unwrap().as_deref(),
        Some("orphaned")
    );
}

/// T006 : horloge injectée, borne SQL inclusive ; la garde pure tranche t==30/600.
#[test]
fn guidance156_store_compteurs_aux_bornes_30s_10min() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    for (at, actor, target, action, outcome) in [
        (NOW - 601, "parent156", "target156", "interrupt", "admitted"),
        (NOW - 600, "parent156", "target156", "interrupt", "admitted"),
        (NOW - 31, "parent156", "target156", "interrupt", "admitted"),
        (NOW - 30, "parent156", "target156", "interrupt", "admitted"),
        (NOW - 29, "other156", "target156", "interrupt", "admitted"),
        (
            NOW - 1,
            "parent156",
            "target156",
            "interrupt",
            "refused:referent_paused",
        ),
        (NOW, "human", "target156", "interrupt", "admitted"),
        (NOW, "parent156", "target156", "steer", "admitted"),
        (NOW, "parent156", "other-target156", "interrupt", "admitted"),
    ] {
        store
            .record_guidance_event(at, actor, target, action, "send_idempotent", None, outcome)
            .unwrap();
    }
    assert_eq!(
        store
            .count_interrupts_pair("parent156", "target156", NOW - 30)
            .unwrap(),
        (1, Some(NOW - 30))
    );
    assert_eq!(
        store
            .count_interrupts_pair("parent156", "target156", NOW - 29)
            .unwrap(),
        (0, None)
    );
    assert_eq!(
        store
            .count_interrupts_target("target156", NOW - 600)
            .unwrap(),
        (4, Some(NOW - 31)),
        "3e plus récent, pas le minimum lorsque plus de trois admissions sont présentes"
    );
    assert_eq!(
        store
            .count_interrupts_target("target156", NOW - 599)
            .unwrap(),
        (3, Some(NOW - 31))
    );
    assert_eq!(
        store
            .count_interrupts_target("target156", NOW - 30)
            .unwrap(),
        (2, None)
    );
    assert_eq!(
        store
            .count_interrupts_pair("human", "target156", NOW - 600)
            .unwrap(),
        (0, None)
    );
}

/// T006 : le reçu existant suffit ; le magasin de rythme peut être indisponible.
#[test]
fn guidance156_store_rejeu_sans_lecture_des_evenements() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    assert!(matches!(
        reserve(&store, b"canon156"),
        Reservation::Prepared { .. }
    ));
    let delivery = SendDelivery {
        delivery_id: "guidance-replay156".into(),
        recipient_instance_id: "instance156".into(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer156"),
    };
    let reply = ReplyTracking {
        request_id: "message-1".into(),
        sender: "sender-peer".into(),
        target: "peer156".into(),
        created_at: NOW,
        deadline_at: NOW + HORIZON,
    };
    let link = DeliveryExecutionLink {
        delivery_id: delivery.delivery_id.clone(),
        submission_id: "submission156".into(),
        execution_id: "execution156".into(),
    };
    let guidance = (
        NOW,
        "parent156",
        "peer156",
        "interrupt",
        "send_idempotent",
        Some("message-1"),
    );
    // Échec réel SQLite APRÈS la remise/ledger/suivi/lien, pas un mock du verdict.
    store
        .conn
        .execute_batch(
            "CREATE TEMP TRIGGER guidance156_admission_refusee
         BEFORE INSERT ON guidance_events
         BEGIN SELECT RAISE(ABORT,'guidance156_persistence_unavailable'); END;",
        )
        .unwrap();
    let failed = store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            Some(&reply),
            Some(&link),
            Some(guidance),
            None,
            None,
        )
        .unwrap_err();
    assert!(
        failed
            .to_string()
            .contains("guidance156_persistence_unavailable"),
        "{failed}"
    );
    assert_eq!(
        store.prepared_expiry(&key(), NOW).unwrap(),
        Some(NOW + HORIZON)
    );
    assert!(store.send_delivery(&key()).unwrap().is_none());
    assert!(
        store
            .delivery_execution_link(&delivery.delivery_id)
            .unwrap()
            .is_none()
    );
    assert_eq!(ledger_rows_for(&store, "message-1"), 0);
    let tracked: u32 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tracked_requests WHERE id='message-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tracked, 0);
    assert_eq!(
        store
            .count_interrupts_pair("parent156", "peer156", NOW - 30)
            .unwrap(),
        (0, None)
    );
    store
        .conn
        .execute_batch("DROP TRIGGER guidance156_admission_refusee;")
        .unwrap();
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            Some(&reply),
            Some(&link),
            Some(guidance),
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        store
            .delivery_execution_link(&delivery.delivery_id)
            .unwrap(),
        Some(link)
    );
    assert_eq!(ledger_rows_for(&store, "message-1"), 1);
    let tracked: u32 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM tracked_requests WHERE id='message-1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(tracked, 1);
    assert_eq!(
        store
            .count_interrupts_pair("parent156", "peer156", NOW - 30)
            .unwrap(),
        (1, Some(NOW))
    );
    assert!(matches!(
        store
            .reserve(&key(), b"canon156", NOW, HORIZON, NOW + 1, 30)
            .unwrap(),
        Reservation::Replayed(_)
    ));
    assert_eq!(
        store
            .count_interrupts_pair("parent156", "peer156", NOW - 30)
            .unwrap(),
        (1, Some(NOW)),
        "un rejeu ne compte pas une deuxième admission"
    );
    assert_eq!(
        store
            .acknowledge_send_delivery(&delivery.delivery_id, "instance156", 1)
            .unwrap(),
        None
    );
    let expected = store.lookup(&key(), NOW).unwrap();
    assert_eq!(
        expected,
        LookupResult::Accepted {
            expires_at: NOW + HORIZON
        }
    );
    store
        .conn
        .execute_batch("DROP TABLE guidance_events;")
        .unwrap();
    assert_eq!(
        store
            .reserve(&key(), b"canon156", NOW, HORIZON, NOW + 1, 30)
            .unwrap(),
        Reservation::Replayed(expected.clone())
    );
    assert_eq!(store.lookup(&key(), NOW + 1).unwrap(), expected);
    assert_eq!(store.record_count().unwrap(), 1);
}

/// T006 : clé/ordre de la saga existante, limites et filtres de la lecture seule.
#[test]
fn guidance156_store_file_rowid_apres_reaffectation() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for (offset, id, instance, expires_at) in [
        (0, "z-first156", "old156", NOW + HORIZON),
        (1, "a-second156", "old156", NOW + HORIZON),
        (2, "acked156", "old156", NOW + HORIZON),
        (3, "expired156", "old156", NOW + 4),
        (4, "foreign156", "foreign156", NOW + HORIZON),
    ] {
        let key = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, id).unwrap();
        store
            .reserve(&key, id.as_bytes(), NOW + offset, HORIZON, NOW + offset, 30)
            .unwrap();
        let delivery = SendDelivery {
            delivery_id: id.into(),
            recipient_instance_id: instance.into(),
            delivery_generation: 1,
            expires_at,
            message_bytes: sample_message_bytes(id, "peer156"),
        };
        store.begin_send_delivery(&key, &delivery).unwrap();
    }
    assert_eq!(
        store
            .acknowledge_send_delivery("acked156", "old156", 1)
            .unwrap(),
        None
    );
    let before: Vec<(String, i64)> = {
        let mut query = store
            .conn
            .prepare("SELECT delivery_id,rowid FROM send_deliveries ORDER BY rowid")
            .unwrap();
        let rows = query
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    };
    assert_eq!(
        store
            .reassign_dispatching_deliveries("old156", "new156", NOW + 5)
            .unwrap(),
        2
    );
    let after: Vec<(String, i64)> = {
        let mut query = store
            .conn
            .prepare("SELECT delivery_id,rowid FROM send_deliveries ORDER BY rowid")
            .unwrap();
        let rows = query
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    };
    assert_eq!(
        after, before,
        "réaffectation en place, sans nouvelle insertion"
    );
    let queued = store
        .queued_deliveries_for_recipient("new156", NOW + 5, 500, false)
        .unwrap();
    assert_eq!(
        queued
            .iter()
            .map(|(delivery, issued, _)| (delivery.delivery_id.as_str(), *issued))
            .collect::<Vec<_>>(),
        vec![("z-first156", NOW), ("a-second156", NOW + 1)]
    );
    assert_eq!(
        queued[0].0.message_bytes,
        sample_message_bytes("z-first156", "peer156")
    );
    assert_eq!(queued[0].0.recipient_instance_id, "new156");
    assert_eq!(
        store
            .queued_deliveries_for_recipient("new156", NOW + 5, 1, false)
            .unwrap(),
        queued[..1]
    );
    assert!(
        store
            .queued_deliveries_for_recipient("new156", NOW + 5, 0, false)
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .queued_deliveries_for_recipient("old156", NOW + 5, 200, false)
            .unwrap()
            .is_empty()
    );
}

/// T006 : mêmes tables150/résultats ; migration additive et entretien24h isolé.
#[test]
fn guidance156_store_migration150_sans_perte_et_retention24h() {
    let path = TempDbGuard::new("guidance156-migration150");
    let delivery = SendDelivery {
        delivery_id: "legacy150-guidance156".into(),
        recipient_instance_id: "legacy-instance150".into(),
        delivery_generation: 1,
        expires_at: NOW + HORIZON,
        message_bytes: sample_message_bytes("message-1", "peer150"),
    };
    {
        let mut legacy = IdempotencyStore::open(path.path()).unwrap();
        assert!(matches!(
            reserve(&legacy, b"legacy150"),
            Reservation::Prepared { .. }
        ));
        legacy.begin_send_delivery(&key(), &delivery).unwrap();
        // Les tables d'idempotence150 restent byte-pour-byte inchangées ; seuls
        // les objets additifs156 sont absents dans cette base privée héritée.
        legacy.conn.execute_batch("DROP TABLE IF EXISTS guidance_events; DROP INDEX IF EXISTS idx_send_deliveries_recipient_phase;").unwrap();
    }
    let store = IdempotencyStore::open(path.path()).unwrap();
    assert_eq!(store.record_count().unwrap(), 1);
    assert_eq!(store.send_delivery(&key()).unwrap(), Some(delivery.clone()));
    assert_eq!(
        store.lookup(&key(), NOW).unwrap(),
        LookupResult::OutcomeUnknown {
            expires_at: NOW + HORIZON
        }
    );
    let canonical: Vec<u8> = store
        .conn
        .query_row(
            "SELECT canonical_bytes FROM idempotency_records",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(canonical, b"legacy150");
    for index in [
        "idx_guidance_events_target",
        "idx_guidance_events_pair",
        "idx_send_deliveries_recipient_phase",
    ] {
        let count: u32 = store
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='index' AND name=?1",
                [index],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "index additif présent : {index}");
    }
    for at in [NOW - 86_401, NOW - 86_400, NOW - 30] {
        store
            .record_guidance_event(
                at,
                "parent156",
                "peer150",
                "interrupt",
                "control",
                Some("control156"),
                "admitted",
            )
            .unwrap();
    }
    assert_eq!(store.purge_guidance_events(NOW).unwrap(), 1);
    let oldest: i64 = store
        .conn
        .query_row("SELECT MIN(at) FROM guidance_events", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        oldest,
        NOW - 86_400,
        "strictement plus de24h, pas la borne exacte"
    );
    assert_eq!(store.record_count().unwrap(), 1);
    assert_eq!(store.send_delivery(&key()).unwrap(), Some(delivery));
    assert_eq!(
        store
            .queued_deliveries_for_recipient("legacy-instance150", NOW, 200, false)
            .unwrap()
            .len(),
        1
    );
}

// T003 : copies privées d’une sauvegarde158B cohérente fournie par ROOT.
// Aucun accès à la DB de production. L’absence d’entrée est un échec, jamais un skip.
fn held159_migration_fixture() -> (TempDbGuard, Connection) {
    use sha2::Digest;
    use std::os::unix::fs::PermissionsExt;
    let source = std::env::var_os("BRIDGET_MIGRATION159_SNAPSHOT").expect(
        "T003 exige BRIDGET_MIGRATION159_SNAPSHOT : sauvegarde réelle158B fournie par ROOT",
    );
    let db = TempDbGuard::new("bridget-held159-migration");
    std::fs::copy(&source, db.path()).expect("copier uniquement le snapshot privé fourni");
    std::fs::set_permissions(db.path(), std::fs::Permissions::from_mode(0o600)).unwrap();
    let digest = format!(
        "{:x}",
        sha2::Sha256::digest(std::fs::read(db.path()).unwrap())
    );
    assert_eq!(
        digest, "f40e18dbb32d236986bd749f56dbc161d48d6d8723b970e295b8b0f6e6579711",
        "l’entrée doit être la sauvegarde158B attestée, jamais une base reconstruite pour le test"
    );
    let conn = Connection::open(db.path()).unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    assert_eq!(
        conn.query_row(
            "SELECT MAX(version) FROM idempotency_schema_migrations",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        6
    );
    assert_eq!(
        conn.query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(held159_migration_rows(&conn, "PRAGMA foreign_key_check").is_empty());
    // Les phases manquantes et les liens sont ajoutés dans CETTE COPIE seulement.
    for phase in ["dispatching", "acked", "indeterminate", "orphaned"] {
        let id = format!("held159-{phase}");
        conn.execute(
            "INSERT INTO idempotency_records
            (issuer_scope,operation_kind,idempotency_key,canonical_bytes,state,issued_at,expires_at)
            VALUES ('159_scope_migration_aaaa','send',?1,?2,'dispatching',?3,?4)",
            params![id, id.as_bytes(), NOW, NOW + HORIZON],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO send_deliveries
            (delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,
             delivery_generation,phase,expires_at,message_bytes)
            VALUES (?1,'159_scope_migration_aaaa','send',?1,'held159-instance',1,?2,?3,?4)",
            params![
                id,
                phase,
                NOW + HORIZON,
                sample_message_bytes(&id, "held159-agent")
            ],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO send_delivery_execution_links
            (delivery_id,submission_id,execution_id) VALUES (?1,?2,?3)",
            params![
                id,
                format!("submission-{phase}"),
                format!("execution-{phase}")
            ],
        )
        .unwrap();
    }
    (db, conn)
}

// Lecture d’oracle commune : cellules SQLite exactes, sans conversion des BLOB en texte.
// Coût froid O(lignes × colonnes) ; seuls les dix cas T003 utilisent ces copies privées.
fn held159_migration_rows(conn: &Connection, sql: &str) -> Vec<Vec<rusqlite::types::Value>> {
    let mut stmt = conn.prepare(sql).unwrap();
    let count = stmt.column_count();
    let rows = stmt
        .query_map([], |row| {
            (0..count)
                .map(|index| row.get::<_, rusqlite::types::Value>(index))
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    rows
}

#[test]
fn held159_migration_sans_perte() {
    let (db, conn) = held159_migration_fixture();
    let counts = held159_migration_rows(
        &conn,
        "SELECT phase,COUNT(*) FROM send_deliveries GROUP BY phase ORDER BY phase",
    );
    assert_eq!(
        counts.len(),
        4,
        "les quatre phases158B doivent être représentées avant migration"
    );
    let deliveries = held159_migration_rows(
        &conn,
        "SELECT delivery_id,issuer_scope,operation_kind,
        idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes
        FROM send_deliveries ORDER BY delivery_id",
    );
    let records = held159_migration_rows(
        &conn,
        "SELECT * FROM idempotency_records
        ORDER BY issuer_scope,operation_kind,idempotency_key",
    );
    let tracking = held159_migration_rows(&conn, "SELECT * FROM tracked_requests ORDER BY id");
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT phase,COUNT(*) FROM send_deliveries GROUP BY phase ORDER BY phase"
        ),
        counts
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,issuer_scope,operation_kind,
        idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes
        FROM send_deliveries ORDER BY delivery_id"
        ),
        deliveries
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT * FROM idempotency_records
        ORDER BY issuer_scope,operation_kind,idempotency_key"
        ),
        records
    );
    assert_eq!(
        held159_migration_rows(&store.conn, "SELECT * FROM tracked_requests ORDER BY id"),
        tracking
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT MAX(version) FROM idempotency_schema_migrations",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        7,
        "la conservation seule en v6 ne prouve pas la migration"
    );
}

#[test]
fn held159_migration_ordre_preserve() {
    let (db, conn) = held159_migration_fixture();
    let before = held159_migration_rows(
        &conn,
        "SELECT delivery_id,rowid FROM send_deliveries ORDER BY rowid",
    );
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,queue_rank FROM send_deliveries ORDER BY queue_rank"
        ),
        before
    );
    let schema: String = store
        .conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='send_deliveries'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        schema.to_ascii_uppercase().contains("AUTOINCREMENT"),
        "le rang doit rester monotone après une purge, pas seulement valoir MAX+1"
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT name FROM pragma_index_info('idx_send_deliveries_recipient_phase') ORDER BY seqno"
        ),
        vec![
            vec![rusqlite::types::Value::Text(
                "recipient_instance_id".to_string()
            )],
            vec![rusqlite::types::Value::Text("phase".to_string())],
            vec![rusqlite::types::Value::Text("queue_rank".to_string())]
        ]
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT name FROM pragma_index_info('idx_send_deliveries_agent_phase') ORDER BY seqno"
        ),
        vec![
            vec![rusqlite::types::Value::Text("recipient_agent".to_string())],
            vec![rusqlite::types::Value::Text("phase".to_string())],
            vec![rusqlite::types::Value::Text("queue_rank".to_string())]
        ]
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT pk FROM pragma_table_info('send_deliveries') WHERE name='queue_rank'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(store.conn.query_row(
        "SELECT \"notnull\" FROM pragma_table_info('send_deliveries') WHERE name='delivery_id'", [],
        |row| row.get::<_, i64>(0)).unwrap(), 1);
}

#[test]
fn held159_migration_recipient_agent_rempli() {
    let (db, conn) = held159_migration_fixture();
    conn.execute(
        "UPDATE send_deliveries SET message_bytes=?1 WHERE delivery_id='held159-indeterminate'",
        params![b"illisible-held159".as_slice()],
    )
    .unwrap();
    let before = held159_migration_rows(
        &conn,
        "SELECT delivery_id,message_bytes,phase
        FROM send_deliveries WHERE delivery_id='held159-indeterminate'",
    );
    drop(conn);
    init_test_warn_logger();
    let guard = TEST_LOG_LOCK.lock().unwrap();
    TEST_WARN_LOGS.lock().unwrap().clear();
    let result = IdempotencyStore::open(db.path());
    let logs = TEST_WARN_LOGS.lock().unwrap().clone();
    drop(guard);
    let store = result.expect("un ancien corps illisible doit être conservé et signalé");
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,message_bytes,phase
        FROM send_deliveries WHERE delivery_id='held159-indeterminate'"
        ),
        before
    );
    assert_eq!(held159_migration_rows(&store.conn, "SELECT delivery_id FROM send_deliveries
        WHERE json_valid(message_bytes) AND recipient_agent IS NOT json_extract(message_bytes,'$.to')"),
        Vec::<Vec<rusqlite::types::Value>>::new());
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT recipient_agent FROM send_deliveries
        WHERE delivery_id='held159-dispatching'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "held159-agent"
    );
    assert!(
        logs.iter()
            .any(|line| line.contains("held159-indeterminate")),
        "warn doit identifier la ligne illisible ; logs={logs:?}"
    );
}

#[test]
fn held159_migration_liens_execution_conserves() {
    let (db, conn) = held159_migration_fixture();
    let before = held159_migration_rows(
        &conn,
        "SELECT delivery_id,submission_id,execution_id FROM send_delivery_execution_links ORDER BY delivery_id",
    );
    assert!(
        before.len() >= 4,
        "une fixture sans lien ne voit pas la cascade de DROP TABLE"
    );
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,submission_id,execution_id FROM send_delivery_execution_links ORDER BY delivery_id"
        ),
        before
    );
    assert_eq!(
        store
            .conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(held159_migration_rows(&store.conn, "PRAGMA foreign_key_check").is_empty());
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT MAX(version) FROM idempotency_schema_migrations",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
}

#[test]
fn held159_migration_phases_held_et_cancelled_admises() {
    let (db, conn) = held159_migration_fixture();
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        store
            .conn
            .execute(
                "UPDATE send_deliveries SET phase='held'
        WHERE delivery_id='held159-dispatching'",
                []
            )
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .conn
            .execute(
                "UPDATE send_deliveries SET phase='cancelled'
        WHERE delivery_id='held159-dispatching'",
                []
            )
            .unwrap(),
        1
    );
    assert!(
        store
            .conn
            .execute(
                "UPDATE send_deliveries SET phase='phase-inconnue159'
        WHERE delivery_id='held159-dispatching'",
                []
            )
            .is_err(),
        "le vocabulaire reste fermé"
    );
}

#[test]
fn held159_migration_turn_gate_defaut_zero() {
    let (db, conn) = held159_migration_fixture();
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries
        WHERE turn_gate IS NOT 0",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT dflt_value FROM pragma_table_info('send_deliveries')
        WHERE name='turn_gate'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "0"
    );
    assert!(
        store
            .conn
            .execute(
                "UPDATE send_deliveries SET turn_gate=2
        WHERE delivery_id='held159-dispatching'",
                []
            )
            .is_err()
    );
}

#[test]
fn held159_migration_rejeu_sans_effet() {
    let (db, conn) = held159_migration_fixture();
    drop(conn);
    let store = IdempotencyStore::open(db.path()).unwrap();
    let deliveries = held159_migration_rows(
        &store.conn,
        "SELECT * FROM send_deliveries ORDER BY queue_rank",
    );
    let links = held159_migration_rows(
        &store.conn,
        "SELECT * FROM send_delivery_execution_links ORDER BY delivery_id",
    );
    let migrations = held159_migration_rows(
        &store.conn,
        "SELECT version FROM idempotency_schema_migrations ORDER BY version",
    );
    let sequence = held159_migration_rows(
        &store.conn,
        "SELECT name,seq FROM sqlite_sequence WHERE name='send_deliveries'",
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM idempotency_schema_migrations
        WHERE version=7",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    drop(store);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT * FROM send_deliveries ORDER BY queue_rank"
        ),
        deliveries
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT * FROM send_delivery_execution_links ORDER BY delivery_id"
        ),
        links
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT version FROM idempotency_schema_migrations ORDER BY version"
        ),
        migrations
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT name,seq FROM sqlite_sequence WHERE name='send_deliveries'"
        ),
        sequence
    );
}

#[test]
fn held159_migration_rollback_si_echec() {
    let (db, mut conn) = held159_migration_fixture();
    conn.execute_batch(
        "CREATE TRIGGER held159_abort_migration
        BEFORE INSERT ON idempotency_schema_migrations WHEN NEW.version=7
        BEGIN SELECT RAISE(ABORT,'held159-migration-forced'); END;",
    )
    .unwrap();
    let schema = held159_migration_rows(
        &conn,
        "SELECT type,name,tbl_name,sql FROM sqlite_master
        WHERE sql IS NOT NULL ORDER BY type,name",
    );
    let deliveries =
        held159_migration_rows(&conn, "SELECT rowid,* FROM send_deliveries ORDER BY rowid");
    let links = held159_migration_rows(
        &conn,
        "SELECT * FROM send_delivery_execution_links ORDER BY delivery_id",
    );
    let error = IdempotencyStore::init_schema(&mut conn)
        .expect_err("l’échec du reçuv7 doit annuler TOUTE la migration");
    assert!(
        error.to_string().contains("held159-migration-forced"),
        "échec SQL injecté réel : {error}"
    );
    assert_eq!(
        held159_migration_rows(
            &conn,
            "SELECT type,name,tbl_name,sql FROM sqlite_master
        WHERE sql IS NOT NULL ORDER BY type,name"
        ),
        schema
    );
    assert_eq!(
        held159_migration_rows(&conn, "SELECT rowid,* FROM send_deliveries ORDER BY rowid"),
        deliveries
    );
    assert_eq!(
        held159_migration_rows(
            &conn,
            "SELECT * FROM send_delivery_execution_links ORDER BY delivery_id"
        ),
        links
    );
    assert_eq!(
        conn.query_row(
            "SELECT MAX(version) FROM idempotency_schema_migrations",
            [],
            |row| row.get::<_, i64>(0)
        )
        .unwrap(),
        6
    );
    assert_eq!(
        conn.query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(held159_migration_rows(&conn, "PRAGMA foreign_key_check").is_empty());
    drop(conn);
    drop(db);
}

#[test]
fn held159_migration_fantomes_ecartes_avec_warn() {
    let (db, conn) = held159_migration_fixture();
    // Simulation EXTERNE comme le témoinv4 existant : FK OFF sur cette seule copie,
    // avant ouverture. La migration doit ensuite travailler avec FK ON.
    conn.pragma_update(None, "foreign_keys", false).unwrap();
    conn.execute(
        "INSERT INTO send_deliveries
        (delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,
         delivery_generation,phase,expires_at,message_bytes)
        VALUES ('held159-ghost','159_scope_absent_aaaa','send','ghost','held159-instance',1,
         'dispatching',?1,?2)",
        params![
            NOW + HORIZON,
            sample_message_bytes("ghost", "held159-agent")
        ],
    )
    .unwrap();
    let valid = held159_migration_rows(
        &conn,
        "SELECT delivery_id FROM send_deliveries WHERE delivery_id!='held159-ghost' ORDER BY delivery_id",
    );
    drop(conn);
    init_test_warn_logger();
    let guard = TEST_LOG_LOCK.lock().unwrap();
    TEST_WARN_LOGS.lock().unwrap().clear();
    let result = IdempotencyStore::open(db.path());
    let logs = TEST_WARN_LOGS.lock().unwrap().clone();
    drop(guard);
    let store = result.expect(
        "un fantôme externe ne doit pas bloquer la migration ni supprimer les enfants valides",
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries
        WHERE delivery_id='held159-ghost'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id FROM send_deliveries ORDER BY delivery_id"
        ),
        valid
    );
    assert!(held159_migration_rows(&store.conn, "PRAGMA foreign_key_check").is_empty());
    assert_eq!(
        store
            .conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(
        logs.iter().any(|line| line.contains("held159-ghost")),
        "le nettoyage doit être dit et identifier le fantôme ; logs={logs:?}"
    );
}

// T014 : mêmes cinq oracles que RED ; réouverture v7 puis API SQL réelle.
#[test]
fn edit159_receipt_store_reserver_puis_rejouer_meme_resultat() {
    use bridget_transport::protocol::{QueueEditAction, QueueEditState};
    let db = TempDbGuard::new("edit159-old-v7");
    let old_user_version;
    {
        let old = IdempotencyStore::open(db.path()).unwrap();
        assert_eq!(
            old.conn
                .query_row(
                    "SELECT MAX(version) FROM idempotency_schema_migrations",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            7
        );
        old_user_version = old
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap();
        old.conn
            .execute_batch("DROP TABLE IF EXISTS queue_edits")
            .unwrap();
        assert!(
            !old.conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='queue_edits')",
                    [],
                    |r| r.get::<_, bool>(0)
                )
                .unwrap()
        );
    }
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT MAX(version) FROM idempotency_schema_migrations",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        7
    );
    assert_eq!(
        store
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        old_user_version
    );
    assert_eq!(store.conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name IN ('idx_queue_edits_expires','idx_queue_edits_delivery')", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    let (first,effect,_notice)=store.queue_edit_once("012_scope_aaaaaaaaaaaa", "edit-1", b"cancel,target,public-message,null", QueueEditAction::Cancel, "author", NOW,
        |tx| {
            tx.execute("INSERT INTO ledger(id,ts,sender,target,body,conversation_key) VALUES('effect-1',?1,'author','target','effect','test')", params![NOW])?;
            Ok((Some("internal-delivery-different".to_string()), None, Ok("committed")))
        }).unwrap();
    assert_eq!(first.state, QueueEditState::Applied);
    assert_eq!(effect, Some("committed"));
    drop(store);
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let (replayed, effect, _notice) = store
        .queue_edit_once::<()>(
            "012_scope_aaaaaaaaaaaa",
            "edit-1",
            b"cancel,target,public-message,null",
            QueueEditAction::Cancel,
            "author",
            NOW + 1,
            |_| panic!("un rejeu ne réapplique aucun effet"),
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(effect, None);
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM ledger WHERE id='effect-1'", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT delivery_id FROM queue_edits WHERE request_id='edit-1'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "internal-delivery-different"
    );
}

#[test]
fn edit159_receipt_store_autres_parametres_envelope_mismatch() {
    use bridget_transport::protocol::{QueueEditAction, QueueEditState};
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let canonical = b"cancel,target,public-message,null";
    let (first, _, _notice) = store
        .queue_edit_once(
            "012_scope_aaaaaaaaaaaa",
            "edit-2",
            canonical,
            QueueEditAction::Cancel,
            "author",
            NOW,
            |_| Ok((Some("internal-2".to_string()), None, Ok(()))),
        )
        .unwrap();
    for changed in [
        b"cancel,other-target,public-message,null".as_slice(),
        b"cancel,target,other-message,null".as_slice(),
        b"cancel,target,public-message,before".as_slice(),
    ] {
        let (receipt, effect, _notice) = store
            .queue_edit_once::<()>(
                "012_scope_aaaaaaaaaaaa",
                "edit-2",
                changed,
                QueueEditAction::Cancel,
                "author",
                NOW,
                |_| panic!("paramètres différents : aucun effet"),
            )
            .unwrap();
        assert_eq!(receipt.state, QueueEditState::Refused);
        assert_eq!(receipt.refusal.as_deref(), Some("envelope_mismatch"));
        assert_eq!(effect, None);
    }
    let (receipt, _, _notice) = store
        .queue_edit_once::<()>(
            "012_scope_aaaaaaaaaaaa",
            "edit-2",
            b"promote,target,public-message,null",
            QueueEditAction::Promote,
            "author",
            NOW,
            |_| panic!("action différente : aucun effet"),
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("envelope_mismatch"));
    let (replayed, _, _notice) = store
        .queue_edit_once::<()>(
            "012_scope_aaaaaaaaaaaa",
            "edit-2",
            canonical,
            QueueEditAction::Cancel,
            "author",
            NOW,
            |_| panic!("ancien reçu préservé"),
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn edit159_receipt_store_refus_memorise() {
    use bridget_transport::protocol::{QueueEditAction, QueueEditState};
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let (first,effect,_notice)=store.queue_edit_once::<()>("012_scope_aaaaaaaaaaaa", "edit-3", b"promote,target,unknown,null", QueueEditAction::Promote, "parent", NOW,
        |tx| {
            tx.execute("INSERT INTO ledger(id,ts,sender,target,body,conversation_key) VALUES('refused-effect',?1,'parent','target','must rollback','test')", params![NOW])?;
            Ok((None, None, Err(("not_held".to_string(),None))))
        }).unwrap();
    assert_eq!(first.state, QueueEditState::Refused);
    assert_eq!(first.refusal.as_deref(), Some("not_held"));
    assert_eq!(first.effect, None);
    assert_eq!(effect, None);
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE id='refused-effect'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT delivery_id FROM queue_edits WHERE request_id='edit-3'",
                [],
                |r| r.get::<_, Option<String>>(0)
            )
            .unwrap(),
        None
    );
    let (replayed, effect, _notice) = store
        .queue_edit_once::<()>(
            "012_scope_aaaaaaaaaaaa",
            "edit-3",
            b"promote,target,unknown,null",
            QueueEditAction::Promote,
            "parent",
            NOW + 1,
            |_| panic!("phase/pause nouvelles ne réévaluent pas le refus reçu"),
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(effect, None);
}

#[test]
fn edit159_receipt_store_effet_et_recu_atomiques() {
    use bridget_transport::protocol::{QueueEditAction, QueueEditState};
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    store.conn.execute_batch("CREATE TEMP TRIGGER edit159_receipt_failure BEFORE INSERT ON queue_edits BEGIN SELECT RAISE(ABORT,'receipt failure'); END;").unwrap();
    let result = store.queue_edit_once("012_scope_aaaaaaaaaaaa", "edit-4", b"cancel,target,message,null", QueueEditAction::Cancel, "author", NOW,
        |tx| {
            tx.execute("INSERT INTO ledger(id,ts,sender,target,body,conversation_key) VALUES('atomic-effect',?1,'author','target','effect','test')", params![NOW])?;
            Ok((Some("internal-4".to_string()), None, Ok(())))
        });
    assert!(matches!(result, Err(IdempotencyError::Sqlite(_))));
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE id='atomic-effect'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
    store
        .conn
        .execute_batch("DROP TRIGGER edit159_receipt_failure")
        .unwrap();
    let (receipt, effect, _notice) = store
        .queue_edit_once(
            "012_scope_aaaaaaaaaaaa",
            "edit-4",
            b"cancel,target,message,null",
            QueueEditAction::Cancel,
            "author",
            NOW,
            |_| Ok((Some("internal-4".to_string()), None, Ok(()))),
        )
        .unwrap();
    assert_eq!(receipt.state, QueueEditState::Applied);
    assert_eq!(effect, Some(()));
}

#[test]
fn edit159_receipt_store_purge_apres_horizon_seulement() {
    use bridget_transport::protocol::QueueEditAction;
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for request_id in ["edit-5a", "edit-5b"] {
        store
            .queue_edit_once(
                "012_scope_aaaaaaaaaaaa",
                request_id,
                b"cancel,target,message,null",
                QueueEditAction::Cancel,
                "author",
                NOW,
                |_| Ok((Some("internal-5".to_string()), None, Ok(()))),
            )
            .unwrap();
    }
    let expires_at: i64 = store
        .conn
        .query_row("SELECT MIN(expires_at) FROM queue_edits", [], |r| r.get(0))
        .unwrap();
    assert_eq!(expires_at, NOW + 7 * 86_400);
    assert_eq!(store.purge_expired(expires_at - 1).unwrap(), 0);
    assert_eq!(store.purge_expired(expires_at).unwrap(), 0);
    assert_eq!(store.purge_expired(expires_at + 1).unwrap(), 2);
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

// T017 Store : sept noms prescrits ; vrais reserve/begin Held, IDs distincts.
#[test]
fn edit159_cancel_phase_cancelled_et_envoi_rejete_cancelled_in_queue() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let bytes = sample_message_bytes("public-cancel", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-cancel".into(),
        recipient_instance_id: "instance-cancel".into(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: bytes,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let (receipt, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(delivery_phase(&store, "delivery-cancel"), "cancelled");
    assert_eq!(
        notice.as_ref().map(|n| n.message_id.as_str()),
        Some("public-cancel")
    );
    let terminal: (String,String) = store.conn.query_row("SELECT public_result_kind, public_result_category FROM idempotency_records WHERE issuer_scope=?1 AND idempotency_key=?2", params![key().issuer_scope,key().idempotency_key], |r| Ok((r.get(0)?,r.get(1)?))).unwrap();
    assert_eq!(
        terminal,
        ("rejected".to_string(), "cancelled_in_queue".to_string())
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT delivery_id FROM queue_edits WHERE request_id='cancel-edit'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "delivery-cancel"
    );
}

#[test]
fn edit159_cancel_prevenance_auteur() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let bytes = sample_message_bytes("public-cancel", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-cancel".into(),
        recipient_instance_id: "instance-cancel".into(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: bytes,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let (receipt, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(notice.unwrap().sender, "sender-peer");
    let durable: (String,String,Option<i64>) = store.conn.query_row("SELECT sender,body,notified_at FROM orphan_emitter_notices WHERE delivery_id='delivery-cancel'",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(durable.0, "sender-peer");
    assert!(durable.1.contains("public-cancel") && durable.1.contains("cancelled_in_queue"));
    assert_eq!(durable.2, None);
}

#[test]
fn edit159_cancel_une_seule_transaction() {
    for (operation, table, filter) in [
        (
            "UPDATE",
            "send_deliveries",
            "NEW.delivery_id='delivery-cancel'",
        ),
        (
            "UPDATE",
            "idempotency_records",
            "NEW.idempotency_key='message-1'",
        ),
        ("UPDATE", "tracked_requests", "NEW.id='public-cancel'"),
        (
            "INSERT",
            "orphan_emitter_notices",
            "NEW.delivery_id='delivery-cancel'",
        ),
        ("INSERT", "queue_edits", "NEW.request_id='cancel-edit'"),
    ] {
        let db = TempDbGuard::new("edit159-cancel-atomic");
        drop(crate::store::Store::open(db.path()).unwrap());
        let mut store = IdempotencyStore::open(db.path()).unwrap();
        let mut message: bridget_core::BridgetMessage =
            serde_json::from_slice(&sample_message_bytes("public-cancel", "agent-2")).unwrap();
        message.reply = true;
        let bytes = serde_json::to_vec(&message).unwrap();
        reserve(&store, &bytes);
        let tracking = ReplyTracking {
            request_id: message.id.clone(),
            sender: message.from.clone(),
            target: message.to.clone(),
            created_at: NOW,
            deadline_at: NOW + HORIZON,
        };
        let delivery = SendDelivery {
            delivery_id: "delivery-cancel".into(),
            recipient_instance_id: "instance-cancel".into(),
            delivery_generation: 3,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key(),
                &delivery,
                Some(&tracking),
                None,
                None,
                Some(8),
                Some("agent-2"),
            )
            .unwrap();
        store.conn.execute_batch(&format!("CREATE TEMP TRIGGER cancel159_failure AFTER {operation} ON {table} WHEN {filter} BEGIN SELECT RAISE(ABORT,'cancel159 failure after write'); END;")).unwrap();
        assert!(
            matches!(
                store.apply_cancel(
                    "012_scope_aaaaaaaaaaaa",
                    "cancel-edit",
                    b"cancel,agent-2,public-cancel,null",
                    "sender-peer",
                    "agent-2",
                    "public-cancel",
                    NOW
                ),
                Err(IdempotencyError::Sqlite(_))
            ),
            "fault on {table}"
        );
        assert_eq!(
            delivery_phase(&store, "delivery-cancel"),
            "held",
            "fault on {table}"
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT state FROM idempotency_records WHERE idempotency_key='message-1'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "dispatching"
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT state FROM tracked_requests WHERE id='public-cancel'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "open"
        );
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM orphan_emitter_notices", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn edit159_cancel_not_held_pour_message_libere() {
    let db = TempDbGuard::new("edit159-cancel-released");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-cancel", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-cancel".into(),
        recipient_instance_id: "instance-cancel".into(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: bytes,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    assert!(store.release_next_held("agent-2", NOW).unwrap().0.is_some());
    let (receipt, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Refused
    );
    assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
    assert_eq!(notice, None);
    assert_eq!(delivery_phase(&store, "delivery-cancel"), "dispatching");
}

#[test]
fn edit159_cancel_not_held_pour_message_jamais_retenu() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let (receipt, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Refused
    );
    assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
    assert_eq!(notice, None);
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT delivery_id FROM queue_edits WHERE request_id='cancel-edit'",
                [],
                |r| r.get::<_, Option<String>>(0)
            )
            .unwrap(),
        None
    );
}

#[test]
fn edit159_cancel_rejeu_meme_resultat() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let bytes = sample_message_bytes("public-cancel", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-cancel".into(),
        recipient_instance_id: "instance-cancel".into(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: bytes,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let (first, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        first.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert!(notice.is_some());
    let (replayed, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(notice, None, "pas de cleanup/notification une seconde fois");
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM orphan_emitter_notices", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn edit159_cancel_autres_parametres_envelope_mismatch() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let bytes = sample_message_bytes("public-cancel", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-cancel".into(),
        recipient_instance_id: "instance-cancel".into(),
        delivery_generation: 3,
        expires_at: NOW + HORIZON,
        message_bytes: bytes,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let (first, _) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(
        first.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    let (mismatch, notice) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,other-target,public-cancel,null",
            "sender-peer",
            "other-target",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(mismatch.refusal.as_deref(), Some("envelope_mismatch"));
    assert_eq!(notice, None);
    let (replayed, _) = store
        .apply_cancel(
            "012_scope_aaaaaaaaaaaa",
            "cancel-edit",
            b"cancel,agent-2,public-cancel,null",
            "sender-peer",
            "agent-2",
            "public-cancel",
            NOW,
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(delivery_phase(&store, "delivery-cancel"), "cancelled");
}

// T019 Store : dix noms planifiés ; droits/pause et libération réservés au daemon.
#[test]
fn edit159_reorder_avant_un_message() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let before = held159_migration_rows(
        &store.conn,
        "SELECT delivery_id,message_bytes,delivery_generation FROM send_deliveries ORDER BY delivery_id",
    );
    let (receipt, _) = store
        .apply_reorder(
            "012_scope_aaaaaaaaaaaa",
            "reorder-edit",
            b"reorder,agent-2,public-B,public-A",
            "parent",
            "agent-2",
            "public-B",
            Some("public-A"),
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-A","delivery-C"]);
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,message_bytes,delivery_generation FROM send_deliveries ORDER BY delivery_id"
        ),
        before
    );
}

#[test]
fn edit159_reorder_a_la_fin() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let (receipt, _) = store
        .apply_reorder(
            "012_scope_aaaaaaaaaaaa",
            "reorder-end",
            b"reorder,agent-2,public-A,null",
            "parent",
            "agent-2",
            "public-A",
            None,
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-C","delivery-A"]);
}

#[test]
fn edit159_reorder_message_libere_not_held() {
    let db = TempDbGuard::new("edit159-reorder-released");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    assert!(store.release_next_held("agent-2", NOW).unwrap().0.is_some());
    let (receipt, effect) = store
        .apply_reorder(
            "012_scope_aaaaaaaaaaaa",
            "reorder-released",
            b"reorder,agent-2,public-A,public-B",
            "parent",
            "agent-2",
            "public-A",
            Some("public-B"),
            NOW,
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
    assert_eq!(effect, None);
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-C"]);
}

#[test]
fn edit159_reorder_before_inconnu_ou_autre_destinataire_not_held() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let other_key =
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "key-D").unwrap();
    let bytes = sample_message_bytes("public-D", "agent-other");
    store
        .reserve(&other_key, &bytes, NOW, HORIZON, NOW, 30)
        .unwrap();
    store
        .begin_send_delivery_with_guidance(
            &other_key,
            &SendDelivery {
                delivery_id: "delivery-D".into(),
                recipient_instance_id: "instance-other".into(),
                delivery_generation: 1,
                expires_at: NOW + HORIZON,
                message_bytes: bytes,
            },
            None,
            None,
            None,
            Some(16),
            Some("agent-other"),
        )
        .unwrap();
    let before = held159_migration_rows(
        &store.conn,
        "SELECT delivery_id,queue_rank FROM send_deliveries ORDER BY delivery_id",
    );
    for before_message in ["unknown", "public-D"] {
        let request_id = format!("bad-before-{before_message}");
        let canonical = format!("reorder,agent-2,public-A,{before_message}");
        let (receipt, effect) = store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                &request_id,
                canonical.as_bytes(),
                "parent",
                "agent-2",
                "public-A",
                Some(before_message),
                NOW,
            )
            .unwrap();
        assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
        assert_eq!(effect, None);
    }
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,queue_rank FROM send_deliveries ORDER BY delivery_id"
        ),
        before
    );
}

#[test]
fn edit159_reorder_arrivee_concurrente_deplacement_valide_d_abord_donne_B_A_C() {
    let db = TempDbGuard::new("edit159-two-writers");
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    for label in ["A", "B"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        store
            .begin_send_delivery_with_guidance(
                &key,
                &SendDelivery {
                    delivery_id: format!("delivery-{label}"),
                    recipient_instance_id: "instance-reorder".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: bytes,
                },
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let ckey = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "key-C").unwrap();
    let cbytes = sample_message_bytes("public-C", "agent-2");
    store
        .reserve(&ckey, &cbytes, NOW, HORIZON, NOW, 30)
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let (committed_tx, committed_rx) = std::sync::mpsc::channel();
    let move_path = db.path().to_path_buf();
    let arrival_path = move_path.clone();
    let move_barrier = barrier.clone();
    let movement = thread::spawn(move || {
        let mut store = IdempotencyStore::open(&move_path).unwrap();
        move_barrier.wait();
        let (receipt, _) = store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "reorder-race",
                b"reorder,agent-2,public-A,null",
                "parent",
                "agent-2",
                "public-A",
                None,
                NOW,
            )
            .unwrap();
        assert_eq!(
            receipt.state,
            bridget_transport::protocol::QueueEditState::Applied
        );
        committed_tx.send(()).unwrap();
    });
    let arrival = thread::spawn(move || {
        let mut store = IdempotencyStore::open(&arrival_path).unwrap();
        barrier.wait();
        committed_rx.recv().unwrap();
        store
            .begin_send_delivery_with_guidance(
                &ckey,
                &SendDelivery {
                    delivery_id: "delivery-C".into(),
                    recipient_instance_id: "instance-reorder".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: cbytes,
                },
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    });
    movement.join().unwrap();
    arrival.join().unwrap();
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-A","delivery-C"]);
}

#[test]
fn edit159_reorder_arrivee_concurrente_arrivee_valide_d_abord_donne_B_C_A() {
    let db = TempDbGuard::new("edit159-two-writers");
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    for label in ["A", "B"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        store
            .begin_send_delivery_with_guidance(
                &key,
                &SendDelivery {
                    delivery_id: format!("delivery-{label}"),
                    recipient_instance_id: "instance-reorder".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: bytes,
                },
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let ckey = IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "key-C").unwrap();
    let cbytes = sample_message_bytes("public-C", "agent-2");
    store
        .reserve(&ckey, &cbytes, NOW, HORIZON, NOW, 30)
        .unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let (committed_tx, committed_rx) = std::sync::mpsc::channel();
    let move_path = db.path().to_path_buf();
    let arrival_path = move_path.clone();
    let move_barrier = barrier.clone();
    let movement = thread::spawn(move || {
        let mut store = IdempotencyStore::open(&move_path).unwrap();
        move_barrier.wait();
        committed_rx.recv().unwrap();
        let (receipt, _) = store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "reorder-race",
                b"reorder,agent-2,public-A,null",
                "parent",
                "agent-2",
                "public-A",
                None,
                NOW,
            )
            .unwrap();
        assert_eq!(
            receipt.state,
            bridget_transport::protocol::QueueEditState::Applied
        );
    });
    let arrival = thread::spawn(move || {
        let mut store = IdempotencyStore::open(&arrival_path).unwrap();
        barrier.wait();
        store
            .begin_send_delivery_with_guidance(
                &ckey,
                &SendDelivery {
                    delivery_id: "delivery-C".into(),
                    recipient_instance_id: "instance-reorder".into(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: cbytes,
                },
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
        committed_tx.send(()).unwrap();
    });
    movement.join().unwrap();
    arrival.join().unwrap();
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-C","delivery-A"]);
}

#[test]
fn edit159_reorder_ordre_persistant_egal_ordre_de_validation() {
    let db = TempDbGuard::new("edit159-order-persistent");
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    assert_eq!(
        store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "reorder-edit",
                b"reorder,agent-2,public-B,public-A",
                "parent",
                "agent-2",
                "public-B",
                Some("public-A"),
                NOW
            )
            .unwrap()
            .0
            .state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(
        store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "second",
                b"reorder,agent-2,public-C,public-B",
                "parent",
                "agent-2",
                "public-C",
                Some("public-B"),
                NOW + 1
            )
            .unwrap()
            .0
            .state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    drop(store);
    let store = IdempotencyStore::open(db.path()).unwrap();
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-C","delivery-B","delivery-A"]);
}

#[test]
fn edit159_reorder_survit_a_la_reaffectation() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    assert_eq!(
        store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "reorder-edit",
                b"reorder,agent-2,public-B,public-A",
                "parent",
                "agent-2",
                "public-B",
                Some("public-A"),
                NOW
            )
            .unwrap()
            .0
            .state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    let before = held159_migration_rows(
        &store.conn,
        "SELECT delivery_id,queue_rank,message_bytes,delivery_generation FROM send_deliveries ORDER BY queue_rank",
    );
    assert_eq!(
        store
            .adopt_held_for_agent("agent-2", "new-instance-reorder", false)
            .unwrap(),
        3
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,queue_rank,message_bytes,delivery_generation FROM send_deliveries ORDER BY queue_rank"
        ),
        before
    );
    assert_eq!(store.conn.prepare("SELECT delivery_id FROM send_deliveries WHERE recipient_agent='agent-2' AND phase='held' ORDER BY queue_rank").unwrap().query_map([],|r|r.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap(),vec!["delivery-B","delivery-A","delivery-C"]);
}

#[test]
fn edit159_reorder_rangs_hors_groupe_inchanges() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let other =
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "other-rank").unwrap();
    let bytes = sample_message_bytes("other-public", "agent-other");
    store
        .reserve(&other, &bytes, NOW, HORIZON, NOW, 30)
        .unwrap();
    store
        .begin_send_delivery_with_guidance(
            &other,
            &SendDelivery {
                delivery_id: "other-delivery".into(),
                recipient_instance_id: "other-instance".into(),
                delivery_generation: 1,
                expires_at: NOW + HORIZON,
                message_bytes: bytes,
            },
            None,
            None,
            None,
            Some(16),
            Some("agent-other"),
        )
        .unwrap();
    let other_before = held159_migration_rows(
        &store.conn,
        "SELECT * FROM send_deliveries WHERE recipient_agent='agent-other'",
    );
    let ranks = held159_migration_rows(
        &store.conn,
        "SELECT queue_rank FROM send_deliveries WHERE recipient_agent='agent-2' ORDER BY queue_rank",
    );
    assert_eq!(
        store
            .apply_reorder(
                "012_scope_aaaaaaaaaaaa",
                "reorder-edit",
                b"reorder,agent-2,public-B,public-A",
                "parent",
                "agent-2",
                "public-B",
                Some("public-A"),
                NOW
            )
            .unwrap()
            .0
            .state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT * FROM send_deliveries WHERE recipient_agent='agent-other'"
        ),
        other_before
    );
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT queue_rank FROM send_deliveries WHERE recipient_agent='agent-2' ORDER BY queue_rank"
        ),
        ranks
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM send_deliveries WHERE queue_rank<0",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn edit159_reorder_rejeu_meme_resultat() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    for label in ["A", "B", "C"] {
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            format!("key-{label}"),
        )
        .unwrap();
        let bytes = sample_message_bytes(&format!("public-{label}"), "agent-2");
        store.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
        let delivery = SendDelivery {
            delivery_id: format!("delivery-{label}"),
            recipient_instance_id: "instance-reorder".into(),
            delivery_generation: 1,
            expires_at: NOW + HORIZON,
            message_bytes: bytes,
        };
        store
            .begin_send_delivery_with_guidance(
                &key,
                &delivery,
                None,
                None,
                None,
                Some(16),
                Some("agent-2"),
            )
            .unwrap();
    }
    let (first, effect) = store
        .apply_reorder(
            "012_scope_aaaaaaaaaaaa",
            "reorder-edit",
            b"reorder,agent-2,public-B,public-A",
            "parent",
            "agent-2",
            "public-B",
            Some("public-A"),
            NOW,
        )
        .unwrap();
    assert_eq!(
        first.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(effect, Some(()));
    let once = held159_migration_rows(
        &store.conn,
        "SELECT delivery_id,queue_rank FROM send_deliveries ORDER BY queue_rank",
    );
    let (replay, effect) = store
        .apply_reorder(
            "012_scope_aaaaaaaaaaaa",
            "reorder-edit",
            b"reorder,agent-2,public-B,public-A",
            "parent",
            "agent-2",
            "public-B",
            Some("public-A"),
            NOW,
        )
        .unwrap();
    assert_eq!(replay, first);
    assert_eq!(effect, None);
    assert_eq!(
        held159_migration_rows(
            &store.conn,
            "SELECT delivery_id,queue_rank FROM send_deliveries ORDER BY queue_rank"
        ),
        once
    );
}

// T021 Store : sept noms planifiés, même renderer que les handlers ; aucun banc neuf.
#[test]
fn edit159_promote_demande_close_juste_avant_ne_promeut_pas() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let mut message: bridget_core::BridgetMessage =
        serde_json::from_slice(&sample_message_bytes("public-promote", "agent-2")).unwrap();
    message.reply = true;
    let bytes = serde_json::to_vec(&message).unwrap();
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    let tracking = ReplyTracking {
        request_id: "public-promote".into(),
        sender: "sender-peer".into(),
        target: "agent-2".into(),
        created_at: NOW,
        deadline_at: NOW + HORIZON,
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            Some(&tracking),
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    store
        .conn
        .execute(
            "UPDATE tracked_requests SET state='answered' WHERE id='public-promote'",
            [],
        )
        .unwrap();
    let (receipt, effect, notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
    assert_eq!(effect, None);
    assert_eq!(delivery_phase(&store, "delivery-promote"), "cancelled");
    assert_eq!(
        notice.as_ref().map(|n| n.message_id.as_str()),
        Some("public-promote")
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT turn_gate FROM send_deliveries WHERE delivery_id='delivery-promote'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
}

#[test]
fn edit159_promote_echeance_juste_avant_ne_promeut_pas() {
    // Deux montages indépendants : échéance demande et horizon saga/canon.
    for deadline in ["request", "saga"] {
        let db = TempDbGuard::new("edit159-promote-store");
        drop(crate::store::Store::open(db.path()).unwrap());
        drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
        let mut store = IdempotencyStore::open(db.path()).unwrap();
        let mut message: bridget_core::BridgetMessage =
            serde_json::from_slice(&sample_message_bytes("public-promote", "agent-2")).unwrap();
        message.reply = true;
        let bytes = serde_json::to_vec(&message).unwrap();
        reserve(&store, &bytes);
        let delivery = SendDelivery {
            delivery_id: "delivery-promote".into(),
            recipient_instance_id: "instance-promote".into(),
            delivery_generation: 7,
            expires_at: NOW + HORIZON,
            message_bytes: bytes.clone(),
        };
        let tracking = ReplyTracking {
            request_id: "public-promote".into(),
            sender: "sender-peer".into(),
            target: "agent-2".into(),
            created_at: NOW,
            deadline_at: NOW + HORIZON,
        };
        store
            .begin_send_delivery_with_guidance(
                &key(),
                &delivery,
                Some(&tracking),
                None,
                None,
                Some(8),
                Some("agent-2"),
            )
            .unwrap();
        let facts = crate::guidance::GuidanceFacts {
            action: crate::guidance::GuidanceAction::QueuePromote,
            path: crate::guidance::GuidancePath::SendIdempotent,
            actor: crate::guidance::GuidanceActor::Agent {
                id: "parent".into(),
            },
            identity_attested: true,
            trusted_origin: None,
            child_proof: false,
            actor_is_target: false,
            actor_is_parent: true,
            actor_is_child: false,
            actor_is_author: false,
            message_editable: true,
            recipient_turn_active: true,
            carrier_queue_fallback: true,
            cross_project: false,
            paused: false,
            target_has_active_native_task: false,
            recent_interrupts_pair: 0,
            recent_interrupts_target: 0,
            now: NOW,
            last_interrupt_pair_at: None,
            third_latest_interrupt_target_at: None,
        };
        if deadline == "request" {
            store
                .conn
                .execute(
                    "UPDATE tracked_requests SET deadline_at=?1 WHERE id='public-promote'",
                    [NOW],
                )
                .unwrap();
        } else {
            store.conn.execute("UPDATE idempotency_records SET expires_at=?1 WHERE idempotency_key='message-1'",[NOW]).unwrap();
        }
        let (receipt, effect, notice) = store
            .apply_promote(
                "012_scope_aaaaaaaaaaaa",
                "promote-edit",
                b"promote,agent-2,public-promote,null",
                "parent",
                "agent-2",
                "public-promote",
                &facts,
                crate::daemon::queue_edit_refusal_code,
                NOW,
            )
            .unwrap();
        assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
        assert_eq!(effect, None);
        assert_eq!(delivery_phase(&store, "delivery-promote"), "cancelled");
        assert_eq!(
            notice.as_ref().map(|n| n.message_id.as_str()),
            Some("public-promote")
        );
    }
}

#[test]
fn edit159_promote_pose_la_reservation_de_tour() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-promote", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    let (receipt, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    assert_eq!(
        receipt.effect,
        Some(bridget_transport::protocol::QueueEditEffect::Unconfirmed)
    );
    assert_eq!(effect, Some(delivery.clone()));
    assert_eq!(delivery_phase(&store, "delivery-promote"), "dispatching");
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT turn_gate FROM send_deliveries WHERE delivery_id='delivery-promote'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT message_bytes FROM send_deliveries WHERE delivery_id='delivery-promote'",
                [],
                |r| r.get::<_, Vec<u8>>(0)
            )
            .unwrap(),
        bytes
    );
    let next_key =
        IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, "next-gate").unwrap();
    let next_bytes = sample_message_bytes("next-public", "agent-2");
    store
        .reserve(&next_key, &next_bytes, NOW, HORIZON, NOW, 30)
        .unwrap();
    store
        .begin_send_delivery_with_guidance(
            &next_key,
            &SendDelivery {
                delivery_id: "next-internal".into(),
                recipient_instance_id: "instance-promote".into(),
                delivery_generation: 8,
                expires_at: NOW + HORIZON,
                message_bytes: next_bytes,
            },
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let (refused, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "next-edit",
            b"promote,agent-2,next-public,null",
            "parent",
            "agent-2",
            "next-public",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(refused.refusal.as_deref(), Some("not_held"));
    assert_eq!(effect, None);
    assert_eq!(delivery_phase(&store, "next-internal"), "held");
}

#[test]
fn edit159_promote_rejeu_meme_resultat() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-promote", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    let (first, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert!(effect.is_some());
    let mut later = facts.clone();
    later.recipient_turn_active = false;
    later.paused = true;
    later.carrier_queue_fallback = false;
    store
        .conn
        .execute("UPDATE control_state SET paused=1 WHERE id=1", [])
        .unwrap();
    let (replayed, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &later,
            crate::daemon::queue_edit_refusal_code,
            NOW + 1,
        )
        .unwrap();
    assert_eq!(replayed, first);
    assert_eq!(effect, None);
    assert_eq!(
        store
            .conn
            .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn edit159_promote_panne_apres_transaction_relivre_en_file_ordinaire() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-promote", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    let (receipt, _, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(
        receipt.state,
        bridget_transport::protocol::QueueEditState::Applied
    );
    drop(store);
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let resumed = store
        .dispatching_deliveries_for_instance("instance-promote", NOW + 1)
        .unwrap();
    assert_eq!(resumed, vec![delivery]);
    let original: bridget_core::BridgetMessage =
        serde_json::from_slice(&resumed[0].message_bytes).unwrap();
    assert_eq!(
        original.intent, None,
        "pas de marqueur/guidage ajouté aux octets durables"
    );
}

#[test]
fn edit159_promote_not_held_pour_message_libere() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-promote", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    assert!(store.release_next_held("agent-2", NOW).unwrap().0.is_some());
    let (receipt, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("not_held"));
    assert_eq!(effect, None);
    assert_eq!(delivery_phase(&store, "delivery-promote"), "dispatching");
}

#[test]
fn edit159_promote_pause_agent_refuse() {
    let db = TempDbGuard::new("edit159-promote-store");
    drop(crate::store::Store::open(db.path()).unwrap());
    drop(crate::execution_store::ExecutionStore::open(db.path()).unwrap());
    let mut store = IdempotencyStore::open(db.path()).unwrap();
    let bytes = sample_message_bytes("public-promote", "agent-2");
    reserve(&store, &bytes);
    let delivery = SendDelivery {
        delivery_id: "delivery-promote".into(),
        recipient_instance_id: "instance-promote".into(),
        delivery_generation: 7,
        expires_at: NOW + HORIZON,
        message_bytes: bytes.clone(),
    };
    store
        .begin_send_delivery_with_guidance(
            &key(),
            &delivery,
            None,
            None,
            None,
            Some(8),
            Some("agent-2"),
        )
        .unwrap();
    let facts = crate::guidance::GuidanceFacts {
        action: crate::guidance::GuidanceAction::QueuePromote,
        path: crate::guidance::GuidancePath::SendIdempotent,
        actor: crate::guidance::GuidanceActor::Agent {
            id: "parent".into(),
        },
        identity_attested: true,
        trusted_origin: None,
        child_proof: false,
        actor_is_target: false,
        actor_is_parent: true,
        actor_is_child: false,
        actor_is_author: false,
        message_editable: true,
        recipient_turn_active: true,
        carrier_queue_fallback: true,
        cross_project: false,
        paused: false,
        target_has_active_native_task: false,
        recent_interrupts_pair: 0,
        recent_interrupts_target: 0,
        now: NOW,
        last_interrupt_pair_at: None,
        third_latest_interrupt_target_at: None,
    };
    store
        .conn
        .execute("UPDATE control_state SET paused=1 WHERE id=1", [])
        .unwrap();
    let (receipt, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-edit",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("referent_paused"));
    assert_eq!(effect, None);
    assert_eq!(delivery_phase(&store, "delivery-promote"), "held");
    // Même oracle, même autorité : priorité de reprise même après pause globale levée.
    store
        .conn
        .execute("UPDATE control_state SET paused=0 WHERE id=1", [])
        .unwrap();
    let executions = crate::execution_store::ExecutionStore::open(db.path()).unwrap();
    let interrupted =
        bridget_core::BridgetMessage::new("parent", "agent-2", "travail interrompu synthétique");
    assert!(
        executions
            .admit_starting_message(&interrupted, "pause-test", NOW)
            .unwrap()
    );
    executions
        .record_pause_interruption("pause-test", "agent-2", 1, NOW)
        .unwrap();
    drop(executions);
    assert!(held159_migration_rows(&store.conn, "PRAGMA foreign_key_check").is_empty());
    let (receipt, effect, _notice) = store
        .apply_promote(
            "012_scope_aaaaaaaaaaaa",
            "promote-pending",
            b"promote,agent-2,public-promote,null",
            "parent",
            "agent-2",
            "public-promote",
            &facts,
            crate::daemon::queue_edit_refusal_code,
            NOW,
        )
        .unwrap();
    assert_eq!(receipt.refusal.as_deref(), Some("referent_paused"));
    assert_eq!(effect, None);
    assert_eq!(delivery_phase(&store, "delivery-promote"), "held");
}
