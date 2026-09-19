//! Faute SQLite injectée à la dernière écriture d'une transaction publique.
//! Complément aux SIGKILL des harnais de socket, pas un crash simulé.

use bridget_core::BridgetMessage;
use bridget_daemon::idempotency::{
    IdempotencyKey, IdempotencyStore, LookupResult, OperationKind, Reservation, SendDelivery,
};
use bridget_daemon::store::{GuichetDeposit, GuichetNext, GuichetReplyInput, GuichetResult, Store};
use bridget_transport::protocol::{GuichetOutcome, ServiceRequestOperation, ServiceRequestPayload};
use rusqlite::Connection;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;

const NOW: i64 = 1_800_000_000;
const SCOPE: &str = "089_scope_0123456789abcdef0123456789abcdef";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("b89sql-{}", uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        Self(root)
    }

    fn database(&self) -> PathBuf {
        self.0.join("store.db")
    }

    fn sql(&self) -> Connection {
        Connection::open(self.database()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn echec_evenement_annule_reply_et_answered_puis_rejoue_les_memes_octets() {
    for sender in ["guichet", "89000000-0000-4000-8000-000000000002"] {
        reply_atomic_with_sender(sender);
    }
}

fn linked_deposit() -> GuichetDeposit {
    GuichetDeposit {
        issuer_scope: SCOPE.to_string(),
        request_id: "deposit".to_string(),
        issued_at: NOW,
        from: "agent".to_string(),
        operation: ServiceRequestOperation::DeliveryReport,
        payload: ServiceRequestPayload::DeliveryReport {
            objective_id: "objective".to_string(),
            delegation_id: "delegation".to_string(),
            delivery_hash: "0".repeat(64),
            in_reply_to: "linked".to_string(),
            review_verdict: None,
        },
        // Le Store reçoit le canon validé par le daemon et ne le reconstruit pas.
        canonical_bytes: b"fixture-request-verbatim".to_vec(),
        authorization_attestation: None,
    }
}

fn reply_atomic_with_sender(sender: &str) {
    let fixture = Fixture::new();
    let mut store = Store::open(&fixture.database()).unwrap();
    store.create_request("linked", sender, "agent", 60).unwrap();
    let deposit = linked_deposit();
    store.deposit_guichet(&deposit, 600, 60, NOW).unwrap();
    let GuichetNext::Claimed(claim) = store.claim_next_guichet("owner", NOW).unwrap() else {
        panic!("claim absent");
    };
    fixture
        .sql()
        .execute_batch(
            "CREATE TRIGGER reject_event BEFORE INSERT ON guichet_lifecycle_events
         BEGIN SELECT RAISE(ABORT, 'fixture-lifecycle-write'); END;",
        )
        .unwrap();
    let reply = b"fixture-reply-verbatim";
    let input = || GuichetReplyInput {
        issuer_scope: SCOPE,
        request_id: "deposit",
        generation: claim.claim_generation,
        token: &claim.claim_token,
        response_message_id: "response",
        reply_bytes: reply,
        in_reply_to: "linked",
        outcome: GuichetOutcome::Accepted,
    };
    let error = store.reply_guichet("owner", input(), NOW).unwrap_err();
    assert!(error.to_string().contains("fixture-lifecycle-write"));
    drop(store);
    let mut reopened = Store::open(&fixture.database()).unwrap();
    // Mutant : commit replied/answered AVANT l'écrivain d'événement. Ces
    // deux lectures d'une NOUVELLE connexion deviennent terminales et rouges.
    assert_eq!(
        reopened.get_request("linked").unwrap().unwrap().state,
        "open"
    );
    let state: (String, Option<Vec<u8>>, Vec<u8>) = fixture.sql().query_row(
        "SELECT state, reply_bytes, canonical_request FROM guichet_requests WHERE request_id = 'deposit'",
        [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).unwrap();
    assert_eq!(
        state,
        ("claimed".to_string(), None, deposit.canonical_bytes)
    );
    assert!(reopened.guichet_lifecycle_events().unwrap().is_empty());
    fixture
        .sql()
        .execute_batch("DROP TRIGGER reject_event")
        .unwrap();
    assert!(
        matches!(reopened.reply_guichet("owner", input(), NOW).unwrap(),
        GuichetResult::Terminal { reply_bytes, newly_finalized: true, .. } if reply_bytes == reply)
    );
    assert_eq!(
        reopened.get_request("linked").unwrap().unwrap().state,
        "answered"
    );
    let events = reopened.guichet_lifecycle_events().unwrap();
    assert_eq!(events.len(), 1);
    assert!(
        matches!(reopened.reply_guichet("owner", input(), NOW + 1).unwrap(),
        GuichetResult::Terminal { reply_bytes, newly_finalized: false, .. } if reply_bytes == reply)
    );
    let replay = reopened.guichet_lifecycle_events().unwrap();
    assert_eq!(replay.len(), 1);
    assert_eq!(replay[0].event_id, events[0].event_id);
    assert_eq!(replay[0].observed_at, events[0].observed_at);
}

#[test]
fn service_ne_clot_pas_une_demande_d_autrui_et_ne_rouvre_pas_un_terminal() {
    for (target, terminal) in [("un-autre-agent", false), ("agent", true)] {
        let fixture = Fixture::new();
        let mut store = Store::open(&fixture.database()).unwrap();
        let sender = "89000000-0000-4000-8000-000000000002";
        store.create_request("linked", sender, target, 60).unwrap();
        if terminal {
            store
                .cancel_request("linked", sender, Some("déjà clos"))
                .unwrap();
        }
        let before = store.get_request("linked").unwrap();
        let deposit = linked_deposit();
        store.deposit_guichet(&deposit, 600, 60, NOW).unwrap();
        let GuichetNext::Claimed(claim) = store.claim_next_guichet("owner", NOW).unwrap() else {
            panic!("claim absent")
        };
        let result = store
            .reply_guichet(
                "owner",
                GuichetReplyInput {
                    issuer_scope: SCOPE,
                    request_id: "deposit",
                    generation: claim.claim_generation,
                    token: &claim.claim_token,
                    response_message_id: "reply",
                    reply_bytes: b"report-verbatim",
                    in_reply_to: "linked",
                    outcome: GuichetOutcome::Accepted,
                },
                NOW,
            )
            .unwrap();
        // L'acceptation du RAPPORT ne permet ni d'inventer une réponse de
        // l'autre agent, ni de rouvrir une demande déjà terminale (D-208).
        assert!(
            matches!(result, GuichetResult::Terminal { reply_bytes, .. } if reply_bytes == b"report-verbatim")
        );
        assert_eq!(
            store.get_request("linked").unwrap(),
            before,
            "mutation : UPDATE sans le couple sender/target ou sans state=open"
        );
        assert!(
            store.guichet_lifecycle_events().unwrap().is_empty(),
            "aucun answered inventé"
        );
    }
}

#[test]
fn echec_answered_annule_ack_et_terminal_sans_effacer_le_ledger_de_dispatch() {
    let fixture = Fixture::new();
    let store = Store::open(&fixture.database()).unwrap();
    store
        .create_request("question", "asker", "responder", 60)
        .unwrap();
    let mut idempotency = IdempotencyStore::open(&fixture.database()).unwrap();
    let key = IdempotencyKey::new(SCOPE, OperationKind::Send, "reply-key").unwrap();
    assert!(matches!(
        idempotency
            .reserve(&key, b"canon", NOW, 600, NOW, 30)
            .unwrap(),
        Reservation::Prepared { .. }
    ));
    let mut message = BridgetMessage::new("responder", "asker", "answer unchanged");
    message.id = "reply-message".to_string();
    message.in_reply_to = Some("question".to_string());
    let bytes = serde_json::to_vec(&message).unwrap();
    idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery".to_string(),
                recipient_instance_id: "instance".to_string(),
                delivery_generation: 1,
                expires_at: NOW + 600,
                message_bytes: bytes.clone(),
            },
        )
        .unwrap();
    fixture
        .sql()
        .execute_batch(
            "CREATE TRIGGER reject_answer BEFORE UPDATE ON tracked_requests
         WHEN NEW.state = 'answered'
         BEGIN SELECT RAISE(ABORT, 'fixture-answer-write'); END;",
        )
        .unwrap();
    let error = idempotency
        .acknowledge_send_delivery("delivery", "instance", 1)
        .unwrap_err();
    assert!(error.to_string().contains("fixture-answer-write"));
    drop(idempotency);
    let mut reopened = IdempotencyStore::open(&fixture.database()).unwrap();
    // Mutant : commit de l'ACK/Accepted AVANT mark_answered_in_transaction.
    // Le retry ne pourrait plus clôturer ; l'état ci-dessous deviendrait terminal.
    assert!(matches!(
        reopened.lookup(&key, NOW).unwrap(),
        LookupResult::OutcomeUnknown { .. }
    ));
    let phase: (String, Vec<u8>) = fixture
        .sql()
        .query_row(
            "SELECT phase, message_bytes FROM send_deliveries WHERE delivery_id = 'delivery'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(phase, ("dispatching".to_string(), bytes));
    assert_eq!(
        store.get_request("question").unwrap().unwrap().state,
        "open"
    );
    let ledger_count = || {
        fixture
            .sql()
            .query_row(
                "SELECT count(*) FROM ledger WHERE id = 'reply-message'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
    };
    // Le ledger atteste déjà le dispatch ; ce fait ne dépend pas de l'ACK.
    assert_eq!(ledger_count(), 1);
    fixture
        .sql()
        .execute_batch("DROP TRIGGER reject_answer")
        .unwrap();
    assert_eq!(
        reopened
            .acknowledge_send_delivery("delivery", "instance", 1)
            .unwrap(),
        Some("question".to_string())
    );
    assert_eq!(
        store.get_request("question").unwrap().unwrap().state,
        "answered"
    );
    assert_eq!(
        reopened
            .acknowledge_send_delivery("delivery", "instance", 1)
            .unwrap(),
        None
    );
    assert_eq!(ledger_count(), 1);
}
