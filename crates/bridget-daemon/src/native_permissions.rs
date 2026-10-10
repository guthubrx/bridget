//! Projection sanitaire des entrées fournisseur, sans fusion de settings maison.
use bridget_transport::protocol::{NativePermissionSnapshot,ProviderPermissions,SpawnPosture};
use crate::lifecycle::{SourceEnvironment,build_environment};
use crate::registry::AgentDefinition;
use serde_json::{Value,json};
use std::path::{Path,PathBuf};

pub(crate) const CHILD_POLICY_ENV:&str="BRIDGET_NATIVE_CHILD_POLICY";
pub(crate) const CHILD_SOURCES_ENV:&str="BRIDGET_NATIVE_PERMISSION_SOURCES";
const TRANSPARENT_GCLAUDE:&str="sha256:dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e";
fn unavailable()->String{"permission_source_unavailable".into()}
fn changed()->String{"settings_revision_changed".into()}

pub(crate) fn source_revision(path:&Path)->Result<String,String>{
    bridget_transport::protocol::permission_source_revision(path,16*1024*1024)
}

fn resolved(command:&str,env:&SourceEnvironment)->Result<PathBuf,String>{
    let path=Path::new(command);
    let candidate=if path.is_absolute(){path.to_owned()}else{
        if path.components().count()!=1{return Err(unavailable())}
        let search=env.get("PATH").ok_or_else(unavailable)?;
        std::env::split_paths(search).filter(|p|p.is_absolute()).map(|p|p.join(command)).find(|p|{
            use std::os::unix::fs::PermissionsExt;
            p.metadata().is_ok_and(|m|m.is_file()&&m.permissions().mode()&0o111!=0)
        }).ok_or_else(unavailable)?
    };
    std::fs::canonicalize(candidate).map_err(|_|unavailable())
}

fn executable_revision(path:&Path)->Result<String,String>{
    if !path.metadata().is_ok_and(|m|m.is_file()){return Err(unavailable())}
    bridget_transport::protocol::permission_source_revision(path,512*1024*1024)
}

fn selected_sources(args:&[String])->Result<Value,String>{
    let mut selected=Value::String("provider_default".into());
    let mut index=0;
    while index<args.len(){
        let arg=&args[index];
        let value=if arg=="--setting-sources"{index+=1;args.get(index).map(String::as_str)}else{arg.strip_prefix("--setting-sources=")};
        if let Some(value)=value{
            let sources:Vec<_>=value.split(',').filter(|s|!s.is_empty()).collect();
            if sources.iter().any(|s|!matches!(*s,"user"|"project"|"local"))||sources.len()>3{return Err(unavailable())}
            selected=json!(sources);
        }
        index+=1;
    }
    Ok(selected)
}

fn read_json_source(path:&Path)->Result<Value,String>{
    use std::io::Read;
    let file=std::fs::File::open(path).map_err(|_|unavailable())?;
    let mut bytes=Vec::new();file.take(16*1024*1024+1).read_to_end(&mut bytes).map_err(|_|unavailable())?;
    if bytes.len()>16*1024*1024{return Err(unavailable())}
    serde_json::from_slice(&bytes).map_err(|_|unavailable())
}

fn security_source_is_observable(path:&Path)->Result<(),String>{
    if !path.is_file(){return Ok(())}
    let value=read_json_source(path)?;
    if ["policyHelper","policyHelpers","processWrapper","hooks"].iter().any(|key|value.get(*key).is_some()){return Err(unavailable())}
    Ok(())
}

/// Références exactes du profil sélectionné. Le fournisseur reste responsable
/// de fusionner ces entrées ; Bridget ne publie que leurs chemins et révisions.
pub(crate) fn capture_context(definition:&AgentDefinition,cwd:&Path,source:&SourceEnvironment)->Result<Value,String>{
    let env=build_environment(definition,source).map_err(|_|unavailable())?;
    capture_context_with_env(definition,cwd,&env)
}

