//! Même façade native pour les agents T3, CLI et MCP externes.
use crate::communication::client::{ClientError, registered_connection, unexpected_response};
use crate::mcp_identity::ResolvedIdentity;
use bridget_transport::protocol::{NativeDelegationRequest, SpawnPosture};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn tools() -> Vec<Value> {
    vec![
        json!({"name":"bridget_capabilities","description":"Catalogue natif des fournisseurs, modèles, efforts et postures accessibles au parent. Fonctionne sans T3.","inputSchema":{"type":"object","properties":{},"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
        json!({"name":"bridget_delegate","description":"Crée un enfant Bridget et remet sa mission en un appel natif. Conserver request_id sur retry identique ; tâche, résultat et annulation appartiennent au parent.","inputSchema":{"type":"object","properties":{"request_id":{"type":"string","minLength":1,"maxLength":128},"agent_type":{"type":"string"},"model":{"type":"string"},"effort":{"type":["string","null"]},"task":{"type":"string","minLength":1,"maxLength":65536},"cwd":{"type":"string"},"posture":{"type":"string","enum":["discovery","development"]}},"required":["request_id","agent_type","model","task","cwd","posture"],"additionalProperties":false},"annotations":{"readOnlyHint":false,"destructiveHint":true,"idempotentHint":true,"openWorldHint":true}}),
        json!({"name":"bridget_task_status","description":"Lit l'état durable et le résultat stable de sa mission native. Une fin de tour ne vaut pas résultat disponible tant que les enfants travaillent.","inputSchema":{"type":"object","properties":{"task_id":{"type":"string"}},"required":["task_id"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true}}),
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
        ],
        "bridget_task_status" | "bridget_task_cancel" => &["task_id"],
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
            posture: match string("posture")?.as_str() {
                "discovery" => SpawnPosture::Discovery,
                "development" => SpawnPosture::Development,
                _ => return Err(ClientError::InvalidParams("posture invalide".into())),
            },
        },
        "bridget_task_status" => NativeDelegationRequest::Status {
            task_id: string("task_id")?,
        },
        "bridget_task_cancel" => NativeDelegationRequest::Cancel {
            task_id: string("task_id")?,
        },
        _ => unreachable!(),
    };
    let mut connection = registered_connection(&identity.name, &identity.instance_id, socket)?;
    match connection.exchange(&WrapperToDaemon::NativeDelegation { request })? {
        DaemonToWrapper::NativeDelegationResult { result } if result["status"] == "refused" => {
            Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: result["code"].as_str().unwrap_or("refused").to_string(),
            })
        }
        DaemonToWrapper::NativeDelegationResult { result } => Ok(result),
        other => unexpected_response(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(tools().len(), 4);
    }
}
