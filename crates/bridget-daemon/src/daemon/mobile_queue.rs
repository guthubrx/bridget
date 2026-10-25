//! File mobile: mêmes remises natives, portée du téléphone vérifiée avant toute lecture.
use super::*;
use crate::mobile::store::{Device, Operation};
use bridget_transport::protocol::{MobileOperation};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn response(result: Value) -> DaemonToWrapper { DaemonToWrapper::MobileOperationResult { result } }
fn failure(code: &str) -> DaemonToWrapper { response(json!({"status":"refused","code":code})) }
fn digest(body: &str) -> String { format!("{:x}", Sha256::digest(body.as_bytes())) }
fn scope(device: &Device) -> String { format!("mobile177_{}",device.device_id.replace('-',"")) }

/// O(log N + H*B), H <= 32 messages retenus, B <= 10 000 octets.
pub(super) fn handle(conn: &str, operation: &MobileOperation, device: &Device, st: &mut DaemonState) -> DaemonToWrapper {
    let agent = match operation {
        MobileOperation::MessageQueue {agent_id} | MobileOperation::MessageQueueChange {agent_id,..} => agent_id,
        _ => return failure("invalid_request"),
    };
    if Uuid::parse_str(agent).is_err() { return failure("invalid_request"); }
    let Some((owner,instance))=st.router.get_agent(agent).and_then(|r|live_connection_identity(st,&r.connection_id)) else { return failure("not_found"); };
    if owner!=*agent {return failure("not_found");}
    let Some(p)=st.presences.get(&instance) else { return failure("not_found"); };
    if !device.scope.permits(Operation::Message,&p.host,agent_communication_project(st,agent).as_ref()) {
        return failure("not_found");
    }
    let issuer=scope(device);
    if let MobileOperation::MessageQueue {..}=operation {
        let read = || -> Result<Vec<Value>,String> {
            let mut stmt=st.store.connection().prepare("SELECT idempotency_key,message_bytes FROM send_deliveries
                WHERE issuer_scope=?1 AND recipient_agent=?2 AND phase='held' ORDER BY queue_rank LIMIT 32").map_err(|e|e.to_string())?;
            let rows=stmt.query_map(params![issuer,agent],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Vec<u8>>(1)?))).map_err(|e|e.to_string())?;
            rows.map(|row| { let (id,bytes)=row.map_err(|e|e.to_string())?;
                let m:bridget_core::BridgetMessage=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
                Ok(json!({"command_id":id,"body":m.body,"revision":digest(&m.body),"state":"held"}))
            }).collect()
        };
        return match read() { Ok(items)=>{
                let phases=||->rusqlite::Result<Vec<Value>> {
                    st.store.connection().prepare("SELECT idempotency_key,phase FROM send_deliveries WHERE issuer_scope=?1 AND recipient_agent=?2 ORDER BY rowid DESC LIMIT 256")?
                        .query_map(params![issuer,agent],|r|Ok(json!({"command_id":r.get::<_,String>(0)?,"phase":r.get::<_,String>(1)?})))?.collect()
                };
                match phases() {Ok(receipts)=>response(json!({"status":"ok","agent_id":agent,"items":items,"receipts":receipts})),Err(_)=>failure("storage_unavailable")}
            },Err(_)=>failure("storage_unavailable") };
    }
    let MobileOperation::MessageQueueChange {command_id,request_id,change:action,expected_body,body,..}=operation else { unreachable!() };
    if ![command_id,request_id].into_iter().all(|id|Uuid::parse_str(id).is_ok_and(|u|u.get_version_num()==4 && u.to_string()==*id))
        || expected_body.len()!=64 || !expected_body.bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !matches!(action.as_str(),"edit"|"cancel"|"promote")
        || (action=="edit")!=body.is_some()
        || body.as_ref().is_some_and(|b|b.trim().is_empty() || b.len()>10_000 || b.contains('\0')) {
        return failure("invalid_request");
    }
    let key=match IdempotencyKey::new(issuer.clone(),OperationKind::Send,command_id.clone()) {Ok(key)=>key,Err(_)=>return failure("invalid_request")};
    // La possession du reçu d'un autre téléphone n'accorde jamais de droit.
    let delivery=match st.idempotency.stored_send_delivery(&key) { Ok(Some(d))=>d,_=>return failure("not_found") };
    let bytes:Vec<u8>=match st.store.connection().query_row("SELECT message_bytes FROM send_deliveries WHERE delivery_id=?1 AND issuer_scope=?2",params![delivery.delivery_id,issuer],|r|r.get(0)) {
        Ok(bytes)=>bytes,Err(_)=>return failure("storage_unavailable")
    };
    let original:bridget_core::BridgetMessage=match serde_json::from_slice(&bytes) {Ok(m)=>m,Err(_)=>return failure("storage_unavailable")};
    if original.to!=*agent || original.id!=*command_id || original.from!="human" { return failure("not_found"); }
    let canon=serde_json::to_vec(&json!({"agent":agent,"message":command_id,"action":action,"expected":expected_body,"body":body})).expect("JSON values");
    let other_table=if action=="edit" {"queue_edits"} else {"mobile_queue_replacements"};
    let collision=st.store.connection().query_row(&format!("SELECT EXISTS(SELECT 1 FROM {other_table} WHERE issuer_scope=?1 AND request_id=?2)"),
        params![issuer,request_id],|r|r.get::<_,bool>(0));
    match collision {Ok(true)=>return failure("envelope_mismatch"),Err(_)=>return failure("storage_unavailable"),_=>{}}
    if action=="edit" {
        return match replace(st,&issuer,request_id,&canon,&delivery.delivery_id,expected_body,body.as_deref().unwrap()) {
            Ok(value)=>response(value),Err(_)=>failure("storage_unavailable")
        };
    }
    // Rejeu d'une mutation déjà appliquée permis ; une nouvelle mutation exige le texte courant.
    let replay:bool=st.store.connection().query_row("SELECT EXISTS(SELECT 1 FROM queue_edits WHERE issuer_scope=?1 AND request_id=?2)",params![issuer,request_id],|r|r.get(0)).unwrap_or(false);
    if !replay && digest(&original.body)!=*expected_body { return failure("queue_changed"); }
    let now=unix_now_secs();
    let mut controls=Vec::new();
    let result=if action=="cancel" {
        st.idempotency.apply_cancel(&issuer,request_id,&canon,"human",agent,command_id,now).map(|(receipt,notice)|(receipt,notice,None))
    } else {
        let mut facts=match guidance_facts(st,Some(conn),agent,crate::guidance::GuidanceAction::QueuePromote,
            crate::guidance::GuidancePath::Cancel,Some(bridget_core::MessageOrigin::Human),false,now) {
                Ok(f)=>f,Err(_)=>return failure("storage_unavailable")
            };
        facts.actor=crate::guidance::GuidanceActor::AttachedHuman; facts.identity_attested=true;
        facts.recipient_turn_active=p.turn_attest && p.state=="busy" && p.last_attested_at.is_some_and(|at|at.elapsed()<=Duration::from_secs(90));
        facts.carrier_queue_fallback=p.queue_fallback;
        st.idempotency.apply_promote(&issuer,request_id,&canon,"human",agent,command_id,&facts,queue_edit_refusal_code,now)
            .map(|(receipt,delivery,notice)|(receipt,notice,delivery))
    };
    match result {
        Ok((receipt,notice,promoted))=>{
            if notice.is_some() || promoted.is_some() {st.held_ids.remove(command_id);}
            if let Some(mut d)=promoted {
                let mut message=original;
                message.intent=Some(bridget_core::MessageIntent::SteerCurrent);
                message.references.push("steer-or-queue".into());
                d.message_bytes=serde_json::to_vec(&message).expect("stored message");
                if let Some(target)=st.router.get_agent(agent).map(|r|r.connection_id.clone()) {
                    if defer_idempotent_delivery(st,&target,d.clone(),&mut controls,None,Some(conn)).is_err() {
                        let _=st.idempotency.mark_delivery_indeterminate(&d.delivery_id,&d.recipient_instance_id,d.delivery_generation);
                    }
                }
                st.pending_post_response_controls.entry(conn.into()).or_default().extend(controls);
            }
            response(json!({"status":"ok","request_id":request_id,"state":receipt.state,"refusal":receipt.refusal,"effect":receipt.effect}))
        },
        Err(_)=>failure("storage_unavailable"),
    }
}

