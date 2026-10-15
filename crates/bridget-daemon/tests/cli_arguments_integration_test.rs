use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn fixture_root(_label: &str) -> PathBuf {
    Path::new("/tmp").join(format!(
        "b89cli-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn run_cli(home: &Path, args: &[&str]) -> Output {
    run_cli_as(home, args, Some("89000000-0000-4000-8000-000000000701"))
}

#[test]
fn spec_091_development_requires_real_human_terminal_even_on_retry() {
    let root = fixture_root("scoped-posture");
    let first = run_cli(
        &root,
        &[
            "spawn",
            "codex",
            "--persistent",
            "--command-id",
            "scoped-cli-091",
            "--posture",
            "development",
        ],
    );
    assert!(!first.status.success());
    assert!(String::from_utf8_lossy(&first.stderr).contains("terminal interactif uniquement"));
    let replay = run_cli(&root, &["spawn", "codex", "--command-id", "scoped-cli-091"]);
    assert!(!replay.status.success());
    assert!(String::from_utf8_lossy(&replay.stderr).contains("terminal interactif uniquement"));
    assert!(!root.join("state/bridget.sock").exists());
    fs::remove_dir_all(root).unwrap();
}

fn run_cli_as(home: &Path, args: &[&str], agent_name: Option<&str>) -> Output {
    bridget_daemon::environment::ensure_private_directory(&home.join("state")).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    command
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("BRIDGET_HOME", home.join("state"))
        .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .stdin(Stdio::null());
    if let Some(agent_name) = agent_name {
        command.env("BRIDGET_AGENT_ID", agent_name);
    }
    command.output().expect("exécuter le vrai binaire bridget")
}

fn short_message_fixture_root() -> PathBuf {
    Path::new("/tmp").join(format!(
        "b41-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

#[derive(Debug)]
struct CapturedMessage {
    serialized_message: Option<String>,
    connection_accepted: bool,
}

fn capture_one_message(
    listener: UnixListener,
    stop: mpsc::Receiver<()>,
) -> thread::JoinHandle<CapturedMessage> {
    thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut register = String::new();
                    match reader.read_line(&mut register) {
                        Ok(0) => {
                            return CapturedMessage {
                                serialized_message: None,
                                connection_accepted: true,
                            };
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                            ) =>
                        {
                            return CapturedMessage {
                                serialized_message: None,
                                connection_accepted: true,
                            };
                        }
                        Ok(_) => {}
                        Err(error) => panic!("lire le Register CLI: {error}"),
                    }
                    if !register.contains("\"type\":\"Register\"") {
                        return CapturedMessage {
                            serialized_message: None,
                            connection_accepted: true,
                        };
                    }
                    let registration: bridget_transport::WrapperToDaemon =
                        serde_json::from_str(&register).unwrap();
                    let bridget_transport::WrapperToDaemon::Register { agent_id, .. } =
                        registration
                    else {
                        panic!("Register attendu");
                    };
                    writeln!(
                        stream,
                        "{}",
                        serde_json::to_string(&bridget_transport::DaemonToWrapper::Registered {
                            credential: None,
                            agent_id
                        })
                        .unwrap()
                    )
                    .unwrap();
                    stream.flush().unwrap();
                    let mut message = String::new();
                    match reader.read_line(&mut message) {
                        Ok(0) => {
                            return CapturedMessage {
                                serialized_message: None,
                                connection_accepted: true,
                            };
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                            ) =>
                        {
                            return CapturedMessage {
                                serialized_message: None,
                                connection_accepted: true,
                            };
                        }
                        Ok(_) => {}
                        Err(error) => panic!("lire le Send CLI: {error}"),
                    }
                    let sent: serde_json::Value = serde_json::from_str(&message).unwrap();
                    writeln!(
                        stream,
                        "{}",
                        serde_json::json!({"type":"Ack", "id":sent["id"]})
                    )
                    .unwrap();
                    stream.flush().unwrap();
                    return CapturedMessage {
                        serialized_message: Some(message),
                        connection_accepted: true,
                    };
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    // Accepter d'abord : une connexion déjà dans la file doit
                    // rester observable même si le client vient de sortir.
                    if stop.try_recv().is_ok() {
                        return CapturedMessage {
                            serialized_message: None,
                            connection_accepted: false,
                        };
                    }
                    thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("accepter la connexion CLI: {error}"),
            }
        }
    })
}

fn run_message_cli(args: &[&str]) -> (Output, Option<String>, bool, bool) {
    run_message_cli_as(
        args,
        Some("89000000-0000-4000-8000-000000000701"),
        Some("89000000-0000-4000-8000-000000000702"),
    )
}

fn run_message_cli_as(
    args: &[&str],
    agent_name: Option<&str>,
    previous_sender: Option<&str>,
) -> (Output, Option<String>, bool, bool) {
    let (output, captured, pid_exists, database_exists) =
        run_message_cli_observed(args, agent_name, previous_sender);
    (
        output,
        captured.serialized_message,
        pid_exists,
        database_exists,
    )
}

fn run_message_cli_observed(
    args: &[&str],
    agent_name: Option<&str>,
    previous_sender: Option<&str>,
) -> (Output, CapturedMessage, bool, bool) {
    let root = short_message_fixture_root();
    let cache = root.join("state");
    bridget_daemon::environment::ensure_private_directory(&cache).unwrap();
    if let Some(previous_sender) = previous_sender {
        let agent_name = agent_name.unwrap_or("human");
        fs::write(
            cache.join(format!("last-sender-{agent_name}")),
            format!("{previous_sender}\n"),
        )
        .unwrap();
    }
    let listener = UnixListener::bind(cache.join("bridget.sock")).unwrap();
    let (stop_tx, stop_rx) = mpsc::channel();
    let capture = capture_one_message(listener, stop_rx);

    let output = run_cli_as(&root, args, agent_name);
    let _ = stop_tx.send(());
    let captured = capture.join().unwrap();
    let pid_exists = cache.join("bridget.pid").exists();
    let database_exists = cache.join("bridget.db").exists();
    fs::remove_dir_all(&root).unwrap();
    (output, captured, pid_exists, database_exists)
}

fn assert_command_value_rejected(command: &str, option: &str, invalid_value: Option<&str>) {
    let mut args = match command {
        "send" => vec!["send", "--to", "89000000-0000-4000-8000-000000000702"],
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

fn assert_unknown_message_option_rejected(args: &[&str], unknown: &str) {
    let (output, serialized_message, pid_exists, database_exists) = run_message_cli(args);

    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    assert!(
        serialized_message.is_none(),
        "{unknown} a corrompu le message sérialisé: {serialized_message:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(unknown), "jeton absent de {stderr}");
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

fn assert_missing_message_option_value_rejected(args: &[&str], option: &str) {
    let (output, serialized_message, pid_exists, database_exists) = run_message_cli(args);

    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    assert!(
        serialized_message.is_none(),
        "{option} sans valeur a laissé partir {serialized_message:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(option), "option absente de {stderr}");
    assert!(
        stderr.contains("requiert une valeur"),
        "cause absente de {stderr}"
    );
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

fn assert_reply_contract_rejected(
    args: &[&str],
    agent_name: Option<&str>,
    previous_sender: Option<&str>,
    option: &str,
) {
    let (output, captured, pid_exists, database_exists) =
        run_message_cli_observed(args, agent_name, previous_sender);

    assert!(
        !captured.connection_accepted,
        "{args:?}: la contradiction a ouvert une connexion: {captured:?}"
    );
    assert!(
        captured.serialized_message.is_none(),
        "{args:?}: la contradiction a laissé partir {:?}",
        captured.serialized_message
    );
    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    assert!(output.stdout.is_empty(), "stdout inattendu: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(option), "option absente de {stderr}");
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

#[test]
fn daemon_stop_est_refuse_avant_tout_effet_de_bord() {
    let root = fixture_root("daemon");
    let cache = root.join("state");
    bridget_daemon::environment::ensure_private_directory(&cache).unwrap();
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
fn reply_refuse_les_valeurs_invalides_sans_dernier_expediteur() {
    let root = fixture_root("reply-sans-expediteur");
    bridget_daemon::environment::ensure_private_directory(&root.join("state")).unwrap();

    let outputs = ["--timeout", "--hops"]
        .map(|option| (option, run_cli(&root, &["reply", option, "abc", "message"])));
    fs::remove_dir_all(&root).unwrap();

    for (option, output) in outputs {
        assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(option), "option absente de {stderr}");
    }
}

#[test]
fn send_et_reply_refusent_timeout_sans_reply_avant_connexion() {
    for (args, previous_sender) in [
        (
            vec![
                "send",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "--timeout",
                "30",
                "message",
            ],
            Some("89000000-0000-4000-8000-000000000702"),
        ),
        (
            vec!["reply", "--timeout", "30", "message"],
            Some("89000000-0000-4000-8000-000000000702"),
        ),
    ] {
        assert_reply_contract_rejected(
            &args,
            Some("89000000-0000-4000-8000-000000000701"),
            previous_sender,
            "--timeout",
        );
    }
}

#[test]
fn send_et_reply_refusent_reply_humain_avant_connexion() {
    for (args, previous_sender) in [
        (
            vec![
                "send",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "--reply",
                "--timeout",
                "30",
                "message",
            ],
            Some("89000000-0000-4000-8000-000000000702"),
        ),
        (
            vec![
                "send",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "--from",
                "human",
                "--reply",
                "message",
            ],
            Some("89000000-0000-4000-8000-000000000702"),
        ),
        (
            vec!["reply", "--reply", "--timeout", "30", "message"],
            Some("89000000-0000-4000-8000-000000000702"),
        ),
    ] {
        assert_reply_contract_rejected(&args, None, previous_sender, "--reply");
    }
}

#[test]
fn send_et_reply_conservent_les_valeurs_numeriques_valides() {
    for args in [
        vec![
            "send",
            "--to",
            "89000000-0000-4000-8000-000000000702",
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
        assert_eq!(message["reply"], true);
        assert_eq!(message["reply_timeout"], 9);
        assert_eq!(message["hops"], 2);
        assert!(!pid_exists, "la commande ne doit pas créer le PID file");
        assert!(!database_exists, "la commande ne doit pas créer la base");
    }
}

#[test]
fn send_refuse_les_options_inconnues_sans_serialiser_de_message() {
    for (args, unknown) in [
        (
            vec![
                "send",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "--to-fallback",
            ],
            "--to-fallback",
        ),
        (
            vec![
                "send",
                "--sonde-avant",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "corps",
                "explicite",
            ],
            "--sonde-avant",
        ),
        (
            vec![
                "send",
                "--to",
                "89000000-0000-4000-8000-000000000702",
                "corps",
                "explicite",
                "--sonde-option-inexistante",
            ],
            "--sonde-option-inexistante",
        ),
    ] {
        assert_unknown_message_option_rejected(&args, unknown);
    }
}

#[test]
fn reply_refuse_les_options_inconnues_sans_serialiser_de_message() {
    for (args, unknown) in [
        (
            vec!["reply", "--sonde-avant", "corps", "explicite"],
            "--sonde-avant",
        ),
        (
            vec!["reply", "corps", "explicite", "--sonde-option-inexistante"],
            "--sonde-option-inexistante",
        ),
    ] {
        assert_unknown_message_option_rejected(&args, unknown);
    }
}

#[test]
fn le_separateur_preserve_un_corps_commencant_par_un_tiret() {
    for args in [
        vec![
            "send",
            "--to",
            "89000000-0000-4000-8000-000000000702",
            "--",
            "--body",
            "TEST",
            "IDENTITE",
            "CLI",
        ],
        vec!["reply", "--", "--body", "TEST", "IDENTITE", "CLI"],
    ] {
        let (output, message, pid_exists, database_exists) = run_message_cli(&args);
        assert!(output.status.success(), "sortie réelle: {output:?}");
        let message: serde_json::Value =
            serde_json::from_str(message.as_deref().expect("message sérialisé")).unwrap();
        assert_eq!(message["body"], "--body TEST IDENTITE CLI");
        assert!(!pid_exists, "la commande ne doit pas créer le PID file");
        assert!(!database_exists, "la commande ne doit pas créer la base");
    }
}

#[test]
fn send_nomme_les_options_de_texte_privees_de_valeur() {
    assert_missing_message_option_value_rejected(&["send", "--to"], "--to");
    assert_missing_message_option_value_rejected(
        &[
            "send",
            "--to",
            "89000000-0000-4000-8000-000000000702",
            "message",
            "--from",
        ],
        "--from",
    );
}

#[test]
fn message_humain_sans_demande_suivie_reste_envoye() {
    let (output, message, pid_exists, database_exists) = run_message_cli_as(
        &[
            "send",
            "--to",
            "89000000-0000-4000-8000-000000000702",
            "message",
        ],
        None,
        None,
    );
    assert!(output.status.success(), "sortie réelle: {output:?}");
    let message: serde_json::Value =
        serde_json::from_str(message.as_deref().expect("message sérialisé")).unwrap();
    assert_eq!(message["from"], "human");
    assert_eq!(message["reply"], false);
    assert!(message["reply_timeout"].is_null());
    assert!(!pid_exists, "la commande ne doit pas créer le PID file");
    assert!(!database_exists, "la commande ne doit pas créer la base");
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
fn cleanup_retire_est_refuse_meme_avec_les_anciens_arguments() {
    let root = fixture_root("cleanup-retire");
    let output = run_cli(&root, &["cleanup", "--dry-run", "SURPLUS"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("retiré du noyau"));
    assert!(!root.join("state/bridget.db").exists());
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

fn routing168_catalogue_cli_fixture(
    count: usize,
    json_output: bool,
    old: bool,
    machine: Option<&str>,
) -> (Output, Vec<bridget_transport::WrapperToDaemon>, PathBuf) {
    use bridget_transport::protocol::{CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole};
    use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
    let root = fixture_root("catalogue168");
    bridget_daemon::environment::ensure_private_directory(&root.join("state")).unwrap();
    let listener = UnixListener::bind(root.join("state/bridget.sock")).unwrap();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut captured = Vec::new();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let role: WrapperToDaemon = serde_json::from_str(&line).unwrap();
        assert!(matches!(
            role,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            }
        ));
        captured.push(role);
        writeln!(
            stream,
            "{}",
            serde_json::to_string(&DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
            .unwrap()
        )
        .unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        let hello: WrapperToDaemon = serde_json::from_str(&line).unwrap();
        assert!(
            matches!(&hello,WrapperToDaemon::ClientHello{contract_version,capabilities,..}
            if *contract_version==CLIENT_CONTRACT_VERSION && capabilities==&vec![ClientCapability::RoutingCatalogueV1])
        );
        captured.push(hello);
        writeln!(
            stream,
            "{}",
            serde_json::to_string(&DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                build_id: "fixture168".into(),
                horizon_secs: 600,
                issued_at_tolerance_secs: 30,
                capabilities: if old {
                    vec![]
                } else {
                    vec![ClientCapability::RoutingCatalogueV1]
                },
            })
            .unwrap()
        )
        .unwrap();
        line.clear();
        if old {
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "aucune requête catalogue sans capacité"
            );
        } else {
            reader.read_line(&mut line).unwrap();
            let request: WrapperToDaemon = serde_json::from_str(&line).unwrap();
            assert!(matches!(
                request,
                WrapperToDaemon::NativeDelegation {
                    request:
                        bridget_transport::protocol::NativeDelegationRequest::RoutingCatalogueV1 {}
                }
            ));
            captured.push(request);
            let options=(0..count).map(|index|serde_json::json!({
                "machine_id":if index%2==0{"machine-a"}else{"machine-b"},"engine":"codex",
                "connection_id":"environment168","model":format!("real-model-{index:03}"),
                "supported_efforts":[],"default_effort":null,"provenance":"operator_observation",
                "observed_at":100,"entitlement":"unknown",
            })).collect::<Vec<_>>();
            writeln!(
                stream,
                "{}",
                serde_json::to_string(&DaemonToWrapper::NativeDelegationResult {
                    result: serde_json::json!({"version":1,"observed_at":100,"options":options})
                })
                .unwrap()
            )
            .unwrap();
        }
        captured
    });
    let mut args = vec!["delegate", "--catalogue"];
    if let Some(machine) = machine {
        args.extend(["--machine", machine]);
    }
    if json_output {
        args.push("--json");
    }
    let output = run_cli_as(&root, &args, None);
    (output, server.join().unwrap(), root)
}

#[test]
fn spec168_catalogue_cli_humain_negocie_12_128_et_filtre_machine_sans_identite() {
    for count in [12, 128] {
        let (output, messages, root) = routing168_catalogue_cli_fixture(count, true, false, None);
        assert!(output.status.success(), "{output:?}");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["options"].as_array().unwrap().len(), count);
        assert_eq!(
            value["options"][0]["default_effort"],
            serde_json::Value::Null
        );
        assert_eq!(
            messages.len(),
            3,
            "pas de Register agent ni opération cachée"
        );
        assert!(!root.join("state/bridget.db").exists());
        fs::remove_dir_all(root).unwrap();
        let (output, _, root) =
            routing168_catalogue_cli_fixture(count, true, false, Some("machine-a"));
        assert!(output.status.success());
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["options"].as_array().unwrap().len(), count / 2);
        fs::remove_dir_all(root).unwrap();
    }
    let (output, _, root) = routing168_catalogue_cli_fixture(12, false, false, None);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(text.lines().count(), 13);
    assert!(text.contains("real-model-011"));
    assert!(text.contains("natif"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_catalogue_cli_ancien_maitre_refuse_avant_toute_lecture() {
    let (output, messages, root) = routing168_catalogue_cli_fixture(12, true, true, None);
    assert!(!output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "capability_unavailable");
    assert_eq!(messages.len(), 2);
    assert!(!root.join("state/bridget.db").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_delegate_workload_et_axes_refusent_les_valeurs_invalides_avant_connexion() {
    let root = fixture_root("invalid168");
    for option in ["--engine", "--connection-id", "--workload"] {
        let value = if option == "--workload" {
            "compile"
        } else {
            ""
        };
        let output = run_cli_as(
            &root,
            &[
                "delegate",
                "Mission",
                "--cwd",
                root.to_str().unwrap(),
                "--request-id",
                "invalid168",
                option,
                value,
                "--json",
            ],
            None,
        );
        assert!(!output.status.success());
        assert!(!root.join("state/bridget.sock").exists());
        assert!(!root.join("state/bridget.db").exists());
    }
    fs::remove_dir_all(root).unwrap();
}
