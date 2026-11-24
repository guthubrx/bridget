//! La suppression d'une façade doit refuser son ancienne surface avant tout I/O.
use std::fs;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn ui_est_refusee_avant_namespace_socket_endpoint_ou_execution() {
    let root = fs::canonicalize("/tmp").unwrap().join(format!(
        "b89ui-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    ));
    fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
    let fixture = Fixture(root);
    let endpoint = fixture.0.join("ui-endpoint.json");
    let sentinel = br#"{"token":"secret-de-fixture-ne-doit-jamais-sortir","port":1}"#;
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&endpoint)
        .unwrap();
    file.write_all(sentinel).unwrap();
    let socket = fixture.0.join("s");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    // L'ancienne sous-commande `ui endpoint` retirée en 089 reste strictement refusée
    // avant tout accès au namespace, au socket ou au fichier BRIDGET_UI_ENDPOINT.
    for args in [vec!["ui", "endpoint", "--json"], vec!["ui", "endpoint"]] {
        let stdout = fixture.0.join("stdout");
        let stderr = fixture.0.join("stderr");
        let output = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&stdout)
            .unwrap();
        let error = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&stderr)
            .unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args(&args)
            .env_clear()
            .env("HOME", fixture.0.join("home-absent"))
            .env("BRIDGET_HOME", fixture.0.join("namespace-absent"))
            .env("BRIDGET_SOCKET", &socket)
            .env("BRIDGET_UI_ENDPOINT", &endpoint)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(output)
            .stderr(error)
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                let observed = Command::new("/bin/ps")
                    .args(["-p", &child.id().to_string(), "-o", "ppid=,command="])
                    .output()
                    .unwrap();
                let command = String::from_utf8_lossy(&observed.stdout);
                assert!(
                    command
                        .trim_start()
                        .starts_with(&std::process::id().to_string())
                        && command.contains(env!("CARGO_BIN_EXE_bridget"))
                        && !command.to_lowercase().contains("firefox"),
                    "PID non possédé : {command}"
                );
                child.kill().unwrap(); // uniquement cet enfant isolé, après identification
                child.wait().unwrap();
                panic!("ancienne surface UI ne refuse pas dans le budget");
            }
            std::thread::sleep(Duration::from_millis(5)); // watchdog, pas synchronisation métier
        };
        assert_eq!(status.code(), Some(2));
        assert!(fs::read(&stdout).unwrap().is_empty());
        assert!(
            fs::read_to_string(&stderr)
                .unwrap()
                .contains("interface retirée du noyau")
        );
        assert_eq!(fs::read(&endpoint).unwrap(), sentinel);
        assert!(!fixture.0.join("namespace-absent").exists());
        assert!(!fixture.0.join("home-absent").exists());
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    }

    // La nouvelle commande unifiée `bridget ui --help` expose la console locale
    // sans toucher BRIDGET_UI_ENDPOINT ni ouvrir de socket ou créer de namespace.
    let help = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["ui", "--help"])
        .env_clear()
        .env("HOME", fixture.0.join("home-absent"))
        .env("BRIDGET_HOME", fixture.0.join("namespace-absent"))
        .env("BRIDGET_SOCKET", &socket)
        .env("BRIDGET_UI_ENDPOINT", &endpoint)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert_eq!(help.status.code(), Some(0));
    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(
        help_stdout.contains("bridget ui [usage|accounts|machines|settings]"),
        "aide ui inattendue : {help_stdout}"
    );
    assert_eq!(fs::read(&endpoint).unwrap(), sentinel);
    assert!(!fixture.0.join("namespace-absent").exists());
    assert!(!fixture.0.join("home-absent").exists());
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
}

#[test]
fn le_noyau_ne_compile_plus_de_serveur_http_ni_de_renderer() {
    let modules = include_str!("../src/lib.rs");
    for forbidden in [
        "pub mod ui;",
        "pub mod mission_projection;",
        "pub mod artifact_fetch;",
    ] {
        assert!(!modules.contains(forbidden));
    }
    let manifest = include_str!("../Cargo.toml");
    for dependency in ["reqwest", "tauri", "hyper"] {
        assert!(
            !manifest
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("{dependency} =")))
        );
    }
    // Session 177 : axum sert le compagnon mobile authentifié et borné.
    // Sessions 300+ : axum sert aussi la console web locale authentifiée par jeton
    // sur loopback (127.0.0.1) dans `console_{ui,admin,login}*.rs`.
    // L'ancienne interface web retirée en 089 ne doit pas revenir ailleurs.
    fn sources(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                sources(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    let outside: Vec<_> = files
        .iter()
        .filter(|path| !path.starts_with(src.join("mobile")))
        .filter(|path| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !matches!(
                name,
                "console_ui.rs"
                    | "console_ui_tests.rs"
                    | "console_admin.rs"
                    | "console_admin_tests.rs"
                    | "console_login.rs"
            )
        })
        .filter(|path| fs::read_to_string(path).unwrap().contains("axum::"))
        .collect();
    assert!(
        outside.is_empty(),
        "axum hors du compagnon mobile et de la console locale : {outside:?}"
    );
}
