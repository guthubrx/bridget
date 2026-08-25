//! Oracles routines (manche 4) :
//! - panne → exactement N sautee `horloge_arretee` + 1 ouverte
//! - occurrence vivante → differee, puis clôture → terminee → nouvelle ouverte
//! - rejeu → zéro doublon sous (routine_id, bucket)
//! - garde hash : gabarit altéré refusé
//! - pause/resume : pas de rattrapage des buckets de pause
//! - rattrapage borné : sentinel `rattrapage_borne:<N>` (N = buckets effacés)

use maicie::app::DelegationCandidate;
use maicie::config::DurationClasses;
use maicie::domain::SuiteObjective;
use maicie::routines::{
    EtatOccurrence, EtatRoutine, EvaluateRoutinesOpts, MAX_CATCHUP_BUCKETS, ProposeRoutineRequest,
    approve_routine, bucket_for, evaluate_routines, evaluate_routines_with, pause_routine,
    propose_routine, resume_routine, sealed_template_hash, template_hash,
};
use maicie::store::MaicieStore;
use rusqlite::params;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("maicie-routines-{label}-{}", Uuid::new_v4()))
}

/// Nettoie le répertoire de tir même quand le banc `panic!`, parce qu'un
/// `remove_dir_all` en fin de fonction est sauté par le déroulement de pile :
/// c'est ainsi que 43 répertoires orphelins ont rempli le volume le 2026-08-24.
struct RootGuard {
    path: PathBuf,
}

impl RootGuard {
    fn new(label: &str) -> Self {
        Self { path: root(label) }
    }
}

impl Drop for RootGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
        .filter(|occ| {
            occ.reason
                .as_deref()
                .is_some_and(|r| r.starts_with("rattrapage_borne:"))
        })
        .collect();
    assert_eq!(borne.len(), 1, "une seule sentinelle de troncature");
    let skipped: i64 = borne[0]
        .reason
        .as_deref()
        .and_then(|r| r.strip_prefix("rattrapage_borne:"))
        .and_then(|n| n.parse().ok())
        .expect("sentinelle doit porter le compte");
    // skipped = gap - MAX_CATCHUP_BUCKETS - 1 (sentinelle + MAX derniers traités).
    let current = bucket_for(later, period);
    let after_approve = bucket_for(t0, period).saturating_sub(1);
    let gap_initial = current - after_approve;
    let expected_skipped = (gap_initial - MAX_CATCHUP_BUCKETS - 1).max(0);
    assert_eq!(
        skipped, expected_skipped,
        "sentinelle doit chiffrer le trou effacé (gap={gap_initial})"
    );
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

/// Mesures brutes d'un tir du banc v3 (régimes sous / juste / loin au-delà
/// de [`MAX_CATCHUP_BUCKETS`]).
struct Relec1TirV3 {
    coupe_vide: bool,
    deleg_apres_coupure: i64,
    occ_apres_coupure: i64,
    deleg_final: i64,
    etat_n: Option<(EtatOccurrence, Option<String>)>,
    /// Buckets réellement mandatés, en offset depuis le bucket de la coupure.
    offsets_mandates: Vec<i64>,
}

/// Buckets portés par les mandats émis, relus dans le goal
/// `[routine <id> bucket <n>] ...`. Le COMPTE seul ne dit pas QUEL bucket.
fn offsets_mandates(connexion: &rusqlite::Connection, bucket_reference: i64) -> Vec<i64> {
    let mut requete = connexion
        .prepare(
            "SELECT o.payload_json FROM objectives o \
             JOIN delegations d ON d.objective_id = o.id",
        )
        .unwrap();
    let payloads: Vec<Vec<u8>> = requete
        .query_map([], |row| row.get::<_, Vec<u8>>(0))
        .unwrap()
        .map(|payload| payload.unwrap())
        .collect();
    let mut offsets: Vec<i64> = payloads
        .iter()
        .filter_map(|payload| {
            let texte = String::from_utf8_lossy(payload);
            let debut = texte.find("bucket ")? + "bucket ".len();
            let reste = &texte[debut..];
            let fin = reste.find(']')?;
            let bucket = reste[..fin].trim().parse::<i64>().ok()?;
            Some(bucket - bucket_reference)
        })
        .collect();
    offsets.sort_unstable();
    offsets
}

