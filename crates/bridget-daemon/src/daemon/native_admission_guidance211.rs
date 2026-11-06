use super::*;
pub(super) fn refusal(code:&str)->Option<Value>{
    if let Some(detail)=code.strip_prefix("origin_request_invalid:") {
        let (field,cause)=detail.split_once(':').unwrap_or((detail,"unobserved"));
        let cause_text=match cause {
            "invalid_envelope"=>"la forme de l’enveloppe est invalide",
            "permissions_changed"=>"le fait de permissions est invalide",
            "resource_limit"=>"la taille admise est dépassée",
            "unobserved"=>"la cause précise n’est pas conservée",
            _=>return None,
        };
        if !matches!(field,"worker"|"version"|"correlation_id"|"connection_epoch"|"expires_at_ms"
            |"envelope"|"parent"|"blocked"|"root"|"worktree"|"expected_thread"|"proof"
            |"target_machine"|"project_id"|"cwd"|"expected_base_commit"|"workspace") {return None;}
        let mut view=refusal("origin_request_invalid")?;
        view["detail"]["field"]=json!(field);
        view["detail"]["cause"]=json!(cause);
        if cause=="resource_limit" {
            view["detail"]["limit_bytes"]=match field {"envelope"=>json!(128*1024),"workspace"=>json!(4096),_=>Value::Null};
        }
        view["detail"]["recovery"]=json!(format!("Le validateur du relais d’origine a refusé le contrôle {field} : {cause_text}. Faire vérifier ce champ par le responsable Bridget dans la même demande et le même fait de session. Ce nom ne donne ni la valeur reçue ni un droit supplémentaire. Conserver la clé et l’enveloppe ; lire detail.admission avant toute reprise. Ne pas créer de remplacement d’issue inconnue. Les autres travaux autorisés suivent leur propre état."));
        return Some(view);
    }
    let (message,recovery,field)=match code {
        "project_preparation_machine_mismatch"=>(
            "Faire vérifier l'identité provisionnée de la machine cible dans les registres source, maître et worker.",
            "Ces identités diffèrent, ou l'une a changé pendant la préparation. Ce refus concerne le raccordement de cette demande. Un dépôt peut déjà être préparé sur la cible ; le conserver, sans l'effacer ni rejouer une mission. Le responsable Bridget doit comparer worker_id, révision et empreinte à l'autorité native actuelle. Aucun changement de compte ou de droits n'est déduit. Lire l'admission durable avant une reprise ; les autres travaux autorisés suivent leur propre état. Demander une décision humaine seulement si l'identité cible voulue reste ambiguë.",
            None),
        "project_preparation_additional_root_changed"=>(
            "Faire vérifier l'autorisation de la racine additionnelle déjà utilisée par ce projet.",
            "La source, son propriétaire, son mode ou son accès diffère de l'autorisation connue, ou cette autorisation manque au maître. Une nouvelle destination privée seule ne change pas ce droit. Le responsable Bridget doit comparer ces champs dans les registres source et maître pour la même machine cible. Conserver les dépôts préparés, la clé et l'enveloppe. Ne pas ajouter un accès pour contourner le refus ; si un nouveau périmètre est réellement nécessaire, demander à l'utilisateur cette seule décision. Ce refus concerne cette admission ; les autres travaux autorisés suivent leur propre état.",
            None),
        "reply_timeout_out_of_range"=>(
            "Fixez une durée de mission comprise entre 60 et 86400 secondes, ou omettez reply_timeout pour garder 3600 secondes.",
            "reply_timeout borne la durée de la mission, pas l'attente de la réponse de cet outil. Ce refus concerne cette demande. Lire detail.admission : si un refus durable sans tâche est enregistré, corriger la durée dans une nouvelle demande identifiée ; un rejeu identique conserve ce refus. Une demande en cours ou d'issue inconnue garde sa clé et son enveloppe. Les autres travaux autorisés suivent leur propre état. Aucun changement de droits ni décision humaine n'est nécessaire pour corriger ce paramètre dans le périmètre déjà autorisé.",
            Some("reply_timeout")),
        "project_preparation_context_missing"=>(
            "Faire charger la correction du raccordement de projet sur le worker source et le maître.",
            "La préparation a répondu sans transmettre de correspondance utilisable au maître. Ce refus concerne cette admission ; l'identité et le projet ne sont pas déclarés inconnus. Le responsable Bridget doit vérifier les images chargées et le retour du contexte natif de préparation. Conserver la clé et l'enveloppe ; lire l'admission durable avant une reprise, sans mission de remplacement. Les autres travaux autorisés suivent leur propre état. Si la maintenance nécessaire n'est pas autorisée, demander seulement l'accord pour charger cette correction.",
            None),
        "expected_base_commit_required"=>(
            "Fournissez expected_base_commit : le SHA Git source complet manque.",
            "Dans cwd sur la machine du parent, lire le commit résolu : HEAD en shared, workspace.base_ref (HEAD si absent) en worktree. Donner le SHA complet de 40 caractères sans changer la version demandée. Le champ est requis dans un dépôt Git et pour compilation/worktree ; hors Git en general/shared seulement il peut être omis. Consulter detail.admission avant correction de l’enveloppe : une demande en cours ou d’issue inconnue ne doit pas être dupliquée ; un refus durable ne se rouvre pas au rejeu.",
            Some("expected_base_commit")),
        "cwd_outside_parent_project"=>(
            "Utilisez un cwd inclus dans le dossier attesté du parent.",
            "Un worktree voisin ou la racine Git peuvent appartenir au même dépôt tout en restant hors du périmètre de ce parent. Conserver les droits actuels : travailler dans son dossier autorisé, ou demander à l’utilisateur d’ouvrir le travail depuis le parent approprié. Vérifier l’issue de l’ancienne demande avant tout changement de clé ou de paramètres.",
            Some("cwd")),
        "project_mapping_preference_invalid"=>(
            "Faites vérifier le choix d’emplacement des prochaines missions sur cette machine.",
            "Plusieurs correspondances couvrent cette source et leur préférence est invalide ou cite une clé absente. Les tâches vivantes et les autres travaux autorisés suivent leur propre état. Demander au responsable de vérifier la section project_preferences et la présence du même mapping sur la source, le maître et le worker. Conserver les anciens mappings utilisés par les tâches vivantes. Ce refus ne déplace ni ne supprime leurs dossiers. Garder la clé et l’enveloppe ; lire detail.admission avant tout rejeu, attendre la fin de tous les appels et ne pas créer de remplacement d’issue inconnue.",
            None),
        "project_mapping_ambiguous"=>(
            "Faites préciser l’emplacement prévu pour les prochaines missions sur cette machine.",
            "Plusieurs correspondances couvrent cette source. Le responsable doit qualifier le nouvel emplacement puis déclarer une préférence unique entre mappings de même source et mêmes droits additionnels. Un identifiant project_id explicite reste strict ; ne pas le changer dans une admission déjà enregistrée. Ne remplacer ni l’ancien mapping ni un dossier vivant. Garder la clé et lire detail.admission avant un rejeu ; les mêmes paramètres et l’absence d’appel concurrent restent requis.",
            None),
        "project_mapping_mismatch"=>(
            "Faire vérifier la correspondance imposée pour cette demande.",
            "L’association demandée ne correspond pas à l’hôte ou au dossier source de ce parent. Conserver le projet, sa clé et son enveloppe ; comparer le mapping source, maître et worker avant toute correction autorisée. Ne pas modifier les droits ni déplacer un dossier actif. Lire detail.admission avant toute reprise ; une demande en cours ou d’issue inconnue ne doit pas être dupliquée.",
            Some("project_id")),
        "project_mapping_unknown"=>(
            "Faire vérifier les correspondances de projet examinées pour cette demande.",
            "Le code indique un refus de résolution de correspondance, sans prouver une absence sur tous les serveurs. Lire detail.project_checks lorsqu’il est présent : étape et machines concernées. Un ancien reçu sans ce détail ne permet pas de retrouver la machine ni de prouver qu’une cible était déjà choisie. Vérifier les registres source, maître et worker, le dossier source et project_id seulement s’il est imposé. Demander au responsable Bridget une préparation native autorisée ; ne pas inventer un identifiant, déplacer un dossier actif ou copier des clés. Ce code seul ne prouve aucune panne serveur.",
            Some("project_id")),
        "origin_request_invalid"=>(
            "Transmettre ce refus au responsable Bridget pour vérifier la source et son attestation.",
            "L’enveloppe du relais d’origine n’a pas passé sa validation. Le champ divergent n’est pas attesté par ce reçu. Ce texte ne renseigne pas la version de la façade. Vérifier le cwd dans le périmètre du parent, la forme du SHA et les faits de session sans élargir les droits. Une réattestation seule ne prouve pas la résolution. Conserver la clé et vérifier son état durable ; ne pas créer de doublon.",
            None),
        "brief_invalid"=>(
            "Validez le brief avec le prévol bridget_handoff action=preview.",
            "Lire le champ exact du refus de validation. objective et summary sont requis ; une référence kind=file exige label, host et path absolu. Une modification du brief change l’enveloppe. Si un reçu existe, conserver son canon et suivre son état ; ne pas modifier les paramètres d’un rejeu strict. Aucun ancien refus ne doit être relancé pour tester le format.",
            Some("brief")),
        "envelope_mismatch"=>(
            "Reprenez les paramètres d’origine de cette clé pour retrouver son issue.",
            "Cette clé appartient déjà à une enveloppe différente. Changer le brief, le SHA, le cwd ou le workspace constitue une nouvelle demande. Une nouvelle clé est permise seulement après clarification de l’ancienne issue et absence de mission à doubler. Ne pas modifier le reçu ou recréer une tâche inconnue.",
            None),
        "task_unavailable"=>(
            "Utilisez l’identifiant réel renvoyé par l’admission, depuis son parent propriétaire.",
            "Ce parent ne peut pas lire cette tâche avec cet identifiant. Ce refus ne distingue pas une tâche absente d’une tâche hors de ses droits. Vérifier l’identifiant et la session propriétaire ; ne pas utiliser un identifiant fictif et ne pas demander à un autre agent de contourner la propriété. Ne pas relancer une mission pour tester la lecture.",
            Some("task_id")),
        _=>return None
    };
    Some(json!({"status":"refused","code":code,
        "message":format!("{message} Ce refus concerne cette demande ; les autres travaux autorisés suivent leur propre état."),
        "detail":{"message":message,"recovery":recovery,"field":field,
        "human_action":"Si la résolution dépasse vos droits, demander à l’utilisateur une décision ciblée sur le dossier, le placement ou l’accès aux faits manquants décrits ci-dessus. Ne pas demander un arrêt global ou une augmentation de droits pour contourner le refus."}}))
}


