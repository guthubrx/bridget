//! Adaptation du protocole natif T3 V2 vers les vues internes du pont.
use crate::t3code_contract::{
    Activity, ContractError, LatestTurn, Message, Snapshot, ThreadDetail,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use tungstenite::{
    client::IntoClientRequest, client::client_with_config, protocol::WebSocketConfig,
};

fn invalid(field: &str) -> ContractError {
    ContractError::InvalidShape {
        source: "T3 orchestration V2",
        field: field.into(),
    }
}

fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, ContractError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid(field))
}

fn nullable_string<'a>(value: &'a Value, field: &str) -> Result<Option<&'a str>, ContractError> {
    match value.get(field) {
        Some(Value::Null) => Ok(None),
        _ => string(value, field).map(Some),
    }
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, ContractError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .filter(|a| a.len() <= 100_000)
        .ok_or_else(|| invalid(field))
}

fn state(value: &str) -> Result<&str, ContractError> {
    match value {
        "preparing" | "queued" | "starting" | "running" | "waiting" | "completed"
        | "interrupted" => Ok(value),
        "failed" => Ok("error"),
        "cancelled" | "rolled_back" => Ok("interrupted"),
        _ => Err(invalid("run.status")),
    }
}

fn validate_relationship(thread: &Value) -> Result<(), ContractError> {
    let Some(lineage) = thread.get("lineage") else {
        return Ok(());
    };
    if lineage.is_null() {
        return Ok(());
    }
    let Some(lineage) = lineage.as_object() else {
        return Err(invalid("lineage.relationshipToParent"));
    };
    match lineage.get("relationshipToParent") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::String(value)) if matches!(value.as_str(), "subagent" | "fork") => Ok(()),
        _ => Err(invalid("lineage.relationshipToParent")),
    }
}

