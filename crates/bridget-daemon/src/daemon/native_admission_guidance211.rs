use super::*;
pub(super) fn refusal(code:&str)->Option<Value>{
    let (message,recovery,field)=match code {
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
        "project_mapping_unknown"=>(
            "Faire vérifier le placement natif de ce projet sur la machine choisie.",
            "Aucun mapping applicable n’a été trouvé pour cette demande. Vérifier la machine, project_id s’il est imposé et le dossier source. Demander au responsable Bridget une préparation native du projet autorisée ; ne pas inventer un project_id, déplacer un dossier actif ou copier des clés. L’absence de mapping ne prouve pas une panne du serveur.",
            Some("project_id")),
        "origin_request_invalid"=>(
            "Transmettre ce refus au responsable Bridget pour vérifier la source et son attestation.",
            "L’enveloppe du relais d’origine n’a pas passé sa validation. Le champ divergent n’est pas attesté par cet ancien code. Vérifier le cwd dans le périmètre du parent, la forme du SHA et les faits de session sans élargir les droits. Une réattestation seule ne prouve pas la résolution. Conserver la clé et vérifier son état durable ; ne pas créer de doublon.",
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec211_refusals_name_action_scope_and_human_decision(){
        for code in ["expected_base_commit_required","cwd_outside_parent_project",
            "project_mapping_unknown","project_mapping_preference_invalid","project_mapping_ambiguous","origin_request_invalid","brief_invalid","envelope_mismatch","task_unavailable"]{
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
