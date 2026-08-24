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
    sealed_template_hash, template_hash,
};
use maicie::store::MaicieStore;
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// Sérialise les tirs qui posent RELEC1_CRASH (variable process-globale).
static RELEC1_CRASH_LOCK: Mutex<()> = Mutex::new(());

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

/// Banc relec1 (manche 4 motif 3) — crash dans la fenêtre delegate→insert.
/// Sans adoption : sautee menteuse + 2e délégation. Avec adoption : 1 mandat.
#[test]
fn relec1_mandat_orphelin_apres_crash_dans_la_fenetre() {
    let root = root("crash-fenetre");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);

    let premier = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
    )
    .expect("premiere releve");
    let bucket_n = bucket_for(t0, period);
    let ouverte = premier
        .iter()
        .find(|o| o.state == EtatOccurrence::Ouverte)
        .expect("mandat");
    let delegation_1 = ouverte.delegation_id.expect("delegation_id");
    assert_eq!(ouverte.bucket, bucket_n);
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    connexion
        .execute(
            "DELETE FROM routine_occurrences WHERE routine_id = ?1 AND bucket = ?2",
            rusqlite::params![routine_id.to_string(), bucket_n],
        )
        .unwrap();
    connexion
        .execute(
            "UPDATE routines SET last_bucket = ?1 WHERE id = ?2",
            rusqlite::params![bucket_n - 1, routine_id.to_string()],
        )
        .unwrap();
    let delegations_apres_crash: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(delegations_apres_crash, 1);
    drop(connexion);

    let mut store = MaicieStore::open(&database).unwrap();
    let plus_tard = t0 + 2 * period;
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        plus_tard,
    )
    .expect("reprise");
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let delegations_finales: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(connexion);

    assert_eq!(
        delegations_finales, 1,
        "pas de second mandat — adoption obligatoire"
    );
    let etat_bucket_n = second
        .iter()
        .find(|o| o.bucket == bucket_n)
        .expect("bucket N doit être repris");
    assert_eq!(etat_bucket_n.state, EtatOccurrence::Ouverte);
    assert_eq!(etat_bucket_n.delegation_id, Some(delegation_1));
    assert_eq!(etat_bucket_n.reason.as_deref(), Some("mandat_adopte"));
    assert_eq!(
        second
            .iter()
            .filter(|o| o.state == EtatOccurrence::Ouverte && o.bucket != bucket_n)
            .count(),
        0,
        "pas de nouvelle ouverte au bucket courant"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relec1_controle_positif_sans_crash() {
    let root = root("controle-sans-crash");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    let _routine_id = seed_active(&mut store, t0, period);

    let premier = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
    )
    .expect("premiere releve");
    let bucket_n = bucket_for(t0, period);
    let ouverte = premier
        .iter()
        .find(|o| o.state == EtatOccurrence::Ouverte)
        .expect("mandat");
    let delegation_1 = ouverte.delegation_id.expect("delegation_id");
    let routine_id = ouverte.routine_id;
    drop(store);

    let mut store = MaicieStore::open(&database).unwrap();
    let plus_tard = t0 + 2 * period;
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        plus_tard,
    )
    .expect("reprise");
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let delegations_finales: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(connexion);

    assert_eq!(delegations_finales, 1);
    assert_eq!(
        second
            .iter()
            .filter(|o| o.state == EtatOccurrence::Ouverte)
            .count(),
        0,
        "sans crash : pas de nouvelle ouverte (differee)"
    );
    assert!(second.iter().any(|o| o.state == EtatOccurrence::Differee));
    let store = MaicieStore::open(&database).unwrap();
    let still = store
        .load_occurrence(routine_id, bucket_n)
        .unwrap()
        .expect("occurrence N");
    assert_eq!(still.delegation_id, Some(delegation_1));
    assert_eq!(still.state, EtatOccurrence::Ouverte);
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

