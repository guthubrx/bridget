use super::*;
use crate::project_compat::{
    DogfoodingBridgetMode, DogfoodingBridgetState, ProjectEnvironmentState,
};
use bridget_transport::greffe_authorization::GreffeAuthorizationAttestation;
use bridget_transport::protocol::{
    GuichetLifecycleState, GuichetOutcome, ProjectAdminOperation, ProjectBackend,
    ProjectBindStatus, ProjectBindingStatus, ProjectRegistryRefusal, ProjectRole,
    ProjectRoundOperation, ProjectRoundRefusal, ServiceRequestOperation, ServiceRequestPayload,
};
use rusqlite::params;
use uuid::Uuid;

fn authorization_attestation(signature: &str) -> GreffeAuthorizationAttestation {
    GreffeAuthorizationAttestation {
        version: 1,
        principal: bridget_transport::greffe_authorization::GreffePrincipal {
            name: "agent-autorise".to_string(),
            instance_id: "instance-autorisee".to_string(),
        },
        action: bridget_transport::greffe_authorization::GreffeMutationAction::Delegate,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "request-authorization-replay".to_string(),
        request_issued_at: 1_787_500_000,
        canonical_request_sha256:
            "b46a71da9cfb187a84a26628604f5d51154f7aa410e3dcaf9d2f7f9b1a0d08f0".to_string(),
        grant_expires_at: 1_787_500_600,
        policy_generation: 7,
        signature: signature.to_string(),
    }
}

fn guichet_deposit(request_id: &str, bytes: &[u8]) -> GuichetDeposit {
    GuichetDeposit {
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: request_id.to_string(),
        issued_at: 1_787_500_000,
        from: "codex-1".to_string(),
        operation: ServiceRequestOperation::MissionStatus,
        payload: ServiceRequestPayload::Delegation {
            delegation_id: "delegation-1".to_string(),
        },
        canonical_bytes: bytes.to_vec(),
        authorization_attestation: None,
    }
}

