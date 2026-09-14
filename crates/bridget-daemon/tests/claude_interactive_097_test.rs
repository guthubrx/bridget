//! Recette du wrapper `bridget claude` sans tmux : vrai daemon, vrai wrapper,
//! faux fournisseur sous pseudo-terminal. Aucun compte, aucun tmux.
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// Un harnais à la fois : identité Bridget fixe et faux fournisseur nommé,
/// deux scénarios simultanés se verraient l'un l'autre.
static HARNESS: Mutex<()> = Mutex::new(());

#[test]
fn claude_interactif_refuse_un_pipe_avant_presence_ou_fournisseur() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("claude")
        .stdin(Stdio::null())
        .env_remove("TMUX")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("stdin et stdout sur un terminal"),
        "refus nommé attendu : {stderr}"
    );
}

fn harness(mode: &str) {
    let _serial = HARNESS.lock().unwrap_or_else(|poison| poison.into_inner());
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/claude_interactive_097.py"
    );
    let output = Command::new("/usr/bin/python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_bridget"))
        .arg(mode)
        .env_remove("TMUX")
        .env_remove("TMUX_PANE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

#[test]
fn session_pty_relaie_remet_et_restaure_le_terminal() {
    harness("--basic");
}

#[test]
fn code_de_sortie_du_fournisseur_est_relaye() {
    harness("--exit-code");
}

#[test]
fn bypass_explicite_de_l_humain_est_relaye_tel_quel() {
    harness("--user-bypass");
}

#[test]
fn saisie_partielle_preservee_corps_hostile_refuse_et_notification_remise() {
    harness("--busy");
}

#[test]
fn sigterm_du_wrapper_termine_le_fournisseur_et_restaure_le_terminal() {
    harness("--sigterm");
}

#[test]
fn daemon_redemarre_meme_identite_notification_et_remise_par_le_pty() {
    harness("--daemon-restart");
}

#[test]
#[ignore = "recette réelle Claude Code : BRIDGET_CLAUDE_097_BIN, BRIDGET_CLAUDE_097_CWD et compte local requis"]
fn vraie_tui_claude_code_recoit_un_message_et_repond() {
    harness("--live");
}

#[test]
fn type_tmux_sans_pane_est_refuse_avant_toute_presence() {
    // Aucun serveur tmux joignable : répertoire de sockets privé et vide.
    let sockets = std::env::temp_dir().join(format!("bridget-097-tmux-{}", std::process::id()));
    std::fs::create_dir_all(&sockets).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("gemini")
        .stdin(Stdio::null())
        .env_remove("TMUX")
        .env_remove("TMUX_PANE")
        .env("TMUX_TMPDIR", &sockets)
        .output()
        .unwrap();
    let _ = std::fs::remove_dir_all(&sockets);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aucun pane tmux") && stderr.contains("bridget spawn gemini"),
        "refus nommé avec l'alternative attendue : {stderr}"
    );
}
