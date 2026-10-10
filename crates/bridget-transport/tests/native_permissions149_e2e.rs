//! Session 149 — E2E transport du refus can_use_tool (S149-33 branche c,
//! oracle G-P-08 côté transport). Le CLI Claude est remplacé par `/bin/sh` qui
//! parle le flux JSON : aucune mission fournisseur, aucun modèle. Chaque test
//! atteste le request_id exact, l'absence de succès silencieux et le refus
//! nommé `provider_permission_denied`.

use bridget_core::BridgetMessage;
use bridget_transport::managed_session::{ManagedEventKind, ManagedSession, ManagedTerminal};
use bridget_transport::transport::Transport;
use bridget_transport::{ClaudeStreamJsonOptions, ClaudeStreamJsonTransport};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn message(id: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new("bridget", "claude", "mission-149");
    message.id = id.to_string();
    message.reply = true;
    message
}

/// Fixture privée : script faux CLI 0600 dans un dossier 0700, fichier de
/// capture séparé. Le transport reçoit `["-c", script, "--", capture]`.
fn fixture(label: &str, body: &str) -> (ClaudeStreamJsonOptions, PathBuf) {
    let root = std::env::temp_dir().join(format!(
        "bridget-np149e-{}-{}-{}",
        label,
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let script = root.join("cli.sh");
    fs::write(&script, body).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o600)).unwrap();
    let capture = root.join("capture.jsonl");
    let options = ClaudeStreamJsonOptions {
        command: "/bin/sh".to_string(),
        args: vec![
            "-c".to_string(),
            fs::read_to_string(&script).unwrap(),
            "--".to_string(),
            capture.to_string_lossy().into_owned(),
        ],
        provider_kind: "claude".to_string(),
        queue_capacity: 4,
        notify_timeout_secs: 4,
        provider_observation: None,
        session_store_root: None,
        agent_name: None,
    };
    (options, capture)
}

fn attendre(
    transport: &ClaudeStreamJsonTransport,
    condition: impl Fn(&ManagedEventKind) -> bool,
) -> bool {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            if condition(&event.kind) {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

fn refus_nomme(kind: &ManagedEventKind, message_id: &str) -> bool {
    matches!(
        kind,
        ManagedEventKind::DeliveryRejected { message_id: id, reason }
            if id == message_id && reason == "provider_permission_denied"
    )
}

/// Le faux CLI demande un outil, reçoit le refus du transport, le copie tel
/// quel dans la capture, puis rend un résultat apparemment vertueux (« OK »).
const SCRIPT_DENY: &str = r#"printf '%s\n' '{"type":"control_request","request_id":"perm-149","request":{"subtype":"can_use_tool","tool_name":"Bash","input":{}}}'
while IFS= read -r line; do
  case "$line" in
    *control_response*)
      printf '%s\n' "$line" >> "$1"
      printf '%s\n' '{"type":"assistant","message":{"id":"msg-149","role":"assistant","content":[{"type":"text","text":"OK"}]}}'
      printf '%s\n' '{"type":"result","is_error":false,"terminal_reason":"completed"}'
      exit 0;;
  esac
done
"#;

#[test]
fn native149_can_use_tool_deny_est_correle_par_request_id_exact_et_refuse_la_remise() {
    let (options, capture) = fixture("deny", SCRIPT_DENY);
    let mut transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    transport.deliver(&message("mission-149")).unwrap();
    assert!(
        attendre(&transport, |kind| refus_nomme(kind, "mission-149")),
        "la remise doit être refusée par son nom malgré le résultat « OK »"
    );
    assert!(
        !attendre(&transport, |kind| matches!(
            kind,
            ManagedEventKind::TurnFinished { .. }
        )),
        "aucun TurnFinished sur un tour refusé"
    );
    transport.stop();
    let trace = fs::read_to_string(&capture).unwrap_or_default();
    assert!(
        trace.contains("\"request_id\":\"perm-149\""),
        "le request_id doit être corrélé à l'identique : {trace}"
    );
    assert!(trace.contains("\"behavior\":\"deny\""), "{trace}");
    assert!(trace.contains("provider_permission_denied"), "{trace}");
    let _ = fs::remove_dir_all(capture.parent().unwrap());
}

#[test]
fn native149_remise_positive_termine_le_tour_sans_refus() {
    let script = concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"id\":\"msg-ok\",\"role\":\"assistant\",\"content\":[{\"type\":\"text\",\"text\":\"OK\"}]}}'; ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"OK\"}'; ",
        "done"
    );
    let (options, capture) = fixture("positif", script);
    let mut transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    transport.deliver(&message("mission-149")).unwrap();
    let mut refuse = false;
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut termine = false;
    while Instant::now() < deadline && !(termine && !refuse) {
        for event in transport.drain_events() {
            termine |= matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    ref message,
                    terminal: ManagedTerminal::Completed,
                    ..
                } if message.id == "mission-149"
            );
            refuse |= matches!(event.kind, ManagedEventKind::DeliveryRejected { .. });
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(termine, "le tour positif doit finir en Completed");
    assert!(!refuse, "aucun refus nommé sans demande de permission");
    assert!(
        fs::read_to_string(&capture).unwrap_or_default().is_empty(),
        "aucune réponse de contrôle sans demande"
    );
    transport.stop();
    let _ = fs::remove_dir_all(capture.parent().unwrap());
}

