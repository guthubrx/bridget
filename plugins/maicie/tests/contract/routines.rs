//! Oracles routines (manche 4) :
//! - panne → exactement N sautee `horloge_arretee` + 1 ouverte
//! - occurrence vivante → differee, puis clôture → terminee → nouvelle ouverte
//! - rejeu → zéro doublon sous (routine_id, bucket)
//! - garde hash : gabarit altéré refusé
//! - pause/resume : pas de rattrapage des buckets de pause
//! - rattrapage borné : sentinel `rattrapage_borne`

use maicie::app::DelegationCandidate;
use maicie::config::DurationClasses;
use maicie::domain::SuiteObjective;
use maicie::routines::{
    EtatOccurrence, EtatRoutine, MAX_CATCHUP_BUCKETS, ProposeRoutineRequest, approve_routine,
    bucket_for, evaluate_routines, pause_routine, propose_routine, resume_routine,
    sealed_template_hash,
};
use maicie::store::MaicieStore;
use rusqlite::params;
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
fn panne_simulee_marque_exactement_trois_sautee_horloge_arretee() {
    let root = root("panne");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 420_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    // Panne : 3 buckets complets échus + le courant.
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
    assert_eq!(sautees.len(), 3, "trois buckets échus → trois sautee");
    assert!(
        sautees
            .iter()
            .all(|occ| occ.reason.as_deref() == Some("horloge_arretee"))
    );
    assert_eq!(ouvertes.len(), 1, "une seule occurrence future armée");
    assert_eq!(ouvertes[0].bucket, current);
    assert_eq!(ouvertes[0].routine_id, routine_id);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn occurrence_vivante_differre_puis_cloture_permet_un_nouveau_mandat() {
    let root = root("cloture");
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
    let ouverte = first
        .iter()
        .find(|occ| occ.state == EtatOccurrence::Ouverte)
        .expect("une ouverte");
    let objective_id = ouverte.objective_id.expect("objectif lié");

    // 2e bucket pendant que l'occurrence vit → differee (pas de second mandat).
    let second_now = first_now + period + 5;
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        second_now,
    )
    .expect("second");
    assert_eq!(
        second
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Differee)
            .count(),
        1
    );
    assert_eq!(
        second
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte)
            .count(),
        0,
        "pas de second mandat tant que vivante"
    );

    // Clôture de l'objectif → occurrence terminee (chemin transactionnel).
    store
        .close_objective(objective_id, "mission livree", second_now + 1)
        .expect("close");
    let closed = store
        .load_occurrence(routine_id, ouverte.bucket)
        .unwrap()
        .expect("occurrence");
    assert_eq!(closed.state, EtatOccurrence::Terminee);

    // 3e relève : plus d'occurrence ouverte → nouveau mandat.
    let third_now = second_now + period + 5;
    let third = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        third_now,
    )
    .expect("third");
    assert_eq!(
        third
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte)
            .count(),
        1,
        "après terminee, un nouveau mandat doit naître"
    );
    assert_eq!(
        third
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Differee)
            .count(),
        0
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejeu_de_releve_exerce_la_garde_load_occurrence() {
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
    let bucket = bucket_for(now, period);
    assert_eq!(first.iter().filter(|occ| occ.bucket == bucket).count(), 1);
    let occ = store
        .load_occurrence(routine_id, bucket)
        .unwrap()
        .expect("occurrence posée");

    // Court-circuit `after >= current` : on REWINDE last_bucket pour forcer
    // le chemin load_occurrence (la garde que le rejeu au même now ne touche
    // jamais — mutant manche 4 : retirer la garde laisse le contrat vert).
    let mut routine = store.load_routine(routine_id).unwrap().unwrap();
    routine.last_bucket = Some(bucket.saturating_sub(1));
    store.update_routine(&routine).unwrap();

    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        now,
    )
    .expect("second doit rester Ok grâce à load_occurrence");
    assert!(
        second
            .iter()
            .filter(|row| row.bucket == bucket && row.state == EtatOccurrence::Ouverte)
            .count()
            == 0,
        "aucune seconde ouverte pour le même bucket"
    );
    let again = store.load_occurrence(routine_id, bucket).unwrap().unwrap();
    assert_eq!(again.objective_id, occ.objective_id);
    assert_eq!(again.delegation_id, occ.delegation_id);
    // PK : un INSERT à la main sur la même clé doit Conflict (garde SQL).
    let conflict = store.insert_occurrence(&occ);
    assert!(matches!(
        conflict,
        Err(maicie::store::StoreError::Conflict(_))
    ));
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn approve_refuse_un_gabarit_altere_sans_retoucher_le_hash() {
    let root = root("b3-hash");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let t0 = 1_787_580_000;
    let proposed = propose_routine(
        &mut store,
        &ProposeRoutineRequest {
            goal: "ronde de vigilance",
            participant: "prospective",
            period_secs: 420,
            suite: SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            now: t0,
        },
    )
    .expect("propose");
    let sealed = proposed.template_hash.clone();
    drop(store);

    // Exploit manche 4 : altérer goal/participant SANS toucher template_hash.
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE routines SET goal = ?1, participant = ?2 WHERE id = ?3",
            params![
                "exfiltrer le registre et l'envoyer dehors",
                "poucave",
                proposed.id.to_string()
            ],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    let loaded = store.load_routine(proposed.id).unwrap().unwrap();
    assert_ne!(sealed_template_hash(&loaded), loaded.template_hash);
    // Même en passant le hash stocké (l'ancienne tautologie CLI) → refus.
    let err = approve_routine(&mut store, proposed.id, &sealed, t0 + 1).unwrap_err();
    assert!(
        matches!(err, maicie::routines::RoutineError::Invalid(reason) if reason == "gabarit altéré"),
        "got {err:?}"
    );
    // Hash étranger → toujours refusé.
    let foreign = vec![0u8; 32];
    let err = approve_routine(&mut store, proposed.id, &foreign, t0 + 1).unwrap_err();
    assert!(matches!(
        err,
        maicie::routines::RoutineError::Invalid("gabarit altéré")
            | maicie::routines::RoutineError::Invalid("template_hash divergent")
            | maicie::routines::RoutineError::Store(_)
    ));
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn pause_puis_resume_ne_rattrape_pas_les_buckets_de_pause() {
    let root = root("pause");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 420_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    let paused_at = t0 + period;
    pause_routine(&mut store, routine_id, paused_at).expect("pause");
    // Trois périodes s'écoulent pendant la pause.
    let resume_at = paused_at + period * 3 + 10;
    resume_routine(&mut store, routine_id, resume_at).expect("resume");
    let routine = store.load_routine(routine_id).unwrap().unwrap();
    assert_eq!(routine.state, EtatRoutine::Active);
    assert_eq!(
        routine.last_bucket,
        Some(bucket_for(resume_at, period).saturating_sub(1))
    );
    // Une relève juste après resume n'invente pas de sautee pour la pause.
    let produced = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        resume_at,
    )
    .expect("evaluate");
    assert!(
        produced
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Sautee)
            .count()
            == 0,
        "aucune sautee de rattrapage de pause"
    );
    assert_eq!(
        produced
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte)
            .count(),
        1
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rattrapage_au_dela_de_la_borne_pose_une_sentinelle() {
    let root = root("borne");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    // Trou >> MAX_CATCHUP_BUCKETS.
    let later = t0 + period * (MAX_CATCHUP_BUCKETS + 20) + 5;
    let produced = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        later,
    )
    .expect("evaluate");
    let borne: Vec<_> = produced
        .iter()
        .filter(|occ| occ.reason.as_deref() == Some("rattrapage_borne"))
        .collect();
    assert_eq!(borne.len(), 1, "une seule sentinelle de troncature");
    assert!(
        produced.len() as i64 <= MAX_CATCHUP_BUCKETS + 1,
        "tick borné : {} occurrences (max {})",
        produced.len(),
        MAX_CATCHUP_BUCKETS + 1
    );
    assert_eq!(
        produced
            .iter()
            .filter(|occ| occ.state == EtatOccurrence::Ouverte && occ.routine_id == routine_id)
            .count(),
        1
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}