#[test]
fn guichet_rejoue_les_octets_et_releve_fifo_apres_reouverture() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let first = guichet_deposit("request-1", br#"{\"request\":1}"#);
    let second = guichet_deposit("request-2", br#"{\"request\":2}"#);
    assert!(matches!(
        store.deposit_guichet(&first, 600, 60, first.issued_at),
        Ok(GuichetResult::Queued { .. })
    ));
    assert!(matches!(
        store.deposit_guichet(&second, 600, 60, second.issued_at),
        Ok(GuichetResult::Queued { .. })
    ));
    let claim = match store
        .claim_next_guichet("service-a", first.issued_at)
        .unwrap()
    {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("premier dépôt FIFO absent"),
    };
    assert_eq!(claim.request_id, "request-1");
    drop(store);

    // Mutation discriminante : sans remise en file au redémarrage, le
    // premier dépôt resterait bloqué claimed et request-2 serait relevé.
    let mut reopened = Store::open(&path).unwrap();
    reopened.recover_guichet_claims_after_restart().unwrap();
    let replay = match reopened
        .claim_next_guichet("service-b", first.issued_at + 1)
        .unwrap()
    {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("dépôt persistant absent"),
    };
    assert_eq!(replay.request_id, "request-1");
    assert_ne!(replay.claim_token, claim.claim_token);
    assert_eq!(replay.claim_generation, claim.claim_generation + 1);
    assert!(matches!(
        reopened.deposit_guichet(&first, 600, 60, first.issued_at + 1),
        Ok(GuichetResult::OutcomeUnknown { .. })
    ));
    assert!(matches!(
        reopened.deposit_guichet(
            &guichet_deposit("request-1", br#"{\"request\":9}"#),
            600,
            60,
            first.issued_at + 1
        ),
        Ok(GuichetResult::CanonicalBytesMismatch)
    ));
    drop(reopened);
    let _ = std::fs::remove_file(path);
}

#[test]
fn rejeu_guichet_restitue_attestation_originale_sans_rafraichir_les_droits() {
    let path = std::env::temp_dir().join(format!(
        "bridget-guichet-authorization-replay-{}.db",
        Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    let original = authorization_attestation("signature-originale");
    let refreshed = authorization_attestation("signature-fraiche-interdite");
    let mut first = guichet_deposit(
        "request-authorization-replay",
        br#"{"request":"authorization-replay"}"#,
    );
    first.operation = ServiceRequestOperation::Delegate;
    first.payload = ServiceRequestPayload::Delegate {
        goal: "déléguer sans rafraîchir les droits".to_string(),
        review_target: None,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: bridget_transport::protocol::GuichetDurationClass::Courte,
        suite: bridget_transport::protocol::ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: None,
    };
    first.authorization_attestation = Some(original.clone());
    assert!(matches!(
        store.deposit_guichet(&first, 600, 60, first.issued_at),
        Ok(GuichetResult::Queued { .. })
    ));

    let mut replay = first.clone();
    replay.authorization_attestation = Some(refreshed.clone());
    assert!(matches!(
        store.deposit_guichet(&replay, 600, 60, first.issued_at + 1),
        Ok(GuichetResult::OutcomeUnknown { .. })
    ));
    let claim = match store
        .claim_next_guichet("service-authorization", first.issued_at + 1)
        .unwrap()
    {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("dépôt autorisé absent"),
    };
    assert_eq!(claim.authorization_attestation, Some(original));
    assert_ne!(claim.authorization_attestation, Some(refreshed));
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn depot_direct_refuse_une_trame_guichet_superieure_a_64_kio() {
    let path =
        std::env::temp_dir().join(format!("bridget-guichet-frame-limit-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let now = 1_787_500_000;
    let exact = guichet_deposit("request-exact", &vec![b'x'; MAX_GUICHET_FRAME_BYTES - 1]);
    let oversized = guichet_deposit("request-oversized", &vec![b'x'; MAX_GUICHET_FRAME_BYTES]);

    assert!(matches!(
        store.deposit_guichet(&exact, 600, 60, now),
        Ok(GuichetResult::Queued { .. })
    ));
    assert!(matches!(
        store.deposit_guichet(&oversized, 600, 60, now),
        Err(StoreError::FrameTooLarge {
            max_frame_bytes: MAX_GUICHET_FRAME_BYTES
        })
    ));
    let oversized_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
            [&oversized.request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(oversized_count, 0, "le refus précède toute persistance");
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn schema_guichet_ne_cree_plus_la_colonne_result_bytes() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-schema-{}.db", Uuid::new_v4()));
    let store = Store::open(&path).unwrap();
    let mut statement = store
        .conn
        .prepare("PRAGMA table_info(guichet_requests)")
        .unwrap();
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(!columns.iter().any(|column| column == "result_bytes"));
    assert!(columns.iter().any(|column| column == "reply_bytes"));
    assert!(
        columns
            .iter()
            .any(|column| column == "authorization_attestation")
    );
    drop(statement);
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn ouverture_ajoute_la_metadonnee_serveur_et_garde_un_depot_historique_sans_preuve() {
    let path = std::env::temp_dir().join(format!(
        "bridget-guichet-authorization-migration-{}.db",
        Uuid::new_v4()
    ));
    let legacy = Connection::open(&path).unwrap();
    legacy
            .execute_batch(
                "CREATE TABLE guichet_requests (
                    deposited_sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                    issuer_scope TEXT NOT NULL,
                    operation_kind TEXT NOT NULL,
                    request_id TEXT NOT NULL,
                    canonical_request BLOB NOT NULL,
                    issued_at INTEGER NOT NULL,
                    expires_at INTEGER NOT NULL,
                    sender TEXT NOT NULL,
                    linked_request_id TEXT,
                    state TEXT NOT NULL CHECK (state IN ('queued', 'claimed', 'replied', 'rejected')),
                    claim_owner TEXT,
                    claim_generation INTEGER NOT NULL DEFAULT 0,
                    claim_token TEXT,
                    claim_lease_expires_at INTEGER,
                    result_issue TEXT,
                    reply_bytes BLOB,
                    UNIQUE (issuer_scope, operation_kind, request_id)
                );
                INSERT INTO guichet_requests (
                    issuer_scope, operation_kind, request_id, canonical_request,
                    issued_at, expires_at, sender, state
                ) VALUES (
                    '015_scope_0123456789abcdef0123456789abcdef',
                    'service_request', 'request-historique', X'010203',
                    1787500000, 1787500600, 'agent-historique', 'queued'
                );",
            )
            .unwrap();
    drop(legacy);

    let mut store = Store::open(&path).unwrap();
    let claim = match store
        .claim_next_guichet("service-historique", 1_787_500_001)
        .unwrap()
    {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("dépôt historique absent après ouverture"),
    };
    assert_eq!(claim.canonical_request, vec![1, 2, 3]);
    assert_eq!(claim.authorization_attestation, None);
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn second_handle_ne_revoque_un_claim_qu_au_bootstrap_explicite() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-handles-{}.db", Uuid::new_v4()));
    let mut first = Store::open(&path).unwrap();
    let now = 1_787_500_000;
    let finalized = guichet_deposit("request-finalized", br#"{\"request\":1}"#);
    let recovered = guichet_deposit("request-recovered", br#"{\"request\":2}"#);
    first.deposit_guichet(&finalized, 600, 60, now).unwrap();
    first.deposit_guichet(&recovered, 600, 60, now).unwrap();
    let claim_finalized = match first.claim_next_guichet("service-a", now).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim A finalisable absent"),
    };

    // Ouvrir SQLite une seconde fois n'est pas un redémarrage : la lease
    // d'A reste active et A peut encore finaliser son premier dépôt.
    let mut second = Store::open(&path).unwrap();
    assert!(matches!(
        first.reply_guichet(
            "service-a",
            GuichetReplyInput {
                issuer_scope: &claim_finalized.issuer_scope,
                request_id: &claim_finalized.request_id,
                generation: claim_finalized.claim_generation,
                token: &claim_finalized.claim_token,
                response_message_id: "reply-finalized",
                reply_bytes: br#"{\"reply\":\"a\"}"#,
                in_reply_to: "",
                outcome: GuichetOutcome::Accepted,
            },
            now,
        ),
        Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
    ));

    let claim_recovered = match first.claim_next_guichet("service-a", now).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim A à reprendre absent"),
    };
    assert_eq!(claim_recovered.request_id, "request-recovered");

    // Mutation discriminante : si Store::open libérait encore les leases,
    // le claim suivant d'A ou la relève de B ne prouverait plus que seul le
    // bootstrap remet les claims vivants en FIFO.
    second.recover_guichet_claims_after_restart().unwrap();
    let claimed_by_b = match second.claim_next_guichet("service-b", now + 1).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim B repris absent"),
    };
    assert_eq!(
        claimed_by_b.deposited_sequence,
        claim_recovered.deposited_sequence
    );
    assert_eq!(claimed_by_b.request_id, claim_recovered.request_id);
    assert_eq!(
        claimed_by_b.claim_generation,
        claim_recovered.claim_generation + 1
    );
    assert_ne!(claimed_by_b.claim_token, claim_recovered.claim_token);
    drop(first);
    drop(second);
    let _ = std::fs::remove_file(path);
}

#[test]
fn guichet_reply_stale_ne_peut_pas_gagner_apres_lease_expire() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-lease-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let deposit = guichet_deposit("request-lease", br#"{\"request\":1}"#);
    let now = deposit.issued_at;
    store.deposit_guichet(&deposit, 600, 60, now).unwrap();
    let a = match store.claim_next_guichet("service-a", now).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim A absent"),
    };
    let b = match store
        .claim_next_guichet("service-b", a.claim_lease_expires_at + 1)
        .unwrap()
    {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim B absent"),
    };
    assert!(matches!(
        store.reply_guichet(
            "service-a",
            GuichetReplyInput {
                issuer_scope: &a.issuer_scope,
                request_id: &a.request_id,
                generation: a.claim_generation,
                token: &a.claim_token,
                response_message_id: "reply-stale",
                reply_bytes: br#"{\"reply\":\"a\"}"#,
                in_reply_to: "",
                outcome: GuichetOutcome::Accepted,
            },
            b.claim_lease_expires_at - 1,
        ),
        Ok(GuichetResult::ClaimStale)
    ));
    assert!(matches!(
        store.reply_guichet(
            "service-b", GuichetReplyInput {
                issuer_scope: &b.issuer_scope, request_id: &b.request_id,
                generation: b.claim_generation, token: &b.claim_token,
                response_message_id: "reply-current",
                reply_bytes: br#"{\"reply\":\"b\"}"#, in_reply_to: "",
                outcome: GuichetOutcome::Accepted,
            },
            b.claim_lease_expires_at - 1,
        ),
        Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
    ));
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn resultat_terminal_restitue_les_octets_durables_a_la_finalisation_au_lookup_et_au_rejeu() {
    let path = std::env::temp_dir().join(format!(
        "bridget-guichet-terminal-bytes-{}.db",
        Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    let deposit = guichet_deposit("request-terminal-bytes", br#"{"request":"terminal-bytes"}"#);
    let now = deposit.issued_at;
    store.deposit_guichet(&deposit, 600, 60, now).unwrap();
    let claim = match store.claim_next_guichet("service-terminal", now).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("claim terminal absent"),
    };
    let expected = br#"{"type":"guichet_reply","result":"created"}"#;

    let finalized = store
        .reply_guichet(
            "service-terminal",
            GuichetReplyInput {
                issuer_scope: &claim.issuer_scope,
                request_id: &claim.request_id,
                generation: claim.claim_generation,
                token: &claim.claim_token,
                response_message_id: "reply-terminal-bytes",
                reply_bytes: expected,
                in_reply_to: "",
                outcome: GuichetOutcome::Accepted,
            },
            now,
        )
        .unwrap();
    match finalized {
        GuichetResult::Terminal {
            newly_finalized: true,
            reply_bytes,
            ..
        } => assert_eq!(reply_bytes, expected, "octets rendus à la finalisation"),
        other => panic!("résultat terminal finalisé attendu, reçu {other:?}"),
    }

    match store
        .lookup_guichet(&claim.issuer_scope, &claim.request_id, now + 1)
        .unwrap()
    {
        GuichetResult::Terminal {
            newly_finalized: false,
            reply_bytes,
            ..
        } => assert_eq!(reply_bytes, expected, "octets relus par lookup"),
        other => panic!("résultat terminal relu attendu, reçu {other:?}"),
    }

    match store
        .reply_guichet(
            "service-terminal",
            GuichetReplyInput {
                issuer_scope: &claim.issuer_scope,
                request_id: &claim.request_id,
                generation: claim.claim_generation,
                token: &claim.claim_token,
                response_message_id: "reply-terminal-bytes",
                reply_bytes: expected,
                in_reply_to: "",
                outcome: GuichetOutcome::Accepted,
            },
            now + 1,
        )
        .unwrap()
    {
        GuichetResult::Terminal {
            newly_finalized: false,
            reply_bytes,
            ..
        } => assert_eq!(reply_bytes, expected, "octets rendus au rejeu terminal"),
        other => panic!("rejeu terminal attendu, reçu {other:?}"),
    }
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn reponse_guichet_accepted_clot_atomiquement_la_demande_liee() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-answer-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let now = 1_787_500_000;
    store
        .create_request("message-lie", "guichet", "codex-1", 60)
        .unwrap();
    let deposit = GuichetDeposit {
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: "request-answer".to_string(),
        issued_at: now,
        from: "codex-1".to_string(),
        operation: ServiceRequestOperation::DeliveryReport,
        payload: ServiceRequestPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
            in_reply_to: "message-lie".to_string(),
            review_verdict: None,
        },
        canonical_bytes: br#"{"type":"service_request"}"#.to_vec(),
        authorization_attestation: None,
    };
    store.deposit_guichet(&deposit, 600, 60, now).unwrap();
    let claim = match store.claim_next_guichet("guichet-connection", now).unwrap() {
        GuichetNext::Claimed(claim) => claim,
        GuichetNext::Empty => panic!("dépôt lié absent"),
    };
    assert!(matches!(
        store.reply_guichet(
            "guichet-connection",
            GuichetReplyInput {
                issuer_scope: &claim.issuer_scope,
                request_id: &claim.request_id,
                generation: claim.claim_generation,
                token: &claim.claim_token,
                response_message_id: "reply-linked",
                reply_bytes: br#"{"type":"guichet_reply"}"#,
                in_reply_to: "message-lie",
                outcome: GuichetOutcome::Accepted,
            },
            now,
        ),
        Ok(GuichetResult::Terminal { ref issue, .. }) if issue == "accepted"
    ));
    assert_eq!(
        store.get_request("message-lie").unwrap().unwrap().state,
        "answered",
        "mutation discriminante : sans mark_answered_in_transaction dans la transaction du reply, la demande resterait open"
    );
    let events = store.guichet_lifecycle_events().unwrap();
    assert!(matches!(
        events.as_slice(),
        [GuichetLifecycleEvent {
            request_id,
            state: GuichetLifecycleState::Answered,
            in_reply_to: Some(in_reply_to),
            response_message_id: Some(response_message_id),
            ..
        }] if request_id == "request-answer"
            && in_reply_to == "message-lie"
            && response_message_id == "reply-linked"
    ));

    // Mutation discriminante : si l'événement était écrit hors de la
    // transaction de clôture, un rejeu pourrait créer une seconde preuve
    // ou laisser une demande answered sans fait durable correspondant.
    assert!(matches!(
        store.reply_guichet(
            "guichet-connection",
            GuichetReplyInput {
                issuer_scope: &claim.issuer_scope,
                request_id: &claim.request_id,
                generation: claim.claim_generation,
                token: &claim.claim_token,
                response_message_id: "reply-linked",
                reply_bytes: br#"{"type":"guichet_reply"}"#,
                in_reply_to: "message-lie",
                outcome: GuichetOutcome::Accepted,
            },
            now + 1,
        ),
        Ok(GuichetResult::Terminal {
            newly_finalized: false,
            ..
        })
    ));
    assert_eq!(store.guichet_lifecycle_events().unwrap().len(), 1);
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn guichet_claim_next_est_atomique_entre_deux_stores() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let path = std::env::temp_dir().join(format!("bridget-guichet-race-{}.db", Uuid::new_v4()));
    let mut seed = Store::open(&path).unwrap();
    let now = 1_787_500_000;
    for request_id in ["request-a", "request-b"] {
        let deposit = guichet_deposit(request_id, request_id.as_bytes());
        seed.deposit_guichet(&deposit, 600, 60, now).unwrap();
    }
    drop(seed);
    let mut first = Store::open(&path).unwrap();
    let mut second = Store::open(&path).unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let left = Arc::clone(&barrier);
    let one = thread::spawn(move || {
        left.wait();
        first.claim_next_guichet("service-a", now).unwrap()
    });
    let right = Arc::clone(&barrier);
    let two = thread::spawn(move || {
        right.wait();
        second.claim_next_guichet("service-b", now).unwrap()
    });
    let claimed = [one.join().unwrap(), two.join().unwrap()]
        .into_iter()
        .map(|next| match next {
            GuichetNext::Claimed(claim) => claim.request_id,
            GuichetNext::Empty => panic!("deux dépôts FIFO devaient être relevés"),
        })
        .collect::<std::collections::BTreeSet<_>>();
    // Mutation discriminante : un SELECT puis UPDATE non IMMEDIATE peut
    // donner request-a deux fois ou échouer au lieu de distribuer A puis B.
    assert_eq!(
        claimed,
        ["request-a".to_string(), "request-b".to_string()].into()
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn cancellation_is_idempotent_and_terminal() {
    let path = std::env::temp_dir().join(format!("bridget-store-{}.db", Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let mut store = Store::open(&path).unwrap();
    store
        .create_request("request-1", "alice", "bob", 60)
        .unwrap();
    let first = store
        .cancel_request("request-1", "alice", Some("priorité changée"))
        .unwrap()
        .unwrap();
    let second = store
        .cancel_request("request-1", "alice", None)
        .unwrap()
        .unwrap();
    assert_eq!(first.state, "cancelled");
    assert_eq!(second.state, "cancelled");
    assert!(!store.mark_answered("request-1", "bob", "alice").unwrap());
    store
        .create_request("request-2", "alice", "bob", 60)
        .unwrap();
    assert!(store.mark_answered("request-2", "bob", "alice").unwrap());
    assert_eq!(
        store.get_request("request-2").unwrap().unwrap().state,
        "answered"
    );
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(
        reopened.get_request("request-1").unwrap().unwrap().state,
        "cancelled"
    );
    drop(reopened);
    let _ = std::fs::remove_file(path);
}

#[test]
fn transitions_cancelled_et_timed_out_deposent_un_fait_guichet_unique() {
    let path = std::env::temp_dir().join(format!("bridget-guichet-terminal-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let now = 1_787_500_000;
    for (request_id, state) in [
        ("message-cancelled", GuichetLifecycleState::Cancelled),
        ("message-timed-out", GuichetLifecycleState::TimedOut),
    ] {
        store
            .create_request(request_id, "alice", "bob", 60)
            .unwrap();
        let deposit = GuichetDeposit {
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: format!("request-{request_id}"),
            issued_at: now,
            from: "alice".to_string(),
            operation: ServiceRequestOperation::DeliveryReport,
            payload: ServiceRequestPayload::DeliveryReport {
                objective_id: "objective-1".to_string(),
                delegation_id: "delegation-1".to_string(),
                delivery_hash: "0".repeat(64),
                in_reply_to: request_id.to_string(),
                review_verdict: None,
            },
            canonical_bytes: format!("{{\"request\":\"{request_id}\"}}").into_bytes(),
            authorization_attestation: None,
        };
        store.deposit_guichet(&deposit, 600, 60, now).unwrap();
        match state {
            GuichetLifecycleState::Cancelled => {
                store
                    .cancel_request(request_id, "alice", Some("annulé"))
                    .unwrap();
                store
                    .cancel_request(request_id, "alice", Some("rejeu"))
                    .unwrap();
            }
            GuichetLifecycleState::TimedOut => {
                assert!(store.mark_timed_out(request_id).unwrap());
                assert!(!store.mark_timed_out(request_id).unwrap());
            }
            GuichetLifecycleState::Answered => unreachable!(),
        }
    }
    let events = store.guichet_lifecycle_events().unwrap();
    assert!(events.iter().any(|event| {
        event.request_id == "request-message-cancelled"
            && event.state == GuichetLifecycleState::Cancelled
            && event.in_reply_to.as_deref() == Some("message-cancelled")
    }));
    assert!(events.iter().any(|event| {
        event.request_id == "request-message-timed-out"
            && event.state == GuichetLifecycleState::TimedOut
            && event.in_reply_to.as_deref() == Some("message-timed-out")
    }));
    assert_eq!(
        events.len(),
        2,
        "chaque transition terminale ne dépose qu'un seul fait"
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn deferred_reminder_is_persisted_for_ledger_readers() {
    let path = std::env::temp_dir().join(format!("bridget-store-events-{}.db", Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let store = Store::open(&path).unwrap();
    store
        .create_request("request-1", "alice", "bob", 60)
        .unwrap();
    store.record_deferred_reminder("request-1", 2).unwrap();
    assert_eq!(
        store
            .latest_deferred_reminder("request-1")
            .unwrap()
            .map(|event| event.0),
        Some(2)
    );
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(
        reopened
            .latest_deferred_reminder("request-1")
            .unwrap()
            .map(|event| event.0),
        Some(2)
    );
    drop(reopened);
    let _ = std::fs::remove_file(path);
}

#[test]
fn purge_supprime_avec_la_demande_les_evenements_associes() {
    let path = std::env::temp_dir().join(format!("bridget-store-purge-{}.db", Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let mut store = Store::open(&path).unwrap();
    store
        .create_request("request-1", "alice", "bob", 60)
        .unwrap();
    assert!(store.mark_timed_out("request-1").unwrap());
    store.record_deferred_reminder("request-1", 2).unwrap();
    store
        .conn
        .execute(
            "UPDATE tracked_requests SET completed_at = ?1 WHERE id = ?2",
            rusqlite::params![now_secs() - 86_401, "request-1"],
        )
        .unwrap();

    store.purge_older_than_days(1).unwrap();

    assert!(store.get_request("request-1").unwrap().is_none());
    assert!(
        store
            .latest_deferred_reminder("request-1")
            .unwrap()
            .is_none()
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn purge_guichet_conserve_les_demandes_encore_vivantes() {
    let path =
        std::env::temp_dir().join(format!("bridget-store-purge-guichet-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let now = now_secs();
    let cutoff = now - 86_400;
    let mut terminal = guichet_deposit("request-terminal", br#"{\"request\":1}"#);
    terminal.issued_at = now;
    let mut live = guichet_deposit("request-live", br#"{\"request\":2}"#);
    live.issued_at = now;
    store.deposit_guichet(&terminal, 600, 60, now).unwrap();
    store.deposit_guichet(&live, 600, 60, now).unwrap();
    store
        .conn
        .execute(
            "UPDATE guichet_requests
                 SET state = 'replied', result_issue = 'accepted', expires_at = ?1
                 WHERE request_id = ?2",
            params![cutoff - 1, terminal.request_id],
        )
        .unwrap();
    store
        .conn
        .execute(
            "UPDATE guichet_requests SET expires_at = ?1 WHERE request_id = ?2",
            params![cutoff - 1, live.request_id],
        )
        .unwrap();
    for deposit in [&terminal, &live] {
        store
            .conn
            .execute(
                "INSERT INTO guichet_lifecycle_events
                     (issuer_scope, request_id, event_id, state, observed_at)
                     VALUES (?1, ?2, ?3, 'answered', ?4)",
                params![
                    deposit.issuer_scope,
                    deposit.request_id,
                    format!("event-{}", deposit.request_id),
                    cutoff - 1
                ],
            )
            .unwrap();
    }

    store.purge_older_than_days(1).unwrap();

    let terminal_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
            [&terminal.request_id],
            |row| row.get(0),
        )
        .unwrap();
    let live_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM guichet_requests WHERE request_id = ?1",
            [&live.request_id],
            |row| row.get(0),
        )
        .unwrap();
    let terminal_event_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM guichet_lifecycle_events WHERE request_id = ?1",
            [&terminal.request_id],
            |row| row.get(0),
        )
        .unwrap();
    let live_event_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM guichet_lifecycle_events WHERE request_id = ?1",
            [&live.request_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(terminal_count, 0);
    assert_eq!(terminal_event_count, 0);
    assert_eq!(live_count, 1, "une demande non terminale reste vivante");
    assert_eq!(
        live_event_count, 1,
        "l'événement d'une demande vivante ne doit pas être purgé"
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn usage_sans_echantillon_reste_inconnu_et_n_invente_pas_zero() {
    let path = std::env::temp_dir().join(format!("bridget-usage-{}.db", Uuid::new_v4()));
    let store = Store::open(&path).unwrap();
    assert_eq!(
        store
            .aggregate_usage_window("tmux-sans-sonde", 1, 100)
            .unwrap(),
        None
    );
    store
        .record_usage_sample(
            "claude-1",
            50,
            bridget_transport::protocol::UsageTokens {
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
            },
            "claude-stream-json",
            Some("claude"),
            Some("claude-opus-5"),
        )
        .unwrap();
    let aggregate = store
        .aggregate_usage_window("claude-1", 1, 100)
        .unwrap()
        .expect("échantillon attesté");
    assert_eq!(aggregate.turns, 1);
    assert_eq!(aggregate.input_tokens, 2);
    assert_eq!(aggregate.output_tokens, 175);
    assert_eq!(aggregate.cache_creation_input_tokens, 40_804);
    assert_eq!(aggregate.cache_read_input_tokens, 13_907);
    assert_eq!(aggregate.facturable_tokens, 40_981);
    assert_eq!(
        store.usage_dashboard_window(1, 100).unwrap(),
        vec![UsageDashboardRow {
            provider_kind: Some("claude".to_string()),
            model: Some("claude-opus-5".to_string()),
            source: "claude-stream-json".to_string(),
            samples: 1,
            input_tokens: 2,
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
        }]
    );
    assert_eq!(
        store.aggregate_usage_window("claude-1", 80, 100).unwrap(),
        None
    );
    assert_eq!(
        store.aggregate_usage_window("claude-1", 1, 40).unwrap(),
        None,
        "échantillon après to_secs exclu"
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

const RECENT_MESSAGES_PLAN_SQL: &str = "SELECT l.id, l.ts, l.sender, l.target, l.body,
            (SELECT d.phase FROM send_deliveries d
             WHERE d.operation_kind = 'send' AND d.idempotency_key = l.id
             ORDER BY CASE d.phase
                 WHEN 'acked' THEN 0
                 WHEN 'orphaned' THEN 1
                 WHEN 'dispatching' THEN 2
                 ELSE 3
             END
             LIMIT 1) AS delivery_phase
     FROM ledger l
     ORDER BY l.ts DESC
     LIMIT ?1";

fn seed_ledger_with_deliveries(path: &Path, n: usize) {
    use crate::idempotency::IdempotencyStore;
    // Ouvre puis ferme : crée schéma + index v4, libère la connexion.
    {
        let _schema = IdempotencyStore::open(path).unwrap();
    }
    let conn = Connection::open(path).unwrap();
    let tx = conn.unchecked_transaction().unwrap();
    for i in 0..n {
        let id = format!("msg-{i:05}");
        let scope = format!("012_scope_{i:016}");
        tx.execute(
            "INSERT INTO idempotency_records (
                    issuer_scope, operation_kind, idempotency_key, canonical_bytes,
                    state, issued_at, expires_at
                 ) VALUES (?1, 'send', ?2, ?3, 'terminal', 1, 9_999_999_999)",
            rusqlite::params![scope, id, format!("canon-{i}").as_bytes()],
        )
        .unwrap();
        let phase = if i % 3 == 0 {
            "acked"
        } else if i % 3 == 1 {
            "dispatching"
        } else {
            "indeterminate"
        };
        tx.execute(
            "INSERT INTO send_deliveries (
                    delivery_id, issuer_scope, operation_kind, idempotency_key,
                    recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
                 ) VALUES (?1, ?2, 'send', ?3, 'instance-1', 1, ?4, 9_999_999_999, X'7B7D')",
            rusqlite::params![format!("del-{i:05}"), scope, id, phase],
        )
        .unwrap();
        tx.execute(
            "INSERT INTO ledger (id, ts, sender, target, body, conversation_key)
                 VALUES (?1, ?2, 'a', 'b', 'corps', 'a|b')",
            rusqlite::params![id, i as i64],
        )
        .unwrap();
    }
    tx.commit().unwrap();
}

fn explain_recent_messages_plan(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {RECENT_MESSAGES_PLAN_SQL}"))
        .unwrap();
    stmt.query_map(rusqlite::params![50_i64], |row| row.get::<_, String>(3))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn plan_searches_kind_key(plan: &[String]) -> bool {
    plan.iter().any(|line| {
        line.contains("send_deliveries")
            && line.contains("SEARCH")
            && line.contains("idx_send_deliveries_kind_key")
    })
}

fn plan_scans_send_deliveries(plan: &[String]) -> bool {
    // SQLite nomme parfois l'alias seul (« SCAN d ») dans la sous-requête.
    plan.iter().any(|line| {
        let trimmed = line.trim();
        (trimmed == "SCAN d" || trimmed.starts_with("SCAN d "))
            || (line.contains("SCAN") && line.contains("send_deliveries"))
    })
}

/// Oracle + mutant : avec l'index → SEARCH ; sans l'index → SCAN (le test
/// ROUGIT si l'on retire seulement l'assertion positive — propriété gardée
/// = dépendance réelle à idx_send_deliveries_kind_key).
#[test]
fn recent_messages_explique_search_pas_scan_sur_send_deliveries() {
    let path = std::env::temp_dir().join(format!(
        "bridget-ledger-explain-{}-{}.db",
        std::process::id(),
        Uuid::new_v4()
    ));
    seed_ledger_with_deliveries(&path, 200);
    let store = Store::open(&path).unwrap();

    let with_index = explain_recent_messages_plan(&store.conn);
    let joined_with = with_index.join("\n");
    assert!(
        plan_searches_kind_key(&with_index),
        "contrôle positif : SEARCH sur idx_send_deliveries_kind_key, plan:\n{joined_with}"
    );
    assert!(
        !plan_scans_send_deliveries(&with_index),
        "contrôle positif : pas de SCAN send_deliveries, plan:\n{joined_with}"
    );
    eprintln!("EXPLAIN avec index:\n{joined_with}");

    store
        .conn
        .execute("DROP INDEX idx_send_deliveries_kind_key", [])
        .unwrap();
    let without_index = explain_recent_messages_plan(&store.conn);
    let joined_without = without_index.join("\n");
    assert!(
        !plan_searches_kind_key(&without_index),
        "mutant : sans l'index le plan ne doit plus SEARCH kind_key, plan:\n{joined_without}"
    );
    assert!(
        plan_scans_send_deliveries(&without_index),
        "mutant : sans l'index attendu SCAN send_deliveries, plan:\n{joined_without}"
    );
    eprintln!("EXPLAIN sans index (mutant):\n{joined_without}");

    drop(store);
    let _ = std::fs::remove_file(path);
}

/// Banc jury : LIMIT 50, 10 000 messages, médiane de 5 rounds < 20 ms.
#[test]
fn recent_messages_dix_mille_mediane_sous_vingt_ms() {
    let path = std::env::temp_dir().join(format!(
        "bridget-ledger-bench-{}-{}.db",
        std::process::id(),
        Uuid::new_v4()
    ));
    seed_ledger_with_deliveries(&path, 10_000);
    let store = Store::open(&path).unwrap();
    // Amorçage : chauffe le plan (index déjà posé par IdempotencyStore).
    let _ = store.recent_messages(50).unwrap();
    let mut samples = Vec::with_capacity(5);
    for _ in 0..5 {
        let started = std::time::Instant::now();
        let rows = store.recent_messages(50).unwrap();
        samples.push(started.elapsed());
        assert_eq!(rows.len(), 50);
    }
    samples.sort();
    let median = samples[2];
    let rendered: Vec<String> = samples
        .iter()
        .map(|d| format!("{:.2}ms", d.as_secs_f64() * 1000.0))
        .collect();
    eprintln!(
        "banc 10k LIMIT 50 ×5 : {:?} médiane={:.2}ms",
        rendered,
        median.as_secs_f64() * 1000.0
    );
    assert!(
        median.as_secs_f64() * 1000.0 < 20.0,
        "médiane {:.2} ms (échantillons {:?}) — seuil jury 20 ms à 10k",
        median.as_secs_f64() * 1000.0,
        rendered
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_066_store_migre_hote_et_persiste_runtime_docker() {
    let path = std::env::temp_dir().join(format!(
        "bridget-project-runtime-store-{}.db",
        Uuid::new_v4()
    ));
    {
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE project_bindings (
                         project_id TEXT PRIMARY KEY,
                         canonical_root TEXT NOT NULL,
                         backend TEXT NOT NULL CHECK (backend = 'host'),
                         state TEXT NOT NULL,
                         generation INTEGER NOT NULL,
                         bound_at INTEGER NOT NULL,
                         updated_at INTEGER NOT NULL,
                         last_reason TEXT
                     );
                     INSERT INTO project_bindings VALUES (
                         'project-host', '/srv/projects/host', 'host', 'active', 1, 10, 10, NULL
                     );",
            )
            .unwrap();
    }

    let mut store = Store::open(&path).unwrap();
    let host = store.project_binding("project-host").unwrap().unwrap();
    assert_eq!(host.backend, ProjectBackend::Host);
    assert_eq!(host.runtime, None);

    let runtime = ProjectRuntimeBinding {
            state: ProjectEnvironmentState::Ready,
            policy_id: "fixture".to_string(),
            policy_version: 7,
            policy_digest: "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            image_reference: "example.invalid/runtime:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            resolved_image_id: Some("sha256:abcdef".to_string()),
            run_as_uid: 1002,
            run_as_gid: 1002,
            environment_epoch: 3,
            topology_digest: "sha256:fixture".to_string(),
            container_id: Some("container-opaque".to_string()),
            last_reason: None,
        };
    let docker = ProjectBinding::docker(
        "project-docker".to_string(),
        "/srv/projects/docker".to_string(),
        runtime.clone(),
        11,
    )
    .unwrap();
    store.insert_project_binding(&docker).unwrap();

    let loaded = store.project_binding("project-docker").unwrap().unwrap();
    assert_eq!(loaded, docker);
    let projection = store
        .project_binding_projection_for_project("project-docker", 12)
        .unwrap()
        .unwrap();
    assert_eq!(projection.backend, Some(ProjectBackend::Docker));
    assert_eq!(projection.runtime_policy, Some(runtime.policy_reference()));

    let mut changed = runtime.clone();
    changed.state = ProjectEnvironmentState::RecreateRequired;
    changed.environment_epoch = 4;
    changed.container_id = None;
    let updated = store
        .update_project_runtime("project-docker", &changed, 13)
        .unwrap();
    assert_eq!(updated.runtime, Some(changed));
    let reasoned = store
        .record_project_runtime_failure("project-docker", "runtime_exec_lost", 14)
        .unwrap();
    assert_eq!(
        reasoned.runtime.and_then(|runtime| runtime.last_reason),
        Some("runtime_exec_lost".to_string())
    );
    assert!(matches!(
        store.update_project_runtime("project-host", &runtime, 14),
        Err(StoreError::Invariant(_))
    ));

    let before_reopen = store.project_binding("project-docker").unwrap().unwrap();
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(
        reopened.project_binding("project-docker").unwrap().unwrap(),
        before_reopen,
        "l’extraction du moteur ne convertit pas une ancienne portée Docker en hôte"
    );
    assert_eq!(
        reopened.project_binding("project-host").unwrap().unwrap(),
        host
    );
    drop(reopened);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_085_activation_docker_publie_apres_preparation_et_rejoue_sans_regression() {
    let path = std::env::temp_dir().join(format!(
        "bridget-project-runtime-activate-{}.db",
        Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    let host = ProjectBinding::active(
        "project-085".to_string(),
        "/srv/projects/085".to_string(),
        ProjectBackend::Host,
        10,
    )
    .unwrap();
    store.insert_project_binding(&host).unwrap();
    let runtime = ProjectRuntimeBinding {
        state: ProjectEnvironmentState::Ready,
        policy_id: "production-linux-amd64".to_string(),
        policy_version: 1,
        policy_digest: format!("sha256:{}", "a".repeat(64)),
        image_reference: format!("sha256:{}", "b".repeat(64)),
        resolved_image_id: Some(format!("sha256:{}", "c".repeat(64))),
        run_as_uid: 1002,
        run_as_gid: 1002,
        environment_epoch: 1,
        topology_digest: "sha256:fixture".to_string(),
        container_id: Some("d".repeat(64)),
        last_reason: None,
    };

    let activated = store
        .activate_project_docker_binding("activate-085", "project-085", 1, runtime.clone(), 11)
        .unwrap();
    assert_eq!(activated.backend, ProjectBackend::Docker);
    assert_eq!(activated.generation, 2);
    assert_eq!(activated.runtime, Some(runtime.clone()));

    let replay = store
        .activate_project_docker_binding("activate-085", "project-085", 1, runtime, 12)
        .unwrap();
    assert_eq!(replay.generation, 2);
    assert!(matches!(
        store.activate_project_docker_binding(
            "activate-other",
            "project-085",
            1,
            replay.runtime.unwrap(),
            13,
        ),
        Err(StoreError::ProjectRegistryRefusal(
            ProjectRegistryRefusal::RebindRequired
        ))
    ));
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_086_un_seul_projet_systeme_et_lease_worktree_exclusive() {
    let path =
        std::env::temp_dir().join(format!("bridget-project-system-role-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    for project_id in ["bridget-system", "ordinary"] {
        store
            .insert_project_binding(
                &ProjectBinding::active(
                    project_id.to_string(),
                    format!("/srv/projects/{project_id}"),
                    ProjectBackend::Host,
                    10,
                )
                .unwrap(),
            )
            .unwrap();
    }
    assert_eq!(
        store.project_role("ordinary").unwrap(),
        ProjectRole::Standard
    );
    store
        .declare_bridget_system_project("bridget-system", 11)
        .unwrap();
    assert_eq!(
        store.project_role("bridget-system").unwrap(),
        ProjectRole::BridgetSystem
    );
    let disabled = store.dogfooding_bridget_state("bridget-system", 1).unwrap();
    assert_eq!(disabled.mode, DogfoodingBridgetMode::Disabled);
    let enabled = DogfoodingBridgetState {
        mode: DogfoodingBridgetMode::Enabled,
        setting_generation: 2,
        ..disabled.clone()
    };
    store
        .apply_dogfooding_bridget_state(&disabled, &enabled, 12)
        .unwrap();
    assert_eq!(
        store.dogfooding_bridget_state("bridget-system", 1).unwrap(),
        enabled
    );
    assert!(
        store
            .apply_dogfooding_bridget_state(&disabled, &enabled, 13)
            .is_err()
    );
    assert!(
        store
            .declare_bridget_system_project("ordinary", 12)
            .is_err()
    );
    assert!(
        store
            .acquire_system_worktree_lease(
                "ordinary",
                "agent-a",
                "/srv/worktrees/ordinary-a",
                1,
                13,
            )
            .is_err()
    );
    store
        .acquire_system_worktree_lease(
            "bridget-system",
            "agent-a",
            "/srv/worktrees/bridget-a",
            1,
            13,
        )
        .unwrap();
    store
        .acquire_system_worktree_lease(
            "bridget-system",
            "agent-a",
            "/srv/worktrees/bridget-a",
            1,
            14,
        )
        .unwrap();
    assert_eq!(
        store.system_worktree_leases("bridget-system").unwrap(),
        vec!["/srv/worktrees/bridget-a".to_string()]
    );
    assert!(
        store
            .acquire_system_worktree_lease(
                "bridget-system",
                "agent-b",
                "/srv/worktrees/bridget-a",
                1,
                15,
            )
            .is_err()
    );
    store
        .acquire_system_worktree_lease(
            "bridget-system",
            "agent-b",
            "/srv/worktrees/bridget-b",
            1,
            16,
        )
        .unwrap();
    assert_eq!(
        store.system_worktree_leases("bridget-system").unwrap(),
        vec![
            "/srv/worktrees/bridget-a".to_string(),
            "/srv/worktrees/bridget-b".to_string()
        ]
    );
    assert!(
        store
            .release_system_worktree_lease("bridget-system", "agent-a", "/srv/worktrees/bridget-a",)
            .unwrap()
    );
    assert!(
        !store
            .release_system_worktree_lease("bridget-system", "agent-a", "/srv/worktrees/bridget-a",)
            .unwrap()
    );
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_079_politique_ronde_absente_idempotente_et_epinglee_au_rebind() {
    let path = std::env::temp_dir().join(format!("bridget-project-round-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    store
        .bind_project_registration("register-round-1", "project-1", "/srv/projects/one", 100)
        .unwrap();
    store
        .bind_project_registration("register-round-2", "project-2", "/srv/projects/two", 100)
        .unwrap();

    let initial = store.project_round_policies(101).unwrap();
    assert_eq!(initial.len(), 2);
    assert!(initial.iter().all(|policy| {
        policy.active && !policy.configured && !policy.enabled && policy.revision == 0
    }));

    let enabled = store
        .apply_project_round_mutation(
            "round-enable-1",
            ProjectRoundOperation::Enable,
            "project-1",
            1,
            102,
        )
        .unwrap();
    assert!(enabled.reason.is_none());
    assert!(matches!(
        enabled.policies.as_slice(),
        [policy]
            if policy.project_id == "project-1"
                && policy.binding_generation == Some(1)
                && policy.configured
                && policy.enabled
                && policy.revision == 1
    ));
    assert_eq!(
        store
            .apply_project_round_mutation(
                "round-enable-1",
                ProjectRoundOperation::Enable,
                "project-1",
                1,
                999,
            )
            .unwrap(),
        enabled,
        "le rejeu exact relit l'issue initiale"
    );
    assert!(matches!(
        store.apply_project_round_mutation(
            "round-enable-1",
            ProjectRoundOperation::Disable,
            "project-1",
            1,
            103,
        ),
        Err(StoreError::ProjectRoundRefusal(
            ProjectRoundRefusal::EnvelopeMismatch
        ))
    ));
    assert_eq!(
        store.enabled_project_round_policies(103).unwrap(),
        enabled.policies
    );

    let disabled = store
        .apply_project_round_mutation(
            "round-disable-1",
            ProjectRoundOperation::Disable,
            "project-1",
            1,
            104,
        )
        .unwrap();
    assert!(matches!(
        disabled.policies.as_slice(),
        [policy] if policy.configured && !policy.enabled && policy.revision == 2
    ));
    assert!(
        store
            .enabled_project_round_policies(104)
            .unwrap()
            .is_empty()
    );

    let rebound = store
        .rebind_project_binding("project-1", "/srv/projects/one-rebound", 105)
        .unwrap();
    assert_eq!(rebound.generation, 2);
    let after_rebind = store
        .project_round_policy_for_project("project-1", 106)
        .unwrap()
        .unwrap();
    assert_eq!(after_rebind.binding_generation, Some(2));
    assert!(!after_rebind.configured);
    assert!(!after_rebind.enabled);
    assert_eq!(after_rebind.revision, 0);

    let stale = store
        .apply_project_round_mutation(
            "round-stale-1",
            ProjectRoundOperation::Enable,
            "project-1",
            1,
            107,
        )
        .unwrap();
    assert_eq!(
        stale.reason,
        Some(ProjectRoundRefusal::BindingGenerationMismatch)
    );
    let missing = store
        .apply_project_round_mutation(
            "round-missing",
            ProjectRoundOperation::Enable,
            "project-missing",
            1,
            108,
        )
        .unwrap();
    assert_eq!(missing.reason, Some(ProjectRoundRefusal::ProjectNotFound));

    store
        .apply_project_admin_mutation(
            "disable-project-2",
            ProjectAdminOperation::Disable,
            "project-2",
            None,
            109,
        )
        .unwrap();
    let inactive = store
        .apply_project_round_mutation(
            "round-inactive",
            ProjectRoundOperation::Enable,
            "project-2",
            1,
            110,
        )
        .unwrap();
    assert_eq!(inactive.reason, Some(ProjectRoundRefusal::ProjectInactive));

    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_081_dernier_dispatch_ronde_est_monotone_et_isole_par_generation() {
    use bridget_transport::protocol::ProjectRoundDispatchState;

    let path =
        std::env::temp_dir().join(format!("bridget-project-round-081-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    store
        .bind_project_registration("register-round-081", "project-081", "/srv/081", 100)
        .unwrap();
    store
        .apply_project_round_mutation(
            "round-enable-081",
            ProjectRoundOperation::Enable,
            "project-081",
            1,
            101,
        )
        .unwrap();

    store
        .record_project_round_dispatch(
            "project-081",
            1,
            840,
            ProjectRoundDispatchState::Deposited,
            850,
        )
        .unwrap();
    store
        .record_project_round_dispatch(
            "project-081",
            1,
            420,
            ProjectRoundDispatchState::Refused,
            851,
        )
        .unwrap();
    let projection = store
        .project_round_policy_for_project("project-081", 852)
        .unwrap()
        .unwrap();
    assert_eq!(projection.last_occurrence_at, Some(840));
    assert_eq!(
        projection.last_dispatch_state,
        Some(ProjectRoundDispatchState::Deposited)
    );
    assert_eq!(projection.last_dispatch_observed_at, Some(850));

    store
        .rebind_project_binding("project-081", "/srv/081-rebound", 900)
        .unwrap();
    let rebound = store
        .project_round_policy_for_project("project-081", 901)
        .unwrap()
        .unwrap();
    assert_eq!(rebound.binding_generation, Some(2));
    assert!(!rebound.configured);
    assert_eq!(rebound.last_occurrence_at, None);
    assert_eq!(rebound.last_dispatch_state, None);
    assert_eq!(rebound.last_dispatch_observed_at, None);

    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_081_migre_une_table_de_politique_079_sans_reecriture() {
    let path = std::env::temp_dir().join(format!(
        "bridget-project-round-migration-081-{}.db",
        Uuid::new_v4()
    ));
    let connection = Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE project_round_policies (
                    project_id TEXT NOT NULL,
                    binding_generation INTEGER NOT NULL,
                    enabled INTEGER NOT NULL,
                    revision INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    command_id TEXT NOT NULL,
                    PRIMARY KEY (project_id, binding_generation)
                );",
        )
        .unwrap();
    drop(connection);

    let store = Store::open(&path).unwrap();
    let columns = store
        .conn
        .prepare("PRAGMA table_info(project_round_policies)")
        .unwrap()
        .query_map([], |row| row.get::<_, String>(1))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(columns.contains(&"last_occurrence_at".to_string()));
    assert!(columns.contains(&"last_dispatch_state".to_string()));
    assert!(columns.contains(&"last_dispatch_observed_at".to_string()));

    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_065_liaison_projet_path_missing_rebind_et_audit_deterministe() {
    let path = std::env::temp_dir().join(format!("bridget-project-binding-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    let binding = ProjectBinding::active(
        "project-1".to_string(),
        "/srv/projects/one".to_string(),
        ProjectBackend::Host,
        100,
    )
    .unwrap();
    store.insert_project_binding(&binding).unwrap();

    let missing = store
        .mark_project_binding_path_missing("project-1", 101)
        .unwrap();
    assert_eq!(missing.state, ProjectBindingState::PathMissing);
    assert_eq!(missing.generation, 1);
    assert_eq!(
        missing.last_reason,
        Some(ProjectRegistryRefusal::RootMissing)
    );

    let rebound = store
        .rebind_project_binding("project-1", "/srv/projects/two", 102)
        .unwrap();
    assert_eq!(rebound.state, ProjectBindingState::Active);
    assert_eq!(rebound.canonical_root, "/srv/projects/two");
    assert_eq!(rebound.generation, 2);

    let audit = ProjectAuditEvent::for_mutation(
        "command-rebind",
        "project-1",
        ProjectAuditOperation::Rebind,
        rebound.generation,
        ProjectAuditOutcome::Applied,
        Some("/srv/projects/one"),
        102,
    );
    assert!(store.record_project_audit_event(&audit).unwrap());
    assert!(!store.record_project_audit_event(&audit).unwrap());
    let audits = store.project_audit_events("project-1").unwrap();
    assert_eq!(audits, vec![audit]);
    assert!(
        !audits[0]
            .previous_root_reference
            .as_deref()
            .unwrap_or_default()
            .contains("/srv/projects/one")
    );

    drop(store);
    let _ = std::fs::remove_file(path);
}
#[test]
fn spec_066_liaison_docker_est_atomique_et_rejoue_exactement_la_meme_issue() {
    let path = std::env::temp_dir().join(format!(
        "bridget-project-runtime-idempotency-{}.db",
        Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    let runtime = ProjectRuntimeBinding {
        state: ProjectEnvironmentState::Absent,
        policy_id: "fixture".to_string(),
        policy_version: 1,
        policy_digest: format!("sha256:{}", "a".repeat(64)),
        image_reference: format!("sha256:{}", "b".repeat(64)),
        resolved_image_id: None,
        run_as_uid: 1002,
        run_as_gid: 1002,
        environment_epoch: 1,
        topology_digest: "sha256:fixture".to_string(),
        container_id: None,
        last_reason: None,
    };
    let first = store
        .bind_project_docker_registration(
            "docker-command-1",
            "project-docker",
            "/srv/projects/docker",
            runtime.clone(),
            10,
        )
        .unwrap();
    let replay = store
        .bind_project_docker_registration(
            "docker-command-1",
            "project-docker",
            "/srv/projects/docker",
            runtime.clone(),
            11,
        )
        .unwrap();
    assert_eq!(replay, first);
    assert_eq!(
        store.project_audit_events("project-docker").unwrap().len(),
        1
    );
    let mut changed = runtime;
    changed.policy_version = 2;
    assert!(matches!(
        store.bind_project_docker_registration(
            "docker-command-1",
            "project-docker",
            "/srv/projects/docker",
            changed,
            12,
        ),
        Err(StoreError::ProjectRegistryRefusal(
            ProjectRegistryRefusal::EnvelopeMismatch
        ))
    ));
    drop(store);

    let _ = std::fs::remove_file(path);
}
#[test]
fn spec_066_rebind_docker_invalide_l_admission_sans_tuer_les_agents_existants() {
    let path = std::env::temp_dir().join(format!("bridget-runtime-rebind-{}.db", Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let mut store = Store::open(&path).unwrap();
    let runtime = ProjectRuntimeBinding {
        state: ProjectEnvironmentState::Running,
        policy_id: "fixture".to_string(),
        policy_version: 1,
        policy_digest: format!("sha256:{}", "a".repeat(64)),
        image_reference: format!("sha256:{}", "b".repeat(64)),
        resolved_image_id: Some(format!("sha256:{}", "c".repeat(64))),
        run_as_uid: 1002,
        run_as_gid: 1002,
        environment_epoch: 3,
        topology_digest: "sha256:fixture".to_string(),
        container_id: Some("d".repeat(64)),
        last_reason: None,
    };
    store
        .bind_project_docker_registration(
            "register-docker",
            "project-docker",
            "/srv/projects/old",
            runtime.clone(),
            10,
        )
        .unwrap();
    let outcome = store
        .apply_project_admin_mutation(
            "rebind-docker",
            ProjectAdminOperation::Rebind,
            "project-docker",
            Some("/srv/projects/new"),
            11,
        )
        .unwrap();
    assert_eq!(outcome.bindings[0].binding_generation, Some(2));
    let rebound = store.project_binding("project-docker").unwrap().unwrap();
    let rebound_runtime = rebound.runtime.unwrap();
    assert_eq!(
        rebound_runtime.state,
        ProjectEnvironmentState::RecreateRequired
    );
    assert_eq!(
        rebound_runtime.environment_epoch,
        runtime.environment_epoch + 1
    );
    assert_eq!(rebound_runtime.container_id, runtime.container_id);
    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_065_enregistrement_projet_est_atomique_rejouable_et_audite() {
    let path = std::env::temp_dir().join(format!(
        "bridget-project-registration-{}.db",
        Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();

    let accepted = store
        .bind_project_registration("command-1", "project-1", "/srv/projects/one", 100)
        .unwrap();
    assert_eq!(accepted.status, ProjectBindStatus::Active);
    assert_eq!(accepted.binding_generation, Some(1));
    assert_eq!(accepted.backend, Some(ProjectBackend::Host));
    assert_eq!(
        store
            .bind_project_registration("command-1", "project-1", "/srv/projects/one", 101)
            .unwrap(),
        accepted,
        "le rejet après crash relit l'issue initiale exacte"
    );
    assert_eq!(store.project_audit_events("project-1").unwrap().len(), 1);

    let collision = store
        .bind_project_registration("command-2", "project-2", "/srv/projects/one", 102)
        .unwrap();
    assert_eq!(collision.status, ProjectBindStatus::RegistrationConflict);
    assert_eq!(collision.existing_project_id.as_deref(), Some("project-1"));
    assert_eq!(collision.existing_binding_generation, Some(1));

    let rebind_required = store
        .bind_project_registration("command-3", "project-1", "/srv/projects/two", 103)
        .unwrap();
    assert_eq!(rebind_required.status, ProjectBindStatus::BindingFailed);
    assert_eq!(
        rebind_required.reason,
        Some(ProjectRegistryRefusal::RebindRequired)
    );

    drop(store);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec_065_rebind_et_disable_sont_idempotents_audites_et_non_destructifs() {
    let path = std::env::temp_dir().join(format!("bridget-project-admin-{}.db", Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();
    store
        .bind_project_registration("register-1", "project-1", "/srv/projects/one", 100)
        .unwrap();

    let rebound = store
        .apply_project_admin_mutation(
            "rebind-1",
            ProjectAdminOperation::Rebind,
            "project-1",
            Some("/srv/projects/two"),
            101,
        )
        .unwrap();
    assert_eq!(rebound.bindings[0].state, ProjectBindingStatus::Active);
    assert_eq!(rebound.bindings[0].binding_generation, Some(2));
    assert_eq!(
        store
            .apply_project_admin_mutation(
                "rebind-1",
                ProjectAdminOperation::Rebind,
                "project-1",
                Some("/srv/projects/two"),
                102,
            )
            .unwrap(),
        rebound,
        "un rejeu de rebind relit l'issue plutôt que de créer un audit"
    );
    let collision = store
        .apply_project_admin_mutation(
            "rebind-collision",
            ProjectAdminOperation::Rebind,
            "project-1",
            Some("/srv/projects/two"),
            103,
        )
        .unwrap();
    assert!(collision.reason.is_none(), "même racine = no-op idempotent");

    let disabled = store
        .apply_project_admin_mutation(
            "disable-1",
            ProjectAdminOperation::Disable,
            "project-1",
            None,
            104,
        )
        .unwrap();
    assert_eq!(disabled.bindings[0].state, ProjectBindingStatus::Disabled);
    assert_eq!(
        store
            .apply_project_admin_mutation(
                "disable-1",
                ProjectAdminOperation::Disable,
                "project-1",
                None,
                105,
            )
            .unwrap(),
        disabled
    );
    let audits = store.project_audit_events("project-1").unwrap();
    assert_eq!(
        audits.len(),
        3,
        "register, rebind, disable exactement une fois"
    );
    assert_eq!(audits[1].operation, ProjectAuditOperation::Rebind);
    assert_eq!(audits[2].operation, ProjectAuditOperation::Disable);
    assert!(audits.iter().all(|audit| {
        !audit
            .previous_root_reference
            .as_deref()
            .unwrap_or_default()
            .contains("/srv/projects")
    }));
    drop(store);
    let _ = std::fs::remove_file(path);
}
#[test]
fn spec_066_switch_vers_host_invalide_l_environnement_docker() {
    let path = std::env::temp_dir().join(format!("bridget-runtime-switch-{}.db", Uuid::new_v4()));
    let _ = std::fs::remove_file(&path);
    let mut store = Store::open(&path).unwrap();
    let runtime = ProjectRuntimeBinding {
        state: ProjectEnvironmentState::Absent,
        policy_id: "fixture".to_string(),
        policy_version: 1,
        policy_digest: format!("sha256:{}", "a".repeat(64)),
        image_reference: format!("sha256:{}", "b".repeat(64)),
        resolved_image_id: None,
        run_as_uid: 1002,
        run_as_gid: 1002,
        environment_epoch: 4,
        topology_digest: "sha256:fixture".to_string(),
        container_id: None,
        last_reason: None,
    };
    store
        .bind_project_docker_registration(
            "register-switch",
            "project-switch",
            "/srv/projects/switch",
            runtime,
            100,
        )
        .unwrap();

    let switched = store
        .switch_project_backend_to_host("project-switch", 101)
        .unwrap();
    assert_eq!(switched.backend, ProjectBackend::Host);
    assert_eq!(switched.generation, 2);
    assert_eq!(switched.runtime, None);
    drop(store);
    let _ = std::fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// Session 107 — journal WAL : lecteurs jamais bloqués par une validation
// ---------------------------------------------------------------------------

fn spec107_path(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bridget-spec107-{label}-{}",
        Uuid::new_v4().simple()
    ));
    bridget_transport::fsutil::create_private_dir(&root).unwrap();
    root.join("bridget.db")
}

fn journal_mode(conn: &Connection) -> String {
    conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap()
}

fn file_mode(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).unwrap().mode() & 0o777
}

#[test]
fn spec107_base_neuve_en_wal_et_base_memoire_acceptee() {
    let path = spec107_path("neuve");
    let store = Store::open(&path).unwrap();
    assert_eq!(journal_mode(store.connection()), "wal");
    // Une base en mémoire ne connaît pas WAL : l'ouverture reste acceptée.
    let memory = Store::open(Path::new(":memory:")).unwrap();
    assert_eq!(journal_mode(memory.connection()), "memory");
}

#[test]
fn spec107_conversion_base_delete_sans_perte_et_idempotente() {
    let path = spec107_path("conversion");
    {
        // Base historique en mode rollback (delete), 10 000 échanges.
        let legacy = Connection::open(&path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE ledger (id TEXT NOT NULL, ts INTEGER NOT NULL, sender TEXT NOT NULL, target TEXT NOT NULL, body TEXT NOT NULL, conversation_key TEXT NOT NULL, PRIMARY KEY (id, target));
                 BEGIN;",
            )
            .unwrap();
        let mut insert = legacy
            .prepare("INSERT INTO ledger VALUES (?1, ?2, 'a', 'b', 'corps', 'a|b')")
            .unwrap();
        for i in 0..10_000i64 {
            insert
                .execute(params![format!("m-{i:05}"), 1_700_000_000 + i])
                .unwrap();
        }
        drop(insert);
        legacy.execute_batch("COMMIT").unwrap();
        assert_eq!(journal_mode(&legacy), "delete");
    }
    let checksum = |conn: &Connection| -> (i64, i64, String) {
        conn.query_row("SELECT count(*), sum(ts), max(id) FROM ledger", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
    };
    let before = checksum(&Connection::open(&path).unwrap());
    for _ in 0..2 {
        let store = Store::open(&path).unwrap();
        assert_eq!(journal_mode(store.connection()), "wal");
        assert_eq!(
            checksum(store.connection()),
            before,
            "aucune perte à la conversion"
        );
    }
    // Une connexion tierce ouverte ensuite hérite du mode persistant.
    assert_eq!(journal_mode(&Connection::open(&path).unwrap()), "wal");
}

#[test]
fn spec107_fichiers_auxiliaires_prives_comme_la_base() {
    use std::os::unix::fs::OpenOptionsExt;
    let path = spec107_path("droits");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    let store = Store::open(&path).unwrap();
    let message = bridget_core::BridgetMessage::new("a", "b", "corps privé");
    store.record_message(&message, "a|b").unwrap();
    let wal = path.with_file_name("bridget.db-wal");
    let shm = path.with_file_name("bridget.db-shm");
    assert!(
        wal.exists() && shm.exists(),
        "fichiers auxiliaires WAL attendus"
    );
    assert_eq!(file_mode(&path), 0o600);
    assert_eq!(file_mode(&wal), 0o600, "journal WAL privé");
    assert_eq!(file_mode(&shm), 0o600, "index partagé privé");
}

#[test]
fn spec107_lecteur_lecture_seule_jamais_bloque_par_ecriture_ni_validation() {
    let path = spec107_path("lecteur");
    let store = Store::open(&path).unwrap();
    store
        .record_message(
            &bridget_core::BridgetMessage::new("a", "b", "amorce"),
            "a|b",
        )
        .unwrap();
    // Lecteur 104 : lecture seule, aucune attente (busy_timeout nul) pour que
    // tout SQLITE_BUSY soit visible immédiatement.
    let reader = Store::open_read_only(&path).unwrap();
    reader.busy_timeout(std::time::Duration::ZERO).unwrap();
    let count = |conn: &Connection| -> rusqlite::Result<i64> {
        conn.query_row("SELECT count(*) FROM ledger", [], |row| row.get(0))
    };
    // 1. Transaction d'écriture ouverte, non validée : le lecteur lit l'état
    //    validé précédent, sans erreur.
    store.connection().execute_batch("BEGIN IMMEDIATE").unwrap();
    store
        .connection()
        .execute(
            "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES ('x', 1, 'a', 'b', ?1, 'a|b')",
            params!["y".repeat(64 * 1024)],
        )
        .unwrap();
    for _ in 0..100 {
        assert_eq!(
            count(&reader).unwrap(),
            1,
            "lecture pendant une écriture ouverte"
        );
    }
    store.connection().execute_batch("COMMIT").unwrap();
    assert_eq!(count(&reader).unwrap(), 2, "validation visible");
    // 2. Validations répétées (fsync) pendant des lectures concurrentes :
    //    0 SQLITE_BUSY côté lecteur.
    let path_writer = path.clone();
    let writer = std::thread::spawn(move || {
        let store = Store::open(&path_writer).unwrap();
        for i in 0..50 {
            let mut message = bridget_core::BridgetMessage::new("a", "b", "z".repeat(64 * 1024));
            message.id = format!("w-{i}");
            store.record_message(&message, "a|b").unwrap();
        }
    });
    let mut busy = 0;
    let mut reads = 0;
    while !writer.is_finished() || reads < 200 {
        match count(&reader) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == rusqlite::ErrorCode::DatabaseBusy
                    || error.code == rusqlite::ErrorCode::DatabaseLocked =>
            {
                busy += 1
            }
            Err(other) => panic!("erreur inattendue : {other}"),
        }
        reads += 1;
        if reads > 100_000 {
            break;
        }
    }
    writer.join().unwrap();
    assert_eq!(
        busy, 0,
        "aucun SQLITE_BUSY côté lecteur sur {reads} lectures"
    );
    assert_eq!(count(&reader).unwrap(), 52);
}
