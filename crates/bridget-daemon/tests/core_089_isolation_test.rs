//! SC08908 : aucun accès implicite à l'instance historique, HOME fournisseur intact.
use bridget_daemon::environment::Namespace;
use bridget_daemon::lifecycle::{SourceEnvironment, build_environment};
use bridget_daemon::registry::AgentDefinition;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from("/tmp").join(format!(
            "b89-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        private_dir(&root);
        for child in ["provider", "state", "tmp", "xdg"] {
            private_dir(&root.join(child));
        }
        Self(root)
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
        command
            .env_clear()
            .env("HOME", self.path("provider"))
            .env("BRIDGET_HOME", self.path("state"))
            .env("TMPDIR", self.path("tmp"))
            .env("XDG_CACHE_HOME", self.path("xdg"))
            .env("XDG_CONFIG_HOME", self.path("xdg"))
            .env("XDG_DATA_HOME", self.path("xdg"))
            .env("XDG_STATE_HOME", self.path("xdg"))
            .env("HOSTNAME", "isolated-core-089")
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn private_dir(path: &Path) {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .unwrap();
}
fn private_file(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
}
struct Process(Child);
impl Process {
    fn start(command: &mut Command) -> Self {
        Self(command.spawn().unwrap())
    }
    fn finish(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(20);
        while self.0.try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline, "processus de recette hors délai");
            std::thread::sleep(Duration::from_millis(5)); // watchdog, jamais oracle métier
        }
        let stdout = std::io::read_to_string(self.0.stdout.take().unwrap()).unwrap();
        let stderr = std::io::read_to_string(self.0.stderr.take().unwrap()).unwrap();
        Output {
            status: self.0.wait().unwrap(),
            stdout: stdout.into_bytes(),
            stderr: stderr.into_bytes(),
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            let observed = Command::new("/bin/ps")
                .args(["-p", &self.0.id().to_string(), "-o", "command="])
                .output()
                .unwrap();
            let command = String::from_utf8_lossy(&observed.stdout);
            assert!(
                (command.contains("bridget") || command.contains("core_089_isolation_test"))
                    && !command.to_ascii_lowercase().contains("firefox"),
                "PID inattendu : {command}"
            );
            assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.0.try_wait().unwrap().is_none() {
                assert!(Instant::now() < deadline, "daemon isolé non arrêté");
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

#[test]
fn namespace_explicite_et_defaut_ne_rejoignent_jamais_historique() {
    let f = Fixture::new();
    let resolved = Namespace::resolve(None, None, Some(f.path("provider"))).unwrap();
    assert_eq!(resolved.root, f.path("provider/.cache/bridget-core"));
    assert!(!resolved.root.exists(), "résoudre ne crée rien");
    assert!(Namespace::resolve(None, None, None).is_err());
    for old in [
        "provider/.cache/bridget",
        "provider/.config/bridget",
        "provider/.local/state/bridget",
    ] {
        assert!(Namespace::resolve(Some(f.path(old)), None, None).is_err());
        assert!(!f.path(old).exists());
    }
    assert!(Namespace::resolve(Some(PathBuf::from("/tmp/bridget")), None, None).is_err());
    assert!(Namespace::resolve(Some(f.path("state")), Some(f.path("other.sock")), None).is_err());
}

#[test]
fn api_wrapper_refuse_un_namespace_de_session_divergent_avant_lancement() {
    let fixture = Fixture::new();
    if std::env::var_os("CORE089_SESSION_PROBE").is_some() {
        let registry = bridget_daemon::registry::AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/false","protocol":"acp"}}}"#,
            fixture.path("registry.json"),
        )
        .unwrap();
        let error = bridget_daemon::wrapper::launch_acp_with(
            "fixture",
            &[],
            None,
            &registry,
            &fixture.path("state/bridget.sock"),
            &fixture.path("provider"),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("namespace de session différent"),
            "{error}"
        );
        assert!(!fixture.path("state/agent-names").exists());
        assert!(!fixture.path("state/sessions").exists());
        return;
    }
    let environment = fixture.command();
    let mut probe = Command::new(std::env::current_exe().unwrap());
    probe
        .env_clear()
        .envs(
            environment
                .get_envs()
                .filter_map(|(name, value)| value.map(|value| (name, value))),
        )
        .env("CORE089_SESSION_PROBE", "1")
        .args([
            "--exact",
            "api_wrapper_refuse_un_namespace_de_session_divergent_avant_lancement",
            "--nocapture",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let result = Process::start(&mut probe).finish();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn symlink_droits_et_fichier_detourne_sont_refuses_avant_creation() {
    let f = Fixture::new();
    symlink(f.path("state"), f.path("link")).unwrap();
    assert!(Namespace::resolve(Some(f.path("link")), None, None).is_err());
    fs::set_permissions(f.path("state"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Namespace::resolve(Some(f.path("state")), None, None).is_err());
    fs::set_permissions(f.path("state"), fs::Permissions::from_mode(0o700)).unwrap();
    private_file(&f.path("victim"), b"intact");
    symlink(f.path("victim"), f.path("state/bridget.db")).unwrap();
    let output = Process::start(f.command().arg("who")).finish();
    assert!(!output.status.success());
    assert_eq!(fs::read(f.path("victim")).unwrap(), b"intact");
    assert!(!f.path("state/tmp").exists());
}

#[test]
fn bootstrap_env_clear_conserve_namespace_et_abonnement_sans_secret_implicite() {
    let f = Fixture::new();
    let definition: AgentDefinition =
        serde_json::from_value(serde_json::json!({"command":"/bin/true"})).unwrap();
    let mut source = SourceEnvironment::new();
    source.insert("HOME".into(), f.path("provider").into_os_string());
    source.insert("BRIDGET_HOME".into(), f.path("state").into_os_string());
    source.insert("API_KEY_NON_DECLAREE".into(), "ne-pas-transmettre".into());
    let env = build_environment(&definition, &source).unwrap();
    assert_eq!(env["HOME"], source["HOME"]);
    assert_eq!(env["BRIDGET_HOME"], source["BRIDGET_HOME"]);
    assert_eq!(
        env["BRIDGET_SOCKET"],
        f.path("state/bridget.sock").as_os_str()
    );
    assert!(!env.contains_key("API_KEY_NON_DECLAREE"));
    source.insert(
        "BRIDGET_HOME".into(),
        f.path("provider/.cache/bridget").into_os_string(),
    );
    assert!(build_environment(&definition, &source).is_err());
}

#[test]
fn vrai_client_refuse_racine_historique_sans_contacter_la_socket() {
    let f = Fixture::new();
    let old = f.path("provider/.cache/bridget");
    private_dir(&old);
    let listener = UnixListener::bind(old.join("bridget.sock")).unwrap();
    listener.set_nonblocking(true).unwrap();
    let output = Process::start(f.command().env("BRIDGET_HOME", &old).arg("who")).finish();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("namespace historique interdit"));
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn migration_explicite_ne_peut_modifier_les_fichiers_historiques() {
    let f = Fixture::new();
    let old = f.path("provider/.cache/bridget");
    private_dir(&old);
    for (option, name) in [("--db", "bridget.db"), ("--fleet", "fleet.json")] {
        let path = old.join(name);
        private_file(&path, b"historique intact");
        let output = Process::start(
            f.command()
                .args(["identity", "migrate", "--apply", option])
                .arg(&path),
        )
        .finish();
        assert!(!output.status.success());
        let expected = "namespace historique interdit";
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(path).unwrap(), b"historique intact");
    }
}

#[test]
fn sous_etats_lies_sont_refuses_avant_bootstrap_ou_ecriture() {
    for relative in [
        "managed",
        "bridget.fleet.json",
        "agent-names/instance-test",
        "sessions",
    ] {
        let f = Fixture::new();
        let other = f.path("other-instance");
        private_dir(&other);
        private_file(&other.join("sentinelle"), b"ne pas interpreter ni modifier");
        let target = f.path("state").join(relative);
        private_dir(target.parent().unwrap());
        symlink(&other, &target).unwrap();
        let result = Process::start(f.command().arg("daemon")).finish();
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("symlink d'état interdit"));
        // L'API explicite ne doit pas contourner le préflight du binaire.
        let config = bridget_daemon::daemon::DaemonConfig {
            socket_path: f.path("state/bridget.sock"),
            db_path: f.path("state/bridget.db"),
            log_path: f.path("state/daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        assert!(
            bridget_daemon::daemon::run(config)
                .unwrap_err()
                .to_string()
                .contains("symlink d'état interdit")
        );
        assert_eq!(
            fs::read(other.join("sentinelle")).unwrap(),
            b"ne pas interpreter ni modifier"
        );
        assert!(!f.path("state/bridget.db").exists());
        assert!(
            !f.path("state/bridget.pid").exists(),
            "aucun bootstrap ne doit commencer"
        );
    }
}

#[test]
fn vrai_daemon_et_clients_n_utilisent_que_namespace_prive() {
    let f = Fixture::new();
    // Des fichiers historiques malformés ne doivent même pas être lus.
    private_dir(&f.path("provider/.config/bridget"));
    private_file(&f.path("provider/.config/bridget/agents.json"), b"invalide");
    private_file(&f.path("tmp/bridget-ne-pas-purger"), b"hors namespace");
    private_file(&f.path("provider/abonnement"), b"HOME fournisseur intact");
    let mut command = f.command();
    command
        .arg("daemon")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let daemon = Process::start(&mut command);
    let socket = f.path("state/bridget.sock");
    let deadline = Instant::now() + Duration::from_secs(20);
    // Readiness sur vraie connexion, pas une durée présumée de démarrage.
    loop {
        if std::os::unix::net::UnixStream::connect(&socket).is_ok() {
            break;
        }
        assert!(Instant::now() < deadline, "socket privée non prête");
        std::thread::yield_now();
    }
    let who = Process::start(f.command().args(["agents", "--json"])).finish();
    assert!(
        who.status.success(),
        "{}",
        String::from_utf8_lossy(&who.stderr)
    );
    serde_json::from_slice::<serde_json::Value>(&who.stdout).unwrap();
    assert!(f.path("state/bridget.db").exists());
    assert_eq!(
        fs::metadata(f.path("state")).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(f.path("state/bridget.db"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    assert!(!f.path("provider/.cache/bridget").exists());
    assert_eq!(
        fs::read(f.path("provider/abonnement")).unwrap(),
        b"HOME fournisseur intact"
    );
    assert_eq!(
        fs::read(f.path("tmp/bridget-ne-pas-purger")).unwrap(),
        b"hors namespace"
    );
    // Le MCP est lui aussi un vrai processus env_clear : sa résolution
    // d'identité et sa connexion doivent converger vers le même daemon.
    private_dir(&f.path("state/agent-names"));
    let name_file = f.path("state/agent-names/test-instance");
    private_file(&name_file, b"a3d27a89-80d5-4e0f-9b84-cf5523ecb026");
    let mut mcp_command = f.command();
    mcp_command
        .arg("mcp")
        .env("BRIDGET_AGENT_ID_FILE", &name_file)
        .env("BRIDGET_AGENT_INSTANCE_ID", "isolation-instance")
        .stdin(Stdio::piped());
    let mut mcp = Process::start(&mut mcp_command);
    {
        let mut input = mcp.0.stdin.take().unwrap();
        for frame in [
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
            serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_who","arguments":{}}}),
        ] {
            writeln!(input, "{frame}").unwrap();
        }
    }
    let output = mcp.finish();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frames: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let response = frames.iter().find(|frame| frame["id"] == 2).unwrap();
    assert!(response.get("error").is_none(), "{response}");
    assert_ne!(response["result"]["isError"], true, "{response}");
    assert_ne!(
        response["result"]["structuredContent"]["status"], "identity_not_found",
        "{response}"
    );
    // Un second daemon du même namespace ne remplace pas le premier.
    let duplicate = Process::start(f.command().arg("daemon")).finish();
    assert!(!duplicate.status.success());
    assert!(
        Process::start(f.command().arg("who"))
            .finish()
            .status
            .success()
    );
    drop(daemon);
}
