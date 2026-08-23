use maicie::domain::{
    ActivationOutbox, ApprobationActivation, ClasseDuree, Delegation, DomainError,
    EtatActivationOutbox, EtatActivationProfil, EtatDelegation, EtatFlux, EtatObjectif,
    EtatOutboxDelegation, ModeObjectif, ObjectifCoordonne, OutboxDelegation, ProfilEquipe,
    SnapshotTransport, SourceSnapshot,
};
use uuid::Uuid;

#[test]
fn objectif_refuse_toute_transition_arriere_et_ne_clot_que_explicitement() {
    let mut objectif = ObjectifCoordonne::nouveau("examiner le contrat", ModeObjectif::Delegue, 10)
        .expect("objectif valide");
    assert_eq!(objectif.transition(EtatObjectif::Synthetise, 11), Err(DomainError::TransitionInterdite));
    objectif.transition(EtatObjectif::EnCoordination, 11).unwrap();
    objectif.transition(EtatObjectif::AEvaluer, 12).unwrap();
    assert_eq!(objectif.transition(EtatObjectif::EnCoordination, 13), Err(DomainError::TransitionInterdite));
    objectif.clore(14).unwrap();
    assert_eq!(objectif.etat, EtatObjectif::Clos);
    assert_eq!(objectif.clore(15), Err(DomainError::TransitionInterdite));
}

#[test]
fn delegation_ne_termine_jamais_directement_apres_la_creation() {
    let mut delegation = Delegation::nouvelle(
        Uuid::new_v4(), "prospective", "vérifie les invariants", ClasseDuree::Normale, "cible explicite",
    )
    .unwrap();
    assert_eq!(delegation.transition(EtatDelegation::Terminee), Err(DomainError::TransitionInterdite));
    delegation.transition(EtatDelegation::AEvaluer).unwrap();
    delegation.transition(EtatDelegation::Terminee).unwrap();
    assert_eq!(delegation.annuler(), Err(DomainError::TransitionInterdite));
}

#[test]
fn outbox_delegation_garde_l_enveloppe_et_son_horizon_bridget() {
    let mut outbox = OutboxDelegation {
        message_id: Uuid::new_v4(), delegation_id: Uuid::new_v4(), target: "prospective".into(),
        body_bytes: b"corps exact".to_vec(), reply: true, timeout_secs: 60,
        deadline_contractuelle: 160, body_hash: vec![1], etat: EtatOutboxDelegation::Prepared,
        attempted_at: None, retry_until: 150, dedup_retained_until: 160,
    };
    outbox.verifier().unwrap();
    outbox.transition(EtatOutboxDelegation::OutcomeUnknown, 100).unwrap();
    outbox.transition(EtatOutboxDelegation::Accepted, 101).unwrap();
    assert_eq!(outbox.transition(EtatOutboxDelegation::OutcomeUnknown, 102), Err(DomainError::TransitionInterdite));
    outbox.retry_until = 161;
    assert_eq!(outbox.verifier(), Err(DomainError::DonneeInvalide("retry hors horizon Bridget")));
}

#[test]
fn snapshot_acp_exige_la_correlation_complete_du_flux() {
    let incomplete = SnapshotTransport {
        message_id: Uuid::new_v4(), request_state: None, observed_at: 10,
        source: SourceSnapshot::AcpSubscription, subscription_id: Some("sub-1".into()),
        seq: None, stream_state: EtatFlux::Gap,
    };
    assert_eq!(incomplete.verifier(), Err(DomainError::DonneeInvalide("corrélation de snapshot invalide")));
}

#[test]
fn approbation_est_mono_usage_expiree_et_revalidee_contre_toctou() {
    let mut approval = ApprobationActivation {
        id: Uuid::new_v4(), objective_id: Uuid::new_v4(), profile_id: "security".into(),
        profile_hash: vec![1], context_hash: vec![2], context_scope: "minimal".into(),
        parameters: "{}".into(), actor: "local_human".into(), expires_at: 100, consumed_at: None,
    };
    assert_eq!(approval.consommer(10, &[9], &[2]), Err(DomainError::ApprobationIncoherente));
    approval.consommer(10, &[1], &[2]).unwrap();
    assert_eq!(approval.consommer(11, &[1], &[2]), Err(DomainError::ApprobationConsommee));
    let mut expired = approval.clone();
    expired.consumed_at = None;
    assert_eq!(expired.consommer(100, &[1], &[2]), Err(DomainError::ApprobationExpiree));
}

#[test]
fn activation_outbox_ne_revient_pas_apres_une_issue_durable() {
    let mut outbox = ActivationOutbox {
        command_id: Uuid::new_v4(), approval_id: Uuid::new_v4(), spawn_order_bytes: vec![1],
        etat: EtatActivationOutbox::Dispatching, retry_until: 100, dedup_retained_until: 100,
    };
    outbox.verifier().unwrap();
    outbox.transition(EtatActivationOutbox::OutcomeUnknown).unwrap();
    outbox.transition(EtatActivationOutbox::Applied).unwrap();
    assert_eq!(outbox.transition(EtatActivationOutbox::Dispatching), Err(DomainError::TransitionInterdite));
}

#[test]
fn profil_ne_peut_etre_lance_sans_approbation_et_domaine_est_serialisable() {
    let mut profile = ProfilEquipe {
        id: "sentry".into(), nom_affiche: "Sentry".into(), capacites: vec!["security".into()],
        personnalite: "strict".into(), outils: vec!["audit".into()],
        spawn_order_ref: "fleet/sentry".into(), etat_activation: EtatActivationProfil::Inactif,
    };
    assert_eq!(profile.transition_activation(EtatActivationProfil::Lance), Err(DomainError::TransitionInterdite));
    profile.transition_activation(EtatActivationProfil::Proposition).unwrap();
    profile.transition_activation(EtatActivationProfil::Approuve).unwrap();
    profile.transition_activation(EtatActivationProfil::Lance).unwrap();
    let objective = ObjectifCoordonne::nouveau("contrat durable", ModeObjectif::Collaboratif, 1).unwrap();
    let encoded = serde_json::to_vec(&objective).expect("objectif sérialisable");
    let decoded: ObjectifCoordonne = serde_json::from_slice(&encoded).expect("objectif désérialisable");
    assert_eq!(decoded, objective);
}
