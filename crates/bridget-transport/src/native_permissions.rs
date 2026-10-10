//! Faits sanitaires de droits. Aucun credential ne rejoint un snapshot durable.
use serde::{Deserialize,Serialize};
use serde_json::Value;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
pub struct ProviderPermissions {
    pub version:u16,
    pub source:String,
    pub run_id:String,
    pub provider_session_id:String,
    pub provider_instance_id:String,
    pub revision:u64,
    pub driver:String,
    pub cwd:String,
    pub runtime_mode:String,
    pub interaction_mode:String,
    pub provider_policy:Value,
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
#[serde(deny_unknown_fields)]
pub struct NativePermissionSnapshot {
    pub version:u16,
    pub source:String,
    pub owner_agent_id:String,
    pub owner_instance_id:String,
    pub parent:ProviderPermissions,
    pub child_policy:Value,
    pub project_cwd:String,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePermissionProof {
    pub endpoint:String,
    pub authorization:super::IdentityCredential,
}

pub fn validate_permissions(fact:&ProviderPermissions,t3:bool)->Result<(),String> {
    let fail=||"permission_attestation_unavailable".to_string();
    let atom=|s:&str,max:usize|!s.is_empty()&&s.len()<=max&&!s.chars().any(bridget_core::is_disallowed_control);
    if fact.version!=1 || fact.revision==0 || fact.revision>super::HUMAN_LINEAGE_MAX_SEQ
        || fact.source!=if t3 {"provider_turn"}else{"native_wrapper"}
        || [&fact.run_id,&fact.provider_session_id,&fact.provider_instance_id].iter().any(|s|!atom(s,256))
        || !atom(&fact.cwd,4096)||!std::path::Path::new(&fact.cwd).is_absolute()
        || !matches!(fact.runtime_mode.as_str(),"approval-required"|"auto-accept-edits"|"auto"|"full-access")
        || !matches!(fact.interaction_mode.as_str(),"default"|"plan") {return Err(fail())}
    let policy=&fact.provider_policy;
    let keys=|v:&Value,required:&[&str],optional:&[&str]|->bool{
        v.as_object().is_some_and(|o|required.iter().all(|k|o.contains_key(*k))&&o.keys().all(|k|required.contains(&k.as_str())||optional.contains(&k.as_str())))
    };
    let array=|v:&Value,paths:bool|v.as_array().is_some_and(|a|a.len()<=128&&a.iter().all(|v|v.as_str().is_some_and(|s|atom(s,if paths{4096}else{512})&&(!paths||std::path::Path::new(s).is_absolute()))));
    match fact.driver.as_str(){
        "codex_app_server"=>{
            if policy["kind"]!="codex"||!keys(policy,&["kind","approval_policy","approvals_reviewer","sandbox_policy"],&[]){return Err(fail())}
            let approval=&policy["approval_policy"];
            if !approval.as_str().is_some_and(|s|matches!(s,"untrusted"|"on-request"|"never")) {
                if !keys(approval,&["granular"],&[])||!keys(&approval["granular"],&["mcp_elicitations","rules","sandbox_approval"],&["request_permissions","skill_approval"])
                    || approval["granular"].as_object().is_none_or(|o|o.values().any(|v|!v.is_boolean())){return Err(fail())}
            }
            if !policy["approvals_reviewer"].as_str().is_some_and(|s|matches!(s,"user"|"auto_review"|"guardian_subagent")){return Err(fail())}
            let sandbox=&policy["sandbox_policy"];
            match sandbox["type"].as_str(){
                Some("dangerFullAccess") if keys(sandbox,&["type"],&[])=>{},
                Some("readOnly") if keys(sandbox,&["type"],&["networkAccess"])&&sandbox.get("networkAccess").is_none_or(Value::is_boolean)=>{},
                Some("workspaceWrite") if keys(sandbox,&["type"],&["networkAccess","writableRoots","excludeSlashTmp","excludeTmpdirEnvVar"])
                    && ["networkAccess","excludeSlashTmp","excludeTmpdirEnvVar"].iter().all(|k|sandbox.get(k).is_none_or(Value::is_boolean))
                    && sandbox.get("writableRoots").is_none_or(|v|array(v,true))=>{},
                Some("externalSandbox") if keys(sandbox,&["type"],&["networkAccess"])&&sandbox.get("networkAccess").is_none_or(|v|v.as_str().is_some_and(|s|matches!(s,"restricted"|"enabled")))=>{},
                _=>return Err(fail()),
            }
        },
        "claude_stream_json"=>{
            if policy["kind"]!="claude"||!keys(policy,&["kind","permission_mode","tools","permission_callback","settings_sources"],&["allowed_tools","disallowed_tools","additional_directories","allow_dangerously_skip_permissions","launch_context"]){return Err(fail())}
            if !policy["permission_mode"].as_str().is_some_and(|s|matches!(s,"default"|"acceptEdits"|"bypassPermissions"|"plan"|"dontAsk"|"auto")) {return Err(fail())}
            if !(array(&policy["tools"],false)||(keys(&policy["tools"],&["type","preset"],&[])&&policy["tools"]["type"]=="preset"&&policy["tools"]["preset"]=="claude_code")){return Err(fail())}
            for (field,paths) in [("allowed_tools",false),("disallowed_tools",false),("additional_directories",true)] {
                if policy.get(field).is_some_and(|v|!array(v,paths)){return Err(fail())}
            }
            if policy.get("allow_dangerously_skip_permissions").is_some_and(|v|!v.is_boolean())||policy["settings_sources"]!="provider_default" {return Err(fail())}
            let callback=&policy["permission_callback"];
            if !keys(callback,&["kind","tool_approval","plan_exit"],&[])||callback["kind"]!=if t3 {"t3_runtime"}else{"native_wrapper"}
                || !callback["tool_approval"].as_str().is_some_and(|s|matches!(s,"prompt"|"allow"))||callback["plan_exit"]!="deny" {return Err(fail())}
            if let Some(context)=policy.get("launch_context") {
                if !keys(context,&["cli_path","cli_revision","resolved_cli_path","resolved_cli_revision","config_dir","settings_sources","permission_sources"],&["settings_overrides"]){return Err(fail())}
                for field in ["cli_path","resolved_cli_path","config_dir"] {if !context[field].as_str().is_some_and(|s|atom(s,4096)&&std::path::Path::new(s).is_absolute()){return Err(fail())}}
                let digest=|v:&Value|v.as_str().is_some_and(|s|s.len()==71&&s.starts_with("sha256:")&&s[7..].bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)));
                if !digest(&context["cli_revision"])||!digest(&context["resolved_cli_revision"]){return Err(fail())}
                if context["settings_sources"]!="provider_default"&&!context["settings_sources"].as_array().is_some_and(|a|a.len()<=3&&a.iter().all(|v|v.as_str().is_some_and(|s|matches!(s,"user"|"project"|"local")))&&a.iter().enumerate().all(|(i,v)|!a[..i].contains(v))) {return Err(fail())}
                if !context["permission_sources"].as_array().is_some_and(|a|a.len()<=32&&a.iter().all(|s|keys(s,&["kind","path","revision"],&[])&&s["kind"].as_str().is_some_and(|s|matches!(s,"user"|"project"|"local"|"managed"|"cli"))&&s["path"].as_str().is_some_and(|p|atom(p,4096)&&std::path::Path::new(p).is_absolute())&&(s["revision"]=="absent"||digest(&s["revision"])))){return Err(fail())}
                if let Some(overrides)=context.get("settings_overrides") {
                    if !keys(overrides,&[],&["permissions"]) {return Err("permission_source_unavailable".into())}
                    if let Some(p)=overrides.get("permissions") {
                        if !keys(p,&[],&["allow","ask","deny","defaultMode","additionalDirectories","disableBypassPermissionsMode"])
                            || ["allow","ask","deny"].iter().any(|key|p.get(key).is_some_and(|v|!array(v,false)))
                            || p.get("additionalDirectories").is_some_and(|v|!array(v,true))
                            || p.get("defaultMode").is_some_and(|v|!v.as_str().is_some_and(|s|matches!(s,"default"|"acceptEdits"|"bypassPermissions"|"plan"|"dontAsk"|"auto")))
                            || p.get("disableBypassPermissionsMode").is_some_and(|v|v!="disable"){return Err("permission_source_unavailable".into())}
                    }
                }
            }
        },
        _=>return Err(fail()),
    }
    if serde_json::to_vec(fact).map_err(|_|fail())?.len()>64*1024 {return Err(fail())}
    Ok(())
}

