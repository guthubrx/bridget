//! Même façade native pour les agents T3, CLI et MCP externes.
use crate::communication::client::{
    ClientError, daemon_supports_wait, registered_connection, registered_connection_until,
    unexpected_response,
};
use crate::mcp_identity::ResolvedIdentity;
use bridget_transport::protocol::{
    ChildRequestDecision, FallbackTarget, NativeDelegationRequest, SpawnPosture, WorkspaceRequest,
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
        {"type":"object","properties":{"strategy":{"const":"worktree"},"base_ref":{"type":"string"},"branch":{"type":"string"}},"required":["strategy"],"additionalProperties":false}
    ]});
    vec![
        json!({"name":"bridget_capabilities","description":"Catalogue natif des fournisseurs, modèles, efforts et postures accessibles au parent. can_run et constraints donnent le verdict de lancement au dossier verdict_context.cwd, avec les droits attestés de la session. Les faits limités à une demande et les autres dossiers restent soumis à l’admission. Les binaires sont vérifiés ; aucune connexion fournisseur ni authentification n’est testée. Fonctionne sans T3.","inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_delegate","description":"Crée un enfant Bridget et remet sa mission en un appel natif. Conserver request_id sur retry identique ; tâche, résultat et annulation appartiennent au parent. keep_alive_secs garde l'enfant après la remise pour une nouvelle ronde. Le brief est déclaré par le parent ; sources_not_verified : aucune source citée n'est vérifiée. Le journal peut être lu plus largement que l'enfant ; aucun secret dans le brief. Corps du dossier ≤ 16 Kio, refusé plutôt que tronqué.","inputSchema":{"type":"object","properties":{"request_id":{"type":"string","minLength":1,"maxLength":128},"agent_type":{"type":"string"},"model":{"type":"string"},"effort":{"type":["string","null"]},"task":{"type":"string","minLength":1,"maxLength":65536},"cwd":{"type":"string"},"posture":{"type":"string","enum":["discovery","development"]},"reply_timeout":{"type":"integer","minimum":60,"maximum":86400,"description":"Délai de réponse de la mission en secondes (défaut 3600)."},"workspace":workspace_schema,"child_requests":{"type":"string","enum":["deny","forward"],"description":"Remonter les demandes de l’enfant Codex ; absent : deny, politique inchangée."},"keep_alive_secs":{"type":"integer","minimum":60,"maximum":7200,"description":"Durée de garde après la remise du résultat ; absent : aucune garde."},"brief":brief_schema,"fallback":{"type":"array","minItems":1,"maxItems":3,"items":{"type":"object","properties":{"agent_type":{"type":"string","minLength":1},"model":{"type":"string","minLength":1},"effort":{"type":"string","minLength":1}},"required":["agent_type","model"],"additionalProperties":false}}},"required":["request_id","agent_type","model","task","cwd"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
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
    let object = args
        .as_object()
        .ok_or_else(|| ClientError::InvalidParams("arguments objet requis".into()))?;
    let allowed: &[&str] = match name {
        "bridget_capabilities" => &[],
        "bridget_delegate" => &[
            "request_id",
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
        ],
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
            // O(1) : au plus trois candidats, sans résolution ni gain de droits.
            fallback: match args.get("fallback") {
                None => Vec::new(),
                Some(Value::Array(values)) if (1..=3).contains(&values.len()) => {
                    let candidates: Vec<FallbackTarget> = serde_json::from_value(Value::Array(values.clone()))
                        .map_err(|_| ClientError::InvalidParams("fallback invalide".into()))?;
                    let requested_type = string("agent_type")?;
                    let requested_model = string("model")?;
                    let requested_effort = args.get("effort").and_then(Value::as_str);
                    for (index, candidate) in candidates.iter().enumerate() {
                        if candidate.agent_type.trim().is_empty() || candidate.model.trim().is_empty()
                            || candidate.agent_type.chars().any(char::is_control) || candidate.model.chars().any(char::is_control)
                            || candidate.effort.as_ref().is_some_and(|value| value.trim().is_empty() || value.chars().any(char::is_control))
                            || values[index].get("effort").is_some_and(|value| !value.is_string())
                            || candidates[..index].contains(candidate)
                            || (candidate.agent_type == requested_type && candidate.model == requested_model
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
    if matches!(
        &request,
        NativeDelegationRequest::Wait { .. } | NativeDelegationRequest::Ack { .. }
    ) && !daemon_supports_wait(socket)
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
    let message = match crate::t3code_mcp::private_proof() {
        Some(proof) => WrapperToDaemon::NativeDelegationT3 { request, proof },
        None => WrapperToDaemon::NativeDelegation { request },
    };
    match connection.exchange(&message)? {
        DaemonToWrapper::NativeDelegationResult { result } if result["status"] == "refused" => {
            Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: match result.get("detail").filter(|value| !value.is_null()) {
                    Some(detail) => format!("{} {}", result["code"].as_str().unwrap_or("refused"), detail),
                    None => result["code"].as_str().unwrap_or("refused").to_string(),
                },
            })
        }
        DaemonToWrapper::NativeDelegationResult { mut result } => {
            if name == "bridget_capabilities" {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native157_fallback_mcp_schema_ferme() {
        let delegate = crate::mcp::tools().into_iter()
            .find(|tool| tool["name"] == "bridget_delegate").unwrap();
        let fallback = &delegate["inputSchema"]["properties"]["fallback"];
        assert_eq!(fallback["type"], "array");
        assert_eq!(fallback["minItems"], 1);
        assert_eq!(fallback["maxItems"], 3);
        assert_eq!(fallback["items"]["type"], "object");
        assert_eq!(fallback["items"]["additionalProperties"], false);
        assert_eq!(fallback["items"]["required"], json!(["agent_type", "model"]));
        for field in ["agent_type", "model"] {
            assert_eq!(fallback["items"]["properties"][field]["type"], "string");
            assert_eq!(fallback["items"]["properties"][field]["minLength"], 1);
        }
        assert_eq!(fallback["items"]["properties"]["effort"]["type"], "string");
        assert_eq!(fallback["items"]["properties"].as_object().unwrap().len(), 3);
        assert!(!delegate["inputSchema"]["required"].as_array().unwrap()
            .iter().any(|key| key == "fallback"));
    }

    #[test]
    fn native157_fallback_mcp_formes_et_canonique() {
        use std::io::{BufRead, BufReader, Write};
        use std::os::fd::AsRawFd;
        use std::os::unix::net::UnixListener;
        let identity = ResolvedIdentity {
            name: "owner".into(), instance_id: "instance".into(), delegated_origin: None,
        };
        let base = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
        let socket = crate::mcp_identity::mock_socket("fallback157-invalid");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        for fallback in [
            json!(null), json!(false), json!("b"), json!({"agent_type":"b","model":"n"}),
            json!([]), json!(vec![json!({"agent_type":"b","model":"n"}); 4]),
            json!([null]), json!([{}]),
            json!([{"agent_type":"b"}]), json!([{"model":"n"}]),
            json!([{"agent_type":"","model":"n"}]), json!([{"agent_type":"b","model":""}]),
            json!([{"agent_type":2,"model":"n"}]), json!([{"agent_type":"b","model":2}]),
            json!([{"agent_type":"b","model":"n","effort":null}]),
            json!([{"agent_type":"b","model":"n","effort":2}]),
            json!([{"agent_type":"b","model":"n","permissions":"allow"}]),
            json!([{"agent_type":"b","model":"n"},{"agent_type":"b","model":"n"}]),
            json!([{"agent_type":"a","model":"m"}]),
        ] {
            let mut args = base.clone(); args["fallback"] = fallback;
            let result = execute(&identity, "bridget_delegate", &args, &socket);
            assert!(matches!(result, Err(ClientError::InvalidParams(_))), "{args}: {result:?}");
            assert!(matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "une forme invalide a ouvert la socket");
        }
        drop(listener);
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());

        // Réemploi du faux daemon MCP : absence byte-identique et liste bornée
        // transmise telle quelle, sans appeler de fournisseur.
        for fallback in [None, Some(json!([{"agent_type":"b","model":"n"}])),
            Some(json!([{"agent_type":"b","model":"n"},
                {"agent_type":"c","model":"p","effort":"high"},
                {"agent_type":"d","model":"q"}]))] {
            let socket = crate::mcp_identity::mock_socket("fallback157-valid");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let backend = std::thread::spawn(move || {
                let mut pollfd = libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 };
                if unsafe { libc::poll(&mut pollfd, 1, 3000) } <= 0 { return None; }
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new(); reader.read_line(&mut line).unwrap();
                let registration: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(registration["type"], "RegisterAuxiliary");
                writeln!(stream, "{}", json!({"type":"Registered","agent_id":"owner","credential":null})).unwrap();
                line.clear(); reader.read_line(&mut line).unwrap();
                let frame: WrapperToDaemon = serde_json::from_str(&line).unwrap();
                let WrapperToDaemon::NativeDelegation { request } = frame else {
                    panic!("trame native attendue : {frame:?}")
                };
                writeln!(stream, "{}", json!({"type":"NativeDelegationResult","result":{"status":"accepted"}})).unwrap();
                Some(request)
            });
            let mut args = base.clone();
            if let Some(value) = &fallback { args["fallback"] = value.clone(); }
            let result = execute(&identity, "bridget_delegate", &args, &socket);
            let request = backend.join().unwrap();
            let _ = std::fs::remove_dir_all(socket.parent().unwrap());
            assert!(result.is_ok(), "une liste valide doit atteindre le daemon : {args}: {result:?}");
            let request = request.expect("requête transmise");
            if let Some(value) = fallback {
                assert_eq!(serde_json::to_value(request).unwrap()["fallback"], value);
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
                matches!(execute(&identity, "bridget_task_wait", &args, Path::new("/missing")), Err(ClientError::Technical { code: "native_delegation_refused", message }) if message == "wait_unsupported")
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
        let catalogue = crate::mcp::tools();
        let brief_schema = &catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_handoff")
            .unwrap()["inputSchema"]["properties"]["draft"];
        assert_eq!(tools(brief_schema).len(), 8);
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
            (json!({"status":"refused","code":"provider_quota_exhausted","detail":{"resets_at":1_900_000_000}}), "provider_quota_exhausted", Some(json!({"resets_at":1_900_000_000}))),
            (json!({"status":"refused","code":"permission_source_unavailable"}), "permission_source_unavailable", None),
        ] {
            let socket = crate::mcp_identity::mock_socket("quota157");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let backend = std::thread::spawn(move || {
                let mut pollfd = libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 };
                assert!(unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0, "connexion MCP attendue");
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_nonblocking(false).unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let registration: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(registration["type"], "RegisterAuxiliary");
                assert_eq!(registration["agent_id"], "owner");
                writeln!(stream, "{}", json!({"type":"Registered","agent_id":"owner","credential":null})).unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let request: Value = serde_json::from_str(&line).unwrap();
                assert_eq!(request["type"], "NativeDelegation");
                writeln!(stream, "{}", json!({"type":"NativeDelegationResult","result":returned})).unwrap();
                request
            });
            let identity = ResolvedIdentity { name: "owner".into(), instance_id: "instance".into(), delegated_origin: None };
            let args = json!({"request_id":"quota157","agent_type":"codex","model":"fixture","task":"fixture","cwd":socket.parent().unwrap()});
            let result = execute(&identity, "bridget_delegate", &args, &socket);
            let request = backend.join().unwrap();
            assert_eq!(request["request"]["agent_type"], "codex");
            let Err(ClientError::Technical { code, message }) = result else { panic!("refus technique MCP attendu : {result:?}") };
            assert_eq!(code, "native_delegation_refused");
            if let Some(detail) = expected_detail {
                let (actual_code, raw_detail) = message.split_once(' ').expect("code suivi du JSON de detail");
                assert_eq!(actual_code, expected_code);
                assert_eq!(serde_json::from_str::<Value>(raw_detail).unwrap(), detail);
            } else {
                assert_eq!(message, expected_code, "les autres refus gardent le message historique");
            }
            let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        }
    }
}
