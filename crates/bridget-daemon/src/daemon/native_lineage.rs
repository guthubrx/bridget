//! Vue humaine froide de la saga native. Aucun moteur de présentation parallèle.
use super::*;
use bridget_transport::protocol::{HumanLineageAction as A, HumanLineageError as E, HumanLineageRequest as Request, HumanLineageWatchEvent as Event};
use crate::delegation::ProjectionMutation;
use serde_json::Value;

#[derive(Clone)]
pub(super) struct Context {
    request: Request,
    project: Option<bridget_transport::protocol::CommunicationProject>,
    owner: String,
    root: String,
}

pub(super) struct Watch {
    context: Context,
    pending: VecDeque<Event>,
    generation: String,
    seq: u64,
    terminal: bool,
    notify: mpsc::SyncSender<()>,
}

pub(super) struct Journal {
    context: Context,
    pub shutdown: Option<UnixStream>,
    notify: mpsc::SyncSender<()>,
}

fn from_guard(e: bridget_transport::protocol::HumanThreadViewError) -> E {
    use bridget_transport::protocol::HumanThreadViewError as H;
    match e {
        H::UnsupportedVersion=>E::UnsupportedVersion,
        H::InvalidRequest=>E::InvalidRequest,
        H::ProjectMismatch=>E::ProjectMismatch,
        H::StorageUnavailable=>E::StoreUnavailable,
        _=>E::BindingUnavailable,
    }
}

fn valid_uuid(s: &str) -> bool {crate::threads::canonical_uuid(s).as_deref()==Some(s)}

fn validate(request: &Request) -> Result<(),E> {
    if request.version!=1 {return Err(E::UnsupportedVersion)}
    if request.t3_thread_id.is_empty() || request.t3_thread_id.len()>2048 || request.t3_thread_id.chars().any(bridget_core::is_disallowed_control)
        || request.project_root.len()>4096 || !PathBuf::from(&request.project_root).is_absolute() {return Err(E::InvalidRequest)}
    match &request.request {
        A::List{limit,cursor} if !(1..=100).contains(limit) || cursor.as_ref().is_some_and(|c|c.len()>2048 || c.chars().any(char::is_control))=>Err(E::InvalidRequest),
        A::Show{task_id,limit,..} if !valid_uuid(task_id) || !(1..=16384).contains(limit)=>Err(E::InvalidRequest),
        A::Journal{task_id,limit,after_seq,..} if !valid_uuid(task_id) || !(1..=100).contains(limit) || *after_seq>bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ=>Err(E::InvalidRequest),
        A::Cancel{task_id,request_id} if !valid_uuid(task_id)||!valid_uuid(request_id)=>Err(E::InvalidRequest),
        _=>Ok(()),
    }
}

fn authorize(state: &Arc<Mutex<DaemonState>>, conn: &str, request: &Request) -> Result<Context,E> {
    validate(request)?;
    let host={let st=state.lock().unwrap_or_else(|e|e.into_inner());
        human_client_guard(&st,conn,request.version,&request.t3_thread_id,request.request.capability()).map_err(from_guard)?;
        st.host.clone()};
    let project=crate::communication::resolve_communication_project(&request.project_root,&host,bridget_transport::protocol::CommunicationProjectSource::T3,None);
    let st=state.lock().unwrap_or_else(|e|e.into_inner());
    human_client_guard(&st,conn,request.version,&request.t3_thread_id,request.request.capability()).map_err(from_guard)?;
    let (owner,root)=human_authority_guard(&st,&request.t3_thread_id,&project).map_err(from_guard)?;
    Ok(Context {request:request.clone(),project,owner,root})
}

fn guard(st: &DaemonState,conn: &str,context: &Context) -> Result<(),E> {
    human_client_guard(st,conn,context.request.version,&context.request.t3_thread_id,context.request.request.capability()).map_err(from_guard)?;
    let (owner,root)=human_authority_guard(st,&context.request.t3_thread_id,&context.project).map_err(from_guard)?;
    if owner!=context.owner || root!=context.root {return Err(E::BindingUnavailable)}
    if let A::Journal{task_id,..}=&context.request.request {
        st.delegation_store.get(task_id).map_err(|_|E::StoreUnavailable)?.filter(|t|t.root_owner()==context.root).ok_or(E::TaskUnavailable)?;
    }
    Ok(())
}