fn is_mapping_code(code: &str) -> bool {
    matches!(code, "project_mapping_unknown" | "project_mapping_mismatch"
        | "project_mapping_ambiguous" | "project_mapping_preference_invalid")
}

/// Diagnostic borné. Les autres familles gardent leur code historique.
pub(super) fn mapping_context_error(stage: &str, refused: &BTreeMap<String, String>) -> String {
    // Une autre famille garde son code et son guidage propres, quel que soit
    // l'ordre des machines. Le contexte de correspondance reste partiel.
    if let Some(code) = refused.values().find(|code| !is_mapping_code(code)) {
        return code.clone();
    }
    let code = refused.values().next().map(String::as_str).unwrap_or("project_mapping_unknown");
    let checks = refused.iter().map(|(machine, reason)| json!({
        "machine_id": machine,
        "code": if is_mapping_code(reason) {reason.as_str()} else {"unobserved"},
    })).collect::<Vec<_>>();
    format!("routing_project_refused:{}", json!({"stage": stage,"code": code,"checks": checks}))
}

pub(super) fn contextual_refusal(encoded: &str) -> Value {
    let fallback = || {
        let mut view = refusal("project_mapping_unknown").expect("known refusal");
        view["detail"]["context_status"] = json!("unavailable");
        view
    };
    if encoded.len() > 8192 { return fallback(); }
    let Ok(value) = serde_json::from_str::<Value>(encoded) else { return fallback(); };
    let Some(object) = value.as_object().filter(|v|v.len()==3) else { return fallback(); };
    let Some(stage) = object.get("stage").and_then(Value::as_str)
        .filter(|v|matches!(*v,"project_preparation"|"candidate_mapping")) else { return fallback(); };
    let Some(code) = object.get("code").and_then(Value::as_str).filter(|v|is_mapping_code(v))
        else { return fallback(); };
    let Some(checks) = object.get("checks").and_then(Value::as_array)
        .filter(|v|!v.is_empty() && v.len()<=16) else { return fallback(); };
    let mut previous: Option<&str> = None;
    for check in checks {
        if !check.as_object().is_some_and(|v|v.len()==2) { return fallback(); }
        let Some(machine) = check["machine_id"].as_str()
            .filter(|v|v.len()<=256 && bridget_core::host_is_attested(v)
                && v.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'-'|b'_'))) else { return fallback(); };
        if previous.is_some_and(|v|v>=machine) { return fallback(); }
        previous=Some(machine);
        if !check["code"].as_str().is_some_and(|v|is_mapping_code(v)||v=="unobserved") {
            return fallback();
        }
    }
    if checks[0]["code"]!=code { return fallback(); }
    let mut result=refusal(code).expect("known refusal");
    result["message"]=json!("Faire vérifier les correspondances indiquées dans detail.project_checks, sur les machines et à l’étape nommées. Ce refus concerne cette demande, sans prouver un choix final de machine ni une panne globale. Les autres travaux autorisés suivent leur propre état.");
    result["detail"]["stage"]=json!(stage);
    result["detail"]["scope"]=json!("this_admission");
    result["detail"]["context_status"]=json!("observed");
    result["detail"]["execution_selection"]=json!("not_attested_by_this_diagnostic");
    result["detail"]["coverage"]=json!("mapping_refusals_only");
    result["detail"]["project_checks"]=json!(checks);
    result["detail"]["verification"]=json!("Les codes connus de correspondance sont conservés par machine. unobserved signifie que la cause précise de cet autre refus n’est pas publiée ici. Ce diagnostic ne prouve pas l’absence de correspondance sur tous les serveurs.");
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec221_project_context_keeps_machine_stage_code_and_closed_bounds() {
        for code in ["project_mapping_unknown","project_mapping_mismatch",
            "project_mapping_ambiguous","project_mapping_preference_invalid"] {
            let error=mapping_context_error("candidate_mapping",&BTreeMap::from([("cartae".into(),code.into())]));
            let view=contextual_refusal(error.strip_prefix("routing_project_refused:").unwrap());
            assert_eq!(view["code"],code);
            assert_eq!(view["detail"]["context_status"],"observed");
        }
        for (first, second) in [
            ("project_mapping_unknown", "origin_proof_busy"),
            ("project_preparation_timeout", "project_mapping_mismatch"),
            ("project_mapping_unknown", "project_preparation_busy"),
        ] {
            let refused=BTreeMap::from([
                ("cartae".into(),first.into()), ("sysle2".into(),second.into()),
            ]);
            let expected=if is_mapping_code(first) {second} else {first};
            assert_eq!(mapping_context_error("project_preparation",&refused),expected);
        }
        let causes=BTreeMap::from([
            ("cartae".into(),"project_mapping_unknown".into()),
            ("sysle2".into(),"project_mapping_mismatch".into()),
        ]);
        let error=mapping_context_error("project_preparation",&causes);
        let view=contextual_refusal(error.strip_prefix("routing_project_refused:").unwrap());
        assert_eq!(view["code"],"project_mapping_unknown");
        assert_eq!(view["detail"]["stage"],"project_preparation");
        assert_eq!(view["detail"]["project_checks"][0]["machine_id"],"cartae");
        assert_eq!(view["detail"]["project_checks"][1]["code"],"project_mapping_mismatch");
        assert!(!view.to_string().contains("PRIVATE"));
        assert_eq!(view["detail"]["execution_selection"],"not_attested_by_this_diagnostic");
        assert!(view.to_string().len()<8192);
        assert_eq!(mapping_context_error("project_preparation",&BTreeMap::from([
            ("cartae".into(),"permission_attestation_unavailable".into())])),
            "permission_attestation_unavailable");
        for raw in ["PRIVATE",r#"{"stage":"project_preparation","code":"PRIVATE","checks":[]}"#,
            r#"{"stage":"chosen","code":"project_mapping_unknown","checks":[{"machine_id":"cartae","code":"project_mapping_unknown"}]}"#] {
            let rejected=contextual_refusal(raw);
            assert_eq!(rejected["detail"]["context_status"],"unavailable");
            assert!(!rejected.to_string().contains("PRIVATE"));
        }
        let mut invalid=serde_json::from_str::<Value>(error.strip_prefix("routing_project_refused:").unwrap()).unwrap();
        invalid["checks"][0]["machine_id"]=json!("/private/path");
        assert_eq!(contextual_refusal(&invalid.to_string())["detail"]["context_status"],"unavailable");
        invalid["checks"]=json!((0..17).map(|i|json!({"machine_id":format!("host-{i:02}"),"code":"project_mapping_unknown"})).collect::<Vec<_>>());
        assert_eq!(contextual_refusal(&invalid.to_string())["detail"]["context_status"],"unavailable");
    }

    #[test]
    fn spec236_origin_validation_field_is_closed_and_not_a_version_claim() {
        let plain=refusal("origin_request_invalid").unwrap();
        assert!(plain["detail"]["field"].is_null());
        assert!(!plain.to_string().contains("ancien code"));
        for field in ["worker","version","correlation_id","connection_epoch","expires_at_ms",
            "envelope","parent","blocked","root","worktree","expected_thread","proof",
            "target_machine","project_id","cwd","expected_base_commit","workspace"] {
            let view=refusal(&format!("origin_request_invalid:{field}")).unwrap();
            assert_eq!(view["code"],"origin_request_invalid");
            assert_eq!(view["detail"]["field"],field);
            assert!(view["detail"]["recovery"].as_str().unwrap().contains(field));
        }
        for (field,limit) in [("envelope",128*1024),("workspace",4096)] {
            let view=refusal(&format!("origin_request_invalid:{field}:resource_limit")).unwrap();
            assert_eq!(view["detail"]["cause"],"resource_limit");
            assert_eq!(view["detail"]["limit_bytes"],limit);
        }
        let view=refusal("origin_request_invalid:parent:permissions_changed").unwrap();
        assert_eq!(view["detail"]["cause"],"permissions_changed");
        for field in ["secret/path","cwd:PRIVATE","","parent\nPRIVATE"] {
            assert!(refusal(&format!("origin_request_invalid:{field}")).is_none());
        }
    }

    #[test]
    fn spec211_refusals_name_action_scope_and_human_decision(){
        for code in ["expected_base_commit_required","cwd_outside_parent_project",
            "project_mapping_unknown","project_mapping_mismatch","project_mapping_preference_invalid","project_mapping_ambiguous","origin_request_invalid","brief_invalid","envelope_mismatch","task_unavailable"]{
            let r=refusal(code).unwrap();
            assert_eq!(r["code"],code);
            assert!(r["message"].as_str().unwrap().contains("cette demande"));
            assert!(r["message"].as_str().unwrap().contains("autres travaux"));
            assert!(r["detail"]["recovery"].as_str().unwrap().len()>120);
            assert!(r["detail"]["human_action"].as_str().unwrap().contains("utilisateur"));
        }
        assert!(refusal("task_unavailable").unwrap()["detail"]["recovery"].as_str().unwrap().contains("ne distingue pas"));
        assert!(refusal("origin_request_invalid").unwrap()["detail"]["recovery"].as_str().unwrap().contains("n’est pas attesté"));
        assert!(refusal("unknown:private").is_none());
    }
}
