use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::config::DurationClasses;
use maicie::domain::{ClasseDuree, ObjectiveOpeningPermit, SuiteObjective};
use maicie::domain::{EtatFlux, ExecutionProjection, ExecutionReference};
use maicie::runtime::{
    RuntimeSignal, execution_projection_from_signal, persist_execution_projection_from_signal,
};
use maicie::store::MaicieStore;
use std::path::PathBuf;
use uuid::Uuid;

fn delegated_store() -> (PathBuf, MaicieStore, Uuid, Uuid) {
    let root = std::env::temp_dir().join(format!("maicie-execution-projection-{}", Uuid::new_v4()));
    let database = root.join("maicie.sqlite3");
    let mut store = MaicieStore::open(&database).unwrap();
    let request = DelegateRequest {
        goal: "projection runtime isolée",
        opening_permit: ObjectiveOpeningPermit::auto_generated(),
        explicit_target: Some("agent-064"),
        required_tags: &[],
        duration: ClasseDuree::Normale,
        reply: true,
        constat_id: None,
        review_target: None,
        suite: SuiteObjective::Aucune,
        depends_on: &[],
        references: &[],
        idempotency_key: "execution-projection-064",
        now: 1_726_000_000,
        retry_until: 1_726_000_030,
        dedup_retained_until: 1_726_000_060,
        max_frame_bytes: 256 * 1024,
    };
    let DelegateResult::Created(created) = delegate(
        &mut store,
        DurationClasses {
            short_secs: 30,
            normal_secs: 60,
            long_secs: 90,
        },
        "maicie",
        &[DelegationCandidate {
            name: "agent-064".to_string(),
            tags: Vec::new(),
            available: true,
            dnd: false,
        }],
        &request,
    )
    .unwrap() else {
        panic!("délégation attendue")
    };
    (root, store, created.objective_id, created.delegation_id)
}

fn reference() -> ExecutionReference {
    ExecutionReference {
        delegation_id: Uuid::new_v4(),
        submission_id: "submission-064".to_string(),
        execution_id: "execution-064".to_string(),
        agent_instance_id: "agent-instance-064".to_string(),
        provider_kind: "cursor".to_string(),
        provider_session_id: Some("session-064".to_string()),
        provider_turn_id: None,
        bound_at: 1_726_000_000,
    }
}

#[test]
fn reference_et_projection_runtime_restent_opaques_et_explicitement_fraiches() {
    let reference = reference();
    reference.verifier().unwrap();
    let projection = ExecutionProjection {
        reference,
        runtime_state: "completed".to_string(),
        waiting_reason: None,
        last_progress_at: Some(1_726_000_010),
        observation_cursor: 42,
        freshness: EtatFlux::Fresh,
        observed_at: 1_726_000_011,
        source_generation: 7,
    };
    projection.verifier().unwrap();
    assert_eq!(projection.freshness, EtatFlux::Fresh);
    assert_eq!(projection.reference.provider_kind, "cursor");
}

#[test]
fn gap_fin_et_indisponibilite_restent_des_observations_sans_decision_metier() {
    for freshness in [EtatFlux::Gap, EtatFlux::Ended, EtatFlux::Unavailable] {
        let projection = ExecutionProjection {
            reference: reference(),
            runtime_state: "unknown".to_string(),
            waiting_reason: Some("provider_unavailable".to_string()),
            last_progress_at: None,
            observation_cursor: 0,
            freshness,
            observed_at: 1_726_000_012,
            source_generation: 8,
        };
        projection.verifier().unwrap();
    }
}

#[test]
fn reference_incomplete_est_refusee_avant_persistance() {
    let mut invalid = reference();
    invalid.execution_id.clear();
    assert!(invalid.verifier().is_err());
}