/// Reprise fournisseur interne : les entrées figées sont recontrôlées avant
/// l'effet, même si le transport remplace lui-même un processus de reprise.
pub fn recheck_frozen_inputs(command:&str,environment:&[(String,String)])->Result<(),String>{
    use std::path::{Path,PathBuf};
    if let Ok(raw)=std::env::var("BRIDGET_NATIVE_PERMISSION_SOURCES"){
        let contexts:Vec<Value>=serde_json::from_str(&raw).map_err(|_|"permission_attestation_unavailable")?;
        if contexts.len()>2{return Err("permission_attestation_unavailable".into())}
        for context in contexts{recheck_permission_context_sources(&context)?;}
    }
    let raw=std::env::var("BRIDGET_NATIVE_CHILD_POLICY").ok();
    let Some(raw)=raw else{return Ok(())};
    let policy:Value=serde_json::from_str(&raw).map_err(|_|"permission_attestation_unavailable")?;
    let Some(context)=policy.get("launch_context")else{return Ok(())};
    let changed=||"settings_revision_changed".to_string();
    let path=environment.iter().find(|(name,_)|name=="PATH").map(|(_,value)|value.clone()).or_else(||std::env::var("PATH").ok()).ok_or_else(changed)?;
    let resolve=|name:&str|->Result<PathBuf,String>{
        let p=Path::new(name);
        let target=if p.is_absolute(){p.to_owned()}else{
            if p.components().count()!=1{return Err(changed())}
            use std::os::unix::fs::PermissionsExt;
            std::env::split_paths(&path).filter(|p|p.is_absolute()).map(|p|p.join(name)).find(|p|p.metadata().is_ok_and(|m|m.is_file()&&m.permissions().mode()&0o111!=0)).ok_or_else(changed)?
        };
        std::fs::canonicalize(target).map_err(|_|changed())
    };
    let launcher=resolve(command)?;
    if launcher.to_str()!=context["cli_path"].as_str()||permission_source_revision(&launcher,512*1024*1024).map_err(|_|changed())?!=context["cli_revision"]{return Err(changed())}
    let target=if context["cli_revision"]=="sha256:dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e"{resolve("claude")?}else{launcher};
    if target.to_str()!=context["resolved_cli_path"].as_str()||permission_source_revision(&target,512*1024*1024).map_err(|_|changed())?!=context["resolved_cli_revision"]{return Err(changed())}
    for source in context["permission_sources"].as_array().ok_or_else(changed)?{
        let path=source["path"].as_str().ok_or_else(changed)?;
        if permission_source_revision(Path::new(path),16*1024*1024).map_err(|_|changed())?!=source["revision"]{return Err(changed())}
    }
    Ok(())
}

