//! Tests de la garde, du watch et du journal lineage 149. Ce fichier est un
//! module de test autonome : le principal le câble en fin de
//! `daemon/native_lineage.rs` avec `#[cfg(test)]
//! #[path = "native_lineage_tests.rs"] mod native_lineage_tests;`. Enfant du
//! module native_lineage, il voit les fonctions privées du daemon sans ajouter
//! aucun code dans la production.
use super::*;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

const T3: &str = "89000000-0000-4000-8000-000000000149";

// ---------------------------------------------------------------------------
// Fixtures locales — aucun fournisseur, aucun processus daemon séparé.
// ---------------------------------------------------------------------------

fn definition() -> bridget_transport::protocol::ResolvedAgentDefinition {
    bridget_transport::protocol::ResolvedAgentDefinition {
        command: "npx".into(),
        args: vec!["fixture-lineage149".into()],
        protocol: "acp".into(),
        forbidden_env: vec!["API_KEY".into()],
        pass_env: Vec::new(),
        claude_config_dir: None,
        permissions: "allow".into(),
        queue_capacity: 32,
        notify_timeout_secs: 600,
        mcp: bridget_transport::ResolvedMcpDefinition {
            interactive: "none".into(),
            acp_session: false,
        },
        capabilities: bridget_transport::AdapterCapabilities::default(),
        digest: "fixture-lineage149".into(),
    }
}

/// Tâche native : racine portée par le propriétaire, enfant UUID stable pour
/// que journal_directory accepte de résoudre sessions/<child>.
fn lineage_task(task_id: &str, owner: &str, state: &str) -> crate::delegation::Task {
    crate::delegation::Task {
        substitution: None,
        remote_placement: None,
        remote_prepared: false,
        remote_result_slots: Vec::new(),
        remote_result_receipts: Vec::new(),
        task_id: task_id.into(),
        root_owner_agent_id: String::new(),
        parent_task_id: None,
        updated_at: 0,
        started_at: None,
        completed_at: None,
        owner: owner.into(),
        owner_instance: format!("inst-{owner}"),
        origin_owner_instance: format!("inst-{owner}"),
        parent_execution_id: None,
        request: bridget_transport::protocol::NativeDelegationRequest::Delegate {
            machine_id: None,
            project_id: None,
            resume_on_limit: None,
            fallback: Vec::new(),
            request_id: format!("req-{task_id}"),
            agent_type: "fixture".into(),
            model: "fixture-model".into(),
            effort: None,
            task: format!("Mission {task_id}\n"),
            cwd: "/tmp".into(),
            posture: None,
            reply_timeout: None,
            child_requests: None,
            workspace: None,
            keep_alive_secs: None,
            brief: None,
        },
        permission_snapshot: None,
        effective_posture: None,
        definition: definition(),
        cwd: "/tmp".into(),
        child_requests: None,
        workspace: None,
        child: crate::t3code::stable_uuid(&format!("child:{task_id}")),
        workspace_report_mission: None,
        child_instance: Some(crate::t3code::stable_uuid(&format!(
            "child-instance:{task_id}"
        ))),
        mission: format!("mission-{task_id}"),
        mission_deadline_at: None,
        created_at: 100,
        state: state.into(),
        result: None,
        error: None,
        result_sent: false,
        cleanup_done: false,
        failure_sent: false,
        reply_timeout: None,
        delivery_acked: false,
        acknowledged_at: None,
        round: 0,
        idle_deadline_at: None,
        round_body: None,
        rounds: Vec::new(),
        guarded: false,
        guard_released: false,
        unacked_rounds: Vec::new(),
        quota_waits: Vec::new(),
        quota_hold: None,
        launch_issued_at: None,
        mission_issued_at: None,
        restart_resumes: 0,
        cancel_stopped: Vec::new(),
    }
}

