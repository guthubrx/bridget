//! Boîte humaine native : une réponse reste un choix offert, son application est acquittée séparément.
use super::*;
use crate::mobile::store::{Operation, Store};
use bridget_transport::protocol::{
    HumanInboxItemFrame, HumanInboxState, MobileOperation, MobileOperationRequest,
};
use serde_json::{Value, json};
use std::path::Path;

fn fail(code: &str) -> Value {
    json!({"status":"refused","code":code})
}
fn uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|v| v.get_version_num() == 4 && v.to_string() == id)
}
fn actor(device: &str, id: &str) -> String {
    format!("mobile177:respond:{device}:{id}")
}
fn projection(
    item: &HumanInboxItemFrame,
    respond: bool,
    st: &DaemonState,
) -> Result<Value, &'static str> {
    if item.context.len() > crate::human_inbox::MAX_CONTEXT_BYTES
        || item.options.len() > 32
        || item.options.iter().any(|s| s.len() > 1024)
    {
        return Err("resource_limit");
    }
    let context: Value = serde_json::from_str(&item.context).unwrap_or(Value::Null);
    let summary = crate::human_inbox::channel_summary(item, 0, false).summary;
    // Le schéma natif155 distingue les faits vérifiés du texte non fiable du modèle.
    let facts = context.get("facts").cloned().unwrap_or(Value::Null);
    if serde_json::to_vec(&facts)
        .map_err(|_| "question_unavailable")?
        .len()
        > 4096
    {
        return Err("resource_limit");
    }
    let quoted = context
        .get("model_text")
        .and_then(|v| v.get("quoted_untrusted"))
        .and_then(Value::as_str)
        .map(|s| s.chars().take(2048).collect::<String>());
    Ok(
        json!({"item_id":item.id,"kind":item.kind,"subject":item.subject,"summary":summary,
        "facts":facts,"quoted_untrusted":quoted,"options":item.options,"state":item.state,
        "created_at":item.created_at,"resolved_at":item.resolved_at,"acked_at":item.acked_at,
        "decision":item.decision.as_ref().map(|d|json!({"decision_id":d.decision_id,"choice":d.choice,"at":d.at})),
        "can_respond":respond && item.state==HumanInboxState::Open,
        "application":application(st,item)?}),
    )
}
pub(super) fn handle(
    request: &MobileOperationRequest,
    root: &Path,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    let device = match Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        Ok(d) => d,
        Err(e) => {
            return DaemonToWrapper::MobileOperationResult {
                result: fail(e.code()),
            };
        }
    };
    if matches!(request.operation,MobileOperation::UserInputs{..}|MobileOperation::UserInputShow{..}|MobileOperation::UserInputRespond{..}|MobileOperation::UserInputReceipt{..}) {
        let result=handle_user_input(&request.operation,&device,&st);
        drop(st);
        if let Err(e)=Store::open(&root.join("mobile-devices.db")).and_then(|s|s.authenticate(request.credential.expose_for_attestation(),unix_now_secs())) {return DaemonToWrapper::MobileOperationResult{result:fail(e.code())};}
        if result["item"]["state"]=="submitted" {user_input::tick_request(state,result["item"]["request_id"].as_str());}
        return DaemonToWrapper::MobileOperationResult{result};
    }
    let writing = matches!(
        request.operation,
        MobileOperation::QuestionRespond { .. } | MobileOperation::QuestionReceipt { .. }
    );
    let op = if writing {
        Operation::TaskRespond
    } else {
        Operation::TaskRead
    };
    if !device.scope.permits_federation(op) {
        return DaemonToWrapper::MobileOperationResult {
            result: fail("forbidden_scope"),
        };
    }
    let conn = st.store.connection();
    let respond = device.scope.permits_federation(Operation::TaskRespond);
    let result = match &request.operation {
        MobileOperation::Questions { after, limit } => {
            if !(1..=50).contains(limit) || after.as_ref().is_some_and(|s| s.len() > 2048) {
                fail("invalid_request")
            } else {
                let cursor = after
                    .as_ref()
                    .map(|s| serde_json::from_str::<(i64, String)>(s))
                    .transpose();
                match cursor {
                    Ok(cursor) if cursor.as_ref().is_none_or(|(t, id)| *t >= 0 && uuid(id)) => {
                        match crate::human_inbox::list_open_page(
                            conn,
                            *limit + 1,
                            cursor.as_ref().map(|(t, id)| (*t, id.as_str())),
                        ) {
                            Ok(rows) => {
                                let mut items = Vec::new();
                                let mut bytes = 1024;
                                let mut last = None;
                                let mut more = false;
                                let mut error = None;
                                for item in rows {
                                    let value = match projection(&item, respond, &st) {
                                        Ok(v) => v,
                                        Err(c) => {
                                            error = Some(c);
                                            break;
                                        }
                                    };
                                    let len =
                                        serde_json::to_vec(&value).map_or(usize::MAX, |v| v.len());
                                    if items.len() >= *limit as usize
                                        || bytes + len > 128 * 1024 - 4096
                                    {
                                        more = true;
                                        break;
                                    }
                                    bytes += len;
                                    last = Some((item.created_at, item.id));
                                    items.push(value);
                                }
                                if let Some(code) = error {
                                    fail(code)
                                } else {
                                    let next = if more {
                                        last.and_then(|v| serde_json::to_string(&v).ok())
                                    } else {
                                        None
                                    };
                                    json!({"status":"ok","items":items,"next_cursor":next,"snapshot":"current_native_inbox"})
                                }
                            }
                            Err(_) => fail("storage_unavailable"),
                        }
                    }
                    _ => fail("invalid_request"),
                }
            }
        }
        MobileOperation::QuestionShow { item_id } => {
            if !uuid(item_id) {
                fail("invalid_request")
            } else {
                match crate::human_inbox::get(conn, item_id) {
                    Ok(Some(item)) => match projection(&item, respond, &st) {
                        Ok(v) => json!({"status":"ok","item":v}),
                        Err(c) => fail(c),
                    },
                    Ok(None) => fail("not_found"),
                    Err(_) => fail("storage_unavailable"),
                }
            }
        }
        MobileOperation::QuestionRespond {
            device_id,
            command_id,
            item_id,
            choice,
        }
        | MobileOperation::QuestionReceipt {
            device_id,
            command_id,
            item_id,
            choice,
        } => {
            if device_id != &device.device_id {
                fail("command_origin_changed")
            } else if !uuid(command_id)
                || !uuid(item_id)
                || choice.is_empty()
                || choice.len() > 1024
            {
                fail("invalid_request")
            } else {
                let actor = actor(&device.device_id, command_id);
                match crate::human_inbox::find_decision_by_actor(conn, &actor) {
                    Ok(Some(item)) => {
                        if item.id != *item_id
                            || item.decision.as_ref().is_none_or(|d| d.choice != *choice)
                        {
                            fail("envelope_mismatch")
                        } else {
                            receipt(&item, command_id, &st)
                        }
                    }
                    Ok(None)
                        if matches!(request.operation, MobileOperation::QuestionReceipt { .. }) =>
                    {
                        fail("not_found")
                    }
                    Ok(None) => match crate::human_inbox::resolve(
                        conn,
                        item_id,
                        choice,
                        &actor,
                        unix_now_secs(),
                    ) {
                        Ok(Ok(item)) => receipt(&item, command_id, &st),
                        Ok(Err(reason)) => {
                            use bridget_transport::protocol::HumanInboxRefusal;
                            fail(match reason {
                                HumanInboxRefusal::AlreadyResolved => "question_already_resolved",
                                HumanInboxRefusal::ChoiceNotOffered => "choice_not_offered",
                                HumanInboxRefusal::UnknownItem => "not_found",
                                _ => "question_unavailable",
                            })
                        }
                        Err(_) => fail("storage_unavailable"),
                    },
                    Err(_) => fail("storage_unavailable"),
                }
            }
        }
        _ => fail("invalid_request"),
    };
    drop(st);
    if let Err(e) = Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        return DaemonToWrapper::MobileOperationResult {
            result: fail(e.code()),
        };
    }
    DaemonToWrapper::MobileOperationResult { result }
}
fn receipt(item: &HumanInboxItemFrame, command: &str, st: &DaemonState) -> Value {
    let Some(decision) = &item.decision else {
        return fail("question_unavailable");
    };
    json!({"status":"ok","operation_kind":"question_respond","command_id":command,"issue":{"kind":"accepted"},
        "receipt":{"item_id":item.id,"decision_id":decision.decision_id,"choice":decision.choice,"state":item.state,
            "acked_at":item.acked_at,"application":match application(st,item){Ok(v)=>v,Err(c)=>return fail(c)}},
        "message":"Choix enregistré ; son application est confirmée séparément par le producteur."})
}