fn journal_directory(st: &DaemonState,child: &str) -> Result<PathBuf,E> {
    if !valid_uuid(child) {return Err(E::JournalUnavailable)}
    let root=st.db_path.parent().ok_or(E::JournalUnavailable)?;
    Ok(root.join("sessions").join(child))
}

fn available(st: &DaemonState,task: &crate::delegation::Task) -> bool {
    journal_directory(st,&task.child).ok().is_some_and(|directory|directory.is_dir())
}

pub(super) fn handle(conn: &str,request: &Request,state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
    let context=match authorize(state,conn,request) {
        Ok(c)=>c,Err(e)=>return Some(refusal(request,e)),
    };
    if matches!(request.request,A::Watch) {return watch(conn,context,state)}
    if matches!(request.request,A::Journal{follow:true,..}) {return journal_follow(conn,context,state)}
    let mut drive=false;
    let result={let st=state.lock().unwrap_or_else(|e|e.into_inner());
        guard(&st,conn,&context).and_then(|()|match &request.request {
            A::List{limit,cursor}=>st.delegation_store.lineage_page(&context.root,*limit,cursor.as_deref(),|task|available(&st,task)),
            A::Show{task_id,offset,limit}=>{
                let task=st.delegation_store.get(task_id).map_err(|_|E::StoreUnavailable)?.filter(|t|t.root_owner()==context.root).ok_or(E::TaskUnavailable)?;
                st.delegation_store.lineage_show(&context.root,task_id,*offset,*limit,available(&st,&task))
            },
            A::Cancel{task_id,request_id}=>{
                let (receipt,run)=st.delegation_store.human_cancel(&context.root,task_id,request_id)?;drive=run;Ok(receipt)
            },
            A::Journal{task_id,after_seq,limit,..}=>{
                let task=st.delegation_store.get(task_id).map_err(|_|E::StoreUnavailable)?.filter(|t|t.root_owner()==context.root).ok_or(E::TaskUnavailable)?;
                // Seule la résolution autorisée est sous verrou. Les E/S suivent hors verrou.
                Ok(serde_json::json!({"directory":journal_directory(&st,&task.child)?,"task":task_id,"after":after_seq,"limit":limit}))
            },
            A::Watch=>unreachable!(),
        })};
    let result=if let (Ok(input),A::Journal{task_id,after_seq,limit,..})=(&result,&request.request) {
        let directory=PathBuf::from(input["directory"].as_str().unwrap_or(""));
        let page=crate::attach::lineage_journal_page(&directory,task_id,*after_seq,*limit);
        let st=state.lock().unwrap_or_else(|e|e.into_inner());
        guard(&st,conn,&context).and(page)
    }else{result};
    if drive {let state=Arc::clone(state);thread::spawn(move||native_delegation::tick(&state));}
    Some(DaemonToWrapper::HumanLineageResult {result:result.unwrap_or_else(E::result)})
}

fn refusal(request:&Request,e:E)->DaemonToWrapper {
    if matches!(request.request,A::Watch) {DaemonToWrapper::HumanLineageWatchEvent {event:Event::Error{version:1,code:e}}}
    else {DaemonToWrapper::HumanLineageResult{result:e.result()}}
}

fn terminate(watch:&mut Watch,code:E) {
    if watch.terminal {return}
    watch.pending.clear();watch.pending.push_back(Event::Error{version:1,code});watch.terminal=true;
    let _=watch.notify.try_send(());
}

pub(super) fn invalidate(st:&mut DaemonState) {
    for conn in st.lineage_watches.keys().cloned().collect::<Vec<_>>() {
        let result=guard(st,&conn,&st.lineage_watches[&conn].context);
        if let Err(e)=result {terminate(st.lineage_watches.get_mut(&conn).unwrap(),e)}
    }
    for (conn,journal) in &st.lineage_journals {
        if guard(st,conn,&journal.context).is_err() {
            let _=journal.notify.try_send(());
            if let Some(socket)=&journal.shutdown {let _=socket.shutdown(std::net::Shutdown::Both);}
        }
    }
}

pub(super) fn invalid_message(st:&mut DaemonState,conn:&str)->bool {
    if let Some(w)=st.lineage_watches.get_mut(conn) {terminate(w,E::InvalidRequest);return true}
    if let Some(j)=st.lineage_journals.get(conn) {
        let _=j.notify.try_send(());
        if let Some(s)=&j.shutdown {let _=s.shutdown(std::net::Shutdown::Both);}
        return true;
    }
    false
}

