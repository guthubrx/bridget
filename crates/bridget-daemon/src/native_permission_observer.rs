//! Observer temporaire du CLI Claude détenu. Il ne décide aucun droit.
use crate::native_permissions;
use crate::registry::AgentDefinition;
use bridget_transport::protocol::{ProviderPermissions,WrapperToDaemon};
use serde_json::{Value,json};
use std::io::{BufRead,BufReader,Read,Write};
use std::os::unix::net::{UnixListener,UnixStream};
use std::path::{Path,PathBuf};
use std::sync::{Arc,Mutex,Condvar};
use std::sync::atomic::{AtomicBool,Ordering};
use std::time::{Duration,Instant};

pub(crate) type Acknowledgements=Arc<(Mutex<Option<(String,bool)>>,Condvar)>;
pub(crate) struct Observer{
    socket:PathBuf,
    overlay:PathBuf,
    stop:Arc<AtomicBool>,
    thread:Option<std::thread::JoinHandle<()>>,
    acknowledgements:Acknowledgements,
    provider:Arc<Mutex<Option<(u32,u64)>>>,
}

impl Observer{
    pub(crate) fn start(root:&Path,definition:&AgentDefinition,args:&[String],instance:&str,
        emit:Arc<dyn Fn(WrapperToDaemon)+Send+Sync>)->Result<Self,String>{
        use std::os::unix::fs::{OpenOptionsExt,PermissionsExt};
        let id=uuid::Uuid::new_v4().simple().to_string();
        let socket=root.join(format!("np-{}.sock",&id[..12]));
        let overlay=root.join(format!("no-{}.json",&id[..12]));
        let nonce=format!("{}{}",uuid::Uuid::new_v4().simple(),uuid::Uuid::new_v4().simple());
        let mut launch=definition.clone();launch.args=args.to_vec();
        // Un deuxième --settings remplacerait la source CLI sélectionnée.
        // Ce contexte exige une composition fournisseur attestée séparément.
        if args.iter().any(|arg|arg=="--settings"||arg.starts_with("--settings=")){return Err("permission_source_unavailable".into())}
        let cwd=std::env::current_dir().map_err(|_|"permission_source_unavailable")?;
        let mut env=crate::lifecycle::source_environment();
        if let Some(profile)=&definition.claude_config_dir{env.insert("CLAUDE_CONFIG_DIR".into(),profile.into());}
        let context=native_permissions::capture_context_with_env(&launch,&cwd,&env)?;
        let executable=std::env::current_exe().map_err(|_|"permission_source_unavailable")?;
        let listener=UnixListener::bind(&socket).map_err(|_|"permission_source_unavailable")?;
        let mut setup=OverlaySetup{socket:socket.clone(),overlay:overlay.clone(),armed:true};
        std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).map_err(|_|"permission_source_unavailable")?;
        let quoted=|value:&str|format!("'{}'",value.replace('\'',"'\\''"));
        let command=format!("{} __native-permission-observer {} {}",quoted(&executable.to_string_lossy()),quoted(&socket.to_string_lossy()),quoted(&nonce));
        let settings=json!({"hooks":{"PreToolUse":[{"matcher":"mcp__bridget__bridget_delegate|mcp__bridget__bridget_capabilities","hooks":[{"type":"command","command":command,"timeout":5,"onFailure":"block"}]}]}});
        let mut file=match std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&overlay){Ok(f)=>f,Err(_)=>{let _=std::fs::remove_file(&socket);return Err("permission_source_unavailable".into())}};
        let bytes=serde_json::to_vec(&settings).map_err(|_|"permission_source_unavailable")?;
        if file.write_all(&bytes).is_err(){let _=std::fs::remove_file(&socket);let _=std::fs::remove_file(&overlay);return Err("permission_source_unavailable".into())}
        file.sync_all().map_err(|_|"permission_source_unavailable")?;
        let overlay_revision=native_permissions::source_revision(&overlay)?;
        let observed_overlay=overlay.clone();
        let provider=Arc::new(Mutex::new(None));let owned_provider=provider.clone();
        let expected_session=args.windows(2).find(|pair|pair[0]=="--session-id").map(|pair|pair[1].clone());
        let acknowledgements:Acknowledgements=Arc::new((Mutex::new(None::<(String,bool)>),Condvar::new()));
        let acks=acknowledgements.clone();let stop=Arc::new(AtomicBool::new(false));let stopping=stop.clone();let instance=instance.to_owned();
        let thread=std::thread::spawn(move||{
            let mut revision=0u64;
            for accepted in listener.incoming(){
                if stopping.load(Ordering::Acquire){break}
                let Ok(mut stream)=accepted else{break};
                let _=stream.set_read_timeout(Some(Duration::from_secs(2)));let _=stream.set_write_timeout(Some(Duration::from_secs(2)));
                let mut line=String::new();
                if BufReader::new((&stream).take(16385)).read_line(&mut line).is_err()||line.len()>16384{continue}
                let input:Value=match serde_json::from_str(&line){Ok(v)=>v,Err(_)=>continue};
                let owner=*owned_provider.lock().unwrap_or_else(|e|e.into_inner());
                if input["nonce"]!=nonce||input["version"]!=1||!owner.is_some_and(|owner|peer_is_provider_descendant(&stream,owner)){continue}
                let valid_atom=|field:&str|input[field].as_str().filter(|s|!s.is_empty()&&s.len()<=256&&!s.chars().any(char::is_control));
                let mode=valid_atom("permission_mode");let session=valid_atom("session_id");let prompt=valid_atom("prompt_id");
                let valid=mode.is_some_and(|m|matches!(m,"default"|"acceptEdits"|"bypassPermissions"|"plan"|"dontAsk"|"auto"))
                    &&session.is_some()&&expected_session.as_ref().is_none_or(|expected|session==Some(expected.as_str()))
                    &&prompt.is_some()&&input["cwd"].as_str()==cwd.to_str()
                    &&native_permissions::source_revision(&observed_overlay).is_ok_and(|revision|revision==overlay_revision)
                    &&input["hook_event_name"]=="PreToolUse"&&input.get("agent_id").is_none_or(Value::is_null)
                    &&matches!(input["tool_name"].as_str(),Some("mcp__bridget__bridget_delegate"|"mcp__bridget__bridget_capabilities"));
                let checked=native_permissions::recheck_context_with_env(&context,&launch,&cwd,&env);
                if !valid||checked.is_err(){
                    let code=checked.err().unwrap_or_else(||"permission_attestation_unavailable".into());
                    emit(WrapperToDaemon::NativePermissionFact{fact:None,request_id:None,observation_id:None,unavailable_code:Some(code.clone())});
                    let _=writeln!(stream,"{}",json!({"accepted":false,"code":code}));continue;
                }
                let request_id=input.pointer("/tool_input/request_id").and_then(Value::as_str).filter(|s|!s.is_empty()&&s.len()<=128&&!s.chars().any(char::is_control)).map(str::to_owned);
                if input["tool_name"]=="mcp__bridget__bridget_delegate"&&request_id.is_none(){let _=writeln!(stream,"{}",json!({"accepted":false,"code":"permission_attestation_unavailable"}));continue}
                revision=revision.saturating_add(1);
                let mode=mode.unwrap();
                let mut policy=json!({"kind":"claude","permission_mode":mode,"tools":{"type":"preset","preset":"claude_code"},"permission_callback":{"kind":"native_wrapper","tool_approval":if matches!(mode,"bypassPermissions"|"auto"){"allow"}else{"prompt"},"plan_exit":"deny"},"settings_sources":"provider_default","launch_context":context});
                if bridget_transport::protocol::claude_launch_policy_options(&launch.args,&mut policy).is_err(){let _=writeln!(stream,"{}",json!({"accepted":false,"code":"permission_source_unavailable"}));continue}
                let fact=ProviderPermissions{version:1,source:"native_wrapper".into(),run_id:prompt.unwrap().into(),provider_session_id:session.unwrap().into(),provider_instance_id:instance.clone(),revision,driver:"claude_stream_json".into(),cwd:cwd.to_string_lossy().into_owned(),runtime_mode:match mode{"default"|"plan"=>"approval-required","acceptEdits"=>"auto-accept-edits","bypassPermissions"=>"full-access",_=>"auto"}.into(),interaction_mode:if mode=="plan"{"plan"}else{"default"}.into(),provider_policy:policy};
                let observation_id=uuid::Uuid::new_v4().to_string();
                *acks.0.lock().unwrap_or_else(|e|e.into_inner())=None;
                emit(WrapperToDaemon::NativePermissionFact{fact:Some(fact),request_id,observation_id:Some(observation_id.clone()),unavailable_code:None});
                let deadline=Instant::now()+Duration::from_secs(2);
                let mut ack=acks.0.lock().unwrap_or_else(|e|e.into_inner());
                while ack.as_ref().is_none_or(|a|a.0!=observation_id)&&Instant::now()<deadline{
                    let waited=acks.1.wait_timeout(ack,deadline.saturating_duration_since(Instant::now())).unwrap_or_else(|e|e.into_inner());ack=waited.0;
                }
                let accepted=ack.as_ref().is_some_and(|a|a.0==observation_id&&a.1);
                let _=writeln!(stream,"{}",json!({"accepted":accepted,"code":if accepted{"ok"}else{"permission_attestation_unavailable"}}));
            }
        });
        setup.armed=false;
        Ok(Self{socket,overlay,stop,thread:Some(thread),acknowledgements,provider})
    }
    pub(crate) fn bind_provider(&self,pid:u32)->Result<(),String>{
        let birth=crate::managed_process::process_birth(pid).map_err(|_|"permission_source_unavailable")?;
        *self.provider.lock().unwrap_or_else(|e|e.into_inner())=Some((pid,birth));Ok(())
    }
    pub(crate) fn overlay(&self)->&Path{&self.overlay}
    pub(crate) fn acknowledgement_sink(&self)->Acknowledgements{self.acknowledgements.clone()}
    pub(crate) fn acknowledge(sink:&Acknowledgements,id:String,accepted:bool){*sink.0.lock().unwrap_or_else(|e|e.into_inner())=Some((id,accepted));sink.1.notify_one();}
}
impl Drop for Observer{
    fn drop(&mut self){self.stop.store(true,Ordering::Release);let _=UnixStream::connect(&self.socket);if let Some(thread)=self.thread.take(){let _=thread.join();}let _=std::fs::remove_file(&self.socket);let _=std::fs::remove_file(&self.overlay);}
}

