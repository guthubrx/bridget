use super::*;

pub(super) fn refusal(code: &str) -> Option<Value> {
    if let Some(encoded) = code.strip_prefix("admission_cancelled:") {
        let workspace: Value = serde_json::from_str(encoded).ok()?;
        if !workspace["path"].is_string() || !workspace["branch"].is_string() {
            return None;
        }
        let mut view = refusal("admission_cancelled")?;
        view["detail"]["retained_workspace"] = workspace;
        return Some(view);
    }
    if let Some(detail) = code.strip_prefix("origin_request_invalid:") {
        let (field, cause) = detail.split_once(':').unwrap_or((detail, "unobserved"));
        let cause_text = match cause {
            "same_machine_target" => {
                "la cible désigne la machine source pour une stratégie dont ce relais ne prend pas en charge la préparation locale"
            }
            "invalid_envelope" => "la forme de l’enveloppe est invalide",
            "permissions_changed" => "le fait de permissions est invalide",
            "resource_limit" => "la taille admise est dépassée",
            "unobserved" => "la cause précise n’est pas conservée",
            _ => return None,
        };
        if !matches!(
            field,
            "worker"
                | "version"
                | "correlation_id"
                | "connection_epoch"
                | "expires_at_ms"
                | "envelope"
                | "parent"
                | "blocked"
                | "root"
                | "worktree"
                | "expected_thread"
                | "proof"
                | "target_machine"
                | "project_id"
                | "cwd"
                | "expected_base_commit"
                | "workspace"
        ) {
            return None;
        }
        let mut view = refusal("origin_request_invalid")?;
        view["detail"]["field"] = json!(field);
        view["detail"]["cause"] = json!(cause);
        if cause == "resource_limit" {
            view["detail"]["limit_bytes"] = match field {
                "envelope" => json!(128 * 1024),
                "workspace" => json!(4096),
                _ => Value::Null,
            };
        }
        view["detail"]["recovery"] = json!(format!(
            "Le validateur du relais d’origine a refusé le contrôle {field} : {cause_text}. Faire vérifier ce champ par le responsable Bridget dans la même demande et le même fait de session. Ce nom ne donne ni la valeur reçue ni un droit supplémentaire. Conserver la clé et l’enveloppe ; lire detail.admission avant toute reprise. Ne pas créer de remplacement d’issue inconnue. Les autres travaux autorisés suivent leur propre état."
        ));
        return Some(view);
    }
    let (message, recovery, field) = match code {
        "admission_cancelled" => (
            "Cette admission a été annulée par son propriétaire avant création de tâche.",
            "La clé conserve ce refus terminal : son rejeu ne crée aucune mission. Conserver les éventuels dossiers préparés ; si une création de worktree était déjà en cours, le journal du coordinateur conserve bridget_admission_workspace_retained avec la clé et le chemin laissé, sans annoncer un échec Git ni supprimer ce dossier. Une autre mission exige une nouvelle clé et son autorisation propre. Ce refus concerne cette admission ; les autres travaux autorisés suivent leur propre état.",
            None,
        ),
        "project_preparation_process_failed" => (
            "Faites lire le refus de préparation à la même heure dans la sortie d’erreur du worker source.",
            "La commande ssh de préparation a rendu un code de sortie non nul. Le responsable Bridget doit lire l’événement bridget_project_preparation à l’heure de ce refus dans la sortie d’erreur du service worker de la machine source du parent : journal de l’unité sous systemd, ou StandardErrorPath du plist chargé sous launchd. Rapprocher heure, opération inspect/prepare et exit_code ; correlation_id identifie le contrôle dans ce journal, mais ne figure pas dans ce reçu et ne suffit pas seul à retrouver la demande. Ces faits ne prouvent ni panne de réseau ni défaut de compte ou de droits ; la cause amont reste à vérifier avant une reprise. Conserver la clé, l’enveloppe et tout dépôt préparé ; lire detail.admission pour suivre une tâche existante ou une admission encore incertaine. Ne pas recréer une mission pour tester la liaison. Les autres travaux autorisés suivent leur propre état.",
            None,
        ),
        "project_preparation_transport_failed" => (
            "Faites vérifier le résultat du relais de préparation et le canal SSH configuré pour cette demande.",
            "Cet ancien code ne distingue pas un processus en échec d’une réponse hors borne et n’a pas conservé son opération ni son code de sortie. La cause reste inconnue. Conserver la clé, l’enveloppe et tout dépôt déjà préparé ; le responsable Bridget doit qualifier les faits disponibles sur le worker source et la cible, puis lire detail.admission avant une reprise. Ne déduire ni panne de réseau ni défaut de compte ou de droits. Ne pas recréer une mission d’issue inconnue. Les autres travaux autorisés suivent leur propre état.",
            None,
        ),
        "project_preparation_output_limit" => (
            "Faites vérifier la taille de la réponse de préparation, sans rejouer la mission.",
            "La réponse a dépassé la borne de lecture du relais de préparation. Ce refus ne prouve pas une panne SSH. Le responsable Bridget doit qualifier le format et la borne de réponse pour cette demande. Conserver la clé, l’enveloppe et tout dépôt déjà préparé ; lire detail.admission avant une reprise. Les autres travaux autorisés suivent leur propre état.",
            None,
        ),
        "project_preparation_git_required" => (
            "Utilisez le dossier existant sur sa machine source, ou faites vérifier une correspondance vers un dossier distant déjà présent.",
            "Le dossier source est hors Git et la destination inspectée n’existe pas. general/shared permet d’omettre expected_base_commit, mais ne copie pas automatiquement les fichiers hors Git vers une nouvelle machine. Pour rester sur la source, employer son machine_id déjà autorisé ; pour une autre machine, faire vérifier son dossier existant et sa correspondance native. Ne pas créer un dépôt Git ni transférer des documents pour contourner ce refus. Lire detail.admission avant de reprendre : une issue inconnue garde sa clé et son enveloppe ; un changement de machine nécessite une nouvelle demande identifiée après vérification qu’aucune tâche n’a été admise. Ce refus ne prouve pas un placement final ni une panne globale. Si le dossier distant requis n’est pas autorisé, demander cette seule décision à l’utilisateur.",
            Some("workspace"),
        ),
        "project_preparation_machine_mismatch" => (
            "Faire vérifier l'identité provisionnée de la machine cible dans les registres source, maître et worker.",
            "Ces identités diffèrent, ou l'une a changé pendant la préparation. Ce refus concerne le raccordement de cette demande. Un dépôt peut déjà être préparé sur la cible ; le conserver, sans l'effacer ni rejouer une mission. Le responsable Bridget doit comparer worker_id, révision et empreinte à l'autorité native actuelle. Aucun changement de compte ou de droits n'est déduit. Lire l'admission durable avant une reprise ; les autres travaux autorisés suivent leur propre état. Demander une décision humaine seulement si l'identité cible voulue reste ambiguë.",
            None,
        ),
        "project_preparation_additional_root_changed" => (
            "Faire vérifier l'autorisation de la racine additionnelle déjà utilisée par ce projet.",
            "La source, son propriétaire, son mode ou son accès diffère de l'autorisation connue, ou cette autorisation manque au maître. Une nouvelle destination privée seule ne change pas ce droit. Le responsable Bridget doit comparer ces champs dans les registres source et maître pour la même machine cible. Conserver les dépôts préparés, la clé et l'enveloppe. Ne pas ajouter un accès pour contourner le refus ; si un nouveau périmètre est réellement nécessaire, demander à l'utilisateur cette seule décision. Ce refus concerne cette admission ; les autres travaux autorisés suivent leur propre état.",
            None,
        ),
        "reply_timeout_out_of_range" => (
            "Fixez une durée de mission comprise entre 60 et 86400 secondes, ou omettez reply_timeout pour garder 3600 secondes.",
            "reply_timeout borne la durée de la mission, pas l'attente de la réponse de cet outil. Ce refus concerne cette demande. Lire detail.admission : si un refus durable sans tâche est enregistré, corriger la durée dans une nouvelle demande identifiée ; un rejeu identique conserve ce refus. Une demande en cours ou d'issue inconnue garde sa clé et son enveloppe. Les autres travaux autorisés suivent leur propre état. Aucun changement de droits ni décision humaine n'est nécessaire pour corriger ce paramètre dans le périmètre déjà autorisé.",
            Some("reply_timeout"),
        ),
        "project_preparation_context_missing" => (
            "Faire charger la correction du raccordement de projet sur le worker source et le maître.",
            "La préparation a répondu sans transmettre de correspondance utilisable au maître. Ce refus concerne cette admission ; l'identité et le projet ne sont pas déclarés inconnus. Le responsable Bridget doit vérifier les images chargées et le retour du contexte natif de préparation. Conserver la clé et l'enveloppe ; lire l'admission durable avant une reprise, sans mission de remplacement. Les autres travaux autorisés suivent leur propre état. Si la maintenance nécessaire n'est pas autorisée, demander seulement l'accord pour charger cette correction.",
            None,
        ),
        "expected_base_commit_required" => (
            "Fournissez expected_base_commit : le SHA Git source complet manque.",
            "Dans cwd sur la machine du parent, lire le commit résolu : HEAD en shared, workspace.base_ref (HEAD si absent) en worktree. Donner le SHA complet de 40 caractères sans changer la version demandée. Le champ est requis dans un dépôt Git et pour compilation/worktree ; hors Git en general/shared seulement il peut être omis. Consulter detail.admission avant correction de l’enveloppe : une demande en cours ou d’issue inconnue ne doit pas être dupliquée ; un refus durable ne se rouvre pas au rejeu.",
            Some("expected_base_commit"),
        ),
        "cwd_outside_parent_project" => (
            "Utilisez un cwd inclus dans le dossier attesté du parent.",
            "Un worktree voisin ou la racine Git peuvent appartenir au même dépôt tout en restant hors du périmètre de ce parent. Conserver les droits actuels : travailler dans son dossier autorisé, ou demander à l’utilisateur d’ouvrir le travail depuis le parent approprié. Vérifier l’issue de l’ancienne demande avant tout changement de clé ou de paramètres.",
            Some("cwd"),
        ),
        "project_mapping_preference_invalid" => (
            "Faites vérifier le choix d’emplacement des prochaines missions sur cette machine.",
            "Plusieurs correspondances couvrent cette source et leur préférence est invalide ou cite une clé absente. Les tâches vivantes et les autres travaux autorisés suivent leur propre état. Demander au responsable de vérifier la section project_preferences et la présence du même mapping sur la source, le maître et le worker. Conserver les anciens mappings utilisés par les tâches vivantes. Ce refus ne déplace ni ne supprime leurs dossiers. Garder la clé et l’enveloppe ; lire detail.admission avant tout rejeu, attendre la fin de tous les appels et ne pas créer de remplacement d’issue inconnue.",
            None,
        ),
        "project_mapping_ambiguous" => (
            "Faites préciser l’emplacement prévu pour les prochaines missions sur cette machine.",
            "Plusieurs correspondances couvrent cette source. Le responsable doit qualifier le nouvel emplacement puis déclarer une préférence unique entre mappings de même source et mêmes droits additionnels. Un identifiant project_id explicite reste strict ; ne pas le changer dans une admission déjà enregistrée. Ne remplacer ni l’ancien mapping ni un dossier vivant. Garder la clé et lire detail.admission avant un rejeu ; les mêmes paramètres et l’absence d’appel concurrent restent requis.",
            None,
        ),
        "project_mapping_mismatch" => (
            "Faire vérifier la correspondance imposée pour cette demande.",
            "L’association demandée ne correspond pas à l’hôte ou au dossier source de ce parent. Conserver le projet, sa clé et son enveloppe ; comparer le mapping source, maître et worker avant toute correction autorisée. Ne pas modifier les droits ni déplacer un dossier actif. Lire detail.admission avant toute reprise ; une demande en cours ou d’issue inconnue ne doit pas être dupliquée.",
            Some("project_id"),
        ),
        "project_mapping_unknown" => (
            "Faire vérifier les correspondances de projet examinées pour cette demande.",
            "Le code indique un refus de résolution de correspondance, sans prouver une absence sur tous les serveurs. Lire detail.project_checks lorsqu’il est présent : étape et machines concernées. Un ancien reçu sans ce détail ne permet pas de retrouver la machine ni de prouver qu’une cible était déjà choisie. Vérifier les registres source, maître et worker, le dossier source et project_id seulement s’il est imposé. Demander au responsable Bridget une préparation native autorisée ; ne pas inventer un identifiant, déplacer un dossier actif ou copier des clés. Ce code seul ne prouve aucune panne serveur.",
            Some("project_id"),
        ),
        "origin_request_invalid" => (
            "Transmettre ce refus au responsable Bridget pour vérifier la source et son attestation.",
            "L’enveloppe du relais d’origine n’a pas passé sa validation. Le champ divergent n’est pas attesté par ce reçu. Ce texte ne renseigne pas la version de la façade. Vérifier le cwd dans le périmètre du parent, la forme du SHA et les faits de session sans élargir les droits. Une réattestation seule ne prouve pas la résolution. Conserver la clé et vérifier son état durable ; ne pas créer de doublon.",
            None,
        ),
        "brief_invalid" => (
            "Validez le brief avec le prévol bridget_handoff action=preview.",
            "Lire le champ exact du refus de validation. objective et summary sont requis ; une référence kind=file exige label, host et path absolu. Une modification du brief change l’enveloppe. Si un reçu existe, conserver son canon et suivre son état ; ne pas modifier les paramètres d’un rejeu strict. Aucun ancien refus ne doit être relancé pour tester le format.",
            Some("brief"),
        ),
        "envelope_mismatch" => (
            "Reprenez les paramètres d’origine de cette clé pour retrouver son issue.",
            "Cette clé appartient déjà à une enveloppe différente. Changer le brief, le SHA, le cwd ou le workspace constitue une nouvelle demande. Une nouvelle clé est permise seulement après clarification de l’ancienne issue et absence de mission à doubler. Ne pas modifier le reçu ou recréer une tâche inconnue.",
            None,
        ),
        "task_unavailable" => (
            "Utilisez l’identifiant réel renvoyé par l’admission, depuis son parent propriétaire.",
            "Ce parent ne peut pas lire cette tâche avec cet identifiant. Ce refus ne distingue pas une tâche absente d’une tâche hors de ses droits. Vérifier l’identifiant et la session propriétaire ; ne pas utiliser un identifiant fictif et ne pas demander à un autre agent de contourner la propriété. Ne pas relancer une mission pour tester la lecture.",
            Some("task_id"),
        ),
        "agent_type_unknown" => (
            "Faire vérifier le profil de programme enregistré pour cette cible.",
            "Le type choisi n’existe pas dans le registre lu. Comparer le reçu, la machine et les profils déclarés avec le responsable Bridget ; ne pas inventer un type ou choisir un autre moteur pour contourner le refus.",
            None,
        ),
        "auto_admission_invalid" | "invalid_request" => (
            "Vérifier les champs de cette demande avec le schéma natif.",
            "Conserver l’enveloppe et le reçu. Comparer ses champs et leurs types au schéma de bridget_auto_delegate ; consulter detail.field et detail.cause s’ils sont fournis. Si le champ fautif n’est pas nommé, transmettre le reçu au mainteneur Bridget pour corriger ce diagnostic, sans multiplier les essais.",
            None,
        ),
        "child_requests_reviewer_unsupported" => (
            "Conserver child_requests=deny pour une revue indépendante.",
            "Le profil de revue choisi ne prend pas en charge le transfert des questions de l’enfant. Ne pas changer son rôle ou ses droits. Vérifier l’absence d’admission avant de corriger ce seul paramètre ; garder toute mission déjà connue.",
            Some("child_requests"),
        ),
        "cwd_unavailable" => (
            "Vérifier le dossier cwd sur la machine du parent.",
            "Le chemin source n’a pas pu être ouvert comme dossier. Vérifier son existence, sa lisibilité et sa forme absolue sur la machine appelante ; le dossier de destination ne le remplace pas. Conserver tous les fichiers et faire rétablir l’accès existant si nécessaire.",
            Some("cwd"),
        ),
        "delegations_halted" => (
            "Attendre la levée de l’arrêt des nouvelles délégations par son propriétaire humain.",
            "Lire l’état natif des contrôles. L’arrêt concerne les nouvelles admissions ; consulter et conserver les missions déjà admises. Un agent ne lève pas cet arrêt et ne le contourne pas avec une autre clé ou une autre machine.",
            None,
        ),
        "referent_paused" => (
            "Attendre la reprise décidée par le propriétaire humain.",
            "Lire le contrôle de pause et conserver cette demande. Ne pas lever une pause humaine ni fabriquer une nouvelle mission pour la contourner. Les lectures et les travaux hors de cette pause suivent leur propre état.",
            None,
        ),
        "effort_unavailable" | "model_unavailable" => (
            "Comparer le modèle et le niveau demandés au catalogue de cette connexion.",
            "Lire fleet.options pour la machine et le moteur imposés. Faire qualifier une observation manquante ; un modèle absent n’est pas une preuve de déconnexion du compte. Un choix humain explicite reste inchangé ; demander son choix si aucune option autorisée ne le satisfait.",
            Some("model/effort"),
        ),
        "expected_base_commit_outside_git" | "workspace_git_required" => (
            "Utiliser comme cwd le dépôt source contenant le commit demandé.",
            "La stratégie ou le SHA demandé exige Git, mais ce dossier n’a pas été reconnu comme dépôt source. Vérifier git rev-parse --show-toplevel dans cwd sur la machine du parent. Ne pas inventer un dépôt, changer le SHA ou déplacer un dossier pour passer le contrôle. Un travail réellement hors Git exige de confirmer ce besoin avant de corriger ses paramètres.",
            Some("cwd"),
        ),
        "keep_alive_out_of_range" => (
            "Donner keep_alive_secs entre60et7200secondes, ou omettre ce champ.",
            "Ce paramètre borne la disponibilité conservée de l’enfant ; il ne règle ni l’attente d’un appel ni la durée de sa mission. Corriger le champ seulement après lecture du sort de l’admission ; ne pas remplacer une tâche connue ou d’issue inconnue.",
            Some("keep_alive_secs"),
        ),
        "provider_quota_exhausted" => (
            "Lire la fenêtre de quota et le retour annoncés pour le compte choisi.",
            "Conserver la mission et son reçu. Attendre le retour attesté si une heure est connue ; sinon faire préciser cette observation. Ne pas transposer le quota d’un autre compte, changer la connexion ou lancer une mission de remplacement sans choix autorisé.",
            None,
        ),
        "retention_saturated" | "task_limit" => (
            "Faire vérifier les missions et les conservations qui occupent les places de cette admission.",
            "Lire les tâches et leurs états de clôture avec le responsable Bridget. Libérer seulement les conservations réellement terminées et vérifiées par le parcours natif. Ne pas supprimer de fichiers, de reçus ou de travaux vivants pour récupérer une place ; signaler un plafond sans lien avec la capacité réelle.",
            None,
        ),
        "workspace_path_unsafe" => (
            "Faire vérifier le chemin de travail demandé et son identité physique.",
            "Comparer le chemin absolu, ses liens, son propriétaire et les racines protégées avec le placement approuvé. Conserver le dossier existant ; ne pas le remplacer ou modifier ses permissions pour passer le contrôle. Toute nouvelle destination doit être confirmée si elle dépasse le mandat.",
            Some("workspace"),
        ),
        "workspace_repo_unsupported" => (
            "Faire qualifier les options Git particulières du dépôt source.",
            "Le dépôt utilise une forme que la préparation n’a pas confirmée : filtre, inclusion, clone partiel ou worktreeConfig selon le détail reçu. Transmettre ce détail et le SHA au mainteneur Bridget. Ne pas réécrire la configuration Git, recopier le projet ou changer de stratégie sans vérifier les travaux conservés.",
            Some("workspace"),
        ),
        "workspace_requires_development" => (
            "Faire confirmer la posture nécessaire à ce dossier de travail.",
            "La stratégie demandée ne correspond pas à la posture effective admise. Une revue en lecture conserve discovery ; un développement exige l’autorisation déjà accordée au parent. Ne pas augmenter les droits d’un enfant pour contourner le refus. Vérifier les paramètres puis demander seulement la décision manquante si nécessaire.",
            Some("posture"),
        ),
        _ => return None,
    };
    Some(json!({"status":"refused","code":code,
        "message":format!("{message} Ce refus concerne cette demande ; les autres travaux autorisés suivent leur propre état."),
        "detail":{"message":message,"recovery":recovery,"field":field,
        "human_action":"Si la résolution dépasse vos droits, demander à l’utilisateur une décision ciblée sur le dossier, le placement ou l’accès aux faits manquants décrits ci-dessus. Ne pas demander un arrêt global ou une augmentation de droits pour contourner le refus."}}))
}