pub fn recheck_permission_context_sources(context:&Value)->Result<(),String>{
    use std::path::Path;
    let changed=||"settings_revision_changed".to_string();
    for (path,expected) in [("cli_path","cli_revision"),("resolved_cli_path","resolved_cli_revision")]{
        if permission_source_revision(Path::new(context[path].as_str().ok_or_else(changed)?),512*1024*1024).map_err(|_|changed())?!=context[expected]{return Err(changed())}
    }
    let sources=context["permission_sources"].as_array().filter(|s|s.len()<=32).ok_or_else(changed)?;
    for source in sources{if permission_source_revision(Path::new(source["path"].as_str().ok_or_else(changed)?),16*1024*1024).map_err(|_|changed())?!=source["revision"]{return Err(changed())}}
    Ok(())
}

/// Options de droits du CLI réellement lancé. Aucun JSON MCP ne les fournit.
pub fn claude_launch_policy_options(args:&[String],policy:&mut Value)->Result<(),String>{
    let mut index=0;
    while index<args.len(){
        let arg=&args[index];
        let (flag,inline)=arg.split_once('=').map(|(f,v)|(f,Some(v))).unwrap_or((arg.as_str(),None));
        let field=match flag{"--tools"=>Some("tools"),"--allowedTools"|"--allowed-tools"=>Some("allowed_tools"),"--disallowedTools"|"--disallowed-tools"=>Some("disallowed_tools"),"--add-dir"=>Some("additional_directories"),_=>None};
        if let Some(field)=field{
            let mut values=Vec::new();
            if let Some(value)=inline{values.extend(value.split(',').filter(|s|!s.is_empty()).map(str::to_owned));}
            else{
                index+=1;
                while index<args.len()&&!args[index].starts_with('-'){
                    values.extend(args[index].split(',').filter(|s|!s.is_empty()).map(str::to_owned));index+=1;
                }
                index=index.saturating_sub(1);
            }
            if values.len()>128||values.iter().any(|v|v.len()>if field=="additional_directories"{4096}else{512}||v.chars().any(bridget_core::is_disallowed_control)
                ||(field=="additional_directories"&&!std::path::Path::new(v).is_absolute())){return Err("permission_source_unavailable".into())}
            if field=="tools"&&values==["default"]{policy[field]=serde_json::json!({"type":"preset","preset":"claude_code"});}
            else{policy[field]=serde_json::json!(values);}
        }else if flag=="--allow-dangerously-skip-permissions"{policy["allow_dangerously_skip_permissions"]=Value::Bool(true);}
        index+=1;
    }
    Ok(())
}

