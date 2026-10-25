//! Autorité181 : un producteur pose sa question, seul un client humain répond.
use super::*;
use crate::user_input as store;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

fn refused(code: &str) -> Value {
    json!({"status":"refused","code":code})
}
fn project(st: &DaemonState, agent: &str) -> Option<String> {
    agent_communication_project(st, agent).and_then(|p| serde_json::to_string(&p).ok())
}
pub(super) fn live(st: &DaemonState, row: &Value) -> bool {
    let Some(agent) = row["owner"].as_str() else {
        return false;
    };
    if matches!(row["origin"].as_str(), Some("codex" | "claude"))
        && !st
            .presences
            .get(row["instance"].as_str().unwrap_or(""))
            .is_some_and(|p| p.state == "busy")
    {
        return false;
    }
    st.router
        .get_agent(agent)
        .and_then(|r| live_connection_identity(st, &r.connection_id))
        .is_some_and(|(a, i)| {
            a == agent && row["instance"] == i && row["project"] == json!(project(st, agent))
        })
}
pub(super) fn public(st: &DaemonState, row: Value) -> Value {
    let mut value = store::projection(row.clone());
    // Même identité et nom que l'annuaire ; aucun identifiant fourni par le modèle.
    if let Some(owner) = row["owner"].as_str() {
        if let Some(presence) = st.presences.get(row["instance"].as_str().unwrap_or("")) {
            value["source_agent"] = json!({"agent_id":owner,
                "display_name":st.store.agent_display_name(owner).ok().flatten().unwrap_or_else(||owner.to_owned()),
                "machine":presence.host,"state":presence.state});
        }
    }
    value["can_respond"] = json!(
        row["state"] == "pending"
            && live(st, &row)
            && row["expires_at"]
                .as_i64()
                .is_some_and(|t| t > unix_now_secs())
    );
    value
}
fn execute(conn: &str, request: &Value, st: &DaemonState) -> Result<Value, String> {
    if serde_json::to_vec(request)
        .map_err(|_| "invalid_request")?
        .len()
        > 64 * 1024
    {
        return Err("resource_limit".into());
    }
    let action = request["action"].as_str().ok_or("invalid_request")?;
    let mut allowed = vec!["action", "request_id"];
    allowed.extend(match action {
        "create" => vec!["questions"],
        "publish" => vec!["questions", "origin", "callback"],
        "wait" => vec!["wait_timeout"],
        "status" | "cancel" => vec![],
        "list" | "pending" => vec!["agent_id", "limit"],
        "respond" => vec!["revision", "command_id", "answers"],
        "ack" => vec!["command_id", "applied"],
        "close" => vec!["state"],
        _ => return Err("invalid_request".into()),
    });
    if !request
        .as_object()
        .is_some_and(|o| o.keys().all(|k| allowed.contains(&k.as_str())))
    {
        return Err("invalid_request".into());
    }
    if !matches!(
        st.connection_roles.get(conn),
        Some(ConnectionRole::Client | ConnectionRole::Wrapper)
    ) {
        return Err("forbidden_scope".into());
    }
    let primary = live_connection_identity(st, conn);
    let negotiated = st.client_negotiations.get(conn).is_some_and(|n| {
        n.version == CLIENT_CONTRACT_VERSION
            && n.capabilities.contains(&ClientCapability::UserInputV1)
    });
    if !negotiated && (primary.is_none() || st.auxiliary_connections.contains(conn)) {
        return Err("capability_not_negotiated".into());
    }
    let human = negotiated
        && st.connection_roles.get(conn) == Some(&ConnectionRole::Client)
        && control_client_actor(st, conn).is_some()
        && st
            .client_negotiations
            .get(conn)
            .is_some_and(|n| n.capabilities.contains(&ClientCapability::ControlStateV1));
    let id = request["request_id"].as_str().unwrap_or("");
    let now = unix_now_secs();
    let db = st.store.connection();
    if matches!(action, "list" | "respond") {
        if !human {
            return Err("human_actor_required".into());
        }
        if action == "list" {
            let limit = request["limit"].as_u64().unwrap_or(50) as usize;
            let rows = store::list(db, request["agent_id"].as_str(), now, limit)?;
            return Ok(
                json!({"status":"ok","items":rows.into_iter().map(|r|public(st,r)).collect::<Vec<_>>()}),
            );
        }
        let row = store::get(db, id)?.ok_or("question_unavailable")?;
        if !live(st, &row) {
            return Err("question_not_resumable".into());
        }
        let result = store::respond(
            db,
            id,
            request["revision"].as_str().ok_or("invalid_request")?,
            request["command_id"].as_str().ok_or("invalid_request")?,
            control_client_actor(st, conn).ok_or("human_actor_required")?,
            &request["answers"],
            now,
        )?;
        return Ok(
            json!({"status":"ok","item":public(st,result),"_changed":row["state"]=="pending"}),
        );
    }
    if action == "status" && human {
        let row = store::get(db, id)?.ok_or("question_unavailable")?;
        return Ok(json!({"status":"ok","item":public(st,row)}));
    }
    let (owner, instance) = primary.ok_or("identity_unavailable")?;
    if st
        .fleet
        .agent_link_for_child(&instance)
        .ok()
        .flatten()
        .is_some_and(|l| l.role == "native_delegate")
    {
        return Err("delegated_tool_forbidden".into());
    }
    if matches!(action, "create" | "publish") {
        let origin = if action == "publish" {
            request["origin"].as_str().ok_or("invalid_request")?
        } else {
            "mcp"
        };
        if action == "publish" && st.auxiliary_connections.contains(conn) {
            return Err("forbidden_scope".into());
        }
        let fresh = store::get(db, id)?.is_none();
        let row = store::create(
            db,
            &owner,
            &instance,
            project(st, &owner).as_deref(),
            id,
            &request["questions"],
            origin,
            if action == "publish" {
                &request["callback"]
            } else {
                &Value::Null
            },
            now,
        )?;
        return Ok(json!({"status":"ok","item":row,"_changed":fresh}));
    }
    if action == "pending" {
        let items = store::list(db, Some(&owner), now, 100)?
            .into_iter()
            .filter(|r| r["instance"] == instance && r["state"] == "submitted")
            .collect::<Vec<_>>();
        return Ok(json!({"status":"ok","items":items}));
    }
    let row = store::get(db, id)?
        .filter(|r| r["owner"] == owner && r["instance"] == instance)
        .ok_or("question_unavailable")?;
    if row["project"] != json!(project(st, &owner)) {
        return Err("forbidden_scope".into());
    }
    let before = row.clone();
    let result = match action {
        "status" | "wait" => {
            if row["state"] == "pending" && row["expires_at"].as_i64().is_none_or(|v| v <= now) {
                store::finish(db, id, &owner, &instance, None, "expired", now)?
            } else {
                row
            }
        }
        "cancel" => store::finish(db, id, &owner, &instance, None, "cancelled", now)?,
        "ack" => store::finish(
            db,
            id,
            &owner,
            &instance,
            request["command_id"].as_str(),
            if request["applied"] == true {
                "applied"
            } else {
                "unknown"
            },
            now,
        )?,
        "close" => store::finish(
            db,
            id,
            &owner,
            &instance,
            None,
            request["state"].as_str().ok_or("invalid_request")?,
            now,
        )?,
        _ => return Err("invalid_request".into()),
    };
    let changed = result != before;
    Ok(json!({"status":"ok","item":result,"_changed":changed}))
}
pub(super) fn handle(
    conn: &str,
    request: &Value,
    state: &Arc<Mutex<DaemonState>>,
    peer: Option<&UnixStream>,
) -> DaemonToWrapper {
    let waiting = request["action"] == "wait";
    let wait = if waiting {
        request["wait_timeout"].as_u64().unwrap_or(3600)
    } else {
        0
    };
    if waiting
        && (!(1..=3600).contains(&wait)
            || request
                .get("wait_timeout")
                .is_some_and(|v| v.as_u64().is_none()))
    {
        return DaemonToWrapper::UserInputResult {
            result: refused("invalid_request"),
        };
    }
    let deadline = Instant::now() + Duration::from_secs(wait);
    let (epoch, active) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        (st.delegation_store.wait_epoch(), st.active_waits.clone())
    };
    let _permit = if waiting {
        match WaitPermit::acquire(&active) {
            Ok(p) => Some(p),
            Err(e) => {
                return DaemonToWrapper::UserInputResult {
                    result: refused(&e),
                };
            }
        }
    } else {
        None
    };
    loop {
        let seen = epoch.get();
        if waiting && (wait_client_gone(peer) || SHUTDOWN_REQUESTED.load(Ordering::Acquire)) {
            return DaemonToWrapper::UserInputResult {
                result: refused("daemon_unreachable"),
            };
        }
        let result = {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            execute(conn, request, &st)
        };
        match result {
            Ok(value)
                if waiting && value["item"]["state"] == "pending" && Instant::now() < deadline => {}
            Ok(mut value) => {
                let changed = value
                    .as_object_mut()
                    .and_then(|o| o.remove("_changed"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if changed {
                    epoch.bump();
                    if let Some(n) = &state
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .mobile_notifier
                    {
                        n.changed();
                    }
                }
                if value["item"]["state"] == "submitted" {
                    tick_request(state, value["item"]["request_id"].as_str());
                }
                return DaemonToWrapper::UserInputResult { result: value };
            }
            Err(code) => {
                return DaemonToWrapper::UserInputResult {
                    result: refused(&code),
                };
            }
        }
        epoch.wait_changed(
            seen,
            Duration::from_secs(1).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn form() -> Value {
        json!([{"id":"q","header":"Choix","question":"Quelle solution ?","options":[{"label":"A"},{"label":"B"}],"multiSelect":false}])
    }
    #[test]
    fn user_input181_authority_separates_producer_human_and_foreign() {
        let label = format!("user-input181-{}", uuid::Uuid::new_v4().simple());
        let (mut st, _config) = super::super::presence_tests::state_with_registered_agent(&label);
        let (writer, _reader) = super::super::presence_tests::control_socket(&label);
        st.connections.insert("conn-1".into(), writer);
        st.connection_roles
            .insert("conn-1".into(), ConnectionRole::Wrapper);
        let id = uuid::Uuid::new_v4().to_string();
        let created = execute(
            "conn-1",
            &json!({"action":"create","request_id":id,"questions":form()}),
            &st,
        )
        .unwrap();
        let rev = created["item"]["revision"].clone();
        let command = uuid::Uuid::new_v4().to_string();
        let response = json!({"action":"respond","request_id":id,"revision":rev,"command_id":command,"answers":{"q":{"selected":["B"],"text":"Complément"}}});
        assert_eq!(
            execute("conn-1", &response, &st).unwrap_err(),
            "human_actor_required"
        );
        assert!(execute("foreign", &json!({"action":"status","request_id":id}), &st).is_err());
        st.connection_roles
            .insert("human181".into(), ConnectionRole::Client);
        st.client_negotiations.insert(
            "human181".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
                capabilities: vec![
                    ClientCapability::ControlStateV1,
                    ClientCapability::UserInputV1,
                ],
            },
        );
        assert_eq!(
            execute("human181", &response, &st).unwrap()["item"]["state"],
            "submitted"
        );
        let forged = json!({"action":"ack","request_id":id,"command_id":command,"applied":true});
        assert!(execute("human181", &forged, &st).is_err());
        assert_eq!(
            execute("conn-1", &forged, &st).unwrap()["item"]["state"],
            "applied"
        );
        assert_eq!(
            execute("human181", &response, &st).unwrap()["item"]["state"],
            "applied"
        );
    }
}

/// Adaptateur des callbacks structurés existants ; la question n’emporte aucun droit.
fn native_form(
    request: &bridget_transport::protocol::ChildProviderRequest,
) -> Result<Value, String> {
    if request.version != 1 {
        return Err("unsupported_version".into());
    }
    if request.truncated || request.details["secret"] == true {
        return Err("secret_not_forwardable".into());
    }
    let raw = request.details["questions"]
        .as_array()
        .ok_or("invalid_questions")?;
    let mut output = Vec::new();
    for (index, q) in raw.iter().enumerate() {
        if q["isSecret"] == true || q["is_secret"] == true {
            return Err("secret_not_forwardable".into());
        }
        let id = q["id"].as_str().ok_or("invalid_questions")?;
        let options=q["options"].as_array().ok_or("invalid_questions")?.iter().map(|o|{
            let mut v=json!({"label":o["label"],"description":o.get("description").cloned().unwrap_or(json!(""))});
            if let Some(value)=o.get("value"){v["value"]=value.clone();}v
        }).collect::<Vec<_>>();
        let mut normalized = json!({"id":id,"header":q.get("header").cloned().unwrap_or(json!(format!("Question {}",index+1))),"question":q["question"],"options":options,"multiSelect":q["multiSelect"].as_bool().unwrap_or(false),"allowCustomAnswer":q["allowCustomAnswer"].as_bool().unwrap_or(true)});
        for key in ["initialAnswer", "required"] {
            if let Some(v) = q.get(key) {
                normalized[key] = v.clone();
            }
        }
        output.push(normalized);
    }
    let form = json!(output);
    store::validate_questions(&form)?;
    if output_ids(&form) != request.question_ids {
        return Err("invalid_questions".into());
    }
    Ok(form)
}
fn output_ids(form: &Value) -> Vec<String> {
    form.as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["id"].as_str().map(str::to_owned))
        .collect()
}
pub(super) fn primary_request(
    conn: &str,
    request: &bridget_transport::protocol::ChildProviderRequest,
    st: &DaemonState,
) -> bool {
    request.kind == bridget_transport::protocol::ChildRequestKind::UserInput
        && (request.details["method"] == "t3.runtime-request.respond"
            || st
                .native_permission_facts
                .get(conn)
                .is_none_or(|(fact, _)| fact.provider_session_id == request.provider_session))
        && !st.auxiliary_connections.contains(conn)
        && live_connection_identity(st, conn).is_some_and(|(_, i)| {
            !st.fleet
                .agent_link_for_child(&i)
                .ok()
                .flatten()
                .is_some_and(|l| l.role == "native_delegate")
        })
}
pub(super) fn receive_native(
    conn: &str,
    request: bridget_transport::protocol::ChildProviderRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    use bridget_transport::protocol::{ChildRequestAnswer, ChildRequestDecision};
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let identity = live_connection_identity(&st, conn);
    let revision_key = if request.details["method"] == "t3.runtime-request.respond" {
        request.details["questions"].clone()
    } else {
        Value::Null
    };
    let key = json!([
        identity,
        request.provider_session,
        request.turn_id,
        request.provider_request_id,
        revision_key
    ])
    .to_string();
    let id = Uuid::new_v5(
        &Uuid::NAMESPACE_OID,
        format!("bridget.user-input.v1:{key}").as_bytes(),
    )
    .to_string();
    let result = (|| -> Result<Value, String> {
        if !primary_request(conn, &request, &st) {
            return Err("request_unlinked".into());
        }
        let (owner, instance) = identity.ok_or("identity_unavailable")?;
        let origin = match request.details["method"].as_str() {
            Some("item/tool/requestUserInput") => "codex",
            Some("AskUserQuestion") => "claude",
            Some("t3.runtime-request.respond") => "t3",
            _ => return Err("request_unsupported".into()),
        };
        let form = native_form(&request)?;
        let callback = serde_json::to_value(&request).map_err(|_| "invalid_request")?;
        let fresh = store::get(st.store.connection(), &id)?.is_none();
        let row = store::create(
            st.store.connection(),
            &owner,
            &instance,
            project(&st, &owner).as_deref(),
            &id,
            &form,
            origin,
            &callback,
            unix_now_secs(),
        )?;
        if fresh {
            if let Some(n) = &st.mobile_notifier {
                n.changed();
            }
        }
        Ok(row)
    })();
    match result {
        Ok(item) => DaemonToWrapper::UserInputResult {
            result: json!({"status":"ok","item":item}),
        },
        Err(reason) => DaemonToWrapper::ChildRequestAnswer {
            answer: ChildRequestAnswer {
                request_id: id,
                provider_session: request.provider_session,
                turn_id: request.turn_id,
                provider_request_id: request.provider_request_id,
                decision: ChildRequestDecision::Decline,
                answer: None,
                outcome: format!("user_input_refused:{reason}"),
            },
        },
    }
}
/// Même trame/requête rejouée : le wrapper155 garde l’application unique et ses reçus.
pub(super) fn tick(state: &Arc<Mutex<DaemonState>>) {
    tick_request(state, None)
}
pub(super) fn tick_request(state: &Arc<Mutex<DaemonState>>, request_id: Option<&str>) {
    use bridget_transport::protocol::{
        ChildProviderRequest, ChildRequestAnswer, ChildRequestDecision,
    };
    let outgoing = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        let after = st
            .user_input_dispatch_after
            .clone()
            .unwrap_or((-1, String::new()));
        let db = st.store.connection();
        let read = (|| -> Result<Vec<Value>, String> {
            let mut stmt=db.prepare("SELECT data FROM user_input_requests WHERE state='submitted' AND origin IN ('codex','claude','t3') AND (?1 IS NULL OR id=?1) AND (?1 IS NOT NULL OR created_at>?2 OR (created_at=?2 AND id>?3)) ORDER BY created_at,id LIMIT 32").map_err(|_|"storage_unavailable")?;
            let rows = stmt
                .query_map(rusqlite::params![request_id, after.0, after.1], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(|_| "storage_unavailable")?;
            rows.map(|r| {
                r.map_err(|_| "storage_unavailable".into()).and_then(|s| {
                    serde_json::from_str(&s).map_err(|_| "storage_unavailable".into())
                })
            })
            .collect()
        })();
        let Ok(rows) = read else {
            return;
        };
        if request_id.is_none() {
            st.user_input_dispatch_after = rows.last().and_then(|r| {
                Some((
                    r["created_at"].as_i64()?,
                    r["request_id"].as_str()?.to_owned(),
                ))
            });
        }
        let mut outgoing = Vec::new();
        for row in rows {
            if !live(&st, &row) {
                continue;
            }
            let Some(owner) = row["owner"].as_str() else {
                continue;
            };
            let Some(route) = st.router.get_agent(owner) else {
                continue;
            };
            let Some(writer) = st.connections.get(&route.connection_id).cloned() else {
                continue;
            };
            let Ok(callback) =
                serde_json::from_value::<ChildProviderRequest>(row["callback"].clone())
            else {
                continue;
            };
            let answer = if matches!(row["origin"].as_str(), Some("codex" | "t3")) {
                let mut map = serde_json::Map::new();
                for id in &callback.question_ids {
                    let a = &row["answers"][id];
                    let mut values = a["selected"].as_array().cloned().unwrap_or_default();
                    if let Some(text) = a["text"].as_str().filter(|t| !t.is_empty()) {
                        values.push(json!(if row["origin"] == "codex" {
                            format!("user_note: {text}")
                        } else {
                            text.to_owned()
                        }));
                    }
                    map.insert(id.clone(), Value::Array(values));
                }
                Value::Object(map)
            } else {
                row["answers"].clone()
            };
            let frame = DaemonToWrapper::ChildRequestAnswer {
                answer: ChildRequestAnswer {
                    request_id: row["request_id"].as_str().unwrap_or("").into(),
                    provider_session: callback.provider_session,
                    turn_id: callback.turn_id,
                    provider_request_id: callback.provider_request_id,
                    decision: ChildRequestDecision::Accept,
                    answer: Some(answer),
                    outcome: if row["origin"] == "t3" {
                        row["command_id"].as_str().unwrap_or("").into()
                    } else {
                        "human_user_input".into()
                    },
                },
            };
            outgoing.push((writer, frame));
        }
        outgoing
    };
    let deadline = Instant::now() + Duration::from_secs(1);
    for (writer, frame) in outgoing {
        if let Err(e) = push_control_message_until(&writer, &frame, deadline) {
            warn!("user_input181 callback non confirmé : {e}");
        }
    }
}
pub(super) fn native_ack(
    conn: &str,
    id: &str,
    applied: Option<bool>,
    closed: Option<bridget_transport::protocol::ChildRequestClosedState>,
    st: &DaemonState,
) -> bool {
    let Ok(Some(row)) = store::get(st.store.connection(), id) else {
        return false;
    };
    if !matches!(row["origin"].as_str(), Some("codex" | "claude" | "t3")) {
        return false;
    }
    let Some((owner, instance)) = live_connection_identity(st, conn) else {
        return true;
    };
    if row["owner"] != owner
        || row["instance"] != instance
        || st.auxiliary_connections.contains(conn)
    {
        return true;
    }
    let phase = if applied == Some(true)
        || (closed == Some(bridget_transport::protocol::ChildRequestClosedState::AlreadyApplied)
            && row["state"] == "submitted")
    {
        "applied"
    } else if applied == Some(false) {
        "closed"
    } else if row["state"] == "pending" {
        "resolved_elsewhere"
    } else {
        "closed"
    };
    if store::finish(
        st.store.connection(),
        id,
        &owner,
        &instance,
        row["command_id"].as_str(),
        phase,
        unix_now_secs(),
    )
    .is_ok_and(|updated| updated["state"] != row["state"])
    {
        st.delegation_store.wait_epoch().bump();
        if let Some(n) = &st.mobile_notifier {
            n.changed();
        }
    }
    true
}

#[cfg(test)]
mod callback_tests {
    use super::*;
    use bridget_transport::protocol::{
        ChildProviderRequest, ChildProviderRequestId, ChildRequestDecision, ChildRequestKind,
    };
    #[test]
    fn user_input181_dispatch_progresses_after_32_unconfirmed_sources() {
        let label = format!("user-input181-fair-{}", Uuid::new_v4().simple());
        let (mut st, _config) = super::super::presence_tests::state_with_registered_agent(&label);
        let (writer, mut reader) = super::super::presence_tests::control_socket(&label);
        st.connections.insert("conn-1".into(), writer);
        st.connection_roles
            .insert("conn-1".into(), ConnectionRole::Wrapper);
        st.set_turn_state("conn-1", true).unwrap();
        let request = ChildProviderRequest {
            version: 1,
            provider_session: "provider181".into(),
            turn_id: "turn181".into(),
            provider_request_id: ChildProviderRequestId::String("fair181".into()),
            kind: ChildRequestKind::UserInput,
            details: json!({"method":"item/tool/requestUserInput","secret":false,"questions":[{"id":"q","header":"Choix","question":"Laquelle ?","options":[{"label":"A"}]}]}),
            offered: vec![ChildRequestDecision::Accept],
            question_ids: vec!["q".into()],
            truncated: false,
        };
        let form = native_form(&request).unwrap();
        let now = unix_now_secs();
        for n in 0..33 {
            let id = Uuid::new_v4().to_string();
            let owner = Uuid::new_v4().to_string();
            let row = store::create(
                st.store.connection(),
                &owner,
                "offline181",
                None,
                &id,
                &form,
                "codex",
                &Value::Null,
                now - 100 + n,
            )
            .unwrap();
            store::respond(
                st.store.connection(),
                &id,
                row["revision"].as_str().unwrap(),
                &Uuid::new_v4().to_string(),
                "human",
                &json!({"q":{"selected":["A"],"text":""}}),
                now,
            )
            .unwrap();
        }
        let state = Arc::new(Mutex::new(st));
        let DaemonToWrapper::UserInputResult { result } = receive_native("conn-1", request, &state)
        else {
            panic!("native question")
        };
        let id = result["item"]["request_id"].as_str().unwrap();
        {
            let st = state.lock().unwrap();
            store::respond(
                st.store.connection(),
                id,
                result["item"]["revision"].as_str().unwrap(),
                &Uuid::new_v4().to_string(),
                "human",
                &json!({"q":{"selected":["A"],"text":""}}),
                now,
            )
            .unwrap();
        }
        tick(&state);
        tick(&state);
        assert!(
            matches!(super::super::presence_tests::read_control(&mut reader),DaemonToWrapper::ChildRequestAnswer{answer} if answer.request_id==id)
        );
    }
    #[test]
    fn user_input181_native_codex_callback_is_bound_and_confirmed_separately() {
        let label = format!("user-input181-cb-{}", Uuid::new_v4().simple());
        let (mut st, _config) = super::super::presence_tests::state_with_registered_agent(&label);
        let (writer, mut reader) = super::super::presence_tests::control_socket(&label);
        st.connections.insert("conn-1".into(), writer);
        st.connection_roles
            .insert("conn-1".into(), ConnectionRole::Wrapper);
        st.set_turn_state("conn-1", true).unwrap();
        let state = Arc::new(Mutex::new(st));
        let request = ChildProviderRequest {
            version: 1,
            provider_session: "provider181".into(),
            turn_id: "turn181".into(),
            provider_request_id: ChildProviderRequestId::String("rpc181".into()),
            kind: ChildRequestKind::UserInput,
            details: json!({"method":"item/tool/requestUserInput","secret":false,"questions":[{"id":"q","header":"Choix","question":"Quelle solution ?","options":[{"label":"A"},{"label":"B"}]}]}),
            offered: vec![ChildRequestDecision::Accept, ChildRequestDecision::Decline],
            question_ids: vec!["q".into()],
            truncated: false,
        };
        let DaemonToWrapper::UserInputResult { result } =
            receive_native("conn-1", request.clone(), &state)
        else {
            panic!("request refused")
        };
        let id = result["item"]["request_id"].as_str().unwrap().to_owned();
        let revision = result["item"]["revision"].as_str().unwrap();
        let cmd = Uuid::new_v4().to_string();
        {
            let st = state.lock().unwrap();
            store::respond(
                st.store.connection(),
                &id,
                revision,
                &cmd,
                "human181",
                &json!({"q":{"selected":["B"],"text":"Conserve les fichiers"}}),
                unix_now_secs(),
            )
            .unwrap();
        }
        tick(&state);
        let frame = super::super::presence_tests::read_control(&mut reader);
        let DaemonToWrapper::ChildRequestAnswer { answer } = frame else {
            panic!("missing callback")
        };
        assert_eq!(answer.provider_request_id, request.provider_request_id);
        assert_eq!(
            answer.answer,
            Some(json!({"q":["B","user_note: Conserve les fichiers"]}))
        );
        {
            let st = state.lock().unwrap();
            assert_eq!(
                store::get(st.store.connection(), &id).unwrap().unwrap()["state"],
                "submitted"
            );
            assert!(native_ack("foreign", &id, Some(true), None, &st));
            assert_eq!(
                store::get(st.store.connection(), &id).unwrap().unwrap()["state"],
                "submitted"
            );
            assert!(native_ack("conn-1", &id, Some(true), None, &st));
            assert_eq!(
                store::get(st.store.connection(), &id).unwrap().unwrap()["state"],
                "applied"
            );
        }
    }
}
