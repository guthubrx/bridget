//! Pages de l'annuaire MCP. Filtrage de présentation seulement : l'annuaire
//! natif authentifié continue de fixer le périmètre visible avant pagination.
use super::*;
use base64::{Engine as _,engine::general_purpose::URL_SAFE_NO_PAD};
use bridget_transport::protocol::{ScopedAgentInfo,CommunicationDirectoryScope};
use serde::{Serialize,Deserialize};
use sha2::{Sha256,Digest};
pub(super) const MAX_PAGE:usize=24*1024;
#[derive(Debug)]
pub(super) struct Options { limit:usize, stopped:bool, cursor:Option<String> }
fn invalid(s:&str)->ToolError {ToolError::InvalidParams(s.into())}
impl Options {
    pub(super) fn parse(a:&serde_json::Map<String,Value>)->Result<Self,ToolError>{
        let limit=match a.get("limit") {None=>20,Some(v)=>v.as_u64().filter(|v|(1..=100).contains(v))
            .ok_or_else(||invalid("limit doit être un entier de 1 à 100"))? as usize};
        let stopped=match a.get("include_stopped"){None=>false,Some(Value::Bool(v))=>*v,
            _=>return Err(invalid("include_stopped doit être un booléen"))};
        let cursor=match a.get("cursor"){None=>None,Some(Value::String(s)) if !s.is_empty()&&s.len()<=1024=>Some(s.clone()),
            _=>return Err(invalid("cursor doit être le next_cursor reçu, au plus 1024 octets"))};
        if let Some(Value::String(s))=a.get("domain") {
            if s.len()>256 || s.chars().any(char::is_control) {return Err(invalid("domain doit rester borné à 256 octets sans contrôle"));}
        }
        Ok(Self{limit,stopped,cursor})
    }
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Cursor {version:u8,query:String,after:String}
fn encode(after:&str,query:&str)->String {
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor{version:1,query:query.into(),after:after.into()}).unwrap())
}
pub(super) fn page(mut agents:Vec<ScopedAgentInfo>,o:&Options,domain:Option<&str>,
    scope:CommunicationDirectoryScope,mut base:Value)->Result<Value,ToolError>{
    let query=format!("{:x}",Sha256::digest(serde_json::to_vec(&(scope,domain,o.stopped)).unwrap()));
    let after=match &o.cursor {
        None=>None,
        Some(text)=>{
            let bytes=URL_SAFE_NO_PAD.decode(text).map_err(|_|invalid("who_cursor_invalid : recopier next_cursor"))?;
            let c:Cursor=serde_json::from_slice(&bytes).map_err(|_|invalid("who_cursor_invalid : recopier next_cursor"))?;
            if c.version!=1 || c.query!=query || c.after.is_empty() || c.after.len()>256
                || c.after.chars().any(char::is_control){
                return Err(invalid("who_cursor_invalid : conserver scope, domain et include_stopped de la page précédente"));
            }
            Some(c.after)
        }
    };
    agents.retain(|a| (o.stopped||a.agent.state!="stopped")
        && domain.is_none_or(|d|a.agent.domain.as_deref()==Some(d)));
    agents.sort_by(|a,b|a.agent.agent_id.cmp(&b.agent.agent_id));
    base["total_matching"]=json!(agents.len());base["live"]=json!(true);
    let mut selected=Vec::new();
    let mut last=None;
    let mut more=false;
    for a in agents {
        if after.as_deref().is_some_and(|id|a.agent.agent_id.as_str()<=id){continue;}
        if selected.len()>=o.limit {more=true;break;}
        let id=a.agent.agent_id.clone();
        selected.push(serde_json::to_value(a).map_err(|_|invalid("who_entry_unavailable"))?);
        base["agents"]=json!(selected);
        // Réserver dès maintenant le plus grand en-tête de continuation.
        base["has_more"]=json!(true);base["next_cursor"]=json!(encode(&id,&query));
        if serde_json::to_vec(&base).map_err(|_|invalid("who_page_unavailable"))?.len()>MAX_PAGE {
            selected.pop();
            if selected.is_empty(){return Err(invalid("who_entry_too_large : une présence dépasse le budget de 24 Kio. Transmettre ce refus au responsable Bridget ; aucun élément n’a été tronqué."));}
            more=true;break;
        }
        last=Some(id);
    }
    base["agents"]=json!(selected);base["has_more"]=json!(more);
    base["next_cursor"]=if more{json!(encode(last.as_deref().unwrap(),&query))}else{Value::Null};
    if serde_json::to_vec(&base).map_err(|_|invalid("who_page_unavailable"))?.len()>MAX_PAGE{
        return Err(invalid("who_page_too_large : métadonnées de l’annuaire hors budget, transmettre au responsable Bridget"));
    }
    Ok(base)
}
#[cfg(test)]
mod tests {
    use super::*;
    use bridget_transport::protocol::ProjectRelation;
    fn agents(n:usize,state:&str,size:usize)->Vec<ScopedAgentInfo>{
        (0..n).map(|i|ScopedAgentInfo{
            agent:serde_json::from_value(json!({"agent_id":format!("{state}-{i:06}"),
                "display_name":"x".repeat(size),"agent_type":"codex","connection_id":"conn",
                "host":"fixture","transport":"codex_app_server","state":state,
                "last_seen_secs":0,"reconnect_count":0,"domain":"fixture","persistent":false})).unwrap(),
            communication_project:None,project_relation:ProjectRelation::Unknown,
        }).collect()
    }
    fn base()->Value{json!({"scope":"global","project":null,"project_warnings":[]})}
    #[test]
    fn spec211_who_filters_history_and_pages_without_loss(){
        let mut data=agents(1200,"stopped",100);data.extend(agents(51,"connected",100));
        let mut args=json!({"limit":20});let mut ids=Vec::new();
        loop{
            let o=Options::parse(args.as_object().unwrap()).unwrap();
            let p=page(data.clone(),&o,None,CommunicationDirectoryScope::Global,base()).unwrap();
            assert!(serde_json::to_vec(&p).unwrap().len()<=MAX_PAGE);
            assert_eq!(p["total_matching"],51);
            for a in p["agents"].as_array().unwrap(){ids.push(a["agent_id"].as_str().unwrap().to_string());}
            if p["has_more"]==false{break;}
            args["cursor"]=p["next_cursor"].clone();
        }
        assert_eq!(ids.len(),51);ids.sort();ids.dedup();assert_eq!(ids.len(),51);
        let o=Options::parse(json!({"include_stopped":true}).as_object().unwrap()).unwrap();
        assert_eq!(page(data,&o,None,CommunicationDirectoryScope::Global,base()).unwrap()["total_matching"],1251);
    }
    #[test]
    fn spec211_who_cursor_filters_and_byte_budget_are_checked(){
        let o=Options::parse(json!({"limit":100}).as_object().unwrap()).unwrap();
        let p=page(agents(100,"connected",1000),&o,None,CommunicationDirectoryScope::Global,base()).unwrap();
        assert_eq!(p["has_more"],true);assert!(p["agents"].as_array().unwrap().len()<100);
        assert!(serde_json::to_vec(&p).unwrap().len()<=MAX_PAGE);
        let q=Options::parse(json!({"cursor":p["next_cursor"]}).as_object().unwrap()).unwrap();
        assert!(page(agents(100,"connected",1000),&q,Some("other"),CommunicationDirectoryScope::Global,base()).is_err());
        assert!(page(agents(1,"connected",MAX_PAGE),&o,None,CommunicationDirectoryScope::Global,base()).is_err());
        for a in [json!({"limit":0}),json!({"limit":101}),json!({"include_stopped":null}),json!({"cursor":""})]{
            assert!(Options::parse(a.as_object().unwrap()).is_err());
        }
    }
}