#[test]
fn projection_persistee_ignore_un_curseur_ancien_sans_muter_la_delegation() {
    let (root, mut store, _objective_id, delegation_id) = delegated_store();
    let mut projection = ExecutionProjection {
        reference: ExecutionReference {
            delegation_id,
            submission_id: "submission-persisted".to_string(),
            execution_id: "execution-persisted".to_string(),
            agent_instance_id: "agent-persisted".to_string(),
            provider_kind: "codex".to_string(),
            provider_session_id: Some("thread-persisted".to_string()),
            provider_turn_id: Some("turn-persisted".to_string()),
            bound_at: 1_726_000_000,
        },
        runtime_state: "running".to_string(),
        waiting_reason: None,
        last_progress_at: Some(1_726_000_001),
        observation_cursor: 11,
        freshness: EtatFlux::Fresh,
        observed_at: 1_726_000_002,
        source_generation: 3,
    };
    assert!(store.upsert_execution_projection(&projection).unwrap());
    projection.runtime_state = "stale".to_string();
    projection.observation_cursor = 10;
    assert!(!store.upsert_execution_projection(&projection).unwrap());
    assert_eq!(
        store
            .execution_projection_for_delegation(delegation_id)
            .unwrap()
            .unwrap()
            .runtime_state,
        "running"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn signal_gap_est_projete_sans_fermer_la_delegation() {
    let projection = execution_projection_from_signal(
        reference(),
        &RuntimeSignal::Gap {
            subscription_id: "sub-064".to_string(),
            from_seq: 12,
            to_seq: 13,
            reason: Some("retard".to_string()),
        },
        1_726_000_013,
        9,
    )
    .unwrap();
    assert_eq!(projection.freshness, EtatFlux::Gap);
    assert_eq!(projection.runtime_state, "unknown");
    assert_eq!(projection.observation_cursor, 13);
    assert_eq!(projection.waiting_reason.as_deref(), Some("projection_gap"));
}

#[test]
fn les_faits_runtime_ne_cloturent_ni_ne_rouvrent_une_mission() {
    let (root, mut store, objective_id, delegation_id) = delegated_store();
    let before = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);

    for (index, (runtime_state, freshness)) in [
        ("completed", EtatFlux::Fresh),
        ("paused", EtatFlux::Fresh),
        ("blocked", EtatFlux::Fresh),
        ("usage_limit", EtatFlux::Fresh),
        ("budget_limit", EtatFlux::Fresh),
        ("terminated", EtatFlux::Fresh),
        ("unreachable", EtatFlux::Unavailable),
        ("stale", EtatFlux::Gap),
    ]
    .into_iter()
    .enumerate()
    {
        let projection = ExecutionProjection {
            reference: ExecutionReference {
                delegation_id,
                submission_id: format!("submission-runtime-{index}"),
                execution_id: format!("execution-runtime-{index}"),
                agent_instance_id: "agent-runtime-064".to_string(),
                provider_kind: "claude".to_string(),
                provider_session_id: None,
                provider_turn_id: None,
                bound_at: 1_726_000_000,
            },
            runtime_state: runtime_state.to_string(),
            waiting_reason: Some("provider_fact_only".to_string()),
            last_progress_at: None,
            observation_cursor: index as u64 + 1,
            freshness,
            observed_at: 1_726_000_100 + index as i64,
            source_generation: 1,
        };
        assert!(store.upsert_execution_projection(&projection).unwrap());
    }

    let after = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    assert_eq!(after.objective.etat, before.objective.etat);
    assert_eq!(after.delegations[0].etat, before.delegations[0].etat);
    assert_eq!(
        store
            .execution_projection_for_delegation(delegation_id)
            .unwrap()
            .unwrap()
            .runtime_state,
        "stale"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn consommation_curseur_gap_persiste_une_copie_sans_effet_metier() {
    let (root, mut store, objective_id, delegation_id) = delegated_store();
    let before = store
        .objective_snapshots(Some(objective_id))
        .unwrap()
        .remove(0);
    let reference = ExecutionReference {
        delegation_id,
        submission_id: "submission-consumer".to_string(),
        execution_id: "execution-consumer".to_string(),
        agent_instance_id: "agent-consumer".to_string(),
        provider_kind: "cursor".to_string(),
        provider_session_id: Some("session-consumer".to_string()),
        provider_turn_id: Some("turn-consumer".to_string()),
        bound_at: 1_726_000_000,
    };
    let changed = persist_execution_projection_from_signal(
        &mut store,
        reference,
        &RuntimeSignal::Gap {
            subscription_id: "sub-consumer".to_string(),
            from_seq: 20,
            to_seq: 21,
            reason: Some("preuve_absente".to_string()),
        },
        1_726_000_200,
        5,
    )
    .unwrap();
    assert_eq!(changed, Some(true));
    let projection = store
        .execution_projection_for_delegation(delegation_id)
        .unwrap()
        .unwrap();
    assert_eq!(projection.observation_cursor, 21);
    assert_eq!(projection.freshness, EtatFlux::Gap);
    assert_eq!(
        store
            .objective_snapshots(Some(objective_id))
            .unwrap()
            .remove(0)
            .objective
            .etat,
        before.objective.etat
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}
