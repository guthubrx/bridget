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

pub(crate) const WAIT_TIMEOUT_DEFAULT_SECS: u64 = 600;
pub(crate) const WAIT_TIMEOUT_MAX_SECS: u64 = 3600;

/// Validation commune des deux outils d'attente : forme JSON puis bornes métier.
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
        {"type":"object","properties":{"strategy":{"const":"shared"}},"required":["strategy"],"additionalProperties":false},
        {"type":"object","properties":{"strategy":{"const":"worktree"},"base_ref":{"type":"string","description":"Référence Git disponible dans cwd sur la machine du parent. Pour une base figée, fournir le même SHA complet que expected_base_commit ; un nom de branche peut avancer. Un commit présent uniquement sur la destination doit d’abord être récupéré dans le dépôt source."},"branch":{"type":"string"}},"required":["strategy"],"additionalProperties":false}
    ]});
    vec![
        json!({"name":"bridget_capabilities","description":"Catalogue natif des fournisseurs, modèles, efforts et postures accessibles au parent. can_run et constraints donnent le verdict de lancement au dossier verdict_context.cwd, avec les droits attestés de la session. Les faits limités à une demande et les autres dossiers restent soumis à l’admission. Les binaires sont vérifiés ; aucune connexion fournisseur ni authentification n’est testée. Fonctionne sans T3.","inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_delegate","description":"Crée un enfant Bridget et remet sa mission en un appel natif. Conserver request_id sur retry identique ; tâche, résultat et annulation appartiennent au parent. keep_alive_secs garde l'enfant après la remise pour une nouvelle ronde. Le brief est déclaré par le parent ; sources_not_verified : aucune source citée n'est vérifiée. Le journal peut être lu plus largement que l'enfant ; aucun secret dans le brief. Corps du dossier ≤ 16 Kio, refusé plutôt que tronqué.","inputSchema":{"type":"object","properties":{"request_id":{"type":"string","minLength":1,"maxLength":128},"agent_type":{"type":"string"},"model":{"type":"string"},"effort":{"type":["string","null"]},"task":{"type":"string","minLength":1,"maxLength":65536},"machine_id":{"type":"string","minLength":1,"maxLength":256,"description":"Machine de destination de la mission. Le champ cwd reste le dossier source sur la machine du parent."},"project_id":{"type":"string","minLength":1,"maxLength":256},"cwd":{"type":"string","minLength":1,"description":"Chemin absolu du dossier source sur la machine du parent appelant. Même avec machine_id distant, ne pas fournir le chemin de destination ; Bridget traduit le chemin via le mapping du projet."},"posture":{"type":"string","enum":["discovery","development"]},"reply_timeout":{"type":"integer","minimum":60,"maximum":86400,"description":"Délai de réponse de la mission en secondes (défaut 3600)."},"workspace":workspace_schema,"child_requests":{"type":"string","enum":["deny","forward"],"description":"Remonter les demandes de l’enfant Codex ; absent : deny, politique inchangée."},"keep_alive_secs":{"type":"integer","minimum":60,"maximum":7200,"description":"Durée de garde après la remise du résultat ; absent : aucune garde."},"resume_on_limit":{"type":"boolean","description":"Attendre puis reprendre après une limite d’usage attestée ; absent ou faux : inactif. Au plus 6 h et 2 rondes, même fournisseur."},"brief":brief_schema,"fallback":{"type":"array","minItems":1,"maxItems":3,"items":{"type":"object","properties":{"agent_type":{"type":"string","minLength":1},"model":{"type":"string","minLength":1},"effort":{"type":"string","minLength":1}},"required":["agent_type","model"],"additionalProperties":false}}},"required":["request_id","agent_type","model","task","cwd"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_auto_delegate","description":"Demande une admission de flotte V2. Les axes machine, moteur, connexion, modèle et effort restent indépendants. cwd désigne le dossier source sur la machine du parent ; machine_id désigne la destination. expected_base_commit exige un SHA complet (40 caractères hexadécimaux (SHA-1)). Conserver request_id au rejeu strict. Un maître sans auto_delegate_v2 refuse explicitement.","inputSchema":{"type":"object","properties":{"request_id":{"type":"string","minLength":1,"maxLength":128},"machine_id":{"type":"string","minLength":1,"maxLength":256,"description":"Machine de destination de la mission. Le champ cwd reste le dossier source sur la machine du parent."},"project_id":{"type":"string","minLength":1,"maxLength":256},"engine":{"type":"string","minLength":1,"maxLength":256},"connection_id":{"type":"string","minLength":1,"maxLength":256},"agent_type":{"type":"string","minLength":1,"maxLength":256},"model":{"type":"string","minLength":1,"maxLength":256},"effort":{"type":"string","minLength":1,"maxLength":256},"workload":{"type":"string","enum":["general","compilation"],"default":"general"},"expected_base_commit":{"type":"string","minLength":40,"maxLength":40,"pattern":"^[0-9a-fA-F]{40}$","description":"SHA Git complet du commit source : exactement 40 caractères hexadécimaux (SHA-1), jamais un SHA abrégé ni un nom de branche. En shared : HEAD source ; en worktree : commit résolu de workspace.base_ref (HEAD si absent)."},"task":{"type":"string","minLength":1,"maxLength":65536},"cwd":{"type":"string","minLength":1,"description":"Chemin absolu du dossier source sur la machine du parent appelant. Même avec machine_id distant, ne pas fournir le chemin de destination ; Bridget traduit le chemin via le mapping du projet."},"posture":{"type":"string","enum":["discovery","development"]},"reply_timeout":{"type":"integer","minimum":60,"maximum":86400},"workspace":workspace_schema,"child_requests":{"type":"string","enum":["deny","forward"]},"keep_alive_secs":{"type":"integer","minimum":60,"maximum":7200},"brief":brief_schema,"urgent":{"type":"boolean"},"deliverable":{"type":"string","minLength":1,"maxLength":128}},"required":["request_id","task","cwd"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_repair_workspace","description":"Répare explicitement le dossier déplacé de sa même tâche distante, après résultat remis. Conserver request_id sur rejeu identique. Aucun tour métier créé. Les pairs ou adaptations non qualifiés refusent sans forçage.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1,"maxLength":128},"target_cwd":{"type":"string","minLength":1,"maxLength":4096},"expected_head":{"type":"string","pattern":"^[0-9a-fA-F]{40}$","minLength":40,"maxLength":40}},"required":["task_id","request_id","target_cwd","expected_head"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_continue","description":"Ouvre une nouvelle ronde sur son enfant encore gardé, après la remise du résultat. Employer une nouvelle request_id à chaque ronde et la conserver sur retry identique. Le brief est déclaré par le parent ; sources_not_verified : aucune source citée n'est vérifiée. Le journal peut être lu plus largement que l'enfant ; aucun secret dans le brief. Corps du dossier ≤ 16 Kio, refusé plutôt que tronqué.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1,"maxLength":128},"task":{"type":"string","minLength":1,"maxLength":65536},"brief":brief_schema},"required":["task_id","request_id","task"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_respond","description":"Répond à une question de son enfant avec l'identifiant lu dans le statut ou l'avis. Les approbations appartiennent à l'humain. Aucun secret dans la réponse.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string","minLength":1},"request_id":{"type":"string","minLength":1},"decision":{"type":"string","enum":["accept","decline","cancel"]},"answer":{"type":"object","description":"Pour accept uniquement : chaque identifiant de question porte un tableau de chaînes, 8 Kio au plus."}},"required":["task_id","request_id","decision"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_status","description":"Lit l'état durable et le résultat stable de sa mission native. Une fin de tour ne vaut pas résultat disponible tant que les enfants travaillent.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string"}},"required":["task_id"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_wait","description":"Attend le résultat d'une ou plusieurs de ses tâches natives, sans polling ni modification de la remise. Une expiration n'annule aucun enfant. Le message de résultat peut arriver après ce retour.","inputSchema":{"type":"object","properties":{"task_ids":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"string","minLength":1}},"wait_timeout":{"type":"integer","minimum":1,"maximum":WAIT_TIMEOUT_MAX_SECS,"default":WAIT_TIMEOUT_DEFAULT_SECS}},"required":["task_ids"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_ack","description":"Déclare avoir lu le résultat ou l'échec de ses tâches natives. Cet accusé est informatif ; il ne remplace pas le message normal et ne modifie ni remise, ni nettoyage, ni rétention.","inputSchema":{"type":"object","properties":{"task_ids":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"string","minLength":1}}},"required":["task_ids"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_task_cancel","description":"Annule sa mission native et sa descendance ; aucune mission d'un autre parent n'est accessible.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string"}},"required":["task_id"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true}}),
    ]
}