#[test]
fn native149_result_avec_permission_denials_est_refuse_sans_trame() {
    let script = concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",",
        "\"permission_denials\":[{\"tool_name\":\"Bash\",\"request_id\":\"perm-149\"}]}'; ",
        "done"
    );
    let (options, capture) = fixture("denials", script);
    let mut transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    transport.deliver(&message("mission-149")).unwrap();
    assert!(
        attendre(&transport, |kind| refus_nomme(kind, "mission-149")),
        "les permission_denials du résultat refusent la remise"
    );
    transport.stop();
    assert!(
        fs::read_to_string(&capture).unwrap_or_default().is_empty(),
        "aucune trame de contrôle n'a été émise"
    );
    let _ = fs::remove_dir_all(capture.parent().unwrap());
}

#[test]
fn native149_control_request_sans_request_id_n_obtient_aucune_reponse() {
    let script = concat!(
        "printf '%s\\n' '{\"type\":\"control_request\",\"request\":{\"subtype\":\"can_use_tool\",\"tool_name\":\"Bash\"}}'\n",
        "line=\"\"\n",
        "while IFS= read -t 2 -r line; do\n",
        "  case \"$line\" in\n",
        "    *control_response*) printf '%s\\n' \"$line\" >> \"$1\";;\n",
        "  esac\n",
        "done\n",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\"}'\n"
    );
    let (options, capture) = fixture("sans-id", script);
    let mut transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    transport.deliver(&message("mission-149")).unwrap();
    assert!(
        attendre(&transport, |kind| refus_nomme(kind, "mission-149")),
        "un control_request sans request_id reste un refus nommé"
    );
    transport.stop();
    assert!(
        fs::read_to_string(&capture).unwrap_or_default().is_empty(),
        "aucune control_response ne répond à un request_id absent"
    );
    let _ = fs::remove_dir_all(capture.parent().unwrap());
}

/// Régression revue r1 (F2) : le cwd fait partie du fait — à session et
/// politique identiques, un cwd mué incrémente la révision observée.
#[test]
fn native149_fait_claude_incremente_la_revision_quand_le_cwd_change() {
    let script = concat!(
        "printf '%s\\n' '{\"type\":\"system\",\"permissionMode\":\"default\",\"session_id\":\"s-149\",\"cwd\":\"/private/tmp/claude-149-a\",\"tools\":[\"Read\",\"Glob\"]}'\n",
        "line=\"\"\n",
        "IFS= read -r line\n",
        "printf '%s\\n' '{\"type\":\"system\",\"permissionMode\":\"default\",\"session_id\":\"s-149\",\"cwd\":\"/private/tmp/claude-149-b\",\"tools\":[\"Read\",\"Glob\"]}'\n",
        "sleep 6\n"
    );
    let (options, capture) = fixture("revision-cwd", script);
    let mut transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut revision_a = None;
    while Instant::now() < deadline {
        if let Some((session, _, revision, cwd)) = transport.provider_permissions() {
            if session == "s-149" && cwd.as_deref() == Some("/private/tmp/claude-149-a") {
                revision_a = Some(revision);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let revision_a = revision_a.unwrap_or_else(|| panic!("première observation manquante"));
    // La remise d'un tour fait avancer le flux : le second system event porte
    // le même mode, le même outillage et la même session, seul le cwd change.
    transport.deliver(&message("mission-149")).unwrap();
    let mut revision_b = None;
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if let Some((session, _, revision, cwd)) = transport.provider_permissions() {
            if session == "s-149" && cwd.as_deref() == Some("/private/tmp/claude-149-b") {
                revision_b = Some(revision);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    transport.stop();
    let revision_b = revision_b.unwrap_or_else(|| panic!("observation après mutation du cwd manquante"));
    assert_eq!(
        revision_b,
        revision_a + 1,
        "un cwd mué à politique identique doit incrémenter la révision d'une unité"
    );
    let _ = fs::remove_dir_all(capture.parent().unwrap());
}
