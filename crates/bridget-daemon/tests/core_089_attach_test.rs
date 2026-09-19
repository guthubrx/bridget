#![cfg(feature = "test-support")]
//! Vrai wrapper et daemon : fichiers source → snapshot → flux live.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{AttachWindow, ConnectionRole};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use std::collections::BTreeMap;
use std::path::Path;

const SOURCE: &[u8] = include_bytes!("../../../fixtures/journal-source-unusual-v1.jsonl");

fn yesterday() -> String {
    // Date de l'hôte comme le journal, pas une horloge/fuseau de consommateur.
    let mut command = std::process::Command::new("/bin/date");
    #[cfg(target_os = "macos")]
    command.args(["-v-1d", "+%F"]);
    #[cfg(not(target_os = "macos"))]
    command.args(["-d", "yesterday", "+%F"]);
    let output = command.output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().into()
}

fn attach(root: &Path, from: u64) -> (Client, String) {
    let mut client = Client::connect(&socket(root));
    client.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Attach,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        client.send(WrapperToDaemon::Subscribe {
            agent: ACP_AGENT.into(),
            window: AttachWindow::Seq(from),
        });
        match client.receive() {
            DaemonToWrapper::Subscribed { subscription_id } => return (client, subscription_id),
            DaemonToWrapper::AttachRejected {
                reason: bridget_transport::protocol::AttachRefusal::JournalUnavailable,
                ..
            } => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "JournalReady jamais attesté"
                );
            }
            other => panic!("refus inattendu de la vraie attache : {other:?}"),
        }
    }
}

fn fragment(
    frame: DaemonToWrapper,
    subscription: &str,
    lines: &mut BTreeMap<u64, Vec<u8>>,
) -> Option<u64> {
    match frame {
        DaemonToWrapper::JournalFragment {
            subscription_id,
            seq,
            offset,
            final_fragment,
            bytes,
        } => {
            assert_eq!(subscription_id, subscription);
            let line = lines.entry(seq).or_default();
            assert_eq!(
                line.len() as u64,
                offset,
                "fragment dupliqué ou lacune masquée"
            );
            line.extend(bytes);
            final_fragment.then_some(seq)
        }
        other => panic!("fragment attendu : {other:?}"),
    }
}

#[test]
fn source_brute_et_rotation_traversent_snapshot_puis_live_et_curseur_perime_annonce_gap() {
    let root = test_root("089-attach");
    let directory = root.join("state/sessions").join(ACP_AGENT);
    private_dir(directory.parent().unwrap()).unwrap();
    private_dir(&directory).unwrap();
    let historical = directory.join(format!("{}.jsonl", yesterday()));
    private_write(&historical, SOURCE).unwrap();
    let daemon = spawn_daemon(&root, None);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(2);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let (mut view, subscription) = attach(&root, 0);
    let mut lines = BTreeMap::new();
    let mut caught_up = 0;
    loop {
        let frame = view.receive();
        if let DaemonToWrapper::SnapshotCaughtUp {
            subscription_id,
            through_seq,
        } = frame
        {
            assert_eq!(subscription_id, subscription);
            assert_eq!(through_seq, Some(5));
            caught_up += 1;
            break;
        }
        fragment(frame, &subscription, &mut lines);
    }
    // Oracle indépendant : espaces et champ inconnu du fichier source, pas
    // encode(decode()) comparé à lui-même. À cet instant le tour live n'existe pas.
    assert_eq!(lines.keys().copied().collect::<Vec<_>>(), [5]);
    // Le délimiteur JSONL n'appartient pas au payload du fragment historique.
    // Retirer ce SEUL LF, jamais trim()/une re-sérialisation des champs.
    assert_eq!(lines[&5], SOURCE.strip_suffix(b"\n").unwrap());
    assert_eq!(caught_up, 1);
    let mut message = BridgetMessage::new(ACTOR, ACP_AGENT, "tour après snapshot");
    message.id = "089-live-after-caught-up".into();
    let command = WrapperToDaemon::SendIdempotent {
        message,
        message_id: "089-live-after-caught-up".into(),
        issued_at: issued_at(),
    };
    let mut sender = negotiate_client(&socket(&root));
    sender.send(command.clone());
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::IdempotencyResult { .. }
    ));
    loop {
        // Un second SnapshotCaughtUp ou un Gap est ici une erreur, pas ignoré.
        if fragment(view.receive(), &subscription, &mut lines) == Some(8) {
            break;
        }
    }
    assert_eq!(lines.keys().copied().collect::<Vec<_>>(), [5, 6, 7, 8]);
    wait_for_accepted(&socket(&root), &command);
    assert_eq!(std::fs::read(&counter).unwrap(), b"x");
    let current = directory.join(format!(
        "{}.jsonl",
        bridget_transport::journal::current_host_date()
    ));
    for line in std::fs::read(current)
        .unwrap()
        .split_inclusive(|byte| *byte == b'\n')
    {
        let event: serde_json::Value = serde_json::from_slice(line).unwrap();
        let seq = event["seq"].as_u64().unwrap();
        assert_eq!(
            lines[&seq],
            line.strip_suffix(b"\n").unwrap(),
            "le relais ne réécrit pas le journal live"
        );
    }
    // Retrait d'UNE fixture de journal, simulant une rétention. Le curseur
    // explicite 5 a perdu son événement ; repartir à 6 en silence serait faux.
    std::fs::remove_file(&historical).unwrap();
    let (mut resumed, resumed_id) = attach(&root, 5);
    assert!(
        matches!(resumed.receive(), DaemonToWrapper::Gap { subscription_id, from_seq:5, to_seq:5, .. } if subscription_id == resumed_id),
        "curseur périmé : lacune obligatoire avant les nouveaux fragments"
    );
    drop(resumed);
    drop(view);
    drop(sender);
    daemon.stop();
    assert_eq!(wrapper.join(), Ok(()));
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(registry_root).unwrap();
}