pub(crate) fn execute(
    identity: &ResolvedIdentity,
    name: &str,
    args: &Value,
    socket: &Path,
) -> Result<Value, ClientError> {
    execute_with_proof(identity, name, args, socket, crate::t3code_mcp::private_proof())
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
    let allowed: &[&str] = match name {
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
        ],
        "bridget_auto_delegate" => &[
            "request_id",
            "machine_id",
            "project_id",
            "engine",
            "connection_id",
            "agent_type",
            "model",
            "effort",
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
        "bridget_task_repair_workspace" => &["task_id", "request_id", "target_cwd", "expected_head"],
        "bridget_task_continue" => &["task_id", "request_id", "task", "brief"],
        "bridget_task_respond" => &["task_id", "request_id", "decision", "answer"],
        "bridget_task_status" | "bridget_task_cancel" => &["task_id"],
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
    let request = match name {
        "bridget_capabilities" => NativeDelegationRequest::Catalogue,
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
            workspace: args
                .get("workspace")
                .map(|value| {
                    serde_json::from_value::<WorkspaceRequest>(value.clone())
                        .map_err(|_| ClientError::InvalidParams("workspace invalide".into()))
                })
                .transpose()?,
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
            agent_type: args
                .get("agent_type")
                .map(|_| string("agent_type"))
                .transpose()?,
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
            workspace: args
                .get("workspace")
                .map(|value| {
                    serde_json::from_value::<WorkspaceRequest>(value.clone())
                        .map_err(|_| ClientError::InvalidParams("workspace invalide".into()))
                })
                .transpose()?,
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
        "bridget_task_wait" => NativeDelegationRequest::Wait {
            task_ids: parse_task_ids(args)?,
            wait_timeout: Some(parse_wait_timeout(object)?),
        },
        "bridget_task_ack" => NativeDelegationRequest::Ack {
            task_ids: parse_task_ids(args)?,
        },
        _ => unreachable!(),
    };
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
    ) && !daemon_supports_wait(socket)?
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
        )?;
        connection.set_deadline(deadline);
        connection
    } else {
        registered_connection(&identity.name, &identity.instance_id, socket)?
    };
    let required_feature = match &request {
        NativeDelegationRequest::RepairWorkspace {..} =>
            Some(("task_workspace_repair_v1","workspace_repair_master_upgrade_required")),

        NativeDelegationRequest::Delegate {
            machine_id: Some(_),
            ..
        } => Some(("remote_worker_v1", "remote_worker_unsupported")),
        NativeDelegationRequest::AutoDelegate(_) => {
            Some(("auto_delegate_v1", "auto_delegate_unsupported"))
        }
        NativeDelegationRequest::AutoDelegateV2(_) => {
            Some(("auto_delegate_v2", "auto_delegate_v2_unsupported"))
        }
        _ => None,
    };
    if let Some((feature, refusal)) = required_feature {
        let supported = match connection.exchange(&WrapperToDaemon::DaemonIdentityRequest)? {
            DaemonToWrapper::DaemonIdentityReport { features, .. } =>
                features.iter().any(|candidate| candidate == feature),
            other => return unexpected_response(other),
        };
        if !supported {
            return Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: refusal.into(),
            });
        }
    }
    // Une admission peut préparer un dépôt ou un worktree. Le délai métier
    // de la mission reste distinct ; les lectures gardent leur budget court.
    let admission = matches!(&request, NativeDelegationRequest::AutoDelegateV2(_)
        | NativeDelegationRequest::AutoDelegate(_) | NativeDelegationRequest::Delegate { .. } | NativeDelegationRequest::RepairWorkspace { .. });
    if admission {
        connection.set_deadline(Instant::now() + Duration::from_secs(600));
    }
    let recover_request_id = match &request {
        NativeDelegationRequest::AutoDelegateV2(auto) => Some(auto.request_id.clone()),
        NativeDelegationRequest::Delegate { request_id, .. } => Some(request_id.clone()),
        _ => None,
    };
    let attestation_budget = if proof.is_some() {
        crate::t3code_mcp::ATTESTATION_MAX_BUDGET
    } else {
        Duration::ZERO
    };
    if !attestation_budget.is_zero() && !admission
    {
        let wait = match &request {
            NativeDelegationRequest::Wait { wait_timeout, .. } =>
                Duration::from_secs(wait_timeout.unwrap_or(WAIT_TIMEOUT_DEFAULT_SECS)),
            _ => Duration::ZERO,
        };
        connection.set_deadline(Instant::now()
            + crate::communication::client::DAEMON_BUDGET + attestation_budget + wait);
    }
    let mutation = !matches!(&request,
        NativeDelegationRequest::Catalogue | NativeDelegationRequest::RoutingCatalogueV1 { .. }
        | NativeDelegationRequest::CallerScope { .. } | NativeDelegationRequest::Status { .. }
        | NativeDelegationRequest::Wait { .. });
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
        ClientError::Technical { code: code @ ("daemon_unreachable" | "outcome_unknown"), message } =>
            ClientError::Technical { code, message: format!("{message} Attendez le retour du lien, puis consultez bridget_task_status avec le même task_id. Sans task_id, conservez request_id et tous les paramètres pour retrouver le sort de l’admission ; ne créez pas une autre demande tant que ce sort reste inconnu.") },
        other => other,
    })?;
    match response {
        DaemonToWrapper::NativeDelegationResult { result } if result["status"] == "refused" => {
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

fn recover_admission_reply171<T>(
    request_id: &str,
    is_pending: impl Fn(&T) -> bool,
    exchange: impl FnMut(Option<Instant>) -> Result<T, ClientError>,
) -> Result<T, ClientError> {
    recover_admission_with_budget171(request_id, is_pending, exchange,
        crate::communication::client::DAEMON_BUDGET)
}

fn recover_admission_with_budget171<T>(
    request_id: &str,
    is_pending: impl Fn(&T) -> bool,
    mut exchange: impl FnMut(Option<Instant>) -> Result<T, ClientError>,
    recovery_budget: Duration,
) -> Result<T, ClientError> {
    let first = exchange(None);
    if !matches!(&first, Err(ClientError::Technical { code: "outcome_unknown", .. }))
        && !first.as_ref().is_ok_and(&is_pending)
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
                message: format!("Admission de la demande {request_id} toujours en cours. Conserver cette clé et les mêmes arguments pour récupérer son résultat ; ne pas créer de doublon."),
            });
        }
        match exchange(Some(deadline)) {
            Ok(value) if is_pending(&value) => {
                std::thread::sleep(Duration::from_millis(250).min(deadline.saturating_duration_since(Instant::now())));
            }
            Err(ClientError::Technical { code: "outcome_unknown" | "daemon_unreachable", .. }) => {
                return Err(ClientError::Technical {
                    code: "outcome_unknown",
                    message: format!("Accusé de la demande {request_id} indisponible après récupération. La mission peut être admise. Conserver cette clé et les mêmes arguments ; ne pas créer une demande en doublon."),
                });
            }
            other => return other,
        }
    }
}

