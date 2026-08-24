use maicie::bridget_client::IdempotencyIssue;
use maicie::domain::{
    ClasseDuree, Delegation, EtatDecision, EtatDelegation, EtatObjectif, EtatOutboxDelegation,
    ModeObjectif, ObjectifCoordonne, OutboxDelegation, TypeDecision,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::{LocalFailureReason, MaicieStore};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const OBJECTIVE_ID: &str = "51000000-0000-4000-8000-000000000001";
const DELEGATION_ID: &str = "52000000-0000-4000-8000-000000000002";
const MESSAGE_ID: &str = "53000000-0000-4000-8000-000000000003";
const ISSUED_AT: i64 = 1_000;

#[test]
fn refus_terminal_fait_evaluer_l_objectif_et_la_delegation_dans_le_meme_audit() {
    let fixture = Fixture::new("rejected");
    let mut store = fixture.store();
    let prepared = prepared(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let issue = IdempotencyIssue::Rejected {
        category: "policy".to_string(),
        reason: "refus attesté".to_string(),
        expires_at: 1_100,
    };
    store
        .record_lookup_issue(uuid(MESSAGE_ID), &issue, 1_010)
        .unwrap();
    store
        .record_lookup_issue(uuid(MESSAGE_ID), &issue, 1_010)
        .unwrap();

    let snapshot = only_snapshot(&store);
    assert_eq!(snapshot.objective.etat, EtatObjectif::AEvaluer);
    assert_eq!(snapshot.objective.mis_a_jour_at, 1_010);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::AEvaluer);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].id, uuid(MESSAGE_ID));
    assert_eq!(snapshot.decisions[0].kind, TypeDecision::ConstaterIssue);
    assert_eq!(snapshot.decisions[0].etat, EtatDecision::Appliquee);
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .outbox
            .state,
        EtatOutboxDelegation::Rejected
    );
}

#[test]
fn annulation_terminale_annule_la_delegation_sans_clore_l_objectif() {
    let fixture = Fixture::new("cancelled");
    let mut store = fixture.store();
    let prepared = prepared(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::Rejected {
                category: "cancelled".to_string(),
                reason: "annulation explicite".to_string(),
                expires_at: 1_100,
            },
            1_010,
        )
        .unwrap();

    let snapshot = only_snapshot(&store);
    assert_eq!(snapshot.objective.etat, EtatObjectif::AEvaluer);
    assert_ne!(snapshot.objective.etat, EtatObjectif::Clos);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::Annulee);
    assert_eq!(snapshot.decisions[0].kind, TypeDecision::ConstaterIssue);
}

#[test]
fn acceptation_de_livraison_ne_simule_ni_reponse_ni_cloture() {
    let fixture = Fixture::new("accepted");
    let mut store = fixture.store();
    let prepared = prepared(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::Accepted { expires_at: 1_100 },
            1_010,
        )
        .unwrap();

    let snapshot = only_snapshot(&store);
    assert_eq!(snapshot.objective.etat, EtatObjectif::EnCoordination);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::Creee);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn echec_local_terminal_est_visible_par_la_coordination_sans_rejeu() {
    let fixture = Fixture::new("local");
    let mut store = fixture.store();
    let prepared = prepared(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    store
        .record_local_failure_at(uuid(MESSAGE_ID), LocalFailureReason::FrameTooLarge, 1_010)
        .unwrap();

    let snapshot = only_snapshot(&store);
    assert_eq!(snapshot.objective.etat, EtatObjectif::AEvaluer);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::AEvaluer);
    assert_eq!(snapshot.decisions.len(), 1);
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
}

#[test]
fn echec_de_l_audit_annule_aussi_la_transition_de_l_issue_terminale() {
    let fixture = Fixture::new("atomic");
    let mut store = fixture.store();
    let prepared = prepared(store.issuer_scope());
    store.create_prepared_delegation(&prepared).unwrap();

    let connection = rusqlite::Connection::open(&fixture.database).unwrap();
    connection
        .execute_batch(
            "CREATE TRIGGER reject_terminal_decision\n\
             BEFORE INSERT ON coordination_decisions\n\
             BEGIN SELECT RAISE(ABORT, 'injection de faute'); END;",
        )
        .unwrap();
    drop(connection);

    let error = store
        .record_lookup_issue(
            uuid(MESSAGE_ID),
            &IdempotencyIssue::Rejected {
                category: "policy".to_string(),
                reason: "refus attesté".to_string(),
                expires_at: 1_100,
            },
            1_010,
        )
        .unwrap_err();
    assert!(error.to_string().contains("injection de faute"));

    let snapshot = only_snapshot(&store);
    assert_eq!(snapshot.objective.etat, EtatObjectif::EnCoordination);
    assert_eq!(snapshot.delegations[0].etat, EtatDelegation::Creee);
    assert!(snapshot.decisions.is_empty());
    assert_eq!(
        store
            .recovery_snapshot(uuid(MESSAGE_ID))
            .unwrap()
            .unwrap()
            .outbox
            .state,
        EtatOutboxDelegation::Prepared
    );
}

fn only_snapshot(store: &MaicieStore) -> maicie::store::ObjectiveSnapshot {
    let mut snapshots = store.objective_snapshots(None).unwrap();
    assert_eq!(snapshots.len(), 1);
    snapshots.remove(0)
}

fn prepared(issuer_scope: &str) -> PreparedDelegation {
    let objective = ObjectifCoordonne {
        id: uuid(OBJECTIVE_ID),
        but: "Observer une issue Bridget".to_string(),
        mode: ModeObjectif::Delegue,
        etat: EtatObjectif::EnCoordination,
        cree_at: ISSUED_AT,
        mis_a_jour_at: ISSUED_AT,
        synthese: None,
        decision_en_attente_id: None,
        suite: None,
        depends_on: Vec::new(),
        references: Vec::new(),
    };
    let delegation = Delegation {
        id: uuid(DELEGATION_ID),
        objectif_id: objective.id,
        constat_id: None,
        participant: "prospective".to_string(),
        instruction: "Vérifie une issue".to_string(),
        duree: ClasseDuree::Normale,
        etat: EtatDelegation::Creee,
        raison: "test T015a".to_string(),
    };
    let body_bytes = delegation.instruction.as_bytes().to_vec();
    let outbox = OutboxDelegation {
        message_id: uuid(MESSAGE_ID),
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_hash: stable_body_hash(&body_bytes),
        body_bytes,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_060,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_100,
    };
    PreparedDelegation::new(
        objective,
        delegation,
        outbox,
        issuer_scope,
        ISSUED_AT,
        256 * 1024,
    )
    .unwrap()
}

fn uuid(value: &str) -> Uuid {
    Uuid::parse_str(value).unwrap()
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("maicie-t015a-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            database: root.join("maicie.sqlite3"),
            root,
        }
    }

    fn store(&self) -> MaicieStore {
        MaicieStore::open(&self.database).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
