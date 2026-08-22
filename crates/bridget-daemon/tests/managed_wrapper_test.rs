use bridget_daemon::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStatus, spawn_managed_bootstrap,
};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn test_root() -> PathBuf {
    PathBuf::from(format!(
        "/tmp/bg906-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

#[test]
fn wrapper_reel_distingue_bootstrap_register_et_startup_failed_correle() {
    let root = test_root();
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry = root.join(".config/bridget/agents.json");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(
        &registry,
        r#"{"agents":{"fixture-managee":{"command":"/adaptateur/t906-absent","protocol":"acp","permissions":"allow","queue_capacity":2,"notify_timeout_secs":1}}}"#,
    )
    .unwrap();
    let listener = UnixListener::bind(&socket).unwrap();
    let (register_tx, register_rx) = mpsc::channel();
    let (accept_tx, accept_rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let register = decode::<WrapperToDaemon>(line.trim_end()).unwrap();
        register_tx.send(register).unwrap();
        accept_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "fixture-managee-1".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        line.clear();
        let _ = reader.read_line(&mut line);
    });

    let identity = ManagedIdentity {
        instance_id: "instance-wrapper-reel".to_string(),
        command_id: "command-wrapper-reel".to_string(),
        generation: 3,
    };
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity: identity.clone(),
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "fixture-managee".to_string(),
            "fixture-managee-1".to_string(),
        ],
        cwd: root.clone(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
        ]),
    };
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    let ready = spawn_managed_bootstrap(&launch).unwrap().wait_ready().unwrap();
    assert_eq!(ready.ready().instance_id, identity.instance_id);
    let mut running = ready
        .persist_marker(&marker_store, "fixture-managee-1")
        .unwrap()
        .release()
        .unwrap();
    assert!(matches!(
        register_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
        WrapperToDaemon::Register {
            instance_id: Some(instance_id),
            ..
        } if instance_id == identity.instance_id
    ));

    running
        .status_reader()
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut line = String::new();
    let before_register = running.status_reader().read_line(&mut line).unwrap_err();
    assert!(matches!(
        before_register.kind(),
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
    ));

    accept_tx.send(()).unwrap();
    running
        .status_reader()
        .get_mut()
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    line.clear();
    running.status_reader().read_line(&mut line).unwrap();
    assert_eq!(
        serde_json::from_str::<ManagedStatus>(line.trim_end()).unwrap(),
        ManagedStatus::StartupFailed {
            kind: "command_missing".to_string(),
            reason: "/adaptateur/t906-absent".to_string(),
            instance_id: identity.instance_id,
            command_id: identity.command_id,
            generation: identity.generation,
        }
    );
    running.close_status();
    assert_eq!(running.child_mut().wait().unwrap().code(), Some(1));
    daemon.join().unwrap();
    fs::remove_dir_all(root).unwrap();
}