#[cfg(test)]
mod admission_recovery171_tests {
    use super::*;

    #[test]
    fn incident171_pending_admission_recovers_same_receipt_with_one_deadline() {
        let pending = |value: &Value| value["status"] == "refused" && value["code"] == "admission_pending";
        let mut calls = 0;
        let mut recovery_deadline = None;
        let result = recover_admission_reply171("real-key", pending, |deadline| {
            calls += 1;
            if calls == 1 {
                assert!(deadline.is_none());
                return Err(ClientError::Technical { code: "outcome_unknown", message: "lost reply".into() });
            }
            let observed = deadline.unwrap();
            if let Some(previous) = recovery_deadline { assert_eq!(observed, previous); }
            recovery_deadline = Some(observed);
            Ok(if calls < 4 { json!({"status":"refused","code":"admission_pending"}) }
                else { json!({"task_id":"same-task","status":"starting"}) })
        }).unwrap();
        assert_eq!(calls, 4);
        assert_eq!(result["task_id"], "same-task");
        let mut calls = 0;
        let refusal = json!({"status":"refused","code":"permission_not_inherited"});
        assert_eq!(recover_admission_reply171("real-key", pending, |_| {
            calls += 1; Ok(refusal.clone())
        }).unwrap(), refusal);
        assert_eq!(calls, 1);
        let mut calls = 0;
        let result = recover_admission_with_budget171("real-key", pending, |_| {
            calls += 1; Ok(json!({"status":"refused","code":"admission_pending"}))
        }, Duration::ZERO);
        assert_eq!(calls, 1);
        assert!(matches!(result, Err(ClientError::Technical { code: "outcome_unknown", message })
            if message.contains("real-key") && message.contains("doublon")));
    }

