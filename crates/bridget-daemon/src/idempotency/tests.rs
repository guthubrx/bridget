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