/// Un tir : coupure injectée à `t0`, reprise à `t0 + reprise_mult * period`.
fn relec1_tir_crash_reel(reprise_mult: i64, label: &str) -> Relec1TirV3 {
    let guard = RootGuard::new(label);
    let database = guard.path.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    seed_active(&mut store, t0, period);
    let bucket_n = bucket_for(t0, period);

    let coupe = evaluate_routines_with(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
        EvaluateRoutinesOpts {
            abort_before_occurrence_insert: true,
        },
    )
    .expect("releve coupee");
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let deleg_apres_coupure: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    let occ_apres_coupure: i64 = connexion
        .query_row("SELECT COUNT(*) FROM routine_occurrences", [], |r| r.get(0))
        .unwrap();
    drop(connexion);

    let mut store = MaicieStore::open(&database).unwrap();
    let second = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + reprise_mult * period,
    )
    .expect("reprise");
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let deleg_final: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    let offsets = offsets_mandates(&connexion, bucket_n);
    drop(connexion);

    Relec1TirV3 {
        coupe_vide: coupe.is_empty(),
        deleg_apres_coupure,
        occ_apres_coupure,
        deleg_final,
        etat_n: second
            .iter()
            .find(|o| o.bucket == bucket_n)
            .map(|o| (o.state, o.reason.clone())),
        offsets_mandates: offsets,
    }
}

/// Banc v3 — série appariée sur trois instants de reprise (sous / juste /
/// loin au-delà de la borne). Régime couvert : gap ∈ {3, 66, 101}.
/// Après remède borne×adoption : 1 délégation partout ; offsets [0] partout.
#[test]
fn relec1_serie_crash_reel_apres_adoption() {
    const N: usize = 5;
    // 2 = sous la borne (contrôle positif historique) ;
    // 65 / 100 = régime que la borne gouverne (motif mesuré 1/2/2 avant remède).
    const POINTS: [i64; 3] = [2, 65, 100];

    println!("=== BANC relec1 v3 — MAX_CATCHUP_BUCKETS = {MAX_CATCHUP_BUCKETS} ===");
    for point in POINTS {
        let mut tirs = Vec::with_capacity(N);
        for tir in 0..N {
            let mesure = relec1_tir_crash_reel(point, &format!("crash-reel-p{point}-{tir}"));
            assert!(
                mesure.coupe_vide,
                "banc faux (point {point}, tir {tir}) : tick coupé rend produced != 0"
            );
            assert_eq!(
                mesure.deleg_apres_coupure, 1,
                "banc faux (point {point}, tir {tir}) : mandat non parti"
            );
            assert_eq!(
                mesure.occ_apres_coupure, 0,
                "banc faux (point {point}, tir {tir}) : occurrence écrite malgré coupure"
            );
            tirs.push(mesure);
        }

        let comptes: Vec<i64> = tirs.iter().map(|t| t.deleg_final).collect();
        println!("--- POINT t0 + {point} * period ---");
        println!("  delegations apres reprise : {comptes:?}");
        println!(
            "  bucket de la coupure      : {:?}",
            tirs.iter()
                .map(|t| match &t.etat_n {
                    Some((etat, raison)) => {
                        format!("{etat:?}/{}", raison.as_deref().unwrap_or("-"))
                    }
                    None => "absent".to_string(),
                })
                .collect::<Vec<_>>()
        );
        println!("  offsets mandates (tir 0)  : {:?}", tirs[0].offsets_mandates);

        assert!(
            comptes.iter().all(|c| *c == 1),
            "adoption hors borne : point {point} doit rendre 1 délégation, obtenu {comptes:?}"
        );
        for (tir, mesure) in tirs.iter().enumerate() {
            assert_eq!(
                mesure.offsets_mandates,
                vec![0],
                "point {point} tir {tir} : seul le bucket de la coupure doit être mandaté"
            );
            assert_eq!(
                mesure.etat_n.as_ref().map(|(s, r)| (*s, r.as_deref())),
                Some((EtatOccurrence::Ouverte, Some("mandat_adopte"))),
                "point {point} tir {tir} : bucket de la coupure adopté"
            );
        }
    }
}