/// Même fixture que la vue humaine 145 : présence T3 enregistrée, rôle client,
/// négociation exacte d'une capacité, projet annoncé et liaison de fil.
fn fixture(label: &str, capability: &str) -> (Arc<Mutex<DaemonState>>, String, UnixStream) {
    let (mut st, _) = crate::daemon::presence_tests::state_with_registered_agent(label);
    st.conn_counter = 1;
    let peer = spec094_live_test_connection(&mut st, "t3");
    let host = st.host.clone();
    assert!(matches!(
        handle_register_with_channel(
            "t3",
            2,
            "fixture".into(),
            crate::t3code::stable_uuid(T3),
            Some(host.clone()),
            Some("t3code".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Cli),
            None,
            Some("test".into()),
            Some(crate::t3code::stable_uuid(&format!("instance:{T3}"))),
            None,
            false,
            Some(false),
            &mut st,
            false
        ),
        DaemonToWrapper::Registered { .. }
    ));
    st.connection_roles
        .insert("human".into(), ConnectionRole::Client);
    st.client_negotiations.insert(
        "human".into(),
        NegotiatedClient {
            version: 1,
            issuer_scope: format!("spec149-{label}"),
            capabilities: vec![serde_json::from_value(json!(capability)).unwrap()],
        },
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let state = Arc::new(Mutex::new(st));
    assert!(matches!(
        announce_communication_project(
            &state,
            "t3",
            &root,
            bridget_transport::protocol::CommunicationProjectSource::T3,
            &host,
            None
        ),
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
    let fact =
        serde_json::from_value(json!({"type":"T3ThreadBindingFact","version":1,"t3_thread_id":T3}))
            .unwrap();
    assert!(handle_wrapper_message("t3", fact, &state).is_none());
    (state, root, peer)
}

fn lineage_request(root: &str, action: Value) -> WrapperToDaemon {
    serde_json::from_value(json!({
        "type":"HumanLineage",
        "request":{"version":1,"t3_thread_id":T3,"project_root":root,"request":action}
    }))
    .unwrap()
}

fn call(state: &Arc<Mutex<DaemonState>>, root: &str, action: Value) -> Value {
    let response = handle_wrapper_message("human", lineage_request(root, action), state).unwrap();
    serde_json::to_value(response).unwrap()["result"].clone()
}

fn owner_agent() -> String {
    crate::t3code::stable_uuid(T3)
}

// ---------------------------------------------------------------------------
// Fermeture des formes ouvertes en entrée.
// ---------------------------------------------------------------------------

#[test]
fn native149_validate_ferme_les_formes_ouvertes() {
    let parse = |v: Value| -> bridget_transport::protocol::HumanLineageRequest {
        serde_json::from_value(v).unwrap()
    };
    let base = json!({"version":1,"t3_thread_id":T3,"project_root":"/tmp",
        "request":{"action":"list","limit":10,"cursor":null}});
    assert!(validate(&parse(base.clone())).is_ok());

    let mut version = base.clone();
    version["version"] = json!(3);
    assert!(matches!(
        validate(&parse(version)),
        Err(E::UnsupportedVersion)
    ));

    let mut formes: Vec<Value> = Vec::new();
    let mut vide = base.clone();
    vide["t3_thread_id"] = json!("");
    formes.push(vide);
    let mut long = base.clone();
    long["t3_thread_id"] = json!("x".repeat(2049));
    formes.push(long);
    let mut controle = base.clone();
    controle["t3_thread_id"] = json!(format!("fil\n{T3}"));
    formes.push(controle);
    let mut racine = base.clone();
    racine["project_root"] = json!("relative/path");
    formes.push(racine);
    let mut racine_longue = base.clone();
    racine_longue["project_root"] = json!("x".repeat(4097));
    formes.push(racine_longue);
    let mut liste = base.clone();
    liste["request"] = json!({"action":"list","limit":0,"cursor":null});
    formes.push(liste);
    let mut liste = base.clone();
    liste["request"] = json!({"action":"list","limit":101,"cursor":null});
    formes.push(liste);
    let mut liste = base.clone();
    liste["request"] = json!({"action":"list","limit":10,"cursor":"x".repeat(2049)});
    formes.push(liste);
    let mut liste = base.clone();
    liste["request"] = json!({"action":"list","limit":10,"cursor":format!("curseur\u{7}")});
    formes.push(liste);
    let id = "14900000-0000-4000-8000-000000000001";
    for borne in [0u32, 16385] {
        let mut show = base.clone();
        show["request"] = json!({"action":"show","task_id":id,"offset":0,"limit":borne});
        formes.push(show);
    }
    let mut show = base.clone();
    show["request"] = json!({"action":"show","task_id":"pas-un-uuid","offset":0,"limit":100});
    formes.push(show);
    for borne in [0u32, 101] {
        let mut journal = base.clone();
        journal["request"] =
            json!({"action":"journal","task_id":id,"after_seq":0,"limit":borne,"follow":false});
        formes.push(journal);
    }
    let mut journal = base.clone();
    journal["request"] =
        json!({"action":"journal","task_id":"pas-un-uuid","after_seq":0,"limit":50,"follow":false});
    formes.push(journal);
    let mut journal = base.clone();
    journal["request"] = json!({"action":"journal","task_id":id,"after_seq":bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ + 1,"limit":50,"follow":false});
    formes.push(journal);
    for (task, demande) in [("pas-un-uuid", id), (id, "pas-un-uuid")] {
        let mut cancel = base.clone();
        cancel["request"] = json!({"action":"cancel","task_id":task,"request_id":demande});
        formes.push(cancel);
    }
    for forme in &formes {
        assert!(
            matches!(validate(&parse(forme.clone())), Err(E::InvalidRequest)),
            "forme acceptée à tort : {forme}"
        );
    }

    // Le journal en suivi passe la fermeture ; c'est la garde qui décide après.
    let mut suivi = base;
    suivi["request"] =
        json!({"action":"journal","task_id":id,"after_seq":0,"limit":50,"follow":true});
    assert!(validate(&parse(suivi)).is_ok());
}

// ---------------------------------------------------------------------------
// Gardes client et autorité : refus avant toute lecture du magasin.
// ---------------------------------------------------------------------------

#[test]
fn native149_gardes_client_et_authorite() {
    let (state, root, _peer) = fixture("l149-guard", "human_lineage_view_v1");
    {
        let mut st = state.lock().unwrap();
        st.connection_roles.remove("human");
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "binding_unavailable"
    );
    {
        let mut st = state.lock().unwrap();
        st.connection_roles
            .insert("human".into(), ConnectionRole::Client);
        st.client_negotiations.remove("human");
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "unsupported_version"
    );
    // Capacité annoncée ≠ capacité demandée.
    {
        let mut st = state.lock().unwrap();
        st.client_negotiations.insert(
            "human".into(),
            NegotiatedClient {
                version: 1,
                issuer_scope: "l149".into(),
                capabilities: vec![
                    serde_json::from_value(json!("human_lineage_watch_v1")).unwrap(),
                ],
            },
        );
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "unsupported_version"
    );
    // Deux capacités casse l'égalité exacte exigée par la garde.
    {
        let mut st = state.lock().unwrap();
        st.client_negotiations.insert(
            "human".into(),
            NegotiatedClient {
                version: 1,
                issuer_scope: "l149".into(),
                capabilities: vec![
                    serde_json::from_value(json!("human_lineage_view_v1")).unwrap(),
                    serde_json::from_value(json!("human_lineage_watch_v1")).unwrap(),
                ],
            },
        );
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "unsupported_version"
    );
    {
        let mut st = state.lock().unwrap();
        st.client_negotiations.insert(
            "human".into(),
            NegotiatedClient {
                version: 1,
                issuer_scope: "l149".into(),
                capabilities: vec![serde_json::from_value(json!("human_lineage_view_v1")).unwrap()],
            },
        );
    }
    // Autorité : liaison absente, puis doublon sur le même fil.
    {
        let mut st = state.lock().unwrap();
        st.t3_thread_bindings.remove("t3");
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "binding_unavailable"
    );
    {
        let mut st = state.lock().unwrap();
        st.t3_thread_bindings.insert("t3".into(), Some(T3.into()));
        st.t3_thread_bindings
            .insert("t3-bis-149".into(), Some(T3.into()));
    }
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["code"],
        "binding_unavailable"
    );
    {
        let mut st = state.lock().unwrap();
        st.t3_thread_bindings.remove("t3-bis-149");
    }
    // Projet annoncé ≠ projet demandé.
    assert_eq!(
        call(
            &state,
            "/bridget-149-root-absent",
            json!({"action":"list","limit":10})
        )["code"],
        "project_mismatch"
    );
    // Le chemin légal redevient servi.
    assert_eq!(
        call(&state, &root, json!({"action":"list","limit":10}))["status"],
        "ok"
    );
}

// ---------------------------------------------------------------------------
// List, show et journal sous garde : racines séparées, journal résolu.
// ---------------------------------------------------------------------------

#[test]
fn native149_list_show_journal_et_disponibilite_journal() {
    let (state, root, _peer) = fixture("l149-view", "human_lineage_view_v1");
    let task_id = "14900000-0000-4000-8000-000000000001";
    // `sessions/<child>` vit sous le parent du db_path, partagé entre tests et
    // exécutions : un reliquat rendrait le journal disponible à tort.
    let stale = state
        .lock()
        .unwrap()
        .db_path
        .parent()
        .unwrap()
        .join("sessions")
        .join(crate::t3code::stable_uuid(&format!("child:{task_id}")));
    let _ = std::fs::remove_dir_all(&stale);
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(task_id, &owner_agent(), "queued"),
                "req-v1",
                b"c",
            )
            .unwrap();
        let mut autre = lineage_task(
            "14900000-0000-4000-8000-0000000000aa",
            "autre-racine-149",
            "queued",
        );
        autre.mission = "mission-autre-149".into();
        st.delegation_store.insert(&autre, "req-aa", b"c").unwrap();
    }
    let liste = call(&state, &root, json!({"action":"list","limit":10}));
    assert_eq!(liste["status"], "ok");
    assert_eq!(liste["root_owner_agent_id"], owner_agent());
    let tasks = liste["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["task_id"], task_id);
    assert_eq!(tasks[0]["journal_available"], false);

    let show = call(
        &state,
        &root,
        json!({"action":"show","task_id":task_id,"offset":0,"limit":100}),
    );
    assert_eq!(show["status"], "ok");
    assert!(show["result"].is_null(), "tâche non terminale : {show}");

    // Tâche d'une autre racine : même refus fermé qu'une tâche absente.
    for absent in [
        "14900000-0000-4000-8000-0000000000aa",
        "00000000-0000-4000-8000-000000000000",
    ] {
        let show = call(
            &state,
            &root,
            json!({"action":"show","task_id":absent,"offset":0,"limit":100}),
        );
        assert_eq!(show["code"], "task_unavailable", "{show}");
    }

    // Journal sans dossier : refus fermé.
    let journal = call(
        &state,
        &root,
        json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":50,"follow":false}),
    );
    assert_eq!(journal["code"], "journal_unavailable", "{journal}");

    // Le dossier sessions/<child> privé rend le journal disponible.
    let (child, directory) = {
        let st = state.lock().unwrap();
        let child = st.delegation_store.get(task_id).unwrap().unwrap().child;
        let directory = st.db_path.parent().unwrap().join("sessions").join(&child);
        (child, directory)
    };
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&directory)
        .unwrap();
    let liste = call(&state, &root, json!({"action":"list","limit":10}));
    assert_eq!(liste["tasks"][0]["journal_available"], true);

    {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join(format!(
                "{}.jsonl",
                bridget_transport::journal::current_host_date()
            )))
            .unwrap();
        writeln!(
            &file,
            "{}",
            json!({"v":1,"seq":1,"event":"note","payload":{"text":"étape un"}})
        )
        .unwrap();
        writeln!(
            &file,
            "{}",
            json!({"v":1,"seq":2,"event":"note","payload":{"text":"étape deux"}})
        )
        .unwrap();
    }
    let journal = call(
        &state,
        &root,
        json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":50,"follow":false}),
    );
    assert_eq!(journal["status"], "ok", "{journal}");
    assert_eq!(journal["events"].as_array().unwrap().len(), 2);
    assert_eq!(journal["next_seq"], 2);
    assert_eq!(journal["caught_up"], true);
    let journal = call(
        &state,
        &root,
        json!({"action":"journal","task_id":task_id,"after_seq":2,"limit":50,"follow":false}),
    );
    assert_eq!(journal["events"].as_array().unwrap().len(), 0);

    // Aucune mutation ne doit avoir traversé ces lectures.
    let st = state.lock().unwrap();
    assert_eq!(st.delegation_store.projection_meta().unwrap().seq, 2);
    assert!(st.lineage_watches.is_empty());
    assert_eq!(st.t3_thread_bindings.get("t3"), Some(&Some(T3.into())));
    drop(st);
    let _ = child;
    let _ = std::fs::remove_dir_all(&directory);
}