/// L'ACK peut aussi solder une réponse devenue invalide ; seul le reçu fournisseur confirme l'application.
fn application(st: &DaemonState, item: &HumanInboxItemFrame) -> Result<Value, &'static str> {
    let (confirmed, state) = if let Some(id) = item.dedup_key.strip_prefix("native-child-request:")
    {
        let row = st
            .delegation_store
            .get_child_request(id)
            .map_err(|_| "storage_unavailable")?
            .filter(|r| r.inbox_item_id.as_deref() == Some(item.id.as_str()))
            .ok_or("question_unavailable")?;
        match row.delivery_state.as_str() {
            "delivered" => (Some(true), "delivered"),
            "void" => (Some(false), "not_applied"),
            "unknown" => (None, "unknown"),
            "pending" | "sent" => (None, "pending"),
            _ => (None, "not_confirmed"),
        }
    } else {
        (None, "not_confirmed")
    };
    Ok(json!({"producer_acknowledged":item.acked_at.is_some(),"confirmed":confirmed,"state":state}))
}

fn user_input_permitted(device:&crate::mobile::store::Device,st:&DaemonState,row:&Value,op:Operation)->bool {
    let Some(owner)=row["owner"].as_str()else{return false;};
    let Some((agent,instance))=st.router.get_agent(owner).and_then(|r|live_connection_identity(st,&r.connection_id))else{return false;};
    let Some(presence)=st.presences.get(&instance)else{return false;};
    row["instance"]==instance && row["project"]==json!(agent_communication_project(st,&agent).and_then(|p|serde_json::to_string(&p).ok()))
        && device.scope.permits(op,&presence.host,agent_communication_project(st,&agent).as_ref())
}
fn handle_user_input(operation:&MobileOperation,device:&crate::mobile::store::Device,st:&DaemonState)->Value {
    let result=(||->Result<Value,String>{
        use crate::user_input as store;
        let db=st.store.connection();let now=unix_now_secs();
        if let MobileOperation::UserInputs{after,limit}=operation {
            if !(1..=50).contains(limit) || after.as_ref().is_some_and(|s|s.len()>256) {return Err("invalid_request".into());}
            let cursor=after.as_ref().map(|s|serde_json::from_str::<(i64,String)>(s)).transpose().map_err(|_|"invalid_request")?;
            if cursor.as_ref().is_some_and(|(t,id)|*t<0 || Uuid::parse_str(id).is_err()) {return Err("invalid_request".into());}
            let rows=store::list_page(db,cursor.as_ref().map(|(t,id)|(*t,id.as_str())),now,*limit as usize+1)?;
            let mut items=Vec::new();let mut last=None;let mut more=false;let mut bytes=1024;
            for (n,row) in rows.into_iter().enumerate() {
                if n>=*limit as usize {more=true;break;}
                if user_input_permitted(device,st,&row,Operation::TaskRead) {
                    let mut public=user_input::public(st,row.clone());
                    public["can_respond"]=json!(public["can_respond"]==true && user_input_permitted(device,st,&row,Operation::TaskRespond));
                    let size=serde_json::to_vec(&public).map_err(|_|"invalid_request")?.len();
                    if bytes+size>120*1024 {more=true;break;} bytes+=size;items.push(public);
                }
                last=Some((row["created_at"].as_i64().unwrap_or(now),row["request_id"].as_str().unwrap_or("").to_owned()));
            }
            return Ok(json!({"status":"ok","items":items,"next_cursor":if more{last.and_then(|v|serde_json::to_string(&v).ok())}else{None}}));
        }
        let (id,writing)=match operation {
            MobileOperation::UserInputShow{request_id}=>(request_id,false),
            MobileOperation::UserInputRespond{request_id,..}=>(request_id,true),
            MobileOperation::UserInputReceipt{request_id,..}=>(request_id,false),_=>return Err("invalid_request".into())};
        let row=store::get(db,id)?.ok_or("not_found")?;
        if !user_input_permitted(device,st,&row,if writing{Operation::TaskRespond}else{Operation::TaskRead}) {return Err("not_found".into());}
        match operation {
            MobileOperation::UserInputShow{..}=>{let mut public=user_input::public(st,row.clone());public["can_respond"]=json!(public["can_respond"]==true && user_input_permitted(device,st,&row,Operation::TaskRespond));Ok(json!({"status":"ok","item":public}))},
            MobileOperation::UserInputRespond{device_id,command_id,revision,answers,..}=>{
                if device_id!=&device.device_id {return Err("command_origin_changed".into());}
                if row["state"]=="pending" && !user_input::live(st,&row) {return Err("question_not_resumable".into());}
                let current=store::respond(db,id,revision,command_id,&format!("mobile181:{}",device.device_id),answers,now)?;
                st.delegation_store.wait_epoch().bump();if let Some(n)=&st.mobile_notifier{n.changed();}
                Ok(user_input_receipt(st,current,command_id))
            },
            MobileOperation::UserInputReceipt{device_id,command_id,..}=>{
                if device_id!=&device.device_id {return Err("command_origin_changed".into());}
                if row["command_id"]!=*command_id || row["actor"]!=format!("mobile181:{}",device.device_id) {return Err("not_found".into());}
                Ok(user_input_receipt(st,row,command_id))
            },_=>Err("invalid_request".into())
        }
    })();result.unwrap_or_else(|e|fail(&e))
}
fn user_input_receipt(st:&DaemonState,row:Value,command:&str)->Value {
    json!({"status":"ok","operation_kind":"user_input_respond","command_id":command,"issue":{"kind":"accepted"},"receipt":{"item_id":row["request_id"],"state":row["state"],"application":{"confirmed":row["state"]=="applied","state":row["state"]}},"item":user_input::public(st,row)})
}
