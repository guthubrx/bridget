//! Routines = gabarit de `delegate` + calendrier évalué à chaque relève.
//!
//! Doctrine : une routine délègue, n'approuve jamais. L'horloge est la relève
//! (60 s) ; aucun timer résident. Clé d'occurrence = `(routine_id, bucket)`.
//!
//! Motifs d'occurrence `sautee` (fermés) :
//! - `horloge_arretee` — buckets échus pendant une indisponibilité du tick
//! - `rattrapage_borne:<N>` — trou tronqué ; N = buckets effacés sans ligne

use crate::app::{DelegateError, DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use crate::config::DurationClasses;
use crate::domain::{ClasseDuree, SuiteObjective};
use crate::store::{MaicieStore, StoreError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Période minimale (évite un spam à la seconde sur une mauvaise frappe).
pub const MIN_PERIOD_SECS: i64 = 60;

/// Nombre max de buckets rattrapés par tick (hors bucket courant).
/// Au-delà : une sautee `rattrapage_borne:<skipped>` puis traitement des `MAX` derniers.
/// Mesure manche 4 : 30 j / 60 s → 43 201 inserts / 5,8 s sans borne.
pub const MAX_CATCHUP_BUCKETS: i64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatRoutine {
    Proposed,
    Active,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatOccurrence {
    Ouverte,
    Sautee,
    Differee,
    Terminee,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Routine {
    pub id: Uuid,
    pub goal: String,
    pub participant: String,
    pub period_secs: i64,
    pub suite: SuiteObjective,
    pub depends_on: Vec<Uuid>,
    pub references: Vec<Uuid>,
    pub template_hash: Vec<u8>,
    pub state: EtatRoutine,
    pub proposed_at: i64,
    pub approved_at: Option<i64>,
    pub paused_at: Option<i64>,
    /// Dernier bucket déjà traité (sautee/differee/ouverte). `None` = jamais.
    pub last_bucket: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineOccurrence {
    pub routine_id: Uuid,
    pub bucket: i64,
    pub state: EtatOccurrence,
    pub reason: Option<String>,
    pub objective_id: Option<Uuid>,
    pub delegation_id: Option<Uuid>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutineStatusRow {
    pub routine_id: Uuid,
    pub state: EtatRoutine,
    pub period_secs: i64,
    pub participant: String,
    pub last_bucket: Option<i64>,
    pub open_occurrence: Option<RoutineOccurrence>,
    pub recent_sautee: Vec<RoutineOccurrence>,
    pub recent_differee: Vec<RoutineOccurrence>,
}

#[derive(Debug, Clone)]
pub struct ProposeRoutineRequest<'a> {
    pub goal: &'a str,
    pub participant: &'a str,
    pub period_secs: i64,
    pub suite: SuiteObjective,
    pub depends_on: &'a [Uuid],
    pub references: &'a [Uuid],
    pub now: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutineError {
    Invalid(&'static str),
    NotFound(Uuid),
    Store(String),
    Delegate(String),
}

impl std::fmt::Display for RoutineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(reason) => write!(f, "routine invalide : {reason}"),
            Self::NotFound(id) => write!(f, "routine introuvable : {id}"),
            Self::Store(reason) => write!(f, "stockage routine impossible : {reason}"),
            Self::Delegate(reason) => write!(f, "délégation routine impossible : {reason}"),
        }
    }
}

impl std::error::Error for RoutineError {}

pub fn template_hash(
    goal: &str,
    participant: &str,
    period_secs: i64,
    suite: &SuiteObjective,
    depends_on: &[Uuid],
    references: &[Uuid],
) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(goal.as_bytes());
    hasher.update(b"\0");
    hasher.update(participant.as_bytes());
    hasher.update(b"\0");
    hasher.update(period_secs.to_le_bytes());
    hasher.update(b"\0");
    match suite {
        SuiteObjective::Aucune => hasher.update(b"aucune"),
        SuiteObjective::Objectif(id) => {
            hasher.update(b"objectif:");
            hasher.update(id.as_bytes());
        }
    }
    hasher.update(b"\0");
    for id in depends_on {
        hasher.update(id.as_bytes());
        hasher.update(b",");
    }
    hasher.update(b"\0");
    for id in references {
        hasher.update(id.as_bytes());
        hasher.update(b",");
    }
    hasher.finalize().to_vec()
}

pub fn sealed_template_hash(routine: &Routine) -> Vec<u8> {
    template_hash(
        &routine.goal,
        &routine.participant,
        routine.period_secs,
        &routine.suite,
        &routine.depends_on,
        &routine.references,
    )
}

pub fn bucket_for(now: i64, period_secs: i64) -> i64 {
    now.div_euclid(period_secs)
}

pub fn propose_routine(
    store: &mut MaicieStore,
    request: &ProposeRoutineRequest<'_>,
) -> Result<Routine, RoutineError> {
    validate_propose(request)?;
    let routine = Routine {
        id: Uuid::new_v4(),
        goal: request.goal.to_string(),
        participant: request.participant.to_string(),
        period_secs: request.period_secs,
        suite: request.suite.clone(),
        depends_on: request.depends_on.to_vec(),
        references: request.references.to_vec(),
        template_hash: template_hash(
            request.goal,
            request.participant,
            request.period_secs,
            &request.suite,
            request.depends_on,
            request.references,
        ),
        state: EtatRoutine::Proposed,
        proposed_at: request.now,
        approved_at: None,
        paused_at: None,
        last_bucket: None,
    };
    store
        .insert_routine(&routine)
        .map_err(routine_store_error)?;
    Ok(routine)
}

pub fn approve_routine(
    store: &mut MaicieStore,
    routine_id: Uuid,
    expected_hash: &[u8],
    now: i64,
) -> Result<Routine, RoutineError> {
    let routine = store
        .load_routine(routine_id)
        .map_err(routine_store_error)?
        .ok_or(RoutineError::NotFound(routine_id))?;
    if routine.state != EtatRoutine::Proposed {
        return Err(RoutineError::Invalid("routine hors état proposed"));
    }
    // Intégrité du gabarit : les champs relus doivent reseeller le hash stocké.
    // Sans ce recalcul, passer le hash lu en base est une tautologie (B3).
    let recomputed = sealed_template_hash(&routine);
    if recomputed != routine.template_hash {
        return Err(RoutineError::Invalid("gabarit altéré"));
    }
    if recomputed != expected_hash {
        return Err(RoutineError::Invalid("template_hash divergent"));
    }
    // Le premier bucket évaluable est celui de l'approve — pas de rattrapage
    // des périodes antérieures à la naissance.
    let last_bucket = bucket_for(now, routine.period_secs).saturating_sub(1);
    store
        .activate_routine_cas(routine_id, &recomputed, now, last_bucket)
        .map_err(routine_store_error)
}

pub fn pause_routine(
    store: &mut MaicieStore,
    routine_id: Uuid,
    now: i64,
) -> Result<Routine, RoutineError> {
    let mut routine = store
        .load_routine(routine_id)
        .map_err(routine_store_error)?
        .ok_or(RoutineError::NotFound(routine_id))?;
    if routine.state != EtatRoutine::Active {
        return Err(RoutineError::Invalid("pause hors routine active"));
    }
    routine.state = EtatRoutine::Paused;
    routine.paused_at = Some(now);
    store
        .update_routine(&routine)
        .map_err(routine_store_error)?;
    Ok(routine)
}

pub fn resume_routine(
    store: &mut MaicieStore,
    routine_id: Uuid,
    now: i64,
) -> Result<Routine, RoutineError> {
    let mut routine = store
        .load_routine(routine_id)
        .map_err(routine_store_error)?
        .ok_or(RoutineError::NotFound(routine_id))?;
    if routine.state != EtatRoutine::Paused {
        return Err(RoutineError::Invalid("reprise hors routine paused"));
    }
    routine.state = EtatRoutine::Active;
    routine.paused_at = None;
    // À la reprise : pas de rattrapage des buckets manqués pendant la pause.
    routine.last_bucket = Some(bucket_for(now, routine.period_secs).saturating_sub(1));
    store
        .update_routine(&routine)
        .map_err(routine_store_error)?;
    Ok(routine)
}

/// Évalue les routines actives à `now`. Matérialise au plus une occurrence
/// ouverte par routine ; les buckets échus sans occurrence deviennent `sautee`.
/// Avant tout : clôture les occurrences dont l'objectif lié est déjà clos.
pub fn evaluate_routines(
    store: &mut MaicieStore,
    durations: &DurationClasses,
    issuer_scope: &str,
    candidates: &[DelegationCandidate],
    now: i64,
) -> Result<Vec<RoutineOccurrence>, RoutineError> {
    // Rattrapage de clôture si l'objectif a été clos hors du hook transactionnel
    // (ou avant le correctif) — idempotent.
    store
        .terminate_occurrences_with_closed_objectives()
        .map_err(routine_store_error)?;

    let actives = store
        .list_routines(Some(EtatRoutine::Active))
        .map_err(routine_store_error)?;
    let mut produced = Vec::new();
    for mut routine in actives {
        let current = bucket_for(now, routine.period_secs);
        let after = routine.last_bucket.unwrap_or(current.saturating_sub(1));
        if after >= current {
            continue;
        }

        let mut from = after + 1;
        let gap = current - after;
        if gap > MAX_CATCHUP_BUCKETS {
            // Sentinel unique pour le trou tronqué, puis au plus MAX buckets.
            // skipped = buckets effacés sans ligne individuelle (mesure manche 4 :
            // 30 j / 60 s → 43 135 disparus — la sentinelle DOIT porter ce compte).
            let truncated_end = current - MAX_CATCHUP_BUCKETS;
            let skipped = (truncated_end - after - 1).max(0);
            if store
                .load_occurrence(routine.id, truncated_end)
                .map_err(routine_store_error)?
                .is_none()
            {
                let occ = RoutineOccurrence {
                    routine_id: routine.id,
                    bucket: truncated_end,
                    state: EtatOccurrence::Sautee,
                    reason: Some(format!("rattrapage_borne:{skipped}")),
                    objective_id: None,
                    delegation_id: None,
                    created_at: now,
                };
                store.insert_occurrence(&occ).map_err(routine_store_error)?;
                produced.push(occ);
            }
            routine.last_bucket = Some(truncated_end);
            from = truncated_end + 1;
        }

        let open = store
            .open_occurrence_for_routine(routine.id)
            .map_err(routine_store_error)?;
        let mut has_open = open.is_some();
        for bucket in from..=current {
            if store
                .load_occurrence(routine.id, bucket)
                .map_err(routine_store_error)?
                .is_some()
            {
                routine.last_bucket = Some(bucket);
                continue;
            }
            let key = format!("routine:{}:{}", routine.id, bucket);
            // Avant toute sautee : adopter un mandat déjà parti (fenêtre
            // delegate→insert_occurrence sans tx commune — manche 4 motif 3).
            if let Some(adopted) = adopt_orphan_mandate(store, routine.id, bucket, &key, now)? {
                produced.push(adopted);
                routine.last_bucket = Some(bucket);
                has_open = true;
                continue;
            }
            if bucket < current {
                let occ = RoutineOccurrence {
                    routine_id: routine.id,
                    bucket,
                    state: EtatOccurrence::Sautee,
                    reason: Some("horloge_arretee".to_string()),
                    objective_id: None,
                    delegation_id: None,
                    created_at: now,
                };
                store.insert_occurrence(&occ).map_err(routine_store_error)?;
                produced.push(occ);
                routine.last_bucket = Some(bucket);
                continue;
            }
            // bucket == current
            if has_open {
                let occ = RoutineOccurrence {
                    routine_id: routine.id,
                    bucket,
                    state: EtatOccurrence::Differee,
                    reason: Some("occurrence_vivante".to_string()),
                    objective_id: None,
                    delegation_id: None,
                    created_at: now,
                };
                store.insert_occurrence(&occ).map_err(routine_store_error)?;
                produced.push(occ);
                routine.last_bucket = Some(bucket);
                continue;
            }
            if candidates.is_empty() {
                // Bridget / annuaire indisponible : ne consomme pas le bucket.
                break;
            }
            let goal = format!(
                "[routine {} bucket {}] {}",
                routine.id, bucket, routine.goal
            );
            let request = DelegateRequest {
                goal: &goal,
                explicit_target: Some(routine.participant.as_str()),
                required_tags: &[],
                duration: ClasseDuree::Normale,
                reply: false,
                constat_id: None,
                suite: routine.suite.clone(),
                depends_on: &routine.depends_on,
                references: &routine.references,
                idempotency_key: &key,
                now,
                retry_until: now
                    .saturating_add(i64::try_from(durations.long_secs).unwrap_or(i64::MAX)),
                dedup_retained_until: now
                    .saturating_add(i64::try_from(durations.long_secs).unwrap_or(i64::MAX)),
                max_frame_bytes: 256 * 1024,
            };
            let created = match delegate(store, *durations, issuer_scope, candidates, &request) {
                Ok(DelegateResult::Created(created)) => created,
                // Cible non résolue / indisponible / autre blip : pas de bucket
                // consumé — prochaine relève. Ne remonte jamais en erreur fatale.
                Ok(DelegateResult::Candidates(_))
                | Err(DelegateError::TargetUnavailable(_))
                | Err(_) => {
                    break;
                }
            };
            let occ = RoutineOccurrence {
                routine_id: routine.id,
                bucket,
                state: EtatOccurrence::Ouverte,
                reason: None,
                objective_id: Some(created.objective_id),
                delegation_id: Some(created.delegation_id),
                created_at: now,
            };
            // Point de coupure testable (oracle relec1 v3 / mutant).
            // Actif UNIQUEMENT si RELEC1_CRASH est posé — jamais en prod normale.
            // Simule un crash entre delegate() et insert_occurrence().
            if std::env::var_os("RELEC1_CRASH").is_some() {
                break;
            }
            store.insert_occurrence(&occ).map_err(routine_store_error)?;
            produced.push(occ);
            routine.last_bucket = Some(bucket);
            has_open = true;
        }
        store
            .update_routine(&routine)
            .map_err(routine_store_error)?;
    }
    Ok(produced)
}

pub fn routines_status_rows(store: &MaicieStore) -> Result<Vec<RoutineStatusRow>, RoutineError> {
    let routines = store.list_routines(None).map_err(routine_store_error)?;
    let mut rows = Vec::with_capacity(routines.len());
    for routine in routines {
        let open = store
            .open_occurrence_for_routine(routine.id)
            .map_err(routine_store_error)?;
        let recent_sautee = store
            .recent_occurrences(routine.id, EtatOccurrence::Sautee, 8)
            .map_err(routine_store_error)?;
        let recent_differee = store
            .recent_occurrences(routine.id, EtatOccurrence::Differee, 8)
            .map_err(routine_store_error)?;
        rows.push(RoutineStatusRow {
            routine_id: routine.id,
            state: routine.state,
            period_secs: routine.period_secs,
            participant: routine.participant,
            last_bucket: routine.last_bucket,
            open_occurrence: open,
            recent_sautee,
            recent_differee,
        });
    }
    Ok(rows)
}

fn validate_propose(request: &ProposeRoutineRequest<'_>) -> Result<(), RoutineError> {
    if request.goal.trim().is_empty() {
        return Err(RoutineError::Invalid("goal vide"));
    }
    if request.participant.trim().is_empty() {
        return Err(RoutineError::Invalid("participant vide"));
    }
    if request.period_secs < MIN_PERIOD_SECS {
        return Err(RoutineError::Invalid("period_secs < 60"));
    }
    if request.now <= 0 {
        return Err(RoutineError::Invalid("now invalide"));
    }
    Ok(())
}

/// Si un mandat `routine:{id}:{bucket}` existe déjà sans occurrence : l'adopter
/// en `ouverte` (jamais `sautee`). Remède manche 4 motif 3.
fn adopt_orphan_mandate(
    store: &mut MaicieStore,
    routine_id: Uuid,
    bucket: i64,
    idempotency_key: &str,
    now: i64,
) -> Result<Option<RoutineOccurrence>, RoutineError> {
    let Some((objective_id, delegation_id)) = store
        .lookup_delegate_ids_by_key(idempotency_key)
        .map_err(routine_store_error)?
    else {
        return Ok(None);
    };
    let occ = RoutineOccurrence {
        routine_id,
        bucket,
        state: EtatOccurrence::Ouverte,
        reason: Some("mandat_adopte".to_string()),
        objective_id: Some(objective_id),
        delegation_id: Some(delegation_id),
        created_at: now,
    };
    store.insert_occurrence(&occ).map_err(routine_store_error)?;
    Ok(Some(occ))
}

fn routine_store_error(error: StoreError) -> RoutineError {
    RoutineError::Store(error.to_string())
}