// ---------------------------------------------------------------------------
// Cancel sous garde : reçu, rejeu, mismatch, tâche terminale.
// ---------------------------------------------------------------------------

#[test]
fn native149_cancel_recu_rejeu_et_mismatch_via_flux() {
    let (state, root, _peer) = fixture("l149-cancel", "human_lineage_cancel_v1");
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(
                    "14900000-0000-4000-8000-000000000003",
                    &owner_agent(),
                    "queued",
                ),
                "req-c3",
                b"c",
            )
            .unwrap();
        st.delegation_store
            .insert(
                &lineage_task(
                    "14900000-0000-4000-8000-000000000004",
                    &owner_agent(),
                    "result_available",
                ),
                "req-c4",
                b"c",
            )
            .unwrap();
    }
    let task_id = "14900000-0000-4000-8000-000000000003";
    let request_id = "49000000-0000-4000-8000-000000000009";
    let cancel = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":task_id,"request_id":request_id}),
    );
    assert_eq!(
        cancel,
        json!({"version":1,"task_id":task_id,"status":"cancelling"})
    );
    // Le reçu est rejoué à l'identique.
    let rejoue = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":task_id,"request_id":request_id}),
    );
    assert_eq!(rejoue, cancel);
    // Même request_id sur une autre tâche : mismatch d'enveloppe.
    let mismatch = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":"14900000-0000-4000-8000-000000000004","request_id":request_id}),
    );
    assert_eq!(mismatch["code"], "envelope_mismatch", "{mismatch}");
    // Tâche terminale : reçu sans mutation.
    let terminal = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":"14900000-0000-4000-8000-000000000004","request_id":"49000000-0000-4000-8000-000000000008"}),
    );
    assert_eq!(terminal["status"], "result_available", "{terminal}");
}

// ---------------------------------------------------------------------------
// Watch sur vraie socket : Ready seq 0, Changed, silences, Resync, invalidation.
// ---------------------------------------------------------------------------

fn open_watch_socket(
    state: &Arc<Mutex<DaemonState>>,
    root: &str,
) -> (
    UnixStream,
    BufReader<UnixStream>,
    thread::JoinHandle<Result<(), String>>,
) {
    let (mut client, server) = UnixStream::pair().unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let shared = state.clone();
    let handle =
        thread::spawn(move || handle_connection(server, shared).map_err(|e| e.to_string()));
    let mut reader = BufReader::new(client.try_clone().unwrap());
    for request in [
        json!({"type":"RoleHandshake","role":"client"}),
        json!({"type":"ClientHello","contract_version":1,"issuer_scope":"spec149-lineage-watch-00000","capabilities":["human_lineage_watch_v1"]}),
    ] {
        writeln!(client, "{request}").unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(
            serde_json::from_str::<Value>(&line).unwrap()["type"] != "ClientRejected",
            "{line}"
        );
    }
    writeln!(
        client,
        "{}",
        json!({"type":"HumanLineage","request":{"version":1,"t3_thread_id":T3,"project_root":root,"request":{"action":"watch"}}})
    )
    .unwrap();
    (client, reader, handle)
}

fn next_event(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    assert!(reader.read_line(&mut line).unwrap() > 0, "flux watch fermé");
    serde_json::from_str::<Value>(&line).unwrap()["event"].clone()
}

fn silent(reader: &mut BufReader<UnixStream>) {
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(400)))
        .unwrap();
    let mut line = String::new();
    let result = reader.read_line(&mut line);
    assert!(result.is_err(), "événement inattendu : {line}");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
}

