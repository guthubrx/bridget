//! Dernier contrôle de portée sur l'identité courante, avant les effets natifs.
use super::*;
use crate::mobile::store::{Operation, Store};
use bridget_transport::protocol::{MobileOperation, MobileOperationRequest};
use std::path::Path;

fn refusal(code: &str) -> DaemonToWrapper {
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({"status":"refused","code":code}),
    }
}
fn valid_command_id(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|u| u.get_version_num() == 4 && u.to_string() == id)
}
fn public_receipt(response: DaemonToWrapper, expected: &str) -> DaemonToWrapper {
    let DaemonToWrapper::IdempotencyResult {
        operation_kind,
        idempotency_key,
        mut issue,
        ..
    } = response
    else {
        return refusal("outcome_unknown");
    };
    if operation_kind != "send" || idempotency_key != expected {
        return refusal("outcome_unknown");
    }
    if let IdempotencyIssue::Rejected {
        category, reason, ..
    } = &mut issue
    {
        if category.len() > 64
            || !category
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'_')
        {
            *category = "native_refused".into();
        }
        *reason = "Commande refusée par les règles natives ; conservez son reçu.".into();
    }
    if let IdempotencyIssue::Orphaned { reason, .. } = &mut issue {
        *reason =
            "Le destinataire de la remise n'est plus disponible ; ne recréez pas la commande."
                .into();
    }
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({
            "status":"ok","command_id":idempotency_key,"operation_kind":"send","issue":issue
        }),
    }
}

/// Le journal historique n'atteste pas encore une partition par projet/machine.
fn agent_journal(
    request: &MobileOperationRequest,
    device: &crate::mobile::store::Device,
    root: &Path,
    agent_id: &str,
    after_seq: u64,
    limit: u32,
    tail: bool,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    if Uuid::parse_str(agent_id).is_err()
        || !(1..=50).contains(&limit)
        || tail && after_seq != 0
        || after_seq > bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ
    {
        return refusal("invalid_request");
    }
    if !device.scope.operations.contains(&Operation::Read) {
        return refusal("forbidden_scope");
    }
    if !device.scope.all_projects || !device.scope.all_machines {
        return refusal("history_scope_unavailable");
    }
    let (task, identity, foreign, socket) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let task = match st.delegation_store.for_child(agent_id) {
            Ok(task) => task,
            Err(_) => return refusal("storage_unavailable"),
        };
        let identity = st
            .router
            .get_agent(agent_id)
            .and_then(|route| live_connection_identity(&st, &route.connection_id));
        if task.is_none() && identity.is_none() {
            return refusal("not_found");
        }
        let foreign=identity.as_ref().and_then(|(_,instance)|st.presences.get(instance))
            .is_some_and(|presence|presence.host!=st.host);
        (task, identity, foreign, st.socket_path.clone())
    };
    let after_seq = if tail {
        match crate::attach::journal_tail_start(&socket,agent_id,limit) {
            Ok(start)=>start, Err(_)=>return refusal("journal_unavailable")
        }
    } else { after_seq };
    let page = if let Some(task) = &task {
        match native_lineage::read_task_journal(state, task, after_seq, limit) {
            Ok((page, _)) => page,
            Err(_) => return refusal("journal_unavailable"),
        }
    } else if foreign {
        match crate::attach::lineage_journal_remote_page(&socket,agent_id,after_seq,limit) {
            Ok(page)=>page,Err(_)=>return refusal("journal_unavailable")
        }
    } else {
        match crate::attach::lineage_journal_page(
            &root.join("sessions").join(agent_id),
            agent_id,
            after_seq,
            limit,
        ) {
            Ok(page) => page,
            Err(_) => return refusal("journal_unavailable"),
        }
    };
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(snapshot) = &task {
            let current = match st.delegation_store.get(&snapshot.task_id) {
                Ok(Some(task)) => task,
                _ => return refusal("journal_unavailable"),
            };
            if current.child != snapshot.child
                || current.child_instance != snapshot.child_instance
                || current.remote_placement != snapshot.remote_placement
            {
                return refusal("journal_unavailable");
            }
        } else if identity
            != st
                .router
                .get_agent(agent_id)
                .and_then(|route| live_connection_identity(&st, &route.connection_id))
        {
            return refusal("journal_unavailable");
        }
    }
    // Révocation/expiration relues après le disque ou le worker distant.
    if let Err(error) = Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        return refusal(error.code());
    }
    let agent_state={let st=state.lock().unwrap_or_else(|e|e.into_inner());
        st.router.get_agent(agent_id).and_then(|route|st.conn_instances.get(&route.connection_id))
            .and_then(|instance|st.presences.get(instance)).map(|p|p.state.clone()).unwrap_or_else(||"unknown".into())};
    DaemonToWrapper::MobileOperationResult {
        result: serde_json::json!({
            "status":"ok","agent_id":agent_id,"agent_state":agent_state,"items":page.get("events"),
            "next_seq":page.get("next_seq"),"caught_up":page.get("caught_up"),"gap":page.get("gap"),
            "older_before_seq":if after_seq>0 {Some(after_seq+1)} else {None}
        }),
    }
}