/// O(fils), conversion à la frontière ; le parseur V1 garde les gardes de projet.
pub(crate) fn parse_snapshot(text: &str) -> Result<Snapshot, ContractError> {
    let mut value: Value = serde_json::from_str(text).map_err(|_| invalid("json"))?;
    if value["schemaVersion"] != 2 {
        return Err(invalid("schemaVersion"));
    }
    let threads = value
        .get_mut("threads")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| invalid("threads"))?;
    let ordinary = std::mem::take(threads)
        .into_iter()
        .filter_map(
            |thread| match crate::t3code_contract::is_native_projection(&thread) {
                Ok(true) => None,
                Ok(false) => Some(Ok(thread)),
                Err(error) => Some(Err(error)),
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    *threads = ordinary;
    for thread in threads {
        validate_relationship(thread)?;
        let status = string(thread, "status")?;
        let run_id = nullable_string(thread, "latestRunId")?;
        let active_id = nullable_string(thread, "activeRunId")?;
        let provider = string(thread, "providerInstanceId")?;
        if thread["modelSelection"]["instanceId"] != provider {
            return Err(invalid("modelSelection.instanceId"));
        }
        let session = json!({"providerName":provider,"providerInstanceId":provider,
            "status":if active_id.is_some() { "running" } else { "ready" },"activeTurnId":active_id});
        let latest = match run_id {
            Some(id) => {
                json!({"turnId":id,"state":state(status)?,"requestedAt":thread.get("latestRunRequestedAt")})
            }
            None if status == "idle" && active_id.is_none() => Value::Null,
            None => return Err(invalid("latestRunId/status")),
        };
        thread["session"] = session;
        thread["latestTurn"] = latest;
    }
    crate::t3code_contract::parse_snapshot(&value.to_string())
}

/// O(runs + messages + items), profondeur de nœuds bornée à64.
pub(crate) fn parse_detail(text: &str, thread_id: &str) -> Result<ThreadDetail, ContractError> {
    let value: Value = serde_json::from_str(text).map_err(|_| invalid("json"))?;
    let projection = value
        .get("projection")
        .ok_or_else(|| invalid("projection"))?;
    if string(&projection["thread"], "id")? != thread_id {
        return Err(invalid("thread.id"));
    }
    // La forme compacte officielle omet les éléments locaux déjà présents dans
    // visibleTurnItems. Reconstituer des références, sans dupliquer leurs textes.
    let compact = match value.get("turnItemsOmitLocalVisible") {
        None | Some(Value::Bool(false)) => false,
        Some(Value::Bool(true)) => true,
        _ => return Err(invalid("turnItemsOmitLocalVisible")),
    };
    let mut items = Vec::new();
    if compact {
        for row in array(projection, "visibleTurnItems")? {
            let local = string(row, "visibility")? == "local";
            let source = string(row, "sourceThreadId")?;
            if local || source == thread_id {
                let item = row
                    .get("item")
                    .ok_or_else(|| invalid("visibleTurnItems.item"))?;
                if source != thread_id || string(item, "threadId")? != thread_id {
                    return Err(invalid("visibleTurnItems.threadId"));
                }
                items.push(item);
            }
        }
    }
    items.extend(array(projection, "turnItems")?);
    if items.len() > 100_000 {
        return Err(invalid("turnItems limit"));
    }
    if compact {
        let mut ids = std::collections::HashSet::new();
        for item in &items {
            if !ids.insert(string(item, "id")?) {
                return Err(invalid("turnItems duplicate"));
            }
        }
    }
    let runs = array(projection, "runs")?;
    let mut by_run = HashMap::new();
    let mut activities = Vec::new();
    let mut latest: Option<(u64, LatestTurn)> = None;
    for run in runs {
        if string(run, "threadId")? != thread_id {
            return Err(invalid("run.threadId"));
        }
        let id = string(run, "id")?;
        let status = state(string(run, "status")?)?;
        let ordinal = run["ordinal"]
            .as_u64()
            .ok_or_else(|| invalid("run.ordinal"))?;
        let user = string(run, "userMessageId")?;
        let requested = string(run, "requestedAt")?;
        if by_run.insert(id, run).is_some() {
            return Err(invalid("run.id duplicate"));
        }
        activities.push(Activity {
            id: format!("v2-run:{id}"),
            kind: "t3v2.run".into(),
            turn_id: Some(id.into()),
            payload: json!({"status":status,"userMessageId":user}),
        });
        if latest
            .as_ref()
            .is_none_or(|(previous, _)| ordinal > *previous)
        {
            latest = Some((
                ordinal,
                LatestTurn {
                    turn_id: id.into(),
                    state: status.into(),
                    assistant_message_id: None,
                    requested_at: Some(requested.into()),
                },
            ));
        }
    }
    let nodes: HashMap<_, _> = array(projection, "nodes")?
        .iter()
        .map(|node| Ok((string(node, "id")?, node)))
        .collect::<Result<_, ContractError>>()?;
    let mut messages = Vec::new();
    for message in array(projection, "messages")? {
        if string(message, "threadId")? != thread_id {
            return Err(invalid("message.threadId"));
        }
        let role = string(message, "role")?;
        if role == "system" {
            continue;
        }
        if !matches!(role, "user" | "assistant") {
            return Err(invalid("message.role"));
        }
        let run_id = nullable_string(message, "runId")?;
        if role == "assistant"
            && !root_message(
                message,
                run_id.and_then(|id| by_run.get(id).copied()),
                &nodes,
            )?
        {
            continue;
        }
        let id = string(message, "id")?;
        let text = message["text"]
            .as_str()
            .ok_or_else(|| invalid("message.text"))?;
        let streaming = message["streaming"]
            .as_bool()
            .ok_or_else(|| invalid("message.streaming"))?;
        messages.push(Message {
            id: id.into(),
            role: role.into(),
            text: text.into(),
            streaming,
            turn_id: run_id.map(str::to_owned),
            created_at: Some(string(message, "createdAt")?.into()),
        });
    }
    for item in &items {
        if string(item, "threadId")? != thread_id {
            return Err(invalid("item.threadId"));
        }
        let kind = string(item, "type")?;
        let status = string(item, "status")?;
        let run_id = nullable_string(item, "runId")?.map(str::to_owned);
        if kind == "compaction"
            && matches!(status, "running" | "completed" | "failed" | "cancelled")
            && root_message(
                item,
                run_id.as_deref().and_then(|id| by_run.get(id).copied()),
                &nodes,
            )?
        {
            let id = string(item, "id")?;
            if id.len() > 256 || id.chars().any(char::is_control) {
                return Err(invalid("compaction.id"));
            }
            activities.push(Activity {
                id: format!("v2-compaction:{id}:{status}"), kind: "context.compaction".into(), turn_id: run_id.clone(),
                payload: json!({"kind":"tool","tool_type":"context_compaction","tool_id":format!("compaction:{}:{id}",run_id.as_deref().unwrap_or("")),
                    "text":"Compactage du contexte","status":status,"turn_id":run_id,"item_id":id}),
            });
        }
        if matches!(
            kind,
            "command_execution"
                | "dynamic_tool"
                | "mcp_tool_call"
                | "web_search"
                | "file_search"
                | "image_view"
        ) && matches!(
            status,
            "pending" | "running" | "completed" | "failed" | "cancelled"
        ) && root_message(
            item,
            run_id.as_deref().and_then(|id| by_run.get(id).copied()),
            &nodes,
        )? {
            activities.push(Activity {
                id: format!("v2-tool:{}:{status}", string(item, "id")?),
                kind: "tool.progress".into(),
                turn_id: run_id.clone(),
                payload: tool_fields(item)?,
            });
        }
        if kind == "file_change" && status == "completed" {
            let files = if let Some(changes) = item.get("changes").and_then(Value::as_array) {
                if changes.len() > 256 {
                    return Err(invalid("changes limit"));
                }
                changes
                    .iter()
                    .map(|change| string(change, "path").map(|path| json!({"path":path})))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                vec![json!({"path":string(item,"fileName")?})]
            };
            activities.push(Activity { id:string(item,"id")?.into(), kind:"tool.completed".into(), turn_id:run_id,
                payload:json!({"itemType":"file_change","status":"completed","data":{"files":files}}) });
        } else if kind == "approval_request" && matches!(status, "running" | "pending") {
            activities.push(Activity {
                id: string(item, "id")?.into(),
                kind: "approval.requested".into(),
                turn_id: run_id,
                payload: Value::Null,
            });
        }
    }
    let requests = array(projection, "runtimeRequests")?;
    for item in items.iter().filter(|i| i["type"] == "user_input_request") {
        if string(item, "threadId")? != thread_id {
            return Err(invalid("user input thread"));
        }
        let run = nullable_string(item, "runId")?;
        if !root_message(item, run.and_then(|id| by_run.get(id).copied()), &nodes)? {
            continue;
        }
        let id = string(item, "requestId")?;
        let Some(request) = requests
            .iter()
            .find(|v| v["id"] == id && v["kind"] == "user_input")
        else {
            continue;
        };
        if request["nodeId"] != item["nodeId"] {
            return Err(invalid("user input node"));
        }
        let questions = item["questions"].clone();
        // Gardes natives, sans reformuler le texte ou les options du fournisseur.
        crate::user_input::validate_questions(&questions)
            .map_err(|_| invalid("user input questions"))?;
        if id.len() > 4096 || serde_json::to_vec(request).map_or(true, |b| b.len() > 16 * 1024) {
            return Err(invalid("user input limit"));
        }
        activities.push(Activity {
            id: format!("v2-user-input:{id}"),
            kind: "user_input.request".into(),
            turn_id: run.map(str::to_owned),
            payload: json!({"request":request,"questions":questions}),
        });
    }
    Ok(ThreadDetail {
        messages,
        latest_turn: latest.map(|(_, turn)| turn),
        activities,
        activities_available: true,
    })
}

/// Même détail autorisé que la source ; charge bornée sous le fragment journal de 16 Kio.
pub(super) fn tool_fields(item: &Value) -> Result<Value, ContractError> {
    for field in ["id", "type", "status"] {
        let value = item[field]
            .as_str()
            .ok_or_else(|| invalid("tool metadata"))?;
        if value.len() > 256 || value.chars().any(char::is_control) {
            return Err(invalid("tool metadata limit"));
        }
    }
    if item["nodeId"]
        .as_str()
        .is_some_and(|s| s.len() > 256 || s.chars().any(char::is_control))
    {
        return Err(invalid("tool node limit"));
    }
    let mut fields = json!({"itemType":item["type"],"status":item["status"],"tool_id":item["id"],
        "source_node_id":item["nodeId"],"tool":item["toolName"].as_str().filter(|s|s.len()<=120 && !s.chars().any(char::is_control)),
        "output_omitted":item["outputOmitted"].as_bool().unwrap_or(false)});
    let mut truncated = false;
    for (source, target, limit) in [
        ("input", "input", 4096),
        ("output", "output", 8192),
        ("pattern", "input", 4096),
        ("patterns", "input", 4096),
        ("results", "output", 8192),
    ] {
        if let Some(value) = item.get(source).filter(|v| !v.is_null()) {
            let text = value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string());
            let mut end = text.len().min(limit);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            truncated |= end < text.len();
            fields[target] = json!(&text[..end]);
        }
    }
    if let Some(exit) = item["exitCode"].as_i64() {
        fields["exit_code"] = json!(exit);
    }
    while fields.to_string().len() > 10 * 1024 {
        let field = if fields["output"].as_str().unwrap_or("").len()
            >= fields["input"].as_str().unwrap_or("").len()
        {
            "output"
        } else {
            "input"
        };
        let text = fields[field].as_str().unwrap_or("");
        if text.is_empty() {
            return Err(invalid("tool detail limit"));
        }
        let mut end = text.len() / 2;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let prefix = text[..end].to_owned();
        fields[field] = json!(prefix);
        truncated = true;
    }
    fields["details_truncated"] = json!(truncated);
    Ok(fields)
}