#[test]
fn native149_watch_ready_changed_silence_resync_et_invalidation() {
    let (state, root, _peer) = fixture("l149-watch", "human_lineage_watch_v1");
    // Graine AVANT le watch : la projection démarre à seq 1, Ready reste seq 0.
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(
                    "14900000-0000-4000-8000-00000000000a",
                    &owner_agent(),
                    "queued",
                ),
                "req-wa",
                b"c",
            )
            .unwrap();
    }
    let generation = {
        let st = state.lock().unwrap();
        st.delegation_store.projection_meta().unwrap().generation
    };
    let (client, mut reader, handle) = open_watch_socket(&state, &root);
    let ready = next_event(&mut reader);
    assert_eq!(ready["status"], "ready", "{ready}");
    assert_eq!(
        ready["seq"], 0,
        "le marqueur Ready est distinct de la seq du magasin"
    );
    assert_eq!(ready["generation"], generation);

    // Une mutation de la racine suivie devient un Changed croissant.
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(
                    "14900000-0000-4000-8000-00000000000b",
                    &owner_agent(),
                    "queued",
                ),
                "req-wb",
                b"c",
            )
            .unwrap();
    }
    let changed = next_event(&mut reader);
    assert_eq!(changed["status"], "changed", "{changed}");
    assert_eq!(changed["seq"], 2);
    assert_eq!(changed["generation"], generation);

    // Lectures et save muet : aucun événement.
    for _ in 0..5 {
        call(&state, &root, json!({"action":"list","limit":10}));
    }
    {
        let st = state.lock().unwrap();
        let identique = st
            .delegation_store
            .get("14900000-0000-4000-8000-00000000000a")
            .unwrap()
            .unwrap();
        st.delegation_store.save(&identique).unwrap();
    }
    silent(&mut reader);
    // Une mutation d'une autre racine ne traverse pas ce watch.
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(
                    "14900000-0000-4000-8000-0000000000xc",
                    "autre-racine-149",
                    "queued",
                ),
                "req-wc",
                b"c",
            )
            .unwrap();
    }
    silent(&mut reader);

    // Resync déterministe : publication directe d'une génération divergente.
    {
        let mut st = state.lock().unwrap();
        publish(
            &mut st,
            &owner_agent(),
            &crate::delegation::ProjectionMutation {
                generation: "88888888-8888-4888-8888-888888888888".into(),
                seq: 900,
            },
        );
    }
    let resync = next_event(&mut reader);
    assert_eq!(resync["status"], "resync", "{resync}");
    assert_eq!(resync["generation"], "88888888-8888-4888-8888-888888888888");
    assert_eq!(resync["seq"], 900);

    // Double liaison : la garde ferme le watch avec une erreur puis EOF.
    {
        let mut st = state.lock().unwrap();
        st.t3_thread_bindings
            .insert("t3-double-149".into(), Some(T3.into()));
        invalidate(&mut st);
    }
    let error = next_event(&mut reader);
    assert_eq!(error["status"], "error", "{error}");
    assert_eq!(error["code"], "binding_unavailable");
    let mut line = String::new();
    assert_eq!(
        reader.read_line(&mut line).unwrap(),
        0,
        "watch non fermé : {line}"
    );
    drop(reader);
    drop(client);
    drop(handle); // handle_connection est libéré par la fermeture ; sans assertion.
    let st = state.lock().unwrap();
    assert!(st.lineage_watches.is_empty());
}

#[test]
fn native149_watch_refuse_sans_socket_ou_sans_capacite() {
    // Négociation view seulement : le watch est refusé avant la socket.
    let (state, root, _peer) = fixture("l149-watchcap", "human_lineage_view_v1");
    let response = handle_wrapper_message(
        "human",
        lineage_request(&root, json!({"action":"watch"})),
        &state,
    )
    .unwrap();
    let event = serde_json::to_value(response).unwrap()["event"].clone();
    assert_eq!(event["status"], "error", "{event}");
    assert_eq!(event["code"], "unsupported_version");
    assert!(state.lock().unwrap().lineage_watches.is_empty());

    // Négociation exacte mais aucune socket cliente : refus de liaison.
    let (state, root, _peer) = fixture("l149-watchnosock", "human_lineage_watch_v1");
    let response = handle_wrapper_message(
        "human",
        lineage_request(&root, json!({"action":"watch"})),
        &state,
    )
    .unwrap();
    let event = serde_json::to_value(response).unwrap()["event"].clone();
    assert_eq!(event["status"], "error", "{event}");
    assert_eq!(event["code"], "binding_unavailable");
    assert!(state.lock().unwrap().lineage_watches.is_empty());
}

// ---------------------------------------------------------------------------
// Page journal directe : bornes, permissions, lacunes, volume.
// ---------------------------------------------------------------------------

fn journal_tree(label: &str) -> std::path::PathBuf {
    let base = std::env::temp_dir().join(format!(
        "bridget-l149-journal-{label}-{}",
        uuid::Uuid::new_v4()
    ));
    let directory = base.join("sessions").join("child-149");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&directory)
        .unwrap();
    directory
}

fn journal_file(directory: &std::path::Path) -> std::path::PathBuf {
    directory.join(format!(
        "{}.jsonl",
        bridget_transport::journal::current_host_date()
    ))
}

fn write_events(directory: &std::path::Path, lines: &[Value]) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(journal_file(directory))
        .unwrap();
    for line in lines {
        writeln!(file, "{line}").unwrap();
    }
}

