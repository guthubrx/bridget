use super::*;

#[test]
fn remote162_lineage_version_de_projection_seulement_pour_list_show() {
    let mut request = HumanLineageRequest {
        version: 2,
        t3_thread_id: "thread".into(),
        project_root: "/project".into(),
        request: HumanLineageAction::List {
            limit: 20,
            cursor: None,
        },
    };
    assert_eq!(request.validate_response_version(), Ok(2));
    assert_eq!(
        request.request.capability(),
        ClientCapability::HumanLineageViewV1
    );
    request.request = HumanLineageAction::Show {
        task_id: "task".into(),
        offset: 0,
        limit: 20,
    };
    assert_eq!(request.validate_response_version(), Ok(2));
    request.request = HumanLineageAction::Watch;
    assert_eq!(
        request.validate_response_version(),
        Err(HumanLineageError::UnsupportedVersion)
    );
    request.version = 1;
    assert_eq!(request.validate_response_version(), Ok(1));
    request.request = HumanLineageAction::List {
        limit: 20,
        cursor: None,
    };
    request.version = HUMAN_LINEAGE_DECISION_VERSION;
    assert_eq!(
        request.validate_response_version(),
        Ok(HUMAN_LINEAGE_DECISION_VERSION)
    );
    request.request = HumanLineageAction::Watch;
    assert_eq!(
        request.validate_response_version(),
        Err(HumanLineageError::UnsupportedVersion)
    );
    assert_eq!(HumanLineageError::UnsupportedVersion.result()["version"], 1);
}

#[test]
fn remote162_lineage_wire_v1_inchange_et_placement_v2_ferme() {
    let wire = r#"{"version":1,"t3_thread_id":"t","project_root":"/p","request":{"action":"list","limit":20,"cursor":null}}"#;
    let request: HumanLineageRequest = serde_json::from_str(wire).unwrap();
    assert_eq!(serde_json::to_string(&request).unwrap(), wire);
    let placement = serde_json::json!({"machine_id":"cartae","display_name":null,
            "worker_id":"worker","last_contact_at":null,"contact_state":"unknown"});
    let decoded: LineagePlacementV2 = serde_json::from_value(placement.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), placement);
    let mut unknown = placement;
    unknown["credential"] = serde_json::json!("forbidden");
    assert!(serde_json::from_value::<LineagePlacementV2>(unknown).is_err());
}

