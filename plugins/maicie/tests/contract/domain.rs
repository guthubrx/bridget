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
    assert_eq!(
        objectif.transition(EtatObjectif::Synthetise, 11),
        Err(DomainError::TransitionInterdite)
    );
    objectif
        .transition(EtatObjectif::EnCoordination, 11)
        .unwrap();
    objectif.transition(EtatObjectif::AEvaluer, 12).unwrap();
    assert_eq!(
        objectif.transition(EtatObjectif::EnCoordination, 13),
        Err(DomainError::TransitionInterdite)
    );
    objectif.clore(14).unwrap();
    assert_eq!(objectif.etat, EtatObjectif::Clos);
    assert_eq!(objectif.clore(15), Err(DomainError::TransitionInterdite));
}

#[test]
fn delegation_ne_termine_jamais_directement_apres_la_creation() {
    let mut delegation = Delegation::nouvelle(
        Uuid::new_v4(),
        "prospective",
        "vérifie les invariants",
        ClasseDuree::Normale,
        "cible explicite",
    )
    .unwrap();
    assert_eq!(
        delegation.transition(EtatDelegation::Terminee),
        Err(DomainError::TransitionInterdite)
    );
    delegation.transition(EtatDelegation::AEvaluer).unwrap();
    delegation.transition(EtatDelegation::Terminee).unwrap();
    assert_eq!(delegation.annuler(), Err(DomainError::TransitionInterdite));
}

/// Garde `ALL` : match exhaustif + appartenance. Des témoins sont construits
/// **hors** de `ALL` ; chaque bras du garde assert la présence. Une variante
/// neuve casse la compilation du match ; l'omettre de `ALL` fait échouer
/// l'assertion dès que le témoin est ajouté.
///
/// Mutant mesuré (relec) : classer `SoldeeParCloture` partout, laisser `ALL`
/// à 5 → `sql_in_clause(est_terminal)` omettait l'état. Avec la macro source
/// unique, ce mutant est impossible ; ce test prouve aussi la bijection
/// témoins ↔ `ALL` et que la clause SQL suit chaque terminal de `ALL`.
#[test]
fn all_est_garanti_par_match_exhaustif() {
    // Témoins hors de `ALL` — liste que le compilateur force à croître avec
    // le match de `assert_listed_in_all` (même ensemble de variantes).
    let temoins = [
        EtatDelegation::EnAttentePrerequis,
        EtatDelegation::Creee,
        EtatDelegation::AEvaluer,
        EtatDelegation::Terminee,
        EtatDelegation::Annulee,
    ];
    for etat in temoins {
        etat.assert_listed_in_all();
    }
    assert_eq!(
        temoins.len(),
        EtatDelegation::ALL.len(),
        "témoins et ALL doivent avoir la même cardinalité"
    );
    for etat in EtatDelegation::ALL {
        assert!(
            temoins.contains(etat),
            "{etat:?} dans ALL mais absent des témoins du garde"
        );
    }
    // La jonction classification ↔ énumération : tout terminal de ALL
    // apparaît dans la clause SQL (le trou exact du mutant relec).
    let clause = EtatDelegation::sql_in_clause(EtatDelegation::est_terminal);
    for etat in EtatDelegation::ALL {
        if etat.est_terminal() {
            assert!(
                clause.contains(etat.as_sql()),
                "sql_in_clause(est_terminal) omet '{}' pourtant est_terminal — \
                 ALL et la classification ont divergé",
                etat.as_sql()
            );
        } else {
            assert!(
                !clause.contains(&format!("'{}'", etat.as_sql())),
                "sql_in_clause(est_terminal) contient '{}' non terminal",
                etat.as_sql()
            );
        }
    }
}

#[test]
fn outbox_delegation_garde_l_enveloppe_et_son_horizon_bridget() {
    let mut outbox = OutboxDelegation {
        message_id: Uuid::new_v4(),
        delegation_id: Uuid::new_v4(),
        target: "prospective".into(),
        body_bytes: b"corps exact".to_vec(),
        reply: true,
        timeout_secs: 60,
        deadline_contractuelle: 160,
        body_hash: vec![1],
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: 150,
        dedup_retained_until: 160,
    };
    outbox.verifier().unwrap();
    outbox
        .transition(EtatOutboxDelegation::OutcomeUnknown, 100)
        .unwrap();
    outbox
        .transition(EtatOutboxDelegation::Accepted, 101)
        .unwrap();
    assert_eq!(
        outbox.transition(EtatOutboxDelegation::OutcomeUnknown, 102),
        Err(DomainError::TransitionInterdite)
    );
    outbox.retry_until = 161;
    assert_eq!(
        outbox.verifier(),
        Err(DomainError::DonneeInvalide("retry hors horizon Bridget"))
    );
}