#[test]
fn native149_page_journal_bornes_permissions_et_fermeture() {
    use crate::attach::lineage_journal_page;
    let task_id = "14900000-0000-4000-8000-000000000002";
    for limite in [0u32, 101] {
        let directory = journal_tree("bornes");
        assert!(matches!(
            lineage_journal_page(&directory, task_id, 0, limite),
            Err(E::InvalidRequest)
        ));
        std::fs::remove_dir_all(directory.parent().unwrap().parent().unwrap()).unwrap();
    }
    // Dossier absent : indisponible, jamais une page vide.
    let absent = std::env::temp_dir().join(format!("bridget-l149-absent-{}", uuid::Uuid::new_v4()));
    assert!(matches!(
        lineage_journal_page(&absent, task_id, 0, 50),
        Err(E::JournalUnavailable)
    ));
    // Dossier lisible par le groupe : refus.
    let ouvert = journal_tree("ouvert");
    std::fs::set_permissions(&ouvert, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(
        lineage_journal_page(&ouvert, task_id, 0, 50),
        Err(E::JournalUnavailable)
    ));
    std::fs::set_permissions(&ouvert, std::fs::Permissions::from_mode(0o700)).unwrap();
    // Fichier trop permissif : refus.
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o644)
            .open(journal_file(&ouvert))
            .unwrap();
        writeln!(file, "{}", json!({"v":1,"seq":1,"event":"note"})).unwrap();
        // L'umask du processus masque `mode(0o644)` : la permissivité est posée explicitement.
        file.set_permissions(std::fs::Permissions::from_mode(0o644))
            .unwrap();
    }
    assert!(matches!(
        lineage_journal_page(&ouvert, task_id, 0, 50),
        Err(E::JournalUnavailable)
    ));
    std::fs::remove_dir_all(ouvert.parent().unwrap().parent().unwrap()).unwrap();

    // Contenu nominal : trois événements, rattrapé.
    let directory = journal_tree("nominal");
    write_events(
        &directory,
        &[
            json!({"v":1,"seq":1,"event":"note","payload":{"text":"un"}}),
            json!({"v":1,"seq":2,"event":"note","payload":{"text":"deux"}}),
            json!({"v":1,"seq":3,"event":"note","payload":{"text":"trois"}}),
        ],
    );
    let page = lineage_journal_page(&directory, task_id, 0, 50).unwrap();
    assert_eq!(page["status"], "ok");
    assert_eq!(page["task_id"], task_id);
    assert_eq!(page["events"].as_array().unwrap().len(), 3);
    assert_eq!(page["next_seq"], 3);
    assert_eq!(page["caught_up"], true);
    assert!(page["gap"].is_null());
    // after au bout : page vide rattrapée.
    let page = lineage_journal_page(&directory, task_id, 3, 50).unwrap();
    assert_eq!(page["events"].as_array().unwrap().len(), 0);
    assert_eq!(page["caught_up"], true);
    // Limite : page partielle non rattrapée.
    let page = lineage_journal_page(&directory, task_id, 0, 2).unwrap();
    assert_eq!(page["events"].as_array().unwrap().len(), 2);
    assert_eq!(page["next_seq"], 2);
    assert_eq!(page["caught_up"], false);
    std::fs::remove_dir_all(directory.parent().unwrap().parent().unwrap()).unwrap();

    // Trou de séquence : gap explicite et reprise bornée.
    let directory_gap = journal_tree("gap");
    write_events(
        &directory_gap,
        &[
            json!({"v":1,"seq":1,"event":"note"}),
            json!({"v":1,"seq":4,"event":"note"}),
        ],
    );
    let page = lineage_journal_page(&directory_gap, task_id, 0, 50).unwrap();
    assert_eq!(page["caught_up"], false);
    assert_eq!(page["gap"]["reason"], "journal_gap");
    assert_eq!(page["gap"]["from_seq"], 2);
    assert_eq!(page["gap"]["to_seq"], 3);
    assert_eq!(page["next_seq"], 3);
    std::fs::remove_dir_all(directory_gap.parent().unwrap().parent().unwrap()).unwrap();

    // Entrée trop lourde en tête de fenêtre : gap entry_too_large.
    let directory_lourd = journal_tree("lourd");
    write_events(
        &directory_lourd,
        &[json!({"v":1,"seq":2,"event":"note","payload":{"text":"z".repeat(17 * 1024)}})],
    );
    let page = lineage_journal_page(&directory_lourd, task_id, 1, 50).unwrap();
    assert_eq!(page["events"].as_array().unwrap().len(), 0);
    assert_eq!(page["gap"]["reason"], "entry_too_large");
    assert_eq!(page["gap"]["from_seq"], 2);
    assert_eq!(page["gap"]["to_seq"], 2);
    std::fs::remove_dir_all(directory_lourd.parent().unwrap().parent().unwrap()).unwrap();

    // Ligne illisible : indisponible.
    let directory_sale = journal_tree("sale");
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(journal_file(&directory_sale))
            .unwrap();
        writeln!(file, "{{v:1}}").unwrap();
    }
    assert!(matches!(
        lineage_journal_page(&directory_sale, task_id, 0, 50),
        Err(E::JournalUnavailable)
    ));
    std::fs::remove_dir_all(directory_sale.parent().unwrap().parent().unwrap()).unwrap();

    // Plus de 256 fichiers : limite de ressources.
    let directory_volume = journal_tree("volume");
    for i in 0..257 {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory_volume.join(format!("{i:04}.jsonl")))
            .unwrap();
    }
    assert!(matches!(
        lineage_journal_page(&directory_volume, task_id, 0, 50),
        Err(E::ResourceLimit)
    ));
    std::fs::remove_dir_all(directory_volume.parent().unwrap().parent().unwrap()).unwrap();
}

// ---------------------------------------------------------------------------
// Marqueur bridgetTaskRef : fermeture stricte et exclusion en V2.
// ---------------------------------------------------------------------------

fn marker() -> Value {
    json!({
        "version":1,
        "taskId":"14900000-0000-4000-8000-000000000005",
        "rootThreadId":T3,
        "parentTaskId":null,
        "generation":"14900000-0000-4000-8000-000000000006",
        "seq":7,
        "status":"queued"
    })
}

fn marker_file(m: Value) -> Value {
    json!({"id":"t-natif","bridgetTaskRef":m})
}

#[test]
fn native149_marqueur_projection_ferme_et_exclut() {
    use crate::t3code_contract::is_native_projection;
    assert_eq!(is_native_projection(&marker_file(marker())).unwrap(), true);
    assert_eq!(is_native_projection(&json!({"id":"t"})).unwrap(), false);
    // La présence du préfixe seul ne suffit pas : l'objet est validé en entier.
    assert!(is_native_projection(&json!({"id":"t","bridgetTaskRef":"x"})).is_err());

    let mut cas: Vec<Value> = Vec::new();
    let mut m = marker();
    m["version"] = json!(2);
    cas.push(m);
    let mut m = marker();
    m.as_object_mut().unwrap().remove("parentTaskId");
    cas.push(m);
    let mut m = marker();
    m["extra"] = json!(1);
    cas.push(m);
    let mut m = marker();
    m["taskId"] = json!("pas-un-uuid");
    cas.push(m);
    let mut m = marker();
    m["generation"] = json!("");
    cas.push(m);
    let mut m = marker();
    m["parentTaskId"] = json!("parent-non-uuid");
    cas.push(m);
    let mut m = marker();
    m["rootThreadId"] = json!("");
    cas.push(m);
    let mut m = marker();
    m["rootThreadId"] = json!(format!("racine\n{T3}"));
    cas.push(m);
    let mut m = marker();
    m["seq"] = json!(bridget_transport::protocol::HUMAN_LINEAGE_MAX_SEQ + 1);
    cas.push(m);
    let mut m = marker();
    m["status"] = json!("archive-inconnue");
    cas.push(m);
    let mut m = marker();
    m.as_object_mut().unwrap().remove("seq");
    cas.push(m);
    for m in &cas {
        assert!(
            is_native_projection(&marker_file(m.clone())).is_err(),
            "marqueur accepté à tort : {m}"
        );
    }
}

#[test]
fn native149_snapshot_v2_exclut_le_filtre_avant_normalisation() {
    let mut coquille = json!({
        "schemaVersion":2,"snapshotSequence":17,
        "projects":[{"id":"p","workspaceRoot":"/fixture/project"}],
        "threads":[{"id":"t","projectId":"p","title":"test","providerInstanceId":"claude_glm",
            "modelSelection":{"instanceId":"claude_glm"},"runtimeMode":"approval-required","interactionMode":"default",
            "latestRunId":"r","latestRunRequestedAt":"2026-10-09T00:00:00.000Z","activeRunId":"r","status":"running",
            "updatedAt":"2026-10-09T00:00:01.000Z","archivedAt":null,"deletedAt":null,"settledOverride":null}]
    });
    // Un fil porteur du marqueur, sans providerInstanceId : exclu AVANT la
    // normalisation, sinon l'exigence providerInstanceId refuserait tout.
    coquille["threads"].as_array_mut().unwrap().push(
        json!({"id":"t-natif","projectId":"p","title":"Mission 149","bridgetTaskRef":marker()}),
    );
    let parse = crate::t3code_contract_v2::parse_snapshot(&coquille.to_string()).unwrap();
    let ids: Vec<&str> = parse.threads.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["t"]);
    // Un marqueur invalide refuse le snapshot entier, jamais en silence.
    coquille["threads"][1]["bridgetTaskRef"] = json!({"version":1});
    assert!(crate::t3code_contract_v2::parse_snapshot(&coquille.to_string()).is_err());
    // Un second fil ordinaire reste intact : comportement 148 inchangé.
    coquille["threads"][1] = json!({"id":"t-2","projectId":"p","title":"autre","providerInstanceId":"claude_glm",
        "modelSelection":{"instanceId":"claude_glm"},"runtimeMode":"approval-required","interactionMode":"default",
        "latestRunId":null,"activeRunId":null,"status":"idle",
        "updatedAt":"2026-10-09T00:00:02.000Z","archivedAt":null,"deletedAt":null,"settledOverride":null});
    let parse = crate::t3code_contract_v2::parse_snapshot(&coquille.to_string()).unwrap();
    assert_eq!(parse.threads.len(), 2);
}

