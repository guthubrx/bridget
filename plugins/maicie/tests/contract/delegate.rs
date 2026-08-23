use maicie::app::{delegate, DelegateError, DelegateRequest, DelegateResult, DelegationCandidate};
use maicie::config::DurationClasses;
use maicie::domain::ClasseDuree;
use maicie::store::MaicieStore;
use std::fs;
use std::path::PathBuf;
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

fn request<'a>(target: Option<&'a str>, tags: &'a [String], duration: ClasseDuree) -> DelegateRequest<'a> {
    DelegateRequest {
        goal: "vérifier le contrat",
        explicit_target: target,
        required_tags: tags,
        duration,
        reply: true,
        idempotency_key: Uuid::new_v4(),
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
    let DelegateResult::Created(created) = result else { panic!("création attendue") };
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
        delegate(&mut store, durations(), "maicie", &candidates, &request(None, &tags, ClasseDuree::Courte)).unwrap(),
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
        delegate(&mut store, durations(), "maicie", &candidates, &request(Some("prospective"), &[], ClasseDuree::Normale)),
        Err(DelegateError::TargetUnavailable("prospective".to_string()))
    );
    assert_eq!(
        delegate(&mut store, durations(), "maicie", &candidates, &request(Some("absent"), &[], ClasseDuree::Normale)),
        Err(DelegateError::TargetUnavailable("absent".to_string()))
    );
    assert!(store.pending_delegation_outboxes().unwrap().is_empty());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