#[test]
fn snapshot_acp_exige_la_correlation_complete_du_flux() {
    let incomplete = SnapshotTransport {
        message_id: Uuid::new_v4(),
        request_state: None,
        observed_at: 10,
        source: SourceSnapshot::AcpSubscription,
        subscription_id: Some("sub-1".into()),
        seq: None,
        stream_state: EtatFlux::Gap,
    };
    assert_eq!(
        incomplete.verifier(),
        Err(DomainError::DonneeInvalide(
            "corrélation de snapshot invalide"
        ))
    );
}

#[test]
fn approbation_est_mono_usage_expiree_et_revalidee_contre_toctou() {
    let mut approval = ApprobationActivation {
        id: Uuid::new_v4(),
        command_id: Uuid::new_v4(),
        objective_id: Uuid::new_v4(),
        profile_id: "security".into(),
        profile_hash: vec![1],
        context_hash: vec![2],
        context_scope: "minimal".into(),
        parameters: "{}".into(),
        actor: "local_human".into(),
        expires_at: 100,
        consumed_at: None,
    };
    assert_eq!(
        approval.consommer(10, &[9], &[2]),
        Err(DomainError::ApprobationIncoherente)
    );
    approval.consommer(10, &[1], &[2]).unwrap();
    assert_eq!(
        approval.consommer(11, &[1], &[2]),
        Err(DomainError::ApprobationConsommee)
    );
    let mut expired = approval.clone();
    expired.consumed_at = None;
    assert_eq!(
        expired.consommer(100, &[1], &[2]),
        Err(DomainError::ApprobationExpiree)
    );
}

#[test]
fn activation_outbox_ne_revient_pas_apres_une_issue_durable() {
    let mut outbox = ActivationOutbox {
        command_id: Uuid::new_v4(),
        approval_id: Uuid::new_v4(),
        spawn_order_bytes: vec![1],
        etat: EtatActivationOutbox::Dispatching,
        retry_until: 100,
        dedup_retained_until: 100,
    };
    outbox.verifier().unwrap();
    outbox
        .transition(EtatActivationOutbox::OutcomeUnknown)
        .unwrap();
    outbox.transition(EtatActivationOutbox::Applied).unwrap();
    assert_eq!(
        outbox.transition(EtatActivationOutbox::Dispatching),
        Err(DomainError::TransitionInterdite)
    );
}

#[test]
fn outbox_rejetee_est_terminale_et_synthese_est_requise() {
    assert!(
        EtatOutboxDelegation::Prepared
            .transition_vers(EtatOutboxDelegation::Rejected)
            .is_ok()
    );
    assert_eq!(
        EtatOutboxDelegation::Rejected.transition_vers(EtatOutboxDelegation::Accepted),
        Err(DomainError::TransitionInterdite)
    );

    let mut objective =
        ObjectifCoordonne::nouveau("produire une synthèse", ModeObjectif::Collaboratif, 1).unwrap();
    objective
        .transition(EtatObjectif::EnCoordination, 2)
        .unwrap();
    objective.transition(EtatObjectif::AEvaluer, 3).unwrap();
    assert!(objective.transition(EtatObjectif::Synthetise, 4).is_err());
    objective.synthese = Some("faits observés".to_string());
    objective.transition(EtatObjectif::Synthetise, 4).unwrap();
}

#[test]
fn profil_ne_peut_etre_lance_sans_approbation_et_domaine_est_serialisable() {
    let mut profile = ProfilEquipe {
        id: "sentry".into(),
        nom_affiche: "Sentry".into(),
        capacites: vec!["security".into()],
        personnalite: "strict".into(),
        outils: vec!["audit".into()],
        spawn_order_ref: "fleet/sentry".into(),
        etat_activation: EtatActivationProfil::Inactif,
    };
    assert_eq!(
        profile.transition_activation(EtatActivationProfil::Lance),
        Err(DomainError::TransitionInterdite)
    );
    profile
        .transition_activation(EtatActivationProfil::Proposition)
        .unwrap();
    profile
        .transition_activation(EtatActivationProfil::Approuve)
        .unwrap();
    profile
        .transition_activation(EtatActivationProfil::Lance)
        .unwrap();
    let objective =
        ObjectifCoordonne::nouveau("contrat durable", ModeObjectif::Collaboratif, 1).unwrap();
    let encoded = serde_json::to_vec(&objective).expect("objectif sérialisable");
    let decoded: ObjectifCoordonne =
        serde_json::from_slice(&encoded).expect("objectif désérialisable");
    assert_eq!(decoded, objective);
}
