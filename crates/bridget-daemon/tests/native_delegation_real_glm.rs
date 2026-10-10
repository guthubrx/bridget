//! Recette opt-in : parent Codex synthétique, GLM réel, daemon/MCP/flotte réels.
//! Aucun modèle lancé par défaut. Aucun secret écrit ou journalisé.
//! Auth, settings et profils source restent intacts.
//! Auth existante par settings ou gclaude attesté. Sur opt-in privé seulement,
//! la variable ZAI_API_KEY existante est transmise uniquement en mémoire.
//! Les sessions utilisent un profil privé et --settings référence la source.
//! Référence CLI officielle : https://code.claude.com/docs/en/cli-reference
//!
//! Exécution explicite après validation identité :
//! BRIDGET_148_REAL_GLM=1 BRIDGET_148_REAL_GLM_REGISTRY=/chemin/agents.json
//! Auth par gclaude/trousseau : BRIDGET_148_REAL_GLM_AUTH_COMMAND_SHA256=<hash attesté>.
//! Pour la variable existante : BRIDGET_148_REAL_GLM_PASS_ZAI_API_KEY=1.
//! BRIDGET_148_REAL_GLM_MODEL=glm-5.3 cargo test -p bridget-daemon
//! --test native_delegation_real_glm -- --ignored --exact
//! native148_parent_codex_fixture_delegates_once_to_real_glm --nocapture
//! Fournisseur par défaut glm ; effort facultatif BRIDGET_148_REAL_GLM_EFFORT.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_core::BridgetMessage;
use bridget_daemon::registry::AgentRegistry;
use bridget_transport::protocol::{CommunicationProjectSource, NativeDelegationRequest};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PARENT: &str = "14800000-0000-4000-8000-000000000101";
const INSTANCE: &str = "native148-codex-fixture-real-glm";
const HOST: &str = "idempotency-isolated";

/// Route privée de la fixture, sans lecture ni écriture de secret hors mémoire.
fn explicit_zai_auth(
    opt_in: Option<&str>,
    agent_type: &str,
    auth_route: &str,
    forbidden: &[String],
    key: Option<std::ffi::OsString>,
) -> Result<Option<std::ffi::OsString>, &'static str> {
    match opt_in {
        None => return Ok(None),
        Some("1") => {}
        Some(_) => return Err("auth_opt_in_invalid"),
    }
    if agent_type != "glm" || auth_route != "existing_gclaude_keychain_command" {
        return Err("auth_command_unattested");
    }
    if forbidden.iter().any(|name| name == "ZAI_API_KEY") {
        return Err("auth_variable_forbidden");
    }
    let key = key.ok_or("auth_variable_absent")?;
    if key.is_empty() {
        return Err("auth_variable_empty");
    }
    Ok(Some(key))
}

