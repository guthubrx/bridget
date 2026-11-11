//! CLI264 → socket réelle → daemon → remise au wrapper, sans modèle ni réseau.
#![allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;
use bridget_core::MessageIntent;
use bridget_transport::protocol::{CommunicationProjectSource, PresenceMode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use support::*;

fn fact(target: &mut Client, root: &Path) {
    target.send(WrapperToDaemon::CommunicationProjectFact {
        root: root.to_string_lossy().into_owned(),
        source: CommunicationProjectSource::Git,
        host: "idempotency-isolated".into(),
        worktree_root: None,
    });
    assert!(matches!(
        target.receive(),
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
}

fn grant(root: &Path, project: &Path, mode: &str) -> PathBuf {
    let path = root.join("authorization264.json");
    let token = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    private_write(
        &path,
        serde_json::to_vec(&json!({"version":1,"token":token,"issuer_scope":SCOPE,
        "project_root":project,"targets":[RECIPIENT],"intent":mode}))
        .unwrap(),
    )
    .unwrap();
    let output = isolated_command(root)
        .args(["loop-control", "authorize", "--file"])
        .arg(&path)
        .arg("--approve")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "grant: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&token));
    path
}

fn command(
    root: &Path,
    project: &Path,
    id: &str,
    at: u64,
    mode: &str,
    grant: Option<&Path>,
) -> Command {
    let mut command = isolated_command(root);
    command
        .args(["send", "--to", RECIPIENT, "--id", id, "--issued-at"])
        .arg(at.to_string())
        .args(["--issuer-scope", SCOPE, "--project-root"])
        .arg(project);
    if mode != "queue" {
        command.args(["--intent", mode]);
    }
    if let Some(path) = grant {
        command.env("BRIDGET_LOOP_CONTROL_FILE", path);
    }
    command.args([
        "--",
        if id == "loop264-message" {
            "Action initiale de qualification264"
        } else {
            "Action refusée pendant DND, reprise après sa levée"
        },
    ]);
    command
}

#[test]
fn spec264_e2e_cli_modes_and_replay_after_revocation() {
    for (mode, intent) in [
        ("queue", None),
        ("steer", Some(MessageIntent::SteerCurrent)),
        ("interrupt", Some(MessageIntent::InterruptAndStart)),
    ] {
        let root = test_root("loop264");
        let project = root.join("project");
        fs::create_dir(&project).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&project)
                .status()
                .unwrap()
                .success()
        );
        let daemon = spawn_daemon(&root, None);
        let mut target = Client::connect(&socket(&root));
        target.send(WrapperToDaemon::Register {
            agent_type: "fixture".into(),
            identity_version: 2,
            agent_id: RECIPIENT.into(),
            host: Some("idempotency-isolated".into()),
            transport: Some("acp".into()),
            channel: None.into(),
            mode: Some(PresenceMode::Acp),
            location: None,
            os: Some("test".into()),
            instance_id: Some("loop264-target".into()),
            domain: None,
            journal_available: None,
            turn_in_progress: false,
            turn_attest: false,
            queue_fallback: false,
        });
        assert!(matches!(
            target.receive(),
            DaemonToWrapper::Registered { .. }
        ));
        fact(&mut target, &project);
        let authorization = (mode != "queue").then(|| grant(&root, &project, mode));
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut cli = command(
            &root,
            &project,
            "loop264-message",
            at,
            mode,
            authorization.as_deref(),
        );
        let sender = thread::spawn(move || cli.output().unwrap());
        let DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            delivery_generation,
            ..
        } = receive_delivery(&mut target)
        else {
            unreachable!()
        };
        assert_eq!(
            message.intent, intent,
            "mode effectivement remis au wrapper"
        );
        target.send(WrapperToDaemon::DeliverAcked {
            delivery_id,
            delivery_generation,
        });
        let output = sender.join().unwrap();
        assert!(
            output.status.success(),
            "send: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mode == "steer" {
            target.send(WrapperToDaemon::Availability {
                agent: RECIPIENT.into(),
                until_secs: Some(at + 60),
            });
            assert!(matches!(target.receive(), DaemonToWrapper::Ack { .. }));
            let refused = command(
                &root,
                &project,
                "loop264-rejected",
                at,
                mode,
                authorization.as_deref(),
            )
            .output()
            .unwrap();
            assert!(!refused.status.success());
            let text = String::from_utf8_lossy(&refused.stderr);
            assert!(
                text.contains("loop_target_do_not_disturb")
                    && text.contains("rejected")
                    && text.contains("nouveaux id et issued_at"),
                "{text}"
            );
            target.send(WrapperToDaemon::Availability {
                agent: RECIPIENT.into(),
                until_secs: None,
            });
            assert!(matches!(target.receive(), DaemonToWrapper::Ack { .. }));
            let old = command(
                &root,
                &project,
                "loop264-rejected",
                at,
                mode,
                authorization.as_deref(),
            )
            .output()
            .unwrap();
            assert!(
                !old.status.success(),
                "la clé refusée reste finale après levée DND"
            );
            assert_no_delivery(&mut target);
            target
                .reader
                .get_ref()
                .set_read_timeout(Some(CLIENT_READ_TIMEOUT))
                .unwrap();
            let mut fresh = command(
                &root,
                &project,
                "loop264-after-dnd",
                at + 1,
                mode,
                authorization.as_deref(),
            );
            let sender = thread::spawn(move || {
                let output = fresh.output().unwrap();
                if !output.status.success() {
                    eprintln!(
                        "nouvel envoi après DND : {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                output
            });
            let DaemonToWrapper::DeliverIdempotent {
                message,
                delivery_id,
                delivery_generation,
                ..
            } = receive_delivery(&mut target)
            else {
                unreachable!()
            };
            assert_eq!(message.intent, intent);
            target.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
            assert!(sender.join().unwrap().status.success());
        }
        if let Some(path) = authorization.as_ref() {
            let output = isolated_command(&root)
                .args(["loop-control", "revoke", "--file"])
                .arg(path)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "revoke: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let output = command(
            &root,
            &project,
            "loop264-message",
            at,
            mode,
            authorization.as_deref(),
        )
        .output()
        .unwrap();
        assert!(
            output.status.success(),
            "rejeu du même reçu : {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_no_delivery(&mut target);
        drop(target);
        drop(daemon);
        fs::remove_dir_all(root).unwrap();
    }
}
