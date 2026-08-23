use maicie::app::{DelegateError, DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::config::DurationClasses;
use maicie::domain::ClasseDuree;
use maicie::store::MaicieStore;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-delegate-{label}-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    }
}

fn request<'a>(
    target: Option<&'a str>,
    tags: &'a [String],
    duration: ClasseDuree,
) -> DelegateRequest<'a> {
    DelegateRequest {
        goal: "vérifier le contrat",
        explicit_target: target,
        required_tags: tags,
        duration,
        reply: true,
        idempotency_key: "delegate-test-default",
        now: 100,
        retry_until: 150,
        dedup_retained_until: 200,
        max_frame_bytes: 256 * 1024,
    }
}

#[test]
fn cible_explicite_cree_objectif_delegation_et_outbox_atomiques() {
    let root = root("explicit");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec!["rust".to_string()],
        available: true,
        dnd: false,
    }];
    let result = delegate(
        &mut store,
        durations(),
        "maicie",
        &candidates,
        &request(Some("prospective"), &[], ClasseDuree::Longue),
    )
    .unwrap();
    let DelegateResult::Created(created) = result else {
        panic!("création attendue")
    };
    assert_eq!(created.participant, "prospective");
    assert_eq!(created.timeout_secs, 90);
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].message_id, created.message_id);
    assert_eq!(pending[0].timeout_secs, 90);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selection_tags_est_stricte_exclut_le_pilote_et_ne_choisit_jamais_ambigu() {
    let root = root("tags");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let tags = vec!["audit".to_string(), "rust".to_string()];
    let candidates = vec![
        DelegationCandidate {
            name: "maicie".to_string(),
            tags: tags.clone(),
            available: true,
            dnd: false,
        },
        DelegationCandidate {
            name: "prospective".to_string(),
            tags: vec!["rust".to_string(), "audit".to_string()],
            available: true,
            dnd: false,
        },
        DelegationCandidate {
            name: "sentry".to_string(),
            tags: tags.clone(),
            available: true,
            dnd: false,
        },
        DelegationCandidate {
            name: "large".to_string(),
            tags: vec!["audit".to_string(), "rust".to_string(), "ops".to_string()],
            available: true,
            dnd: false,
        },
    ];
    assert_eq!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &candidates,
            &request(None, &tags, ClasseDuree::Courte)
        )
        .unwrap(),
        DelegateResult::Candidates(vec!["prospective".to_string(), "sentry".to_string()])
    );
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dnd_et_absence_refusent_une_cible_explicite_sans_ecriture() {
    let root = root("dnd");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec![],
        available: true,
        dnd: true,
    }];
    assert_eq!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &candidates,
            &request(Some("prospective"), &[], ClasseDuree::Normale)
        ),
        Err(DelegateError::TargetUnavailable("prospective".to_string()))
    );
    assert_eq!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &candidates,
            &request(Some("absent"), &[], ClasseDuree::Normale)
        ),
        Err(DelegateError::TargetUnavailable("absent".to_string()))
    );
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cible_explicite_pilote_est_refusee_avant_toute_ecriture() {
    let root = root("pilot-explicit");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "maicie".to_string(),
        tags: vec!["coordination".to_string()],
        available: true,
        dnd: false,
    }];
    assert_eq!(
        delegate(
            &mut store,
            durations(),
            "maicie",
            &candidates,
            &request(Some("maicie"), &[], ClasseDuree::Normale),
        ),
        Err(DelegateError::TargetUnavailable("maicie".to_string()))
    );
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn meme_cle_opaque_rejoue_exactement_les_ids_sans_doublon() {
    let root = root("idempotency-replay");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec![],
        available: true,
        dnd: false,
    }];
    let mut first_request = request(Some("prospective"), &[], ClasseDuree::Normale);
    first_request.idempotency_key = "cli/K:opaque-not-a-uuid";
    let first = delegate(
        &mut store,
        durations(),
        "maicie",
        &candidates,
        &first_request,
    )
    .unwrap();
    let DelegateResult::Created(first) = first else {
        panic!("création attendue")
    };
    assert!(!first.replayed);

    let mut retry = request(Some("prospective"), &[], ClasseDuree::Normale);
    retry.idempotency_key = "cli/K:opaque-not-a-uuid";
    retry.now = 101;
    let replay = delegate(&mut store, durations(), "maicie", &[], &retry).unwrap();
    let DelegateResult::Created(replay) = replay else {
        panic!("rejeu attendu")
    };
    assert!(replay.replayed);
    assert_eq!(replay.objective_id, first.objective_id);
    assert_eq!(replay.delegation_id, first.delegation_id);
    assert_eq!(replay.message_id, first.message_id);
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cle_reutilisee_avec_enveloppe_divergente_est_refusee_sans_mutation() {
    let root = root("idempotency-mismatch");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec![],
        available: true,
        dnd: false,
    }];
    let mut first_request = request(Some("prospective"), &[], ClasseDuree::Normale);
    first_request.idempotency_key = "same-key";
    let first = delegate(
        &mut store,
        durations(),
        "maicie",
        &candidates,
        &first_request,
    )
    .unwrap();
    let DelegateResult::Created(first) = first else {
        panic!("création attendue")
    };

    let mut divergent = request(Some("prospective"), &[], ClasseDuree::Normale);
    divergent.idempotency_key = "same-key";
    divergent.goal = "instruction divergente";
    assert_eq!(
        delegate(&mut store, durations(), "maicie", &candidates, &divergent),
        Err(DelegateError::EnvelopeMismatch)
    );
    let pending = store.pending_delegation_outboxes().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].message_id, first.message_id);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cles_distinctes_creent_des_delegations_distinctes() {
    let root = root("idempotency-distinct");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let candidates = vec![DelegationCandidate {
        name: "prospective".to_string(),
        tags: vec![],
        available: true,
        dnd: false,
    }];
    let mut one = request(Some("prospective"), &[], ClasseDuree::Normale);
    one.idempotency_key = "one";
    let mut two = request(Some("prospective"), &[], ClasseDuree::Normale);
    two.idempotency_key = "two";
    let DelegateResult::Created(one) =
        delegate(&mut store, durations(), "maicie", &candidates, &one).unwrap()
    else {
        panic!("première création attendue")
    };
    let DelegateResult::Created(two) =
        delegate(&mut store, durations(), "maicie", &candidates, &two).unwrap()
    else {
        panic!("seconde création attendue")
    };
    assert_ne!(one.objective_id, two.objective_id);
    assert_ne!(one.message_id, two.message_id);
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 2);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reservations_concurrentes_rejouent_ou_refusent_sans_doublon() {
    for attempt in 0..20 {
        let root = root(&format!("idempotency-concurrent-{attempt}"));
        let database = root.join("maicie.sqlite3");
        let (first, second) = concurrent_pair(
            database.clone(),
            "même commande",
            "shared-key",
            "même commande",
            "shared-key",
        );
        let first = first.unwrap();
        let second = second.unwrap();
        let (created, replayed) = match (first, second) {
            (DelegateResult::Created(created), DelegateResult::Created(replayed))
                if !created.replayed =>
            {
                (created, replayed)
            }
            (DelegateResult::Created(replayed), DelegateResult::Created(created))
                if !created.replayed =>
            {
                (created, replayed)
            }
            results => panic!("réservation concurrente invalide : {results:?}"),
        };
        assert!(replayed.replayed);
        assert_eq!(created.objective_id, replayed.objective_id);
        assert_eq!(created.delegation_id, replayed.delegation_id);
        assert_eq!(created.message_id, replayed.message_id);
        let store = MaicieStore::open(&database).unwrap();
        assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }

    let root = root("idempotency-concurrent-divergent");
    let database = root.join("maicie.sqlite3");
    let (matching, divergent) = concurrent_pair(
        database.clone(),
        "commande identique",
        "conflicting-key",
        "commande divergente",
        "conflicting-key",
    );
    assert!(matches!(
        (matching, divergent),
        (
            Ok(DelegateResult::Created(_)),
            Err(DelegateError::EnvelopeMismatch)
        ) | (
            Err(DelegateError::EnvelopeMismatch),
            Ok(DelegateResult::Created(_))
        )
    ));
    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.pending_delegation_outboxes().unwrap().len(), 1);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