pub(super) fn tool_detail(
    value: Value,
    thread: &str,
    id: &str,
    run: &str,
    node: &Value,
) -> Result<Value, ContractError> {
    let item = &value["item"];
    if item["id"] != id
        || item["threadId"] != thread
        || item["runId"] != run
        || &item["nodeId"] != node
    {
        return Err(invalid("tool detail binding"));
    }
    tool_fields(item)
}

fn root_message(
    message: &Value,
    run: Option<&Value>,
    nodes: &HashMap<&str, &Value>,
) -> Result<bool, ContractError> {
    let Some(run) = run else {
        return Ok(false);
    };
    let mut node_id = nullable_string(message, "nodeId")?;
    let root = nullable_string(run, "rootNodeId")?;
    // Un historique importé sans nœud garde sa réponse, mais jamais une branche enfant connue.
    if node_id.is_none() {
        return Ok(true);
    }
    for _ in 0..64 {
        if node_id == root {
            return Ok(true);
        }
        let Some(node) = node_id.and_then(|id| nodes.get(id)) else {
            return Ok(false);
        };
        if string(node, "kind")? == "subagent" {
            return Ok(false);
        }
        node_id = nullable_string(node, "parentNodeId")?;
        if node_id.is_none() {
            return Ok(false);
        }
    }
    Err(invalid("node lineage"))
}