struct OwnedChild(Child);
impl OwnedChild {
    fn wait(&mut self, budget: Duration) -> Result<bool, String> {
        let deadline = Instant::now() + budget;
        loop {
            if let Some(status) = self.0.try_wait().map_err(|_| "child_state_unavailable")? {
                return Ok(status.success());
            }
            if Instant::now() >= deadline {
                return Err("child_cleanup_timeout".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(&mut self) -> Result<(), String> {
        if self
            .0
            .try_wait()
            .map_err(|_| "child_state_unavailable")?
            .is_some()
        {
            return Ok(());
        }
        let pid = self.0.id();
        let observed = Command::new("/bin/ps")
            .args([
                "-ww",
                "-p",
                &pid.to_string(),
                "-o",
                "pid=,ppid=,stat=,command=",
            ])
            .output()
            .map_err(|_| "child_identity_unavailable")?;
        // Fermer stdin peut terminer MCP entre try_wait et ps. Recontrôler
        // avant toute décision, sans assimiler un zombie à un autre exécutable.
        if self
            .0
            .try_wait()
            .map_err(|_| "child_state_unavailable")?
            .is_some()
        {
            return Ok(());
        }
        if !observed.status.success() {
            return Err("child_identity_unavailable".into());
        }
        let row = std::str::from_utf8(&observed.stdout).map_err(|_| "child_identity_invalid")?;
        let mut remainder = row.trim();
        let mut columns = Vec::new();
        for _ in 0..3 {
            let (column, rest) = remainder
                .split_once(char::is_whitespace)
                .ok_or("child_identity_invalid")?;
            columns.push(column);
            remainder = rest.trim_start();
        }
        if columns[0].parse::<u32>().ok() != Some(pid)
            || columns[1].parse::<u32>().ok() != Some(std::process::id())
        {
            return Err("child_ownership_unproved".into());
        }
        if columns[2].contains('Z') {
            self.wait(Duration::from_secs(30))?;
            return Ok(());
        }
        let expected = Path::new(env!("CARGO_BIN_EXE_bridget"));
        let canonical = fs::canonicalize(expected).map_err(|_| "child_executable_unavailable")?;
        let same_executable = [expected, canonical.as_path()].iter().any(|path| {
            path.to_str()
                .and_then(|path| remainder.strip_prefix(path))
                .is_some_and(|tail| tail.is_empty() || tail.starts_with(char::is_whitespace))
        });
        if !same_executable || remainder.to_lowercase().contains("firefox") {
            return Err("child_executable_unproved".into());
        }
        // Ce Child n'a pas été récolté : son PID ne peut pas être réutilisé.
        let rc = unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        if rc != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err("child_signal_failed".into());
        }
        self.wait(Duration::from_secs(30))?;
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Err(reason) = self.stop() {
            // Aucun secret ni commande complète dans le diagnostic de Drop.
            eprintln!(
                "cleanup refusé ou incomplet : PID {} ; {reason}",
                self.0.id()
            );
        }
    }
}

struct Mcp {
    child: OwnedChild,
    input: Option<BufWriter<ChildStdin>>,
    output: BufReader<ChildStdout>,
    next: u64,
}
impl Mcp {
    fn start(root: &Path) -> Self {
        let marker = root.join("state/parent-name");
        support::private_write(&marker, PARENT).unwrap();
        let mut child = OwnedChild(
            support::isolated_command(root)
                .arg("mcp")
                .env("BRIDGET_AGENT_ID_FILE", marker)
                .env("BRIDGET_AGENT_INSTANCE_ID", INSTANCE)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let input = Some(BufWriter::new(child.0.stdin.take().unwrap()));
        let output = BufReader::new(child.0.stdout.take().unwrap());
        let mut process = Self {
            child,
            input,
            output,
            next: 1,
        };
        let init = process.request("initialize", json!({"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"native148-real-glm","version":"1"}}));
        assert!(init.get("result").is_some(), "MCP initialize refusé");
        writeln!(
            process.input.as_mut().unwrap(),
            "{}",
            json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .unwrap();
        process.input.as_mut().unwrap().flush().unwrap();
        process
    }
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        writeln!(
            self.input.as_mut().unwrap(),
            "{}",
            json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
        )
        .unwrap();
        self.input.as_mut().unwrap().flush().unwrap();
        let mut ready = libc::pollfd {
            fd: self.output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut ready, 1, 20_000) } > 0,
            "MCP sans réponse dans la borne"
        );
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], id);
        response
    }
    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let response = self.request("tools/call", json!({"name":name,"arguments":arguments}));
        assert!(
            response.get("error").is_none() && response["result"]["isError"] != true,
            "outil MCP refusé : {response}"
        );
        let content = response["result"]["content"][0]["text"]
            .as_str()
            .expect("résultat MCP texte");
        serde_json::from_str(content).expect("résultat natif JSON")
    }
}
impl Mcp {
    fn close(&mut self) -> Result<(), String> {
        drop(self.input.take());
        self.child.stop()
    }
}
impl Drop for Mcp {
    fn drop(&mut self) {
        // OwnedChild effectue ensuite son nettoyage sans panic.
        drop(self.input.take());
    }
}

struct Fixture {
    root: PathBuf,
    daemon: Option<OwnedChild>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.daemon.take());
    }
}

fn grant(root: &Path) {
    let (mut master, mut slave) = (-1, -1);
    assert_eq!(
        unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        0
    );
    let _master = unsafe { fs::File::from_raw_fd(master) };
    let slave = unsafe { fs::File::from_raw_fd(slave) };
    let mut child = OwnedChild(
        support::isolated_command(root)
            .args([
                "delegate-grant",
                PARENT,
                "--cwd",
                root.join("project").to_str().unwrap(),
                "--posture",
                "discovery",
            ])
            .stdin(slave.try_clone().unwrap())
            .stdout(slave)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    assert!(
        child
            .wait(Duration::from_secs(10))
            .expect("fin du grant humain"),
        "grant natif humain refusé"
    );
}

fn receive_until_native(parent: &mut support::Client, replies: &mut Vec<BridgetMessage>) -> Value {
    loop {
        match parent.receive() {
            DaemonToWrapper::NativeDelegationResult { result } => return result,
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                parent.send(WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                });
                replies.push(message);
            }
            DaemonToWrapper::Deliver(message)
            | DaemonToWrapper::DeliverExecution { message, .. } => replies.push(message),
            _ => {}
        }
    }
}