fn concurrent_pair(
    database: PathBuf,
    first_goal: &'static str,
    first_key: &'static str,
    second_goal: &'static str,
    second_key: &'static str,
) -> (
    Result<DelegateResult, DelegateError>,
    Result<DelegateResult, DelegateError>,
) {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (first_start_tx, first_start_rx) = mpsc::channel();
    let (second_start_tx, second_start_rx) = mpsc::channel();
    let first = concurrent_delegate(
        database.clone(),
        ready_tx.clone(),
        first_start_rx,
        first_goal,
        first_key,
    );
    let second = concurrent_delegate(database, ready_tx, second_start_rx, second_goal, second_key);
    for _ in 0..2 {
        ready_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("ouverture concurrente bloquée")
            .expect("ouverture concurrente échouée");
    }
    first_start_tx.send(()).unwrap();
    second_start_tx.send(()).unwrap();
    (first.join().unwrap(), second.join().unwrap())
}

fn concurrent_delegate(
    database: PathBuf,
    ready: mpsc::Sender<Result<(), String>>,
    start: mpsc::Receiver<()>,
    goal: &'static str,
    key: &'static str,
) -> thread::JoinHandle<Result<DelegateResult, DelegateError>> {
    thread::spawn(move || {
        let mut store = match MaicieStore::open(database) {
            Ok(store) => {
                ready.send(Ok(())).unwrap();
                store
            }
            Err(error) => {
                let _ = ready.send(Err(format!("{error:?}")));
                return Err(DelegateError::Store(error.to_string()));
            }
        };
        let candidates = vec![DelegationCandidate {
            name: "prospective".to_string(),
            tags: vec![],
            available: true,
            dnd: false,
        }];
        let request = DelegateRequest {
            goal,
            explicit_target: Some("prospective"),
            required_tags: &[],
            duration: ClasseDuree::Normale,
            reply: true,
            idempotency_key: key,
            now: 100,
            retry_until: 150,
            dedup_retained_until: 200,
            max_frame_bytes: 256 * 1024,
        };
        start
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| DelegateError::Store("départ concurrent non reçu".to_string()))?;
        delegate(&mut store, durations(), "maicie", &candidates, &request)
    })
}
