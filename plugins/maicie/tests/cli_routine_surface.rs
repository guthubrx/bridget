//! Contrat CLI des six sous-commandes `maicie routine`.
//! Surface gelée : propose|approve|list|pause|resume|show.

use std::process::Command;

fn maicie() -> Command {
    Command::new(env!("CARGO_BIN_EXE_maicie"))
}

#[test]
fn surface_routine_refuse_une_action_inconnue() {
    let output = maicie()
        .args(["routine", "explode", "--config", "/tmp/nope.json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("propose|approve|list|pause|resume|show"),
        "stderr={stderr}"
    );
}

#[test]
fn surface_routine_exige_une_action() {
    let output = maicie().args(["routine"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("propose|approve|list|pause|resume|show"),
        "stderr={stderr}"
    );
}

#[test]
fn surface_propose_exige_suite_comme_f36() {
    let output = maicie()
        .args([
            "routine",
            "propose",
            "--config",
            "/tmp/nope.json",
            "--goal",
            "x",
            "--to",
            "prospective",
            "--period-secs",
            "60",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("suite") || stderr.contains("--suite"),
        "stderr={stderr}"
    );
}

#[test]
fn surface_approve_pause_resume_show_exigent_id() {
    for action in ["approve", "pause", "resume", "show"] {
        let output = maicie()
            .args(["routine", action, "--config", "/tmp/nope.json"])
            .output()
            .unwrap();
        assert!(!output.status.success(), "action={action}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--id") || stderr.contains("id"),
            "action={action} stderr={stderr}"
        );
    }
}

#[test]
fn surface_list_exige_config() {
    let output = maicie().args(["routine", "list"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--config"), "stderr={stderr}");
}