fn is_project_context_code(code: &str) -> bool {
    matches!(
        code,
        "project_mapping_unknown"
            | "project_mapping_mismatch"
            | "project_mapping_ambiguous"
            | "project_mapping_preference_invalid"
            | "project_preparation_git_required"
    )
}

/// Diagnostic borné. Les autres familles gardent leur code historique.
pub(super) fn mapping_context_error(stage: &str, refused: &BTreeMap<String, String>) -> String {
    // Une autre famille garde son code et son guidage propres, quel que soit
    // l'ordre des machines. Le contexte de correspondance reste partiel.
    if let Some(code) = refused.values().find(|code| !is_project_context_code(code)) {
        return code.clone();
    }
    let code = refused
        .values()
        .next()
        .map(String::as_str)
        .unwrap_or("project_mapping_unknown");
    let checks = refused
        .iter()
        .map(|(machine, reason)| {
            json!({
                "machine_id": machine,
                "code": if is_project_context_code(reason) {reason.as_str()} else {"unobserved"},
            })
        })
        .collect::<Vec<_>>();
    format!(
        "routing_project_refused:{}",
        json!({"stage": stage,"code": code,"checks": checks})
    )
}

pub(super) fn contextual_refusal(encoded: &str) -> Value {
    let fallback = || {
        let mut view = refusal("project_mapping_unknown").expect("known refusal");
        view["detail"]["context_status"] = json!("unavailable");
        view
    };
    if encoded.len() > 8192 {
        return fallback();
    }
    let Ok(value) = serde_json::from_str::<Value>(encoded) else {
        return fallback();
    };
    let Some(object) = value.as_object().filter(|v| v.len() == 3) else {
        return fallback();
    };
    let Some(stage) = object
        .get("stage")
        .and_then(Value::as_str)
        .filter(|v| matches!(*v, "project_preparation" | "candidate_mapping"))
    else {
        return fallback();
    };
    let Some(code) = object
        .get("code")
        .and_then(Value::as_str)
        .filter(|v| is_project_context_code(v))
    else {
        return fallback();
    };
    let Some(checks) = object
        .get("checks")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty() && v.len() <= 16)
    else {
        return fallback();
    };
    let mut previous: Option<&str> = None;
    for check in checks {
        if check.as_object().is_none_or(|v| v.len() != 2) {
            return fallback();
        }
        let Some(machine) = check["machine_id"].as_str().filter(|v| {
            v.len() <= 256
                && bridget_core::host_is_attested(v)
                && v.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
        }) else {
            return fallback();
        };
        if previous.is_some_and(|v| v >= machine) {
            return fallback();
        }
        previous = Some(machine);
        if !check["code"]
            .as_str()
            .is_some_and(|v| is_project_context_code(v) || v == "unobserved")
        {
            return fallback();
        }
    }
    if checks[0]["code"] != code {
        return fallback();
    }
    let mut result = refusal(code).expect("known refusal");
    if code != "project_preparation_git_required" {
        result["message"] = json!(
            "Faire vérifier les correspondances indiquées dans detail.project_checks, sur les machines et à l’étape nommées. Ce refus concerne cette demande, sans prouver un choix final de machine ni une panne globale. Les autres travaux autorisés suivent leur propre état."
        );
    }
    result["detail"]["stage"] = json!(stage);
    result["detail"]["scope"] = json!("this_admission");
    result["detail"]["context_status"] = json!("observed");
    result["detail"]["execution_selection"] = json!("not_attested_by_this_diagnostic");
    result["detail"]["coverage"] = json!(if checks
        .iter()
        .any(|c| c["code"] == "project_preparation_git_required")
    {
        "known_project_preparation_refusals"
    } else {
        "mapping_refusals_only"
    });
    result["detail"]["project_checks"] = json!(checks);
    result["detail"]["verification"] = json!(
        "Les codes fermés de correspondance et de préparation hors Git sont conservés par machine. unobserved signifie que la cause précise de cet autre refus n’est pas publiée ici. Ce diagnostic ne prouve pas l’absence de correspondance sur tous les serveurs."
    );
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn spec271_preparation_guidance_distinguishes_new_facts_from_legacy_unknown() {
        let view = refusal("project_preparation_process_failed").unwrap();
        assert_eq!(view["code"], "project_preparation_process_failed");
        assert!(
            view["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("bridget_project_preparation")
        );
        assert!(view["detail"].get("transport").is_none());
        let legacy = refusal("project_preparation_transport_failed").unwrap();
        assert!(
            legacy["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("inconnue")
        );
        assert!(refusal("project_preparation_output_limit").is_some());
    }

    use super::*;
    #[test]
    fn spec252_non_git_refusal_keeps_machine_and_useful_action() {
        let causes = BTreeMap::from([
            ("cartae".into(), "project_preparation_git_required".into()),
            ("sysle2".into(), "project_mapping_unknown".into()),
        ]);
        let encoded = mapping_context_error("project_preparation", &causes);
        let view = contextual_refusal(encoded.strip_prefix("routing_project_refused:").unwrap());
        assert_eq!(view["code"], "project_preparation_git_required");
        assert_eq!(view["detail"]["project_checks"][0]["machine_id"], "cartae");
        assert_eq!(view["detail"]["stage"], "project_preparation");
        assert!(
            view["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("ne copie pas automatiquement")
        );
        assert!(!view.to_string().contains("sans fournir de diagnostic"));
        // Un refus de lien/droits reste distinct, même dans un lot hors Git.
        assert_eq!(
            mapping_context_error(
                "project_preparation",
                &BTreeMap::from([
                    ("cartae".into(), "origin_proof_busy".into()),
                    ("sysle2".into(), "project_preparation_git_required".into()),
                ])
            ),
            "origin_proof_busy"
        );
        assert!(view.to_string().len() < 8192);
    }
    #[test]
    fn spec221_project_context_keeps_machine_stage_code_and_closed_bounds() {
        for code in [
            "project_mapping_unknown",
            "project_mapping_mismatch",
            "project_mapping_ambiguous",
            "project_mapping_preference_invalid",
        ] {
            let error = mapping_context_error(
                "candidate_mapping",
                &BTreeMap::from([("cartae".into(), code.into())]),
            );
            let view = contextual_refusal(error.strip_prefix("routing_project_refused:").unwrap());
            assert_eq!(view["code"], code);
            assert_eq!(view["detail"]["context_status"], "observed");
        }
        for (first, second) in [
            ("project_mapping_unknown", "origin_proof_busy"),
            ("project_preparation_timeout", "project_mapping_mismatch"),
            ("project_mapping_unknown", "project_preparation_busy"),
        ] {
            let refused = BTreeMap::from([
                ("cartae".into(), first.into()),
                ("sysle2".into(), second.into()),
            ]);
            let expected = if is_project_context_code(first) {
                second
            } else {
                first
            };
            assert_eq!(
                mapping_context_error("project_preparation", &refused),
                expected
            );
        }
        let causes = BTreeMap::from([
            ("cartae".into(), "project_mapping_unknown".into()),
            ("sysle2".into(), "project_mapping_mismatch".into()),
        ]);
        let error = mapping_context_error("project_preparation", &causes);
        let view = contextual_refusal(error.strip_prefix("routing_project_refused:").unwrap());
        assert_eq!(view["code"], "project_mapping_unknown");
        assert_eq!(view["detail"]["stage"], "project_preparation");
        assert_eq!(view["detail"]["project_checks"][0]["machine_id"], "cartae");
        assert_eq!(
            view["detail"]["project_checks"][1]["code"],
            "project_mapping_mismatch"
        );
        assert!(!view.to_string().contains("PRIVATE"));
        assert_eq!(
            view["detail"]["execution_selection"],
            "not_attested_by_this_diagnostic"
        );
        assert!(view.to_string().len() < 8192);
        assert_eq!(
            mapping_context_error(
                "project_preparation",
                &BTreeMap::from([("cartae".into(), "permission_attestation_unavailable".into())])
            ),
            "permission_attestation_unavailable"
        );
        for raw in [
            "PRIVATE",
            r#"{"stage":"project_preparation","code":"PRIVATE","checks":[]}"#,
            r#"{"stage":"chosen","code":"project_mapping_unknown","checks":[{"machine_id":"cartae","code":"project_mapping_unknown"}]}"#,
        ] {
            let rejected = contextual_refusal(raw);
            assert_eq!(rejected["detail"]["context_status"], "unavailable");
            assert!(!rejected.to_string().contains("PRIVATE"));
        }
        let mut invalid =
            serde_json::from_str::<Value>(error.strip_prefix("routing_project_refused:").unwrap())
                .unwrap();
        invalid["checks"][0]["machine_id"] = json!("/private/path");
        assert_eq!(
            contextual_refusal(&invalid.to_string())["detail"]["context_status"],
            "unavailable"
        );
        invalid["checks"]=json!((0..17).map(|i|json!({"machine_id":format!("host-{i:02}"),"code":"project_mapping_unknown"})).collect::<Vec<_>>());
        assert_eq!(
            contextual_refusal(&invalid.to_string())["detail"]["context_status"],
            "unavailable"
        );
    }

    #[test]
    fn spec236_origin_validation_field_is_closed_and_not_a_version_claim() {
        let plain = refusal("origin_request_invalid").unwrap();
        assert!(plain["detail"]["field"].is_null());
        assert!(!plain.to_string().contains("ancien code"));
        for field in [
            "worker",
            "version",
            "correlation_id",
            "connection_epoch",
            "expires_at_ms",
            "envelope",
            "parent",
            "blocked",
            "root",
            "worktree",
            "expected_thread",
            "proof",
            "target_machine",
            "project_id",
            "cwd",
            "expected_base_commit",
            "workspace",
        ] {
            let view = refusal(&format!("origin_request_invalid:{field}")).unwrap();
            assert_eq!(view["code"], "origin_request_invalid");
            assert_eq!(view["detail"]["field"], field);
            assert!(view["detail"]["recovery"].as_str().unwrap().contains(field));
        }
        for (field, limit) in [("envelope", 128 * 1024), ("workspace", 4096)] {
            let view = refusal(&format!("origin_request_invalid:{field}:resource_limit")).unwrap();
            assert_eq!(view["detail"]["cause"], "resource_limit");
            assert_eq!(view["detail"]["limit_bytes"], limit);
        }
        let same = refusal("origin_request_invalid:target_machine:same_machine_target").unwrap();
        assert_eq!(same["detail"]["field"], "target_machine");
        assert_eq!(same["detail"]["cause"], "same_machine_target");
        assert!(
            same["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("cible désigne la machine source")
        );
        let view = refusal("origin_request_invalid:parent:permissions_changed").unwrap();
        assert_eq!(view["detail"]["cause"], "permissions_changed");
        for field in ["secret/path", "cwd:PRIVATE", "", "parent\nPRIVATE"] {
            assert!(refusal(&format!("origin_request_invalid:{field}")).is_none());
        }
    }

    #[test]
    fn spec211_refusals_name_action_scope_and_human_decision() {
        for code in [
            "expected_base_commit_required",
            "cwd_outside_parent_project",
            "project_mapping_unknown",
            "project_mapping_mismatch",
            "project_mapping_preference_invalid",
            "project_mapping_ambiguous",
            "origin_request_invalid",
            "brief_invalid",
            "envelope_mismatch",
            "task_unavailable",
        ] {
            let r = refusal(code).unwrap();
            assert_eq!(r["code"], code);
            assert!(r["message"].as_str().unwrap().contains("cette demande"));
            assert!(r["message"].as_str().unwrap().contains("autres travaux"));
            assert!(r["detail"]["recovery"].as_str().unwrap().len() > 120);
            assert!(
                r["detail"]["human_action"]
                    .as_str()
                    .unwrap()
                    .contains("utilisateur")
            );
        }
        assert!(
            refusal("task_unavailable").unwrap()["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("ne distingue pas")
        );
        assert!(
            refusal("origin_request_invalid").unwrap()["detail"]["recovery"]
                .as_str()
                .unwrap()
                .contains("n’est pas attesté")
        );
        assert!(refusal("unknown:private").is_none());
    }
}

/// Guidage fermé et exhaustif du protocole worker ; un nouveau refus doit
/// recevoir son action ici avant de compiler. Aucun texte distant n'est repris.
pub(super) fn worker_action(reason: bridget_transport::protocol::WorkerRefusal) -> &'static str {
    use bridget_transport::protocol::WorkerRefusal::*;
    match reason {
        UnsupportedVersion | CapabilityRequired => {
            "Faites charger une version Bridget compatible sur les deux composants de cette machine ; conservez la tâche et son reçu."
        }
        InvalidEnvelope => {
            "Faites vérifier le champ invalide du reçu par le mainteneur Bridget, sans changer la clé ni les droits."
        }
        Unauthorized | PermissionsChanged => {
            "Vérifiez la session propriétaire et les droits déjà accordés ; une extension de droits exige une décision humaine ciblée."
        }
        StaleConnection | WorkerUnavailable | ProbeExpired => {
            "Vérifiez le retour du lien natif de cette machine, puis relisez la même tâche ; ne créez pas de remplacement d'issue inconnue."
        }
        PlacementMismatch | CommandMismatch | DefinitionChanged => {
            "Faites comparer le placement, la commande et la définition enregistrés avec le reçu de cette tâche ; conservez ses fichiers."
        }
        PrepareExpired | ReleaseExpired | ResultSlotExpired => {
            "Relisez l'échéance et l'arrêt natif de cette tâche avant toute nouvelle admission autorisée ; le rejeu ne redémarre pas une tâche close."
        }
        ResourceLimit => {
            "Faites lire l'occupation du budget nommé sur cette machine ; conservez les travaux actifs et demandez une décision humaine seulement si la capacité autorisée doit changer."
        }
        StoreUnavailable => {
            "Faites vérifier le journal et l'accès au magasin natif de cette machine ; ne supprimez ni reçu ni tâche pour le rouvrir."
        }
        DiskFull => {
            "Faites vérifier l'espace libre sur le volume de cette tâche et retirez seulement les copies dont les usages et la clôture sont établis."
        }
        FilesystemAccessDenied => {
            "Faites vérifier le propriétaire et le mode du dossier visé ; un accès nouveau exige une décision humaine, pas un contournement."
        }
        WorkspacePrepareFailed | WorkspacePrepareIncomplete => {
            "Faites lire la cause et l'étape de préparation dans ce reçu, puis vérifiez le dossier conservé avant toute reprise."
        }
        JournalUnavailable => {
            "Conservez la tâche et faites vérifier son journal sur le worker ; un journal indisponible ne prouve pas l'arrêt du travail."
        }
        ResultConflict => {
            "Conservez les deux références de résultat et faites comparer leurs empreintes ; aucun résultat ne doit être écrasé."
        }
        UnknownExecution => {
            "Relisez l'identifiant d'exécution de la tâche auprès de son propriétaire ; ne démarrez pas une nouvelle exécution pour retrouver l'ancienne."
        }
    }
}

pub(super) fn worker_start_action(error: &str) -> Option<&'static str> {
    let error = error.strip_prefix("remote_start_failed:").unwrap_or(error);
    let reason = error
        .strip_prefix("remote_prepare:")
        .or_else(|| error.strip_prefix("remote_workspace_prepare:"))?;
    let mut snake = String::new();
    for (i, c) in reason.chars().enumerate() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        if i > 0 && c.is_ascii_uppercase() {
            snake.push('_');
        }
        snake.push(c.to_ascii_lowercase());
    }
    serde_json::from_value::<bridget_transport::protocol::WorkerRefusal>(json!(snake))
        .ok()
        .map(worker_action)
}

#[cfg(test)]
mod exhaustive244_tests {
    use super::*;
    use std::collections::BTreeSet;
    #[test]
    fn spec244_refus_terminaux_admission_sans_repli_opaque() {
        let source = include_str!("native_delegation.rs");
        let function = source
            .split("fn auto_terminal_refusal_code(")
            .nth(1)
            .unwrap()
            .split("fn close_auto_admission_error(")
            .next()
            .unwrap();
        let codes = function
            .split('"')
            .skip(1)
            .step_by(2)
            .filter(|s| s.contains('_'))
            .map(|s| s.split(':').next().unwrap())
            .collect::<BTreeSet<_>>();
        let missing = codes
            .iter()
            .filter(|code| {
                let r = failure(code);
                r["message"]
                    .as_str()
                    .is_none_or(|m| m.contains("sans fournir de diagnostic"))
                    || r["detail"]["recovery"]
                        .as_str()
                        .is_none_or(|m| m.len() < 40)
            })
            .copied()
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "guidages d’admission absents : {missing:?}"
        );
    }
    #[test]
    fn spec244_chaque_refus_worker_a_une_action() {
        let source = include_str!("../../../bridget-transport/src/protocol.rs");
        let body = source
            .split("pub enum WorkerRefusal {")
            .nth(1)
            .unwrap()
            .split('}')
            .next()
            .unwrap();
        for line in body.lines().map(str::trim).filter(|s| !s.is_empty()) {
            let variant = line
                .strip_suffix(',')
                .expect("énumération worker unitaire fermée");
            let error = format!("remote_start_failed:remote_prepare:{variant}");
            let action = worker_start_action(&error)
                .unwrap_or_else(|| panic!("guidage manquant : {variant}"));
            assert!(action.len() > 40, "action absente : {variant}");
        }
        assert!(worker_start_action("remote_prepare:texte libre secret").is_none());
    }
}
