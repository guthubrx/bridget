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
                let item = row.get("item").ok_or_else(|| invalid("visibleTurnItems.item"))?;
                if source != thread_id || string(item, "threadId")? != thread_id {
                    return Err(invalid("visibleTurnItems.threadId"));
                }
                items.push(item);
            }
        }
    }
    items.extend(array(projection, "turnItems")?);
    if items.len() > 100_000 { return Err(invalid("turnItems limit")); }
    if compact {
        let mut ids = std::collections::HashSet::new();
        for item in &items {
            if !ids.insert(string(item, "id")?) { return Err(invalid("turnItems duplicate")); }
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
            && root_message(item, run_id.as_deref().and_then(|id| by_run.get(id).copied()), &nodes)?
        {
            let id = string(item, "id")?;
            if id.len() > 256 || id.chars().any(char::is_control) { return Err(invalid("compaction.id")); }
            activities.push(Activity {
                id: format!("v2-compaction:{id}:{status}"), kind: "context.compaction".into(), turn_id: run_id.clone(),
                payload: json!({"kind":"tool","tool_type":"context_compaction","tool_id":format!("compaction:{}:{id}",run_id.as_deref().unwrap_or("")),
                    "text":"Compactage du contexte","status":status,"turn_id":run_id,"item_id":id}),
            });
        }
        if matches!(kind, "command_execution" | "dynamic_tool" | "mcp_tool_call" | "web_search" | "file_search" | "image_view")
            && matches!(status, "pending" | "running" | "completed" | "failed" | "cancelled")
            && root_message(item, run_id.as_deref().and_then(|id| by_run.get(id).copied()), &nodes)?
        {
            activities.push(Activity {
                id: format!("v2-tool:{}:{status}", string(item, "id")?), kind: "tool.progress".into(), turn_id: run_id.clone(),
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
    for item in items.iter().filter(|i|i["type"]=="user_input_request") {
        if string(item,"threadId")? != thread_id {return Err(invalid("user input thread"));}
        let run = nullable_string(item,"runId")?;
        if !root_message(item,run.and_then(|id|by_run.get(id).copied()),&nodes)? {continue;}
        let id = string(item,"requestId")?;
        let Some(request) = requests.iter().find(|v|v["id"]==id && v["kind"]=="user_input") else {continue;};
        if request["nodeId"] != item["nodeId"] {return Err(invalid("user input node"));}
        let questions = item["questions"].clone();
        // Gardes natives, sans reformuler le texte ou les options du fournisseur.
        crate::user_input::validate_questions(&questions).map_err(|_|invalid("user input questions"))?;
        if id.len()>4096 || serde_json::to_vec(request).map_or(true,|b|b.len()>16*1024) {return Err(invalid("user input limit"));}
        activities.push(Activity{id:format!("v2-user-input:{id}"),kind:"user_input.request".into(),turn_id:run.map(str::to_owned),payload:json!({"request":request,"questions":questions})});
    }
    Ok(ThreadDetail {
        messages,
        latest_turn: latest.map(|(_, turn)| turn),
        activities,
        activities_available: true,
    })
}

/// Même détail autorisé que la source ; charge bornée sous le fragment journal de 16 Kio.
pub(super) fn tool_fields(item: &Value) -> Result<Value,ContractError> {
    for field in ["id","type","status"] {
        let value=item[field].as_str().ok_or_else(||invalid("tool metadata"))?;
        if value.len()>256 || value.chars().any(char::is_control) {return Err(invalid("tool metadata limit"));}
    }
    if item["nodeId"].as_str().is_some_and(|s|s.len()>256 || s.chars().any(char::is_control)) {
        return Err(invalid("tool node limit"));
    }
    let mut fields=json!({"itemType":item["type"],"status":item["status"],"tool_id":item["id"],
        "source_node_id":item["nodeId"],"tool":item["toolName"].as_str().filter(|s|s.len()<=120 && !s.chars().any(char::is_control)),
        "output_omitted":item["outputOmitted"].as_bool().unwrap_or(false)});
    let mut truncated=false;
    for (source,target,limit) in [("input","input",4096),("output","output",8192),("pattern","input",4096),("patterns","input",4096),("results","output",8192)] {
        if let Some(value)=item.get(source).filter(|v| !v.is_null()) {
            let text=value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string());
            let mut end=text.len().min(limit);while !text.is_char_boundary(end) {end-=1;}
            truncated|=end<text.len();fields[target]=json!(&text[..end]);
        }
    }
    if let Some(exit)=item["exitCode"].as_i64() {fields["exit_code"]=json!(exit);}
    while fields.to_string().len()>10*1024 {
        let field=if fields["output"].as_str().unwrap_or("").len()>=fields["input"].as_str().unwrap_or("").len() {"output"} else {"input"};
        let text=fields[field].as_str().unwrap_or("");if text.is_empty() {return Err(invalid("tool detail limit"));}let mut end=text.len()/2;
        while !text.is_char_boundary(end) {end-=1;}let prefix=text[..end].to_owned();fields[field]=json!(prefix);truncated=true;
    }
    fields["details_truncated"]=json!(truncated);Ok(fields)
}

pub(super) fn tool_detail(value: Value, thread: &str, id: &str, run: &str, node: &Value) -> Result<Value,ContractError> {
    let item=&value["item"];
    if item["id"]!=id || item["threadId"]!=thread || item["runId"]!=run || &item["nodeId"]!=node {
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

/// RPC unary natif, taille/durée bornées ; aucun retry ou repli après envoi.
pub(crate) fn rpc(
    base_url: &str,
    token: &str,
    tag: &str,
    payload: &Value,
) -> Result<Value, ContractError> {
    let address: SocketAddr = base_url
        .strip_prefix("http://")
        .ok_or_else(|| invalid("RPC origin"))?
        .parse()
        .map_err(|_| invalid("RPC address"))?;
    if !address.ip().is_loopback() {
        return Err(ContractError::NotLoopback(address.ip().to_string()));
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let stream = TcpStream::connect_timeout(&address, Duration::from_secs(10))
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
        let frame = socket
            .read()
            .map_err(|_| ContractError::Transport("réponse RPC T3 interrompue".into()))?;
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
pub(crate) mod tests {
    use super::*;
    use crate::t3code_contract::Client;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    /// Serveur privé préparé par l'opérateur. Les runs restent différés : aucun fournisseur.
    #[test]
    #[ignore = "requires a disposable T3 V2 server and scoped fixture token"]
    fn spec147_v2_real_server_interop() {
        let base = std::env::var("BRIDGET_TEST_T3_URL").unwrap();
        let token = std::env::var("BRIDGET_TEST_T3_TOKEN").unwrap();
        let workspace = std::env::var("BRIDGET_TEST_T3_WORKSPACE").unwrap();
        assert!(workspace.starts_with("/private/tmp/t3-bridget-v2."));
        let workspace = format!("{workspace}/fixture-workspace");
        let client = Client::for_base_url(&base, &token);
        let snapshot = client.snapshot().unwrap();
        assert!(client.uses_v2());
        assert!(
            snapshot.threads.iter().all(|t| t.latest_turn.is_none()),
            "refus d'un environnement avec historique"
        );
        let project = uuid::Uuid::new_v4().to_string();
        let thread = uuid::Uuid::new_v4().to_string();
        rpc(&base,&token,"projects.mutate",&json!({"type":"project.create","commandId":uuid::Uuid::new_v4().to_string(),"projectId":project,"title":"Fixture Bridget147","workspaceRoot":workspace,"createWorkspaceRootIfMissing":true})).unwrap();
        rpc(&base,&token,"orchestration.dispatchCommand",&json!({"type":"thread.create","commandId":uuid::Uuid::new_v4().to_string(),"threadId":thread,"projectId":project,"title":"Fixture sans modèle","createdBy":"agent","creationSource":"mcp","modelSelection":{"instanceId":"codex","model":"gpt-5"},"runtimeMode":"approval-required","interactionMode":"default","branch":null,"worktreePath":null})).unwrap();
        let command = json!({"type":"thread.turn.start","commandId":uuid::Uuid::new_v4().to_string(),"threadId":thread,
            "message":{"messageId":uuid::Uuid::new_v4().to_string(),"role":"user","text":"Fixture différée sans fournisseur"}});
        let mut native = dispatch_command(&command, None).unwrap();
        native["dispatchMode"] = json!({"type":"defer_start"});
        let first = rpc(&base, &token, "orchestration.dispatchCommand", &native).unwrap();
        assert_eq!(
            rpc(&base, &token, "orchestration.dispatchCommand", &native).unwrap(),
            first
        );
        let summary = client.snapshot().unwrap();
        assert_eq!(summary.threads.len(), snapshot.threads.len() + 1);
        assert_eq!(
            summary
                .threads
                .iter()
                .find(|t| t.id == thread)
                .unwrap()
                .workspace_root
                .as_deref(),
            Some(workspace.as_str())
        );
        let detail = client.thread_detail(&thread, 20).unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].text, "Fixture différée sans fournisseur");
        assert!(detail.messages[0].turn_id.is_some());
        assert_eq!(detail.latest_turn.unwrap().state, "preparing");
        let full = client.thread_detail(&thread, 40).unwrap();
        assert_eq!(full.messages, detail.messages);
        let mut missing = command;
        missing["threadId"] = json!(uuid::Uuid::new_v4().to_string());
        missing["commandId"] = json!(uuid::Uuid::new_v4().to_string());
        assert!(matches!(
            client.dispatch(&missing, None),
            Err(ContractError::Transport(_))
        ));
    }

    pub(crate) fn http_reply(listener: &TcpListener, status: u16, body: &str, expected: &str) {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            headers.push_str(&line);
            if line == "\r\n" || line.is_empty() {
                break;
            }
        }
        assert!(headers.starts_with(expected), "{headers}");
        assert!(
            headers
                .to_lowercase()
                .contains("authorization: bearer fixture")
        );
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    }

    pub(crate) fn rpc_reply(listener: &TcpListener, tag: &str, value: Option<Value>) -> Value {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut socket = tungstenite::accept_hdr(
            stream,
            |request: &tungstenite::handshake::server::Request, response| {
                assert_eq!(request.uri().to_string(), "/ws?orchestrationProtocol=2");
                assert_eq!(request.headers()["authorization"], "Bearer fixture");
                Ok(response)
            },
        )
        .unwrap();
        let frame: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(frame["tag"], tag);
        assert_eq!(frame["_tag"], "Request");
        if let Some(value) = value {
            socket.send(tungstenite::Message::Text(json!([
                {"_tag":"Exit","requestId":"other","exit":{"_tag":"Success","value":"ignore"}},
                {"_tag":"Exit","requestId":"bridget","exit":{"_tag":"Success","value":value}}
            ]).to_string().into())).unwrap();
        }
        frame["payload"].clone()
    }

    #[test]
    fn spec147_v2_client_detects_reads_and_replays_stable_rpc_ids() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        let server = std::thread::spawn(move || {
            http_reply(
                &listener,
                200,
                r#"{"environmentId":"fixture","serverVersion":"fixture","orchestrationProtocolVersion":2}"#,
                "GET /.well-known/t3/environment ",
            );
            http_reply(
                &listener,
                200,
                &shell().to_string(),
                "GET /api/orchestration/shell ",
            );
            rpc_reply(
                &listener,
                "server.getConfig",
                Some(json!({"providers":[{"instanceId":"claude_glm","driver":"claude"}]})),
            );
            http_reply(
                &listener,
                200,
                &detail().to_string(),
                "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 ",
            );
            let first = rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            );
            let second = rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            );
            assert_eq!(first, second);
            assert_eq!(first["commandId"], "cmd");
            assert_eq!(first["messageId"], "m");
            assert_eq!(first["dispatchMode"]["type"], "queue_after_active");
            assert!(first.get("runtimeMode").is_none());
            assert!(first.get("deliveryIntent").is_none());
        });
        assert_eq!(
            client.snapshot().unwrap().threads[0]
                .session
                .as_ref()
                .unwrap()
                .provider_name,
            "claude"
        );
        assert!(client.uses_v2());
        assert_eq!(client.thread_detail("t", 20).unwrap().messages.len(), 2);
        let command = json!({"type":"thread.turn.start","commandId":"cmd","threadId":"t",
            "runtimeMode":"full-access","message":{"messageId":"m","role":"user","text":"exact"}});
        assert_eq!(client.dispatch(&command, None).unwrap().sequence, 42);
        assert_eq!(client.dispatch(&command, None).unwrap().sequence, 42);
        assert_eq!(
            dispatch_command(&command, Some("steer")).unwrap()["deliveryIntent"],
            "steer"
        );
        server.join().unwrap();
    }

    #[test]
    fn spec147_v2_detection_never_masks_auth_or_malformed_response() {
        for (status, body) in [
            (401, "secret"),
            (403, "secret"),
            (500, "secret"),
            (200, "{}"),
            (200, "<html/>"),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let client = Client::for_base_url(
                &format!("http://{}", listener.local_addr().unwrap()),
                "fixture",
            );
            let server = std::thread::spawn(move || {
                http_reply(&listener, status, body, "GET /.well-known/t3/environment ");
                listener.set_nonblocking(true).unwrap();
                assert!(listener.accept().is_err());
            });
            let error = client.snapshot().unwrap_err();
            assert!(!error.to_string().contains("secret"));
            assert!(!client.uses_v2());
            server.join().unwrap();
        }
    }

    #[test]
    fn spec147_v2_announced_protocol_requires_authenticated_snapshot() {
        for version in [1, 2, 3] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let client = Client::for_base_url(
                &format!("http://{}", listener.local_addr().unwrap()),
                "fixture",
            );
            let server = std::thread::spawn(move || {
                http_reply(&listener,200,&json!({"environmentId":"fixture","serverVersion":"fixture","orchestrationProtocolVersion":version}).to_string(),"GET /.well-known/t3/environment ");
                if version < 3 {
                    http_reply(
                        &listener,
                        401,
                        "secret",
                        if version == 2 {
                            "GET /api/orchestration/shell "
                        } else {
                            "GET /api/orchestration/snapshot "
                        },
                    );
                }
            });
            assert!(client.snapshot().is_err());
            assert!(!client.uses_v2());
            server.join().unwrap();
        }
    }

    #[test]
    fn spec147_v2_lost_rpc_reply_is_uncertain_without_http_fallback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server =
            std::thread::spawn(move || rpc_reply(&listener, "orchestration.dispatchCommand", None));
        let command = json!({"type":"thread.turn.start","commandId":"cmd","threadId":"t","message":{"messageId":"m","role":"user","text":"exact"}});
        assert!(matches!(
            client.dispatch(&command, None),
            Err(ContractError::Transport(_))
        ));
        assert_eq!(server.join().unwrap()["messageId"], "m");
    }

    #[test]
    fn spec147_v2_v1_detection_preserves_legacy_detail() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        let server = std::thread::spawn(move || {
            http_reply(
                &listener,
                200,
                r#"{"environmentId":"fixture","serverVersion":"fixture"}"#,
                "GET /.well-known/t3/environment ",
            );
            http_reply(
                &listener,
                200,
                r#"{"snapshotSequence":0,"projects":[],"threads":[]}"#,
                "GET /api/orchestration/snapshot ",
            );
            http_reply(
                &listener,
                200,
                r#"{"thread":{"messages":[]}}"#,
                "GET /api/orchestration/threads/t?turnLimit=20 ",
            );
        });
        assert!(client.snapshot().unwrap().threads.is_empty());
        assert!(!client.uses_v2());
        assert!(client.thread_detail("t", 20).unwrap().messages.is_empty());
        server.join().unwrap();
    }

    fn shell() -> Value {
        json!({"schemaVersion":2,"snapshotSequence":17,"projects":[{"id":"p","workspaceRoot":"/fixture/project"}],
            "threads":[{"id":"t","projectId":"p","title":"test","providerInstanceId":"claude_glm",
                "modelSelection":{"instanceId":"claude_glm"},"runtimeMode":"approval-required","interactionMode":"default",
                "lineage":{"relationshipToParent":"subagent","parentThreadId":"parent"},
                "latestRunId":"r","latestRunRequestedAt":"2026-10-09T00:00:00.000Z","activeRunId":"r","status":"running",
                "updatedAt":"2026-10-09T00:00:01.000Z","archivedAt":null,"deletedAt":null,"settledOverride":null}]})
    }

    #[test]
    fn mobile183_compaction_items_keep_root_lineage_and_status_identity() {
        let mut raw=detail();
        raw["projection"]["turnItems"]=json!([
            {"id":"comp183","type":"compaction","threadId":"t","runId":"r","nodeId":"root","status":"running"},
            {"id":"child-comp","type":"compaction","threadId":"t","runId":"r","nodeId":"child","status":"running"}
        ]);
        let parsed=parse_detail(&raw.to_string(),"t").unwrap();
        let acts:Vec<_>=parsed.activities.iter().filter(|a|a.kind=="context.compaction").collect();
        assert_eq!(acts.len(),1);assert_eq!(acts[0].payload["status"],"running");
        let first=acts[0].id.clone();
        raw["projection"]["turnItems"][0]["status"]=json!("completed");
        let done=parse_detail(&raw.to_string(),"t").unwrap();
        assert_ne!(done.activities.iter().find(|a|a.kind=="context.compaction").unwrap().id,first);
        raw["projection"]["turnItems"][0]["threadId"]=json!("foreign");
        assert!(parse_detail(&raw.to_string(),"t").is_err());
    }

    #[test]
    fn spec221_compact_t3_snapshot_keeps_local_tools_and_rejects_foreign_rows() {
        let mut plain = detail();
        let root=json!({"id":"compact221","type":"compaction","threadId":"t","runId":"r","nodeId":"root","status":"running"});
        let child=json!({"id":"child221","type":"compaction","threadId":"t","runId":"r","nodeId":"child","status":"running"});
        plain["projection"]["turnItems"]=json!([root.clone(),child.clone()]);
        let expected=parse_detail(&plain.to_string(),"t").unwrap();
        let mut compact=plain.clone();
        compact["turnItemsOmitLocalVisible"]=json!(true);
        compact["projection"]["turnItems"]=json!([]);
        compact["projection"]["visibleTurnItems"]=json!([
            {"visibility":"local","sourceThreadId":"t","item":root.clone()},
            {"visibility":"local","sourceThreadId":"t","item":child},
            {"visibility":"inherited","sourceThreadId":"ancestor","item":{"id":"foreign221","threadId":"ancestor"}}
        ]);
        let actual=parse_detail(&compact.to_string(),"t").unwrap();
        assert_eq!(actual,expected,
            "la forme compacte ne doit perdre ni message, ni outil racine, ni état");
        compact["projection"]["visibleTurnItems"][0]["item"]["threadId"]=json!("foreign");
        assert!(parse_detail(&compact.to_string(),"t").is_err());
        compact["projection"]["visibleTurnItems"][0]["item"]=root;
        compact["projection"]["visibleTurnItems"][0]["sourceThreadId"]=json!("foreign");
        assert!(parse_detail(&compact.to_string(),"t").is_err());
        compact["projection"]["visibleTurnItems"][0]["sourceThreadId"]=json!("t");
        compact["projection"]["turnItems"]=json!([compact["projection"]["visibleTurnItems"][0]["item"].clone()]);
        assert!(parse_detail(&compact.to_string(),"t").is_err(),"un même outil ne doit pas être émis deux fois");
    }

    #[test]
    fn spec221_compact_http_avoids_duplicate_payload_without_raising_limit() {
        let mut plain=detail();
        let item=json!({"id":"large221","threadId":"t","runId":"r","nodeId":"root",
            "type":"dynamic_tool","status":"completed","output":"x".repeat(9*1024*1024)});
        plain["projection"]["turnItems"]=json!([item.clone()]);
        plain["projection"]["visibleTurnItems"]=json!([
            {"visibility":"local","sourceThreadId":"t","item":item}]);
        assert!(plain.to_string().len()>16*1024*1024);
        plain["projection"]["turnItems"]=json!([]);
        plain["turnItemsOmitLocalVisible"]=json!(true);
        let body=plain.to_string();
        assert!(body.len()<16*1024*1024);
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let client=Client::for_base_url(&format!("http://{}",listener.local_addr().unwrap()),"fixture");
        client.set_test_protocol(true);
        let server=std::thread::spawn(move||http_reply(&listener,200,&body,
            "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 "));
        let parsed=client.thread_detail("t",20).unwrap();
        let tool=parsed.activities.iter().find(|a|a.kind=="tool.progress").unwrap();
        assert_eq!(tool.payload["tool_id"],"large221");
        assert_eq!(tool.payload["details_truncated"],true);
        assert!(tool.payload.to_string().len()<16*1024);
        server.join().unwrap();
    }

    pub(super) fn detail() -> Value {
        json!({"snapshotSequence":18,"projection":{
            "thread":{"id":"t"},
            "runs":[{"id":"r","threadId":"t","ordinal":1,"userMessageId":"u","rootNodeId":"root","status":"completed","requestedAt":"2026-10-09T00:00:00.000Z"}],
            "nodes":[{"id":"root","kind":"root_turn","parentNodeId":null},{"id":"child","kind":"subagent","parentNodeId":"root"}],
            "messages":[
                {"id":"u","threadId":"t","runId":"r","nodeId":null,"role":"user","text":"question","streaming":false,"createdAt":"2026-10-09T00:00:00.000Z"},
                {"id":"child-message","threadId":"t","runId":"r","nodeId":"child","role":"assistant","text":"secret sous-agent","streaming":false,"createdAt":"2026-10-09T00:00:01.000Z"},
                {"id":"a","threadId":"t","runId":"r","nodeId":"root","role":"assistant","text":"réponse exacte","streaming":false,"createdAt":"2026-10-09T00:00:02.000Z"}],
            "turnItems":[],"runtimeRequests":[],"visibleTurnItems":[]}})
    }

    #[test]
    fn user_input181_t3_response_uses_same_command_after_lost_reply() {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let client=Client::for_base_url(&format!("http://{}",listener.local_addr().unwrap()),"fixture");client.set_test_protocol(true);
        let server=std::thread::spawn(move || {
            let first=rpc_reply(&listener,"orchestration.dispatchCommand",None);
            let second=rpc_reply(&listener,"orchestration.dispatchCommand",Some(json!({"sequence":42})));
            assert_eq!(first,second);first
        });
        let answers=json!({"q":["A","ma précision"]});
        assert!(client.answer_user_input("t","req181","command181",&answers).is_err());
        client.answer_user_input("t","req181","command181",&answers).unwrap();
        assert_eq!(server.join().unwrap(),json!({"type":"runtime-request.respond","commandId":"command181","threadId":"t","requestId":"req181","answers":answers}));
    }

    #[test]
    fn user_input181_t3_preserves_questions_and_external_resolution() {
        let mut value=detail();
        let questions=json!([{"id":"q","header":"Choix","question":"Lequel ?","options":[{"label":"A","description":"Exact"}],"multiSelect":false,"allowCustomAnswer":true,"initialAnswer":"Brouillon","required":true}]);
        value["projection"]["turnItems"]=json!([{"id":"ui","type":"user_input_request","threadId":"t","runId":"r","nodeId":"root","status":"pending","requestId":"req181","questions":questions}]);
        value["projection"]["runtimeRequests"]=json!([{"id":"req181","nodeId":"root","kind":"user_input","status":"pending","responseCapability":{"type":"live","providerSessionId":"sdk181"}}]);
        let parsed=parse_detail(&value.to_string(),"t").unwrap();
        let question=parsed.activities.iter().find(|a|a.kind=="user_input.request").unwrap();
        assert_eq!(question.payload["questions"],questions);
        let mut compact=value.clone();
        compact["turnItemsOmitLocalVisible"]=json!(true);
        compact["projection"]["visibleTurnItems"]=json!([{"visibility":"local","sourceThreadId":"t","item":value["projection"]["turnItems"][0].clone()}]);
        compact["projection"]["turnItems"]=json!([]);
        assert_eq!(parse_detail(&compact.to_string(),"t").unwrap(),parsed);

        value["projection"]["runtimeRequests"][0]["status"]=json!("resolved");
        value["projection"]["runtimeRequests"][0]["answers"]=json!({"q":["A"]});
        let parsed=parse_detail(&value.to_string(),"t").unwrap();
        assert_eq!(parsed.activities.iter().find(|a|a.kind=="user_input.request").unwrap().payload["request"]["status"],"resolved");
        value["projection"]["runtimeRequests"][0]["responseCapability"]=json!({"type":"not_resumable","reason":"gone"});
        assert_eq!(parse_detail(&value.to_string(),"t").unwrap().activities.iter().find(|a|a.kind=="user_input.request").unwrap().payload["request"]["responseCapability"]["type"],"not_resumable");
        value["projection"]["turnItems"][0]["threadId"]=json!("foreign");
        assert!(parse_detail(&value.to_string(),"t").is_err());
    }

    #[test]
    fn mobile177_activity_projects_only_root_details_and_distinct_states() {
        let mut raw=detail();
        raw["projection"]["turnItems"]=json!([
            {"id":"tool","threadId":"t","runId":"r","nodeId":"root","type":"dynamic_tool","status":"running","toolName":"Read","input":{"path":"/fixture/readme.md"},"output":"file contents"},
            {"id":"child-tool","threadId":"t","runId":"r","nodeId":"child","type":"dynamic_tool","status":"running","toolName":"Private child","input":"never-copy"}
        ]);
        let current=parse_detail(&raw.to_string(),"t").unwrap();
        let tools:Vec<_>=current.activities.iter().filter(|a|a.kind=="tool.progress").collect();
        assert_eq!(tools.len(),1);
        assert_eq!(tools[0].payload["tool"],"Read");
        assert_eq!(tools[0].payload["tool_id"],"tool");
        assert!(tools[0].payload["input"].as_str().unwrap().contains("/fixture/readme.md"));
        assert_eq!(tools[0].payload["output"],"file contents");
        assert!(!tools[0].payload.to_string().contains("never-copy"));
        let first=tools[0].id.clone();
        raw["projection"]["turnItems"][0]["status"]=json!("completed");
        let done=parse_detail(&raw.to_string(),"t").unwrap();
        assert_ne!(done.activities.iter().find(|a|a.kind=="tool.progress").unwrap().id,first);
    }

    #[test]
    fn mobile177_tool_detail_is_bound_and_large_unicode_output_is_explicit() {
        let item=json!({"id":"x","threadId":"t","runId":"r","nodeId":"root","type":"command_execution","status":"completed","input":"echo confirmed","output":"é".repeat(9000),"exitCode":0});
        let value=json!({"item":item});
        let fields=tool_detail(value.clone(),"t","x","r",&json!("root")).unwrap();
        assert_eq!(fields["input"],"echo confirmed");assert_eq!(fields["exit_code"],0);
        assert_eq!(fields["details_truncated"],true);assert!(fields.to_string().len()<13*1024);
        for (thread,id,run,node) in [("other","x","r","root"),("t","other","r","root"),("t","x","other","root"),("t","x","r","child")] {
            assert!(tool_detail(value.clone(),thread,id,run,&json!(node)).is_err());
        }
    }

    #[test]
    fn mobile177_tool_metadata_is_bounded_before_detail_reduction() {
        let item=json!({"id":"x".repeat(16000),"nodeId":"root","type":"command_execution","status":"completed"});
        assert!(tool_fields(&item).is_err());
        for field in ["nodeId","type","status"] {
            let mut item=json!({"id":"x","nodeId":"root","type":"command_execution","status":"completed"});
            item[field]=json!("x".repeat(16000));assert!(tool_fields(&item).is_err());
        }
    }

    #[test]
    fn mobile177_tool_detail_fits_the_actual_final_journal_entry() {
        let item=json!({"id":"\"".repeat(256),"nodeId":"root","type":"command_execution","status":"completed","input":"\"".repeat(4000),"output":"\"".repeat(5000),"exitCode":0});
        let mut fields=tool_fields(&item).unwrap();
        let activity=format!("v2-tool:{}:completed",item["id"].as_str().unwrap());
        fields["kind"]=json!("tool");fields["t3_activity_id"]=json!(activity);
        let message=format!("t3-activity:{activity}:0");
        let root=std::env::temp_dir().join(format!("mobile-tool-size-{}",uuid::Uuid::new_v4()));
        let agent=uuid::Uuid::new_v4().to_string();
        let mut journal=bridget_transport::journal::SessionJournal::new(&root,&agent,&uuid::Uuid::new_v4().to_string()).unwrap();
        journal.append("update",Some(&message),fields).unwrap();drop(journal);
        let page=crate::attach::lineage_journal_page(&root.join(&agent),&agent,0,50).unwrap();
        assert!(page["gap"].is_null());assert_eq!(page["events"].as_array().unwrap().len(),1);
        assert!(page["events"][0].to_string().len()<16*1024);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec147_v2_shell_preserves_busy_state_and_project() {
        let parsed = parse_snapshot(&shell().to_string()).unwrap();
        let thread = &parsed.threads[0];
        assert_eq!(thread.workspace_root.as_deref(), Some("/fixture/project"));
        assert_eq!(thread.latest_turn.as_ref().unwrap().turn_id, "r");
        assert_eq!(thread.relationship_to_parent.as_deref(), Some("subagent"));
        assert_eq!(
            thread.session.as_ref().unwrap().active_turn_id.as_deref(),
            Some("r")
        );
        assert_eq!(
            crate::t3code::thread_provider(thread).as_deref(),
            Some("claude")
        );
    }

    #[test]
    fn spec165_v2_relation_reelle_est_portee_et_inconnue_refusee() {
        for (value, expected) in [
            (json!("subagent"), Some("subagent")),
            (json!("fork"), Some("fork")),
            (Value::Null, None),
        ] {
            let mut fixture = shell();
            fixture["threads"][0]["lineage"]["relationshipToParent"] = value;
            assert_eq!(
                parse_snapshot(&fixture.to_string()).unwrap().threads[0]
                    .relationship_to_parent
                    .as_deref(),
                expected
            );
        }
        let mut absent = shell();
        absent["threads"][0]
            .as_object_mut()
            .unwrap()
            .remove("lineage");
        assert_eq!(
            parse_snapshot(&absent.to_string()).unwrap().threads[0].relationship_to_parent,
            None
        );
        let mut null_lineage = shell();
        null_lineage["threads"][0]["lineage"] = Value::Null;
        assert_eq!(
            parse_snapshot(&null_lineage.to_string()).unwrap().threads[0].relationship_to_parent,
            None
        );
        for invalid in [json!("future"), json!(7), json!({})] {
            let mut fixture = shell();
            fixture["threads"][0]["lineage"]["relationshipToParent"] = invalid;
            assert!(matches!(
                parse_snapshot(&fixture.to_string()),
                Err(ContractError::InvalidShape { field, .. })
                    if field == "lineage.relationshipToParent"
            ));
        }
    }

    #[test]
    fn spec147_v2_unknown_status_and_incomplete_shell_refuse() {
        for (field, value) in [("schemaVersion", json!(3)), ("schemaVersion", Value::Null)] {
            let mut shell = shell();
            shell[field] = value;
            assert!(parse_snapshot(&shell.to_string()).is_err());
        }
        let mut mismatch = shell();
        mismatch["threads"][0]["modelSelection"]["instanceId"] = json!("other");
        assert!(parse_snapshot(&mismatch.to_string()).is_err());
        for status in ["new-future-status", ""] {
            let mut value = shell();
            value["threads"][0]["status"] = json!(status);
            assert!(parse_snapshot(&value.to_string()).is_err());
        }
        let mut value = shell();
        value["threads"][0]
            .as_object_mut()
            .unwrap()
            .remove("activeRunId");
        assert!(parse_snapshot(&value.to_string()).is_err());
    }

    #[test]
    fn spec147_v2_detail_preserves_text_and_run_without_child_reply() {
        let parsed = parse_detail(&detail().to_string(), "t").unwrap();
        assert_eq!(
            parsed
                .messages
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["u", "a"]
        );
        assert_eq!(parsed.messages[0].turn_id.as_deref(), Some("r"));
        assert_eq!(parsed.messages[1].text, "réponse exacte");
        assert_eq!(parsed.latest_turn.unwrap().state, "completed");
        assert!(
            parsed
                .activities
                .iter()
                .any(|a| a.kind == "t3v2.run" && a.payload["userMessageId"] == "u")
        );
    }

    #[test]
    fn spec147_v2_detail_refuses_wrong_thread_and_unknown_run_state() {
        assert!(parse_detail(&detail().to_string(), "foreign").is_err());
        let mut value = detail();
        value["projection"]["runs"][0]["status"] = json!("unknown");
        assert!(parse_detail(&value.to_string(), "t").is_err());
    }

    #[test]
    fn spec147_v2_run_end_states_are_explicit() {
        for (status, expected) in [
            ("failed", "error"),
            ("cancelled", "interrupted"),
            ("rolled_back", "interrupted"),
            ("queued", "queued"),
        ] {
            let mut value = detail();
            value["projection"]["runs"][0]["status"] = json!(status);
            assert_eq!(
                parse_detail(&value.to_string(), "t")
                    .unwrap()
                    .latest_turn
                    .unwrap()
                    .state,
                expected
            );
        }
    }
    #[test]
    fn guidance156_t3_steer_pose_delivery_intent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server = std::thread::spawn(move || {
            rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            )
        });
        let command = json!({"type":"thread.turn.start","commandId":"guide",
            "threadId":"thread-1","message":{"messageId":"message-guide","role":"user","text":"complément"}});
        assert_eq!(
            client.dispatch(&command, Some("steer")).unwrap().sequence,
            42
        );
        let frame = server.join().unwrap();
        assert_eq!(frame["deliveryIntent"], "steer");
        assert_eq!(frame["text"], "complément");
        assert_eq!(frame["commandId"], "guide");
    }

    #[test]
    fn guidance156_t3_interrupt_pose_restart() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server = std::thread::spawn(move || {
            rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":43})),
            )
        });
        let command = json!({"type":"thread.turn.start","commandId":"restart",
            "threadId":"thread-1","message":{"messageId":"message-restart","role":"user","text":"nouvelle direction"}});
        assert_eq!(
            client.dispatch(&command, Some("restart")).unwrap().sequence,
            43
        );
        let frame = server.join().unwrap();
        assert_eq!(frame["deliveryIntent"], "restart");
        assert_eq!(frame["text"], "nouvelle direction");
    }
}