/// Propriété : une occurrence ne doit jamais attester un mandat **annulé**
/// ou disparu. `terminee` (mission accomplie) n'est pas un cadavre à
/// rétracter — l'occurrence attend la clôture d'objectif.
/// Matrice recalibrée (hotfix production), chemin coupure/orphelin N=5 :
/// - terminal (`annulee`/`terminee`) → jamais adopté ; calendrier REPART
/// - `annulee` sur une `ouverte` existante → RÉTRACTE (oracle voisin)
/// Mutant rétractation : remettre `'terminee'` dans la liste morte fait
/// rougir `mission_accomplie_n_est_pas_retractee_en_sautee` (et peut
/// inventer une `sautee` sur mission accomplie ici si une ouverte existe).
#[test]
fn occurrence_n_atteste_jamais_un_mandat_mort() {
    const N: usize = 5;
    let cas = [
        ("terminee", true),
        ("terminee", false),
        ("annulee", true),
        ("annulee", false),
    ];
    for (etat, clore) in cas {
        for tir in 0..N {
            let guard = RootGuard::new(&format!("prop-mort-{etat}-{clore}-{tir}"));
            let database = guard.path.join("maicie.sqlite3");
            let mut store = MaicieStore::open(&database).unwrap();
            let period = 60_i64;
            let t0 = 1_787_580_000;
            seed_active(&mut store, t0, period);

            let coupe = evaluate_routines_with(
                &mut store,
                &durations(),
                "maicie",
                &[candidate("prospective")],
                t0,
                EvaluateRoutinesOpts {
                    abort_before_occurrence_insert: true,
                },
            )
            .expect("coupure");
            assert!(coupe.is_empty());
            drop(store);

            let cx = rusqlite::Connection::open(&database).unwrap();
            let deleg_id: String = cx
                .query_row("SELECT id FROM delegations LIMIT 1", [], |r| r.get(0))
                .unwrap();
            cx.execute(
                "UPDATE delegations SET state = ?1 WHERE id = ?2",
                rusqlite::params![etat, deleg_id],
            )
            .unwrap();
            if clore {
                cx.execute("UPDATE objectives SET state = 'clos'", [])
                    .unwrap();
            }
            drop(cx);

            let mut store = MaicieStore::open(&database).unwrap();
            for k in 0..3 {
                let _ = evaluate_routines(
                    &mut store,
                    &durations(),
                    "maicie",
                    &[candidate("prospective")],
                    t0 + (100 + k) * period,
                )
                .expect("reprise");
            }
            drop(store);

            let cx = rusqlite::Connection::open(&database).unwrap();
            let delegations: i64 = cx
                .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
                .unwrap();
            // Menteuse = ouverte qui atteste un mandat disparu ou annulé.
            // `terminee` n'y figure PAS : mission accomplie, pas cadavre.
            let menteuses: i64 = cx
                .query_row(
                    "SELECT COUNT(*) FROM routine_occurrences o\n\
                     LEFT JOIN delegations d ON d.id = o.delegation_id\n\
                     WHERE o.state = 'ouverte'\n\
                       AND (o.delegation_id IS NULL\n\
                            OR d.state = 'annulee'\n\
                            OR d.id IS NULL)",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            let sautees_fausses: i64 = cx
                .query_row(
                    "SELECT COUNT(*) FROM routine_occurrences o\n\
                     JOIN delegations d ON d.id = o.delegation_id\n\
                     WHERE o.state = 'sautee'\n\
                       AND o.reason = 'mandat_plus_vivant'\n\
                       AND d.state = 'terminee'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            drop(cx);

            assert_eq!(
                menteuses, 0,
                "{etat} clos={clore} tir {tir} : aucune ouverte ne doit attester un annulé/disparu"
            );
            assert_eq!(
                sautees_fausses, 0,
                "{etat} clos={clore} tir {tir} : jamais sautee/mandat_plus_vivant sur une mission accomplie"
            );
            // Chemin coupure/orphelin : un mandat terminal n'est jamais adopté ;
            // le calendrier repart (deleg≥2). La non-relance d'une mission
            // accomplie déjà attestée par une `ouverte` est l'oracle
            // `mission_accomplie_n_est_pas_retractee_en_sautee`.
            assert!(
                delegations >= 2,
                "{etat} clos={clore} tir {tir} : routine doit repartir (deleg≥2), obtenu {delegations}"
            );
        }
    }
}

/// Rétractation : une ouverte devenue cadavre après adoption ne gèle plus.
#[test]
fn ouverte_retractee_quand_le_mandat_meurt_apres_coup() {
    let guard = RootGuard::new("retract-apres-coup");
    let database = guard.path.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    seed_active(&mut store, t0, period);

    // Coupure puis reprise sous la borne → adoption vivante.
    let _ = evaluate_routines_with(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
        EvaluateRoutinesOpts {
            abort_before_occurrence_insert: true,
        },
    )
    .unwrap();
    drop(store);
    let mut store = MaicieStore::open(&database).unwrap();
    let _ = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + 2 * period,
    )
    .unwrap();
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    assert_eq!(
        cx.query_row(
            "SELECT COUNT(*) FROM routine_occurrences WHERE state = 'ouverte'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .unwrap(),
        1
    );
    cx.execute("UPDATE delegations SET state = 'annulee'", [])
        .unwrap();
    // Objectif reste OUVERT — seul le rattrapage mandat_plus_vivant doit agir.
    drop(cx);

    let mut store = MaicieStore::open(&database).unwrap();
    let _ = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + 5 * period,
    )
    .unwrap();
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    let retractees: i64 = cx
        .query_row(
            "SELECT COUNT(*) FROM routine_occurrences\n\
             WHERE state = 'sautee' AND reason = 'mandat_plus_vivant'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let ouvertes: i64 = cx
        .query_row(
            "SELECT COUNT(*) FROM routine_occurrences WHERE state = 'ouverte'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let delegations: i64 = cx
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(cx);

    assert_eq!(retractees, 1, "ouverte rétractée en mandat_plus_vivant");
    assert_eq!(ouvertes, 1, "un neuf doit pouvoir ouvrir");
    assert!(delegations >= 2, "redélégation après cadavre, objectif encore ouvert");
}

/// relec5 — contrôle : délégation VIVANTE → l'occurrence n'est PAS rétractée.
/// Distingue la clause `delegation_id IS NULL` de la clause d'état.
#[test]
fn controle_delegation_vivante_non_retractee() {
    let guard = RootGuard::new("vivante-non-retract");
    let database = guard.path.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000_i64;
    seed_active(&mut store, t0, period);
    evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
    )
    .unwrap();
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    let did: Option<String> = cx
        .query_row(
            "SELECT delegation_id FROM routine_occurrences",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let etat: String = cx
        .query_row("SELECT state FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(cx);

    let mut store = MaicieStore::open(&database).unwrap();
    evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + period,
    )
    .unwrap();
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    let bn = bucket_for(t0, period);
    let retractee: i64 = cx
        .query_row(
            "SELECT COUNT(*) FROM routine_occurrences\n\
             WHERE bucket = ?1 AND state = 'sautee'\n\
               AND reason = 'mandat_plus_vivant'",
            [bn],
            |r| r.get(0),
        )
        .unwrap();
    drop(cx);

    assert!(
        did.is_some(),
        "banc faux : occurrence sans delegation_id (état délégation={etat})"
    );
    assert_eq!(
        retractee, 0,
        "CLAUSE delegation_id IS NULL : occurrence normale rétractée alors que délégation='{etat}'"
    );
}

/// relec5 — mission ACCOMPLIE (`terminee`, objectif pas encore clos) :
/// l'occurrence ne doit PAS passer `sautee/mandat_plus_vivant`, et aucun
/// mandat neuf ne doit partir. Mutant : remettre `'terminee'` dans la
/// liste de rétractation → cette assertion meurt.
#[test]
fn mission_accomplie_n_est_pas_retractee_en_sautee() {
    let guard = RootGuard::new("mission-accomplie");
    let database = guard.path.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000_i64;
    seed_active(&mut store, t0, period);
    let produced = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
    )
    .unwrap();
    let bn = bucket_for(t0, period);
    assert!(
        produced
            .iter()
            .any(|o| o.state == EtatOccurrence::Ouverte && o.bucket == bn),
        "banc faux : pas d'occurrence ouverte initiale"
    );
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    let n = cx
        .execute("UPDATE delegations SET state = 'terminee'", [])
        .unwrap();
    assert_eq!(n, 1, "banc faux : une délégation attendue");
    let etat_obj: String = cx
        .query_row("SELECT state FROM objectives", [], |r| r.get(0))
        .unwrap();
    assert_ne!(etat_obj, "clos", "banc faux : objectif déjà clos");
    let avant: Vec<(i64, String, Option<String>)> = {
        let mut stmt = cx
            .prepare("SELECT bucket, state, reason FROM routine_occurrences ORDER BY bucket")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    drop(cx);

    let mut store = MaicieStore::open(&database).unwrap();
    evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + period,
    )
    .unwrap();
    drop(store);

    let cx = rusqlite::Connection::open(&database).unwrap();
    let apres: Vec<(i64, String, Option<String>)> = {
        let mut stmt = cx
            .prepare("SELECT bucket, state, reason FROM routine_occurrences ORDER BY bucket")
            .unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    };
    let deleg: i64 = cx
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    drop(cx);

    let _ = (&avant, &apres); // pièces de mesure (greffe relec5)
    let retractee = apres.iter().any(|(b, s, r)| {
        *b == bn && s == "sautee" && r.as_deref() == Some("mandat_plus_vivant")
    });
    assert!(
        !retractee,
        "MISSION ACCOMPLIE RÉTRACTÉE : bucket {bn} en 'sautee/mandat_plus_vivant' \
         alors que la délégation est 'terminee', objectif '{etat_obj}'. \
         Le greffe dirait « sautée » pour un travail qui a été fait."
    );
    assert_eq!(
        deleg, 1,
        "relance interdite : mission accomplie ne doit pas ouvrir un 2ᵉ mandat (obtenu {deleg})"
    );
}

/// Contrôle positif du filtre d'état : mandat vivant → toujours adopté.
#[test]
fn adoption_accepte_un_mandat_vivant_au_dela_de_la_borne() {
    let mesure = relec1_tir_crash_reel(100, "mandat-vivant-controle");
    assert_eq!(mesure.deleg_final, 1);
    assert_eq!(
        mesure.etat_n.as_ref().map(|(s, r)| (*s, r.as_deref())),
        Some((EtatOccurrence::Ouverte, Some("mandat_adopte")))
    );
}

/// Banc relec1 m6 — garde de contrat sur le saut de `resume_routine` :
/// adopter les orphelins de l'intervalle sauté avant `last_bucket`.
/// Sans garde (API seule) : pause+resume → deleg=2. Avec garde : deleg=1
/// des deux bras. Ce n'est pas un incident CLI (« mandat perdu au resume ») :
/// c'est une fragilité de composition couverte par construction.
fn relec1_m6_tir(avec_pause: bool, label: &str) -> (i64, bool) {
    let guard = RootGuard::new(label);
    let database = guard.path.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let period = 60_i64;
    let t0 = 1_787_580_000;
    let routine_id = seed_active(&mut store, t0, period);
    let bucket_n = bucket_for(t0, period);

    let coupe = evaluate_routines_with(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0,
        EvaluateRoutinesOpts {
            abort_before_occurrence_insert: true,
        },
    )
    .expect("releve coupee");
    assert!(coupe.is_empty(), "la coupure ne produit aucune occurrence");

    if avec_pause {
        pause_routine(&mut store, routine_id, t0 + period).expect("pause");
        resume_routine(&mut store, routine_id, t0 + 10 * period).expect("resume");
    }

    let _reprise = evaluate_routines(
        &mut store,
        &durations(),
        "maicie",
        &[candidate("prospective")],
        t0 + 10 * period,
    )
    .expect("reprise");
    drop(store);

    let connexion = rusqlite::Connection::open(&database).unwrap();
    let delegations: i64 = connexion
        .query_row("SELECT COUNT(*) FROM delegations", [], |r| r.get(0))
        .unwrap();
    // L'adoption peut avoir lieu DANS resume (hors produced du tick suivant).
    let orphelin_adopte: bool = connexion
        .query_row(
            "SELECT COUNT(*) FROM routine_occurrences\n\
             WHERE bucket = ?1 AND state = 'ouverte' AND reason = 'mandat_adopte'",
            rusqlite::params![bucket_n],
            |r| r.get::<_, i64>(0),
        )
        .map(|n| n > 0)
        .unwrap_or(false);
    drop(connexion);
    (delegations, orphelin_adopte)
}

#[test]
fn relec1_m6_saut_de_resume_adopte_les_orphelins() {
    const N: usize = 5;
    for tir in 0..N {
        let (d_ctrl, adopte_ctrl) = relec1_m6_tir(false, &format!("m6-ctrl-{tir}"));
        let (d_pause, adopte_pause) = relec1_m6_tir(true, &format!("m6-pause-{tir}"));
        assert_eq!(
            d_ctrl, 1,
            "tir {tir} contrôle sans pause : 1 délégation attendue"
        );
        assert!(
            adopte_ctrl,
            "tir {tir} contrôle sans pause : orphelin adopté"
        );
        assert_eq!(
            d_pause, 1,
            "tir {tir} pause+resume : 1 délégation (garde resume) — obtenu {d_pause}"
        );
        assert!(
            adopte_pause,
            "tir {tir} pause+resume : orphelin adopté avant le saut de last_bucket"
        );
    }
}