pub(crate) fn dispatch_command(
    command: &Value,
    delivery_intent: Option<&str>,
) -> Result<Value, ContractError> {
    if string(command, "type")? != "thread.turn.start" {
        return Err(invalid("command.type"));
    }
    let message = &command["message"];
    if string(message, "role")? != "user" {
        return Err(invalid("message.role"));
    }
    let mut result = json!({"type":"message.dispatch", "commandId":string(command,"commandId")?,
        "threadId":string(command,"threadId")?, "messageId":string(message,"messageId")?,
        "text":message["text"].as_str().ok_or_else(||invalid("message.text"))?,
        "attachments":[], "createdBy":"agent", "creationSource":"mcp",
        "dispatchMode":{"type":"queue_after_active"}});
    if let Some(intent) = delivery_intent {
        if !matches!(intent, "steer" | "restart") {
            return Err(invalid("deliveryIntent"));
        }
        result["deliveryIntent"] = json!(intent);
    }
    Ok(result)
}

// Une échéance totale couvre connexion, handshake et fragments WebSocket,
// même si le pair envoie un octet avant chaque timeout système.
struct RpcSocket {
    stream: TcpStream,
    deadline: Instant,
}
impl RpcSocket {
    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "délai RPC T3 dépassé"))
    }
}
impl Read for RpcSocket {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        self.stream.read(bytes)
    }
}
impl Write for RpcSocket {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.stream.set_write_timeout(Some(self.remaining()?))?;
        self.stream.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

fn rpc_read_error(error: tungstenite::Error) -> ContractError {
    let code = match error {
        tungstenite::Error::Io(error)
            if matches!(
                error.kind(),
                io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
            ) =>
        {
            "rpc_response_timeout"
        }
        tungstenite::Error::Capacity(_) => "rpc_response_limit",
        tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed => {
            "rpc_connection_closed"
        }
        _ => "rpc_response_interrupted",
    };
    // Après envoi, ces causes ne prouvent pas l'absence d'effet.
    ContractError::Transport(code.into())
}

fn rpc_budget(tag: &str, payload: &Value) -> Duration {
    // Une remise peut relire un fil volumineux avant son commit. Le délai
    // historique de 10 s interrompait cette lecture réelle (13,5 s observées).
    Duration::from_secs(
        if tag == "orchestration.dispatchCommand" && payload["type"] == "message.dispatch" {
            60
        } else {
            10
        },
    )
}

/// RPC unary natif, taille/durée bornées ; aucun retry ou repli après envoi.
pub(crate) fn rpc(
    base_url: &str,
    token: &str,
    tag: &str,
    payload: &Value,
) -> Result<Value, ContractError> {
    rpc_internal(base_url, token, tag, payload, false)
}

pub(crate) fn rpc_agent_tracking(
    base_url: &str,
    token: &str,
    payload: &Value,
) -> Result<Value, ContractError> {
    if !matches!(
        payload["type"].as_str(),
        Some("thread.settle" | "thread.unsettle")
    ) || payload["commandId"].as_str().is_none()
    {
        return Err(invalid("agent_tracking command"));
    }
    rpc_internal(
        base_url,
        token,
        "orchestration.dispatchCommand",
        payload,
        true,
    )
}

/// Seul un reçu de refus PRÉCÉDEMMENT gravé prouve l'absence d'effet.
/// La forme est corrélée exactement ; aucun texte libre n'autorise un nouvel ID.
fn tracking_rejection(exit: &Value, command: &Value) -> bool {
    let Some(reasons) = exit["cause"].as_array().filter(|a| a.len() == 1) else {
        return false;
    };
    let reason = &reasons[0];
    let error = &reason["error"];
    exit["_tag"] == "Failure"
        && reason["_tag"] == "Fail"
        && error["_tag"] == "OrchestrationV2DispatchCommandError"
        && error["commandId"] == command["commandId"]
        && error["commandType"] == command["type"]
        && error["cause"]["name"] == "OrchestratorCommandPreviouslyRejectedError"
}

fn rpc_internal(
    base_url: &str,
    token: &str,
    tag: &str,
    payload: &Value,
    tracking: bool,
) -> Result<Value, ContractError> {
    let address: SocketAddr = base_url
        .strip_prefix("http://")
        .ok_or_else(|| invalid("RPC origin"))?
        .parse()
        .map_err(|_| invalid("RPC address"))?;
    if !address.ip().is_loopback() {
        return Err(ContractError::NotLoopback(address.ip().to_string()));
    }
    let budget = rpc_budget(tag, payload);
    let deadline = Instant::now() + budget;
    let stream = TcpStream::connect_timeout(&address, budget.min(Duration::from_secs(10)))
        .map_err(|_| ContractError::Transport("connexion RPC T3 impossible".into()))?;
    let mut request = format!("ws://{address}/ws?orchestrationProtocol=2")
        .into_client_request()
        .map_err(|_| invalid("RPC request"))?;
    request.headers_mut().insert(
        "authorization",
        format!("Bearer {token}")
            .parse()
            .map_err(|_| invalid("RPC credentials"))?,
    );
    let config = WebSocketConfig::default()
        .write_buffer_size(0)
        .max_write_buffer_size(4 * 1024 * 1024)
        .max_message_size(Some(4 * 1024 * 1024))
        .max_frame_size(Some(4 * 1024 * 1024));
    let (mut socket, _) = client_with_config(request, RpcSocket { stream, deadline }, Some(config))
        .map_err(|error| match error {
            tungstenite::HandshakeError::Failure(tungstenite::Error::Http(response))
                if response.status().as_u16() == 401 =>
            {
                ContractError::Unauthorized
            }
            _ => ContractError::Transport("négociation RPC T3 refusée".into()),
        })?;
    let request = json!({"_tag":"Request","id":"bridget","tag":tag,"payload":payload,"headers":[]})
        .to_string();
    if request.len() > 4 * 1024 * 1024 {
        return Err(invalid("RPC request limit"));
    }
    socket
        .send(tungstenite::Message::Text(request.into()))
        .map_err(|_| ContractError::Transport("envoi RPC T3 incertain".into()))?;
    // Le serveur peut grouper plusieurs réponses dans une trame JSON.
    loop {
        let frame = socket.read().map_err(rpc_read_error)?;
        match frame {
            tungstenite::Message::Text(text) => {
                let value: Value = serde_json::from_str(&text).map_err(|_| invalid("RPC JSON"))?;
                let replies = value
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or_else(|| std::slice::from_ref(&value));
                for reply in replies {
                    if reply["_tag"] == "Exit" && reply["requestId"] == "bridget" {
                        // Un échec peut survenir après un commit : ne jamais le convertir
                        // en rejet certain supprimant la corrélation persistée.
                        let result = if reply["exit"]["_tag"] == "Success" {
                            reply["exit"]
                                .get("value")
                                .cloned()
                                .ok_or_else(|| invalid("RPC value"))
                        } else if tracking && tracking_rejection(&reply["exit"], payload) {
                            Err(ContractError::CommandRejected {
                                command_id: payload["commandId"]
                                    .as_str()
                                    .ok_or_else(|| invalid("tracking commandId"))?
                                    .into(),
                            })
                        } else {
                            Err(ContractError::Transport(
                                "RPC T3 terminé sans succès confirmé".into(),
                            ))
                        };
                        let _ = socket.close(None);
                        return result;
                    }
                    if reply["_tag"] == "Defect" {
                        return Err(ContractError::Transport(
                            "défaut du protocole RPC T3".into(),
                        ));
                    }
                }
            }
            tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_) => {}
            _ => return Err(invalid("RPC frame")),
        }
    }
}

