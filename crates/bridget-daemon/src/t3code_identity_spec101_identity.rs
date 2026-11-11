use super::*;

#[test]
#[ignore = "requires a disposable T3 V2 server database"]
fn spec147_v2_real_database_schema_is_readable() {
    let base = std::env::var("BRIDGET_TEST_T3_WORKSPACE").unwrap();
    assert!(base.starts_with("/private/tmp/t3-bridget-v2."));
    let path = Path::new(&base).join("userdata/statev2.sqlite");
    assert!(path.is_file());
    assert!(
        read_sessions_v2(&path, &[binding("fixture-absent")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn spec147_v2_identity_reads_only_current_primary_strong_session() {
    let temp = std::env::temp_dir().join(format!("bi147-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&temp).unwrap();
    let path = temp.join("statev2.sqlite");
    let live = [binding("t")];
    assert!(read_sessions_v2(&path, &live).is_err());
    assert!(!path.exists());
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TABLE orchestration_v2_projection_threads(thread_id TEXT PRIMARY KEY,active_provider_thread_id TEXT,provider_instance_id TEXT,deleted_at TEXT);
            CREATE TABLE orchestration_v2_projection_provider_threads(provider_thread_id TEXT PRIMARY KEY,thread_id TEXT,owner_node_id TEXT,provider_session_id TEXT,driver TEXT,provider_instance_id TEXT,status TEXT,payload_json TEXT);
            CREATE TABLE orchestration_v2_projection_provider_sessions(provider_session_id TEXT PRIMARY KEY,driver TEXT,provider_instance_id TEXT,status TEXT);
            INSERT INTO orchestration_v2_projection_threads VALUES('t','p','cx-pro',NULL);
            INSERT INTO orchestration_v2_projection_provider_sessions VALUES('s','codex','cx-pro','running');
            INSERT INTO orchestration_v2_projection_provider_threads VALUES('p','t',NULL,'s','codex','cx-pro','active','{}');").unwrap();
    let payload = serde_json::json!({"nativeThreadRef":{"driver":"codex","strength":"strong","nativeId":"native"}});
    db.execute(
        "UPDATE orchestration_v2_projection_provider_threads SET payload_json=?",
        [payload.to_string()],
    )
    .unwrap();
    assert_eq!(
        read_sessions_v2(&path, &live).unwrap(),
        vec![("t".into(), "codex".into(), "native".into())]
    );
    for (sql, restore) in [
        (
            "UPDATE orchestration_v2_projection_provider_threads SET owner_node_id='child'",
            "UPDATE orchestration_v2_projection_provider_threads SET owner_node_id=NULL",
        ),
        (
            "UPDATE orchestration_v2_projection_provider_threads SET thread_id='other'",
            "UPDATE orchestration_v2_projection_provider_threads SET thread_id='t'",
        ),
        (
            "UPDATE orchestration_v2_projection_provider_threads SET provider_instance_id='old'",
            "UPDATE orchestration_v2_projection_provider_threads SET provider_instance_id='cx-pro'",
        ),
        (
            "UPDATE orchestration_v2_projection_provider_sessions SET driver='claude'",
            "UPDATE orchestration_v2_projection_provider_sessions SET driver='codex'",
        ),
        (
            "UPDATE orchestration_v2_projection_provider_sessions SET status='stopped'",
            "UPDATE orchestration_v2_projection_provider_sessions SET status='running'",
        ),
        (
            "UPDATE orchestration_v2_projection_threads SET active_provider_thread_id='old'",
            "UPDATE orchestration_v2_projection_threads SET active_provider_thread_id='p'",
        ),
        (
            "UPDATE orchestration_v2_projection_threads SET deleted_at='now'",
            "UPDATE orchestration_v2_projection_threads SET deleted_at=NULL",
        ),
    ] {
        db.execute_batch(sql).unwrap();
        assert!(read_sessions_v2(&path, &live).unwrap().is_empty(), "{sql}");
        db.execute_batch(restore).unwrap();
    }
    for native in [
        serde_json::json!({"driver":"codex","strength":"weak","nativeId":"native"}),
        serde_json::json!({"driver":"claude","strength":"strong","nativeId":"native"}),
        serde_json::json!({"driver":"codex","strength":"strong","nativeId":null}),
    ] {
        db.execute(
            "UPDATE orchestration_v2_projection_provider_threads SET payload_json=?",
            [serde_json::json!({"nativeThreadRef":native}).to_string()],
        )
        .unwrap();
        assert!(read_sessions_v2(&path, &live).unwrap().is_empty());
    }
    // Une ancienne base V1 reste indépendante du chemin explicitement choisi.
    let old = rusqlite::Connection::open(temp.join("state.sqlite")).unwrap();
    old.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT); INSERT INTO provider_session_runtime VALUES('t','claude','{\"resume\":\"old-native\"}');").unwrap();
    assert_eq!(
        read_sessions(&temp.join("state.sqlite"), &live).unwrap()[0].2,
        "old-native"
    );
    assert!(read_sessions_v2(&path, &live).unwrap().is_empty());
    assert!(
        read_sessions_v2(&temp.join("absent"), &[])
            .unwrap()
            .is_empty()
    );
    drop(db);
    drop(old);
    std::fs::remove_dir_all(&temp).unwrap();
}

fn binding(thread: &str) -> (String, String, String) {
    (
        thread.into(),
        crate::t3code::stable_uuid(thread),
        format!("instance-{thread}"),
    )
}

fn process(pid: u32, sessions: &[&str]) -> ProcessEvidence {
    ProcessEvidence {
        pid,
        birth: 10,
        lineage: vec![(pid, 10), (1, 20)],
        provider: "codex".into(),
        sessions: sessions.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn fournisseur_interne_n_est_pas_un_agent_t3_principal() {
    let mut principal = process(10, &["native-principal"]);
    principal.lineage = vec![(10, 10), (1, 20)];
    let mut interne = process(20, &["native-interne"]);
    interne.lineage = vec![(20, 30), (30, 40), (10, 10), (1, 20)];
    let mut voisin = process(40, &["native-voisin"]);
    voisin.lineage = vec![(40, 50), (1, 20)];

    let principaux = primary_provider_processes(vec![interne, voisin, principal]);
    let pids: HashSet<u32> = principaux.into_iter().map(|process| process.pid).collect();

    assert_eq!(pids, HashSet::from([10, 40]));
}

#[test]
fn spec133_fournisseur_interne_recoit_le_parent_unique_et_une_reference_opaque() {
    let mut principal = process(10, &["native-principal"]);
    principal.lineage = vec![(10, 10), (1, 20)];
    let mut interne = process(20, &["native-interne"]);
    interne.lineage = vec![(20, 30), (30, 40), (10, 10), (1, 20)];
    let processes = vec![principal, interne];
    let principals = primary_provider_processes(processes.clone());
    let live = [binding("parent")];
    let rows = [("parent".into(), "codex".into(), "native-principal".into())];
    let selected = select_bindings(&rows, &principals, &live);

    let delegated = select_delegated_bindings(&processes, &selected);

    assert_eq!(delegated.len(), 1);
    assert_eq!(delegated[0].0, 20);
    assert_eq!(delegated[0].1, live[0]);
    assert_eq!(delegated[0].2.provider, "codex");
    assert_eq!(delegated[0].2.child_ref.len(), 16);
    assert!(!delegated[0].2.child_ref.contains("native-interne"));
}

#[test]
fn deux_fils_meme_cwd_sous_agent_plus_recent_ne_change_pas_identite() {
    // Le cwd et le mtime n'entrent volontairement pas dans ce contrat.
    let rows = vec![
        ("a".into(), "codex".into(), "native-a".into()),
        ("b".into(), "codex".into(), "native-b".into()),
    ];
    let processes = [
        process(2, &["child", "native-a"]),
        process(3, &["native-b"]),
    ];
    let live = [binding("a"), binding("b")];
    let selected = select_bindings(&rows, &processes, &live);
    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].1.0, "a");
    assert_eq!(selected[1].1.0, "b");
}

#[test]
fn plusieurs_correspondances_ou_processus_refuses() {
    let rows = vec![
        ("a".into(), "codex".into(), "one".into()),
        ("b".into(), "codex".into(), "two".into()),
    ];
    assert!(
        select_bindings(
            &rows,
            &[process(2, &["one", "two"])],
            &[binding("a"), binding("b")]
        )
        .is_empty()
    );
    assert!(
        select_bindings(
            &rows,
            &[process(2, &["one"]), process(3, &["one"])],
            &[binding("a")]
        )
        .is_empty()
    );
    assert!(
        select_bindings(
            &rows,
            &[process(2, &["one", "two"]), process(3, &["one"])],
            &[binding("a"), binding("b")]
        )
        .is_empty()
    );
}

#[test]
fn pid_recycle_ou_parent_recycle_refuse() {
    let evidence = process(2, &["native"]);
    assert!(lineage_valid(&evidence, |pid| Some(if pid == 1 {
        20
    } else {
        10
    })));
    assert!(!lineage_valid(&evidence, |_| Some(99)));
    assert!(!lineage_valid(&evidence, |pid| Some(if pid == 1 {
        21
    } else {
        10
    })));
}

#[test]
fn sqlite_absent_ne_cree_pas_de_base_et_curseur_nul_ignore() {
    let temp = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&temp).unwrap();
    let path = temp.join("state.sqlite");
    let live = [binding("a"), binding("b")];
    assert!(read_sessions(&path, &live).is_err());
    assert!(!path.exists());
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT); INSERT INTO provider_session_runtime VALUES ('a','codex',NULL), ('b','codex','{\"threadId\":\"native-b\"}');").unwrap();
    assert_eq!(
        read_sessions(&path, &live).unwrap(),
        vec![("b".into(), "codex".into(), "native-b".into())]
    );
    db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<600) INSERT INTO provider_session_runtime SELECT 'archive-'||x,'codex','{\"threadId\":\"old\"}' FROM n;").unwrap();
    assert_eq!(
        read_sessions(&path, &live).unwrap().len(),
        1,
        "l'historique inactif ne consomme pas la borne"
    );
    db.execute_batch("INSERT INTO provider_session_runtime VALUES ('claude-resume','claude','{\"threadId\":\"not-provider\",\"resume\":\"native-resume\"}'),('claude-session','claude','{\"sessionId\":\"native-session\"}'),('claude-wrong','claude','{\"threadId\":\"not-provider\"}');").unwrap();
    assert_eq!(
        read_sessions(
            &path,
            &[
                binding("claude-resume"),
                binding("claude-session"),
                binding("claude-wrong")
            ]
        )
        .unwrap(),
        vec![
            (
                "claude-resume".into(),
                "claude".into(),
                "native-resume".into()
            ),
            (
                "claude-session".into(),
                "claude".into(),
                "native-session".into()
            )
        ]
    );
    drop(db);
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn sqlite_alias_reel_claude_agent_utilise_la_normalisation_du_pont() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let path = root.join("state.sqlite");
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT); INSERT INTO provider_session_runtime VALUES ('claude-real','claudeAgent','{\"resume\":\"session-native\"}');").unwrap();
    let rows = read_sessions(&path, &[binding("claude-real")]).unwrap();
    assert_eq!(
        rows,
        vec![(
            "claude-real".into(),
            "claude".into(),
            "session-native".into()
        )]
    );
    drop(db);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec147_native_t3_refresh_publishes_and_revokes_real_os_processes() {
    use std::os::unix::process::CommandExt;
    use std::process::{Child, ChildStdin, Command, Stdio};
    use std::time::Instant;
    // Même SQLite privée que spec114 et l'alias claudeAgent ci-dessus.
    // Le tuple live et le credential restent des fixtures de protocole.
    // T040 MCP/daemon vérifie séparément les credentials authentiques.
    let root = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        .join(format!("t3147-{}", uuid::Uuid::new_v4().simple()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let base = root.join("t3");
    fs::create_dir_all(base.join("userdata")).unwrap();
    let db = rusqlite::Connection::open(base.join("userdata/state.sqlite")).unwrap();
    db.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT);").unwrap();
    let session = format!("native147-{}", uuid::Uuid::new_v4().simple());
    db.execute(
        "INSERT INTO provider_session_runtime VALUES (?1,'claudeAgent',?2)",
        rusqlite::params!["fil147", serde_json::json!({"resume":session}).to_string()],
    )
    .unwrap();
    let rows_before = db
        .query_row(
            "SELECT thread_id,provider_name,resume_cursor_json FROM provider_session_runtime",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .unwrap();
    let changes_before = db.total_changes();
    let socket = root.join("bridget.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let live = binding("fil147");
    crate::mcp_identity::mock_private_identity(&socket, &live.1, &live.2);

    struct Provider {
        child: Child,
        input: Option<ChildStdin>,
        birth: u64,
        session: String,
    }
    impl Drop for Provider {
        fn drop(&mut self) {
            drop(self.input.take());
            let deadline = Instant::now() + Duration::from_secs(2);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            let pid = self.child.id();
            if process_birth(pid).ok() != Some(self.birth)
                || process_parent(pid).ok() != Some(std::process::id())
            {
                return;
            }
            let Ok(observed) = Command::new("/bin/ps")
                .args(["-p", &pid.to_string(), "-o", "command="])
                .output()
            else {
                return;
            };
            let command = String::from_utf8_lossy(&observed.stdout);
            if command.contains(&self.session) && !command.to_ascii_lowercase().contains("firefox")
            {
                // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGTERM);
                }
                std::thread::sleep(Duration::from_secs(3));
                let _ = self.child.try_wait();
            }
        }
    }
    let spawn = || {
        // argv[0] synthétique reconnu par le collecteur OS. /bin/sh ne
        // lance que read/exit intégrés ; jamais Claude ni un modèle.
        let mut child = Command::new("/bin/sh")
            .arg0("claude")
            .args([
                "-c",
                "read fixture_gate || exit 0",
                "--session-id",
                &session,
            ])
            .env_clear()
            .env("HOME", &root)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        Provider {
            birth: process_birth(child.id()).unwrap(),
            input: child.stdin.take(),
            child,
            session: session.clone(),
        }
    };
    let mut first = spawn();
    let pid = first.child.id();
    // Sous charge, la lecture juste après spawn peut précéder l'exec :
    // attendre (5 s au plus) que le processus porte ses arguments réels.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut arguments = process_arguments(pid);
    while arguments
        .as_ref()
        .is_none_or(|args| args.first().map(String::as_str) != Some("claude"))
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(20));
        arguments = process_arguments(pid);
    }
    let arguments = arguments.unwrap_or_else(|| {
            let child_status = first.child.try_wait();
            let birth = process_birth(pid);
            let parent = process_parent(pid);
            let still_owned = matches!(&child_status, Ok(None))
                && birth.as_ref().ok() == Some(&first.birth)
                && parent.as_ref().ok() == Some(&std::process::id());
            let snapshot = if still_owned {
                Command::new("/bin/ps")
                    .args(["-p", &pid.to_string(), "-o", "pid=,ppid=,stat=,command="])
                    .output()
                    .map(|output| {
                        (
                            output.status.code(),
                            String::from_utf8_lossy(&output.stdout).into_owned(),
                            String::from_utf8_lossy(&output.stderr).into_owned(),
                        )
                    })
            } else {
                Err(std::io::Error::other("enfant terminé ou identité non confirmée"))
            };
            panic!(
                "arguments fixture absents: pid={pid} expected_birth={} child_status={child_status:?} birth={birth:?} parent={parent:?} os_snapshot={snapshot:?}",
                first.birth
            );
        });
    assert_eq!(arguments[0], "claude");
    let runtime = ServerRuntime {
        pid: std::process::id(),
        port: 3773,
    };
    let mut bindings = IdentityBindings::new(&root);
    let marker_path = root.join("agent-pids").join(pid.to_string());
    assert!(!marker_path.exists());
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let count = bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), false)
            .unwrap();
        if count == 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "fournisseur synthétique absent de refresh OS"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let marker = read_marker(&marker_path).unwrap();
    assert_eq!(marker.pid, pid);
    assert_eq!(marker.birth, first.birth);
    assert_eq!(marker.instance_id, live.2);
    assert_eq!(
        crate::mcp_identity::read_name(&marker.name_file).as_deref(),
        Some(live.1.as_str())
    );
    assert_eq!(process_parent(pid).unwrap(), runtime.pid);
    struct NativeTree;
    impl crate::mcp_identity::ProcessTree for NativeTree {
        fn birth(&self, pid: u32) -> Option<u64> {
            process_birth(pid).ok()
        }
        fn parent(&self, pid: u32) -> Option<u32> {
            process_parent(pid).ok()
        }
    }
    let resolved = crate::mcp_identity::resolve_identity_with(
        None,
        &root.join("agent-pids"),
        None,
        pid,
        &NativeTree,
    )
    .unwrap();
    assert_eq!(resolved.name, live.1);
    assert_eq!(resolved.instance_id, live.2);
    // Le même processus réel est attesté depuis V2, sans ouvrir la base V1
    // en écriture. Une session V2 arrêtée révoque le marqueur malgré V1.
    let v2db = rusqlite::Connection::open(base.join("userdata/statev2.sqlite")).unwrap();
    v2db.execute_batch("CREATE TABLE orchestration_v2_projection_threads(thread_id TEXT PRIMARY KEY,active_provider_thread_id TEXT,provider_instance_id TEXT,deleted_at TEXT);
            CREATE TABLE orchestration_v2_projection_provider_threads(provider_thread_id TEXT PRIMARY KEY,thread_id TEXT,owner_node_id TEXT,provider_session_id TEXT,driver TEXT,provider_instance_id TEXT,status TEXT,payload_json TEXT);
            CREATE TABLE orchestration_v2_projection_provider_sessions(provider_session_id TEXT PRIMARY KEY,driver TEXT,provider_instance_id TEXT,status TEXT);
            INSERT INTO orchestration_v2_projection_threads VALUES('fil147','p','claude_glm',NULL);
            INSERT INTO orchestration_v2_projection_provider_sessions VALUES('s','claude','claude_glm','running');
            INSERT INTO orchestration_v2_projection_provider_threads VALUES('p','fil147',NULL,'s','claude','claude_glm','active','{}');").unwrap();
    v2db.execute("UPDATE orchestration_v2_projection_provider_threads SET payload_json=?",[serde_json::json!({"nativeThreadRef":{"driver":"claude","strength":"strong","nativeId":session}}).to_string()]).unwrap();
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), true)
            .unwrap(),
        1
    );
    assert_eq!(read_marker(&marker_path).unwrap().instance_id, live.2);
    v2db.execute_batch("UPDATE orchestration_v2_projection_provider_sessions SET status='stopped'")
        .unwrap();
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), true)
            .unwrap(),
        0
    );
    assert!(!marker_path.exists());
    v2db.execute_batch("UPDATE orchestration_v2_projection_provider_sessions SET status='running'")
        .unwrap();
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), true)
            .unwrap(),
        1
    );
    let second = spawn();
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), true)
            .unwrap(),
        0
    );
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), false)
            .unwrap(),
        0,
        "deux fournisseurs d'une même session ne deviennent pas uniques"
    );
    assert!(!marker_path.exists());
    assert!(
        !root
            .join("agent-pids")
            .join(second.child.id().to_string())
            .exists()
    );
    assert!(
        crate::mcp_identity::resolve_identity_with(
            None,
            &root.join("agent-pids"),
            None,
            pid,
            &NativeTree
        )
        .is_err()
    );
    let second_pid = second.child.id();
    drop(second);
    assert!(process_birth(second_pid).is_err());
    assert_eq!(
        bindings
            .refresh(&base, &runtime, std::slice::from_ref(&live), false)
            .unwrap(),
        1
    );
    assert_eq!(bindings.refresh(&base, &runtime, &[], false).unwrap(), 0);
    assert!(!marker_path.exists());
    assert!(!marker.name_file.exists());
    assert!(
        crate::mcp_identity::auxiliary_registration(&live.1, &live.2, &socket).is_ok(),
        "la révocation du marqueur ne retire pas le credential de fixture"
    );
    let rows_after = db
        .query_row(
            "SELECT thread_id,provider_name,resume_cursor_json FROM provider_session_runtime",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(rows_after, rows_before);
    assert_eq!(db.total_changes(), changes_before);
    bindings.clear();
    drop(first);
    assert!(process_birth(pid).is_err());
    eprintln!(
        "SPEC147 native-T3 refresh namespace={} PID/birth/lineage/marqueur/résolution/ambiguïté/retrait vérifiés ; live/credential synthétiques",
        root.display()
    );
    drop(db);
    drop(_listener);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inventaire_partiellement_illisible_ne_rend_pas_autre_processus_unique() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let readable = root.join("a.jsonl");
    let missing = root.join("unreadable.jsonl");
    fs::write(
        &readable,
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"A\"}}\n",
    )
    .unwrap();
    let candidates = vec![process(2, &[]), process(3, &[])];
    let files = HashMap::from([(2, vec![readable.clone()]), (3, vec![readable, missing])]);
    let complete = complete_session_inventory(vec![process(2, &[])], &files).unwrap();
    assert_eq!(complete[0].sessions, vec!["A"]);
    // P1=A, P2=A+inaccessible : aucun jeu de candidats partiel ne doit
    // atteindre select_bindings et rendre artificiellement P1 unique.
    assert!(complete_session_inventory(candidates, &files).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn claude_exige_flag_exact_et_unique() {
    assert_eq!(
        claude_session(&["claude".into(), "--resume".into(), "session-a".into()]),
        Some("session-a".into())
    );
    assert!(
        claude_session(&[
            "claude".into(),
            "--prompt".into(),
            "--resume session-a".into()
        ])
        .is_none()
    );
    assert!(
        claude_session(&[
            "claude".into(),
            "--resume".into(),
            "a".into(),
            "--session-id".into(),
            "b".into()
        ])
        .is_none()
    );
}

#[test]
fn rollout_lit_metadata_exacte_et_refuse_fragment() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let path = root.join("rollout avec espaces.jsonl");
    fs::write(&path, "{\"type\":\"session_meta\",\"payload\":{\"id\":\"parent\"}}\n{\"type\":\"other\",\"payload\":{\"id\":\"child\"}}\n").unwrap();
    assert_eq!(rollout_session(&path).as_deref(), Some("parent"));
    fs::write(
        &path,
        "{\"type\":\"session_meta\",\"payload\":{\"id\":\"parent\"}}",
    )
    .unwrap();
    assert_eq!(rollout_session(&path), None);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn marqueur_prive_retire_sur_clear_sans_ecraser_autre_proprietaire() {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    let evidence = process(42, &["native"]);
    owner.publish(&evidence, &binding("a")).unwrap();
    let path = root.join("agent-pids/42");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let existing = read_marker(&path).unwrap();
    assert_eq!(
        crate::mcp_identity::write_marker_if_absent(&root.join("agent-pids"), &existing)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::AlreadyExists
    );
    struct Tree;
    impl crate::mcp_identity::ProcessTree for Tree {
        fn birth(&self, pid: u32) -> Option<u64> {
            (pid == 42).then_some(10)
        }
        fn parent(&self, _: u32) -> Option<u32> {
            None
        }
    }
    let resolved =
        crate::mcp_identity::resolve_identity_with(None, &root.join("agent-pids"), None, 42, &Tree)
            .unwrap();
    assert_eq!(resolved.name, binding("a").1);
    assert_eq!(resolved.instance_id, "instance-a");
    let mut other = IdentityBindings::new(&root);
    other.publish(&evidence, &binding("b")).unwrap();
    assert!(other.owned.is_empty());
    assert_eq!(read_marker(&path).unwrap().instance_id, "instance-a");
    owner.clear();
    assert!(!path.exists());
    other.publish(&evidence, &binding("b")).unwrap();
    owner.clear();
    assert_eq!(read_marker(&path).unwrap().instance_id, "instance-b");
    other.clear();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec133_clear_retire_seulement_le_marqueur_enfant_possede() {
    use std::os::unix::fs::PermissionsExt;

    let root = std::env::temp_dir().join(format!("bi133-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    let parent = process(42, &["native-parent"]);
    let parent_binding = binding("parent");
    owner.publish(&parent, &parent_binding).unwrap();
    let mut child = process(43, &["native-child"]);
    child.lineage = vec![(43, 10), (42, 10), (1, 20)];
    let origin = bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
    };
    owner
        .publish_delegated(&child, 42, &parent_binding, &origin)
        .unwrap();
    let owned_path = root.join("delegated-pids/43");
    assert_eq!(
        fs::metadata(&owned_path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let third_name = root.join("agent-names/third-child-parent");
    bridget_transport::fsutil::write_private_file_atomic(
        &third_name,
        binding("third").1.as_bytes(),
    )
    .unwrap();
    let third = DelegatedPidMarker {
        pid: 44,
        birth: 10,
        parent_instance_id: "instance-third".into(),
        parent_name_file: third_name,
        provider: "codex".into(),
        child_ref: "fedcba9876543210".into(),
        owner: format!(
            "{}-{}-{}",
            std::process::id(),
            process_birth(std::process::id()).unwrap(),
            uuid::Uuid::new_v4()
        ),
    };
    crate::mcp_identity::write_delegated_marker_if_absent(&root.join("delegated-pids"), &third)
        .unwrap();

    owner.clear();
    assert!(!owned_path.exists());
    assert!(root.join("delegated-pids/44").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inventaire_ouvert_transitoire_est_retente_avant_retrait() {
    let root = std::env::temp_dir().join(format!("bi-hotfix-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    owner
        .publish(&process(42, &["native"]), &binding("a"))
        .unwrap();
    let marker = root.join("agent-pids/42");
    let mut attempts = 0;

    let result = owner.refresh_with_retry(Duration::ZERO, |state| {
        attempts += 1;
        if attempts == 1 {
            Err("inventaire des sessions ouvertes incomplet".into())
        } else {
            Ok(state.owned.len())
        }
    });

    assert_eq!(result, Ok(1));
    assert_eq!(attempts, 2);
    assert!(marker.exists(), "le marqueur reste publié pendant le retry");
    owner.clear();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn double_echec_inventaire_ouvert_revoque_le_marqueur() {
    let root = std::env::temp_dir().join(format!("bi-hotfix-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    owner
        .publish(&process(42, &["native"]), &binding("a"))
        .unwrap();
    let marker = root.join("agent-pids/42");
    let mut attempts = 0;

    let result = owner.refresh_with_retry(Duration::ZERO, |_| {
        attempts += 1;
        Err("inventaire des sessions ouvertes incomplet".into())
    });

    assert_eq!(
        result.unwrap_err(),
        "inventaire des sessions ouvertes incomplet"
    );
    assert_eq!(attempts, 2);
    assert!(
        !marker.exists(),
        "deux échecs consécutifs ferment l'identité"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn remplacement_possede_prepare_la_nouvelle_identite_avant_la_bascule() {
    let root = std::env::temp_dir().join(format!("bi117-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    let evidence = process(42, &["native"]);
    owner.publish(&evidence, &binding("a")).unwrap();
    let marker_path = root.join("agent-pids/42");
    let previous = read_marker(&marker_path).unwrap();

    owner.publish(&evidence, &binding("b")).unwrap();

    let current = read_marker(&marker_path).unwrap();
    assert_ne!(
        current.name_file, previous.name_file,
        "le nouveau nom doit être préparé dans un fichier distinct avant la bascule atomique"
    );
    assert!(!previous.name_file.exists());
    assert_eq!(
        crate::mcp_identity::read_name(&current.name_file),
        Some(binding("b").1)
    );
    assert_eq!(current.instance_id, "instance-b");
    owner.clear();
    fs::remove_dir_all(root).unwrap();
}

/// Session 114 : Cursor ne met pas son identifiant de session en ligne de
/// commande ; il le révèle par le dossier `acp-sessions` qu'il tient ouvert.
#[test]
fn spec114_session_cursor_lue_dans_le_dossier_acp() {
    assert_eq!(
        cursor_session(Path::new(
            "/Users/u/.cursor/acp-sessions/df338dbf-96b3-4dbf-90f3-ee27fbf1af83/store.db"
        )),
        Some("df338dbf-96b3-4dbf-90f3-ee27fbf1af83".into())
    );
    // Les fichiers frères du même dossier désignent la même session.
    assert_eq!(
        cursor_session(Path::new("/x/acp-sessions/s-1/store.db-wal")),
        Some("s-1".into())
    );
    for refuse in [
        "/Users/u/.cursor/acp-sessions",
        "/Users/u/.codex/sessions/rollout.jsonl",
        "/x/acp-sessions/../evade/store.db",
    ] {
        assert_eq!(cursor_session(Path::new(refuse)), None, "{refuse}");
    }
}

/// Chaque fournisseur ne lit que SES fichiers de session : un rollout
/// Codex ouvert par Cursor ne devient pas une session Cursor, et un
/// fichier étranger n'invalide pas l'inventaire.
#[test]
fn spec114_inventaire_trie_les_fichiers_par_fournisseur() {
    let paths = vec![
        PathBuf::from("/x/acp-sessions/s-1/store.db"),
        PathBuf::from("/x/rollout.jsonl"),
        PathBuf::from("/x/node_modules/tree-sitter.node"),
    ];
    assert_eq!(
        session_files_of("cursor", &paths),
        vec![&PathBuf::from("/x/acp-sessions/s-1/store.db")]
    );
    assert_eq!(
        session_files_of("codex", &paths),
        vec![&PathBuf::from("/x/rollout.jsonl")]
    );
    assert!(session_files_of("claude", &paths).is_empty());
}

/// Session 114 : une seconde application T3 sur le même dossier réécrit
/// `server-runtime.json`. Notre serveur, lui, n'a pas changé : exiger
/// l'égalité du fichier révoquait les identités de TOUS les fils.
#[test]
fn spec114_fichier_d_etat_etranger_ne_revoque_pas_les_identites() {
    let root = std::env::temp_dir().join(format!("bi114-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let base = root.join("t3");
    fs::create_dir_all(base.join("userdata")).unwrap();
    let db = rusqlite::Connection::open(base.join("userdata/state.sqlite")).unwrap();
    db.execute_batch(
            "CREATE TABLE provider_session_runtime (thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT);
             INSERT INTO provider_session_runtime VALUES ('fil-a','cursor','{\"sessionId\":\"s-a\"}');",
        )
        .unwrap();
    drop(db);
    fs::write(
            base.join("userdata/server-runtime.json"),
            r#"{"version":1,"pid":4294967000,"host":"127.0.0.1","port":3774,"origin":"http://127.0.0.1:3774"}"#,
        )
        .unwrap();
    let ours = ServerRuntime {
        pid: std::process::id(),
        port: 3773,
    };
    let mut bindings = IdentityBindings::new(&root);
    let result = bindings.refresh(&base, &ours, &[binding("fil-a")], false);
    assert!(
        result.is_ok(),
        "un fichier d'état étranger ne révoque plus rien : {result:?}"
    );
    bindings.clear();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn disparition_et_ambiguite_revoquent_uniquement_marqueurs_possedes() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    let mut owner = IdentityBindings::new(&root);
    owner
        .publish(&process(42, &["native"]), &binding("a"))
        .unwrap();
    let runtime = ServerRuntime {
        pid: 42,
        port: 1234,
    };
    assert!(
        owner
            .refresh(&root.join("absent"), &runtime, &[binding("a")], false)
            .is_err()
    );
    assert!(!root.join("agent-pids/42").exists());
    assert!(owner.owned.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_apres_crash_retire_orphelin_mais_preserve_pont_vivant_et_tiers() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    let mut crashed = IdentityBindings::new(&root);
    // PID réutilisé : le propriétaire déclaré avait une autre naissance.
    // Aucun sous-processus ni signal réel n'est nécessaire pour cette preuve.
    crashed.owner = format!(
        "{}-{}-{}",
        std::process::id(),
        crashed.owner_birth - 1,
        uuid::Uuid::new_v4()
    );
    crashed
        .publish(&process(42, &["a"]), &binding("a"))
        .unwrap();
    let old_name = crashed.owned[&42].name_file.clone();
    let mut live = IdentityBindings::new(&root);
    live.publish(&process(43, &["b"]), &binding("b")).unwrap();
    let third_name = root.join("agent-names/other-wrapper");
    bridget_transport::fsutil::write_private_file_atomic(
        &third_name,
        binding("third").1.as_bytes(),
    )
    .unwrap();
    crate::mcp_identity::write_marker(&root.join("agent-pids"), 44, 10, "third", &third_name)
        .unwrap();
    let mut restarted = IdentityBindings::new(&root);
    restarted
        .refresh(&root, &ServerRuntime { pid: 1, port: 1 }, &[], false)
        .unwrap();
    assert!(!root.join("agent-pids/42").exists());
    assert!(!old_name.exists());
    assert!(root.join("agent-pids/43").exists());
    assert!(root.join("agent-pids/44").exists());
    restarted
        .publish(&process(42, &["a"]), &binding("a"))
        .unwrap();
    crashed.clear();
    assert!(
        root.join("agent-pids/42").exists(),
        "l'ancien propriétaire ne retire pas le nouveau marqueur"
    );
    restarted.clear();
    live.clear();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inventaire_os_borne_conserve_tous_fichiers_ouverts_avec_espaces() {
    let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let first = root.join("parent avec espaces.jsonl");
    let second = root.join("child.jsonl");
    let file1 = fs::File::create(&first).unwrap();
    let file2 = fs::File::create(&second).unwrap();
    let pid = std::process::id();
    assert!(process_ids().unwrap().contains(&pid));
    assert!(!process_arguments(pid).unwrap().is_empty());
    let started = std::time::Instant::now();
    let inventory = crate::runtime::open_session_files(&[pid], is_session_file).unwrap();
    let elapsed = started.elapsed();
    // /var est un alias de /private/var sur macOS ; lsof publie le canonique.
    assert!(inventory[&pid].contains(&first.canonicalize().unwrap()));
    assert!(inventory[&pid].contains(&second.canonicalize().unwrap()));
    assert!(elapsed < Duration::from_secs(3));
    eprintln!("inventaire identité local : {} ms", elapsed.as_millis());
    drop((file1, file2));
    fs::remove_dir_all(root).unwrap();
}