pub(crate) fn capture_context_with_env(definition:&AgentDefinition,cwd:&Path,env:&SourceEnvironment)->Result<Value,String>{
    if definition.protocol!="claude_stream_json"{return Err(unavailable())}
    if definition.args.iter().any(|arg|matches!(arg.split('=').next(),Some("--agent"|"--agents"|"--plugin-dir"))){return Err(unavailable())}
    if env.keys().any(|key|matches!(key.as_str(),"BASH_ENV"|"ENV")||key.starts_with("BASH_FUNC_")){return Err(unavailable())}
    let launcher=resolved(&definition.command,env)?;
    let launcher_revision=executable_revision(&launcher)?;
    let direct_cli=resolved("claude",env)?;
    let cli=if launcher_revision==TRANSPARENT_GCLAUDE{direct_cli}else{
        // A direct provider CLI is the command resolved by the owner's PATH.
        // An arbitrary binary launcher is not proof that flags are forwarded.
        if launcher!=direct_cli{return Err(unavailable())}
        launcher.clone()
    };
    // Une chaîne shell inconnue n'est pas une capacité de lancement prouvée.
    use std::io::Read;
    let mut head=[0u8;2];
    let n=std::fs::File::open(&cli).map_err(|_|unavailable())?.read(&mut head).map_err(|_|unavailable())?;
    if n==2&&head==*b"#!"{return Err(unavailable())}
    let config=env.get("CLAUDE_CONFIG_DIR").map(PathBuf::from)
        .or_else(||env.get("HOME").map(|home|PathBuf::from(home).join(".claude"))).ok_or_else(unavailable)?;
    let config=std::fs::canonicalize(config).map_err(|_|unavailable())?;
    let selected=selected_sources(&definition.args)?;
    let managed=PathBuf::from("/Library/Application Support/ClaudeCode");
    let mut paths=vec![("user",config.join("settings.json")),("managed",config.join("remote-settings.json")),("managed",managed.join("managed-settings.json")),("managed",managed.join("managed-settings.d"))];
    // Les politiques MDM ne sont pas assimilées aux settings fichiers.
    let home=env.get("HOME").map(PathBuf::from).ok_or_else(unavailable)?;
    for path in [PathBuf::from("/Library/Managed Preferences/com.anthropic.claudecode.plist"),home.join("Library/Preferences/com.anthropic.claudecode.plist")]{
        if path.exists(){return Err(unavailable())}paths.push(("managed",path));
    }
    for ancestor in cwd.ancestors(){
        paths.push(("project",ancestor.join(".claude/settings.json")));
        paths.push(("local",ancestor.join(".claude/settings.local.json")));
        if paths.len()>24{return Err(unavailable())}
        if ancestor.join(".git").exists(){break}
    }
    let mut overrides=None;
    let mut index=0;
    while index<definition.args.len(){
        let arg=&definition.args[index];
        let inline=if arg=="--settings"{index+=1;definition.args.get(index).map(String::as_str)}else{arg.strip_prefix("--settings=")};
        if let Some(value)=inline{
            if value.starts_with('{'){
                let parsed:Value=serde_json::from_str(value).map_err(|_|unavailable())?;
                if !parsed.as_object().is_some_and(|o|o.keys().all(|k|k=="permissions")){return Err(unavailable())}
                overrides=Some(parsed);
            }else{
                let path=PathBuf::from(value);if !path.is_absolute(){return Err(unavailable())}paths.push(("cli",path));
            }
        }
        index+=1;
    }
    let settings=config.join("settings.json");
    let settings:Value=if settings.exists(){read_json_source(&settings)?}else{Value::Null};
    if let Some(enabled)=settings.get("enabledPlugins").and_then(Value::as_object){
        let installed_path=config.join("plugins/installed_plugins.json");
        paths.push(("user",installed_path.clone()));
        let installed:Value=read_json_source(&installed_path)?;
        let mut names=enabled.iter().filter_map(|(name,value)|if value.as_bool()==Some(true){Some(name)}else{None}).collect::<Vec<_>>();names.sort();
        for name in names{
            let entries=installed["plugins"][name.as_str()].as_array().ok_or_else(unavailable)?;
            let entries=entries.iter().filter(|e|e["scope"]=="user").collect::<Vec<_>>();
            if entries.len()!=1{return Err(unavailable())}
            let path=PathBuf::from(entries[0]["installPath"].as_str().ok_or_else(unavailable)?);
            if !path.is_absolute(){return Err(unavailable())}
            let path=std::fs::canonicalize(path).map_err(|_|unavailable())?;
            if !path.starts_with(config.join("plugins/cache")){return Err(unavailable())}
            let manifest_path=path.join(".claude-plugin/plugin.json");
            let manifest:Value=read_json_source(&manifest_path)?;
            if ["hooks","settings","permissions","policyHelper","policyHelpers","processWrapper","mcpServers","lspServers"].iter().any(|key|manifest.get(*key).is_some()){return Err(unavailable())}
            paths.push(("user",manifest_path));
            for suffix in ["hooks/hooks.json","settings.json",".mcp.json"]{
                let source=path.join(suffix);if source.exists(){return Err(unavailable())}paths.push(("user",source));
            }
        }
    }
    if paths.len()>32{return Err(unavailable())}
    let mut sources=Vec::new();
    for (kind,path) in paths{
        security_source_is_observable(&path)?;
        if path!=config.join("settings.json")&&path.is_file()&&path.file_name().and_then(|name|name.to_str()).is_some_and(|name|matches!(name,"settings.json"|"settings.local.json"|"managed-settings.json"|"remote-settings.json")){
            if read_json_source(&path)?.get("enabledPlugins").is_some(){return Err(unavailable())}
        }
        if path.is_dir(){for entry in std::fs::read_dir(&path).map_err(|_|unavailable())?{security_source_is_observable(&entry.map_err(|_|unavailable())?.path())?;}}
        sources.push(json!({"kind":kind,"path":path,"revision":source_revision(&path)?}));
    }
    let mut context=json!({"cli_path":launcher,"cli_revision":launcher_revision,"resolved_cli_path":cli,"resolved_cli_revision":executable_revision(&cli)?,"config_dir":config,"settings_sources":selected,"permission_sources":sources});
    if let Some(overrides)=overrides{context["settings_overrides"]=overrides;}
    Ok(context)
}

