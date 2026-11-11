//! Contrat filaire lineage 149 : aucun processus, socket ou environnement.
//! Chaque forme acceptée ou refusée est figée ici comme référence de codec.
use bridget_transport::protocol::{
    ClientCapability, DaemonToWrapper, HUMAN_COMPOSE_BODY_MAX_BYTES, HUMAN_LINEAGE_MAX_SEQ,
    HUMAN_LINEAGE_VERSION, HumanLineageAction, HumanLineageError, HumanLineageRequest,
    HumanLineageWatchEvent, WrapperToDaemon, decode, encode,
};
use serde_json::{Value, json};

#[test]
fn lineage149_version_et_borne_seq_figees() {
    assert_eq!(HUMAN_LINEAGE_VERSION, 1);
    // 2^53−1 : la plus grande séquence consommable par un client JavaScript.
    assert_eq!(HUMAN_LINEAGE_MAX_SEQ, 9_007_199_254_740_991);
    assert_eq!(HUMAN_LINEAGE_MAX_SEQ, (1u64 << 53) - 1);
}

#[test]
fn lineage149_actions_snake_case_roundtrip_et_fermeture() {
    let demandes = [
        json!({"action":"list","limit":50,"cursor":null}),
        json!({"action":"list","limit":50,"cursor":"curseur"}),
        json!({"action":"show","task_id":"14900000-0000-4000-8000-000000000001","offset":0,"limit":16384}),
        json!({"action":"journal","task_id":"14900000-0000-4000-8000-000000000001","after_seq":7,"limit":50,"follow":false}),
        json!({"action":"journal","task_id":"14900000-0000-4000-8000-000000000001","after_seq":0,"limit":50,"follow":true}),
        json!({"action":"watch"}),
        json!({"action":"cancel","task_id":"14900000-0000-4000-8000-000000000001","request_id":"49000000-0000-4000-8000-000000000002"}),
    ];
    for demande in demandes {
        let action: HumanLineageAction = serde_json::from_value(demande.clone())
            .unwrap_or_else(|_| panic!("{}", demande.to_string()));
        // Round-trip sémantique : le fil ne peut pas dériver du contrat.
        let line = encode(&action).unwrap();
        assert_eq!(
            serde_json::to_value(decode::<HumanLineageAction>(&line).unwrap()).unwrap(),
            demande,
            "{demande}"
        );
        assert_eq!(serde_json::to_value(&action).unwrap(), demande, "{demande}");
    }
    // Formes ouvertes refusées : action inconnue et champ en trop.
    for refuse in [
        json!({"action":"purge","task_id":"x"}),
        json!({"action":"list","limit":50,"cursor":null,"task_id":"en-trop"}),
    ] {
        assert!(
            serde_json::from_value::<HumanLineageAction>(refuse.clone()).is_err(),
            "action acceptée à tort : {refuse}"
        );
    }
    // La requête entière refuse aussi les champs inconnus.
    let requete = json!({"version":1,"t3_thread_id":"fil","project_root":"/tmp",
        "request":{"action":"watch"},"extra":1});
    assert!(serde_json::from_value::<HumanLineageRequest>(requete).is_err());
    let valide = json!({"version":1,"t3_thread_id":"fil","project_root":"/tmp","request":{"action":"watch"}});
    let relu = decode::<HumanLineageRequest>(
        &encode(&decode::<HumanLineageRequest>(&valide.to_string()).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(serde_json::to_value(&relu).unwrap(), valide);
}

#[test]
fn lineage149_capacite_exigee_par_action() {
    use bridget_transport::protocol::ClientCapability;
    let vue = json!({"action":"list","limit":50,"cursor":null});
    let action: HumanLineageAction = serde_json::from_value(vue).unwrap();
    assert_eq!(action.capability(), ClientCapability::HumanLineageViewV1);
    let show = json!({"action":"show","task_id":"14900000-0000-4000-8000-000000000001","offset":0,"limit":100});
    assert_eq!(
        serde_json::from_value::<HumanLineageAction>(show)
            .unwrap()
            .capability(),
        ClientCapability::HumanLineageViewV1
    );
    let journal = json!({"action":"journal","task_id":"14900000-0000-4000-8000-000000000001","after_seq":0,"limit":50,"follow":false});
    assert_eq!(
        serde_json::from_value::<HumanLineageAction>(journal)
            .unwrap()
            .capability(),
        ClientCapability::HumanLineageViewV1
    );
    let watch = json!({"action":"watch"});
    assert_eq!(
        serde_json::from_value::<HumanLineageAction>(watch)
            .unwrap()
            .capability(),
        ClientCapability::HumanLineageWatchV1
    );
    let cancel = json!({"action":"cancel","task_id":"14900000-0000-4000-8000-000000000001","request_id":"49000000-0000-4000-8000-000000000002"});
    assert_eq!(
        serde_json::from_value::<HumanLineageAction>(cancel)
            .unwrap()
            .capability(),
        ClientCapability::HumanLineageCancelV1
    );
    // Les noms filaires exacts : la négociation client les annonce ainsi.
    assert_eq!(
        serde_json::to_value(ClientCapability::HumanLineageViewV1).unwrap(),
        "human_lineage_view_v1"
    );
    assert_eq!(
        serde_json::to_value(ClientCapability::HumanLineageWatchV1).unwrap(),
        "human_lineage_watch_v1"
    );
    assert_eq!(
        serde_json::to_value(ClientCapability::HumanLineageCancelV1).unwrap(),
        "human_lineage_cancel_v1"
    );
}

/// Forme d'écriture `compose` : opération fermée, capacité dédiée, bornes
/// locales. Le daemon garde la corrélation tâche/ronde/instance.
fn compose(operation: Value) -> Value {
    json!({"action":"compose","task_id":"14900000-0000-4000-8000-000000000001",
        "expected_round":3,"request_id":"49000000-0000-4000-8000-000000000002",
        "operation":operation})
}

#[test]
fn spec258_compose_decode_les_quatre_operations_avec_capacite_dediee() {
    let forms = [
        json!({"kind":"read"}),
        json!({"kind":"send","body":"bonjour","intent":"queue"}),
        json!({"kind":"send","body":"ligne\nsuite","intent":"steer"}),
        json!({"kind":"select_runtime","model":"gpt-6.1-sol","effort":"high"}),
        json!({"kind":"select_runtime","model":"simple","effort":null}),
        json!({"kind":"interrupt"}),
    ];
    for operation in forms {
        let action: HumanLineageAction = serde_json::from_value(compose(operation.clone()))
            .unwrap_or_else(|e| panic!("{operation}: {e}"));
        assert_eq!(action.capability(), ClientCapability::HumanLineageComposeV1);
        assert_eq!(serde_json::to_value(&action).unwrap(), compose(operation));
    }
    assert_eq!(
        serde_json::to_value(ClientCapability::HumanLineageComposeV1).unwrap(),
        "human_lineage_compose_v1"
    );
}

#[test]
fn spec258_catalogue_capacite_absente_legacy_et_lecture_sans_effet() {
    let legacy = json!({"type":"ObservationCapabilities","events":[]});
    assert!(matches!(
        serde_json::from_value::<WrapperToDaemon>(legacy.clone()).unwrap(),
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: false,
            ..
        }
    ));
    assert_eq!(
        serde_json::to_value(WrapperToDaemon::ObservationCapabilities {
            events: Vec::new(),
            runtime_catalogue: false
        })
        .unwrap(),
        legacy
    );
    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum Legacy {
        ObservationCapabilities {
            events: Vec<bridget_transport::protocol::ObservationKind>,
        },
    }
    let new = json!({"type":"ObservationCapabilities","events":[],"runtime_catalogue":true});
    let Legacy::ObservationCapabilities { events } = serde_json::from_value(new).unwrap();
    assert!(events.is_empty());
    for invalid in [
        json!({"status":"observed","models":[],"selection":{"model":"forged"}}),
        json!({"status":"mystery"}),
    ] {
        assert!(
            serde_json::from_value::<bridget_transport::protocol::RuntimeCatalogueOutcome>(invalid)
                .is_err()
        );
    }
}

#[test]
fn spec258_compose_refuse_operation_ou_champ_inconnu() {
    for operation in [
        json!({"kind":"resume"}),
        json!({"kind":"send","body":"x","intent":"interrupt"}),
        json!({"kind":"send","body":"x","intent":"queue","agent":"autre"}),
        json!({"kind":"interrupt","force":true}),
        json!({"kind":"read","body":null}),
        json!({"kind":"read","intent":"queue"}),
        json!({"kind":"select_runtime","model":"m"}),
    ] {
        assert!(
            serde_json::from_value::<HumanLineageAction>(compose(operation.clone())).is_err(),
            "{operation}"
        );
    }
    let mut champ_inconnu = compose(json!({"kind":"read"}));
    champ_inconnu["agent_type"] = json!("claude");
    assert!(serde_json::from_value::<HumanLineageAction>(champ_inconnu).is_err());
}

#[test]
fn spec258_compose_bornes_locales_fermees() {
    let bornes = |operation: Value| {
        serde_json::from_value::<HumanLineageAction>(compose(operation))
            .map(|a| match a {
                HumanLineageAction::Compose {
                    operation,
                    expected_round,
                    ..
                } => operation.bounded(expected_round),
                _ => unreachable!(),
            })
            .unwrap()
    };
    assert!(bornes(json!({"kind":"interrupt"})));
    assert!(bornes(
        json!({"kind":"send","body":"a\tb\nc","intent":"queue"})
    ));
    assert!(bornes(
        json!({"kind":"select_runtime","model":"m","effort":"high"})
    ));
    assert!(!bornes(
        json!({"kind":"send","body":"   ","intent":"queue"})
    ));
    assert!(!bornes(
        json!({"kind":"send","body":"a\u{0007}b","intent":"steer"})
    ));
    let trop_long = "x".repeat(HUMAN_COMPOSE_BODY_MAX_BYTES + 1);
    assert!(!bornes(
        json!({"kind":"send","body":trop_long,"intent":"queue"})
    ));
    let limite = "x".repeat(HUMAN_COMPOSE_BODY_MAX_BYTES);
    assert!(bornes(
        json!({"kind":"send","body":limite,"intent":"queue"})
    ));
    assert!(!bornes(
        json!({"kind":"select_runtime","model":"","effort":"high"})
    ));
    assert!(!bornes(
        json!({"kind":"select_runtime","model":"m","effort":"hi gh"})
    ));
}

#[test]
fn spec258_compose_lecture_de_ronde_et_mutations_exigent_une_ronde_constatee() {
    let avec_ronde = |round: u64, operation: Value| {
        let mut value = compose(operation);
        value["expected_round"] = json!(round);
        serde_json::from_value::<HumanLineageAction>(value).map(|a| match a {
            HumanLineageAction::Compose {
                operation,
                expected_round,
                ..
            } => operation.bounded(expected_round),
            _ => unreachable!(),
        })
    };
    assert!(avec_ronde(0, json!({"kind":"read"})).unwrap());
    assert!(!avec_ronde(3, json!({"kind":"read"})).unwrap());
    assert!(!avec_ronde(0, json!({"kind":"interrupt"})).unwrap());
    assert!(!avec_ronde(0, json!({"kind":"send","body":"x","intent":"queue"})).unwrap());
    assert!(
        avec_ronde(
            1,
            json!({"kind":"select_runtime","model":"m","effort":"high"})
        )
        .unwrap()
    );
    assert!(avec_ronde(u32::MAX as u64 + 1, json!({"kind":"read"})).is_err());
}

#[test]
fn spec258_reponse_compose_fermee_et_nullable() {
    use bridget_transport::protocol::{HumanComposeResponseV1, HumanComposeStatus};
    let reponse = json!({
        "version": 1,
        "status": "ok",
        "request_id": "49000000-0000-4000-8000-000000000002",
        "task_id": "14900000-0000-4000-8000-000000000001",
        "round": 3,
        "outcome": "observed",
        "code": null,
        "message": "",
        "state": "running",
        "selection": null,
        "models": [{"id": "gpt-6.1-sol", "name": "GPT 6.1 Sol", "efforts": ["low", "high"]}],
        "capabilities": {"send": true, "steer": true, "interrupt": true,
            "select_runtime": true, "attachments": false}
    });
    let decoded: HumanComposeResponseV1 = serde_json::from_value(reponse.clone()).unwrap();
    assert_eq!(decoded.status, HumanComposeStatus::Ok);
    assert_eq!(serde_json::to_value(&decoded).unwrap(), reponse);
    let mut selected = reponse.clone();
    selected["selection"] = json!({"model": "gpt-6.1-sol", "effort": null});
    assert!(serde_json::from_value::<HumanComposeResponseV1>(selected).is_ok());
    let mut champ = reponse.clone();
    champ["agent_type"] = json!("claude");
    assert!(serde_json::from_value::<HumanComposeResponseV1>(champ).is_err());
    let mut sans_null = reponse;
    sans_null.as_object_mut().unwrap().remove("code");
    assert!(serde_json::from_value::<HumanComposeResponseV1>(sans_null).is_err());
}

#[test]
fn lineage149_evenements_watch_fermes() {
    let evenements = [
        json!({"status":"ready","version":1,"generation":"14900000-0000-4000-8000-000000000003","seq":0}),
        json!({"status":"changed","version":1,"generation":"14900000-0000-4000-8000-000000000003","seq":42}),
        json!({"status":"resync","version":1,"generation":"14900000-0000-4000-8000-000000000004","seq":1}),
        json!({"status":"error","version":1,"code":"binding_unavailable"}),
    ];
    for evenement in evenements {
        let event: HumanLineageWatchEvent = serde_json::from_value(evenement.clone())
            .unwrap_or_else(|_| panic!("{}", evenement.to_string()));
        let line = encode(&event).unwrap();
        assert_eq!(
            serde_json::to_value(decode::<HumanLineageWatchEvent>(&line).unwrap()).unwrap(),
            evenement
        );
        assert_eq!(serde_json::to_value(&event).unwrap(), evenement);
    }
    // Champ en trop, statut inconnu et champ manquant : fermés.
    for refuse in [
        json!({"status":"ready","version":1,"generation":"g","seq":0,"extra":1}),
        json!({"status":"pending","version":1,"generation":"g","seq":0}),
        json!({"status":"ready","version":1,"generation":"g"}),
    ] {
        assert!(
            serde_json::from_value::<HumanLineageWatchEvent>(refuse.clone()).is_err(),
            "événement accepté à tort : {refuse}"
        );
    }
}

#[test]
fn lineage149_codes_erreur_fermes_et_result() {
    let codes = [
        ("unsupported_version", HumanLineageError::UnsupportedVersion),
        ("invalid_request", HumanLineageError::InvalidRequest),
        ("binding_unavailable", HumanLineageError::BindingUnavailable),
        ("project_mismatch", HumanLineageError::ProjectMismatch),
        ("task_unavailable", HumanLineageError::TaskUnavailable),
        ("snapshot_changed", HumanLineageError::SnapshotChanged),
        ("journal_unavailable", HumanLineageError::JournalUnavailable),
        (
            "result_offset_invalid",
            HumanLineageError::ResultOffsetInvalid,
        ),
        ("envelope_mismatch", HumanLineageError::EnvelopeMismatch),
        ("resource_limit", HumanLineageError::ResourceLimit),
        ("store_unavailable", HumanLineageError::StoreUnavailable),
    ];
    for (nom, code) in codes {
        assert_eq!(serde_json::to_value(code).unwrap(), nom);
        let roundtrip: HumanLineageError = serde_json::from_value(json!(nom)).unwrap();
        assert_eq!(roundtrip, code);
        let result = code.result();
        assert_eq!(result["version"], 1);
        assert_eq!(result["status"], "error");
        assert_eq!(result["code"], nom);
        // Seule la panne de magasin est rejouable.
        assert_eq!(
            result["retryable"],
            code == HumanLineageError::StoreUnavailable
        );
        for cle in ["version", "status", "code", "retryable"] {
            assert!(result.get(cle).is_some(), "result sans {cle}");
        }
    }
    assert!(serde_json::from_value::<HumanLineageError>(json!("archive_inconnue")).is_err());
}

#[test]
fn lineage149_enveloppes_wire_humaines() {
    // Vers le daemon : {"type":"HumanLineage","request":{...}}.
    let requete = json!({
        "type":"HumanLineage",
        "request":{"version":1,"t3_thread_id":"fil-149","project_root":"/projet",
            "request":{"action":"list","limit":50,"cursor":null}}
    });
    let demande = decode::<WrapperToDaemon>(&requete.to_string())
        .unwrap_or_else(|_| panic!("{}", requete.to_string()));
    let line = encode(&demande).unwrap();
    assert_eq!(
        serde_json::to_value(decode::<WrapperToDaemon>(&line).unwrap()).unwrap(),
        requete
    );
    let relu: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(relu["type"], "HumanLineage");
    assert_eq!(relu["request"]["version"], 1);
    assert_eq!(relu["request"]["t3_thread_id"], "fil-149");
    assert_eq!(relu["request"]["project_root"], "/projet");
    assert_eq!(relu["request"]["request"]["action"], "list");

    // Depuis le daemon : résultat ponctuel.
    let resultat = json!({"type":"HumanLineageResult","result":{"version":1,"status":"ok"}});
    let reponse = decode::<DaemonToWrapper>(&resultat.to_string()).unwrap();
    assert_eq!(
        serde_json::to_value(decode::<DaemonToWrapper>(&encode(&reponse).unwrap()).unwrap())
            .unwrap(),
        resultat
    );
    let relu: Value = serde_json::from_str(&encode(&reponse).unwrap()).unwrap();
    assert_eq!(relu["type"], "HumanLineageResult");
    assert_eq!(relu["result"]["status"], "ok");

    // Depuis le daemon : événement de watch.
    let evenement = json!({
        "type":"HumanLineageWatchEvent",
        "event":{"status":"ready","version":1,"generation":"14900000-0000-4000-8000-000000000003","seq":0}
    });
    let watch = decode::<DaemonToWrapper>(&evenement.to_string()).unwrap();
    assert_eq!(
        serde_json::to_value(decode::<DaemonToWrapper>(&encode(&watch).unwrap()).unwrap()).unwrap(),
        evenement
    );
    let relu: Value = serde_json::from_str(&encode(&watch).unwrap()).unwrap();
    assert_eq!(relu["type"], "HumanLineageWatchEvent");
    assert_eq!(relu["event"]["status"], "ready");
    assert_eq!(relu["event"]["seq"], 0);

    // Une enveloppe de type inconnu reste refusée.
    assert!(decode::<WrapperToDaemon>(r#"{"type":"HumanLineagePurge"}"#).is_err());
}
