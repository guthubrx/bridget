//! Même façade native pour les agents T3, CLI et MCP externes.
use crate::communication::client::{
    ClientError, daemon_supports_wait, registered_connection, registered_connection_until,
    unexpected_response,
};
use crate::mcp_identity::ResolvedIdentity;
use bridget_transport::protocol::{
    AutoDelegateV2, AutoDelegateWorkload, ChildRequestDecision, FallbackTarget,
    NativeDelegationRequest, SpawnPosture, WorkspaceRequest,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

/// Même schéma fermé pour les deux admissions ; aucune valeur JSON dans le refus.
fn parse_workspace(value: &Value) -> Result<WorkspaceRequest, ClientError> {
    if value.get("strategy").and_then(Value::as_str) == Some("commit_view")
        && !crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
    {
        return Err(ClientError::InvalidParams("La lecture de commit est en préparation sur ce client. Cet appel n'a pas été envoyé. Conserver toute clé ou tâche connue ; ne pas changer de mode pour forcer le passage. Faire qualifier et charger le parcours de vue et son retour compatible par le responsable Bridget. Les autres travaux autorisés suivent leur propre état.".into()));
    }
    serde_json::from_value(value.clone()).map_err(|_| {
        ClientError::InvalidParams(
            "Corrigez la forme fermée de workspace : shared ou worktree, et commit_view lorsque le parcours de vue est chargé. En commit_view, seul strategy est accepté ; expected_base_commit et posture=discovery appartiennent à bridget_auto_delegate. En shared, seul strategy est accepté ; retirez base_ref, branch et tout autre champ. En worktree, seuls strategy, base_ref et branch sont acceptés ; base_ref et branch sont des chaînes si présents, jamais null. Le SHA source complet se place dans expected_base_commit pour l’admission automatique. Ce refus local concerne cet appel : il n’a pas été envoyé au serveur. Il ne prouve pas l’absence d’une demande antérieure de même request_id. Attendez la fin des appels concurrents et vérifiez tout reçu ou tâche déjà connu avant de corriger les paramètres ; un reçu existant conserve son canon. Ne créez aucun remplacement d’issue inconnue. Les autres travaux autorisés suivent leur propre état. Si le client chargé ne décrit pas ces règles, demandez sa reconnexion complète ; une simple reprise de tour peut garder l’ancien programme.".into(),
        )
    })
}

pub(crate) const WAIT_TIMEOUT_DEFAULT_SECS: u64 = 600;
pub(crate) const WAIT_TIMEOUT_MAX_SECS: u64 = 3600;

/// Validation commune des deux outils d'attente : forme JSON puis bornes métier.
pub(crate) fn pre_dispatch_failure(error: ClientError, stage: &str, args: &Value) -> ClientError {
    match error {
        ClientError::Technical { code, message }
            if matches!(code, "daemon_unreachable" | "outcome_unknown") =>
        {
            let reference = args
                .get("task_id")
                .and_then(Value::as_str)
                .map(|id| format!("task_id={id}"))
                .or_else(|| {
                    args.get("request_id")
                        .and_then(Value::as_str)
                        .map(|id| format!("request_id={id}"))
                })
                .or_else(|| {
                    parse_task_ids(args)
                        .ok()
                        .filter(|ids| {
                            ids.iter().all(|id| {
                                id.len() <= 128
                                    && id.bytes().all(|byte| {
                                        byte.is_ascii_alphanumeric() || b"-_:.".contains(&byte)
                                    })
                            })
                        })
                        .map(|ids| format!("task_ids={}", json!(ids)))
                })
                .unwrap_or_else(|| "les identifiants des tâches de cet appel".into());
            ClientError::Technical {
                code,
                message: format!(
                    "{message} Étape : {stage}. La commande de cet appel n'a pas été transmise. Conservez {reference} et tout reçu antérieur. Vérifiez le lien natif depuis cette machine avant de reprendre exactement cet appel ; si la lecture reste indisponible, le responsable Bridget doit mesurer cette liaison. La cause précise du retard reste inconnue. Ce refus concerne cet appel ; les autres travaux autorisés suivent leur propre état."
                ),
            }
        }
        other => other,
    }
}

pub(crate) fn parse_wait_timeout(
    args: &serde_json::Map<String, Value>,
) -> Result<u64, ClientError> {
    let seconds = match args.get("wait_timeout") {
        None => WAIT_TIMEOUT_DEFAULT_SECS,
        Some(Value::Number(value)) if value.is_u64() => value.as_u64().expect("entier positif"),
        Some(Value::Number(value)) if value.is_i64() => 0,
        _ => {
            return Err(ClientError::InvalidParams(
                "wait_timeout doit être un entier".into(),
            ));
        }
    };
    if !(1..=WAIT_TIMEOUT_MAX_SECS).contains(&seconds) {
        return Err(ClientError::Technical {
            code: "native_delegation_refused",
            message: "wait_timeout_out_of_range".into(),
        });
    }
    Ok(seconds)
}

/// O(k), k <= 16 : la liste ne porte aucune autorité de parent.
fn parse_task_ids(args: &Value) -> Result<Vec<String>, ClientError> {
    let ids = args
        .get("task_ids")
        .and_then(Value::as_array)
        .filter(|ids| (1..=16).contains(&ids.len()))
        .ok_or_else(|| {
            ClientError::InvalidParams("task_ids doit contenir 1 à 16 chaînes distinctes".into())
        })?;
    let mut seen = HashSet::with_capacity(ids.len());
    ids.iter()
        .map(|id| {
            let id = id.as_str().filter(|id| !id.is_empty()).ok_or_else(|| {
                ClientError::InvalidParams("task_ids doit contenir des chaînes non vides".into())
            })?;
            if !seen.insert(id) {
                return Err(ClientError::InvalidParams(
                    "task_ids contient un doublon".into(),
                ));
            }
            Ok(id.to_string())
        })
        .collect()
}

pub(crate) fn tools(brief_schema: &Value) -> Vec<Value> {
    let workspace_schema = json!({"oneOf":[
        {"type":"object","properties":{"strategy":{"const":"shared","description":"Dossier partagé : strategy est le seul champ autorisé. Le SHA source appartient à expected_base_commit ; ne pas ajouter base_ref ou branch."}},"required":["strategy"],"additionalProperties":false},
        {"type":"object","properties":{"strategy":{"const":"worktree","description":"Dossier de travail isolé : seuls strategy et les chaînes facultatives base_ref et branch sont acceptés."},"base_ref":{"type":"string","description":"Référence Git disponible dans cwd sur la machine du parent. Pour une base figée, fournir le même SHA complet que expected_base_commit ; un nom de branche peut avancer. Un commit présent uniquement sur la destination doit d’abord être récupéré dans le dépôt source."},"branch":{"type":"string"}},"required":["strategy"],"additionalProperties":false}
    ]});
    let mut definitions = vec![
        json!({"name":"bridget_specialists","description":"Catalogue des spécialistes. Fournir version:1 comme nombre JSON (sans guillemets) à chaque appel. Lecture : action search (role_id facultatif), list ou get (specialist_agent_id requis). Exemple : {version:1,action:search,role_id:mainteneur-bridget}. L’action catalogue n’existe pas. Le schéma expose les autres actions de profils, rôles et mémoire sourcée. Déclarer ses rôles ne donne aucun droit. request_role choisit un responsable unique et conserve sa clé ; request_status relit la demande sans réveil. Une activation durable ou un mandat permanent exige l'administration humaine. Budget Bridget sans plafond par défaut ; restrictions techniques et fournisseur restent distinctes.","inputSchema":specialists_schema222(),"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_read_commit_file","description":"Rend disponible un fichier omis dans la vue privée du commit exact de sa tâche active. path est le chemin relatif indiqué dans la carte de la vue, sans ../ ni chemin absolu. Sans task_id, l'enfant lit sa propre tâche ; avec task_id, le parent doit en être propriétaire. Copie privée contrôlée, aucun changement du dépôt source, de la base, des comptes ou des droits. Un fichier existant divergent n'est jamais écrasé. Après succès, lire le chemin de données renvoyé. Une ancienne tâche clôturée ne redémarre pas par cette lecture.",
            "inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1,"maxLength":128},
                "path":{"type":"string","minLength":1,"maxLength":4096}},"required":["path"],"additionalProperties":false},
            "annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_refresh_models","description":"Demande une nouvelle observation des modèles annoncés par Codex sur les serveurs. Aucun login, modèle, profil, droit ou mission modifié. La réponse requested n’atteste pas une lecture terminée : relire ensuite bridget_capabilities et fleet.connection_catalogues. Une demande par serveur au plus toutes les 30 secondes. Un ancien worker reste non vérifié.","inputSchema":{"type":"object","properties":{"machine_id":{"type":"string","minLength":1,"maxLength":256}},"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":false}}),
        json!({"name":"bridget_capabilities","description":"Catalogue natif : providers contient les verdicts locaux sur la machine d’origine ; ces refus ne décrivent pas les serveurs. fleet.options donne les choix distants déclarés pour auto_delegate (machine_id, engine, model). Ces choix ne prouvent ni éligibilité ni authentification : l’admission vérifie la cible, les droits et le dossier. Fonctionne sans T3.","inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_delegate","description":"Crée un enfant Bridget et remet sa mission en un appel natif. Conserver request_id sur retry identique ; tâche, résultat et annulation appartiennent au parent. keep_alive_secs garde l'enfant après la remise pour une nouvelle ronde. Le brief est déclaré par le parent ; sources_not_verified : aucune source citée n'est vérifiée. Le journal peut être lu plus largement que l'enfant ; aucun secret dans le brief. Corps du dossier ≤ 16 Kio, refusé plutôt que tronqué.","inputSchema":{"type":"object","properties":{"quota_fallback_authorized":{"type":"boolean","description":"true seulement après accord humain explicite pour une bascule de moteur ou de compte en cas de quota dans cette mission. Absent/faux : question humaine. Aucun droit ni budget supplémentaire."},"request_id":{"type":"string","minLength":1,"maxLength":128},"agent_type":{"type":"string"},"model":{"type":"string"},"effort":{"type":["string","null"]},"task":{"type":"string","minLength":1,"maxLength":65536},"machine_id":{"type":"string","minLength":1,"maxLength":256,"description":"Machine de destination de la mission. Vérifier les prérequis propres au projet : données externes, accès SSH et outils ne sont pas garantis par le placement du dépôt. Le champ cwd reste le dossier source sur la machine du parent."},"project_id":{"type":"string","minLength":1,"maxLength":256},"cwd":{"type":"string","minLength":1,"description":"Chemin absolu du dossier source sur la machine du parent appelant. Même avec machine_id distant, ne pas fournir le chemin de destination ; Bridget traduit le chemin via le mapping du projet."},"posture":{"type":"string","enum":["discovery","development"],"description":"Omettre pour hériter des droits attestés du parent. discovery réduit à la lecture seule ; avec Claude/GLM, seuls Read, Glob et Grep sont permis, sans WebSearch, WebFetch ni Bash. Ne convient pas à une recherche web. Changer de machine ne lève pas cette limite. development reste borné par les droits du parent."},"reply_timeout":{"type":"integer","minimum":60,"maximum":86400,"description":"Délai de réponse de la mission en secondes (défaut 3600)."},"workspace":workspace_schema,"child_requests":{"type":"string","enum":["deny","forward"],"description":"Remonter les demandes de l’enfant Codex ; absent : deny, politique inchangée."},"keep_alive_secs":{"type":"integer","minimum":60,"maximum":7200,"description":"Durée de garde après la remise du résultat ; absent : aucune garde. La garde conserve une place de flotte ; un ACK ne la libère pas. reply_timeout ne prolonge pas cette garde. Chaque tâche est bornée à huit rondes, première mission comprise ; consulter round_limit et rounds_remaining."},"resume_on_limit":{"type":"boolean","description":"Attendre puis reprendre après une limite d’usage attestée ; absent ou faux : inactif. Au plus 6 h et 2 rondes, même fournisseur."},"brief":brief_schema,"fallback":{"type":"array","minItems":1,"maxItems":3,"items":{"type":"object","properties":{"agent_type":{"type":"string","minLength":1},"model":{"type":"string","minLength":1},"effort":{"type":"string","minLength":1}},"required":["agent_type","model"],"additionalProperties":false}}},"required":["request_id","agent_type","model","task","cwd"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_auto_delegate","description":"Demande une admission de flotte V2. Les axes machine, moteur, connexion, modèle et effort restent indépendants. Omettre agent_type : utiliser engine pour le moteur et, seulement si nécessaire, connection_id issu du catalogue pour une connexion précise. cwd désigne le dossier source sur la machine du parent ; machine_id désigne la destination. expected_base_commit est obligatoire dans un dépôt Git, en worktree ou pour compilation ; hors Git seulement, general/shared peut l’omettre : dossier local sur la machine source, ou dossier distant déjà présent et mappé ; aucun transfert automatique hors Git. La stratégie commit_view exige explicitement posture=discovery et expected_base_commit ; pour écrire, utiliser shared ou worktree avec posture=development. Il exige un SHA complet (40 caractères hexadécimaux (SHA-1)). Conserver request_id au rejeu strict. Un maître sans auto_delegate_v2 refuse explicitement. Après une réponse perdue, conserver le propriétaire, la request_id et l’enveloppe complète. Un rejeu strict retrouve une tâche admise mais peut admettre la mission si elle n’avait jamais été reçue : seulement si le mandat reste valide. guichet_request_status ne consulte pas ces admissions natives. Le placement du dépôt ne transmet ni données externes ni clés ou alias SSH : vérifier ces prérequis sur la destination. Une admission devenue périmée s’annule avec bridget_cancel, kind=admission, id=request_id.","inputSchema":{"type":"object","properties":{"quota_fallback_authorized":{"type":"boolean","description":"true seulement si l’utilisateur a déjà autorisé explicitement le changement de moteur ou de compte en cas de quota pour cette mission. Omettre sinon : une question humaine sera ouverte après un refus fournisseur constaté. Aucun accord n’est déduit du choix automatique initial ; aucun droit ni budget supplémentaire."},"request_id":{"type":"string","minLength":1,"maxLength":128},"machine_id":{"type":"string","minLength":1,"maxLength":256,"description":"Machine de destination de la mission. Vérifier les prérequis propres au projet : données externes, accès SSH et outils ne sont pas garantis par le placement du dépôt. Le champ cwd reste le dossier source sur la machine du parent."},"project_id":{"type":"string","minLength":1,"maxLength":256},"engine":{"type":"string","minLength":1,"maxLength":256},"connection_id":{"type":"string","minLength":1,"maxLength":256},"model":{"type":"string","minLength":1,"maxLength":256},"effort":{"type":"string","minLength":1,"maxLength":256},"workload":{"type":"string","enum":["general","compilation"],"default":"general"},"expected_base_commit":{"type":"string","minLength":40,"maxLength":40,"pattern":"^[0-9a-fA-F]{40}$","description":"Obligatoire dans un dépôt Git et pour compilation/worktree. Omission permise seulement hors Git en general/shared. SHA source complet : exactement 40 caractères hexadécimaux (SHA-1), jamais un nom de branche. En shared : HEAD source ; en worktree : commit résolu de workspace.base_ref (HEAD si absent)."},"task":{"type":"string","minLength":1,"maxLength":65536},"cwd":{"type":"string","minLength":1,"description":"Chemin absolu du dossier source sur la machine du parent appelant. Même avec machine_id distant, ne pas fournir le chemin de destination ; Bridget traduit le chemin via le mapping du projet."},"posture":{"type":"string","enum":["discovery","development"],"description":"Omettre pour hériter des droits attestés du parent. discovery réduit à la lecture seule ; avec Claude/GLM, seuls Read, Glob et Grep sont permis, sans WebSearch, WebFetch ni Bash. Ne convient pas à une recherche web. Changer de machine ne lève pas cette limite. development reste borné par les droits du parent."},"reply_timeout":{"type":"integer","minimum":60,"maximum":86400,"default":3600,"description":"Durée maximale de chaque ronde en secondes, à partir de sa remise effective. Une nouvelle ronde admise renouvelle ce budget ; un rejeu strict ne le renouvelle pas. Défaut : 3600 (1 h), y compris development. Pour un travail long, fournir par exemple 14400 (4 h), au plus 86400 et dans la limite du parent. Ce délai est distinct de wait_timeout et keep_alive_secs. Fixer la valeur avant la première admission ; ne pas modifier une enveloppe déjà admise au rejeu."},"workspace":workspace_schema,"child_requests":{"type":"string","enum":["deny","forward"]},"keep_alive_secs":{"type":"integer","minimum":60,"maximum":7200,"description":"Durée de garde après la remise du résultat ; absent : aucune garde. La garde conserve une place de flotte ; un ACK ne la libère pas. reply_timeout ne prolonge pas cette garde. Chaque tâche est bornée à huit rondes, première mission comprise ; consulter round_limit et rounds_remaining."},"brief":brief_schema,"urgent":{"type":"boolean"},"deliverable":{"type":"string","minLength":1,"maxLength":128}},"required":["request_id","task","cwd"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_repair_workspace","description":"Répare explicitement le dossier déplacé de sa même tâche distante, après résultat remis. Conserver request_id sur rejeu identique. Aucun tour métier créé. Les pairs ou adaptations non qualifiés refusent sans forçage.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1,"maxLength":128},"target_cwd":{"type":"string","minLength":1,"maxLength":4096},"expected_head":{"type":"string","pattern":"^[0-9a-fA-F]{40}$","minLength":40,"maxLength":40}},"required":["task_id","request_id","target_cwd","expected_head"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_continue","description":"Ouvre une nouvelle ronde sur son enfant encore gardé, après la remise du résultat. Prévoir keep_alive_secs dès l’admission si une suite est nécessaire ; une tâche déjà terminée sans garde ne peut plus continuer. Un SHA cité dans task ou brief n’est ni importé ni synchronisé : rendre les objets et données autorisés disponibles avant la ronde, sans déplacer silencieusement son dossier. Employer une nouvelle request_id à chaque ronde et la conserver sur retry identique. Le brief est déclaré par le parent ; sources_not_verified : aucune source citée n'est vérifiée. Le journal peut être lu plus largement que l'enfant ; aucun secret dans le brief. Corps du dossier ≤ 16 Kio, refusé plutôt que tronqué. Dans brief.references, chaque objet exige kind et label ; kind=file exige aussi host et path absolu.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1,"maxLength":128},"task":{"type":"string","minLength":1,"maxLength":65536},"brief":brief_schema},"required":["task_id","request_id","task"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_respond","description":"Répond à une question de son enfant avec l'identifiant lu dans le statut ou l'avis. Les approbations appartiennent à l'humain. Aucun secret dans la réponse.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1},"decision":{"type":"string","enum":["accept","decline","cancel"]},"answer":{"type":"object","description":"Pour accept uniquement : chaque identifiant de question porte un tableau de chaînes, 8 Kio au plus."}},"required":["task_id","request_id","decision"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_status","description":"Lit l'état durable et le résultat stable de sa mission native. Une fin de tour ne vaut pas résultat disponible tant que les enfants travaillent.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string"}},"required":["task_id"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_wait","description":"Attend le résultat d'une ou plusieurs de ses tâches natives, sans polling ni modification de la remise. Une expiration n'annule aucun enfant. Le message de résultat peut arriver après ce retour.","inputSchema":{"type":"object","properties":{"task_ids":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"string","minLength":1}},"wait_timeout":{"type":"integer","minimum":1,"maximum":WAIT_TIMEOUT_MAX_SECS,"default":WAIT_TIMEOUT_DEFAULT_SECS}},"required":["task_ids"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_ack","description":"Déclare avoir lu le résultat ou l'échec de ses tâches natives. Cet accusé est informatif ; il ne remplace pas le message normal et ne modifie ni remise, ni nettoyage, ni rétention. Tout le groupe est refusé sans ACK partiel si une tâche n’est pas prête ; le refus identifie ses task_id, ronde et état après contrôle de propriété du groupe.","inputSchema":{"type":"object","properties":{"task_ids":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"string","minLength":1}}},"required":["task_ids"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_cancel","description":"Annule sa mission native et sa descendance ; aucune mission d'un autre parent n'est accessible.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string"}},"required":["task_id"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true}}),
    ];
    for tool in &mut definitions {
        if tool["name"] == "bridget_auto_delegate" {
            if crate::provider_catalogue::MODEL_OPTIONS_LAUNCH_ENABLED {
                tool["inputSchema"]["properties"]["service_tier"] = json!({
                    "type":"string","minLength":1,"maxLength":256,
                    "description":"Niveau de service explicite du modèle, annoncé sur cette connexion et autorisé par son profil. Aucun défaut ni remplacement automatique ; peut consommer davantage de quota ou de crédits."});
            }
            let mut sha_workspaces = vec!["worktree"];
            if crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED {
                tool["inputSchema"]["properties"]["workspace"]["oneOf"].as_array_mut().unwrap().push(json!({
                    "type":"object","properties":{"strategy":{"const":"commit_view",
                        "description":"Vue partielle du commit exact, en lecture seule. Seul strategy est autorisé. Fournir posture=discovery et expected_base_commit. Aucun dépôt Git dans la vue ; les fichiers omis se lisent par bridget_read_commit_file."}},
                    "required":["strategy"],"additionalProperties":false}));
                sha_workspaces.push("commit_view");
                tool["inputSchema"]["properties"]["expected_base_commit"]["description"] = json!(
                    "SHA source complet de 40 caractères hexadécimaux. Obligatoire en Git, worktree, commit_view ou compilation. En commit_view, lire ce commit exact même si HEAD a avancé ; ne pas fournir base_ref ou branch."
                );
            }
            tool["inputSchema"]["allOf"] = json!([{
                "if":{"anyOf":[
                    {"required":["workload"],"properties":{"workload":{"const":"compilation"}}},
                    {"required":["workspace"],"properties":{"workspace":{"required":["strategy"],"properties":{"strategy":{"enum":sha_workspaces}}}}}
                ]},
                "then":{"required":["expected_base_commit"]}
            }]);
            if crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED {
                tool["inputSchema"]["allOf"].as_array_mut().unwrap().push(json!({
                    "if":{"required":["workspace"],"properties":{"workspace":{
                        "required":["strategy"],"properties":{"strategy":{"const":"commit_view"}}}}},
                    "then":{"required":["posture"],"properties":{"posture":{"const":"discovery"}}}}));
            }
        }
    }
    let automatic = definitions
        .iter()
        .find(|d| d["name"] == "bridget_auto_delegate")
        .map(|d| d["inputSchema"].clone());
    if let Some(automatic) = automatic
        && let Some(tool) = definitions
            .iter_mut()
            .find(|d| d["name"] == "bridget_specialists")
        && let Some(branches) = tool["inputSchema"]["oneOf"].as_array_mut()
    {
        branches.push(json!({"type":"object","properties":{"version":{"const":1},"action":{"const":"request_temporary"},
                    "request_id":{"type":"string","minLength":1,"maxLength":128},"profile_id":{"type":"string"},
                    "profile_revision":{"type":"integer","minimum":1},"delegation":automatic},
                    "required":["version","action","request_id","profile_id","profile_revision","delegation"],"additionalProperties":false}));
    }
    definitions
}

pub(crate) fn execute(
    identity: &ResolvedIdentity,
    name: &str,
    args: &Value,
    socket: &Path,
) -> Result<Value, ClientError> {
    execute_with_proof(
        identity,
        name,
        args,
        socket,
        crate::t3code_mcp::private_proof(),
    )
}

fn execute_with_proof(
    identity: &ResolvedIdentity,
    name: &str,
    args: &Value,
    socket: &Path,
    proof: Option<bridget_transport::protocol::NativePermissionProof>,
) -> Result<Value, ClientError> {
    let object = args
        .as_object()
        .ok_or_else(|| ClientError::InvalidParams("arguments objet requis".into()))?;
    if name == "bridget_auto_delegate" && object.contains_key("agent_type") {
        return Err(ClientError::InvalidParams(
            "agent_type est interdit dans bridget_auto_delegate. Omettez ce champ ; utilisez engine pour le moteur (par exemple codex), et connection_id seulement pour une connexion précise du catalogue. La machine reste machine_id. Refus de forme avant connexion : cet appel n’a pas été envoyé. Si une demande antérieure a pu être enregistrée avec cette request_id, vérifiez son sort avant de corriger ses paramètres.".into()));
    }
    if name == "bridget_auto_delegate"
        && object.contains_key("service_tier")
        && !crate::provider_catalogue::MODEL_OPTIONS_LAUNCH_ENABLED
    {
        return Err(ClientError::InvalidParams(
            "service_tier : le choix d'un niveau de service n'est pas encore activé. Faire terminer son activation par le responsable Bridget avant de le demander pour une mission. Cet appel n'a pas été envoyé ; conserver la clé et vérifier toute demande antérieure avant un nouvel essai. Les autres travaux autorisés suivent leur propre état.".into()));
    }
    let allowed: &[&str] = match name {
        "bridget_specialists" => &[
            "version",
            "action",
            "request_id",
            "specialist_agent_id",
            "profile_id",
            "role_id",
            "project_id",
            "memory_id",
            "cursor",
            "limit",
            "expected_revision",
            "revision",
            "profile_revision",
            "durable",
            "auto_start",
            "official",
            "shared",
            "task",
            "text",
            "query",
            "kind",
            "mandate",
            "scope",
            "budget",
            "launch_policy",
            "definition",
            "roles",
            "source",
            "after_seq",
            "operation",
            "delegation",
        ],
        "bridget_capabilities" => &[],
        "bridget_delegate" => &[
            "request_id",
            "machine_id",
            "project_id",
            "agent_type",
            "model",
            "effort",
            "task",
            "cwd",
            "posture",
            "reply_timeout",
            "workspace",
            "child_requests",
            "keep_alive_secs",
            "brief",
            "fallback",
            "resume_on_limit",
            "quota_fallback_authorized",
        ],
        "bridget_auto_delegate" => &[
            "request_id",
            "quota_fallback_authorized",
            "machine_id",
            "project_id",
            "engine",
            "connection_id",
            "model",
            "effort",
            "service_tier",
            "workload",
            "expected_base_commit",
            "task",
            "cwd",
            "posture",
            "reply_timeout",
            "workspace",
            "child_requests",
            "keep_alive_secs",
            "brief",
            "urgent",
            "deliverable",
        ],
        "bridget_read_commit_file" => &["task_id", "path"],
        "bridget_refresh_models" => &["machine_id"],
        "bridget_task_repair_workspace" => {
            &["task_id", "request_id", "target_cwd", "expected_head"]
        }
        "bridget_task_continue" => &["task_id", "request_id", "task", "brief"],
        "bridget_task_respond" => &["task_id", "request_id", "decision", "answer"],
        "bridget_task_status" | "bridget_task_cancel" => &["task_id"],
        "bridget_admission_cancel" => &["request_id"],
        "bridget_task_wait" => &["task_ids", "wait_timeout"],
        "bridget_task_ack" => &["task_ids"],
        _ => return Err(ClientError::InvalidParams("outil natif inconnu".into())),
    };
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(ClientError::InvalidParams("champ inconnu".into()));
    }
    let string = |key: &str| -> Result<String, ClientError> {
        args.get(key)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                ClientError::InvalidParams(format!("{key} doit être une chaîne non vide"))
            })
    };
    if crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED
        && args.pointer("/workspace/strategy").and_then(Value::as_str) == Some("commit_view")
        && (name != "bridget_auto_delegate"
            || args.get("expected_base_commit").is_none()
            || args.get("posture").and_then(Value::as_str) != Some("discovery"))
    {
        return Err(ClientError::InvalidParams(
            "Une vue de commit exige bridget_auto_delegate, posture=discovery et expected_base_commit avec le SHA exact. Cet appel n'a pas été envoyé. Conservez la clé et vérifiez toute tâche ou reçu antérieur avant de modifier les paramètres ; aucun remplacement d'issue inconnue.".into()));
    }
    let request = match name {
        "bridget_specialists" => NativeDelegationRequest::Specialists {
            request: serde_json::from_value(args.clone()).map_err(|_| {
                let correction = if args.get("version").and_then(Value::as_u64).is_none() {
                    r#"Fournissez version:1 comme nombre JSON, sans guillemets. Le champ version est absent ou n'est pas un entier positif ; une chaîne comme "1" n'est pas acceptée."#
                } else if args.get("action").and_then(Value::as_str) == Some("catalogue") {
                    "Utilisez une action du schéma, par exemple search. L’action catalogue n’existe pas."
                } else {
                    "Vérifiez les champs et leurs types dans le schéma de bridget_specialists ; la forme de cet appel est invalide."
                };
                ClientError::InvalidParams(format!(
                    r#"{correction} Exemple de recherche : {{"version":1,"action":"search","role_id":"mainteneur-bridget"}}. Aucun appel envoyé : seul cet appel est refusé. Conservez les opérations déjà connues ; les autres travaux autorisés suivent leur propre état."#
                ))
            })?,
        },
        "bridget_read_commit_file"=>{
            let path=string("path")?;
            if !bridget_transport::protocol::commit_view_file_path(&path) {
                return Err(ClientError::InvalidParams("path doit être un chemin de fichier relatif exact de la carte du commit, sans préfixe absolu, ../, ./ ou séparateur inverse. Cet appel n'a pas été envoyé ; conserver toute tâche et opération déjà connue.".into()));
            }
            NativeDelegationRequest::ReadCommitFile{
                task_id:args.get("task_id").map(|_|string("task_id")).transpose()?,path}
        },
        "bridget_capabilities" => NativeDelegationRequest::Catalogue,
        "bridget_refresh_models" => NativeDelegationRequest::ProviderCatalogueV1 {refresh:true,machine_id:args.get("machine_id").map(|_|string("machine_id")).transpose()?},
        "bridget_delegate" => NativeDelegationRequest::Delegate {
            machine_id: args
                .get("machine_id")
                .map(|_| string("machine_id"))
                .transpose()?,
            project_id: args
                .get("project_id")
                .map(|_| string("project_id"))
                .transpose()?,
            // O(1) : au plus trois candidats, sans résolution ni gain de droits.
            quota_fallback_authorized: args.get("quota_fallback_authorized").map(|v|v.as_bool().ok_or_else(||ClientError::InvalidParams("quota_fallback_authorized exige un booléen après accord humain explicite.".into()))).transpose()?,
            resume_on_limit: match args.get("resume_on_limit") {
                None => None,
                Some(Value::Bool(value)) => Some(*value),
                _ => {
                    return Err(ClientError::InvalidParams(
                        "resume_on_limit doit être un booléen".into(),
                    ));
                }
            },
            fallback: match args.get("fallback") {
                None => Vec::new(),
                Some(Value::Array(values)) if (1..=3).contains(&values.len()) => {
                    let candidates: Vec<FallbackTarget> =
                        serde_json::from_value(Value::Array(values.clone()))
                            .map_err(|_| ClientError::InvalidParams("fallback invalide".into()))?;
                    let requested_type = string("agent_type")?;
                    let requested_model = string("model")?;
                    let requested_effort = args.get("effort").and_then(Value::as_str);
                    for (index, candidate) in candidates.iter().enumerate() {
                        if candidate.agent_type.trim().is_empty()
                            || candidate.model.trim().is_empty()
                            || candidate.agent_type.chars().any(char::is_control)
                            || candidate.model.chars().any(char::is_control)
                            || candidate.effort.as_ref().is_some_and(|value| {
                                value.trim().is_empty() || value.chars().any(char::is_control)
                            })
                            || values[index]
                                .get("effort")
                                .is_some_and(|value| !value.is_string())
                            || candidates[..index].contains(candidate)
                            || (candidate.agent_type == requested_type
                                && candidate.model == requested_model
                                && candidate.effort.as_deref() == requested_effort)
                        {
                            return Err(ClientError::InvalidParams("fallback invalide".into()));
                        }
                    }
                    candidates
                }
                _ => return Err(ClientError::InvalidParams("fallback invalide".into())),
            },
            request_id: string("request_id")?,
            agent_type: string("agent_type")?,
            model: string("model")?,
            effort: match args.get("effort") {
                None | Some(Value::Null) => None,
                Some(Value::String(value)) if !value.is_empty() => Some(value.clone()),
                _ => return Err(ClientError::InvalidParams("effort invalide".into())),
            },
            task: string("task")?,
            cwd: string("cwd")?,
            posture: match args.get("posture") {
                None => None,
                Some(Value::String(value)) if value == "discovery" => Some(SpawnPosture::Discovery),
                Some(Value::String(value)) if value == "development" => {
                    Some(SpawnPosture::Development)
                }
                _ => return Err(ClientError::InvalidParams("posture invalide".into())),
            },
            reply_timeout: match args.get("reply_timeout") {
                None | Some(Value::Null) => None,
                Some(Value::Number(value)) if value.is_u64() => value.as_u64(),
                _ => return Err(ClientError::InvalidParams("reply_timeout invalide".into())),
            },
            child_requests: match args.get("child_requests") {
                None => None,
                Some(Value::String(mode)) if matches!(mode.as_str(), "deny" | "forward") => {
                    Some(mode.clone())
                }
                _ => return Err(ClientError::InvalidParams("child_requests invalide".into())),
            },
            workspace: args.get("workspace").map(parse_workspace).transpose()?,
            keep_alive_secs: match args.get("keep_alive_secs") {
                None => None,
                Some(Value::Number(value)) if value.is_u64() => value.as_u64(),
                _ => {
                    return Err(ClientError::InvalidParams(
                        "keep_alive_secs invalide".into(),
                    ));
                }
            },
            brief: args.get("brief").cloned(),
        },
        "bridget_auto_delegate" => NativeDelegationRequest::AutoDelegateV2(AutoDelegateV2 {
            quota_fallback_authorized: args.get("quota_fallback_authorized").map(|v|v.as_bool().ok_or_else(||ClientError::InvalidParams("quota_fallback_authorized exige un booléen ; true seulement après accord humain explicite pour cette mission.".into()))).transpose()?,
            service_tier: args.get("service_tier").map(|_|string("service_tier")).transpose()?,
            request_id: string("request_id")?,
            machine_id: args
                .get("machine_id")
                .map(|_| string("machine_id"))
                .transpose()?,
            project_id: args
                .get("project_id")
                .map(|_| string("project_id"))
                .transpose()?,
            engine: args.get("engine").map(|_| string("engine")).transpose()?,
            connection_id: args
                .get("connection_id")
                .map(|_| string("connection_id"))
                .transpose()?,
            agent_type: None,
            model: args.get("model").map(|_| string("model")).transpose()?,
            effort: args.get("effort").map(|_| string("effort")).transpose()?,
            workload: match args.get("workload") {
                None => AutoDelegateWorkload::General,
                Some(Value::String(value)) if value == "general" => AutoDelegateWorkload::General,
                Some(Value::String(value)) if value == "compilation" => {
                    AutoDelegateWorkload::Compilation
                }
                _ => return Err(ClientError::InvalidParams("workload invalide".into())),
            },
            expected_base_commit: match args.get("expected_base_commit") {
                None => None,
                Some(_) => {
                    let value = string("expected_base_commit")?;
                    if value.len() != 40
                        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
                    {
                        return Err(ClientError::InvalidParams(
                            "expected_base_commit invalide : fournir un SHA Git complet de 40 caractères hexadécimaux (SHA-1), pas un SHA abrégé ni un nom de branche.".into(),
                        ));
                    }
                    Some(value)
                }
            },
            task: string("task")?,
            cwd: string("cwd")?,
            posture: match args.get("posture") {
                None => None,
                Some(Value::String(value)) if value == "discovery" => Some(SpawnPosture::Discovery),
                Some(Value::String(value)) if value == "development" => {
                    Some(SpawnPosture::Development)
                }
                _ => return Err(ClientError::InvalidParams("posture invalide".into())),
            },
            reply_timeout: match args.get("reply_timeout") {
                None => None,
                Some(Value::Number(value)) if value.is_u64() => value.as_u64(),
                _ => return Err(ClientError::InvalidParams("reply_timeout invalide".into())),
            },
            workspace: args.get("workspace").map(parse_workspace).transpose()?,
            keep_alive_secs: match args.get("keep_alive_secs") {
                None => None,
                Some(Value::Number(value)) if value.is_u64() => value.as_u64(),
                _ => {
                    return Err(ClientError::InvalidParams(
                        "keep_alive_secs invalide".into(),
                    ));
                }
            },
            brief: args.get("brief").cloned(),
            child_requests: match args.get("child_requests") {
                None => None,
                Some(Value::String(mode)) if matches!(mode.as_str(), "deny" | "forward") => {
                    Some(mode.clone())
                }
                _ => return Err(ClientError::InvalidParams("child_requests invalide".into())),
            },
            urgent: match args.get("urgent") {
                None => None,
                Some(Value::Bool(value)) => Some(*value),
                _ => return Err(ClientError::InvalidParams("urgent invalide".into())),
            },
            deliverable: args
                .get("deliverable")
                .map(|_| string("deliverable"))
                .transpose()?,
        }),
        "bridget_task_repair_workspace" => NativeDelegationRequest::RepairWorkspace {
            task_id: string("task_id")?,
            repair: bridget_transport::protocol::WorkspaceRepairRequestV1 {
                request_id: string("request_id")?,
                target_cwd: string("target_cwd")?,
                expected_head: string("expected_head")?,
            },
        },
        "bridget_task_continue" => NativeDelegationRequest::Continue {
            task_id: string("task_id")?,
            request_id: string("request_id")?,
            task: string("task")?,
            brief: args.get("brief").cloned(),
        },
        "bridget_task_respond" => NativeDelegationRequest::Respond {
            task_id: string("task_id")?,
            request_id: string("request_id")?,
            decision: match args.get("decision").and_then(Value::as_str) {
                Some("accept") => ChildRequestDecision::Accept,
                Some("decline") => ChildRequestDecision::Decline,
                Some("cancel") => ChildRequestDecision::Cancel,
                _ => {
                    return Err(ClientError::Technical {
                        code: "native_delegation_refused",
                        message: "decision_not_offered".into(),
                    });
                }
            },
            answer: match args.get("answer") {
                None => None,
                Some(value)
                    if value.is_object()
                        && serde_json::to_vec(value).is_ok_and(|bytes| bytes.len() <= 8192) =>
                {
                    Some(value.clone())
                }
                _ => {
                    return Err(ClientError::Technical {
                        code: "native_delegation_refused",
                        message: "answer_invalid".into(),
                    });
                }
            },
        },
        "bridget_task_status" => NativeDelegationRequest::Status {
            task_id: string("task_id")?,
        },
        "bridget_task_cancel" => NativeDelegationRequest::Cancel {
            task_id: string("task_id")?,
        },
        "bridget_admission_cancel" => NativeDelegationRequest::CancelAdmission {
            request_id: string("request_id")?,
        },
        "bridget_task_wait" => NativeDelegationRequest::Wait {
            task_ids: parse_task_ids(args)?,
            wait_timeout: Some(parse_wait_timeout(object)?),
        },
        "bridget_task_ack" => NativeDelegationRequest::Ack {
            task_ids: parse_task_ids(args)?,
        },
        _ => unreachable!(),
    };
    if let NativeDelegationRequest::RepairWorkspace { repair, .. } = &request {
        repair.validate().map_err(|_|ClientError::InvalidParams(
            "Réparation : request_id non vide, chemin absolu et SHA complet de 40 caractères requis.".into()))?;
    }
    if let NativeDelegationRequest::AutoDelegate(auto) = &request {
        let canonical = serde_json::to_vec(&request)
            .map_err(|_| ClientError::InvalidParams("auto_delegate invalide".into()))?;
        if auto.task.len() > 65_536
            || canonical.len() > crate::delegation::MAX_AUTO_ADMISSION_CANONICAL_BYTES
        {
            return Err(ClientError::InvalidParams(
                "auto_delegate hors limites".into(),
            ));
        }
    }
    if let NativeDelegationRequest::AutoDelegateV2(auto) = &request {
        let canonical = serde_json::to_vec(&request)
            .map_err(|_| ClientError::InvalidParams("auto_delegate_v2 invalide".into()))?;
        if auto.task.len() > 65_536
            || canonical.len() > crate::delegation::MAX_AUTO_ADMISSION_CANONICAL_BYTES
        {
            return Err(ClientError::InvalidParams(
                "auto_delegate_v2 hors limites".into(),
            ));
        }
    }
    if matches!(
        &request,
        NativeDelegationRequest::Wait { .. } | NativeDelegationRequest::Ack { .. }
    ) && !daemon_supports_wait(socket)
        .map_err(|e| pre_dispatch_failure(e, "lecture des capacités d’attente", args))?
    {
        return Err(ClientError::Technical {
            code: "native_delegation_refused",
            message: "wait_unsupported".into(),
        });
    }
    let mut connection = if let NativeDelegationRequest::Wait { wait_timeout, .. } = &request {
        let deadline = Instant::now()
            + Duration::from_secs(wait_timeout.unwrap_or(WAIT_TIMEOUT_DEFAULT_SECS) + 10);
        let mut connection = registered_connection_until(
            &identity.name,
            &identity.instance_id,
            socket,
            Instant::now() + Duration::from_secs(3),
        )
        .map_err(|e| pre_dispatch_failure(e, "raccordement de cette session", args))?;
        connection.set_deadline(deadline);
        connection
    } else {
        registered_connection(&identity.name, &identity.instance_id, socket)
            .map_err(|e| pre_dispatch_failure(e, "raccordement de cette session", args))?
    };
    if matches!(&request, NativeDelegationRequest::Specialists { .. }) {
        let cap = bridget_transport::protocol::ClientCapability::SpecialistsV1;
        match connection.exchange(&WrapperToDaemon::ClientHello{contract_version:bridget_transport::protocol::CLIENT_CONTRACT_VERSION,
            issuer_scope:crate::communication::issuer_scope("bridget-specialists-agent"),capabilities:vec![cap]})? {
            DaemonToWrapper::ClientWelcome{capabilities,..} if capabilities.contains(&cap)=>{},
            _=>return Err(ClientError::Technical{code:"specialists_capability_required",message:"Faire charger le client et le daemon specialists_v1. Cet appel n'a pas été transmis ; conserver toute demande antérieure.".into()}),
        }
    }
    let required_feature = match &request {
        NativeDelegationRequest::CancelAdmission { .. } => Some((
            "admission_cancel_v1",
            "admission_cancel_master_upgrade_required",
        )),
        NativeDelegationRequest::ReadCommitFile { .. } => Some((
            "commit_view_files_v1",
            "commit_view_file_master_upgrade_required",
        )),
        NativeDelegationRequest::RepairWorkspace { .. } => Some((
            "task_workspace_repair_v1",
            "workspace_repair_master_upgrade_required",
        )),

        NativeDelegationRequest::Delegate {
            machine_id: Some(_),
            ..
        } => Some(("remote_worker_v1", "remote_worker_unsupported")),
        NativeDelegationRequest::AutoDelegate(_) => {
            Some(("auto_delegate_v1", "auto_delegate_unsupported"))
        }
        NativeDelegationRequest::AutoDelegateV2(auto) if auto.service_tier.is_some() => {
            Some(("auto_delegate_v2", "model_options_master_upgrade_required"))
        }
        NativeDelegationRequest::AutoDelegateV2(auto)
            if matches!(auto.workspace, Some(WorkspaceRequest::CommitView {})) =>
        {
            Some(("auto_delegate_v2", "commit_view_master_upgrade_required"))
        }
        NativeDelegationRequest::AutoDelegateV2(_) => {
            Some(("auto_delegate_v2", "auto_delegate_v2_unsupported"))
        }
        _ => None,
    };
    if let Some((feature, refusal)) = required_feature {
        let supported = match connection.exchange(&WrapperToDaemon::DaemonIdentityRequest)? {
            DaemonToWrapper::DaemonIdentityReport { features, .. } => {
                features.iter().any(|candidate| candidate == feature)
                    && (!matches!(&request,NativeDelegationRequest::AutoDelegateV2(auto)
                            if matches!(auto.workspace,Some(WorkspaceRequest::CommitView {})))
                        || features
                            .iter()
                            .any(|candidate| candidate == "worker_workspace_view_create_v1"))
                    && (!matches!(&request,NativeDelegationRequest::AutoDelegateV2(auto) if auto.service_tier.is_some())
                        || features
                            .iter()
                            .any(|candidate| candidate == "model_options_v1"))
            }
            other => return unexpected_response(other),
        };
        if !supported {
            return Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: if refusal == "admission_cancel_master_upgrade_required" {
                    "admission_cancel_master_upgrade_required : Faire charger le parcours admission_cancel_v1 sur le maître. Cette commande n’a pas été transmise et n’a rien annulé. Conserver la clé et tout task_id connu ; ne pas rejouer l’admission pour l’annuler ni changer de canal ou d’identité. Les autres travaux autorisés suivent leur propre état.".into()
                } else if refusal == "commit_view_master_upgrade_required" {
                    "commit_view_master_upgrade_required : Le maître ne déclare pas le parcours de création des vues. Demandez au responsable Bridget de charger et qualifier ce parcours et son retour compatible. Cette demande n'a pas été transmise pour admission ; conservez la clé et toute tâche ou reçu antérieur avant un rejeu strict après résolution. Ne créez pas de worktree pour contourner ce refus. Les autres travaux autorisés suivent leur propre état.".into()
                } else if refusal == "model_options_master_upgrade_required" {
                    "model_options_master_upgrade_required : Le maître n'a pas confirmé les capacités d'admission et de choix d'options requises pour cette demande. Faire charger et qualifier ce parcours par le responsable Bridget. Cet appel n'a pas été transmis pour admission ; conserver la clé et tout reçu antérieur. Ne pas changer le modèle, le compte, le niveau ou les droits. Les autres travaux autorisés suivent leur propre état.".into()
                } else {
                    refusal.into()
                },
            });
        }
    }
    // Une admission peut préparer un dépôt ou un worktree. Le délai métier
    // de la mission reste distinct ; les lectures gardent leur budget court.
    let continuation = match &request {
        NativeDelegationRequest::Continue {
            task_id,
            request_id,
            ..
        } => Some((task_id.clone(), request_id.clone())),
        _ => None,
    };
    let commit_file = matches!(&request, NativeDelegationRequest::ReadCommitFile { .. });
    let admission = matches!(
        &request,
        NativeDelegationRequest::AutoDelegateV2(_)
            | NativeDelegationRequest::AutoDelegate(_)
            | NativeDelegationRequest::Delegate { .. }
            | NativeDelegationRequest::RepairWorkspace { .. }
    );
    if admission {
        connection.set_deadline(Instant::now() + Duration::from_secs(600));
    } else if commit_file {
        connection.set_deadline(
            Instant::now()
                + Duration::from_secs(125)
                + if proof.is_some() {
                    crate::t3code_mcp::ATTESTATION_MAX_BUDGET
                } else {
                    Duration::ZERO
                },
        );
    }
    let recover_request_id = match &request {
        NativeDelegationRequest::AutoDelegateV2(auto) => Some(auto.request_id.clone()),
        NativeDelegationRequest::Delegate { request_id, .. } => Some(request_id.clone()),
        _ => None,
    };
    let admission_key232 = recover_request_id.clone();
    let attestation_budget = if proof.is_some() {
        crate::t3code_mcp::ATTESTATION_MAX_BUDGET
    } else {
        Duration::ZERO
    };
    if !attestation_budget.is_zero() && !admission && !commit_file {
        let wait = match &request {
            NativeDelegationRequest::Wait { wait_timeout, .. } => {
                Duration::from_secs(wait_timeout.unwrap_or(WAIT_TIMEOUT_DEFAULT_SECS))
            }
            _ => Duration::ZERO,
        };
        connection.set_deadline(
            Instant::now()
                + crate::communication::client::DAEMON_BUDGET
                + attestation_budget
                + wait,
        );
    }
    if continuation.is_some() {
        // Sources d'origine (30 s), preuve worker (30 s), puis reçu durable.
        // Ajouter l'attestation sans étendre les budgets des lectures ordinaires.
        connection.set_deadline(
            Instant::now()
                + crate::communication::client::DAEMON_BUDGET
                + attestation_budget
                + Duration::from_secs(60),
        );
    }
    let mutation = !matches!(&request,NativeDelegationRequest::Specialists{request} if request.is_read())
        && !matches!(
            &request,
            NativeDelegationRequest::Catalogue
                | NativeDelegationRequest::RoutingCatalogueV1 { .. }
                | NativeDelegationRequest::ProviderCatalogueV1 { refresh: false, .. }
                | NativeDelegationRequest::CallerScope { .. }
                | NativeDelegationRequest::Status { .. }
                | NativeDelegationRequest::Wait { .. }
        );
    let message = match proof {
        Some(proof) => WrapperToDaemon::NativeDelegationT3 { request, proof },
        None => WrapperToDaemon::NativeDelegation { request },
    };
    let response = if let Some(request_id) = recover_request_id {
        let is_pending = |response: &DaemonToWrapper| {
            matches!(response, DaemonToWrapper::NativeDelegationResult { result }
                if result["status"] == "refused" && result["code"] == "admission_pending")
        };
        let exchange = |recovery_deadline| {
            if let Some(deadline) = recovery_deadline {
                connection = registered_connection_until(&identity.name, &identity.instance_id, socket, deadline)?;
            }
            // Même commande, même preuve et même clé. Le maître récupère son
            // reçu durable ; cette reprise ne crée pas une nouvelle mission.
            connection.send_then_wait(&message)
        };
        if attestation_budget.is_zero() {
            recover_admission_reply171(&request_id, is_pending, exchange)
        } else {
            recover_admission_with_budget171(&request_id, is_pending, exchange,
                crate::communication::client::DAEMON_BUDGET + attestation_budget)
        }
    } else if mutation {
        // Continue, Respond, Cancel et Ack peuvent avoir été appliqués avant
        // une coupure. Conserver cette incertitude sans les rejouer ici.
        connection.send_then_wait(&message)
    } else {
        connection.exchange(&message)
    }.map_err(|error| match error {
        ClientError::Technical { code: code @ ("daemon_unreachable" | "outcome_unknown"), message } => {
            let recovery = if let Some((task_id, request_id)) = &continuation {
                format!("Consultez bridget_task_status avec task_id={task_id}. La continuation request_id={request_id} peut déjà être enregistrée et remise malgré la perte de réponse. Conservez cette tâche, cette clé et l’enveloppe ; ne renvoyez pas la consigne et ne créez pas de remplacement tant que le sort de la ronde reste inconnu. Ce refus concerne la réponse à cet appel, pas un arrêt attesté de la tâche. Les autres travaux autorisés suivent leur propre état. Si le statut reste inaccessible, demandez au responsable Bridget de vérifier le reçu de cette ronde.")
            } else {
                "Attendez le retour du lien, puis consultez bridget_task_status avec le même task_id. Sans task_id, conservez request_id et tous les paramètres pour retrouver le sort de l’admission ; ne créez pas une autre demande tant que ce sort reste inconnu.".into()
            };
            ClientError::Technical { code, message: format!("{message} {recovery}") }
        },
        other => other,
    })?;
    match response {
        DaemonToWrapper::NativeDelegationResult { mut result } if result["status"] == "refused" => {
            admission_refusal_context232(&mut result, admission_key232.as_deref());
            Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: match result.get("detail").filter(|value| !value.is_null()) {
                    Some(detail) => format!(
                        "{} {}",
                        result["code"].as_str().unwrap_or("refused"),
                        detail
                    ),
                    None => result["code"].as_str().unwrap_or("refused").to_string(),
                },
            })
        }
        DaemonToWrapper::NativeDelegationResult { mut result } => {
            if name == "bridget_capabilities" {
                result["features"]["resume_on_limit"] = json!(true);
                result["features"]["resume_max_wait_secs"] = json!(21600);
                result["features"]["resume_max_rounds"] = json!(2);
                // Conserve le booléen held attesté par le daemon et les métadonnées par type.
                result["queue"]["actions"] = json!(["list", "cancel", "reorder", "promote"]);
                result["guidance"] = json!({
                    "intents":["queue","steer","interrupt"],
                    "interrupt_cooldown_secs":30,"interrupt_cap_per_10min":3,
                    "queue_tool":true
                });
            }
            Ok(result)
        }
        other => unexpected_response(other),
    }
}