pub(crate) fn recheck_context(context:&Value,definition:&AgentDefinition,cwd:&Path,source:&SourceEnvironment)->Result<(),String>{
    let environment=build_environment(definition,source).map_err(|_|changed())?;
    recheck_context_with_env(context,definition,cwd,&environment)
}

pub(crate) fn recheck_context_with_env(context:&Value,definition:&AgentDefinition,cwd:&Path,environment:&SourceEnvironment)->Result<(),String>{
    let captured=capture_context_with_env(definition,cwd,environment).map_err(|_|changed())?;
    // Les overrides ajoutés par le montage MCP natif ne remplacent pas les
    // entrées de droits du parent. Leurs règles restent vérifiées séparément.
    for key in ["cli_path","cli_revision","resolved_cli_path","resolved_cli_revision","config_dir","settings_sources","permission_sources"]{
        if context.get(key)!=captured.get(key){return Err(changed())}
    }
    if let Some(sources)=context["permission_sources"].as_array(){
        for source in sources{
            let path=source["path"].as_str().ok_or_else(changed)?;
            if source_revision(Path::new(path)).map_err(|_|changed())?!=source["revision"]{return Err(changed())}
        }
    }
    Ok(())
}

fn claude_inputs_without_rules(context:&Value)->Result<bool,String>{
    let sources=context["permission_sources"].as_array().ok_or_else(unavailable)?;
    for source in sources{
        let path=Path::new(source["path"].as_str().ok_or_else(unavailable)?);
        if source_revision(path)?!=source["revision"]{return Err(changed())}
        if source["revision"]=="absent"{continue}
        let settings_paths=if path.is_dir(){
            std::fs::read_dir(path).map_err(|_|unavailable())?.take(257).map(|entry|entry.map(|entry|entry.path()).map_err(|_|unavailable())).collect::<Result<Vec<_>,_>>()?
        }else if matches!(source["kind"].as_str(),Some("managed"|"cli"))||path.file_name().and_then(|p|p.to_str()).is_some_and(|name|matches!(name,"settings.json"|"settings.local.json"|"managed-settings.json"|"remote-settings.json")){
            vec![path.to_owned()]
        }else{continue};
        if settings_paths.len()>256{return Err(unavailable())}
        for settings_path in settings_paths{
            let settings=read_json_source(&settings_path)?;
            if !settings.as_object().is_some_and(|o|o.keys().all(|key|matches!(key.as_str(),"modelPricing"|"model"|"effortLevel"|"enabledPlugins"|"extraKnownMarketplaces"))){return Ok(false)}
        }
    }
    if context.get("settings_overrides").is_some(){return Ok(false)}
    Ok(true)
}