    #[test]
    fn delivery171_lost_admission_reply_is_recovered_once_without_masking_refusals() {
        let mut calls = Vec::new();
        let result = recover_admission_reply171("same-key", |_| false, |deadline| {
            calls.push(deadline.is_some());
            if deadline.is_some() { Ok(json!({"task_id":"same-task","status":"starting"})) }
            else { Err(ClientError::Technical { code:"outcome_unknown", message:"lecture interrompue".into() }) }
        }).unwrap();
        assert_eq!(calls, vec![false,true]);
        assert_eq!(result["task_id"], "same-task");
        let mut calls=0;
        let result: Result<(),_> = recover_admission_reply171("same-key", |_| false, |_| {
            calls+=1;
            Err(ClientError::Technical { code:"outcome_unknown", message:"délai".into() })
        });
        assert_eq!(calls,2);
        assert!(matches!(result,Err(ClientError::Technical {code:"outcome_unknown",message}) if message.contains("same-key") && message.contains("doublon")));
        let mut calls=0;
        let result: Result<(),_> = recover_admission_reply171("same-key", |_| false, |_| {
            calls+=1;
            Err(ClientError::Technical { code:"native_delegation_refused", message:"permission_not_inherited".into() })
        });
        assert_eq!(calls,1);
        assert!(matches!(result,Err(ClientError::Technical {code:"native_delegation_refused",..})));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec174_source_arguments_expliques_et_sha_valide_avant_connexion() {
        let catalogue = tools(&json!({}));
        for name in ["bridget_delegate", "bridget_auto_delegate"] {
            let tool = catalogue.iter().find(|tool| tool["name"] == name).unwrap();
            let properties = &tool["inputSchema"]["properties"];
            assert!(properties["cwd"]["description"]
                .as_str()
                .unwrap()
                .contains("machine du parent"));
            assert!(properties["machine_id"]["description"]
                .as_str()
                .unwrap()
                .contains("destination"));
        }
        let automatic = catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_auto_delegate")
            .unwrap();
        assert_eq!(
            automatic["inputSchema"]["properties"]["expected_base_commit"]["pattern"],
            "^[0-9a-fA-F]{40}$"
        );
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for invalid in [
            "09d8fa7e9".into(),
            "a".repeat(41),
            "a".repeat(63),
            "F".repeat(64),
            format!("{}\n", "a".repeat(40)),
            "g".repeat(40),
            "feature/private-ref".into(),
        ] {
            let result = execute_with_proof(
                &identity,
                "bridget_auto_delegate",
                &json!({"request_id":"source-sha","task":"lecture","cwd":"/source",
                    "expected_base_commit":invalid}),
                Path::new("/missing/source-sha.sock"),
                None,
            );
            match result {
                Err(ClientError::InvalidParams(message)) => {
                    assert!(message.contains("40 caractères"));
                    assert!(message.contains("SHA abrégé"));
                    assert!(
                        !message.contains(&invalid),
                        "ne pas répéter la valeur fournie"
                    );
                }
                other => panic!("refus de forme attendu avant connexion : {other:?}"),
            }
        }
        for valid in ["a".repeat(40), "F".repeat(40)] {
            let result = execute_with_proof(
                &identity,
                "bridget_auto_delegate",
                &json!({"request_id":"source-sha","task":"lecture","cwd":"/source",
                    "expected_base_commit":valid}),
                Path::new("/missing/source-sha.sock"),
                None,
            );
            assert!(matches!(
                result,
                Err(ClientError::Technical {
                    code: "daemon_unreachable",
                    ..
                })
            ));
        }
    }

    #[test]
    fn spec168_wire163_byte_compatible_et_nouveau_client_refuse_ancien_maitre() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        let legacy: NativeDelegationRequest = serde_json::from_value(json!({
            "operation":"auto_delegate","request_id":"legacy163","task":"mission","cwd":"/tmp"}))
        .unwrap();
        assert_eq!(
            serde_json::to_string(&legacy).unwrap(),
            r#"{"operation":"auto_delegate","request_id":"legacy163","task":"mission","cwd":"/tmp"}"#
        );
        let socket = crate::mcp_identity::mock_socket("legacy168-refusal");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
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
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture",
                "instance_id":"fixture","features":["auto_delegate_v1"]})
            )
            .unwrap();
            line.clear();
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "aucun envoi V2 à un ancien maître"
            );
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"new168","task":"mission","cwd":"/tmp"}),
            &socket,
        );
        assert!(
            matches!(result,Err(ClientError::Technical{message,..}) if message=="auto_delegate_v2_unsupported")
        );
        backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn spec168_mcp_auto_delegate_negocie_v2_et_preserve_le_schema_ferme() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixListener;
        let tool = tools(&json!({}))
            .into_iter()
            .find(|tool| tool["name"] == "bridget_auto_delegate")
            .unwrap();
        assert_eq!(tool["inputSchema"]["additionalProperties"], false);
        assert_eq!(
            tool["inputSchema"]["required"],
            json!(["request_id", "task", "cwd"])
        );
        assert!(tool["inputSchema"]["properties"].get("fallback").is_none());

        let socket = crate::mcp_identity::mock_socket("auto163-mcp");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let mut pollfd = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            assert!(unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0);
            let (mut stream, _) = listener.accept().unwrap();
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
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            writeln!(stream, "{}", json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["auto_delegate_v2"]})).unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            writeln!(stream, "{}", json!({"type":"NativeDelegationResult","result":{"status":"accepted","task_id":"task-163"}})).unwrap();
            request
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"auto-163","task":"mission","cwd":"/tmp",
                "brief":{"objective":"Choisir","summary":"Candidats bornés"},"urgent":true}),
            &socket,
        )
        .unwrap();
        assert_eq!(result["task_id"], "task-163");
        let request = backend.join().unwrap();
        assert_eq!(request["type"], "NativeDelegation");
        assert_eq!(request["request"]["operation"], "auto_delegate_v2");
        assert_eq!(request["request"]["urgent"], true);
        assert!(request["request"].get("fallback").is_none());
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn native157_fallback_mcp_schema_ferme() {
        let delegate = crate::mcp::tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_delegate")
            .unwrap();
        let fallback = &delegate["inputSchema"]["properties"]["fallback"];
        assert_eq!(fallback["type"], "array");
        assert_eq!(fallback["minItems"], 1);
        assert_eq!(fallback["maxItems"], 3);
        assert_eq!(fallback["items"]["type"], "object");
        assert_eq!(fallback["items"]["additionalProperties"], false);
        assert_eq!(
            fallback["items"]["required"],
            json!(["agent_type", "model"])
        );
        for field in ["agent_type", "model"] {
            assert_eq!(fallback["items"]["properties"][field]["type"], "string");
            assert_eq!(fallback["items"]["properties"][field]["minLength"], 1);
        }
        assert_eq!(fallback["items"]["properties"]["effort"]["type"], "string");
        assert_eq!(
            fallback["items"]["properties"].as_object().unwrap().len(),
            3
        );
        assert!(
            !delegate["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|key| key == "fallback")
        );
    }

    #[test]
    fn native157_fallback_mcp_formes_et_canonique() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixListener;
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let base = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
        let socket = crate::mcp_identity::mock_socket("fallback157-invalid");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        for fallback in [
            json!(null),
            json!(false),
            json!("b"),
            json!({"agent_type":"b","model":"n"}),
            json!([]),
            json!(vec![json!({"agent_type":"b","model":"n"}); 4]),
            json!([null]),
            json!([{}]),
            json!([{"agent_type":"b"}]),
            json!([{"model":"n"}]),
            json!([{"agent_type":"","model":"n"}]),
            json!([{"agent_type":"b","model":""}]),
            json!([{"agent_type":2,"model":"n"}]),
            json!([{"agent_type":"b","model":2}]),
            json!([{"agent_type":"b","model":"n","effort":null}]),
            json!([{"agent_type":"b","model":"n","effort":2}]),
            json!([{"agent_type":"b","model":"n","permissions":"allow"}]),
            json!([{"agent_type":"b","model":"n"},{"agent_type":"b","model":"n"}]),
            json!([{"agent_type":"a","model":"m"}]),
        ] {
            let mut args = base.clone();
            args["fallback"] = fallback;
            let result = execute(&identity, "bridget_delegate", &args, &socket);
            assert!(
                matches!(result, Err(ClientError::InvalidParams(_))),
                "{args}: {result:?}"
            );
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "une forme invalide a ouvert la socket"
            );
        }
        drop(listener);
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());

        // Réemploi du faux daemon MCP : absence byte-identique et liste bornée
        // transmise telle quelle, sans appeler de fournisseur.
        for (fallback, resume_on_limit) in [
            (None, None),
            (Some(json!([{"agent_type":"b","model":"n"}])), None),
            (
                Some(json!([{"agent_type":"b","model":"n"},
                {"agent_type":"c","model":"p","effort":"high"},
                {"agent_type":"d","model":"q"}])),
                None,
            ),
            (None, Some(false)),
            (None, Some(true)),
        ] {
            let socket = crate::mcp_identity::mock_socket("fallback157-valid");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let backend = std::thread::spawn(move || {
                let mut pollfd = libc::pollfd {
                    fd: listener.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                if unsafe { libc::poll(&mut pollfd, 1, 3000) } <= 0 {
                    return None;
                }
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let registration: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(registration["type"], "RegisterAuxiliary");
                writeln!(
                    stream,
                    "{}",
                    json!({"type":"Registered","agent_id":"owner","credential":null})
                )
                .unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let frame: WrapperToDaemon = serde_json::from_str(&line).unwrap();
                let WrapperToDaemon::NativeDelegation { request } = frame else {
                    panic!("trame native attendue : {frame:?}")
                };
                writeln!(
                    stream,
                    "{}",
                    json!({"type":"NativeDelegationResult","result":{"status":"accepted"}})
                )
                .unwrap();
                Some(request)
            });
            let mut args = base.clone();
            if let Some(value) = &fallback {
                args["fallback"] = value.clone();
            }
            if let Some(value) = resume_on_limit {
                args["resume_on_limit"] = json!(value);
            }
            let result = execute(&identity, "bridget_delegate", &args, &socket);
            let request = backend.join().unwrap();
            let _ = std::fs::remove_dir_all(socket.parent().unwrap());
            assert!(
                result.is_ok(),
                "une liste valide doit atteindre le daemon : {args}: {result:?}"
            );
            let request = request.expect("requête transmise");
            if let Some(value) = fallback {
                assert_eq!(serde_json::to_value(request).unwrap()["fallback"], value);
            } else if let Some(expected) = resume_on_limit {
                let NativeDelegationRequest::Delegate {
                    resume_on_limit, ..
                } = request
                else {
                    panic!("Delegate attendu")
                };
                assert_eq!(resume_on_limit, Some(expected));
            } else {
                let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
                assert_eq!(serde_json::to_vec(&request).unwrap(), historical);
            }
        }
    }

    #[test]
    fn native154_mcp_workspace_schema_ferme_et_champs_interdits() {
        let delegate = crate::mcp::tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_delegate")
            .unwrap();
        let workspace = &delegate["inputSchema"]["properties"]["workspace"];
        let variants = workspace["oneOf"].as_array().unwrap();
        assert_eq!(variants.len(), 2);
        for variant in variants {
            assert_eq!(variant["type"], "object");
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["required"], json!(["strategy"]));
            for forbidden in ["start_from_origin", "push", "merge", "cleanup"] {
                assert!(variant["properties"].get(forbidden).is_none());
            }
        }
        assert_eq!(variants[0]["properties"]["strategy"]["const"], "shared");
        assert_eq!(variants[1]["properties"]["strategy"]["const"], "worktree");
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let base = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
        for forbidden in ["start_from_origin", "push", "merge", "cleanup"] {
            let mut args = base.clone();
            args["workspace"] = json!({"strategy":"worktree"});
            args["workspace"][forbidden] = json!(true);
            assert!(matches!(
                execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
                Err(ClientError::InvalidParams(_))
            ));
            args = base.clone();
            args[forbidden] = json!(true);
            assert!(matches!(
                execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
                Err(ClientError::InvalidParams(_))
            ));
        }
        for valid in [
            json!({"strategy":"shared"}),
            json!({"strategy":"worktree","base_ref":"main","branch":"bridget/task"}),
        ] {
            let mut args = base.clone();
            args["workspace"] = valid;
            assert!(!matches!(
                execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
                Err(ClientError::InvalidParams(_))
            ));
        }
    }

    #[test]
    fn native151_facade_task_wait_schema_ferme_et_lecture_pure() {
        let wait = crate::mcp::tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_task_wait")
            .expect("outil d'attente présent");
        assert_eq!(wait["inputSchema"]["additionalProperties"], false);
        assert_eq!(wait["inputSchema"]["required"], json!(["task_ids"]));
        assert_eq!(wait["inputSchema"]["properties"]["task_ids"]["minItems"], 1);
        assert_eq!(
            wait["inputSchema"]["properties"]["task_ids"]["maxItems"],
            16
        );
        assert_eq!(
            wait["inputSchema"]["properties"]["task_ids"]["uniqueItems"],
            true
        );
        assert_eq!(
            wait["inputSchema"]["properties"]["wait_timeout"]["default"],
            600
        );
        assert_eq!(
            wait["inputSchema"]["properties"]["wait_timeout"]["maximum"],
            3600
        );
        assert_eq!(wait["annotations"]["readOnlyHint"], true);
        assert_eq!(wait["annotations"]["destructiveHint"], false);
    }

    #[test]
    fn native151_facade_task_wait_bornes_et_types_avant_connexion() {
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for value in [json!(-1), json!(0), json!(3601), json!(u64::MAX)] {
            let refusal = execute(
                &identity,
                "bridget_task_wait",
                &json!({"task_ids":["task"],"wait_timeout":value}),
                Path::new("/missing"),
            );
            assert!(
                matches!(refusal, Err(ClientError::Technical { code: "native_delegation_refused", message }) if message == "wait_timeout_out_of_range")
            );
        }
        for value in [json!(null), json!(1.5), json!("1"), json!(true)] {
            assert!(matches!(
                execute(
                    &identity,
                    "bridget_task_wait",
                    &json!({"task_ids":["task"],"wait_timeout":value}),
                    Path::new("/missing")
                ),
                Err(ClientError::InvalidParams(_))
            ));
        }
        for args in [
            json!({"task_ids":["task"],"parent":"forged"}),
            json!({"task_ids":[]}),
            json!({"task_ids":["task","task"]}),
            json!({"task_ids":vec!["task";17]}),
            json!({"task_ids":[""]}),
            json!({"task_ids":[1]}),
        ] {
            assert!(matches!(
                execute(&identity, "bridget_task_wait", &args, Path::new("/missing")),
                Err(ClientError::InvalidParams(_))
            ));
        }
        for value in [None, Some(json!(1)), Some(json!(3600))] {
            let mut args = json!({"task_ids":["task"]});
            if let Some(value) = value {
                args["wait_timeout"] = value;
            }
            assert!(
                matches!(execute(&identity, "bridget_task_wait", &args, Path::new("/missing")), Err(ClientError::Technical { code: "daemon_unreachable", .. }))
            );
        }
    }

    #[test]
    fn closed_arguments_reject_forged_parent_without_connecting() {
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        assert!(matches!(
            execute(
                &identity,
                "bridget_delegate",
                &json!({"parent":"other"}),
                Path::new("/missing")
            ),
            Err(ClientError::InvalidParams(_))
        ));
        for invalid in [
            json!({"request_id":"auto-task","task":"x".repeat(65_537),"cwd":"/tmp"}),
            json!({"request_id":"auto-canon","task":"t","cwd":"/tmp",
                "brief":{"objective":"x".repeat(512 * 1024)}}),
            json!({"request_id":"auto-parent","task":"t","cwd":"/tmp","parent":"other"}),
        ] {
            assert!(matches!(
                execute(
                    &identity,
                    "bridget_auto_delegate",
                    &invalid,
                    Path::new("/missing")
                ),
                Err(ClientError::InvalidParams(_))
            ));
        }
        let catalogue = crate::mcp::tools();
        let brief_schema = &catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_handoff")
            .unwrap()["inputSchema"]["properties"]["draft"];
        assert_eq!(tools(brief_schema).len(), 9);
    }

    #[test]
    fn native150_delai_schema_liste_blanche_et_forme() {
        let delegate = crate::mcp::tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_delegate")
            .unwrap();
        let property = &delegate["inputSchema"]["properties"]["reply_timeout"];
        assert_eq!(property["type"], "integer");
        assert_eq!(
            (property["minimum"].as_u64(), property["maximum"].as_u64()),
            (Some(60), Some(86_400))
        );
        assert!(
            !delegate["inputSchema"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|key| key == "reply_timeout")
        );
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let base = |extra: Value| {
            let mut args =
                json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
            args.as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            args
        };
        // Forme invalide : refusée avant toute connexion. Les bornes restent au daemon.
        for invalide in [json!("abc"), json!(-5), json!(60.5), json!(true)] {
            assert!(matches!(
                execute(
                    &identity,
                    "bridget_delegate",
                    &base(json!({"reply_timeout": invalide})),
                    Path::new("/missing")
                ),
                Err(ClientError::InvalidParams(_))
            ));
        }
        // Forme valide : le champ passe la liste blanche, l'échec vient de la connexion.
        for valide in [json!(120), json!(null)] {
            assert!(!matches!(
                execute(
                    &identity,
                    "bridget_delegate",
                    &base(json!({"reply_timeout": valide})),
                    Path::new("/missing")
                ),
                Err(ClientError::InvalidParams(_))
            ));
        }
    }

    #[test]
    fn native157_quota_detail_visible_cote_mcp() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixListener;
        for (returned, expected_code, expected_detail) in [
            (
                json!({"status":"accepted","features":{"native_delegation":true}}),
                "catalogue160",
                None,
            ),
            (
                json!({"status":"refused","code":"provider_quota_exhausted","detail":{"resets_at":1_900_000_000}}),
                "provider_quota_exhausted",
                Some(json!({"resets_at":1_900_000_000})),
            ),
            (
                json!({"status":"refused","code":"remote_execution_ended","message":"La commande distante est terminée ou son lancement a été refusé.","detail":{"message":"La commande distante est terminée ou son lancement a été refusé."}}),
                "remote_execution_ended",
                Some(json!({"message":"La commande distante est terminée ou son lancement a été refusé."})),
            ),
            (
                json!({"status":"refused","code":"remote_execution_unconfirmed","message":"Vérifiez l’état de la tâche sans créer de doublon.","detail":{"message":"Vérifiez l’état de la tâche sans créer de doublon."}}),
                "remote_execution_unconfirmed",
                Some(json!({"message":"Vérifiez l’état de la tâche sans créer de doublon."})),
            ),
            (
                json!({"status":"refused","code":"workspace_preflight_technical","detail":{"message":"Le contrôle préalable du dossier source a refusé la demande.","recovery":"Le champ cwd doit désigner le dossier source sur la machine du parent qui appelle Bridget, pas le dossier de destination. Vérifiez que ce dossier existe, que Git peut le lire et que la référence demandée est disponible. Avant un nouvel essai, vérifiez si une tâche existe déjà pour cette demande ; ne créez pas de doublon et ne changez pas la base automatiquement."}}),
                "workspace_preflight_technical",
                Some(json!({"message":"Le contrôle préalable du dossier source a refusé la demande.","recovery":"Le champ cwd doit désigner le dossier source sur la machine du parent qui appelle Bridget, pas le dossier de destination. Vérifiez que ce dossier existe, que Git peut le lire et que la référence demandée est disponible. Avant un nouvel essai, vérifiez si une tâche existe déjà pour cette demande ; ne créez pas de doublon et ne changez pas la base automatiquement."})),
            ),
            (
                json!({"status":"refused","code":"workspace_base_preflight_failed","detail":{"message":"Bridget n’a pas pu contrôler le dossier source avant l’admission.","recovery":"Le champ cwd désigne un dossier sur la machine du parent appelant. machine_id choisit seulement la destination : vérifier le chemin par SSH sur cette destination ne valide pas la source. Sur la machine du parent, vérifiez que cwd existe et est lisible, puis exécutez git rev-parse --show-toplevel et git rev-parse HEAD dans ce dossier. Corrigez le chemin ou l’accès Git en cas d’échec. En shared, le HEAD source doit être égal à expected_base_commit. Ne changez pas le SHA attendu et ne déplacez pas un checkout utilisé pour contourner le refus. Si shared est imposé, conservez ce mode ; sinon, pour une base figée en worktree autorisé, donnez le même SHA complet à workspace.base_ref et expected_base_commit. Ce refus technique ne clôt pas une admission déjà en cours et ne prouve pas l’absence de tâche. Suivez toute tâche connue avec bridget_task_status. Un rejeu strict conserve request_id et tous les paramètres : admission_pending signifie que la demande reste en attente. Avant une nouvelle clé pour des paramètres corrigés, confirmez qu’aucune tâche et aucune admission en cours ne subsistent pour l’ancienne demande. Si le sort reste inconnu, ne créez pas de nouvelle demande."}}),
                "workspace_base_preflight_failed",
                Some(json!({"message":"Bridget n’a pas pu contrôler le dossier source avant l’admission.","recovery":"Le champ cwd désigne un dossier sur la machine du parent appelant. machine_id choisit seulement la destination : vérifier le chemin par SSH sur cette destination ne valide pas la source. Sur la machine du parent, vérifiez que cwd existe et est lisible, puis exécutez git rev-parse --show-toplevel et git rev-parse HEAD dans ce dossier. Corrigez le chemin ou l’accès Git en cas d’échec. En shared, le HEAD source doit être égal à expected_base_commit. Ne changez pas le SHA attendu et ne déplacez pas un checkout utilisé pour contourner le refus. Si shared est imposé, conservez ce mode ; sinon, pour une base figée en worktree autorisé, donnez le même SHA complet à workspace.base_ref et expected_base_commit. Ce refus technique ne clôt pas une admission déjà en cours et ne prouve pas l’absence de tâche. Suivez toute tâche connue avec bridget_task_status. Un rejeu strict conserve request_id et tous les paramètres : admission_pending signifie que la demande reste en attente. Avant une nouvelle clé pour des paramètres corrigés, confirmez qu’aucune tâche et aucune admission en cours ne subsistent pour l’ancienne demande. Si le sort reste inconnu, ne créez pas de nouvelle demande."})),
            ),
            (
                json!({"status":"refused","code":"expected_base_commit_mismatch","detail":{"message":"La base résolue ne correspond pas au commit exact demandé.","recovery":"Pour une revue figée, utiliser le même SHA complet dans workspace.base_ref et expected_base_commit."}}),
                "expected_base_commit_mismatch",
                Some(json!({"message":"La base résolue ne correspond pas au commit exact demandé.","recovery":"Pour une revue figée, utiliser le même SHA complet dans workspace.base_ref et expected_base_commit."})),
            ),
            (
                json!({"status":"refused","code":"permission_source_unavailable"}),
                "permission_source_unavailable",
                None,
            ),
            (
                json!({"status":"refused","code":"workspace_base_unavailable","message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","detail":{"message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","recovery":"Dans cwd, sur la machine du parent, vérifiez que Git peut résoudre le commit demandé. Sa présence sur la machine de destination ne suffit pas. Si le commit manque, identifiez le dépôt distant et la branche qui le contiennent, puis récupérez cette branche avec git fetch dans le dépôt source. Vérifiez à nouveau le SHA exact, sans changer expected_base_commit, faire de reset ou avancer la branche de travail. Si une tâche existe, suivez-la avec bridget_task_status. Sinon, après correction, faites au plus un rejeu strict avec la même request_id et les mêmes paramètres. Un refus enregistré reste un refus au rejeu : si ce refus durable et l’absence de tâche sont confirmés, utilisez une nouvelle request_id après correction. Ne multipliez pas les essais si le sort reste inconnu. Ce code ne désigne pas le plafond des worktrees."}}),
                "workspace_base_unavailable",
                Some(json!({"message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","recovery":"Dans cwd, sur la machine du parent, vérifiez que Git peut résoudre le commit demandé. Sa présence sur la machine de destination ne suffit pas. Si le commit manque, identifiez le dépôt distant et la branche qui le contiennent, puis récupérez cette branche avec git fetch dans le dépôt source. Vérifiez à nouveau le SHA exact, sans changer expected_base_commit, faire de reset ou avancer la branche de travail. Si une tâche existe, suivez-la avec bridget_task_status. Sinon, après correction, faites au plus un rejeu strict avec la même request_id et les mêmes paramètres. Un refus enregistré reste un refus au rejeu : si ce refus durable et l’absence de tâche sont confirmés, utilisez une nouvelle request_id après correction. Ne multipliez pas les essais si le sort reste inconnu. Ce code ne désigne pas le plafond des worktrees."})),
            ),
        ] {
            let socket = crate::mcp_identity::mock_socket("quota157");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let backend = std::thread::spawn(move || {
                let mut pollfd = libc::pollfd {
                    fd: listener.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                assert!(
                    unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0,
                    "connexion MCP attendue"
                );
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let registration: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(registration["type"], "RegisterAuxiliary");
                assert_eq!(registration["agent_id"], "owner");
                writeln!(
                    stream,
                    "{}",
                    json!({"type":"Registered","agent_id":"owner","credential":null})
                )
                .unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(request["type"], "NativeDelegation");
                writeln!(
                    stream,
                    "{}",
                    json!({"type":"NativeDelegationResult","result":returned})
                )
                .unwrap();
                request
            });
            let identity = ResolvedIdentity {
                name: "owner".into(),
                instance_id: "instance".into(),
                delegated_origin: None,
            };
            let args = json!({"request_id":"quota157","agent_type":"codex","model":"fixture","task":"fixture","cwd":socket.parent().unwrap()});
            let catalogue = expected_code == "catalogue160";
            let catalogue_args = json!({});
            let result = execute(
                &identity,
                if catalogue {
                    "bridget_capabilities"
                } else {
                    "bridget_delegate"
                },
                if catalogue { &catalogue_args } else { &args },
                &socket,
            );
            let request = backend.join().unwrap();
            if catalogue {
                assert_eq!(request["request"]["operation"], "catalogue");
                let result = result.unwrap();
                assert_eq!(
                    result["features"],
                    json!({"native_delegation":true,
                    "resume_on_limit":true,"resume_max_wait_secs":21600,"resume_max_rounds":2})
                );
                let _ = std::fs::remove_dir_all(socket.parent().unwrap());
                continue;
            }
            assert_eq!(request["request"]["agent_type"], "codex");
            let Err(ClientError::Technical { code, message }) = result else {
                panic!("refus technique MCP attendu : {result:?}")
            };
            assert_eq!(code, "native_delegation_refused");
            if let Some(detail) = expected_detail {
                let (actual_code, raw_detail) = message
                    .split_once(' ')
                    .expect("code suivi du JSON de detail");
                assert_eq!(actual_code, expected_code);
                assert_eq!(serde_json::from_str::<Value>(raw_detail).unwrap(), detail);
            } else {
                assert_eq!(
                    message, expected_code,
                    "les autres refus gardent le message historique"
                );
            }
            let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        }
    }

    #[test]
    fn native160_optin_sans_champ_octets_identiques() {
        use std::os::unix::net::UnixListener;
        let tool = tools(&json!({}))
            .into_iter()
            .find(|tool| tool["name"] == "bridget_delegate")
            .unwrap();
        let schema = &tool["inputSchema"];
        assert_eq!(schema["properties"]["resume_on_limit"]["type"], "boolean");
        assert!(
            !schema["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field == "resume_on_limit")
        );
        assert_eq!(
            schema["properties"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "agent_type",
                "brief",
                "child_requests",
                "cwd",
                "effort",
                "fallback",
                "keep_alive_secs",
                "machine_id",
                "model",
                "posture",
                "project_id",
                "reply_timeout",
                "request_id",
                "resume_on_limit",
                "task",
                "workspace"
            ]
        );
        let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
        let request: NativeDelegationRequest = serde_json::from_slice(historical).unwrap();
        assert_eq!(serde_json::to_vec(&request).unwrap(), historical);
        let socket = crate::mcp_identity::mock_socket("resume160-invalid");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for value in [
            json!(null),
            json!(0),
            json!(1),
            json!(0.5),
            json!("true"),
            json!([]),
            json!({}),
        ] {
            let args = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp","resume_on_limit":value});
            assert!(
                matches!(
                    execute(&identity, "bridget_delegate", &args, &socket),
                    Err(ClientError::InvalidParams(_))
                ),
                "{args}"
            );
            assert!(
                matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
            );
        }
        drop(listener);
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn native160_optin_absent_et_faux_deux_enveloppes() {
        let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
        let absent: NativeDelegationRequest = serde_json::from_slice(historical).unwrap();
        let mut explicit_false = absent.clone();
        let NativeDelegationRequest::Delegate {
            resume_on_limit, ..
        } = &mut explicit_false
        else {
            panic!("Delegate attendu")
        };
        *resume_on_limit = Some(false);
        let absent_canonical = serde_json::to_vec(&absent).unwrap();
        let false_canonical = serde_json::to_vec(&explicit_false).unwrap();
        assert_ne!(absent_canonical, false_canonical);
        let store = crate::delegation::DelegationStore::open(Path::new(":memory:")).unwrap();
        store
            .record_refusal("owner", "k", &absent_canonical, "model_unavailable", 100)
            .unwrap();
        assert_eq!(
            store
                .by_agent_request("owner", "k", &false_canonical)
                .unwrap_err(),
            "envelope_mismatch"
        );
        assert_eq!(
            store
                .by_agent_request("owner", "k", &absent_canonical)
                .unwrap_err(),
            "model_unavailable"
        );
    }
}

#[cfg(test)]
mod spec172_transport_tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;

    fn attested_response(name: &'static str, disconnect: bool) {
        let wait = name == "bridget_task_wait";
        let mutation = matches!(name, "bridget_task_cancel" | "bridget_task_ack");
        let socket = crate::mcp_identity::mock_socket("172-attested-deadline");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            if wait || name == "bridget_task_ack" {
                let (mut stream, _) = listener.accept().unwrap();
                let mut line = String::new();
                BufReader::new(stream.try_clone().unwrap())
                    .read_line(&mut line)
                    .unwrap();
                assert_eq!(
                    serde_json::from_str::<Value>(&line).unwrap()["type"],
                    "DaemonIdentityRequest"
                );
                writeln!(stream, "{}", json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["wait_v1"]})).unwrap();
            }
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
            assert_eq!(request["type"], "NativeDelegationT3");
            assert_eq!(
                request["request"]["operation"],
                name.strip_prefix("bridget_task_").unwrap()
            );
            // Trois échanges HTTP peuvent dépasser 10 s sans dépasser leur budget individuel.
            if !disconnect {
                std::thread::sleep(Duration::from_secs(12));
            }
            let result = if wait {
                json!({"status":"refused","code":"permission_attestation_unavailable"})
            } else {
                json!({"task_id":"task","status":"result_available","result":"résultat".repeat(2048)})
            };
            let sent = writeln!(
                stream,
                "{}",
                if disconnect { json!({"type":"Disconnect"}) }
                else { json!({"type":"NativeDelegationResult","result":result}) }
            );
            line.clear();
            let remaining = reader.read_line(&mut line).unwrap();
            assert_eq!(
                remaining, 0,
                "aucun rejeu, changement de clé ou deuxième lancement"
            );
            sent.is_ok()
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let proof = bridget_transport::protocol::NativePermissionProof {
            endpoint: "http://127.0.0.1:12345/mcp".into(),
            authorization: bridget_transport::protocol::IdentityCredential::new(format!(
                "Bearer {}",
                "a".repeat(43)
            )),
        };
        let result = execute_with_proof(
            &identity,
            name,
            &if wait {
                json!({"task_ids":["task"],"wait_timeout":1})
            } else if name == "bridget_task_ack" {
                json!({"task_ids":["task"]})
            } else {
                json!({"task_id":"task"})
            },
            &socket,
            Some(proof),
        );
        let sent = backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        if disconnect {
            let expected = if mutation { "outcome_unknown" } else { "daemon_unreachable" };
            assert!(matches!(result, Err(ClientError::Technical { code, message })
                if code == expected && message.contains("bridget_task_status") && message.contains("même task_id")
                    && message.contains("ne créez pas une autre demande")));
        } else if wait {
            assert!(
                matches!(result, Err(ClientError::Technical { code:"native_delegation_refused", message }) if message=="permission_attestation_unavailable"),
                "le refus doit rester explicite après la réattestation"
            );
        } else {
            let result = result.expect("le client ne doit pas expirer avant la réattestation");
            assert_eq!(result["task_id"], "task");
            assert_eq!(
                result["result"].as_str().unwrap().len(),
                "résultat".repeat(2048).len()
            );
        }
        assert!(sent, "la réponse complète doit être consommée");
    }

    #[test]
    fn spec172_status_atteste_attend_la_reponse_complete() {
        attested_response("bridget_task_status", false);
    }

    #[test]
    fn spec172_wait_atteste_preserve_le_refus_tardif() {
        attested_response("bridget_task_wait", false);
    }

    #[test]
    fn spec174_disconnect_attente_et_statut_sans_rejeu() {
        attested_response("bridget_task_wait", true);
        attested_response("bridget_task_status", true);
    }

    #[test]
    fn spec174_disconnect_mutations_sans_rejeu() {
        attested_response("bridget_task_cancel", true);
        attested_response("bridget_task_ack", true);
    }

    #[test]
    fn spec172_reprise_attestee_garde_la_cle_et_le_budget() {
        let socket = crate::mcp_identity::mock_socket("172-attested-recovery");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let mut first = None;
            for index in 0..2 {
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
                assert_eq!(request["type"], "NativeDelegationT3");
                assert_eq!(request["request"]["request_id"], "same-key-172");
                if index == 0 {
                    first = Some(request);
                } else {
                    assert_eq!(
                        Some(request),
                        first,
                        "même enveloppe, même preuve, même clé"
                    );
                }
                if index == 1 {
                    std::thread::sleep(Duration::from_secs(12));
                }
                let result = if index == 0 {
                    json!({"status":"refused","code":"admission_pending"})
                } else {
                    json!({"task_id":"same-task","status":"starting"})
                };
                let _ = writeln!(
                    stream,
                    "{}",
                    json!({"type":"NativeDelegationResult","result":result})
                );
            }
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let proof = bridget_transport::protocol::NativePermissionProof {
            endpoint: "http://127.0.0.1:12345/mcp".into(),
            authorization: bridget_transport::protocol::IdentityCredential::new(format!(
                "Bearer {}",
                "a".repeat(43)
            )),
        };
        let result = execute_with_proof(
            &identity,
            "bridget_delegate",
            &json!({"request_id":"same-key-172","agent_type":"codex","model":"fixture","task":"mission","cwd":"/tmp"}),
            &socket,
            Some(proof),
        );
        backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        assert_eq!(
            result.expect("la reprise doit aussi couvrir la réattestation")["task_id"],
            "same-task"
        );
    }

    #[test]
    fn spec172_sonde_admission_en_panne_reste_transport() {
        let socket = crate::mcp_identity::mock_socket("172-admission-probe");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            // Fermer sans annonce de capacité, avant toute demande de lancement.
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute_with_proof(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"not-admitted","task":"mission","cwd":"/tmp"}),
            &socket,
            None,
        );
        backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        assert!(matches!(
            result,
            Err(ClientError::Technical {
                code: "daemon_unreachable",
                ..
            })
        ));
    }

    #[test]
    fn spec172_sonde_en_panne_ne_devient_pas_unsupported() {
        let socket = crate::mcp_identity::mock_socket("172-missing-probe");
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for (name, args) in [
            ("bridget_task_wait", json!({"task_ids":["task"]})),
            ("bridget_task_ack", json!({"task_ids":["task"]})),
        ] {
            let result = execute_with_proof(&identity, name, &args, &socket, None);
            assert!(
                matches!(
                    result,
                    Err(ClientError::Technical {
                        code: "daemon_unreachable",
                        ..
                    })
                ),
                "une panne de transport ne prouve pas l'absence de capacité"
            );
        }
    }
}