pub(super) fn handle(
    conn_id: &str,
    request: MobileOperationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let root = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client)
            || st.auxiliary_connections.contains(conn_id)
            || st.conn_names.contains_key(conn_id)
            || st.peer_uids.get(conn_id).copied() != Some(unsafe { libc::geteuid() })
            || !st.client_negotiations.get(conn_id).is_some_and(|n| {
                n.version == CLIENT_CONTRACT_VERSION
                    && n.capabilities.as_slice() == [ClientCapability::MobileOperationsV1]
            })
        {
            return refusal("authorization_required");
        }
        let Some(root) = st.db_path.parent() else {
            return refusal("storage_unavailable");
        };
        root.to_path_buf()
    };
    // Pas d'I/O mobile sous le verrou métier ; aucune opération n'est acceptée sur le seul label ClientHello.
    let device = match Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
    {
        Ok(device) => device,
        Err(error) => return refusal(error.code()),
    };
    if matches!(
        request.operation,
        MobileOperation::UpdatesStatus
            | MobileOperation::UpdatesCheck
            | MobileOperation::UpdatesInstall { .. }
            | MobileOperation::UpdatesConfigure { .. }
    ) {
        let granted = match crate::updates::device_granted(&root, &device.device_id) {
            Ok(granted) => granted,
            Err(code) => return refusal(code),
        };
        if !device.scope.all_machines || !device.scope.all_projects || !granted {
            return refusal("forbidden_scope");
        }
        let q = match request.operation {
            MobileOperation::UpdatesCheck => bridget_transport::protocol::UpdateRequest::Check,
            MobileOperation::UpdatesInstall { version } => {
                bridget_transport::protocol::UpdateRequest::Install { version }
            }
            MobileOperation::UpdatesConfigure { automatic } => {
                bridget_transport::protocol::UpdateRequest::Configure { automatic }
            }
            _ => bridget_transport::protocol::UpdateRequest::Status,
        };
        return match crate::updates::policy(&root, &q) {
            Ok(result) => DaemonToWrapper::MobileOperationResult { result },
            Err(code) => refusal(code),
        };
    }
    if matches!(
        request.operation,
        MobileOperation::Questions { .. }
            | MobileOperation::QuestionShow { .. }
            | MobileOperation::QuestionRespond { .. }
            | MobileOperation::QuestionReceipt { .. }
    ) {
        return super::mobile_questions::handle(&request, &root, state);
    }
    if matches!(
        request.operation,
        MobileOperation::TaskList { .. }
            | MobileOperation::TaskShow { .. }
            | MobileOperation::TaskCancel { .. }
            | MobileOperation::TaskCancelReceipt { .. }
    ) {
        return super::mobile_tasks::handle(&request, &root, state);
    }
    if matches!(
        request.operation,
        MobileOperation::ArtifactUpload { .. }
            | MobileOperation::ArtifactRead { .. }
            | MobileOperation::ArtifactList { .. }
            | MobileOperation::ArtifactLookup { .. }
    ) {
        return super::mobile_artifacts::handle(&request, &device, &root, state);
    }
    if let MobileOperation::AgentJournal {
        agent_id,
        after_seq,
        limit,
        tail,
    } = &request.operation
    {
        return agent_journal(
            &request, &device, &root, agent_id, *after_seq, *limit, *tail, state,
        );
    }
    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
    if st.conn_names.contains_key(conn_id)
        || st.connection_roles.get(conn_id) != Some(&ConnectionRole::Client)
    {
        return refusal("authorization_required");
    }
    let Some(negotiated) = st.client_negotiations.get_mut(conn_id) else {
        return refusal("authorization_required");
    };
    negotiated.issuer_scope = format!("mobile177_{}", device.device_id.replace('-', ""));
    match request.operation {
        MobileOperation::UpdatesStatus
        | MobileOperation::UpdatesCheck
        | MobileOperation::UpdatesInstall { .. }
        | MobileOperation::UpdatesConfigure { .. }
        | MobileOperation::Questions { .. }
        | MobileOperation::QuestionShow { .. }
        | MobileOperation::QuestionRespond { .. }
        | MobileOperation::QuestionReceipt { .. }
        | MobileOperation::TaskList { .. }
        | MobileOperation::TaskShow { .. }
        | MobileOperation::TaskCancel { .. }
        | MobileOperation::TaskCancelReceipt { .. }
        | MobileOperation::AgentJournal { .. }
        | MobileOperation::ArtifactUpload { .. }
        | MobileOperation::ArtifactRead { .. }
        | MobileOperation::ArtifactList { .. }
        | MobileOperation::ArtifactLookup { .. } => refusal("invalid_request"),
        MobileOperation::Federation { after, limit } => {
            if !(1..=50).contains(&limit) || after.as_ref().is_some_and(|s| s.len() > 256) {
                return refusal("invalid_request");
            }
            if !device.scope.operations.contains(&Operation::Read) {
                return refusal("forbidden_scope");
            }
            let mut names = std::collections::BTreeSet::new();
            names.insert(st.host.clone());
            names.extend(st.registry.machines().keys().cloned());
            names.extend(st.remote_workers.keys().cloned());
            names.extend(st.presences.values().map(|p| p.host.clone()));
            let selected = names
                .into_iter()
                .filter(|name| device.scope.all_machines || device.scope.machines.contains(name))
                .filter(|name| after.as_ref().is_none_or(|cursor| name > cursor))
                .take(usize::from(limit) + 1)
                .collect::<Vec<_>>();
            let more = selected.len() > usize::from(limit);
            let next = more.then(|| selected[usize::from(limit) - 1].clone());
            let items=selected.into_iter().take(usize::from(limit)).map(|machine| {
                let (availability,age)=if machine==st.host {("connected",Some(0))}
                    else if let Some(worker)=st.remote_workers.get(&machine) {
                        let current=worker.contact==bridget_transport::protocol::WorkerContactState::Connected
                            && worker.last_contact.elapsed()<Duration::from_secs(bridget_transport::protocol::WORKER_STALE_SECS as u64);
                        (if current {"connected"}else{"stale"},Some(worker.last_contact.elapsed().as_secs()))
                    } else if st.presences.values().any(|p|p.host==machine
                        && matches!(p.state.as_str(),"connected"|"busy") && presence_within_retention(p)) {
                        ("observed_agent",None)
                    } else {("unknown",None)};
                serde_json::json!({"machine":machine,"availability":availability,"observed_age_secs":age})
            }).collect::<Vec<_>>();
            DaemonToWrapper::MobileOperationResult {
                result: serde_json::json!({
                    "status":"ok","items":items,"next_cursor":next,"observed_at":unix_now_secs()
                }),
            }
        }
        MobileOperation::Lookup { command_id } => {
            if !valid_command_id(&command_id) {
                return refusal("invalid_request");
            }
            if !device.scope.operations.contains(&Operation::Message) {
                return refusal("forbidden_scope");
            }
            public_receipt(
                handle_idempotency_lookup(conn_id, "send".into(), command_id.clone(), &st),
                &command_id,
            )
        }
        MobileOperation::Message {
            device_id,
            command_id,
            issued_at,
            agent_id,
            body,
            guide_now,
        } => {
            if device_id != device.device_id {
                return refusal("command_origin_changed");
            }
            if !valid_command_id(&command_id)
                || Uuid::parse_str(&agent_id).is_err()
                || body.trim().is_empty()
                || body.len() > 10_000
                || body.contains('\0')
            {
                return refusal("invalid_request");
            }
            let Some((target, instance)) = st
                .router
                .get_agent(&agent_id)
                .and_then(|route| live_connection_identity(&st, &route.connection_id))
            else {
                return refusal("not_found");
            };
            if target != agent_id {
                return refusal("not_found");
            }
            let Some(presence) = st.presences.get(&instance) else {
                return refusal("not_found");
            };
            let project = agent_communication_project(&st, &agent_id);
            if !device
                .scope
                .permits(Operation::Message, &presence.host, project.as_ref())
            {
                return refusal("not_found");
            }
            let mut message = bridget_core::BridgetMessage::new("human", &agent_id, body);
            message.id = command_id.clone();
            if guide_now {
                message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
                message.references.push("steer-or-queue".into());
            }
            // Pas de reply réservé à une identité d'agent ; les résultats se lisent dans le journal.
            let mut controls = Vec::new();
            let response = handle_idempotent_send_with_result_authority(
                conn_id,
                message,
                command_id.clone(),
                issued_at,
                IdempotentSendAdmission {
                    project: None,
                    issued_at_tolerance_secs: 300,
                },
                &mut st,
                &mut controls,
                None,
                true,
            );
            st.pending_post_response_controls
                .entry(conn_id.into())
                .or_default()
                .extend(controls);
            public_receipt(response, &command_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mobile::store::Scope;
    use bridget_transport::protocol::{CommunicationProject, IdentityCredential};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    struct Fixture {
        state: Arc<Mutex<DaemonState>>,
        root: PathBuf,
        native: DaemonConfig,
        _recipient: UnixStream,
        key: String,
        device: String,
        agent: String,
    }
    impl Fixture {
        fn new() -> Self {
            let (mut state, native) =
                presence_tests::state_with_registered_agent("mobile177-message");
            let recipient = spec094_live_test_connection(&mut state, "conn-1");
            state.conn_counter = 1; // conn-1 appartient déjà au destinataire de fixture.
            let root = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("bridget177-operations-{}", Uuid::new_v4()));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&root)
                .unwrap();
            state.db_path = root.join("bridget.db");
            state.artifact_root = root.join("artifacts");
            state.communication_projects.insert(
                "conn-1".into(),
                (
                    Some(CommunicationProject {
                        host: "macbook".into(),
                        root: "/project/a".into(),
                    }),
                    None,
                    false,
                ),
            );
            state
                .connection_roles
                .insert("phone".into(), ConnectionRole::Client);
            state
                .peer_uids
                .insert("phone".into(), unsafe { libc::geteuid() });
            state.client_negotiations.insert(
                "phone".into(),
                NegotiatedClient {
                    version: 1,
                    issuer_scope: crate::communication::issuer_scope("untrusted-client-label"),
                    capabilities: vec![ClientCapability::MobileOperationsV1],
                },
            );
            let mut store = Store::open(&root.join("mobile-devices.db")).unwrap();
            let grant = store
                .issue_pairing(
                    &Scope {
                        all_machines: false,
                        machines: vec!["macbook".into()],
                        all_projects: false,
                        projects: vec![CommunicationProject {
                            host: "macbook".into(),
                            root: "/project/a".into(),
                        }],
                        operations: vec![Operation::Read, Operation::Message],
                    },
                    unix_now_secs(),
                )
                .unwrap();
            let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
            let device = store
                .pair(
                    &grant.credential,
                    &Uuid::new_v4().to_string(),
                    &key,
                    "Phone",
                    unix_now_secs(),
                )
                .unwrap()
                .device_id;
            let agent = state.presences["instance-1"].name.clone();
            Self {
                state: Arc::new(Mutex::new(state)),
                root,
                native,
                _recipient: recipient,
                key,
                device,
                agent,
            }
        }
        fn request(&self, op: MobileOperation) -> MobileOperationRequest {
            MobileOperationRequest {
                credential: IdentityCredential::new(self.key.clone()),
                operation: op,
            }
        }
        fn command(&self, id: &str, body: &str) -> MobileOperation {
            MobileOperation::Message {
                device_id: self.device.clone(),
                command_id: id.into(),
                issued_at: unix_now_secs(),
                agent_id: self.agent.clone(),
                body: body.into(),
                guide_now: false,
            }
        }
        fn deliveries(&self) -> i64 {
            self.state
                .lock()
                .unwrap()
                .store
                .connection()
                .query_row("SELECT count(*) FROM send_deliveries", [], |r| r.get(0))
                .unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
            for suffix in ["", "-wal", "-shm"] {
                let _ =
                    std::fs::remove_file(format!("{}{}", self.native.db_path.display(), suffix));
            }
        }
    }
    fn value(response: DaemonToWrapper) -> serde_json::Value {
        match response {
            DaemonToWrapper::MobileOperationResult { result } => result,
            _ => panic!("mobile response expected"),
        }
    }

    #[test]
    fn updates179_phone_requires_explicit_grant_and_revocation_is_live() {
        let f = Fixture::new();
        let (key, device) = full_device(&f);
        let call = |operation| {
            value(handle(
                "phone",
                MobileOperationRequest {
                    credential: IdentityCredential::new(key.clone()),
                    operation,
                },
                &f.state,
            ))
        };
        assert_eq!(
            call(MobileOperation::UpdatesConfigure { automatic: false })["code"],
            "forbidden_scope"
        );
        let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
        let legacy_scope = store.authenticate(&key, unix_now_secs()).unwrap().scope;
        assert!(crate::updates::grant_device(&f.root, &device).unwrap());
        assert_eq!(store.authenticate(&key, unix_now_secs()).unwrap().scope, legacy_scope,
            "The previous binary can still deserialize the unchanged device scope");
        assert_eq!(call(MobileOperation::UpdatesStatus)["automatic"], true);
        assert_eq!(
            call(MobileOperation::UpdatesConfigure { automatic: false })["automatic"],
            false
        );
        assert_eq!(call(MobileOperation::UpdatesStatus)["automatic"], false);
        store.revoke(&device, unix_now_secs()).unwrap();
        assert_eq!(
            call(MobileOperation::UpdatesConfigure { automatic: true })["code"],
            "device_revoked"
        );
        assert_eq!(
            crate::updates::policy(&f.root, &bridget_transport::protocol::UpdateRequest::Status)
                .unwrap()["automatic"],
            false
        );
    }

    fn full_device(f: &Fixture) -> (String, String) {
        let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
        let pairing = store
            .issue_pairing(
                &Scope {
                    all_machines: true,
                    machines: vec![],
                    all_projects: true,
                    projects: vec![],
                    operations: vec![Operation::Read, Operation::Message],
                },
                unix_now_secs(),
            )
            .unwrap();
        let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let id = store
            .pair(
                &pairing.credential,
                &Uuid::new_v4().to_string(),
                &key,
                "Full phone",
                unix_now_secs(),
            )
            .unwrap()
            .device_id;
        (key, id)
    }
    #[test]
    fn mobile177_artifact_native_receipt_read_pagination_and_envelope_binding() {
        use bridget_transport::protocol::MobileFileUpload;
        let f = Fixture::new();
        let (key, device) = full_device(&f);
        let request = |operation| MobileOperationRequest {
            credential: IdentityCredential::new(key.clone()),
            operation,
        };
        let call = |operation| value(handle("phone", request(operation), &f.state));
        let mut uploads = Vec::new();
        for n in 0..2 {
            let bytes = format!("Fichier {n} — données UTF-8").into_bytes();
            let upload = MobileFileUpload {
                device_id: device.clone(),
                command_id: Uuid::new_v4().to_string(),
                agent_id: f.agent.clone(),
                file_name: format!("preuve-{n}.txt"),
                media_type: "text/plain".into(),
                expected_digest: crate::artifact_types::sha256_hex(&bytes),
                bytes,
            };
            let first = call(MobileOperation::ArtifactUpload {
                upload: upload.clone(),
            });
            assert_eq!(first["status"], "ok", "{first}");
            assert_eq!(first["replayed"], false);
            let replay = call(MobileOperation::ArtifactUpload {
                upload: upload.clone(),
            });
            assert_eq!(first["receipt"], replay["receipt"]);
            assert_eq!(replay["replayed"], true);
            let lookup = call(MobileOperation::ArtifactLookup {
                command_id: upload.command_id.clone(),
            });
            assert_eq!(lookup["receipt"], first["receipt"]);
            uploads.push((upload, first));
        }
        let page = call(MobileOperation::ArtifactList {
            agent_id: f.agent.clone(),
            after: None,
            limit: 1,
        });
        assert_eq!(page["items"].as_array().unwrap().len(), 1);
        let next = page["next_cursor"].as_str().unwrap().to_string();
        let last = call(MobileOperation::ArtifactList {
            agent_id: f.agent.clone(),
            after: Some(next),
            limit: 1,
        });
        assert_eq!(last["items"].as_array().unwrap().len(), 1);
        assert_ne!(
            page["items"][0]["version_ref"],
            last["items"][0]["version_ref"]
        );
        assert!(last["next_cursor"].is_null());
        assert!(!page.to_string().contains("canonical_path"));
        assert!(!page.to_string().contains(&f.root.display().to_string()));
        let (upload, receipt) = &uploads[0];
        let read = call(MobileOperation::ArtifactRead {
            agent_id: f.agent.clone(),
            artifact_ref: receipt["receipt"]["artifact_ref"].as_str().unwrap().into(),
            version_ref: receipt["receipt"]["version_ref"].as_str().unwrap().into(),
            offset: 0,
            limit: 16384,
        });
        assert_eq!(read["status"], "ok", "{read}");
        let content: Vec<u8> = serde_json::from_value(read["chunk"]["bytes"].clone()).unwrap();
        assert_eq!(content, upload.bytes);
        let foreign = call(MobileOperation::ArtifactRead {
            agent_id: Uuid::new_v4().to_string(),
            artifact_ref: receipt["receipt"]["artifact_ref"].as_str().unwrap().into(),
            version_ref: receipt["receipt"]["version_ref"].as_str().unwrap().into(),
            offset: 0,
            limit: 16384,
        });
        assert_eq!(foreign["code"], "not_found");
        let mut changed = upload.clone();
        changed.file_name = "autre.txt".into();
        assert_eq!(
            call(MobileOperation::ArtifactUpload { upload: changed })["code"],
            "envelope_mismatch"
        );
        let store =
            crate::artifact_store::ArtifactStore::open(&f.state.lock().unwrap().db_path).unwrap();
        assert_eq!(
            store.total_published_blob_bytes().unwrap(),
            uploads
                .iter()
                .map(|(u, _)| u.bytes.len() as u64)
                .sum::<u64>()
        );
        f.state
            .lock()
            .unwrap()
            .router
            .unregister_by_conn("conn-1")
            .unwrap();
        let offline = call(MobileOperation::ArtifactList {
            agent_id: f.agent.clone(),
            after: None,
            limit: 50,
        });
        assert_eq!(offline["items"].as_array().unwrap().len(), 2);
        assert_eq!(
            call(MobileOperation::ArtifactRead {
                agent_id: f.agent.clone(),
                artifact_ref: receipt["receipt"]["artifact_ref"].as_str().unwrap().into(),
                version_ref: receipt["receipt"]["version_ref"].as_str().unwrap().into(),
                offset: 0,
                limit: 16384
            })["status"],
            "ok"
        );
        assert_eq!(f.deliveries(), 0);
    }
    #[test]
    fn mobile177_generated_html_is_listed_and_read_only_in_its_conversation() {
        let f = Fixture::new();
        let (key, _) = full_device(&f);
        let (db, blobs, instance, project) = {
            let st = f.state.lock().unwrap();
            let (_, instance, project) = artifact_access_scope(&st, "conn-1").unwrap();
            (st.db_path.clone(), st.artifact_root.clone(), instance, project)
        };
        let html = "<article><h1>Document généré</h1></article>";
        let mut publication: crate::artifact_types::ArtifactPublicationV1 = serde_json::from_str(
            include_str!("../../tests/fixtures/artifacts/chart-external-v1.json"),
        ).unwrap();
        publication.kind = crate::artifact_types::ArtifactKind::Html;
        publication.payload = serde_json::json!({"html": html});
        let context = crate::artifact_store::ArtifactPublicationContext {
            project_id: project.clone(),
            conversation_reference: format!("conversation:{project}:{}", f.agent),
            turn_reference: "turn:mobile177-html".into(),
            created_by: f.agent.clone(),
            origin_instance: instance,
            observed_at: unix_timestamp(),
        };
        let mut service = ArtifactService::open(&db, &blobs, Default::default()).unwrap();
        service.publish(context, publication).unwrap();
        let call = |operation| value(handle("phone", MobileOperationRequest {
            credential: IdentityCredential::new(key.clone()), operation,
        }, &f.state));
        let page = call(MobileOperation::ArtifactList {
            agent_id: f.agent.clone(), after: None, limit: 50,
        });
        assert_eq!(page["status"], "ok", "{page}");
        let item = &page["items"][0];
        assert_eq!(item["media_type"], "text/html");
        assert_eq!(item["digest"], crate::artifact_types::sha256_hex(html.as_bytes()));
        let read = |agent| call(MobileOperation::ArtifactRead {
            agent_id: agent,
            artifact_ref: item["artifact_ref"].as_str().unwrap().into(),
            version_ref: item["version_ref"].as_str().unwrap().into(),
            offset: 0, limit: 16384,
        });
        let bytes: Vec<u8> = serde_json::from_value(read(f.agent.clone())["chunk"]["bytes"].clone()).unwrap();
        assert_eq!(bytes, html.as_bytes());
        assert_eq!(read(Uuid::new_v4().to_string())["code"], "not_found");
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_http_eight_mib_file_replays_one_native_publication() {
        use bridget_transport::protocol::MobileFileUpload;
        let f = Fixture::new();
        let (key, device) = full_device(&f);
        let bytes = vec![b'x'; 8 * 1024 * 1024];
        let upload = MobileFileUpload {
            device_id: device,
            command_id: Uuid::new_v4().to_string(),
            agent_id: f.agent.clone(),
            file_name: "preuve.txt".into(),
            media_type: "text/plain".into(),
            expected_digest: crate::artifact_types::sha256_hex(&bytes),
            bytes,
        };
        let body = serde_json::to_value(&upload).unwrap();
        assert!(body.to_string().len() < bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES);
        let socket = f.root.join("artifacts.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let state = f.state.clone();
        let native = thread::spawn(move || {
            for _ in 0..5 {
                let (peer, _) = listener.accept().unwrap();
                handle_connection(peer, state.clone()).unwrap();
            }
        });
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap().block_on(async {
            let (addr,http)=crate::mobile::http::tests::start_native(&f.root,&socket).await;
            let parse=|raw:String|serde_json::from_str::<serde_json::Value>(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/artifacts",Some(&key),&body,"").await;
            assert_eq!(status,200,"{raw}");
            let first=parse(raw)["result"].clone();
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/artifacts/uploads/{}",upload.command_id),Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["receipt"],first["receipt"]);
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/artifacts",Some(&key),&body,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["receipt"],first["receipt"]);
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/agents/{}/artifacts",f.agent),Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["items"].as_array().unwrap().len(),1);
            let read=serde_json::json!({"action":"artifact_read","agent_id":f.agent,
                "artifact_ref":first["receipt"]["artifact_ref"],"version_ref":first["receipt"]["version_ref"],
                "offset":0,"limit":16384});
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/artifacts/read",Some(&key),&read,"").await;
            assert_eq!(status,200,"{raw}");
            let chunk=parse(raw)["result"]["chunk"].clone();
            assert_eq!(chunk["bytes"].as_array().unwrap().len(),16384);
            assert_eq!(chunk["total_len"],8*1024*1024);
            http.abort();
        });
        native.join().unwrap();
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_artifact_rejects_types_paths_scope_and_revocation_without_publication() {
        use bridget_transport::protocol::MobileFileUpload;
        let f = Fixture::new();
        let bytes = b"<html>not an image</html>".to_vec();
        let mut upload = MobileFileUpload {
            device_id: f.device.clone(),
            command_id: Uuid::new_v4().to_string(),
            agent_id: f.agent.clone(),
            file_name: "test.png".into(),
            media_type: "image/png".into(),
            expected_digest: crate::artifact_types::sha256_hex(&bytes),
            bytes,
        };
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::ArtifactUpload {
                    upload: upload.clone()
                }),
                &f.state
            ))["code"],
            "invalid_file"
        );
        upload.media_type = "text/plain".into();
        upload.file_name = "../preuve.txt".into();
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::ArtifactUpload {
                    upload: upload.clone()
                }),
                &f.state
            ))["code"],
            "invalid_file"
        );
        upload.file_name = "preuve.txt".into();
        upload.bytes = vec![b'x'; 8 * 1024 * 1024 + 1];
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::ArtifactUpload {
                    upload: upload.clone()
                }),
                &f.state
            ))["code"],
            "invalid_file"
        );
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::ArtifactList {
                    agent_id: f.agent.clone(),
                    after: None,
                    limit: 50
                }),
                &f.state
            ))["code"],
            "history_scope_unavailable"
        );
        Store::open(&f.root.join("mobile-devices.db"))
            .unwrap()
            .revoke(&f.device, unix_now_secs())
            .unwrap();
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::ArtifactUpload { upload }),
                &f.state
            ))["code"],
            "device_revoked"
        );
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_task_projection_cancel_receipt_and_late_result_keep_native_state() {
        let f = Fixture::new();
        let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
        let pair = store
            .issue_pairing(
                &Scope {
                    all_machines: true,
                    machines: vec![],
                    all_projects: true,
                    projects: vec![],
                    operations: vec![
                        Operation::TaskRead,
                        Operation::TaskCancel,
                        Operation::TaskContinue,
                        Operation::Message,
                    ],
                },
                unix_now_secs(),
            )
            .unwrap();
        let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let device = store
            .pair(
                &pair.credential,
                &Uuid::new_v4().to_string(),
                &key,
                "Phone",
                unix_now_secs(),
            )
            .unwrap()
            .device_id;
        let request = |operation| MobileOperationRequest {
            credential: IdentityCredential::new(key.clone()),
            operation,
        };
        let call = |operation| value(handle("phone", request(operation), &f.state));
        let first = Uuid::new_v4().to_string();
        let second = Uuid::new_v4().to_string();
        let third = Uuid::new_v4().to_string();
        {
            let st = f.state.lock().unwrap();
            for (id, owner) in [(&first, &f.agent), (&second, &f.agent), (&third, &third)] {
                let mut task = crate::delegation::lineage_test_task(
                    id,
                    owner,
                    "instance-1",
                    "instance-1",
                    &Uuid::new_v4().to_string(),
                    None,
                    "result_available",
                    unix_now_secs(),
                );
                task.root_owner_agent_id = owner.clone();
                task.guarded = false;
                task.guard_released = true;
                task.result = Some("Résultat conservé".into());
                task.error = Some("private-stderr-canary".into());
                st.delegation_store
                    .insert(&task, &format!("req-{id}"), b"canonical")
                    .unwrap();
            }
        }
        let page = call(MobileOperation::TaskList {
            agent_id: f.agent.clone(),
            after: None,
            limit: 1,
        });
        assert_eq!(page["status"], "ok", "{page}");
        assert_eq!(page["tasks"].as_array().unwrap().len(), 1);
        assert!(!page.to_string().contains("private-stderr-canary"));
        let entry = &page["tasks"][0];
        assert_eq!(entry["actions"]["direct_pause"], false);
        assert_eq!(entry["actions"]["request_pause"], true);
        let last = call(MobileOperation::TaskList {
            agent_id: f.agent.clone(),
            after: Some(page["next_cursor"].as_str().unwrap().into()),
            limit: 1,
        });
        assert_eq!(last["tasks"].as_array().unwrap().len(), 1);
        assert_ne!(page["tasks"][0]["task_id"], last["tasks"][0]["task_id"]);
        let id = Uuid::new_v4().to_string();
        let cancel = || MobileOperation::TaskCancel {
            device_id: device.clone(),
            command_id: id.clone(),
            task_id: first.clone(),
        };
        let receipt = call(cancel());
        assert_eq!(receipt["receipt"]["status"], "result_available");
        assert_eq!(call(cancel())["receipt"], receipt["receipt"]);
        assert_eq!(
            call(MobileOperation::TaskCancelReceipt {
                device_id: device.clone(),
                command_id: id.clone(),
                task_id: first.clone()
            })["receipt"],
            receipt["receipt"]
        );
        for other in [&second, &third] {
            assert_eq!(
                call(MobileOperation::TaskCancel {
                    device_id: device.clone(),
                    command_id: id.clone(),
                    task_id: other.clone()
                })["code"],
                "envelope_mismatch"
            );
        }
        let shown = call(MobileOperation::TaskShow {
            task_id: first.clone(),
            offset: 0,
        });
        assert_eq!(shown["task"]["status"], "result_available");
        assert_eq!(shown["result"], "Résultat conservé");
        assert_eq!(
            f.state
                .lock()
                .unwrap()
                .delegation_store
                .get(&first)
                .unwrap()
                .unwrap()
                .state,
            "result_available"
        );
        assert_eq!(f.deliveries(), 0);
    }
    #[test]
    fn mobile177_task_partial_scope_cannot_read_or_cancel_global_history() {
        let f = Fixture::new();
        for operation in [
            MobileOperation::TaskList {
                agent_id: f.agent.clone(),
                after: None,
                limit: 50,
            },
            MobileOperation::TaskShow {
                task_id: Uuid::new_v4().to_string(),
                offset: 0,
            },
            MobileOperation::TaskCancel {
                device_id: f.device.clone(),
                command_id: Uuid::new_v4().to_string(),
                task_id: Uuid::new_v4().to_string(),
            },
        ] {
            assert_eq!(
                value(handle("phone", f.request(operation), &f.state))["code"],
                "forbidden_scope"
            );
        }
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_question_choice_replay_binding_and_producer_ack_are_distinct() {
        use bridget_transport::protocol::{HumanInboxKind, HumanInboxProducer, HumanInboxSubject};
        let f = Fixture::new();
        let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
        let pair = store
            .issue_pairing(
                &Scope {
                    all_machines: true,
                    machines: vec![],
                    all_projects: true,
                    projects: vec![],
                    operations: vec![Operation::TaskRead, Operation::TaskRespond],
                },
                unix_now_secs(),
            )
            .unwrap();
        let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let device = store
            .pair(
                &pair.credential,
                &Uuid::new_v4().to_string(),
                &key,
                "Phone",
                unix_now_secs(),
            )
            .unwrap()
            .device_id;
        let call = |op| {
            value(handle(
                "phone",
                MobileOperationRequest {
                    credential: IdentityCredential::new(key.clone()),
                    operation: op,
                },
                &f.state,
            ))
        };
        let items = {
            let st = f.state.lock().unwrap();
            let mut items = Vec::new();
            for n in 0..2 {
                let context=serde_json::json!({"summary":"Question humaine de fixture",
                    "facts":{"method":"requestApproval","command":"printf fixture","cwd_verified":false},
                    "model_text":{"quoted_untrusted":"Texte du modèle"},"provider_secret":"private-canary-not-projected"}).to_string();
                let item = crate::human_inbox::deposit(
                    st.store.connection(),
                    crate::human_inbox::DepositRequest {
                        dedup_key: &format!("mobile177-question-{n}"),
                        kind: HumanInboxKind::InterventionRequired,
                        subject: &HumanInboxSubject {
                            agent_id: Some(f.agent.clone()),
                            ..Default::default()
                        },
                        context: &context,
                        options: &["accept".into(), "decline".into()],
                        producer: HumanInboxProducer::Daemon,
                        now: unix_now_secs(),
                    },
                )
                .unwrap()
                .unwrap();
                items.push(item.item_id);
            }
            items
        };
        let page = call(MobileOperation::Questions {
            after: None,
            limit: 1,
        });
        assert_eq!(page["items"].as_array().unwrap().len(), 1);
        assert!(!page.to_string().contains("private-canary"));
        assert_eq!(page["items"][0]["facts"]["command"], "printf fixture");
        let next = call(MobileOperation::Questions {
            after: Some(page["next_cursor"].as_str().unwrap().into()),
            limit: 1,
        });
        assert_ne!(page["items"][0]["item_id"], next["items"][0]["item_id"]);
        let id = Uuid::new_v4().to_string();
        let response = |item: &String, choice: &str| MobileOperation::QuestionRespond {
            device_id: device.clone(),
            command_id: id.clone(),
            item_id: item.clone(),
            choice: choice.into(),
        };
        assert_eq!(
            call(MobileOperation::QuestionReceipt {
                device_id: device.clone(),
                command_id: id.clone(),
                item_id: items[0].clone(),
                choice: "decline".into()
            })["code"],
            "not_found"
        );
        assert_eq!(
            call(response(&items[0], "not-offered"))["code"],
            "choice_not_offered"
        );
        let first = call(response(&items[0], "decline"));
        assert_eq!(first["status"], "ok");
        assert_eq!(
            first["receipt"]["application"]["producer_acknowledged"],
            false
        );
        assert_eq!(
            call(response(&items[0], "decline"))["receipt"],
            first["receipt"]
        );
        assert_eq!(
            call(response(&items[0], "accept"))["code"],
            "envelope_mismatch"
        );
        assert_eq!(
            call(response(&items[1], "decline"))["code"],
            "envelope_mismatch"
        );
        assert_eq!(
            call(MobileOperation::QuestionShow {
                item_id: items[0].clone()
            })["item"]["can_respond"],
            false
        );
        {
            let st = f.state.lock().unwrap();
            let decision = first["receipt"]["decision_id"].as_str().unwrap();
            crate::human_inbox::ack(st.store.connection(), decision, unix_now_secs())
                .unwrap()
                .unwrap();
        }
        let receipt = call(MobileOperation::QuestionReceipt {
            device_id: device.clone(),
            command_id: id.clone(),
            item_id: items[0].clone(),
            choice: "decline".into(),
        });
        assert_eq!(
            receipt["receipt"]["application"]["producer_acknowledged"],
            true
        );
        assert!(
            receipt["receipt"]["application"]["confirmed"].is_null(),
            "ACK seul ne prouve aucune application fournisseur"
        );
        let second = call(MobileOperation::QuestionShow {
            item_id: items[1].clone(),
        });
        assert_eq!(second["item"]["can_respond"], true);
        Store::open(&f.root.join("mobile-devices.db"))
            .unwrap()
            .revoke(&device, unix_now_secs())
            .unwrap();
        assert_eq!(
            call(response(&items[1], "accept"))["code"],
            "device_revoked"
        );
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_http_task_and_question_controls_use_native_receipts_without_new_mission() {
        use bridget_transport::protocol::{HumanInboxKind, HumanInboxProducer, HumanInboxSubject};
        let f = Fixture::new();
        let (key, device) = {
            let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
            let pair = store
                .issue_pairing(
                    &Scope {
                        all_machines: true,
                        machines: vec![],
                        all_projects: true,
                        projects: vec![],
                        operations: vec![
                            Operation::TaskRead,
                            Operation::TaskCancel,
                            Operation::TaskRespond,
                        ],
                    },
                    unix_now_secs(),
                )
                .unwrap();
            let key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
            let device = store
                .pair(
                    &pair.credential,
                    &Uuid::new_v4().to_string(),
                    &key,
                    "Phone",
                    unix_now_secs(),
                )
                .unwrap()
                .device_id;
            (key, device)
        };
        let task_id = Uuid::new_v4().to_string();
        let question = {
            let st = f.state.lock().unwrap();
            let mut task = crate::delegation::lineage_test_task(
                &task_id,
                &f.agent,
                "instance-1",
                "instance-1",
                &Uuid::new_v4().to_string(),
                None,
                "result_available",
                unix_now_secs(),
            );
            task.root_owner_agent_id = f.agent.clone();
            task.guarded = false;
            task.guard_released = true;
            task.result = Some("Résultat préservé".into());
            st.delegation_store
                .insert(&task, "mobile177-wire-task", b"canonical")
                .unwrap();
            crate::human_inbox::deposit(
                st.store.connection(),
                crate::human_inbox::DepositRequest {
                    dedup_key: "mobile177-wire-question",
                    kind: HumanInboxKind::InterventionRequired,
                    subject: &HumanInboxSubject::default(),
                    context: r#"{"summary":"Question de recette isolée"}"#,
                    options: &["decline".into()],
                    producer: HumanInboxProducer::Daemon,
                    now: unix_now_secs(),
                },
            )
            .unwrap()
            .unwrap()
            .item_id
        };
        let socket = f.root.join("controls.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let state = f.state.clone();
        let native = thread::spawn(move || {
            for _ in 0..8 {
                let (peer, _) = listener.accept().unwrap();
                handle_connection(peer, state.clone()).unwrap();
            }
        });
        tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap().block_on(async {
            let (addr,http)=crate::mobile::http::tests::start_native(&f.root,&socket).await;
            let parse=|raw:String|serde_json::from_str::<serde_json::Value>(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/tasks?agent_id={}",f.agent),Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["tasks"][0]["task_id"],task_id);
            let cancel=serde_json::json!({"type":"task_cancel","device_id":device,"command_id":Uuid::new_v4().to_string(),"task_id":task_id});
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/commands",Some(&key),&cancel,"").await;
            assert_eq!(status,200,"{raw}");let receipt=parse(raw)["result"]["receipt"].clone();
            assert_eq!(receipt["status"],"result_available");
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/tasks/{task_id}/cancel/{}",cancel["command_id"].as_str().unwrap()),Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["receipt"],receipt);
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/commands",Some(&key),&cancel,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["receipt"],receipt);
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET","/v1/questions",Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["items"][0]["item_id"],question);
            let answer=serde_json::json!({"type":"question_respond","device_id":device,"command_id":Uuid::new_v4().to_string(),"item_id":question,"choice":"decline"});
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/commands",Some(&key),&answer,"").await;
            assert_eq!(status,200,"{raw}");let response=parse(raw)["result"]["receipt"].clone();
            assert!(response["application"]["confirmed"].is_null());
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/questions/{question}/responses/{}?choice=decline",answer["command_id"].as_str().unwrap()),
                Some(&key),&serde_json::Value::Null,"").await;
            assert_eq!(status,200,"{raw}");assert_eq!(parse(raw)["result"]["receipt"],response);
            let (status,raw)=crate::mobile::http::tests::request(addr,"POST","/v1/commands",Some(&key),
                &serde_json::json!({"type":"task_pause","task_id":task_id}),"").await;
            assert_eq!(status,400,"{raw}");
            let (status,raw)=crate::mobile::http::tests::request(addr,"GET",
                &format!("/v1/tasks?agent_id={}",f.agent),Some(&f.key),&serde_json::Value::Null,"").await;
            assert_eq!(status,403,"{raw}");
            http.abort();
        });
        native.join().unwrap();
        let st = f.state.lock().unwrap();
        assert_eq!(st.delegation_store.tasks().unwrap().len(), 1);
        assert_eq!(
            st.delegation_store
                .test_connection()
                .query_row(
                    "SELECT count(*) FROM native_delegation_cancel_receipts",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        drop(st);
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_native_refuses_an_envelope_from_another_pairing_without_receipt() {
        let f = Fixture::new();
        let mut op = f.command(&Uuid::new_v4().to_string(), "Keep origin");
        if let MobileOperation::Message { device_id, .. } = &mut op {
            *device_id = Uuid::new_v4().to_string();
        }
        let response = value(handle("phone", f.request(op), &f.state));
        assert_eq!(response["code"], "command_origin_changed");
        assert_eq!(f.deliveries(), 0);
        assert_eq!(
            f.state
                .lock()
                .unwrap()
                .store
                .connection()
                .query_row("SELECT count(*) FROM idempotency_records", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn mobile177_native_rechecks_changed_project_before_creating_any_receipt() {
        let f = Fixture::new();
        f.state
            .lock()
            .unwrap()
            .communication_projects
            .get_mut("conn-1")
            .unwrap()
            .0
            .as_mut()
            .unwrap()
            .root = "/project/b".into();
        let response = value(handle(
            "phone",
            f.request(f.command(&Uuid::new_v4().to_string(), "Work")),
            &f.state,
        ));
        assert_eq!(response["code"], "not_found");
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_native_revocation_and_agent_identity_cannot_inherit_phone_authority() {
        let f = Fixture::new();
        f.state
            .lock()
            .unwrap()
            .conn_names
            .insert("phone".into(), f.agent.clone());
        assert_eq!(
            value(handle(
                "phone",
                f.request(f.command(&Uuid::new_v4().to_string(), "Work")),
                &f.state
            ))["code"],
            "authorization_required"
        );
        f.state.lock().unwrap().conn_names.remove("phone");
        Store::open(&f.root.join("mobile-devices.db"))
            .unwrap()
            .revoke(&f.device, unix_now_secs())
            .unwrap();
        assert_eq!(
            value(handle(
                "phone",
                f.request(f.command(&Uuid::new_v4().to_string(), "Work")),
                &f.state
            ))["code"],
            "device_revoked"
        );
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_guidance_requires_the_paired_device_and_recipient_scope() {
        let f=Fixture::new();
        let mut op=f.command(&Uuid::new_v4().to_string(), "Scoped instruction");
        if let MobileOperation::Message { guide_now, device_id, .. }=&mut op {
            *guide_now=true; *device_id=Uuid::new_v4().to_string();
        }
        let refused=value(handle("phone",f.request(op),&f.state));
        assert_eq!(refused["code"],"command_origin_changed");
        let mut op=f.command(&Uuid::new_v4().to_string(), "Other project");
        if let MobileOperation::Message { guide_now, .. }=&mut op { *guide_now=true; }
        f.state.lock().unwrap().communication_projects.get_mut("conn-1").unwrap().0=
            Some(CommunicationProject { host:"macbook".into(),root:"/other/project".into() });
        let refused=value(handle("phone",f.request(op),&f.state));
        assert_eq!(refused["code"],"not_found");
        assert_eq!(f.deliveries(),0);
    }

    #[test]
    fn mobile177_guidance_is_idempotent_and_distinct_from_normal_send() {
        let f=Fixture::new();
        let id=Uuid::new_v4().to_string();
        let mut op=f.command(&id,"Take this into account");
        if let MobileOperation::Message { guide_now, .. }=&mut op { *guide_now=true; }
        let first=value(handle("phone", f.request(op.clone()), &f.state));
        let replay=value(handle("phone", f.request(op), &f.state));
        // Both calls expose the same durable command, never a replacement.
        assert_eq!(first["issue"]["kind"], "outcome_unknown", "{first}");
        assert!(first["issue"]["delivery_id"].is_string());
        assert_eq!(first,replay);
        assert_eq!(f.deliveries(),1,"{first}");
        let changed=value(handle("phone", f.request(f.command(&id,"Take this into account")), &f.state));
        assert!(serde_json::to_string(&changed).unwrap().contains("envelope_mismatch"));
    }

    #[test]
    fn mobile177_native_socket_replay_and_receipt_share_one_durable_delivery() {
        let f = Fixture::new();
        let socket = f.root.join("s");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let state = f.state.clone();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let (peer, _) = listener.accept().unwrap();
                handle_connection(peer, state.clone()).unwrap();
            }
        });
        let command_id = Uuid::new_v4().to_string();
        let command = f.command(&command_id, "One mission");
        // La réponse reçue est volontairement jetée ; la commande conservée est inchangée.
        let first =
            crate::mobile::client::mobile_operation(&socket, &f.key, command.clone()).unwrap();
        assert_eq!(first["status"], "ok", "Réponse mobile de fixture : {first}");
        let lookup = crate::mobile::client::mobile_operation(
            &socket,
            &f.key,
            MobileOperation::Lookup {
                command_id: command_id.clone(),
            },
        )
        .unwrap();
        let replay = crate::mobile::client::mobile_operation(&socket, &f.key, command).unwrap();
        assert_eq!(lookup["command_id"], command_id);
        assert_eq!(replay["command_id"], command_id);
        assert_eq!(f.deliveries(), 1);
        server.join().unwrap();
    }

    #[test]
    fn mobile177_http_disconnect_after_native_commit_replays_one_delivery() {
        use std::io::{BufRead, BufReader, Write};
        use tokio::io::AsyncWriteExt;
        let f = Fixture::new();
        let backend = f.root.join("backend");
        let backend_listener = UnixListener::bind(&backend).unwrap();
        let front = f.root.join("front");
        let front_listener = UnixListener::bind(&front).unwrap();
        for socket in [&backend, &front] {
            std::fs::set_permissions(socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        let state = f.state.clone();
        let native = thread::spawn(move || {
            for _ in 0..3 {
                let (stream, _) = backend_listener.accept().unwrap();
                handle_connection(stream, state.clone()).unwrap();
            }
        });
        let (committed_tx, committed_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        // Retenir la vraie réponse Unix après commit, avant qu'HTTP puisse la recevoir.
        let proxy = thread::spawn(move || {
            for client in 0..3 {
                let (mut peer, _) = front_listener.accept().unwrap();
                peer.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
                peer.set_write_timeout(Some(Duration::from_secs(4)))
                    .unwrap();
                let mut reader = BufReader::new(peer.try_clone().unwrap());
                let mut connection =
                    crate::communication::client::DaemonConnection::connect(&backend).unwrap();
                for frame in 0..3 {
                    let mut line = String::new();
                    assert!(reader.read_line(&mut line).unwrap() > 0);
                    let request: WrapperToDaemon = serde_json::from_str(&line).unwrap();
                    let response = connection.exchange(&request).unwrap();
                    if client == 0 && frame == 2 {
                        assert!(matches!(
                            response,
                            DaemonToWrapper::MobileOperationResult { .. }
                        ));
                        committed_tx.send(()).unwrap();
                        release_rx.recv_timeout(Duration::from_secs(4)).unwrap();
                    }
                    let encoded = serde_json::to_string(&response).unwrap() + "\n";
                    peer.write_all(encoded.as_bytes()).unwrap();
                }
            }
        });
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let (addr, http) = crate::mobile::http::tests::start_native(&f.root, &front).await;
            let id = Uuid::new_v4().to_string();
            let body = serde_json::json!({
                "type":"message", "device_id":f.device, "command_id":id,
                "issued_at":unix_now_secs(), "agent_id":f.agent, "body":"Exactly one instruction"
            });
            let encoded = body.to_string();
            let raw = format!("POST /v1/commands HTTP/1.1\r\nHost: bridget.example.net\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                f.key, encoded.len(), encoded);
            let mut lost = tokio::net::TcpStream::connect(addr).await.unwrap();
            lost.write_all(raw.as_bytes()).await.unwrap();
            tokio::task::spawn_blocking(move || committed_rx.recv_timeout(Duration::from_secs(4)).unwrap()).await.unwrap();
            assert_eq!(f.deliveries(), 1);
            // Aucun octet de réponse HTTP n'a été lu ; la réponse native est encore retenue.
            drop(lost);
            release_tx.send(()).unwrap();
            let (status, receipt) = crate::mobile::http::tests::request(addr, "GET",
                &format!("/v1/commands/{id}"), Some(&f.key), &serde_json::Value::Null, "").await;
            assert_eq!(status, 200);
            assert!(receipt.contains(&id));
            let (status, replay) = crate::mobile::http::tests::request(addr, "POST",
                "/v1/commands", Some(&f.key), &body, "").await;
            assert_eq!(status, 200);
            assert!(replay.contains(&id));
            assert_eq!(f.deliveries(), 1);
            http.abort();
        });
        proxy.join().unwrap();
        native.join().unwrap();
    }

    #[test]
    fn mobile177_http_directory_filters_real_native_neighbours_before_pagination() {
        let f = Fixture::new();
        {
            let mut st = f.state.lock().unwrap();
            let kind = st.router.get_agent(&f.agent).unwrap().agent_type.clone();
            for (n, name, host, root) in [
                (
                    2,
                    "80000000-0000-4000-8000-000000000102",
                    "macbook",
                    "/project/b",
                ),
                (
                    3,
                    "80000000-0000-4000-8000-000000000103",
                    "cartae",
                    "/project/a",
                ),
            ] {
                let conn = format!("conn-{n}");
                let instance = format!("instance-{n}");
                st.router.register(name, &kind, &conn).unwrap();
                let mut presence = st.presences["instance-1"].clone();
                presence.name = name.into();
                presence.host = host.into();
                st.presences.insert(instance.clone(), presence);
                st.conn_names.insert(conn.clone(), name.into());
                st.conn_instances.insert(conn.clone(), instance);
                st.communication_projects.insert(
                    conn,
                    (
                        Some(CommunicationProject {
                            host: host.into(),
                            root: root.into(),
                        }),
                        None,
                        false,
                    ),
                );
            }
            st.conn_counter = 3;
        }
        let socket = f.root.join("directory");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let state = f.state.clone();
        let native = thread::spawn(move || {
            for _ in 0..5 {
                let (peer, _) = listener.accept().unwrap();
                handle_connection(peer, state.clone()).unwrap();
            }
        });
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let (addr, http) = crate::mobile::http::tests::start_native(&f.root, &socket).await;
                for (path, expected) in [
                    ("/v1/agents?limit=1", 1usize),
                    ("/v1/projects?limit=1", 1),
                    ("/v1/agents?machine=cartae", 0),
                    (
                        "/v1/agents?project_host=macbook&project_root=%2Fproject%2Fb",
                        0,
                    ),
                    ("/v1/federation", 1),
                ] {
                    let (status, body) = crate::mobile::http::tests::request(
                        addr,
                        "GET",
                        path,
                        Some(&f.key),
                        &serde_json::Value::Null,
                        "",
                    )
                    .await;
                    assert_eq!(status, 200, "{path}: {body}");
                    let value: serde_json::Value =
                        serde_json::from_str(body.split("\r\n\r\n").nth(1).unwrap()).unwrap();
                    assert_eq!(value["result"]["items"].as_array().unwrap().len(), expected);
                    assert!(!body.contains("80000000-0000-4000-8000-000000000102"));
                    assert!(!body.contains("80000000-0000-4000-8000-000000000103"));
                    assert!(!body.contains("/project/b"));
                    assert!(!body.contains("cartae"));
                    assert!(!body.contains(&f.key));
                    if path == "/v1/agents?limit=1" {
                        assert_eq!(value["result"]["items"][0]["agent_id"], f.agent);
                        assert!(value["result"]["next_cursor"].is_null());
                    }
                }
                http.abort();
            });
        native.join().unwrap();
        assert_eq!(f.deliveries(), 0);
    }

    #[test]
    fn mobile177_native_receipt_cannot_inject_private_refusal_text() {
        let id = Uuid::new_v4().to_string();
        let response = value(public_receipt(
            DaemonToWrapper::IdempotencyResult {
                project_warnings: vec![],
                operation_kind: "send".into(),
                idempotency_key: id.clone(),
                issue: IdempotencyIssue::Rejected {
                    category: "native_refused".into(),
                    reason: "private-canary-argument".into(),
                    expires_at: 100,
                },
            },
            &id,
        ));
        assert!(!response.to_string().contains("private-canary"));
        assert_eq!(response["issue"]["category"], "native_refused");
    }
    #[test]
    fn mobile177_native_federation_keeps_stale_machine_and_hides_ungranted_hosts() {
        let f = Fixture::new();
        {
            let mut st = f.state.lock().unwrap();
            st.remote_workers.insert(
                "macbook".into(),
                RemoteWorkerConnection {
                    identity: bridget_transport::protocol::WorkerIdentity {
                        machine_id: "macbook".into(),
                        worker_id: "fixture-worker".into(),
                        provisioning_revision: 1,
                    },
                    conn_id: "offline-fixture".into(),
                    epoch: "fixture-epoch".into(),
                    capabilities: vec![],
                    last_contact: Instant::now() - Duration::from_secs(60),
                    last_contact_at: unix_now_secs() - 60,
                    contact: bridget_transport::protocol::WorkerContactState::Stale,
                },
            );
        }
        let reply = value(handle(
            "phone",
            f.request(MobileOperation::Federation {
                after: None,
                limit: 50,
            }),
            &f.state,
        ));
        assert_eq!(reply["status"], "ok");
        assert_eq!(reply["items"].as_array().unwrap().len(), 1);
        assert_eq!(reply["items"][0]["machine"], "macbook");
        assert_eq!(reply["items"][0]["availability"], "stale");
        assert!(!reply.to_string().contains("fixture-worker"));
        assert!(!reply.to_string().contains("credential"));
        assert_eq!(f.deliveries(), 0);
    }
    #[test]
    fn mobile177_native_journal_pages_and_gaps_are_preserved_without_expanding_scope() {
        use bridget_transport::journal::SessionJournal;
        use bridget_transport::protocol::AttachWindow;
        let mut f = Fixture::new();
        { let mut st=f.state.lock().unwrap();st.host=st.presences["instance-1"].host.clone(); }
        let mut journal =
            SessionJournal::new(f.root.join("sessions"), &f.agent, "fixture-session").unwrap();
        journal
            .append(
                "update",
                None,
                serde_json::json!({"kind":"text","text":"First event"}),
            )
            .unwrap();
        journal
            .append(
                "update",
                None,
                serde_json::json!({"kind":"text","text":"Third event"}),
            )
            .unwrap();
        drop(journal);
        let operation = MobileOperation::AgentJournal {
            agent_id: f.agent.clone(),
            after_seq: 0,
            limit: 50,
        tail: false,};
        let partial = value(handle("phone", f.request(operation.clone()), &f.state));
        assert_eq!(partial["code"], "history_scope_unavailable");
        assert!(!partial.to_string().contains("First event"));
        let mut store = Store::open(&f.root.join("mobile-devices.db")).unwrap();
        let grant = store
            .issue_pairing(
                &Scope {
                    all_machines: true,
                    machines: vec![],
                    all_projects: true,
                    projects: vec![],
                    operations: vec![Operation::Read],
                },
                unix_now_secs(),
            )
            .unwrap();
        f.key = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        f.device = store
            .pair(
                &grant.credential,
                &Uuid::new_v4().to_string(),
                &f.key,
                "History phone",
                unix_now_secs(),
            )
            .unwrap()
            .device_id;
        let directory = f.root.join("sessions").join(&f.agent);
        let path = std::fs::read_dir(directory)
            .unwrap()
            .filter_map(Result::ok)
            .find(|entry| entry.path().extension().is_some_and(|e| e == "jsonl"))
            .unwrap()
            .path();
        let original = std::fs::read_to_string(&path).unwrap();
        let mut lines = original
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        lines[1]["seq"] = serde_json::json!(3);
        std::fs::write(
            &path,
            lines
                .iter()
                .map(|line| line.to_string() + "\n")
                .collect::<String>(),
        )
        .unwrap();
        let first = value(handle("phone", f.request(operation), &f.state));
        assert_eq!(first["status"], "ok");
        assert_eq!(first["items"].as_array().unwrap().len(), 1);
        assert_eq!(first["items"][0]["seq"], 1);
        assert_eq!(first["gap"]["from_seq"], 2);
        assert_eq!(first["next_seq"], 2);
        let second = value(handle(
            "phone",
            f.request(MobileOperation::AgentJournal {
                agent_id: f.agent.clone(),
                after_seq: 2,
                limit: 50,
            tail: false,}),
            &f.state,
        ));
        assert_eq!(second["items"][0]["seq"], 3);
        assert!(second["caught_up"].as_bool().unwrap());
        // Le fichier homonyme du coordinateur ne peut remplacer le journal étranger.
        let socket=f.root.join("origin-journal.sock");
        let listener=std::os::unix::net::UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket,std::fs::Permissions::from_mode(0o600)).unwrap();
        {let mut st=f.state.lock().unwrap();st.host="coordinator177".into();st.socket_path=socket;}
        let producer=std::thread::spawn(move||{
            use std::io::{BufRead,BufReader,Write};
            for case in 0..4 {
            let (mut stream,_)=listener.accept().unwrap();let mut reader=BufReader::new(stream.try_clone().unwrap());
            for response in [DaemonToWrapper::RoleAccepted {role:ConnectionRole::Attach},
                DaemonToWrapper::Subscribed {subscription_id:"remote177".into()}] {
                let mut line=String::new();reader.read_line(&mut line).unwrap();
                if matches!(&response,DaemonToWrapper::Subscribed {..}) {
                    let request:WrapperToDaemon=serde_json::from_str(&line).unwrap();
                    let expected=if case==2 {AttachWindow::Tail(1)} else {AttachWindow::Seq(if case==3 {251} else {1})};
                    assert!(matches!(request,WrapperToDaemon::Subscribe {window,..} if window==expected));
                }
                writeln!(stream,"{}",serde_json::to_string(&response).unwrap()).unwrap();
            }
            let mut responses=Vec::new();
            let range=match case {0=>1..=1,1=>1..=0,2=>300..=300,_=>251..=300};
            for seq in range { responses.push(DaemonToWrapper::JournalFragment {subscription_id:"remote177".into(),seq,offset:0,final_fragment:true,
                bytes:serde_json::to_vec(&serde_json::json!({"seq":seq,"text":"Foreign journal event"})).unwrap()}); }
            responses.push(DaemonToWrapper::SnapshotCaughtUp {subscription_id:"remote177".into(),through_seq:match case {0=>Some(1),1=>None,_=>Some(300)}});
            for response in responses {writeln!(stream,"{}",serde_json::to_string(&response).unwrap()).unwrap();}
            }
        });
        let operation=||MobileOperation::AgentJournal {agent_id:f.agent.clone(),after_seq:0,limit:50,tail: false,};
        let foreign=value(handle("phone",f.request(operation()),&f.state));
        assert_eq!(foreign["items"][0]["text"],"Foreign journal event");
        assert!(!foreign.to_string().contains("First event"));
        let empty=value(handle("phone",f.request(operation()),&f.state));
        assert_eq!(empty["items"].as_array().unwrap().len(),0);
        assert_eq!(empty["next_seq"],0);
        assert_eq!(empty["caught_up"],true,"an empty acknowledged snapshot must keep long polling enabled");
        let tail=value(handle("phone",f.request(MobileOperation::AgentJournal {agent_id:f.agent.clone(),after_seq:0,limit:50,tail:true}),&f.state));
        assert_eq!(tail["items"].as_array().unwrap().len(),50);
        assert_eq!(tail["items"][0]["seq"],251);
        assert_eq!(tail["items"][49]["seq"],300);
        assert_eq!(tail["next_seq"],300);
        assert_eq!(tail["older_before_seq"],251);
        producer.join().unwrap();
        let unavailable=value(handle("phone",f.request(operation()),&f.state));
        assert_eq!(unavailable["code"],"journal_unavailable");
        assert!(!unavailable.to_string().contains("First event"));
        store.revoke(&f.device, unix_now_secs()).unwrap();
        assert_eq!(
            value(handle(
                "phone",
                f.request(MobileOperation::AgentJournal {
                    agent_id: f.agent.clone(),
                    after_seq: 2,
                    limit: 50,
                tail: false,}),
                &f.state
            ))["code"],
            "device_revoked"
        );
    }
}