#[cfg(test)]
#[path = "t3code_contract_v2_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
mod timeout226_tests {
    #[test]
    fn spec226_rpc_budget_is_reserved_for_message_delivery() {
        use super::*;
        assert_eq!(
            rpc_budget(
                "orchestration.dispatchCommand",
                &json!({"type":"message.dispatch"})
            ),
            Duration::from_secs(60)
        );
        for tag in [
            "server.getConfig",
            "orchestration.getTurnItem",
            "orchestration.dispatchCommand",
        ] {
            assert_eq!(
                rpc_budget(tag, &json!({"type":"runtime-request.respond"})),
                Duration::from_secs(10)
            );
        }
    }
    #[test]
    fn spec226_rpc_failure_keeps_closed_cause_without_private_text() {
        use super::*;
        assert_eq!(
            rpc_read_error(tungstenite::Error::Io(io::Error::new(
                io::ErrorKind::TimedOut,
                "PRIVATE"
            ))),
            ContractError::Transport("rpc_response_timeout".into())
        );
        assert_eq!(
            rpc_read_error(tungstenite::Error::ConnectionClosed),
            ContractError::Transport("rpc_connection_closed".into())
        );
        assert_eq!(
            rpc_read_error(tungstenite::Error::Capacity(
                tungstenite::error::CapacityError::MessageTooLong {
                    size: 5,
                    max_size: 4
                }
            )),
            ContractError::Transport("rpc_response_limit".into())
        );
    }
}