pub(crate) fn child_policy(parent:&ProviderPermissions,definition:&AgentDefinition,cwd:&Path,source:&SourceEnvironment,posture:Option<SpawnPosture>)->Result<(Value,SpawnPosture),String>{
    bridget_transport::protocol::validate_permissions(parent,parent.source=="provider_turn")?;
    let mut policy=parent.provider_policy.clone();
    let readonly=posture==Some(SpawnPosture::Discovery);
    match (parent.driver.as_str(),definition.protocol.as_str()){
        ("codex_app_server","codex_app_server")=>{
            if policy["sandbox_policy"]["type"]=="externalSandbox"{return Err("provider_confinement_unavailable".into())}
            if readonly{policy["sandbox_policy"]=json!({"type":"readOnly","networkAccess":false});}
            if posture==Some(SpawnPosture::Development)&&policy["sandbox_policy"]["type"]=="readOnly"{return Err("permission_not_inherited".into())}
            // La politique réelle est conservée. Le transport refuse toute
            // extension nécessitant une approbation ; il n'ouvre pas de UI.
        },
        ("codex_app_server","claude_stream_json")=>{
            let inherited_readonly=policy["sandbox_policy"]["type"]=="readOnly";
            if inherited_readonly&&!readonly&&policy["sandbox_policy"].get("networkAccess")!=Some(&Value::Bool(false)){return Err("permission_mapping_unavailable".into())}
            let read=readonly||inherited_readonly;
            if !read&&policy["sandbox_policy"]["type"]!="dangerFullAccess"{return Err("provider_confinement_unavailable".into())}
            if !read&&policy["approval_policy"]!="never"{return Err("permission_mapping_unavailable".into())}
            policy=json!({"kind":"claude","permission_mode":if read{"plan"}else{"bypassPermissions"},"tools":{"type":"preset","preset":"claude_code"},"permission_callback":{"kind":"native_wrapper","tool_approval":"allow","plan_exit":"deny"},"settings_sources":"provider_default","launch_context":capture_context(definition,cwd,source)?});
            if read{policy["tools"]=json!(["Read","Glob","Grep"]);policy["disallowed_tools"]=json!(["Bash","Edit","Write","NotebookEdit","WebFetch","WebSearch"]);}
        },
        ("claude_stream_json","claude_stream_json")=>{
            let context=policy.get("launch_context").ok_or("permission_source_unavailable")?;
            recheck_context(context,definition,cwd,source)?;
            if Path::new(&parent.cwd)!=cwd{return Err("permission_mapping_unavailable".into())}
            if !readonly && policy["permission_callback"]["kind"]=="t3_runtime" && policy["permission_callback"]["tool_approval"]=="allow"
                && !policy["permission_mode"].as_str().is_some_and(|m|matches!(m,"bypassPermissions"|"auto")){
                return Err("permission_mapping_unavailable".into())
            }
            if readonly{
                let available=policy["tools"].as_array();
                let tools:Vec<&str>=["Read","Glob","Grep"].into_iter().filter(|name|
                    available.is_none_or(|tools|tools.iter().any(|tool|tool.as_str()==Some(name)))
                    &&policy["disallowed_tools"].as_array().is_none_or(|tools|!tools.iter().any(|tool|tool.as_str()==Some(name)))).collect();
                policy["permission_mode"]=json!("plan");policy["tools"]=json!(tools);
                let mut denied=policy["disallowed_tools"].as_array().cloned().unwrap_or_default();
                for name in ["Bash","Edit","Write","NotebookEdit","WebFetch","WebSearch"]{
                    if !denied.iter().any(|tool|tool==name){denied.push(json!(name));}
                }
                policy["disallowed_tools"]=json!(denied);
                // Existing allowed rules remain below the tool inventory and
                // never add a tool removed by the discovery reduction.
            }else if posture==Some(SpawnPosture::Development)&&policy["permission_mode"]=="plan"{return Err("permission_not_inherited".into())}
            // The inherited inputs enforce rules in Claude; none cannot approve
            // an action which requires the parent's human mediation.
            policy["permission_callback"]["kind"]=json!("native_wrapper");
        },
        ("claude_stream_json","codex_app_server")=>{
            let context=policy.get("launch_context").ok_or("permission_source_unavailable")?;
            if !claude_inputs_without_rules(context)?||policy["disallowed_tools"].as_array().is_some_and(|a|!a.is_empty()){
                return Err("permission_mapping_unavailable".into())
            }
            let tools=&policy["tools"];
            // An empty built-in inventory must never gain Codex file tools.
            // Nonempty auto-approval rules have no exact Codex translation.
            if tools.as_array().is_some_and(Vec::is_empty)
                ||policy["allowed_tools"].as_array().is_some_and(|rules|!rules.is_empty()){
                return Err("permission_mapping_unavailable".into())
            }
            let full=policy["permission_mode"]=="bypassPermissions"&&policy["permission_callback"]["tool_approval"]=="allow"
                &&(tools["preset"]=="claude_code"||tools.as_array().is_some_and(|a|a.iter().any(|t|t=="Bash")));
            let read=readonly||(policy["permission_mode"]=="plan"&&tools.as_array().is_some_and(|a|a.iter().all(|t|t.as_str().is_some_and(|t|matches!(t,"Read"|"Glob"|"Grep")))));
            if !full&&!read{return Err("permission_mapping_unavailable".into())}
            policy=json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":if read{json!({"type":"readOnly","networkAccess":false})}else{json!({"type":"dangerFullAccess"})}});
        },
        _=>return Err("permission_mapping_unavailable".into()),
    }
    let only_read_tools=policy["tools"].as_array().is_some_and(|tools|tools.iter().all(|tool|tool.as_str().is_some_and(|name|matches!(name,"Read"|"Glob"|"Grep"))));
    let effective=if readonly||policy["sandbox_policy"]["type"]=="readOnly"||policy["permission_mode"]=="plan"||only_read_tools{SpawnPosture::Discovery}else{SpawnPosture::Development};
    if posture==Some(SpawnPosture::Development)&&effective==SpawnPosture::Discovery{return Err("permission_not_inherited".into())}
    Ok((policy,effective))
}

