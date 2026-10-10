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
    let ordinary=std::mem::take(threads).into_iter().filter_map(|thread| {
        match crate::t3code_contract::is_native_projection(&thread) {
            Ok(true)=>None,Ok(false)=>Some(Ok(thread)),Err(error)=>Some(Err(error)),
        }
    }).collect::<Result<Vec<_>,_>>()?;
    *threads=ordinary;
    for thread in threads {
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
    for item in array(projection, "turnItems")? {
        if string(item, "threadId")? != thread_id {
            return Err(invalid("item.threadId"));
        }
        let kind = string(item, "type")?;
        let status = string(item, "status")?;
        let run_id = nullable_string(item, "runId")?.map(str::to_owned);
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
    Ok(ThreadDetail {
        messages,
        latest_turn: latest.map(|(_, turn)| turn),
        activities,
        activities_available: true,
    })
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
    steer_current: bool,
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
    if steer_current {
        result["deliveryIntent"] = json!("steer");
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
mod tests {
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
        let mut native = dispatch_command(&command, false).unwrap();
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
            client.dispatch(&missing, false),
            Err(ContractError::Transport(_))
        ));
    }

    fn http_reply(listener: &TcpListener, status: u16, body: &str, expected: &str) {
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

    fn rpc_reply(listener: &TcpListener, tag: &str, value: Option<Value>) -> Value {
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
                "GET /api/orchestration/threads/t/bounded ",
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
        assert_eq!(client.dispatch(&command, false).unwrap().sequence, 42);
        assert_eq!(client.dispatch(&command, false).unwrap().sequence, 42);
        assert_eq!(
            dispatch_command(&command, true).unwrap()["deliveryIntent"],
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
            client.dispatch(&command, false),
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
                "latestRunId":"r","latestRunRequestedAt":"2026-10-09T00:00:00.000Z","activeRunId":"r","status":"running",
                "updatedAt":"2026-10-09T00:00:01.000Z","archivedAt":null,"deletedAt":null,"settledOverride":null}]})
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
    fn spec147_v2_shell_preserves_busy_state_and_project() {
        let parsed = parse_snapshot(&shell().to_string()).unwrap();
        let thread = &parsed.threads[0];
        assert_eq!(thread.workspace_root.as_deref(), Some("/fixture/project"));
        assert_eq!(thread.latest_turn.as_ref().unwrap().turn_id, "r");
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
}