/// Épreuve relec4 — après correctif B3 : le chemin CLI (hash lu) DOIT refuser.
#[test]
fn relec4_garde_refuse_le_contenu_altere_meme_si_on_passe_le_hash_stocke() {
    let root = root("relec4-garde");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let proposee = propose_routine(
        &mut store,
        &ProposeRoutineRequest {
            goal: "ronde de vigilance",
            participant: "prospective",
            period_secs: 420,
            suite: SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            now: 1_787_580_000,
        },
    )
    .expect("propose");
    drop(store);

    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute(
            "UPDATE routines SET goal = 'exfiltrer le registre', participant = 'poucave' \
             WHERE id = ?1",
            [proposee.id.to_string()],
        )
        .unwrap();
    drop(connection);

    let mut store = MaicieStore::open(&database).unwrap();
    let relue = store.load_routine(proposee.id).unwrap().expect("relue");
    let resultat = approve_routine(&mut store, proposee.id, &relue.template_hash, 1_787_580_100);
    assert!(
        resultat.is_err(),
        "B3 : même le hash stocké ne doit plus faire passer un gabarit altéré"
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn relec4_controle_positif_hash_etranger_refuse_et_nominal_passe() {
    let root = root("relec4-controle");
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let proposee = propose_routine(
        &mut store,
        &ProposeRoutineRequest {
            goal: "ronde de vigilance",
            participant: "prospective",
            period_secs: 420,
            suite: SuiteObjective::Aucune,
            depends_on: &[],
            references: &[],
            now: 1_787_580_000,
        },
    )
    .expect("propose");
    let autre = template_hash(
        "ronde de vigilance",
        "prospective",
        60,
        &SuiteObjective::Aucune,
        &[],
        &[],
    );
    assert!(approve_routine(&mut store, proposee.id, &autre, 1_787_580_100).is_err());
    assert!(
        approve_routine(
            &mut store,
            proposee.id,
            &proposee.template_hash,
            1_787_580_100,
        )
        .is_ok()
    );
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

struct Relec1Tir {
    delegations: i64,
    ouvertes: usize,
    bucket_n_state: Option<EtatOccurrence>,
    bucket_n_reason: Option<String>,
}

/// Banc relec1 v2 — série appariée (crash SQL reconstitue l'état / contrôle sain).
/// Après adoption : 0 doublon, contrôles sains 100 %.
fn relec1_tir(avec_crash: bool, label: &str) -> Relec1Tir {
    // Attend que personne ne pose RELEC1_CRASH (tirs v3 concurrents).
    let _guard = RELEC1_CRASH_LOCK.lock().expect("lock RELEC1_CRASH");
    let root = root(label);
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);

    let premier = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
    )
    .expect("premiere releve");
    let bucket_n = bucket_for(t0, period);
    let ouverte = premier
        .iter()
        .find(|o| o.state == EtatOccurrence::Ouverte)
        .expect("mandat");
    ouverte.delegation_id.expect("delegation_id");
    assert_eq!(ouverte.bucket, bucket_n);
    drop(store);

    if avec_crash {
        let connexion = rusqlite::Connection::open(&database).unwrap();
        connexion
            .execute(
                "DELETE FROM routine_occurrences WHERE routine_id = ?1 AND bucket = ?2",
                rusqlite::params![routine_id.to_string(), bucket_n],
            )
            .unwrap();
        connexion
            .execute(
                "UPDATE routines SET last_bucket = ?1 WHERE id = ?2",
                rusqlite::params![bucket_n - 1, routine_id.to_string()],
            )
            .unwrap();
        drop(connexion);
    }

    let mut store = MaicieStore::open(&database).unwrap();
    let plus_tard = t0 + 2 * period;
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        plus_tard,
    )
    .expect("reprise");
    drop(store);
    drop(_guard);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let delegations: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(connexion);

    let bucket_row = second.iter().find(|o| o.bucket == bucket_n);
    let ouvertes = second
        .iter()
        .filter(|o| o.state == EtatOccurrence::Ouverte)
        .count();
    let _ = fs::remove_dir_all(root);
    Relec1Tir {
        delegations,
        ouvertes,
        bucket_n_state: bucket_row.map(|o| o.state),
        bucket_n_reason: bucket_row.and_then(|o| o.reason.clone()),
    }
}