pub fn permission_source_revision(path:&std::path::Path,max_bytes:u64)->Result<String,String>{
    use sha2::{Digest,Sha256};
    use std::io::Read;
    let unavailable=||"permission_source_unavailable".to_string();
    let metadata=match std::fs::symlink_metadata(path){Ok(m)=>m,Err(e)if e.kind()==std::io::ErrorKind::NotFound=>return Ok("absent".into()),Err(_)=>return Err(unavailable())};
    if metadata.file_type().is_symlink(){return Err(unavailable())}
    if metadata.is_dir(){
        let mut entries=Vec::new();
        for entry in std::fs::read_dir(path).map_err(|_|unavailable())?{
            let entry=entry.map_err(|_|unavailable())?;let name=entry.file_name().into_string().map_err(|_|unavailable())?;
            if entries.len()>=256||name.chars().any(bridget_core::is_disallowed_control)||!entry.file_type().map_err(|_|unavailable())?.is_file(){return Err(unavailable())}
            entries.push((name,permission_source_revision(&entry.path(),max_bytes)?));
        }
        entries.sort_by(|a,b|a.0.cmp(&b.0));
        let values=entries.into_iter().map(|(name,revision)|serde_json::json!({"name":name,"revision":revision})).collect::<Vec<_>>();
        return Ok(format!("sha256:{:x}",Sha256::digest(serde_json::to_vec(&values).map_err(|_|unavailable())?)))
    }
    if !metadata.is_file()||metadata.len()>max_bytes{return Err(unavailable())}
    let mut file=std::fs::File::open(path).map_err(|_|unavailable())?;let mut digest=Sha256::new();let mut bytes=[0u8;65536];let mut consumed=0u64;
    loop{let n=file.read(&mut bytes).map_err(|_|unavailable())?;if n==0{break}consumed=consumed.saturating_add(n as u64);if consumed>max_bytes{return Err(unavailable())}digest.update(&bytes[..n]);}
    Ok(format!("sha256:{:x}",digest.finalize()))
}
