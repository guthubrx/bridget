//! Client humain négocié, sans inscription d'agent ni démarrage du daemon.
use super::*;
use bridget_transport::protocol::{HumanLineageAction as A,HumanLineageError as E,HumanLineageRequest,HumanLineageWatchEvent as Event};

pub(crate) fn human_lineage(
    socket:&Path,request:HumanLineageRequest,
    mut output:impl FnMut(&serde_json::Value)->io::Result<()>,
    mut cancelled:impl FnMut()->bool,
) -> Result<(),ClientError> {
    let capability=request.request.capability();
    let mut connection=DaemonConnection::connect_until(socket,Instant::now()+Duration::from_secs(6))?;
    connection.max_response_bytes=128*1024;
    let watch=matches!(request.request,A::Watch);
    let following=matches!(request.request,A::Journal{follow:true,..});
    let refusal=|code:E| if watch {serde_json::json!({"version":1,"status":"error","code":code})}else{code.result()};
    if !matches!(connection.exchange(&WrapperToDaemon::RoleHandshake{role:ConnectionRole::Client})?,DaemonToWrapper::RoleAccepted{role:ConnectionRole::Client}) {
        output(&refusal(E::UnsupportedVersion)).map_err(watch_output_error)?;return Ok(());
    }
    if !matches!(connection.exchange(&WrapperToDaemon::ClientHello{contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:crate::communication::issuer_scope("human-lineage"),capabilities:vec![capability]})?,DaemonToWrapper::ClientWelcome{version,capabilities,..} if version==CLIENT_CONTRACT_VERSION && capabilities.as_slice()==[capability]) {
        output(&refusal(E::UnsupportedVersion)).map_err(watch_output_error)?;return Ok(());
    }
    let mut response=connection.exchange(&WrapperToDaemon::HumanLineage{request})?;
    let mut generation=None;let mut seq=0;let mut caught_up=false;
    loop {
        let (value,terminal)=match response {
            DaemonToWrapper::HumanLineageWatchEvent{event} if watch=>{
                match &event {
                    Event::Ready{version:1,generation:current,seq:0} if generation.is_none() && crate::threads::canonical_uuid(current).as_deref()==Some(current.as_str())=>generation=Some(current.clone()),
                    Event::Changed{version:1,generation:current,seq:next} if generation.as_ref()==Some(current) && *next>seq && *next<=bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ=>seq=*next,
                    Event::Resync{version:1,generation:current,seq:next} if generation.is_some() && *next<=bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ && crate::threads::canonical_uuid(current).as_deref()==Some(current.as_str()) && (generation.as_ref()!=Some(current)||*next>seq)=>{generation=Some(current.clone());seq=*next;},
                    Event::Error{version:1,..}=>{},
                    _=>return Err(watch_protocol_error()),
                }
                let terminal=matches!(event,Event::Error{..});
                (serde_json::to_value(event).map_err(|_|watch_protocol_error())?,terminal)
            },
            DaemonToWrapper::HumanLineageResult{result} if !watch=>{
                if result["version"]!=1{return Err(watch_protocol_error())}
                if following && result["caught_up"]==true {caught_up=true;}
                let terminal=result["status"]=="error";
                if terminal {serde_json::from_value::<E>(result["code"].clone()).map_err(|_|watch_protocol_error())?;}
                (result,terminal)
            },
            _=>return Err(watch_protocol_error()),
        };
        output(&value).map_err(watch_output_error)?;
        if terminal || (!watch&&!following){return Ok(())}
        loop {
            if cancelled(){connection.poison();return Ok(())}
            if !connection.reader.buffer().is_empty(){break}
            let mut fd=libc::pollfd {fd:connection.reader.get_ref().as_raw_fd(),events:libc::POLLIN,revents:0};
            let ready=unsafe{libc::poll(&mut fd,1,100)};
            if ready>0{break}
            if ready<0 && io::Error::last_os_error().kind()!=io::ErrorKind::Interrupted{return Err(watch_protocol_error())}
        }
        connection.reader.get_ref().set_read_timeout(None).map_err(watch_output_error)?;
        let frame=bridget_transport::jsonl::read_unix_line(&mut connection.reader,128*1024,bridget_transport::jsonl::LineDeadline::AfterFirstByte(Duration::from_secs(6))).map_err(|_|watch_protocol_error())?;
        let Some(frame)=frame else {
            return if following && caught_up {Ok(())}else{Err(watch_protocol_error())};
        };
        response=serde_json::from_slice(&frame).map_err(|_|watch_protocol_error())?;
    }
}