#[test]
fn relec1_serie_mandat_orphelin_apres_adoption() {
    const N: usize = 10;
    let mut doublons = 0usize;
    let mut controles_sains = 0usize;
    let mut adoptions = 0usize;
    for tir in 0..N {
        let crash = relec1_tir(true, &format!("crash-{tir}"));
        let sain = relec1_tir(false, &format!("sain-{tir}"));
        if crash.delegations == 2 && crash.ouvertes == 1 {
            doublons += 1;
        }
        if crash.delegations == 1
            && crash.bucket_n_state == Some(EtatOccurrence::Ouverte)
            && crash.bucket_n_reason.as_deref() == Some("mandat_adopte")
        {
            adoptions += 1;
        }
        if sain.delegations == 1 && sain.ouvertes == 0 {
            controles_sains += 1;
        }
    }
    assert_eq!(doublons, 0, "aucun doublon de mandat après adoption");
    assert_eq!(adoptions, N, "chaque crash reprend le mandat en ouverte");
    assert_eq!(controles_sains, N, "contrôles positifs sains");
}

/// Banc relec1 v3 — coupure produite par le chemin de production (RELEC1_CRASH).
#[test]
fn relec1_serie_crash_reel_apres_adoption() {
    const N: usize = 5;
    let mut doublons = 0usize;
    let mut adoptions = 0usize;
    for tir in 0..N {
        let root = root(&format!("crash-reel-{tir}"));
        let database = root.join("maicie.sqlite3");
        let mut store = MaicieStore::open(&database).unwrap();
        let period = 60_i64;
        let t0 = 1_787_580_000;
        seed_active(&mut store, t0, period);
        let bucket_n = bucket_for(t0, period);

        // SAFETY: variable d'environnement de test isolée sous mutex, retirée juste après.
        let _guard = RELEC1_CRASH_LOCK.lock().expect("lock RELEC1_CRASH");
        unsafe { std::env::set_var("RELEC1_CRASH", "1") };
        let coupe = evaluate_routines(
            &mut store,
            &durations(),
            "maicie",
            &[candidate("prospective")],
            t0,
        )
        .expect("releve coupee");
        unsafe { std::env::remove_var("RELEC1_CRASH") };
        drop(_guard);
        drop(store);

        assert!(
            coupe.is_empty(),
            "aggravation relec1 : tick coupe rend Ok(produced=0)"
        );

        let connexion = rusqlite::Connection::open(&database).unwrap();
        let deleg_apres_coupure: i64 = connexion
            .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
            .unwrap();
        let occ_apres_coupure: i64 = connexion
            .query_row("SELECT COUNT(*) FROM routine_occurrences", [], |r| r.get(0))
            .unwrap();
        drop(connexion);
        assert_eq!(deleg_apres_coupure, 1, "mandat parti");
        assert_eq!(occ_apres_coupure, 0, "occurrence jamais écrite");

        let mut store = MaicieStore::open(&database).unwrap();
        let second = evaluate_routines(
            &mut store,
            &durations(),
            "maicie",
            &[candidate("prospective")],
            t0 + 2 * period,
        )
        .expect("reprise");
        drop(store);

        let connexion = rusqlite::Connection::open(&database).unwrap();
        let deleg_final: i64 = connexion
            .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
            .unwrap();
        drop(connexion);

        let etat_n = second
            .iter()
            .find(|o| o.bucket == bucket_n)
            .map(|o| (o.state, o.reason.clone()));
        let ouvertes = second
            .iter()
            .filter(|o| o.state == EtatOccurrence::Ouverte)
            .count();
        if deleg_final == 2 && ouvertes == 1 {
            doublons += 1;
        }
        if deleg_final == 1
            && matches!(
                etat_n.as_ref().map(|(s, r)| (*s, r.as_deref())),
                Some((EtatOccurrence::Ouverte, Some("mandat_adopte")))
            )
        {
            adoptions += 1;
        }
        let _ = fs::remove_dir_all(root);
    }
    assert_eq!(doublons, 0, "crash réel : zéro doublon après adoption");
    assert_eq!(adoptions, N, "crash réel : adoption à chaque reprise");
}
