//! Grammaire CLI fermée des lectures humaines natives.
use bridget_transport::protocol::{HumanLineageAction as A,HumanLineageError as E,HumanLineageRequest};
use std::collections::BTreeMap;
use std::io::Write;

fn parse(args:&[String])->Result<HumanLineageRequest,E> {
    let command=args.first().ok_or(E::InvalidRequest)?;
    let mut flags=BTreeMap::new();let mut i=1;
    while i<args.len() {
        let key=args[i].as_str();
        if !key.starts_with("--") || flags.contains_key(key){return Err(E::InvalidRequest)}
        let val=if matches!(key,"--json"|"--follow"){String::new()}else{
            i+=1;let val=args.get(i).ok_or(E::InvalidRequest)?;
            if val.starts_with("--"){return Err(E::InvalidRequest)}val.clone()
        };
        flags.insert(key,val);i+=1;
    }
    if flags.remove("--json").is_none(){return Err(E::InvalidRequest)}
    let t3_thread_id=flags.remove("--t3-thread").ok_or(E::InvalidRequest)?;
    let project_root=flags.remove("--project-root").ok_or(E::InvalidRequest)?;
    if t3_thread_id.is_empty()||t3_thread_id.len()>2048||t3_thread_id.chars().any(bridget_core::is_disallowed_control)||project_root.len()>4096||!std::path::Path::new(&project_root).is_absolute(){return Err(E::InvalidRequest)}
    let number=|value:Option<String>,default:u64,max:u64,min:u64|->Result<u64,E>{
        let value=match value{Some(v)=>v.parse::<u64>().map_err(|_|E::InvalidRequest)?,None=>default};
        if value<min||value>max{return Err(E::InvalidRequest)}Ok(value)
    };
    let uuid=|value:Option<String>|->Result<String,E>{let v=value.ok_or(E::InvalidRequest)?;
        if crate::threads::canonical_uuid(&v).as_deref()!=Some(v.as_str()){return Err(E::InvalidRequest)}Ok(v)};
    let request=match command.as_str(){
        "watch"=>A::Watch,
        "cancel"=>A::Cancel{task_id:uuid(flags.remove("--task"))?,request_id:uuid(flags.remove("--request-id"))?},
        "inspect"=>match flags.remove("--action").as_deref(){
            Some("list")=>A::List{limit:number(flags.remove("--limit"),50,100,1)? as u32,cursor:flags.remove("--cursor")},
            Some("show")=>A::Show{task_id:uuid(flags.remove("--task"))?,offset:number(flags.remove("--offset"),0,256*1024,0)? as u32,limit:number(flags.remove("--limit"),16384,16384,1)? as u32},
            Some("journal")=>A::Journal{task_id:uuid(flags.remove("--task"))?,after_seq:number(flags.remove("--after-seq"),0,bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ,0)?,limit:number(flags.remove("--limit"),50,100,1)? as u32,follow:flags.remove("--follow").is_some()},
            _=>return Err(E::InvalidRequest),
        },
        _=>return Err(E::InvalidRequest),
    };
    if !flags.is_empty(){return Err(E::InvalidRequest)}
    if let A::List{cursor:Some(c),..}=&request{if c.len()>2048||c.chars().any(char::is_control){return Err(E::InvalidRequest)}}
    Ok(HumanLineageRequest{version:1,t3_thread_id,project_root,request})
}

pub(super) fn run(args:&[String]) {
    let mut out=std::io::stdout().lock();
    let mut emit=|value:&serde_json::Value|->std::io::Result<()>{serde_json::to_writer(&mut out,value)?;writeln!(out)?;out.flush()};
    let request=match parse(args){Ok(r)=>r,Err(e)=>{let _=emit(&e.result());std::process::exit(2)}};
    let namespace=match crate::environment::Namespace::from_environment(){Ok(n)=>n,Err(_)=>{let _=emit(&E::BindingUnavailable.result());std::process::exit(3)}};
    let mut refused=false;
    let result=crate::communication::client::human_lineage(&namespace.socket,request,|value|{refused|=value["status"]=="error";emit(value)},||{
        let mut fd=libc::pollfd{fd:libc::STDOUT_FILENO,events:libc::POLLOUT,revents:0};unsafe{libc::poll(&mut fd,1,0);}
        fd.revents&(libc::POLLHUP|libc::POLLERR|libc::POLLNVAL)!=0
    });
    if let Err(error)=result {
        let closed=matches!(error,crate::communication::client::ClientError::Technical{code:"watch_output_closed",..});
        if !closed{let _=emit(&E::BindingUnavailable.result());}
        std::process::exit(if closed{0}else{3});
    }
    if refused{std::process::exit(2)}
}