fn admission_refusal_context232(result: &mut Value, request_id: Option<&str>) {
    let Some(request_id) = request_id else {
        return;
    };
    if result["code"] != "permission_attestation_unavailable" {
        return;
    }
    if !result["detail"].is_object() {
        result["detail"] = json!({});
    }
    if let Some(message) = result["detail"].get("message").cloned() {
        result["detail"]["attestation_message"] = message;
    }
    result["detail"]["message"] = json!(
        "Une mission peut déjà être admise avec cette clé. Le contrôle d'attestation de cet appel a échoué ; le sort de l'admission reste inconnu."
    );
    result["detail"]["admission"] = json!({"request_id":request_id,"state":"outcome_unknown"});
    result["detail"]["admission_recovery"] = json!(
        "Le contrôle de cet appel a échoué ; cela ne prouve pas l'absence d'admission. La tâche peut déjà exister après une réponse perdue ou une relecture. Conserver cette même request_id et tous les paramètres ; vérifier son reçu après le retour du lien, sans appel manuel censé réparer les droits, sans nouvelle clé et sans mission de remplacement."
    );
}

fn recover_admission_reply171<T>(
    request_id: &str,
    is_pending: impl Fn(&T) -> bool,
    exchange: impl FnMut(Option<Instant>) -> Result<T, ClientError>,
) -> Result<T, ClientError> {
    recover_admission_with_budget171(
        request_id,
        is_pending,
        exchange,
        crate::communication::client::DAEMON_BUDGET,
    )
}