// ---------------------------------------------------------------------------
// Session 150 : la purge d'entretien force un Resync, l'ouverture ne purge rien.
// ---------------------------------------------------------------------------

/// Tâche annulée, nettoyée et finie en 1970 : purgeable dès le premier passage.
fn tache_ancienne_150(task_id: &str, owner: &str) -> crate::delegation::Task {
    let mut task = lineage_task(task_id, owner, "cancelled");
    task.cleanup_done = true;
    task.completed_at = Some(1_000);
    task
}

#[test]
fn native150_purge_entretien_signale_resync() {
    let (state, root, _peer) = fixture("l150-resync", "human_lineage_watch_v1");
    let task_id = "15000000-0000-4000-8000-00000000000a";
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &tache_ancienne_150(task_id, &owner_agent()),
                "req-150a",
                b"c",
            )
            .unwrap();
    }
    let (client, mut reader, handle) = open_watch_socket(&state, &root);
    assert_eq!(next_event(&mut reader)["status"], "ready");
    let avant = state
        .lock()
        .unwrap()
        .delegation_store
        .projection_meta()
        .unwrap();
    {
        let mut st = state.lock().unwrap();
        super::super::maintain_stores(&mut st, super::super::unix_now_secs());
        assert!(
            st.delegation_store.get(task_id).unwrap().is_none(),
            "arbre purgé"
        );
    }
    let resync = next_event(&mut reader);
    assert_eq!(resync["status"], "resync", "{resync}");
    assert_ne!(resync["generation"], json!(avant.generation));
    // Le lecteur refait une lecture complète : plus de tâche fantôme.
    let page = state
        .lock()
        .unwrap()
        .delegation_store
        .lineage_page(&owner_agent(), 10, None, |_| false)
        .unwrap();
    assert!(page["tasks"].as_array().unwrap().is_empty());
    drop(reader);
    drop(client);
    drop(handle);
}