/// O(log E + log N + B), canon de création conservé ; phase et révision relues sous transaction.
fn replace(st:&DaemonState,issuer:&str,request:&str,canon:&[u8],delivery:&str,expected:&str,body:&str)->rusqlite::Result<Value> {
    let tx=rusqlite::Transaction::new_unchecked(st.store.connection(),rusqlite::TransactionBehavior::Immediate)?;
    let previous:Option<(Vec<u8>,String)>=tx.query_row("SELECT canonical_bytes,result FROM mobile_queue_replacements WHERE issuer_scope=?1 AND request_id=?2",
        params![issuer,request],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if let Some((bytes,result))=previous {return Ok(if bytes==canon {serde_json::from_str(&result).unwrap_or(json!({"status":"refused","code":"storage_unavailable"}))} else {json!({"status":"refused","code":"envelope_mismatch"})});}
    let row:Option<(String,Vec<u8>,i64)>=tx.query_row("SELECT phase,message_bytes,expires_at FROM send_deliveries WHERE delivery_id=?1 AND issuer_scope=?2",
        params![delivery,issuer],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
    let result=match row {
        Some((phase,bytes,expires)) if phase=="held" && expires>unix_now_secs()=>{
            let mut m:Value=serde_json::from_slice(&bytes).map_err(|_|rusqlite::Error::InvalidQuery)?;
            if m["body"].as_str().map(digest).as_deref()!=Some(expected) {
                json!({"status":"refused","code":"queue_changed"})
            } else {
                m["body"]=json!(body);
                tx.execute("UPDATE send_deliveries SET message_bytes=?1 WHERE delivery_id=?2 AND phase='held'",params![serde_json::to_vec(&m).unwrap(),delivery])?;
                tx.execute("UPDATE ledger SET body=?1 WHERE id=?2 AND target=?3 AND sender='human'",params![body,m["id"].as_str(),m["to"].as_str()])?;
                json!({"status":"ok","request_id":request,"state":"applied"})
            }
        },
        _=>json!({"status":"refused","code":"not_held"}),
    };
    tx.execute("DELETE FROM mobile_queue_replacements WHERE expires_at<=?1",[unix_now_secs()])?;
    tx.execute("INSERT INTO mobile_queue_replacements VALUES(?1,?2,?3,?4,?5)",params![issuer,request,canon,result.to_string(),unix_now_secs()+7*86400])?;
    tx.commit()?;
    Ok(result)
}