pub(crate) fn apply_child_arguments(protocol:&str,policy:&Value,args:&mut Vec<String>)->Result<(),String>{
    let mut kept=Vec::new();let mut index=0;
    while index<args.len(){
        let arg=&args[index];
        let flag=arg.split('=').next().unwrap_or(arg);
        if matches!(flag,"--permission-mode"|"--permission-prompts"|"--sandbox"|"--ask-for-approval") {index+=if arg.contains('='){1}else{2};continue}
        if matches!(flag,"--tools"|"--allowedTools"|"--allowed-tools"|"--disallowedTools"|"--disallowed-tools"|"--add-dir"){
            index+=1;
            if !arg.contains('='){while index<args.len()&&!args[index].starts_with('-'){index+=1;}}
            continue
        }
        if protocol=="claude_stream_json" && matches!(flag,"--setting-sources"|"--settings"){
            // Preserve a selected settings file; sanitised inline overrides are
            // rebuilt from the frozen parent policy below.
            let value=arg.split_once('=').map(|(_,v)|v).or_else(||args.get(index+1).map(String::as_str));
            if flag=="--setting-sources"||value.is_some_and(|v|v.trim_start().starts_with('{')){
                index+=if arg.contains('='){1}else{2};continue
            }
        }
        if matches!(arg.as_str(),"--dangerously-skip-permissions"|"--allow-dangerously-skip-permissions"|"--yolo"|"--restricted"|"--full-auto"){index+=1;continue}
        if arg.starts_with("--permission-mode=")||arg.starts_with("--permission-prompts="){index+=1;continue}
        if matches!(arg.as_str(),"-c"|"--config")&&args.get(index+1).is_some_and(|v|["approval_policy=","sandbox_mode=","sandbox_workspace_write."].iter().any(|k|v.starts_with(k))){index+=2;continue}
        if ["approval_policy=","sandbox_mode=","sandbox_workspace_write."].iter().any(|k|arg.starts_with(k)){index+=1;continue}
        kept.push(arg.clone());index+=1;
    }
    match protocol{
        "codex_app_server"=>{}, // Exact app-server policy accompanies each turn.
        "claude_stream_json"=>{
            if policy.pointer("/launch_context/settings_overrides").is_some()&&kept.iter().any(|arg|arg=="--settings"||arg.starts_with("--settings=")){
                return Err("permission_source_unavailable".into())
            }
            let mode=policy["permission_mode"].as_str().ok_or("permission_mapping_unavailable")?;
            kept.extend(["--permission-mode".into(),mode.into(),"--permission-prompts".into(),"none".into()]);
            // --restricted drops user/project/local settings. Preserve the
            // frozen inputs and enforce the reduction with tools and denies.
            if policy["allow_dangerously_skip_permissions"]==true{kept.push("--allow-dangerously-skip-permissions".into());}
            if let Some(tools)=policy["tools"].as_array(){kept.extend(["--tools".into(),tools.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(",")]);}
            for (field,flag) in [("allowed_tools","--allowedTools"),("disallowed_tools","--disallowedTools")]{
                if let Some(values)=policy[field].as_array(){kept.extend([flag.into(),values.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(",")]);}
            }
            if let Some(values)=policy["additional_directories"].as_array(){for value in values{kept.extend(["--add-dir".into(),value.as_str().ok_or("permission_mapping_unavailable")?.into()]);}}
            if let Some(value)=policy.pointer("/launch_context/settings_overrides"){kept.extend(["--settings".into(),serde_json::to_string(value).map_err(|_|unavailable())?]);}
            if let Some(selected)=policy.pointer("/launch_context/settings_sources").and_then(Value::as_array){kept.extend(["--setting-sources".into(),selected.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(",")]);}
        },
        _=>return Err("permission_mapping_unavailable".into()),
    }
    *args=kept;Ok(())
}