#[test]
fn native150_purge_ouverture_du_daemon_ne_purge_rien() {
    let (st, config) = crate::daemon::presence_tests::state_with_registered_agent("l150-ouverture");
    st.delegation_store
        .insert(
            &tache_ancienne_150("15000000-0000-4000-8000-00000000000b", "proprietaire-150"),
            "req-150b",
            b"c",
        )
        .unwrap();
    drop(st);
    let (tx, _rx) = mpsc::channel();
    let rouvert = DaemonState::new(&config, tx).unwrap();
    assert_eq!(
        rouvert.delegation_store.tasks().unwrap().len(),
        1,
        "aucune purge à l'ouverture"
    );
    drop(rouvert);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn native150_pause_annulation_humaine_disponible() {
    let (state, root, _peer) = fixture("l150-pause-cancel", "human_lineage_cancel_v1");
    let task_id = "15000000-0000-4000-8000-00000000000c";
    {
        let st = state.lock().unwrap();
        st.delegation_store
            .insert(
                &lineage_task(task_id, &owner_agent(), "queued"),
                "req-150c",
                b"c",
            )
            .unwrap();
        let conn = st.store.connection();
        let generation = crate::referent_control::read(conn).unwrap().generation;
        crate::referent_control::set(
            conn,
            crate::referent_control::ControlMutation {
                command_id: "pause-150-lineage",
                expected_generation: generation,
                paused: Some(true),
                auto_objectives_cap: None,
                reason: Some("test 150"),
                actor: "humain",
                now: 1_788_400_000,
                agent_posture: None,
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
        assert!(crate::referent_control::read(conn).unwrap().paused);
    }
    let cancel = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":task_id,"request_id":"50000000-0000-4000-8000-00000000000c"}),
    );
    assert_eq!(
        cancel,
        json!({"version":1,"task_id":task_id,"status":"cancelling"}),
        "{cancel}"
    );
    // Le reçu est durable : le rejeu rend le même reçu, toujours en pause.
    let rejoue = call(
        &state,
        &root,
        json!({"action":"cancel","task_id":task_id,"request_id":"50000000-0000-4000-8000-00000000000c"}),
    );
    assert_eq!(rejoue, cancel);
}

#[test]
fn native162_lineage_list_show_v2_gardent_capacite_v1_et_contact_inconnu() {
    use bridget_transport::protocol::RemotePlacement;
    let (state, root, _peer) = fixture("l162-view", "human_lineage_view_v1");
    let distant_id = "16200000-0000-4000-8000-000000000011";
    let local_id = "16200000-0000-4000-8000-000000000012";
    let mut distant = lineage_task(distant_id, &owner_agent(), "queued");
    distant.remote_placement = Some(RemotePlacement {
        machine_id: "cartae".into(),
        worker_id: "worker-cartae".into(),
        provisioning_revision: 1,
        project_id: "projet".into(),
        mapping_revision: 1,
        origin_host: "mac".into(),
        origin_cwd: "/source".into(),
        source_root_identity: "fixture".into(),
        relative_cwd: String::new(),
        execution_cwd: "/remote".into(),
        base_commit: None,
        definition_digest: "definition".into(),
        permission_digest: "permissions".into(),
    });
    {
        let st = state.lock().unwrap();
        assert_eq!(st.client_negotiations["human"].version, 1);
        assert_eq!(
            st.client_negotiations["human"].capabilities,
            vec![bridget_transport::protocol::ClientCapability::HumanLineageViewV1]
        );
        st.delegation_store
            .insert(&distant, "remote162", b"remote")
            .unwrap();
        st.delegation_store
            .insert(
                &lineage_task(local_id, &owner_agent(), "queued"),
                "local162",
                b"local",
            )
            .unwrap();
    }
    let call_v2 = |project: &str, action: Value| {
        let mut request = lineage_request(project, action);
        let WrapperToDaemon::HumanLineage { request: body } = &mut request else {
            unreachable!()
        };
        body.version = 2;
        let response = handle_wrapper_message("human", request, &state).unwrap();
        serde_json::to_value(response).unwrap()["result"].clone()
    };
    let v1 = call(&state, &root, json!({"action":"list","limit":10}));
    assert_eq!(v1["version"], 1);
    assert!(
        v1["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|t| t.get("placement").is_none())
    );
    let v2 = call_v2(&root, json!({"action":"list","limit":10}));
    assert_eq!(v2["version"], 2);
    assert_eq!(v2["status"], "ok");
    assert_eq!(v2["tasks"].as_array().unwrap().len(), 2);
    let expected = json!({"machine_id":"cartae","display_name":null,"worker_id":"worker-cartae",
        "last_contact_at":null,"contact_state":"unknown"});
    for id in [distant_id, local_id] {
        let entry = v2["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["task_id"] == id)
            .unwrap();
        let placement = if id == distant_id {
            expected.clone()
        } else {
            Value::Null
        };
        assert_eq!(entry.get("placement"), Some(&placement));
        let action = json!({"action":"show","task_id":id,"offset":0,"limit":100});
        let show1 = call(&state, &root, action.clone());
        assert!(show1["task"].get("placement").is_none());
        let mut show2 = call_v2(&root, action);
        assert_eq!(show2["version"], 2);
        assert_eq!(show2["task"].get("placement"), Some(&placement));
        show2["version"] = json!(1);
        show2["task"].as_object_mut().unwrap().remove("placement");
        assert_eq!(show2, show1);
    }
    let absent = call_v2(
        &root,
        json!({"action":"show","task_id":"16200000-0000-4000-8000-000000000099","offset":0,"limit":100}),
    );
    assert_eq!(absent["version"], 1);
    assert_eq!(absent["code"], "task_unavailable");
    let mismatch = call_v2("/autre-projet162", json!({"action":"list","limit":10}));
    assert_eq!(mismatch["version"], 1);
    assert_eq!(mismatch["code"], "project_mismatch");
    state
        .lock()
        .unwrap()
        .client_negotiations
        .get_mut("human")
        .unwrap()
        .capabilities
        .clear();
    let refused = call_v2(&root, json!({"action":"list","limit":10}));
    assert_eq!(refused["version"], 1);
    assert_eq!(refused["code"], "unsupported_version");
    {
        let mut st = state.lock().unwrap();
        st.client_negotiations
            .get_mut("human")
            .unwrap()
            .capabilities = vec![bridget_transport::protocol::ClientCapability::HumanLineageViewV1];
        st.connection_roles.remove("human");
    }
    for action in [
        json!({"action":"list","limit":10}),
        json!({"action":"show","task_id":distant_id,"offset":0,"limit":100}),
    ] {
        let refused = call_v2(&root, action);
        assert_eq!(refused["version"], 1);
        assert_eq!(refused["code"], "binding_unavailable");
    }
}

#[test]
fn native162_lineage_autres_actions_v2_refusees_avec_erreurs_v1() {
    let (state, root, _peer) = fixture("l162-refus", "human_lineage_view_v1");
    let task_id = "16200000-0000-4000-8000-000000000021";
    for action in [
        json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":10,"follow":false}),
        json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":10,"follow":true}),
        json!({"action":"watch"}),
        json!({"action":"cancel","task_id":task_id,"request_id":"16200000-0000-4000-8000-000000000022"}),
    ] {
        let mut request = lineage_request(&root, action.clone());
        let WrapperToDaemon::HumanLineage { request: body } = &mut request else {
            unreachable!()
        };
        body.version = 2;
        let response =
            serde_json::to_value(handle_wrapper_message("human", request, &state).unwrap())
                .unwrap();
        let result = if action["action"] == "watch" {
            &response["event"]
        } else {
            &response["result"]
        };
        assert_eq!(result["version"], 1, "{response}");
        assert_eq!(result["status"], "error", "{response}");
        assert_eq!(result["code"], "unsupported_version", "{response}");
    }
    assert!(
        state
            .lock()
            .unwrap()
            .delegation_store
            .tasks()
            .unwrap()
            .is_empty()
    );
}

/// T029/T032 : lecture froide authentifiée, sans présence du fournisseur ni
/// réplication locale. La socket permet de modifier les gardes pendant le RPC.
#[test]
fn native162_journal_distant_froid_gap_perte_et_gardes_sans_repli_local() {
    use crate::fleet::{SpawnOrder, SpawnSubmission};
    use bridget_transport::protocol::{
        ClientCapability, IdentityCredential, RemotePlacement, SpawnOwnership, WorkerHello,
        WorkerIdentity, WorkerJournalFragment, WorkerRequestOperation, WorkerResponse,
        WorkerResponseOperation,
    };
    use sha2::{Digest, Sha256};

    for case in ["cold", "gap", "lost", "placement", "permissions", "human"] {
        let (state, root, _t3_peer) = fixture(&format!("l162-j-{case}"), "human_lineage_view_v1");
        let task_id = uuid::Uuid::new_v4().to_string();
        let mut task = lineage_task(&task_id, &owner_agent(), "working");
        task.remote_prepared = true;
        task.remote_placement = Some(RemotePlacement {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
            project_id: "project".into(),
            mapping_revision: 1,
            origin_host: "mac".into(),
            origin_cwd: root.clone(),
            source_root_identity: "git:fixture162".into(),
            relative_cwd: ".".into(),
            execution_cwd: "/tmp".into(),
            base_commit: None,
            definition_digest: "a".repeat(64),
            permission_digest: "b".repeat(64),
        });
        let worker_peer =
            {
                let mut st = state.lock().unwrap();
                st.registry = AgentRegistry::from_json(&json!({"machines":{"cartae":{
                "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
                "credential_fingerprint":format!("{:x}", Sha256::digest(b"lineage162-secret")),
                "allowed_agent_types":["fixture"],"mappings":{}
            }}}).to_string(), "/tmp/lineage162-agents.json").unwrap();
                let now = unix_now_secs();
                let order = SpawnOrder {
                    posture: None,
                    agent_type: "fixture".into(),
                    project: None,
                    requested_name: Some(task.child.clone()),
                    cwd: PathBuf::from("/tmp"),
                    persistent: false,
                    command_id: uuid::Uuid::new_v4().to_string(),
                    issued_at: now,
                    deadline_at: now + 60,
                    ownership: Some(SpawnOwnership {
                        parent_instance_id: task.owner_instance.clone(),
                        parent_execution_id: None,
                        objective_id: None,
                        delegation_id: Some(task_id.clone()),
                        project: None,
                        role: "native_delegate".into(),
                        max_children: None,
                        max_depth: None,
                    }),
                    remote_placement: task.remote_placement.clone(),
                };
                let SpawnSubmission::Start(lease) = st.fleet.request_spawn(&order, now).unwrap()
                else {
                    panic!("réservation distante attendue, cas {case}");
                };
                task.child_instance = Some(lease.instance_id);
                st.delegation_store
                    .insert(&task, &format!("journal162-{case}"), b"remote-journal162")
                    .unwrap();
                assert!(
                    st.router.get_agent(&task.child).is_none(),
                    "aucun fournisseur local"
                );
                native_delegation::remote_command_identity(&st, &task).unwrap();
                let peer = spec094_live_test_connection(&mut st, "worker162-journal");
                st.connection_roles
                    .insert("worker162-journal".into(), ConnectionRole::Worker);
                st.peer_uids
                    .insert("worker162-journal".into(), unsafe { libc::geteuid() });
                peer
            };
        let hello = WorkerHello {
            version: 1,
            identity: WorkerIdentity {
                machine_id: "cartae".into(),
                worker_id: "worker-cartae".into(),
                provisioning_revision: 1,
            },
            credential: IdentityCredential::new("lineage162-secret".into()),
            build: "fixture162".into(),
            capability: ClientCapability::RemoteWorkerV1,
        };
        assert!(matches!(
            handle_worker_message(
                &state,
                "worker162-journal",
                WrapperToDaemon::WorkerHello { hello }
            ),
            Some(DaemonToWrapper::WorkerAccepted { .. })
        ));

        // Un vrai journal local lisible détecte un repli accidentel vers le maître.
        let directory = journal_directory(&state.lock().unwrap(), &task.child).unwrap();
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&directory)
            .unwrap();
        write_events(
            &directory,
            &[json!({"v":1,"seq":1,"event":"note",
            "payload":{"text":"LOCAL_SENTINEL_162"}})],
        );
        assert_eq!(
            crate::attach::lineage_journal_page(&directory, &task_id, 0, 2).unwrap()["events"][0]["payload"]
                ["text"],
            "LOCAL_SENTINEL_162"
        );
        let action =
            json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":2,"follow":false});
        worker_peer
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut reader = BufReader::new(worker_peer);
        let result = if case == "lost" {
            state.lock().unwrap().remote_workers.remove("cartae");
            let result = call(&state, &root, action);
            reader
                .get_ref()
                .set_read_timeout(Some(Duration::from_millis(50)))
                .unwrap();
            let error = reader.read_line(&mut String::new()).unwrap_err();
            assert!(matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ));
            result
        } else {
            let rpc_state = Arc::clone(&state);
            let rpc_root = root.clone();
            let rpc = thread::spawn(move || call(&rpc_state, &rpc_root, action));
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = serde_json::from_str(&line).unwrap()
            else {
                panic!("lecture froide worker attendue : {line}");
            };
            assert_eq!(
                request.request,
                WorkerRequestOperation::JournalPage {
                    after_seq: 0,
                    limit: 2
                }
            );
            assert_eq!(request.command.task_id, task_id);
            assert_eq!(
                request.command.instance_id,
                task.child_instance.clone().unwrap()
            );
            assert_eq!(request.worker.machine_id, "cartae");
            let WrapperToDaemon::HumanLineage {
                request: live_request,
            } = lineage_request(
                &root,
                json!({"action":"journal","task_id":task_id,"after_seq":0,"limit":2,"follow":true}),
            )
            else {
                unreachable!()
            };
            let live_context = authorize(&state, "human", &live_request).unwrap();
            {
                let mut st = state.lock().unwrap();
                let (notify, invalidated) = mpsc::sync_channel(1);
                let (shutdown, mut live_peer) = UnixStream::pair().unwrap();
                live_peer
                    .set_read_timeout(Some(Duration::from_millis(100)))
                    .unwrap();
                guard_journal_snapshot(&st, "human", &live_context, &task, Some(&request.command))
                    .unwrap();
                st.lineage_journals.insert(
                    "human".into(),
                    Journal {
                        context: live_context.clone(),
                        task: task.clone(),
                        command: Some(request.command.clone()),
                        shutdown: Some(shutdown),
                        notify,
                    },
                );
                if matches!(case, "placement" | "permissions") {
                    let mut current = st.delegation_store.get(&task_id).unwrap().unwrap();
                    let placement = current.remote_placement.as_mut().unwrap();
                    if case == "placement" {
                        placement.mapping_revision += 1;
                    } else {
                        placement.permission_digest = "c".repeat(64);
                    }
                    st.delegation_store.save(&current).unwrap();
                } else if case == "human" {
                    st.client_negotiations
                        .get_mut("human")
                        .unwrap()
                        .capabilities
                        .clear();
                }
                invalidate(&mut st);
                if matches!(case, "placement" | "permissions" | "human") {
                    assert!(
                        invalidated.try_recv().is_ok(),
                        "abonnement live invalidé : {case}"
                    );
                    let mut byte = [0u8; 1];
                    assert_eq!(
                        std::io::Read::read(&mut live_peer, &mut byte).unwrap(),
                        0,
                        "socket live fermée avant publication : {case}"
                    );
                    assert!(
                        guard_journal_snapshot(
                            &st,
                            "human",
                            &live_context,
                            &task,
                            Some(&request.command)
                        )
                        .is_err()
                    );
                } else {
                    assert!(matches!(
                        invalidated.try_recv(),
                        Err(mpsc::TryRecvError::Empty)
                    ));
                }
                st.lineage_journals.remove("human");
            }
            let response = WorkerResponse {
                version: request.version,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::JournalPage {
                    fragments: vec![WorkerJournalFragment {
                        seq: 1,
                        offset: 0,
                        final_fragment: true,
                        bytes: serde_json::to_vec(&json!({"v":1,"seq":1,"event":"note",
                            "payload":{"text":"REMOTE_COLD_162"}}))
                        .unwrap(),
                    }],
                    through_seq: Some(if case == "gap" { 3 } else { 1 }),
                    more: case == "gap",
                },
            };
            assert!(
                handle_worker_message(
                    &state,
                    "worker162-journal",
                    WrapperToDaemon::WorkerResponse { response }
                )
                .is_none()
            );
            rpc.join().unwrap()
        };
        assert!(
            !result.to_string().contains("LOCAL_SENTINEL_162"),
            "{case}: {result}"
        );
        if matches!(case, "cold" | "gap") {
            assert_eq!(result["status"], "ok", "{case}: {result}");
            assert_eq!(result["events"].as_array().unwrap().len(), 1);
            assert_eq!(result["events"][0]["payload"]["text"], "REMOTE_COLD_162");
            assert_eq!(result["next_seq"], if case == "gap" { 3 } else { 1 });
            assert_eq!(result["caught_up"], case == "cold");
            assert_eq!(
                result["gap"],
                if case == "gap" {
                    json!({"from_seq":2,"to_seq":3,"reason":"journal_gap"})
                } else {
                    Value::Null
                }
            );
        } else {
            assert_eq!(result["status"], "error", "{case}: {result}");
            assert_eq!(
                result["code"],
                if case == "human" {
                    "unsupported_version"
                } else {
                    "journal_unavailable"
                },
                "{case}: {result}"
            );
            assert!(
                result.get("events").is_none(),
                "réponse refusée sans contenu distant"
            );
        }
        assert_eq!(
            state
                .lock()
                .unwrap()
                .delegation_store
                .get(&task_id)
                .unwrap()
                .unwrap()
                .state,
            "working"
        );
        assert!(state.lock().unwrap().pending_worker_responses.is_empty());
        std::fs::remove_file(journal_file(&directory)).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