fn publish(st:&mut DaemonState,root:&str,mutation:&ProjectionMutation) {
    for conn in st.lineage_watches.keys().cloned().collect::<Vec<_>>() {
        if st.lineage_watches[&conn].context.root!=root {continue}
        if let Err(e)=guard(st,&conn,&st.lineage_watches[&conn].context) {terminate(st.lineage_watches.get_mut(&conn).unwrap(),e);continue}
        let watch=st.lineage_watches.get_mut(&conn).unwrap();
        if watch.terminal || (watch.generation==mutation.generation && watch.seq>=mutation.seq) {continue}
        let resync=watch.generation!=mutation.generation || watch.pending.len()>=16 || watch.pending.iter().any(|e|matches!(e,Event::Resync{..}));
        if resync {watch.pending.retain(|e|matches!(e,Event::Ready{..}))}
        let event=if resync {Event::Resync{version:1,generation:mutation.generation.clone(),seq:mutation.seq}}
            else {Event::Changed{version:1,generation:mutation.generation.clone(),seq:mutation.seq}};
        watch.generation=mutation.generation.clone();watch.seq=mutation.seq;watch.pending.push_back(event);
        let _=watch.notify.try_send(());
    }
}

fn watch(conn:&str,context:Context,state:&Arc<Mutex<DaemonState>>)->Option<DaemonToWrapper> {
    let mut st=state.lock().unwrap_or_else(|e|e.into_inner());
    if let Err(e)=guard(&st,conn,&context) {return Some(refusal(&context.request,e))}
    if st.lineage_watches.len()+st.lineage_journals.len()>=128 {return Some(refusal(&context.request,E::ResourceLimit))}
    let meta=match st.delegation_store.projection_meta(){Ok(m)=>m,Err(e)=>return Some(refusal(&context.request,e))};
    let Some(writer)=st.connections.get(conn).cloned() else{return Some(refusal(&context.request,E::BindingUnavailable))};
    let Some(shutdown)=writer.try_lock().ok().and_then(|w|w.get_ref().try_clone().ok()) else{return Some(refusal(&context.request,E::BindingUnavailable))};
    let (notify,receiver)=mpsc::sync_channel(1);
    st.lineage_watches.insert(conn.into(),Watch{context,pending:VecDeque::from([Event::Ready{version:1,generation:meta.generation.clone(),seq:0}]),generation:meta.generation,seq:meta.seq,terminal:false,notify});
    if let Some(receiver)=st.delegation_store.change_receiver() {
        let weak=Arc::downgrade(state);
        thread::spawn(move||while receiver.recv().is_ok(){
            let Some(state)=weak.upgrade()else{break};
            let mut st=state.lock().unwrap_or_else(|e|e.into_inner());
            for (root,mutation) in st.delegation_store.take_changes(){publish(&mut st,&root,&mutation)}
        });
    }
    let weak=Arc::downgrade(state);let conn=conn.to_string();
    thread::spawn(move||{
        loop {
            let Some(state)=weak.upgrade()else{break};
            let event={let mut st=state.lock().unwrap_or_else(|e|e.into_inner());
                let Some(w)=st.lineage_watches.get(&conn)else{break};
                if let Err(e)=guard(&st,&conn,&w.context){terminate(st.lineage_watches.get_mut(&conn).unwrap(),e)}
                st.lineage_watches.get_mut(&conn).unwrap().pending.pop_front()};
            if let Some(event)=event {
                let mut terminal=matches!(event,Event::Error{..});
                let sent=push_control_message_until_checked(&writer,&DaemonToWrapper::HumanLineageWatchEvent{event},Instant::now()+CANCEL_NOTIFICATION_BUDGET,||{
                    let st=state.try_lock().map_err(|_|std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
                    let e=st.lineage_watches.get(&conn).ok_or(E::BindingUnavailable).and_then(|w|guard(&st,&conn,&w.context)).err();
                    Ok(e.map(|code|{terminal=true;DaemonToWrapper::HumanLineageWatchEvent{event:Event::Error{version:1,code}}}))
                });
                if terminal||sent.is_err(){state.lock().unwrap_or_else(|e|e.into_inner()).lineage_watches.remove(&conn);break}
            } else {drop(state);if receiver.recv().is_err(){break}}
        }
        let _=shutdown.shutdown(std::net::Shutdown::Both);
    });
    None
}

fn journal_follow(conn:&str,context:Context,state:&Arc<Mutex<DaemonState>>)->Option<DaemonToWrapper> {
    let mut st=state.lock().unwrap_or_else(|e|e.into_inner());
    let refusal=|e|Some(DaemonToWrapper::HumanLineageResult{result:E::result(e)});
    if let Err(e)=guard(&st,conn,&context){return refusal(e)}
    if st.lineage_watches.len()+st.lineage_journals.len()>=128{return refusal(E::ResourceLimit)}
    let A::Journal{task_id,after_seq,limit,..}=&context.request.request else{unreachable!()};
    let task=match st.delegation_store.get(task_id){Ok(Some(t))if t.root_owner()==context.root=>t,_=>return refusal(E::TaskUnavailable)};
    let directory=match journal_directory(&st,&task.child){Ok(d)=>d,Err(e)=>return refusal(e)};
    let socket=match crate::environment::Namespace::from_environment(){Ok(n)=>n.socket,Err(_)=>return refusal(E::JournalUnavailable)};
    let live=st.router.get_agent(&task.child).is_some();
    let Some(writer)=st.connections.get(conn).cloned()else{return refusal(E::BindingUnavailable)};
    let shutdown=match writer.try_lock().ok().and_then(|w|w.get_ref().try_clone().ok()){Some(s)=>s,None=>return refusal(E::BindingUnavailable)};
    let task_id=task_id.clone();let after_seq=*after_seq;let limit=*limit;
    let (notify,receiver)=mpsc::sync_channel(1);
    st.lineage_journals.insert(conn.into(),Journal{context:context.clone(),shutdown:None,notify});
    let weak=Arc::downgrade(state);let conn=conn.to_string();
    thread::spawn(move||{
        let mut emit=|result:Value|->Result<(),E>{
            let state=weak.upgrade().ok_or(E::BindingUnavailable)?;
            let mut refused=None;
            let sent=push_control_message_until_checked(&writer,&DaemonToWrapper::HumanLineageResult{result},Instant::now()+CANCEL_NOTIFICATION_BUDGET,||{
                let st=state.try_lock().map_err(|_|std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
                Ok(guard(&st,&conn,&context).err().map(|e|{refused=Some(e);DaemonToWrapper::HumanLineageResult{result:e.result()}}))
            });
            sent.map_err(|_|E::BindingUnavailable)?;
            if let Some(e)=refused {return Err(e)}Ok(())
        };
        let result=if live {
            crate::attach::lineage_journal_follow(&socket,&task.child,&task_id,after_seq,limit,|stream|{
                let state=weak.upgrade().ok_or(E::BindingUnavailable)?;
                let mut st=state.lock().unwrap_or_else(|e|e.into_inner());guard(&st,&conn,&context)?;
                let journal=st.lineage_journals.get_mut(&conn).ok_or(E::BindingUnavailable)?;
                journal.shutdown=Some(stream.try_clone().map_err(|_|E::JournalUnavailable)?);Ok(())
            },&mut emit)
        }else{
            let mut cursor=after_seq;
            loop {
                let page=match crate::attach::lineage_journal_page(&directory,&task_id,cursor,limit){Ok(p)=>p,Err(e)=>break Err(e)};
                let caught_up=page["caught_up"]==true;
                let next=page["next_seq"].as_u64().unwrap_or(cursor);
                if let Err(e)=emit(page){break Err(e)}
                if caught_up {break Ok(())}
                if next<=cursor{break Err(E::JournalUnavailable)}cursor=next;
            }
        };
        let result=result.and_then(|()|{
            // Une frontière caught_up n'est pas une fin d'abonnement. Les seules
            // reprises ici sont les invalidations de garde, sans relire le corps.
            while receiver.recv().is_ok() {
                let state=weak.upgrade().ok_or(E::BindingUnavailable)?;
                let st=state.lock().unwrap_or_else(|e|e.into_inner());
                if !st.lineage_journals.contains_key(&conn) {return Ok(())}
                guard(&st,&conn,&context)?;
                return Err(E::InvalidRequest);
            }
            Ok(())
        });
        if let Err(error)=result {
            let error=weak.upgrade().and_then(|s|guard(&s.lock().unwrap_or_else(|e|e.into_inner()),&conn,&context).err()).unwrap_or(error);
            let _=emit(error.result());
        }
        if let Some(state)=weak.upgrade(){state.lock().unwrap_or_else(|e|e.into_inner()).lineage_journals.remove(&conn);}
        let _=shutdown.shutdown(std::net::Shutdown::Both);
    });
    None
}

#[cfg(test)]
#[path = "native_lineage_tests.rs"]
mod native_lineage_tests;