struct OverlaySetup{socket:PathBuf,overlay:PathBuf,armed:bool}
impl Drop for OverlaySetup{
    fn drop(&mut self){if self.armed{let _=std::fs::remove_file(&self.socket);let _=std::fs::remove_file(&self.overlay);}}
}

fn peer_is_provider_descendant(stream:&UnixStream,owner:(u32,u64))->bool{
    use std::os::fd::AsRawFd;
    if crate::managed_process::process_birth(owner.0).ok()!=Some(owner.1){return false}
    #[cfg(target_os="macos")]
    let peer={
        let mut pid:libc::pid_t=0;let mut length=std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
        let result=unsafe{libc::getsockopt(stream.as_raw_fd(),libc::SOL_LOCAL,libc::LOCAL_PEERPID,(&mut pid as *mut libc::pid_t).cast(),&mut length)};
        if result!=0||length as usize!=std::mem::size_of::<libc::pid_t>()||pid<=0{return false}pid as u32
    };
    #[cfg(target_os="linux")]
    let peer={
        let mut credentials=std::mem::MaybeUninit::<libc::ucred>::zeroed();let mut length=std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        let result=unsafe{libc::getsockopt(stream.as_raw_fd(),libc::SOL_SOCKET,libc::SO_PEERCRED,credentials.as_mut_ptr().cast(),&mut length)};
        if result!=0||length as usize!=std::mem::size_of::<libc::ucred>(){return false}
        let credentials=unsafe{credentials.assume_init()};if credentials.pid<=0{return false}credentials.pid as u32
    };
    let mut current=peer;
    for _ in 0..32{
        if current==owner.0{return true}
        let Ok(parent)=crate::mcp_identity::process_parent(current)else{return false};
        if parent==0||parent==current{return false}current=parent;
    }
    false
}

