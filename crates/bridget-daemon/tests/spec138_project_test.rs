//! SPEC138 : contrôle réel sur le daemon et une socket entièrement isolés.
#![allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_core::{BridgetMessage, MessageIntent};
use bridget_daemon::execution_store::{ConditionalTransition, ExecutionStore};
use bridget_transport::protocol::*;
use std::path::Path;
use std::time::Duration;

fn same_frame(actual: DaemonToWrapper, expected: &DaemonToWrapper) {
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}
use support::*;

fn git_project(root: &Path, name: &str) -> std::path::PathBuf {
    let path = root.join(name);
    std::fs::create_dir(&path).unwrap();
    assert!(
        std::process::Command::new("/usr/bin/git")
            .args(["init", "--quiet"])
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    path
}

fn fact(client: &mut Client, root: &Path) {
    client.send(WrapperToDaemon::CommunicationProjectFact {
        root: root.to_string_lossy().into_owned(),
        source: CommunicationProjectSource::Git,
        host: "idempotency-isolated".into(),
        worktree_root: None,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        }
    ));
}

fn controller(socket: &Path, root: &Path) -> Client {
    let mut client = Client::connect(socket);
    client.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::RoleAccepted { .. }
    ));
    client.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: "scope138_control_namespace".into(),
        capabilities: vec![
            ClientCapability::ExecutionControlV1,
            ClientCapability::CommunicationProjectsV1,
            ClientCapability::Lookup,
        ],
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    fact(&mut client, root);
    client
}

fn registered_target(socket: &Path) -> Client {
    let mut client = Client::connect(socket);
    client.send(WrapperToDaemon::Register {
        identity_version: 2,
        agent_type: "fixture".into(),
        agent_id: RECIPIENT.into(),
        host: Some("idempotency-isolated".into()),
        transport: Some("acp".into()),
        channel: None.into(),
        mode: Some(PresenceMode::Acp),
        location: None,
        os: Some("test".into()),
        instance_id: Some("scope138-target".into()),
        domain: None,
        journal_available: None,
        turn_in_progress: false,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::Registered { .. }
    ));
    client
}

#[test]
fn spec138_real_daemon_steer_scope_and_durable_replay() {
    let root = test_root("spec138-control");
    let socket = socket(&root);
    let project_a = git_project(&root, "project-a");
    let project_b = git_project(&root, "project-b");
    let mut daemon = Some(spawn_daemon(&root, None));
    let mut target = registered_target(&socket);
    fact(&mut target, &project_b);
    let mut source = controller(&socket, &project_a);
    let store = ExecutionStore::open(&root.join("state/bridget.db")).unwrap();
    store
        .record_starting("submission138", "execution138", RECIPIENT, 10)
        .unwrap();
    assert!(matches!(
        store
            .transition_if_current(
                "execution138",
                "starting",
                0,
                1,
                "running",
                "provider_accepted",
                11
            )
            .unwrap(),
        ConditionalTransition::Applied(_)
    ));
    let mut message = BridgetMessage::new("operateur", RECIPIENT, "body exact / PRIVATE_PROMPT");
    message.intent = Some(MessageIntent::SteerCurrent);
    let mut command = ExecutionControlCommand {
        version: 1,
        command_id: "control138-denied".into(),
        execution_id: "execution138".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::SteerCurrent,
        message: Some(message),
    };
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    let denied = source.receive();
    assert!(
        matches!(&denied,DaemonToWrapper::ControlExecutionResult {outcome:ExecutionControlOutcome::Refused(ExecutionControlRefusal::CrossProjectReasonRequired),project_warnings,..} if project_warnings.is_empty())
    );
    assert_no_delivery(&mut target);
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    same_frame(source.receive(), &denied);
    command.command_id = "control138-invalid".into();
    command.message.as_mut().unwrap().cross_project_reason = Some("review\n".into());
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    let invalid = source.receive();
    assert!(matches!(
        &invalid,
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::Refused(
                ExecutionControlRefusal::InvalidCrossProjectReason
            ),
            ..
        }
    ));
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    same_frame(source.receive(), &invalid);
    assert_no_delivery(&mut target);
    command.command_id = "control138-admitted".into();
    target
        .reader
        .get_ref()
        .set_read_timeout(Some(CLIENT_READ_TIMEOUT))
        .unwrap();
    command.message.as_mut().unwrap().cross_project_reason = Some("  shared review  ".into());
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    let admitted = source.receive();
    assert!(
        matches!(&admitted,DaemonToWrapper::ControlExecutionResult {outcome:ExecutionControlOutcome::OutcomeUnknown,project_warnings,..} if project_warnings[0].code=="cross_project" && project_warnings[0].reason.as_deref()==Some("shared review"))
    );
    let DaemonToWrapper::ControlExecutionDispatch {
        issuer_scope,
        command: dispatched,
    } = target.receive()
    else {
        panic!("control delivery expected")
    };
    assert_eq!(
        dispatched.message.as_ref().unwrap().body,
        command.message.as_ref().unwrap().body
    );
    assert_eq!(
        dispatched
            .message
            .as_ref()
            .unwrap()
            .cross_project_reason
            .as_deref(),
        Some("shared review")
    );
    target.send(WrapperToDaemon::ControlExecutionReported {
        issuer_scope,
        command_id: command.command_id.clone(),
        execution_id: command.execution_id.clone(),
        accepted: true,
        refusal_reason: None,
    });
    let accepted = source.receive();
    assert!(
        matches!(&accepted,DaemonToWrapper::ControlExecutionResult {outcome:ExecutionControlOutcome::Accepted,project_warnings,..} if project_warnings.len()==1)
    );
    drop(source);
    drop(target);
    drop(store);
    let mut process = daemon.take().unwrap();
    signal_test_group(&mut process.child, libc::SIGTERM);
    wait_child(&mut process.child, Duration::from_secs(5));
    drop(process);
    daemon = Some(spawn_daemon(&root, None));
    let mut target = registered_target(&socket);
    // Projet volontairement devenu identique : le reçu accepté ne change pas.
    fact(&mut target, &project_a);
    let mut source = controller(&socket, &project_a);
    source.send(WrapperToDaemon::ControlExecution {
        command: command.clone(),
    });
    same_frame(source.receive(), &accepted);
    assert_no_delivery(&mut target);
    source.send(WrapperToDaemon::Lookup {
        operation_kind: "execution_control".into(),
        idempotency_key: command.command_id,
    });
    same_frame(source.receive(), &accepted);
    drop(source);
    drop(target);
    let mut process = daemon.take().unwrap();
    signal_test_group(&mut process.child, libc::SIGTERM);
    wait_child(&mut process.child, Duration::from_secs(5));
}
