use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-cli-arguments-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn run_cli(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("BRIDGET_AGENT_NAME", "probe")
        .stdin(Stdio::null())
        .output()
        .expect("exécuter le vrai binaire bridget")
}

fn short_message_fixture_root() -> PathBuf {
    std::env::temp_dir().join(format!(
        "b41-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn capture_one_message(listener: UnixListener) -> thread::JoinHandle<Option<String>> {
    thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_millis(100);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut register = String::new();
                    reader.read_line(&mut register).unwrap();
                    assert!(register.contains("\"type\":\"Register\""));
                    writeln!(stream, "{{\"type\":\"Registered\",\"name\":\"probe\"}}").unwrap();
                    stream.flush().unwrap();
                    let mut message = String::new();
                    reader.read_line(&mut message).unwrap();
                    writeln!(stream, "{{\"type\":\"Ack\",\"id\":\"probe-ack\"}}").unwrap();
                    stream.flush().unwrap();
                    return Some(message);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return None;
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("accepter la connexion CLI: {error}"),
            }
        }
    })
}

fn run_message_cli(args: &[&str]) -> (Output, Option<String>, bool, bool) {
    let root = short_message_fixture_root();
    let cache = root.join(".cache/bridget");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("last-sender-probe"), "destinataire\n").unwrap();
    let listener = UnixListener::bind(cache.join("bridget.sock")).unwrap();
    let capture = capture_one_message(listener);

    let output = run_cli(&root, args);
    let serialized_message = capture.join().unwrap();
    let pid_exists = cache.join("bridget.pid").exists();
    let database_exists = cache.join("bridget.db").exists();
    fs::remove_dir_all(&root).unwrap();
    (output, serialized_message, pid_exists, database_exists)
}

fn assert_command_value_rejected(command: &str, option: &str, invalid_value: Option<&str>) {
    let mut args = match command {
        "send" => vec!["send", "--to", "destinataire"],
        "reply" => vec!["reply"],
        _ => panic!("commande de fixture inconnue: {command}"),
    };
    if let Some(value) = invalid_value {
        args.extend([option, value, "message"]);
    } else {
        args.extend(["message", option]);
    }
    let (output, serialized_message, pid_exists, database_exists) = run_message_cli(&args);

    assert!(
        serialized_message.is_none(),
        "{command} {option}: la valeur invalide a laissé partir {serialized_message:?}"
    );
    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    assert!(output.stdout.is_empty(), "stdout inattendu: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(option), "option absente de {stderr}");
    if let Some(value) = invalid_value {
        assert!(stderr.contains(value), "valeur absente de {stderr}");
    }
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

#[test]
fn daemon_stop_est_refuse_avant_tout_effet_de_bord() {
    let root = fixture_root("daemon");
    let cache = root.join(".cache/bridget");
    fs::create_dir_all(&cache).unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o500)).unwrap();

    let output = run_cli(&root, &["daemon", "stop"]);
    let pid_exists = cache.join("bridget.pid").exists();
    let socket_exists = cache.join("bridget.sock").exists();
    let database_exists = cache.join("bridget.db").exists();

    fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
    fs::remove_dir_all(&root).unwrap();

    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("daemon"), "commande absente de {stderr}");
    assert!(stderr.contains("stop"), "argument absent de {stderr}");
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!socket_exists, "la validation ne doit pas créer la socket");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

#[test]
fn send_refuse_un_timeout_invalide_avant_tout_effet_de_bord() {
    for value in [Some("abc"), Some("0"), Some("18446744073709551616"), None] {
        assert_command_value_rejected("send", "--timeout", value);
    }
}

#[test]
fn reply_refuse_un_timeout_invalide_avant_de_lire_son_etat() {
    for value in [Some("abc"), Some("0"), Some("18446744073709551616"), None] {
        assert_command_value_rejected("reply", "--timeout", value);
    }
}

#[test]
fn send_refuse_des_hops_invalides_avant_tout_effet_de_bord() {
    for value in [Some("abc"), Some("0"), Some("-1"), Some("2147483648"), None] {
        assert_command_value_rejected("send", "--hops", value);
    }
}

#[test]
fn reply_refuse_des_hops_invalides_avant_de_lire_son_etat() {
    for value in [Some("abc"), Some("0"), Some("-1"), Some("2147483648"), None] {
        assert_command_value_rejected("reply", "--hops", value);
    }
}

#[test]
fn send_et_reply_conservent_les_valeurs_numeriques_valides() {
    for args in [
        vec![
            "send",
            "--to",
            "destinataire",
            "--reply",
            "--timeout",
            "9",
            "--hops",
            "2",
            "message",
        ],
        vec![
            "reply",
            "--reply",
            "--timeout",
            "9",
            "--hops",
            "2",
            "message",
        ],
    ] {
        let (output, message, pid_exists, database_exists) = run_message_cli(&args);
        assert!(output.status.success(), "sortie réelle: {output:?}");
        let message: serde_json::Value =
            serde_json::from_str(message.as_deref().expect("message sérialisé")).unwrap();
        assert_eq!(message["reply_timeout"], 9);
        assert_eq!(message["hops"], 2);
        assert!(!pid_exists, "la commande ne doit pas créer le PID file");
        assert!(!database_exists, "la commande ne doit pas créer la base");
    }
}

#[test]
fn toutes_les_commandes_fermees_refusent_le_surplus_dans_le_vrai_binaire() {
    let root = fixture_root("matrice");
    fs::create_dir_all(&root).unwrap();
    let cases = vec![
        vec!["mcp", "SURPLUS"],
        vec!["discover", "SURPLUS"],
        vec!["status", "SURPLUS"],
        vec!["ledger", "SURPLUS"],
        vec!["version", "SURPLUS"],
        vec!["help", "SURPLUS"],
        vec!["cancel", "--SURPLUS"],
        vec!["cancel", "demande", "SURPLUS"],
        vec!["hook", "claude-runtime", "SURPLUS"],
        vec!["hook", "claude-statusline", "SURPLUS"],
        vec!["install-hooks", "--remove", "SURPLUS"],
        vec!["domain", "revue", "SURPLUS"],
        vec!["dnd", "off", "SURPLUS"],
        vec!["agents", "--json", "SURPLUS"],
        vec!["who", "--domain", "revue", "SURPLUS"],
        vec!["cleanup", "--dry-run", "SURPLUS"],
    ];

    for args in cases {
        let output = run_cli(&root, &args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{} n'a pas refusé le surplus: {output:?}",
            args.join(" ")
        );
        assert!(
            stderr.contains("SURPLUS"),
            "{} n'a pas nommé le surplus dans {stderr}",
            args.join(" ")
        );
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nom_de_hook_inconnu_reste_une_exception_fail_soft() {
    let root = fixture_root("hook-inconnu");
    fs::create_dir_all(&root).unwrap();
    let output = run_cli(&root, &["hook", "extension-future", "payload-opaque"]);
    fs::remove_dir_all(root).unwrap();

    assert!(output.status.success(), "sortie réelle: {output:?}");
    assert!(output.stdout.is_empty(), "stdout doit rester vide");
}