#[test]
#[ignore = "GLM réel : opt-in explicite après validation de l’identité"]
fn native148_parent_codex_fixture_delegates_once_to_real_glm() {
    assert_eq!(
        std::env::var("BRIDGET_148_REAL_GLM").as_deref(),
        Ok("1"),
        "opt-in BRIDGET_148_REAL_GLM=1 requis"
    );
    let source = PathBuf::from(
        std::env::var_os("BRIDGET_148_REAL_GLM_REGISTRY").expect("registre existant requis"),
    );
    assert!(source.is_absolute());
    let metadata = fs::metadata(&source).unwrap();
    assert_eq!(
        metadata.permissions().mode() & 0o077,
        0,
        "registre privé requis"
    );
    let content = fs::read_to_string(&source).unwrap();
    let registry = AgentRegistry::from_json(&content, &source).expect("registre existant valide");
    let agent_type =
        std::env::var("BRIDGET_148_REAL_GLM_AGENT_TYPE").unwrap_or_else(|_| "glm".into());
    let definition = registry.get(&agent_type).expect("profil GLM existant");
    assert_eq!(definition.protocol, "claude_stream_json");
    let model = std::env::var("BRIDGET_148_REAL_GLM_MODEL").expect("modèle exact explicite requis");
    let capabilities = definition
        .capabilities
        .models
        .get(&model)
        .expect("modèle absent du registre : aucune substitution");
    let effort = std::env::var("BRIDGET_148_REAL_GLM_EFFORT").ok();
    if let Some(effort) = &effort {
        assert!(capabilities.efforts.contains(effort));
    }
    let profile = PathBuf::from(
        definition
            .claude_config_dir
            .as_ref()
            .expect("profil GLM explicite requis"),
    );
    let settings = profile.join("settings.json");
    assert!(profile.is_absolute() && settings.is_file());
    let settings_bytes = fs::read(&settings).expect("lecture settings existant");
    let settings_hash = Sha256::digest(&settings_bytes);
    let settings_value: Value = serde_json::from_slice(&settings_bytes).expect("settings JSON");
    assert!(
        Path::new(&definition.command).is_absolute(),
        "commande réelle absolue requise"
    );
    let source_command = fs::canonicalize(&definition.command).expect("commande existante");
    let command_bytes = fs::read(&source_command).expect("lecture commande pour attestation");
    let command_hash = format!("{:x}", Sha256::digest(&command_bytes));
    let auth_route = if settings_value["env"].get("ANTHROPIC_API_KEY").is_some()
        || settings_value["env"].get("ANTHROPIC_AUTH_TOKEN").is_some()
        || settings_value.get("apiKeyHelper").is_some()
    {
        "existing_profile_settings"
    } else {
        // La voie autonome existante lit le trousseau dans gclaude. Son hash
        // est attesté avant lancement. L'opt-in privé ci-dessous peut fournir
        // la variable existante en mémoire, sans lecture du trousseau par le test
        // et sans écrire de secret ni modifier settings ou profil source.
        assert_eq!(
            source_command.file_name().and_then(|name| name.to_str()),
            Some("gclaude")
        );
        let expected_hash = std::env::var("BRIDGET_148_REAL_GLM_AUTH_COMMAND_SHA256")
            .expect("commande gclaude : hash source attesté requis");
        assert_eq!(
            command_hash, expected_hash,
            "commande auth différente de l'attestation"
        );
        let script = std::str::from_utf8(&command_bytes).expect("wrapper auth existant lisible");
        assert!(
            script.contains("security find-generic-password")
                && script.contains("exec claude \"$@\""),
            "voie trousseau existante non attestée"
        );
        "existing_gclaude_keychain_command"
    };
    let opt_in = std::env::var_os("BRIDGET_148_REAL_GLM_PASS_ZAI_API_KEY");
    let zai_api_key = explicit_zai_auth(
        opt_in
            .as_ref()
            .map(|value| value.to_str().unwrap_or("invalid")),
        &agent_type,
        auth_route,
        &definition.forbidden_env,
        std::env::var_os("ZAI_API_KEY"),
    )
    .expect("préconditions auth privée non satisfaites");
    let pass_zai = zai_api_key.is_some();
    let gclaude_route = auth_route == "existing_gclaude_keychain_command";
    let auth_route = if pass_zai {
        "existing_gclaude_in_memory_ZAI_API_KEY"
    } else {
        auth_route
    };
    let private_pass_env: Vec<&str> = if pass_zai {
        vec!["ZAI_API_KEY"]
    } else {
        vec![]
    };
    let (fixture_path, source_cli) = if gclaude_route {
        // Le wrapper existant exécute `claude` par son nom. Aucun PATH hérité.
        let directory = source_command.parent().expect("répertoire commande source");
        let cli = directory.join("claude");
        assert!(
            cli.is_file(),
            "CLI source Claude introuvable à côté de gclaude"
        );
        let canonical_cli = fs::canonicalize(cli).expect("CLI source canonique existant");
        let cli_hash = format!(
            "{:x}",
            Sha256::digest(fs::read(&canonical_cli).expect("attestation CLI source"))
        );
        (
            format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", directory.display()),
            Some((canonical_cli, cli_hash)),
        )
    } else {
        ("/usr/bin:/bin:/usr/sbin:/sbin".into(), None)
    };
    assert!(
        !definition
            .args
            .iter()
            .any(|arg| arg.to_lowercase().contains("token=")
                || arg.to_lowercase().contains("api-key")),
        "un secret ne peut pas figurer dans le registre de recette"
    );

    let root = fs::canonicalize("/tmp").unwrap().join(format!(
        "ng148-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..10]
    ));
    for relative in ["", "state", "provider", "tmp", "project", "glm-profile"] {
        support::private_dir(&root.join(relative)).unwrap();
    }
    assert!(
        Command::new("/usr/bin/git")
            .args(["init", "--quiet"])
            .arg(root.join("project"))
            .status()
            .unwrap()
            .success()
    );
    let nonce = format!("GLM_NATIVE_148_{}", uuid::Uuid::new_v4().simple());
    support::private_write(&root.join("project/marker.txt"), &nonce).unwrap();
    let evidence = root.join("effective-model.jsonl");
    support::private_write(&evidence, []).unwrap();
    let relay = root.join("relay.py");
    support::private_write(
        &relay,
        include_bytes!("fixtures/native_delegation_real_glm_relay.py"),
    )
    .unwrap();
    let mut args = vec![
        relay.to_string_lossy().into_owned(),
        evidence.to_string_lossy().into_owned(),
        definition.command.clone(),
    ];
    args.extend(definition.args.clone());
    args.extend([
        "--settings".into(),
        settings.to_string_lossy().into_owned(),
        "--setting-sources".into(),
        "".into(),
    ]);
    let private_registry = json!({"agents":{agent_type.clone():{
        "command":"/usr/bin/python3", "args":args, "protocol":"claude_stream_json", "permissions":"deny",
        "claude_config_dir":root.join("glm-profile"), "forbidden_env":definition.forbidden_env,
        "pass_env":private_pass_env, "mcp":{"interactive":"claude","acp_session":false}, "capabilities":definition.capabilities
    }}});
    support::private_write(
        &root.join("state/agents.json"),
        serde_json::to_vec(&private_registry).unwrap(),
    )
    .unwrap();
    let mut f = Fixture { root, daemon: None };
    let log = f.root.join("daemon.log");
    support::private_write(&log, []).unwrap();
    let mut daemon_command = support::isolated_command(&f.root);
    daemon_command.env("PATH", &fixture_path);
    if let Some(key) = zai_api_key.as_ref() {
        // Unique variable explicitement consentie. Jamais de valeur sérialisée.
        daemon_command.env("ZAI_API_KEY", key);
    }
    f.daemon = Some(OwnedChild(
        daemon_command
            .arg("daemon")
            .env("TMPDIR", f.root.join("tmp"))
            .stdout(Stdio::null())
            .stderr(fs::OpenOptions::new().append(true).open(log).unwrap())
            .spawn()
            .unwrap(),
    ));
    let deadline = Instant::now() + Duration::from_secs(10);
    while std::os::unix::net::UnixStream::connect(support::socket(&f.root)).is_err() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let mut parent = support::Client::connect(&support::socket(&f.root));
    parent.send(WrapperToDaemon::Register {
        identity_version: 2,
        agent_type: "codex".into(),
        agent_id: PARENT.into(),
        host: Some(HOST.into()),
        transport: Some("codex_app_server".into()),
        channel: None.into(),
        mode: Some(bridget_transport::protocol::PresenceMode::Acp),
        location: None,
        os: Some("fixture".into()),
        instance_id: Some(INSTANCE.into()),
        domain: None,
        journal_available: None,
        turn_in_progress: false,
    });
    let DaemonToWrapper::Registered {
        credential: Some(credential),
        ..
    } = parent.receive()
    else {
        panic!("preuve native requise")
    };
    support::save_fixture_credential(&support::socket(&f.root), PARENT, INSTANCE, credential);
    parent.send(WrapperToDaemon::CommunicationProjectFact {
        root: f.root.join("project").to_string_lossy().into_owned(),
        source: CommunicationProjectSource::Git,
        host: HOST.into(),
        worktree_root: None,
    });
    assert!(
        matches!(
            parent.receive(),
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ),
        "projet natif non attesté"
    );
    grant(&f.root);
    let mut mcp = Mcp::start(&f.root);
    let catalogue = mcp.tool("bridget_capabilities", json!({}));
    assert!(
        catalogue["providers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["agent_type"] == agent_type
                && entry["models"].get(&model).is_some()
                && entry["discovery"] == true)
    );
    // Seul appel bridget_delegate de la recette. Le nonce n'est pas dans la mission.
    let accepted = mcp.tool("bridget_delegate", json!({"request_id":"real-glm-one-delegate","agent_type":agent_type,"model":model,"effort":effort,
        "task":"Lis seulement marker.txt dans ton répertoire courant. Réponds exactement avec son contenu, sans commentaire. Ne change aucun fichier. La réponse finale sera retournée au parent par Bridget.",
        "cwd":f.root.join("project"),"posture":"discovery"}));
    let task_id = accepted["task_id"]
        .as_str()
        .expect("tâche native admise")
        .to_owned();
    let mission = accepted["message_id"].as_str().unwrap().to_owned();
    let child = accepted["child_agent_id"].as_str().unwrap().to_owned();
    let mut replies = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(180);
    let terminal = loop {
        parent.send(WrapperToDaemon::NativeDelegation {
            request: NativeDelegationRequest::Status {
                task_id: task_id.clone(),
            },
        });
        let status = receive_until_native(&mut parent, &mut replies);
        assert_ne!(
            status["status"], "failed",
            "mission réelle échouée : {status}"
        );
        if status["status"] == "result_available"
            && replies
                .iter()
                .any(|reply| reply.in_reply_to.as_deref() == Some(&mission))
        {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "mission réelle hors délai ; preuves privées : {}",
            f.root.display()
        );
        thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(terminal["result"].as_str().unwrap().trim(), nonce);
    let correlated: Vec<_> = replies
        .iter()
        .filter(|reply| reply.in_reply_to.as_deref() == Some(&mission))
        .collect();
    assert_eq!(correlated.len(), 1);
    assert_eq!(correlated[0].from, child);
    assert_eq!(correlated[0].to, PARENT);
    assert_eq!(correlated[0].body.trim(), nonce);
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let payload: String = db
        .query_row(
            "SELECT payload FROM native_delegations WHERE task_id=?1",
            [&task_id],
            |row| row.get(0),
        )
        .unwrap();
    let stored: Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(stored["definition"]["protocol"], "claude_stream_json");
    assert_eq!(stored["request"]["model"], model);
    assert_eq!(stored["request"]["effort"], json!(effort));
    assert_eq!(
        db.query_row("SELECT count(*) FROM native_delegations", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    let observed: Vec<Value> = fs::read_to_string(&evidence)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let effective: Vec<_> = observed
        .iter()
        .flat_map(|event| event["models"].as_array().unwrap())
        .filter_map(Value::as_str)
        .collect();
    assert!(
        !effective.is_empty(),
        "le fournisseur n'annonce pas son modèle : sélection prouvée, modèle effectif non prouvé"
    );
    assert!(
        effective.iter().all(|actual| *actual == model),
        "modèle fournisseur différent : {effective:?}"
    );
    assert_eq!(
        Sha256::digest(fs::read(&settings).unwrap()),
        settings_hash,
        "settings source modifié"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(fs::read(&source_command).unwrap())),
        command_hash,
        "commande source modifiée pendant la recette"
    );
    parent.send(WrapperToDaemon::Unregister);
    drop(db);
    mcp.close().expect("cleanup MCP explicite");
    drop(mcp);
    drop(parent);
    f.daemon
        .as_mut()
        .expect("daemon fixture")
        .stop()
        .expect("cleanup daemon explicite");
    if let Some((cli, hash)) = source_cli.as_ref() {
        assert_eq!(
            format!("{:x}", Sha256::digest(fs::read(cli).unwrap())),
            *hash,
            "CLI source modifié"
        );
    }
    support::private_write(&f.root.join("receipt.json"), serde_json::to_vec_pretty(&json!({"parent":"fixture native Codex, aucun modèle Codex exécuté","glm":"réel","task_id":task_id,"child_agent_id":child,"message_id":mission,"result_correlated":true,"requested_model":model,"provider_models":effective,"t3_used":false,"source_registry":source,"source_command":source_command,"source_command_sha256":command_hash,"source_cli":source_cli.map(|(path,sha256)|json!({"path":path,"sha256":sha256})),"cleanup_confirmed":true,"auth_route":auth_route,"source_settings":settings,"settings_unchanged":true,"command_unchanged":true})).unwrap()).unwrap();
    eprintln!(
        "Recette GLM réel + parent synthétique : {}",
        f.root.display()
    );
    drop(f);
}

#[test]
fn native148_real_fixture_auth_requires_explicit_unique_private_route() {
    use std::ffi::OsString;
    let fake = || Some(OsString::from("fixture-only-auth-value"));
    let route = "existing_gclaude_keychain_command";
    assert!(
        explicit_zai_auth(None, "glm", route, &[], fake())
            .unwrap()
            .is_none()
    );
    assert!(
        explicit_zai_auth(Some("1"), "glm", route, &[], fake())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        explicit_zai_auth(Some("0"), "glm", route, &[], fake()).unwrap_err(),
        "auth_opt_in_invalid"
    );
    assert_eq!(
        explicit_zai_auth(Some("1"), "other", route, &[], fake()).unwrap_err(),
        "auth_command_unattested"
    );
    assert_eq!(
        explicit_zai_auth(Some("1"), "glm", "unattested", &[], fake()).unwrap_err(),
        "auth_command_unattested"
    );
    assert_eq!(
        explicit_zai_auth(Some("1"), "glm", route, &["ZAI_API_KEY".into()], fake()).unwrap_err(),
        "auth_variable_forbidden"
    );
    assert_eq!(
        explicit_zai_auth(Some("1"), "glm", route, &[], None).unwrap_err(),
        "auth_variable_absent"
    );
    assert_eq!(
        explicit_zai_auth(Some("1"), "glm", route, &[], Some(OsString::new())).unwrap_err(),
        "auth_variable_empty"
    );
}

fn dry_fixture_root() -> PathBuf {
    let root = fs::canonicalize("/tmp").unwrap().join(format!(
        "nd148-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..10]
    ));
    for relative in ["", "state", "provider", "tmp"] {
        support::private_dir(&root.join(relative)).unwrap();
    }
    root
}

#[test]
fn native148_real_fixture_mcp_cleanup_after_eof() {
    let root = dry_fixture_root();
    let mut mcp = Mcp::start(&root);
    let pid = mcp.child.0.id();
    mcp.close().expect("MCP EOF : nettoyage attesté");
    assert!(mcp.child.0.try_wait().unwrap().is_some());
    drop(mcp);
    assert_eq!(unsafe { libc::kill(pid as i32, 0) }, -1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native148_real_fixture_mcp_cleanup_during_unwind() {
    let root = dry_fixture_root();
    let pid = std::cell::Cell::new(0);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mcp = Mcp::start(&root);
        pid.set(mcp.child.0.id());
        panic!("oracle sec : unwind attendu");
    }));
    assert!(result.is_err());
    assert_ne!(pid.get(), 0);
    assert_eq!(
        unsafe { libc::kill(pid.get() as i32, 0) },
        -1,
        "MCP récolté pendant unwind"
    );
    fs::remove_dir_all(root).unwrap();
}