pub(crate) fn hook(args:&[String])->Result<(),String>{
    if args.len()!=2{return Err("permission_attestation_unavailable".into())}
    let mut payload=String::new();std::io::stdin().take(16385).read_to_string(&mut payload).map_err(|_|"permission_attestation_unavailable")?;
    if payload.len()>16384{return Err("permission_attestation_unavailable".into())}
    let payload:Value=serde_json::from_str(&payload).map_err(|_|"permission_attestation_unavailable")?;
    let mut stream=UnixStream::connect(&args[0]).map_err(|_|"permission_attestation_unavailable")?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).map_err(|_|"permission_attestation_unavailable")?;
    stream.set_write_timeout(Some(Duration::from_secs(1))).map_err(|_|"permission_attestation_unavailable")?;
    writeln!(stream,"{}",json!({"version":1,"nonce":args[1],"permission_mode":payload["permission_mode"],"session_id":payload["session_id"],"prompt_id":payload["prompt_id"],"cwd":payload["cwd"],"tool_name":payload["tool_name"],"tool_input":payload["tool_input"],"hook_event_name":payload["hook_event_name"],"agent_id":payload["agent_id"]})).map_err(|_|"permission_attestation_unavailable")?;
    let mut line=String::new();BufReader::new(stream.take(4097)).read_line(&mut line).map_err(|_|"permission_attestation_unavailable")?;
    let response:Value=serde_json::from_str(&line).map_err(|_|"permission_attestation_unavailable")?;
    if response["accepted"]!=true{return Err(response["code"].as_str().unwrap_or("permission_attestation_unavailable").to_owned())}
    Ok(())
}

#[cfg(test)]
#[path = "native_permission_observer149_tests.rs"]
mod native_permission_observer149_tests;