#[test]
fn spec163_lineage_v3_decision_est_ferme_et_v1_v2_restent_distincts() {
    let summary = serde_json::json!({
        "mode":"automatic", "candidate_id":"c01", "reason":"profil adapté"
    });
    let decoded: LineageDecisionV3 = serde_json::from_value(summary.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), summary);
    let mut unknown = summary;
    unknown["endpoint"] = serde_json::json!("forbidden");
    assert!(serde_json::from_value::<LineageDecisionV3>(unknown).is_err());
    assert!(
        serde_json::from_value::<LineageDecisionV3>(serde_json::json!({
            "mode":"automatic", "candidate_id":"profile-secret"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<LineageDecisionV3>(serde_json::json!({
            "mode":"automatic", "candidate_id":"c08", "reason":"x".repeat(257)
        }))
        .is_err()
    );
    assert_eq!(
        serde_json::to_value(LineageDecisionV3 {
            mode: LineageDecisionModeV3::Fallback,
            candidate_id: "c08".into(),
            reason: None,
        })
        .unwrap(),
        serde_json::json!({"mode":"fallback","candidate_id":"c08"})
    );
}

#[test]
fn spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3() {
    let value = serde_json::json!({
        "candidate_id":"c128", "connection_id":"conn", "machine_id":"cartae",
        "engine":"codex", "model":"gpt-6.1-sol", "effort":null,
        "source":"deterministic"
    });
    let decoded: LineageDecisionV4 = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(serde_json::to_value(&decoded).unwrap(), value);
    assert!(
        decoded.facts.is_none(),
        "ancien reçu v4 conservé sans faits inventés"
    );
    let facts = serde_json::json!({
        "scored_at":1000,
        "resources":{"observed_at":990,"age_secs":10,"fresh":true,
            "cpu_percent":40,"available_memory_percent":50,
            "normalized_load_percent":20,"compilation_slots":2},
        "compilation_capacity":8,"continuity":"pinned_base",
        "quota":{"observed_at":995,"age_secs":5,"used_percent":20,"resets_at":1200},
        "score_terms":{"cpu":12,"memory":10,"load":16,"continuity":10,
            "compilation":8,"quota_remaining":4,"quota_reset":3,
            "saturation_penalty":0,"total":63}
    });
    let mut with_facts = value.clone();
    with_facts["facts"] = facts;
    let decoded: LineageDecisionV4 = serde_json::from_value(with_facts.clone()).unwrap();
    assert!(
        decoded.reason.is_none(),
        "les faits locaux ne dépendent pas d'une reason"
    );
    assert!(
        serde_json::to_vec(decoded.facts.as_ref().unwrap())
            .unwrap()
            .len()
            <= MAX_LINEAGE_DECISION_FACTS_BYTES
    );
    assert_eq!(serde_json::to_value(&decoded).unwrap(), with_facts);
    for (field, bad) in [
        (
            "resources",
            serde_json::json!({"observed_at":990,"age_secs":10,
                "fresh":true,"cpu_percent":101,"available_memory_percent":50,
                "normalized_load_percent":20,"compilation_slots":2}),
        ),
        (
            "quota",
            serde_json::json!({"observed_at":995,"age_secs":6,
                "used_percent":20,"resets_at":1200}),
        ),
        ("compilation_capacity", serde_json::json!(11)),
    ] {
        let mut invalid = with_facts.clone();
        invalid["facts"][field] = bad;
        assert!(
            serde_json::from_value::<LineageDecisionV4>(invalid).is_err(),
            "{field}"
        );
    }
    let mut unknown_key = with_facts.clone();
    unknown_key["facts"]["resources"]["credential"] = serde_json::json!("forbidden");
    assert!(serde_json::from_value::<LineageDecisionV4>(unknown_key).is_err());
    let mut invalid_total = decoded.clone();
    invalid_total.facts.as_mut().unwrap().score_terms.total += 1;
    assert!(
        serde_json::to_value(invalid_total).is_err(),
        "l'écriture des metadata applique la même validation que leur lecture"
    );

    let mut unknown_facts = with_facts.clone();
    unknown_facts["facts"]["resources"] = serde_json::json!({
        "observed_at":null,"age_secs":null,"fresh":false,"cpu_percent":null,
        "available_memory_percent":null,"normalized_load_percent":null,
        "compilation_slots":null
    });
    unknown_facts["facts"]["quota"] = serde_json::json!({
        "observed_at":null,"age_secs":null,"used_percent":null,"resets_at":null
    });
    unknown_facts["facts"]["score_terms"] = serde_json::json!({
        "cpu":0,"memory":0,"load":0,"continuity":10,"compilation":8,
        "quota_remaining":0,"quota_reset":0,"saturation_penalty":0,"total":18
    });
    let unknown_decoded: LineageDecisionV4 = serde_json::from_value(unknown_facts.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(unknown_decoded).unwrap(),
        unknown_facts,
        "inconnu reste null, jamais une mesure libre à zéro"
    );

    for credits in ["usable", "unusable", "unknown"] {
        let mut exhausted = with_facts.clone();
        exhausted["facts"]["quota"]["used_percent"] = serde_json::json!(100);
        exhausted["facts"]["quota"]["credits"] = serde_json::json!(credits);
        exhausted["facts"]["score_terms"]["quota_remaining"] = serde_json::json!(0);
        exhausted["facts"]["score_terms"]["total"] = serde_json::json!(59);
        let decoded: LineageDecisionV4 = serde_json::from_value(exhausted.clone()).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            exhausted,
            "quota inclus à100 et état de crédits sont deux faits distincts : {credits}"
        );
    }
    let mut invalid_credits = with_facts.clone();
    invalid_credits["facts"]["quota"]["credits"] = serde_json::json!({"balance":"opaque"});
    assert!(serde_json::from_value::<LineageDecisionV4>(invalid_credits).is_err());

    let mut bad_slots = with_facts.clone();
    bad_slots["facts"]["resources"]["compilation_slots"] = serde_json::json!(11);
    assert!(
        serde_json::from_value::<LineageDecisionV4>(bad_slots).is_err(),
        "compilation_slots=11 est refusé à la lecture"
    );
    let mut bad_slots = decoded.clone();
    bad_slots
        .facts
        .as_mut()
        .unwrap()
        .resources
        .compilation_slots = Some(11);
    assert!(
        serde_json::to_value(bad_slots).is_err(),
        "compilation_slots=11 est aussi refusé à l'écriture metadata"
    );

    let mut stale = unknown_facts.clone();
    stale["facts"]["resources"]["observed_at"] = serde_json::json!(1);
    stale["facts"]["resources"]["age_secs"] = serde_json::json!(999);
    stale["facts"]["resources"]["compilation_slots"] = serde_json::json!(2);
    stale["facts"]["quota"]["credits"] = serde_json::json!("unknown");
    let decoded: LineageDecisionV4 = serde_json::from_value(stale.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(decoded).unwrap(),
        stale,
        "date/âge périmés conservés, nombres inconnus null et capacité déclarée intacte"
    );
    assert_eq!(stale["facts"]["compilation_capacity"], 8);
    assert!(stale["facts"]["resources"]["cpu_percent"].is_null());

    let mut unknown = value;
    unknown["credential"] = serde_json::json!("forbidden");
    assert!(serde_json::from_value::<LineageDecisionV4>(unknown).is_err());
    assert!(
        serde_json::from_value::<LineageDecisionV4>(serde_json::json!({
            "candidate_id":"c08", "connection_id":"conn", "machine_id":"cartae",
            "engine":"codex", "model":"gpt", "effort":null, "source":"explicit"
        }))
        .is_err()
    );
    let request = HumanLineageRequest {
        version: HUMAN_LINEAGE_ROUTING_VERSION,
        t3_thread_id: "t".into(),
        project_root: "/p".into(),
        request: HumanLineageAction::List {
            limit: 1,
            cursor: None,
        },
    };
    assert_eq!(
        request.validate_response_version(),
        Ok(HUMAN_LINEAGE_ROUTING_VERSION)
    );
}
