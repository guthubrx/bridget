//! Oracles routines : panne → sautee ; occurrence vivante → differee ;
//! rejeu de relève → zéro doublon sous (routine_id, bucket).

use maicie::app::DelegationCandidate;
use maicie::config::DurationClasses;
use maicie::domain::SuiteObjective;
use maicie::routines::{
    EtatOccurrence, EtatRoutine, ProposeRoutineRequest, approve_routine, bucket_for,
    evaluate_routines, propose_routine,
};
use maicie::store::MaicieStore;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-routines-{label}-{}", Uuid::new_v4()))
}

fn durations() -> DurationClasses {
    DurationClasses {
        short_secs: 30,
        normal_secs: 60,
        long_secs: 90,
    }
}

fn candidate(name: &str) -> DelegationCandidate {
    DelegationCandidate {
        name: name.to_string(),
        tags: Vec::new(),
        available: true,
        dnd: false,
    }
}

fn seed_active(store: &mut MaicieStore, now: i64, period_secs: i64) -> Uuid {
    let proposed = propose_routine(
        store,
        &ProposeRoutineRequest {
            goal: "ronde de vigilance",
            participant: "prospective",
            period_secs,
            suite: SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            now,
        },
    )
    .expect("propose");
    let approved =
        approve_routine(store, proposed.id, &proposed.template_hash, now).expect("approve");
    assert_eq!(approved.state, EtatRoutine::Active);
    approved.id
}

#[test]
fn schema_v15_pose_les_tables_routines() {
    let root = root("schema");
    let database = root.join("maicie.sqlite3");
    let store = MaicieStore::open(&database).unwrap();
    assert_eq!(store.schema_version().unwrap(), 15);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn panne_simulee_marque_les_buckets_sautee_et_arme_une_seule_occurrence_future() {
    let root = root("panne");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 420_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    // Panne : on avance de 3 buckets complets.
    let later = t0 + period * 3 + 10;
    let current = bucket_for(later, period);
    let produced = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        later,
    )
    .expect("evaluate");
    let sautees: Vec<_> = produced
        .iter()
        .filter(|occ| occ.state == EtatOccurrence::Sautee)
        .collect();
    let ouvertes: Vec<_> = produced
        .iter()
        .filter(|occ| occ.state == EtatOccurrence::Ouverte)
        .collect();
    assert!(
        !sautees.is_empty(),
        "les buckets échus doivent être tracés sautee"
    );
    assert!(
        sautees
            .iter()
            .all(|occ| occ.reason.as_deref() == Some("horloge_arretee"))
    );
    assert_eq!(
        ouvertes.len(),
        1,
        "une seule occurrence future armée, pas de rafale"
    );
    assert_eq!(ouvertes[0].bucket, current);
    assert_eq!(ouvertes[0].routine_id, routine_id);
    let status = maicie::routines::routines_status_rows(&store).unwrap();
    let row = status
        .iter()
        .find(|row| row.routine_id == routine_id)
        .expect("row");
    assert!(!row.recent_sautee.is_empty(), "visible dans status");
    assert!(row.open_occurrence.is_some());
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn occurrence_vivante_differre_le_bucket_du_sans_second_mandat() {
    let root = root("differee");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 420_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    let first_now = t0 + period + 5;
    let first = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        first_now,
    )
    .expect("first");
    assert_eq!(
        first
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte)
            .count(),
        1
    );
    let second_now = first_now + period + 5;
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        second_now,
    )
    .expect("second");
    let differees: Vec<_> = second
        .iter()
        .filter(|occ| occ.state == EtatOccurrence::Differee)
        .collect();
    assert_eq!(differees.len(), 1);
    assert_eq!(differees[0].reason.as_deref(), Some("occurrence_vivante"));
    assert_eq!(differees[0].routine_id, routine_id);
    assert!(
        second
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte)
            .count()
            == 0,
        "pas de second mandat"
    );
    let status = maicie::routines::routines_status_rows(&store).unwrap();
    let row = status
        .iter()
        .find(|row| row.routine_id == routine_id)
        .unwrap();
    assert!(!row.recent_differee.is_empty(), "differee visible status");
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejeu_de_releve_zero_doublon_sous_cle_routine_bucket() {
    let root = root("rejeu");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 420_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    let now = t0 + period + 5;
    let first = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        now,
    )
    .expect("first");
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        now,
    )
    .expect("second");
    assert!(
        second.is_empty(),
        "rejeu à même now : zéro nouvelle occurrence"
    );
    let bucket = bucket_for(now, period);
    let occ = store.load_occurrence(routine_id, bucket).unwrap();
    assert!(occ.is_some());
    assert_eq!(first.iter().filter(|occ| occ.bucket == bucket).count(), 1);
    // INSERT OR conflit : rejouer insert_occurrence sur la même clé doit refuser.
    let conflict = store.insert_occurrence(occ.as_ref().unwrap());
    assert!(matches!(
        conflict,
        Err(maicie::store::StoreError::Conflict(_))
    ));
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
