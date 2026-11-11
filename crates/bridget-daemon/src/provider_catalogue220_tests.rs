use super::*;
fn root() -> PathBuf {
    let root = std::env::temp_dir().join(format!("b220-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&root).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    root
}
fn fixture(
    root: &Path,
    scenario: &str,
) -> (
    crate::registry::AgentDefinition,
    crate::lifecycle::SourceEnvironment,
) {
    let program = root.join("codex");
    std::fs::write(&program,r#"#!/usr/bin/python3
import socket,sys,json,os,base64,hashlib,struct,signal,time
path=sys.argv[sys.argv.index('--listen')+1].removeprefix('unix://')
hold=os.environ.get('BRIDGET220_SCENARIO')=='hold_stop'
if hold:signal.signal(signal.SIGTERM,lambda *args:None)
server=socket.socket(socket.AF_UNIX);server.bind(path);server.listen(1)
conn,_=server.accept();reader=conn.makefile('rb');reads=0
headers={}
reader.readline()
while True:
    line=reader.readline()
    if line in [b'\r\n',b'\n',b'']:break
    name,value=line.decode().split(':',1);headers[name.lower()]=value.strip()
key=headers['sec-websocket-key']+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11'
accept=base64.b64encode(hashlib.sha1(key.encode()).digest())
conn.sendall(b'HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: '+accept+b'\r\n\r\n')
def send(value):
    data=json.dumps(value).encode();n=len(data)
    header=bytes([129,n]) if n<126 else bytes([129,126])+struct.pack('!H',n) if n<65536 else bytes([129,127])+struct.pack('!Q',n)
    conn.sendall(header+data)
while True:
    h=reader.read(2)
    if len(h)!=2:break
    size=h[1]&127
    if size==126:size=struct.unpack('!H',reader.read(2))[0]
    elif size==127:size=struct.unpack('!Q',reader.read(8))[0]
    mask=reader.read(4) if h[1]&128 else None
    data=reader.read(size)
    if mask:data=bytes(v^mask[i%4] for i,v in enumerate(data))
    if h[0]&15==8:break
    v=json.loads(data);method=v.get('method');
    with open(os.environ['BRIDGET220_LOG'],'a') as f:f.write(str(method)+'\n')
    if 'id' not in v:continue
    scenario=os.environ['BRIDGET220_SCENARIO']
    if method=='initialize':result={}
    elif method=='account/read':
        reads+=1
        email='fake-before@example.invalid' if reads==1 or scenario!='changed' else 'fake-after@example.invalid'
        result={'account':None if scenario=='absent' else {'type':'chatgpt','email':email,'planType':'pro'},'requiresOpenaiAuth':True}
        if scenario=='unidentified_change':result['account']={'type':'apiKey' if reads==1 else 'externalAuth'}
    elif method=='model/list':
        result={'data':[{'model':'model-'+str(n),'supportedReasoningEfforts':[{'reasoningEffort':'high'}],'defaultReasoningEffort':'high'} for n in range(257 if scenario=='large' else 1)],'nextCursor':None}
    else:raise Exception('forbidden method')
    send({'id':v['id'],'result':result})
while hold and not os.path.exists(os.path.join(os.environ['HOME'],'allow-exit')):time.sleep(.02)
"#).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
    let def=serde_json::from_value(json!({"command":program,"args":["app-server"],"protocol":"codex_app_server","pass_env":["BRIDGET220_LOG","BRIDGET220_SCENARIO"]})).unwrap();
    let mut source = crate::lifecycle::source_environment();
    source.insert("HOME".into(), root.as_os_str().to_owned());
    source.insert("BRIDGET220_LOG".into(), root.join("trace").into_os_string());
    source.insert("BRIDGET220_SCENARIO".into(), scenario.into());
    (def, source)
}
#[test]
fn spec221_c06_profile_signal_ignores_primary_auth_metadata() {
    let root = root();
    let (mut def, mut source) = fixture(&root, "present");
    let primary = root.join("primary");
    let isolated = root.join("isolated");
    std::fs::create_dir(&primary).unwrap();
    std::fs::create_dir(&isolated).unwrap();
    std::fs::write(primary.join("auth.json"), b"first").unwrap();
    std::fs::write(isolated.join("auth.json"), b"private").unwrap();
    std::fs::set_permissions(
        isolated.join("auth.json"),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    source.insert("CODEX_HOME".into(), primary.as_os_str().to_owned());
    def.pass_env.push("CODEX_HOME".into());
    def.codex_home = Some(isolated.to_string_lossy().into_owned());
    let first = config_stamp(&def, "definition", &source);
    std::fs::write(primary.join("auth.json"), b"primary-changed").unwrap();
    assert_eq!(first, config_stamp(&def, "definition", &source));
    // Le propriétaire modifie sa fixture, puis rétablit l'interdiction de
    // lecture. Seules ses métadonnées alimentent config_stamp.
    std::fs::set_permissions(
        isolated.join("auth.json"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    std::fs::write(isolated.join("auth.json"), b"isolated-changed").unwrap();
    std::fs::set_permissions(
        isolated.join("auth.json"),
        std::fs::Permissions::from_mode(0o000),
    )
    .unwrap();
    assert_ne!(first, config_stamp(&def, "definition", &source));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec220_probe_real_bootstrap_reads_account_models_and_stops_without_turn() {
    let root = root();
    let directory = root.join("catalogue");
    private_dir(&directory).unwrap();
    private_dir(&directory.join("markers")).unwrap();
    let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
    let (def, source) = fixture(&root, "present");
    let (models, account) = probe(
        &directory,
        &markers,
        &def,
        &source,
        &[7; 32],
        &AtomicBool::new(false),
    )
    .map_err(|e| e.0)
    .unwrap();
    assert_eq!(models[0].model, "model-0");
    assert!(account.is_some());
    let trace = std::fs::read_to_string(root.join("trace")).unwrap();
    assert_eq!(
        trace.lines().collect::<Vec<_>>(),
        [
            "initialize",
            "initialized",
            "account/read",
            "model/list",
            "model/list",
            "account/read"
        ]
    );
    assert!(recover_probe(&markers, false));
    assert!(!directory.join("c.sock").exists());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn spec220_probe_refuses_changed_account_and_oversized_list_and_reports_signout() {
    let root = root();
    let directory = root.join("catalogue");
    private_dir(&directory).unwrap();
    private_dir(&directory.join("markers")).unwrap();
    let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
    for (scenario, expected) in [
        ("changed", Some(CatalogueCause::AccountChanged)),
        ("large", Some(CatalogueCause::OutputLimit)),
        ("unidentified_change", Some(CatalogueCause::AccountChanged)),
        ("absent", None),
    ] {
        let (def, source) = fixture(&root, scenario);
        let value = probe(
            &directory,
            &markers,
            &def,
            &source,
            &[8; 32],
            &AtomicBool::new(false),
        );
        if let Some(expected) = expected {
            assert_eq!(value.err().map(|e| e.0), Some(expected));
        } else {
            assert_eq!(value.map_err(|e| e.0).unwrap(), (Vec::new(), None));
        }
        assert!(recover_probe(&markers, false));
    }
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn spec220_retained_probe_survives_refresh_and_reconnection_generation() {
    let root = root();
    let directory = root.join("catalogue");
    private_dir(&directory).unwrap();
    private_dir(&directory.join("markers")).unwrap();
    struct Unblock(PathBuf);
    impl Drop for Unblock {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.0, b"exit");
        }
    }
    let _unblock = Unblock(root.join("allow-exit"));
    let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
    let (def, source) = fixture(&root, "hold_stop");
    let result = probe(
        &directory,
        &markers,
        &def,
        &source,
        &[8; 32],
        &AtomicBool::new(false),
    );
    let (cause, child) = result.expect_err("retained probe expected");
    assert_eq!(cause, CatalogueCause::StopUnconfirmed);
    let mut child = child.expect("owned child retained");
    assert!(child.try_wait().unwrap().is_none());
    let service = CatalogueService {
        generation: Arc::new(AtomicU64::new(0)),
        token: Arc::new(Mutex::new(String::new())),
        enabled: Arc::new(AtomicBool::new(false)),
        stopping: Arc::new(AtomicBool::new(false)),
        latest: Arc::new(Mutex::new(BTreeMap::new())),
    };
    service.activate("first-epoch");
    let mut sequence = 0;
    service.publish(
        service.generation.load(Ordering::Acquire),
        &mut sequence,
        observation(
            "main",
            &"a".repeat(64),
            CatalogueState::Unavailable,
            Some(cause),
        ),
    );
    // Retrait puis réintégration : même processus vivant, même contrainte.
    service.retain_connections(&BTreeSet::new());
    assert_eq!(
        service.snapshots()[0].2.cause,
        Some(CatalogueCause::StopUnconfirmed)
    );
    service.retain_connections(&BTreeSet::from(["main"]));
    assert_eq!(service.snapshots().len(), 1);
    for token in ["refresh-token", "reconnected-epoch"] {
        service.activate(token);
        let snapshots = service.snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].0, token);
        assert_eq!(snapshots[0].2.cause, Some(CatalogueCause::StopUnconfirmed));
    }
    let trace = std::fs::read_to_string(root.join("trace")).unwrap();
    assert_eq!(trace.lines().filter(|m| *m == "initialize").count(), 1);
    std::fs::write(root.join("allow-exit"), b"exit").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !child.reap_terminated_group().unwrap() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(recover_probe(&markers, false));
    service.confirm_probe_stopped();
    assert!(service.snapshots().is_empty());
    service.shutdown();
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn spec221_l2a_pruning_keeps_stop_constraints_without_growing_slots() {
    let service = CatalogueService {
        generation: Arc::new(AtomicU64::new(0)),
        token: Arc::new(Mutex::new(String::new())),
        enabled: Arc::new(AtomicBool::new(false)),
        stopping: Arc::new(AtomicBool::new(false)),
        latest: Arc::new(Mutex::new(BTreeMap::new())),
    };
    service.activate("epoch221");
    let generation = service.generation.load(Ordering::Acquire);
    let mut sequence = 0;
    for n in 0..MAX_CATALOGUE_CONNECTIONS {
        service.publish(
            generation,
            &mut sequence,
            observation(
                &format!("held-{n}"),
                &"a".repeat(64),
                CatalogueState::Unavailable,
                Some(CatalogueCause::StopUnconfirmed),
            ),
        );
    }
    service.retain_connections(&BTreeSet::new());
    assert_eq!(service.snapshots().len(), MAX_CATALOGUE_CONNECTIONS);
    let before = sequence;
    service.publish(
        generation,
        &mut sequence,
        observation(
            "new",
            &"a".repeat(64),
            CatalogueState::Unavailable,
            Some(CatalogueCause::SourceUnavailable),
        ),
    );
    assert_eq!(sequence, before);
    assert_eq!(service.snapshots().len(), MAX_CATALOGUE_CONNECTIONS);
    // Ni échec, ni succès de métadonnées ne prouvent l'arrêt.
    for (state, cause) in [
        (
            CatalogueState::Unavailable,
            Some(CatalogueCause::SourceUnavailable),
        ),
        (CatalogueState::Observed, None),
    ] {
        service.publish(
            generation,
            &mut sequence,
            observation("held-0", &"a".repeat(64), state, cause),
        );
        assert_eq!(sequence, before);
        assert!(
            service
                .snapshots()
                .iter()
                .all(|v| v.2.cause == Some(CatalogueCause::StopUnconfirmed))
        );
    }
    // La preuve d'arrêt est explicite, exercée avec le vrai Child dans220.
    service.confirm_probe_stopped();
    assert!(service.snapshots().is_empty());
    service.publish(
        generation,
        &mut sequence,
        observation(
            "new",
            &"a".repeat(64),
            CatalogueState::Unavailable,
            Some(CatalogueCause::SourceUnavailable),
        ),
    );
    assert_eq!(service.snapshots().len(), 1);
    service.shutdown();
}

#[test]
fn spec221_c02_collector_reloads_changed_program_and_reconnection_without_turn() {
    let root = root();
    let (def, source) = fixture(&root, "present");
    let program = PathBuf::from(&def.command);
    let state = root.join("account-state");
    std::fs::write(&state, "first").unwrap();
    // Le faux programme ne lit aucune connexion réelle ni variable secrète.
    // Les chemins de ses deux seules données sont inscrits dans sa fixture.
    let trace = serde_json::to_string(&root.join("trace").to_string_lossy()).unwrap();
    let account = serde_json::to_string(&state.to_string_lossy()).unwrap();
    let script = std::fs::read_to_string(&program).unwrap()
            .replace("hold=os.environ.get('BRIDGET220_SCENARIO')=='hold_stop'", "hold=False")
            .replace("os.environ['BRIDGET220_LOG']", &trace)
            .replace("scenario=os.environ['BRIDGET220_SCENARIO']",
                &format!("scenario=open({account}).read().strip()"))
            .replace("email='fake-before@example.invalid' if reads==1 or scenario!='changed' else 'fake-after@example.invalid'",
                "email='fake-'+scenario+'@example.invalid'")
            .replace("'model-'+str(n)", "scenario+'-model-'+str(n)");
    std::fs::write(&program, &script).unwrap();
    let registry = root.join("agents.json");
    let raw = json!({
        "agents":{"catalogue-test":{"command":program,"args":["app-server"],
            "protocol":"codex_app_server","forbidden_env":[],"pass_env":[],
            "capabilities":{"execution_paths":["codex_app_server"],
                "models":{"model-0":{"efforts":["high"]}}}}},
        "routing_policy":{"version":2,"revision":1,"enabled":true,
            "connections":[{"connection_id":"main","machine_id":"local",
                "engine":"codex","agent_type":"catalogue-test","enabled":true}],
            "model_catalog":{"main":[{"model":"model-0","supported_efforts":["high"],
                "default_effort":"high","provenance":"operator_observation"}]},
            "machine_policies":{"local":{"enabled":true}}}
    });
    std::fs::write(&registry, raw.to_string()).unwrap();
    std::fs::set_permissions(&registry, std::fs::Permissions::from_mode(0o600)).unwrap();
    AgentRegistry::load_from_path_quiet(registry.clone()).unwrap();
    let service = CatalogueService::start(registry, root.clone(), "local".into(), source);
    let _guard = ServiceGuard(service.clone());
    service.activate("first-epoch");
    let deadline = Instant::now() + Duration::from_secs(30);
    let first = loop {
        if let Some(row) = service
            .snapshots()
            .into_iter()
            .find(|(_, _, v)| v.state == CatalogueState::Observed)
        {
            break row;
        }
        assert!(Instant::now() < deadline, "première observation absente");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(first.0, "first-epoch");
    assert_eq!(first.2.models[0].model, "first-model-0");
    let first_nonce = first.2.account_scope.clone().unwrap();

    // Un signal reconnu change : les métadonnées du programme configuré.
    // Il n'est pas nécessaire d'attendre le TTL de120s ni d'appeler un modèle.
    std::fs::write(&state, "second").unwrap();
    std::fs::write(&program, format!("{script}\n# metadata-change\n")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let second = loop {
        if let Some(row) = service.snapshots().into_iter().find(|(_, _, v)| {
            v.state == CatalogueState::Observed && v.models[0].model == "second-model-0"
        }) {
            break row;
        }
        assert!(
            Instant::now() < deadline,
            "signal de programme non renouvelé"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(second.1 > first.1);
    assert_ne!(second.2.account_scope.as_ref().unwrap(), &first_nonce);
    assert_eq!(second.2.models.len(), 1);
    assert!(second.2.observed_at >= first.2.observed_at);

    service.deactivate();
    assert!(service.snapshots().is_empty());
    service.activate("reconnected-epoch");
    let deadline = Instant::now() + Duration::from_secs(30);
    let reconnected = loop {
        if let Some(row) = service.snapshots().into_iter().find(|(token, _, v)| {
            token == "reconnected-epoch" && v.state == CatalogueState::Observed
        }) {
            break row;
        }
        assert!(
            Instant::now() < deadline,
            "génération reconnectée non renouvelée"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(reconnected.1 > second.1);
    assert_eq!(reconnected.2.account_scope, second.2.account_scope);
    assert_eq!(reconnected.2.models, second.2.models);
    service.shutdown();
    drop(_guard);
    let deadline = Instant::now() + Duration::from_secs(3);
    while Arc::strong_count(&service.generation) != 1 {
        assert!(Instant::now() < deadline, "collecteur non arrêté");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(recover_probe(
        &ManagedMarkerStore::at_directory(root.join("catalogue/markers")),
        false
    ));
    let trace = std::fs::read_to_string(root.join("trace")).unwrap();
    assert_eq!(trace.lines().filter(|m| *m == "initialize").count(), 3);
    assert!(trace.lines().all(|m| matches!(
        m,
        "initialize" | "initialized" | "account/read" | "model/list"
    )));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec221_c02_auth_metadata_tracks_the_configured_home_without_reading_secrets() {
    let root = root();
    let (mut def, mut source) = fixture(&root, "present");
    let default_home = root.join(".codex");
    let other_home = root.join("other-codex-home");
    std::fs::create_dir(&default_home).unwrap();
    std::fs::create_dir(&other_home).unwrap();
    let auth = default_home.join("auth.json");
    std::fs::write(&auth, "fixture-only").unwrap();
    std::fs::set_permissions(&auth, std::fs::Permissions::from_mode(0o000)).unwrap();
    let initial = config_stamp(&def, &"a".repeat(64), &source);
    let replacement = default_home.join("auth.new");
    std::fs::write(&replacement, "fixture-next").unwrap();
    std::fs::set_permissions(&replacement, std::fs::Permissions::from_mode(0o000)).unwrap();
    std::fs::rename(&replacement, &auth).unwrap();
    let changed = config_stamp(&def, &"a".repeat(64), &source);
    assert_ne!(
        initial, changed,
        "un remplacement atomique du login doit invalider"
    );
    def.pass_env.push("CODEX_HOME".into());
    source.insert("CODEX_HOME".into(), other_home.as_os_str().to_owned());
    let alternate = config_stamp(&def, &"a".repeat(64), &source);
    std::fs::remove_file(&auth).unwrap();
    assert_eq!(
        alternate,
        config_stamp(&def, &"a".repeat(64), &source),
        "le compte principal ne doit plus fournir le signal du profil explicite"
    );
    std::fs::write(other_home.join("auth.json"), "another-fixture-only").unwrap();
    assert_ne!(alternate, config_stamp(&def, &"a".repeat(64), &source));
    assert_eq!(source["HOME"], root.as_os_str());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec220_rpc_timeout_and_unknown_method_are_bounded() {
    let (a, _b) = UnixStream::pair().unwrap();
    let socket = tungstenite::WebSocket::from_raw_socket(
        MetadataSocket {
            stream: a,
            deadline: Instant::now() + Duration::from_millis(30),
            bytes: 0,
        },
        tungstenite::protocol::Role::Client,
        Some(ws_config()),
    );
    let mut rpc = Rpc {
        socket,
        id: 0,
        bytes: 0,
    };
    assert_eq!(
        rpc.call("thread/start", json!({})),
        Err(CatalogueCause::ProtocolUnavailable)
    );
    assert_eq!(
        rpc.call("account/read", json!({})),
        Err(CatalogueCause::TimedOut)
    );
}
#[test]
fn spec220_generation_fences_old_snapshots_and_private_paths_reject_links() {
    let service = CatalogueService {
        generation: Arc::new(AtomicU64::new(0)),
        token: Arc::new(Mutex::new(String::new())),
        enabled: Arc::new(AtomicBool::new(false)),
        stopping: Arc::new(AtomicBool::new(false)),
        latest: Arc::new(Mutex::new(BTreeMap::new())),
    };
    service.activate("first");
    let g = service.generation.load(Ordering::Acquire);
    let mut seq = 0;
    service.publish(
        g,
        &mut seq,
        observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
    );
    assert_eq!(service.snapshots().len(), 1);
    service.activate("second");
    service.publish(
        g,
        &mut seq,
        observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
    );
    assert!(service.snapshots().is_empty());
    let g = service.generation.load(Ordering::Acquire);
    service.publish(
        g,
        &mut seq,
        observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
    );
    assert_eq!(service.snapshots()[0].0, "second");
    service.deactivate();
    assert!(service.snapshots().is_empty());
    let root = root();
    let link = root.join("link");
    std::os::unix::fs::symlink(&root, &link).unwrap();
    assert!(private_dir(&link).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
