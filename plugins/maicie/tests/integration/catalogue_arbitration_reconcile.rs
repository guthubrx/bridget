use maicie::app::{reconcile_catalogue_from_store, unix_secs_to_rfc3339_z};
use maicie::catalogue::{
    AddEntry, CatalogueEntry, CatalogueJournal, MissionSource, MissionSourceKind, Severity,
    TransitionTrigger, project_registre,
};
use maicie::domain::{
    ClasseDuree, Delegation, EtatObjectif, EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne,
    OutboxDelegation,
};
use maicie::outbox::{PreparedDelegation, stable_body_hash};
use maicie::store::MaicieStore;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use uuid::Uuid;

const FRAME_LIMIT: usize = 256 * 1024;

#[test]
fn unix_secs_to_rfc3339_z_est_deterministe_et_utc() {
    assert_eq!(
        unix_secs_to_rfc3339_z(0).as_deref(),
        Some("1970-01-01T00:00:00Z")
    );
    assert_eq!(
        unix_secs_to_rfc3339_z(1_724_457_600).as_deref(),
        Some("2024-08-24T00:00:00Z")
    );
    assert!(unix_secs_to_rfc3339_z(-1).is_none());
}

#[test]
fn reconcile_depuis_store_produit_une_transition_unique_au_rejeu() {
    let fixture = Fixture::new("reconcile");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let linked = prepared(&store, "objectif lié", Some("c-open"));
    let ordinary = prepared(&store, "objectif ordinaire", None);
    store.create_prepared_delegation(&linked).unwrap();
    store.create_prepared_delegation(&ordinary).unwrap();
    store
        .close_objective(linked.objective.id, "fait attesté", 1_724_457_600)
        .unwrap();
    store
        .close_objective(ordinary.objective.id, "sans lien", 1_724_457_700)
        .unwrap();

    let mut journal = CatalogueJournal::open(&fixture.catalogue).unwrap();
    journal
        .append_add(sample_add("c-open", "2026-08-24T06:00:00Z"))
        .unwrap();

    let first = reconcile_catalogue_from_store(&store, &mut journal).unwrap();
    assert_eq!(first.appended, 1);
    assert_eq!(first.skipped, 0);

    let replay = reconcile_catalogue_from_store(&store, &mut journal).unwrap();
    assert_eq!(replay.appended, 0);
    assert_eq!(replay.skipped, 1);

    let entries = journal.read_entries().unwrap();
    let transitions: Vec<_> = entries
        .iter()
        .filter_map(|entry| match entry {
            CatalogueEntry::Transition(transition) => Some(transition),
            _ => None,
        })
        .collect();
    assert_eq!(transitions.len(), 1);
    assert_eq!(transitions[0].constat_id, "c-open");
    assert_eq!(transitions[0].objective_id, linked.objective.id.to_string());
    assert_eq!(transitions[0].observed_at, "2024-08-24T00:00:00Z");
    assert_eq!(transitions[0].trigger, TransitionTrigger::ObjectiveClosed);

    let view = project_registre(&entries);
    assert_eq!(view.ouverts.len(), 0);
}

#[test]
fn cloture_sans_lien_et_lien_sans_cloture_n_ecrivent_rien() {
    let fixture = Fixture::new("orphan");
    let mut store = MaicieStore::open(&fixture.database).unwrap();
    let waiting = prepared(&store, "en attente", Some("c-wait"));
    let orphan = prepared(&store, "clos sans lien", None);
    store.create_prepared_delegation(&waiting).unwrap();
    store.create_prepared_delegation(&orphan).unwrap();
    store
        .close_objective(orphan.objective.id, "sans lien", 1_100)
        .unwrap();

    let mut journal = CatalogueJournal::open(&fixture.catalogue).unwrap();
    journal
        .append_add(sample_add("c-wait", "2026-08-24T06:00:00Z"))
        .unwrap();

    let report = reconcile_catalogue_from_store(&store, &mut journal).unwrap();
    assert_eq!(report.appended, 0);
    assert_eq!(report.skipped, 0);
    assert_eq!(journal.read_entries().unwrap().len(), 1);
}

fn sample_add(id: &str, date: &str) -> AddEntry {
    AddEntry {
        v: 1,
        kind: maicie::catalogue::AddKind::Add,
        id: id.into(),
        date: date.into(),
        mission_source: MissionSource {
            kind: MissionSourceKind::Review,
            id: "r-1".into(),
            failed: None,
        },
        severity: Severity::Major,
        text: "constat ouvert".into(),
        recurrence_of: None,
    }
}

fn prepared(store: &MaicieStore, goal: &str, constat_id: Option<&str>) -> PreparedDelegation {
    let mut objective = ObjectifCoordonne::nouveau(goal, ModeObjectif::Delegue, 1_000).unwrap();
    objective
        .transition(EtatObjectif::EnCoordination, 1_001)
        .unwrap();
    let delegation = Delegation::nouvelle(
        objective.id,
        "agent",
        goal,
        ClasseDuree::Normale,
        "arbitrage humain",
    )
    .and_then(|delegation| match constat_id {
        Some(id) => delegation.pour_constat(id),
        None => Ok(delegation),
    })
    .unwrap();
    let body = goal.as_bytes().to_vec();
    let outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_hash: stable_body_hash(&body),
        body_bytes: body,
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 1_061,
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 1_050,
        dedup_retained_until: 1_100,
    };
    PreparedDelegation::new(
        objective,
        delegation,
        outbox,
        store.issuer_scope(),
        1_001,
        FRAME_LIMIT,
    )
    .unwrap()
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    catalogue: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-arbitration-{label}-{}",
            Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.db");
        let catalogue = root.join("catalogue.jsonl");
        fs::write(&catalogue, "").unwrap();
        Self {
            root,
            database,
            catalogue,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