pub(crate) fn snapshot(owner:&str,instance:&str,parent:ProviderPermissions,child_policy:Value,cwd:&Path)->NativePermissionSnapshot{
    NativePermissionSnapshot{version:1,source:parent.source.clone(),owner_agent_id:owner.into(),owner_instance_id:instance.into(),parent,child_policy,project_cwd:cwd.to_string_lossy().into_owned()}
}

pub(crate) fn snapshot_contexts(snapshot:&NativePermissionSnapshot)->Vec<Value>{
    let mut contexts=Vec::new();
    for context in [snapshot.parent.provider_policy.get("launch_context"),snapshot.child_policy.get("launch_context")].into_iter().flatten(){
        if !contexts.contains(context){contexts.push(context.clone());}
    }
    contexts
}

pub(crate) fn runtime_mode(policy:&Value)->&'static str{
    match policy["permission_mode"].as_str(){Some("default"|"plan")=>"approval-required",Some("acceptEdits")=>"auto-accept-edits",Some("bypassPermissions")=>"full-access",Some("auto"|"dontAsk")=>"auto",_=>if policy["sandbox_policy"]["type"]=="dangerFullAccess"&&policy["approval_policy"]=="never"{"full-access"}else if policy["approval_policy"]=="never"{"auto"}else{"approval-required"}}
}

pub(crate) fn observed_fact(instance:&str,definition:&AgentDefinition,context:Option<&Value>,transport:&dyn bridget_transport::ManagedSession)->Result<Option<ProviderPermissions>,String>{
    let Some((session,mut policy,revision,provider_cwd))=transport.provider_permissions()else{return Ok(None)};
    let cwd=match provider_cwd{
        Some(cwd)=>PathBuf::from(cwd),
        None if definition.protocol=="claude_stream_json"=>std::env::current_dir().map_err(|_|unavailable())?,
        None=>return Err("permission_attestation_unavailable".into()),
    };
    if definition.protocol=="claude_stream_json"{
        policy["launch_context"]=context.cloned().ok_or_else(unavailable)?;
    }
    let readonly=policy["permission_mode"]=="plan"||policy["sandbox_policy"]["type"]=="readOnly";
    let runtime=runtime_mode(&policy);
    let fact=ProviderPermissions{version:1,source:"native_wrapper".into(),run_id:transport.provider_identity().and_then(|i|i.active_turn_id).unwrap_or_else(||session.clone()),provider_session_id:session,provider_instance_id:instance.into(),revision,driver:definition.protocol.clone(),cwd:cwd.to_string_lossy().into_owned(),runtime_mode:runtime.into(),interaction_mode:if readonly{"plan"}else{"default"}.into(),provider_policy:policy};
    bridget_transport::protocol::validate_permissions(&fact,false)?;
    Ok(Some(fact))
}

#[cfg(test)]
#[path = "native_permissions149_tests.rs"]
mod native_permissions149_tests;