fn recover_admission_with_budget171<T>(
    request_id: &str,
    is_pending: impl Fn(&T) -> bool,
    mut exchange: impl FnMut(Option<Instant>) -> Result<T, ClientError>,
    recovery_budget: Duration,
) -> Result<T, ClientError> {
    let first = exchange(None);
    if !matches!(
        &first,
        Err(ClientError::Technical {
            code: "outcome_unknown",
            ..
        })
    ) && !first.as_ref().is_ok_and(&is_pending)
    {
        return first;
    }
    // Le même budget de récupération couvre toutes les relectures.
    // Même clé/enveloppe/preuve ; aucun nouveau choix ni lancement au rejeu.
    let deadline = Instant::now() + recovery_budget;
    loop {
        if Instant::now() >= deadline {
            return Err(ClientError::Technical {
                code: "outcome_unknown",
                message: format!(
                    "Rétablissez le lien natif, puis relisez la demande {request_id} avec la même clé et les mêmes arguments. Son issue finale reste inconnue : elle peut être en attente, admise ou refusée. Aucun nouveau lancement n'est justifié par cette réponse. Ne créez pas de doublon. Ce constat concerne cette lecture ; les autres travaux autorisés suivent leur propre état. Si le lien reste indisponible, demandez au responsable de lire le reçu de cette clé."
                ),
            });
        }
        match exchange(Some(deadline)) {
            Ok(value) if is_pending(&value) => {
                std::thread::sleep(
                    Duration::from_millis(250)
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
            Err(ClientError::Technical {
                code: "outcome_unknown" | "daemon_unreachable",
                ..
            }) => {
                // Une coupure de maintenance n'épuise pas à elle seule le budget.
                // Relire le même reçu ; une réponse terminale sera rendue telle quelle.
                std::thread::sleep(
                    Duration::from_millis(250)
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
            other => return other,
        }
    }
}

#[cfg(test)]
#[path = "delegation_mcp_admission_recovery171_tests.rs"]
mod admission_recovery171_tests;

#[cfg(test)]
#[path = "delegation_mcp_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "delegation_mcp_spec172_transport_tests.rs"]
mod spec172_transport_tests;

#[cfg(test)]
#[test]
fn spec207_auto_duration_explains_default_and_replay() {
    let tool = tools(&json!({}))
        .into_iter()
        .find(|t| t["name"] == "bridget_auto_delegate")
        .unwrap();
    let duration = &tool["inputSchema"]["properties"]["reply_timeout"];
    assert_eq!(duration["default"], 3600);
    assert_eq!(duration["maximum"], 86400);
    let help = duration["description"].as_str().unwrap();
    for term in [
        "14400",
        "parent",
        "rejeu",
        "keep_alive_secs",
        "wait_timeout",
    ] {
        assert!(help.contains(term));
    }
}

#[cfg(test)]
mod schema211_tests {
    use super::*;
    #[test]
    fn spec211_expected_base_schema_is_conditional_and_explicit() {
        let list = tools(&json!({}));
        let auto = list
            .iter()
            .find(|t| t["name"] == "bridget_auto_delegate")
            .unwrap();
        assert!(
            !auto["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .contains(&json!("expected_base_commit"))
        );
        assert_eq!(
            auto["inputSchema"]["allOf"][0]["then"]["required"],
            json!(["expected_base_commit"])
        );
        assert!(
            auto["inputSchema"]["properties"]["expected_base_commit"]["description"]
                .as_str()
                .unwrap()
                .contains("Obligatoire")
        );
        assert!(auto["description"].as_str().unwrap().contains("hors Git"));
    }
}

#[cfg(test)]
mod continuation_receipt221_tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    fn continuation_response(disconnect: bool) {
        let socket = crate::mcp_identity::mock_socket("221-continuation-receipt");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(15)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "RegisterAuxiliary"
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["request"]["operation"], "continue");
            assert_eq!(request["request"]["task_id"], "same-task");
            assert_eq!(request["request"]["request_id"], "same-round-key");
            if disconnect {
                writeln!(stream, "{}", json!({"type":"Disconnect"})).unwrap();
            } else {
                // Une preuve autorisée peut dépasser les 10 s du budget lecture.
                std::thread::sleep(Duration::from_secs(11));
                writeln!(stream, "{}", json!({"type":"NativeDelegationResult","result":{
                    "task_id":"same-task","status":"mission_pending","round":2,"message_id":"same-message"}})).unwrap();
            }
            line.clear();
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "aucun rejeu automatique"
            );
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute_with_proof(
            &identity,
            "bridget_task_continue",
            &json!({
            "task_id":"same-task","request_id":"same-round-key","task":"suite"}),
            &socket,
            None,
        );
        backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        if disconnect {
            assert!(
                matches!(result, Err(ClientError::Technical {code:"outcome_unknown", message})
                if message.contains("task_id=same-task") && message.contains("request_id=same-round-key")
                    && message.contains("peut déjà être enregistrée") && message.contains("ne renvoyez pas")
                    && message.contains("autres travaux autorisés"))
            );
        } else {
            let result = result.expect("le reçu tardif doit être lu intégralement");
            assert_eq!(result["round"], 2);
            assert_eq!(result["message_id"], "same-message");
        }
    }
    #[test]
    fn spec221_continuation_receipt_outlives_read_budget_without_resend() {
        continuation_response(false);
    }
    #[test]
    fn spec221_continuation_lost_response_preserves_identifiers_without_resend() {
        continuation_response(true);
    }
}

fn specialists_schema222() -> Value {
    let text = json!({"type":"string","minLength":1,"maxLength":256});
    let scope = json!({"type":"object","properties":{"projects":{"type":"array","maxItems":64,"items":text},"transversal":{"type":"boolean"}},"additionalProperties":false});
    let budget = json!({"type":["object","null"],"properties":{"max_duration_secs":{"type":["integer","null"],"minimum":0},"max_facturable_tokens":{"type":["integer","null"],"minimum":0},"max_descendants":{"type":["integer","null"],"minimum":0}},"additionalProperties":false});
    let launch = json!({"type":["object","null"],"properties":{"agent_type":text,"cwd":{"type":"string","minLength":1,"maxLength":4096},"posture":{"enum":["discovery","development"]},"prepare_requirements":{"type":"boolean"}},"required":["agent_type","cwd","posture"],"additionalProperties":false});
    let definition = json!({"type":"object","properties":{
        "profile_id":text,"name":{"type":"string","minLength":1,"maxLength":80},"kind":{"enum":["profile","team"]},
        "instructions":{"type":"string","maxLength":8000},"roles":{"type":"array","maxItems":32,"items":text},
        "procedures":{"type":"array","maxItems":32,"items":{"type":"object","properties":{"name":text,"instructions":{"type":"string","maxLength":8000}},"required":["name","instructions"],"additionalProperties":false}},
        "requirements":{"type":"array","maxItems":32,"items":{"type":"object","properties":{"kind":{"enum":["tool","capability","program"]},"name":text},"required":["kind","name"],"additionalProperties":false}},
        "preferences":{"type":"object","properties":{"machine_id":text,"engine":text,"model":text,"effort":text},"additionalProperties":false},
        "members":{"type":"array","maxItems":32,"items":{"type":"object","properties":{"key":text,"role_id":text,"deliverable":{"type":"string","minLength":1,"maxLength":4000},"depends_on":{"type":"array","items":text,"maxItems":32}},"required":["key","role_id","deliverable"],"additionalProperties":false}},
        "provenance":{"type":["string","null"],"maxLength":4096}
    },"required":["profile_id","name","kind","instructions"],"additionalProperties":false});
    let mut fields = serde_json::Map::new();
    for name in [
        "request_id",
        "specialist_agent_id",
        "profile_id",
        "role_id",
        "project_id",
        "memory_id",
        "cursor",
    ] {
        fields.insert(name.into(), text.clone());
    }
    fields.insert(
        "limit".into(),
        json!({"type":"integer","minimum":1,"maximum":100}),
    );
    fields.insert(
        "expected_revision".into(),
        json!({"type":"integer","minimum":0}),
    );
    for name in ["revision", "profile_revision"] {
        fields.insert(name.into(), json!({"type":"integer","minimum":1}));
    }
    for name in ["durable", "auto_start", "official", "shared"] {
        fields.insert(name.into(), json!({"type":"boolean"}));
    }
    fields.insert(
        "task".into(),
        json!({"type":"string","minLength":1,"maxLength":32000}),
    );
    fields.insert(
        "text".into(),
        json!({"type":"string","minLength":1,"maxLength":16000}),
    );
    fields.insert("query".into(), json!({"type":"string","maxLength":4096}));
    fields.insert(
        "kind".into(),
        json!({"enum":["fact","decision","hypothesis","reported_result"]}),
    );
    fields.insert(
        "mandate".into(),
        json!({"type":["string","null"],"maxLength":16000}),
    );
    fields.insert("scope".into(), scope.clone());
    fields.insert("budget".into(), budget);
    fields.insert("launch_policy".into(), launch);
    fields.insert("definition".into(), definition);
    fields.insert("roles".into(),json!({"type":"array","maxItems":32,"items":{"type":"object","properties":{"role_id":text,"label":{"type":"string","minLength":1,"maxLength":160},"scope":scope},"required":["role_id","label","scope"],"additionalProperties":false}}));
    fields.insert("source".into(),json!({"oneOf":[
        {"type":"object","properties":{"kind":{"const":"statement"},"request_id":text},"required":["kind","request_id"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"task"},"task_id":text,"round":{"type":"integer","minimum":1},"part":{"type":"integer","minimum":0},"digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["kind","task_id","round","digest"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"message"},"message_id":text,"in_reply_to":text,"part":{"type":"integer","minimum":0},"digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["kind","message_id","in_reply_to","digest"],"additionalProperties":false},
        {"type":"object","properties":{"kind":{"const":"journal"},"agent_id":text,"file":{"type":"string","maxLength":4096},"from_seq":{"type":"integer","minimum":0},"to_seq":{"type":"integer","minimum":0},"part":{"type":"integer","minimum":0},"digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["kind","agent_id","file","from_seq","to_seq","digest"],"additionalProperties":false}
    ]}));
    fields.insert("after_seq".into(), json!({"type":"integer","minimum":0}));
    let rows = [
        (
            "capture_history",
            vec!["request_id", "specialist_agent_id"],
            vec!["project_id", "after_seq", "limit"],
        ),
        ("profiles", vec![], vec!["cursor", "limit"]),
        (
            "profile_versions",
            vec!["profile_id"],
            vec!["cursor", "limit"],
        ),
        (
            "attach_profile",
            vec![
                "request_id",
                "profile_id",
                "profile_revision",
                "expected_revision",
            ],
            vec![],
        ),
        ("list", vec![], vec!["cursor", "limit"]),
        (
            "search",
            vec!["role_id"],
            vec!["project_id", "cursor", "limit"],
        ),
        ("get", vec!["specialist_agent_id"], vec![]),
        (
            "declare_roles",
            vec!["request_id", "expected_revision", "roles"],
            vec![],
        ),
        ("propose_profile", vec!["request_id", "definition"], vec![]),
        (
            "propose_revision",
            vec!["request_id", "expected_revision", "definition"],
            vec![],
        ),
        (
            "approve_revision",
            vec!["request_id", "profile_id", "revision"],
            vec![],
        ),
        (
            "activate",
            vec![
                "request_id",
                "specialist_agent_id",
                "profile_id",
                "profile_revision",
                "expected_revision",
                "scope",
                "durable",
                "auto_start",
            ],
            vec!["mandate", "budget", "launch_policy"],
        ),
        (
            "disable",
            vec!["request_id", "specialist_agent_id", "expected_revision"],
            vec![],
        ),
        (
            "configure_mandate",
            vec![
                "request_id",
                "specialist_agent_id",
                "expected_revision",
                "mandate",
                "scope",
                "auto_start",
            ],
            vec!["budget", "launch_policy"],
        ),
        (
            "revoke",
            vec!["request_id", "specialist_agent_id", "expected_revision"],
            vec![],
        ),
        (
            "designate_role",
            vec!["request_id", "specialist_agent_id", "role_id", "official"],
            vec![],
        ),
        (
            "memory_recall",
            vec!["specialist_agent_id", "project_id", "query"],
            vec!["cursor", "limit"],
        ),
        (
            "memory_versions",
            vec!["memory_id"],
            vec!["cursor", "limit"],
        ),
        (
            "memory_write",
            vec![
                "request_id",
                "specialist_agent_id",
                "project_id",
                "shared",
                "kind",
                "text",
                "source",
            ],
            vec![],
        ),
        (
            "memory_correct",
            vec![
                "request_id",
                "memory_id",
                "expected_revision",
                "text",
                "kind",
            ],
            vec![],
        ),
        (
            "memory_forget",
            vec!["request_id", "memory_id", "expected_revision"],
            vec![],
        ),
        ("export", vec!["profile_id", "revision"], vec![]),
        ("import", vec!["request_id", "definition"], vec![]),
        (
            "convert",
            vec![
                "request_id",
                "specialist_agent_id",
                "profile_id",
                "profile_revision",
                "expected_revision",
            ],
            vec![],
        ),
        (
            "request_role",
            vec!["request_id", "role_id", "project_id", "task"],
            vec!["specialist_agent_id"],
        ),
        ("request_status", vec!["request_id"], vec![]),
        (
            "request_team",
            vec!["request_id", "profile_id", "revision", "project_id", "task"],
            vec![],
        ),
    ];
    let mut branches = Vec::new();
    for (action, required, optional) in rows {
        let mut properties = serde_json::Map::new();
        properties.insert("version".into(), json!({"const":1}));
        properties.insert("action".into(), json!({"const":action}));
        for name in required.iter().chain(optional.iter()) {
            properties.insert((*name).into(), fields[*name].clone());
        }
        let mut req = vec!["version", "action"];
        req.extend(required);
        if action == "memory_versions" {
            properties.insert("cursor".into(), json!({"type":"integer","minimum":1}));
        }
        branches.push(json!({"type":"object","properties":properties,"required":req,"additionalProperties":false}));
    }
    let approved = branches
        .iter()
        .filter(|v| {
            matches!(
                v["properties"]["action"]["const"].as_str(),
                Some(
                    "approve_revision"
                        | "activate"
                        | "disable"
                        | "configure_mandate"
                        | "revoke"
                        | "designate_role"
                        | "import"
                        | "convert"
                        | "request_role"
                )
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    branches.push(json!({"type":"object","properties":{"version":{"const":1},"action":{"const":"request_approval"},
        "request_id":{"type":"string","format":"uuid"},"operation":{"oneOf":approved}},
        "required":["version","action","request_id","operation"],"additionalProperties":false}));
    json!({"type":"object","oneOf":branches})
}

#[cfg(test)]
mod tracking230_preflight_tests {
    use super::*;

    #[test]
    fn spec259_preflight_refs_name_bounded_ack_ids_and_preserve_refusals() {
        let error = || ClientError::Technical {
            code: "daemon_unreachable",
            message: "budget JSONL dépassé".into(),
        };
        let valid = json!({"task_ids":["task259", "other-task"]});
        let ClientError::Technical { message, .. } = pre_dispatch_failure(error(), "test", &valid)
        else {
            panic!()
        };
        assert!(message.contains("task_ids=[\"task259\",\"other-task\"]"));
        for invalid in [
            json!({"task_ids":["x".repeat(129)]}),
            json!({"task_ids":["task\nforged"]}),
            json!({"task_ids":["task259","task259"]}),
            json!({"task_ids":[3]}),
            json!({"task_ids":(0..17).map(|n| format!("task-{n}")).collect::<Vec<_>>()}),
        ] {
            let ClientError::Technical { message, .. } =
                pre_dispatch_failure(error(), "test", &invalid)
            else {
                panic!()
            };
            assert!(message.contains("les identifiants des tâches de cet appel"));
            assert!(!message.contains("task_ids="));
        }
        assert!(matches!(pre_dispatch_failure(ClientError::Technical {
            code:"native_delegation_refused",message:"refus conservé".into()}, "test", &valid),
            ClientError::Technical {code:"native_delegation_refused",message} if message=="refus conservé"));
    }

    #[test]
    fn spec259_wait_registration_failure_names_ids_without_wait_dispatch() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        let socket = crate::mcp_identity::mock_socket("259-wait-registration");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut line = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            writeln!(stream, "{}", json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["wait_v1"]})).unwrap();
            drop(stream);
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream);
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert_eq!(
                    serde_json::from_str::<Value>(&line).unwrap()["type"],
                    "RegisterAuxiliary"
                );
            }
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute(
            &identity,
            "bridget_task_wait",
            &json!({"task_ids":["task259"],"wait_timeout":1}),
            &socket,
        );
        assert!(
            matches!(&result, Err(ClientError::Technical {code:"daemon_unreachable",message})
            if message.contains("raccordement de cette session") && message.contains("task259")
                && message.contains("n'a pas été transmise")),
            "refus inattendu : {result:?}"
        );
        server.join().unwrap();
    }
    #[test]
    fn tracking230_refus_avant_depart_precise_etape_et_tache_sans_accuser_le_modele() {
        let error = pre_dispatch_failure(
            ClientError::Technical {
                code: "daemon_unreachable",
                message: "budget JSONL dépassé".into(),
            },
            "lecture des capacités d’attente",
            &json!({"task_id":"t230"}),
        );
        let ClientError::Technical { code, message } = error else {
            panic!()
        };
        assert_eq!(code, "daemon_unreachable");
        for text in [
            "task_id=t230",
            "n'a pas été transmise",
            "cause précise",
            "cet appel",
            "autres travaux autorisés",
        ] {
            assert!(message.contains(text));
        }
        assert!(matches!(
            pre_dispatch_failure(
                ClientError::InvalidParams("forme".into()),
                "test",
                &json!({})
            ),
            ClientError::InvalidParams(_)
        ));
    }
}
