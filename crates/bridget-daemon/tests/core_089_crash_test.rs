#![cfg(feature = "test-support")]

//! Matrice 012 portée dans le paquet 089 : SIGKILL aux barrières, mêmes octets.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{IdempotencyIssue, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[path = "support/idempotent.rs"]
pub mod fixture;
use fixture::*;

#[allow(dead_code)]
fn run_amont_cycle(point: &str, serial: usize) {
    let root = test_root(point);
    let sync = root.join("sync");
    private_dir(&sync).expect("répertoire de synchronisation");
    make_fifo(&sync.join(format!("{point}.fifo")));
    let marker = sync.join(format!("{point}.ready"));
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let _recipient = register_recipient(&socket_path);
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = format!("poste-essai-{point}-{serial}");
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let _recipient = register_recipient(&socket_path);
    let first = retry_issue(&socket_path, message_id.clone(), issued_at);
    let second = retry_issue(&socket_path, message_id, issued_at);
    assert_eq!(first, second, "une issue rejouée doit être stable");
    assert!(matches!(first, IdempotencyIssue::OutcomeUnknown { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage cycle");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "banc de gate SC-001 : cargo test --features test-support --test core_089_crash_test -- --ignored --test-threads=1"
)]
fn matrice_crash_sc001_redelivre_cinquante_prompts_uniques() {
    let root = test_root("sc001-matrix");
    let sync = root.join("sync");
    private_dir(&sync).expect("répertoire de synchronisation");
    let points = [
        "before_reservation",
        "after_prepared",
        "after_delivery_before_issue",
        "after_issue_before_client_ack",
    ];
    arm_checkpoint(&sync, &points, points[0]);

    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(MATRIX_CYCLES);
    let wrapper = WrapperProcess::start(&root, &registry, MATRIX_AGENT);
    wait_for_registered_agent(&socket_path, MATRIX_AGENT);
    let deadline = Instant::now() + GLOBAL_TIMEOUT;

    for serial in 0..MATRIX_CYCLES {
        eprintln!(
            "SC-001 cycle {serial}/{MATRIX_CYCLES}, racine {}",
            root.display()
        );
        assert!(
            Instant::now() < deadline,
            "matrice matrice de reprise dépassée après {GLOBAL_TIMEOUT:?}"
        );
        let point = points[serial % points.len()];
        let marker = sync.join(format!("{point}.ready"));
        let _ = fs::remove_file(&marker);
        let issued_at = issued_at();
        let message_id = format!("sc001-{serial}");
        let mut message = BridgetMessage::new(ACTOR, MATRIX_AGENT, "matrice SC-001");
        message.id = message_id.clone();
        let command = WrapperToDaemon::SendIdempotent {
            message,
            message_id,
            issued_at,
        };
        let mut client = negotiate_client(&socket_path);
        client.send(command.clone());
        watch_marker(&sync, &marker);
        daemon.crash();

        eprintln!("SC-001 {serial}: relève après SIGKILL à {point}");
        daemon.restart(&root);
        wait_for_registered_agent(&socket_path, MATRIX_AGENT);
        let first_replay = retry_command_issue(&socket_path, command.clone());
        let second_replay = retry_command_issue(&socket_path, command.clone());
        assert_eq!(first_replay, second_replay, "le rejeu en vol est stable");
        assert!(matches!(
            first_replay,
            IdempotencyIssue::OutcomeUnknown { .. }
        ));
        wait_for_counter(&counter, serial + 1);
        wait_for_accepted(&socket_path, &command);
        let first_terminal = retry_command_issue(&socket_path, command.clone());
        let second_terminal = retry_command_issue(&socket_path, command.clone());
        assert_eq!(
            first_terminal, second_terminal,
            "le rejeu terminal est stable"
        );
        assert!(matches!(first_terminal, IdempotencyIssue::Accepted { .. }));
        if serial + 1 < MATRIX_CYCLES {
            eprintln!("SC-001 {serial}: réarmement du jalon suivant");
            arm_checkpoint(&sync, &points, points[(serial + 1) % points.len()]);
            daemon.restart_with_sync(&root, &sync);
            wait_for_registered_agent(&socket_path, MATRIX_AGENT);
        }
    }

    assert_eq!(
        fs::read(&counter).expect("compteur ACP"),
        vec![b'x'; MATRIX_CYCLES],
        "chaque crash remet exactement un prompt, sans doublon"
    );
    daemon.stop();
    assert_eq!(wrapper.join(), Ok(()));
    fs::remove_dir_all(root).expect("nettoyage matrice");
    fs::remove_dir_all(registry_root).expect("nettoyage registre ACP");
}

#[test]
fn reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance() {
    let root = test_root("redelivery");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-stable");
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = "t1205bis-redelivery".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let first = receive_delivery(&mut recipient);
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-stable");
    let replayed = receive_delivery(&mut recipient);
    assert_eq!(
        encode(&first).expect("première remise encodable"),
        encode(&replayed).expect("remise reprise encodable"),
        "la reprise doit relire et rejouer les bytes persistés"
    );
    let (delivery_id, delivery_generation) = match replayed {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    let terminal = retry_issue(&socket_path, message_id, issued_at);
    assert!(matches!(terminal, IdempotencyIssue::Accepted { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage reprise");
}

#[test]
fn spec094_domaine_mcp_persiste_et_revient_par_la_vraie_reconnexion_wrapper() {
    fn observed_domain(socket: &std::path::Path) -> Option<String> {
        let mut observer = Client::connect(socket);
        observer.send(WrapperToDaemon::ListAgents);
        let agents = match observer.receive() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("annuaire attendu, reçu {other:?}"),
        };
        agents
            .into_iter()
            .find(|agent| agent.agent_id == ACP_AGENT)
            .and_then(|agent| agent.domain)
    }

    fn wait_for_domain(socket: &std::path::Path, expected: &str) {
        let deadline = Instant::now() + GLOBAL_TIMEOUT;
        loop {
            let observed = observed_domain(socket);
            if observed.as_deref() == Some(expected) {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "domaine attendu {expected:?}, dernier observé {observed:?}"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    let root = test_root("spec094-domain-reconnect");
    let daemon = spawn_daemon(&root, None);
    let (registry, registry_root, _counter) = registry_with_counting_acp_agent(1);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    let socket_path = socket(&root);
    wait_for_registered_agent(&socket_path, ACP_AGENT);
    let initial_domain = observed_domain(&socket_path).expect("domaine dérivé initial attesté");
    assert_ne!(initial_domain, "reconnexion-094");
    let instance_id = fs::read_dir(root.join("state/agent-names"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .find_map(|name| name.strip_prefix("instance-").map(str::to_string))
        .expect("instance du vrai wrapper");

    let mut mcp = McpProcess::start(&root, ACP_AGENT, &instance_id);
    assert!(
        mcp.request(serde_json::json!({
            "jsonrpc":"2.0", "id":1, "method":"initialize", "params":{}
        }))
        .get("error")
        .is_none()
    );
    mcp.notify(serde_json::json!({
        "jsonrpc":"2.0", "method":"notifications/initialized"
    }));
    let changed = mcp.request(serde_json::json!({
        "jsonrpc":"2.0", "id":2, "method":"tools/call", "params":{
            "name":"bridget_domain", "arguments":{"domain":"reconnexion-094"}
        }
    }));
    assert_eq!(changed["result"]["structuredContent"]["status"], "applied");
    assert_eq!(
        fs::read_to_string(root.join("state/agent-domains").join(ACP_AGENT)).unwrap(),
        "reconnexion-094"
    );
    mcp.stop();

    daemon.crash();
    let restarted = spawn_daemon(&root, None);
    wait_for_registered_agent(&socket_path, ACP_AGENT);
    // Register principal atteste le dérivé avant que la connexion auxiliaire
    // ait nécessairement reçu l'ACK de réapplication de l'override.
    wait_for_domain(&socket_path, "reconnexion-094");

    let mut resumed_mcp = McpProcess::start(&root, ACP_AGENT, &instance_id);
    assert!(
        resumed_mcp
            .request(serde_json::json!({
                "jsonrpc":"2.0", "id":3, "method":"initialize", "params":{}
            }))
            .get("error")
            .is_none()
    );
    resumed_mcp.notify(serde_json::json!({
        "jsonrpc":"2.0", "method":"notifications/initialized"
    }));
    let reset = resumed_mcp.request(serde_json::json!({
        "jsonrpc":"2.0", "id":4, "method":"tools/call", "params":{
            "name":"bridget_domain", "arguments":{"reset":true}
        }
    }));
    assert_eq!(reset["result"]["structuredContent"]["status"], "applied");
    assert!(!root.join("state/agent-domains").join(ACP_AGENT).exists());
    resumed_mcp.stop();
    wait_for_domain(&socket_path, &initial_domain);

    restarted.stop();
    assert_eq!(wrapper.join(), Ok(()));
    fs::remove_dir_all(root).expect("nettoyage reconnexion 094");
    fs::remove_dir_all(registry_root).expect("nettoyage registre 094");
}

#[test]
fn destination_remplacee_reste_indeterminee_sans_reroutage() {
    let root = test_root("destination-remplacee");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-original");
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let message_id = "t1205bis-destination-remplacee".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliveryIndeterminate {
        delivery_id,
        delivery_generation,
    });
    daemon.stop();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut replacement = register_recipient_as(&socket_path, "recipient-replacement");
    assert_no_delivery(&mut replacement);
    let issue = retry_issue(&socket_path, message_id, issued_at);
    assert!(matches!(issue, IdempotencyIssue::OutcomeUnknown { .. }));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage destination remplacée");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_terminal_acked_rejoue_accepted_apres_crash_daemon() {
    let root = test_root("recovery-terminal");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-terminal-instance");
    let issued_at = issued_at();
    let message_id = "recovery-terminal".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-terminal-instance");
    assert_no_delivery(&mut recipient);
    wait_for_accepted(&socket_path, &idempotent_send(message_id, issued_at));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage terminal");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_ack_d_une_reponse_liee_cloture_la_demande_atomiquement() {
    let root = test_root("linked-reply-ack");
    let database = root.join("state/bridget.db");
    private_dir(database.parent().expect("parent base")).expect("répertoire base");
    fixture_store(&database)
        .expect("store initial")
        .create_request("request-open", RECIPIENT, ACTOR, 60)
        .expect("demande suivie initiale");

    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "linked-reply-instance");
    let issued_at = issued_at();
    let mut command = idempotent_send("linked-reply-ack".to_string(), issued_at);
    let WrapperToDaemon::SendIdempotent { message, .. } = &mut command else {
        unreachable!("commande idempotente attendue");
    };
    message.in_reply_to = Some("request-open".to_string());
    let mut client = negotiate_client(&socket_path);
    client.send(command);
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let mut restarted = MatrixDaemonGuard::start(&root, &sync);
    let request = fixture_store(&database)
        .expect("store après redémarrage")
        .get_request("request-open")
        .expect("demande lisible")
        .expect("demande présente");
    assert_eq!(request.state, "answered");
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage réponse liée");
}

#[test]
fn outil_mcp_rejette_la_reponse_liee_divergente_sans_muter_les_demandes() {
    let root = test_root("mcp-linked-mismatch");
    let database = root.join("state/bridget.db");
    private_dir(database.parent().expect("parent base")).expect("répertoire base");
    let store = fixture_store(&database).expect("store initial");
    store
        .create_request("request-a", RECIPIENT, ACTOR, 60)
        .expect("demande A initiale");
    store
        .create_request("request-b", RECIPIENT, ACTOR, 60)
        .expect("demande B initiale");

    let sync = root.join("sync");
    private_dir(&sync).expect("synchronisation vide");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "mcp-linked-recipient");
    let _actor = register_agent_as(&socket_path, ACTOR, "mcp-linked-instance");
    let mut mcp = McpProcess::start(&root, ACTOR, "mcp-linked-instance");
    let initialize = mcp.request(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    }));
    assert_eq!(initialize["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    }));

    let issued_at = issued_at();
    let call = |id, in_reply_to| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": "bridget_send",
                "arguments": {
                    "to": RECIPIENT,
                    "body": "réponse MCP liée",
                    "in_reply_to": in_reply_to,
                    "id": "mcp-linked-retry",
                    "issued_at": issued_at
                }
            }
        })
    };
    // Le premier envoi est un DÉPÔT RÉUSSI, et ce banc le prouve bout en bout :
    // le destinataire reçoit sa remise dès la ligne suivante. L'annoncer
    // `outcome_unknown` gravait le défaut dans le contrat — c'est ce statut,
    // lu comme une perte, qui a fait diagnostiquer un canal cassé à un
    // relecteur et graver un constat bloquant faux au registre.
    let first = mcp.request(call(2, "request-a"));
    assert_eq!(first["result"]["structuredContent"]["status"], "in_flight");
    assert!(
        first["result"]["structuredContent"]["delivery_id"].is_string(),
        "un dépôt attesté doit publier la preuve qui le distingue d'un sort inconnu"
    );
    // Oracle (ii) : visible au ledger AVANT DeliverAcked, pendant dispatching.
    let store_before_ack = fixture_store(&database).expect("store avant ack");
    let before_ack = store_before_ack
        .recent_messages(20)
        .expect("ledger avant ack");
    let en_vol = before_ack
        .iter()
        .find(|entry| entry.id == "mcp-linked-retry")
        .expect("message visible avant ack");
    assert_eq!(
        en_vol.delivery_phase.as_deref(),
        Some("dispatching"),
        "phase en vol avant DeliverAcked"
    );
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });

    let accepted = mcp.request(call(3, "request-a"));
    assert_eq!(
        accepted["result"]["structuredContent"]["status"],
        "accepted"
    );
    let tool_ledger = mcp.request(serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "bridget_ledger",
            "arguments": { "view": "messages", "limit": 20 }
        }
    }));
    let tool_messages = tool_ledger["result"]["structuredContent"]["messages"]
        .as_array()
        .expect("projection MCP messages");
    let tool_entry = tool_messages
        .iter()
        .find(|entry| entry["id"] == "mcp-linked-retry")
        .expect("message au ledger MCP après ack");
    assert_eq!(
        tool_entry["delivery_status"], "recu",
        "après DeliverAcked le DTO doit exposer reçu, pas en_vol"
    );
    let cli_ledger = run_isolated(&root, &["ledger"], false);
    assert!(cli_ledger.status.success(), "ledger CLI: {cli_ledger:?}");
    let cli_output = String::from_utf8(cli_ledger.stdout).expect("ledger CLI UTF-8");
    assert!(
        cli_output.contains("réponse MCP liée"),
        "le renderer CLI doit exposer l'envoi MCP livré: {cli_output}"
    );
    assert!(
        cli_output.contains("[reçu]"),
        "le renderer CLI doit marquer le message accusé: {cli_output}"
    );
    assert_eq!(
        store
            .recent_messages(20)
            .expect("ledger persistant lisible")
            .iter()
            .filter(|entry| entry.id == "mcp-linked-retry")
            .count(),
        1,
        "le ledger persistant ne contient qu'une remise émise (visibilité ≠ accusé)"
    );
    let before_a = store
        .get_request("request-a")
        .expect("demande A lisible")
        .expect("demande A présente");
    let before_b = store
        .get_request("request-b")
        .expect("demande B lisible")
        .expect("demande B présente");
    assert_eq!(before_a.state, "answered");
    assert_eq!(before_b.state, "open");

    let mismatch = mcp.request(call(5, "request-b"));
    assert_eq!(
        mismatch["result"]["structuredContent"]["status"],
        "envelope_mismatch"
    );
    assert_eq!(
        store
            .get_request("request-a")
            .expect("demande A finale lisible")
            .expect("demande A finale présente"),
        before_a
    );
    assert_eq!(
        store
            .get_request("request-b")
            .expect("demande B finale lisible")
            .expect("demande B finale présente"),
        before_b
    );
    mcp.stop();
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).expect("nettoyage MCP réponse liée");
}

#[test]
fn binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee() {
    let root = test_root("cli-mcp-linked-parity");
    let database = root.join("state/bridget.db");
    private_dir(database.parent().expect("parent base")).expect("répertoire base");
    let store = fixture_store(&database).expect("store initial");
    store
        .create_request("request-open", RECIPIENT, ACTOR, 60)
        .expect("demande suivie initiale");

    let sync = root.join("sync");
    private_dir(&sync).expect("synchronisation vide");
    let mut daemon = MatrixDaemonGuard::start(&root, &sync);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recipient-parity-instance");
    let _actor = register_agent_as(&socket_path, ACTOR, "shared-cli-mcp-instance");

    let first = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--in-reply-to",
            "request-open",
            "réponse liée paritaire",
        ],
    );
    // Le premier envoi est nominalement « sort inconnu » : le daemon répond
    // avant l'accusé du destinataire. Il a pourtant PRIS la remise, donc le
    // dépôt a réussi et le code de sortie doit le dire. Ce banc attestait
    // l'inverse — c'était le défaut, gravé en contrat.
    assert!(
        first.status.success(),
        "une remise en vol est un dépôt réussi: {}",
        output_text(&first)
    );
    assert!(output_text(&first).contains("en vol"));
    assert!(
        !output_text(&first).contains("perdu"),
        "rien n'est perdu tant que la remise est en vol: {}",
        output_text(&first)
    );
    let message_id = output_field(&first, "id");
    let sent_at = output_field(&first, "issued_at")
        .parse::<i64>()
        .expect("issued_at CLI entier");
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            delivery_generation,
            ..
        } => {
            assert_eq!(message.id, message_id);
            assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
            (delivery_id, delivery_generation)
        }
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });

    let retry = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--in-reply-to",
            "request-open",
            "--id",
            &message_id,
            "--issued-at",
            &sent_at.to_string(),
            "réponse liée paritaire",
        ],
    );
    assert!(retry.status.success(), "retry CLI: {}", output_text(&retry));
    assert!(output_text(&retry).contains("accepted"));
    assert_no_delivery(&mut recipient);

    let mut mcp = McpProcess::start(&root, ACTOR, "shared-cli-mcp-instance");
    let initialized = mcp.request(serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
    }));
    assert_eq!(initialized["result"]["protocolVersion"], "2025-06-18");
    mcp.notify(serde_json::json!({
        "jsonrpc": "2.0", "method": "notifications/initialized"
    }));
    let mcp_retry = mcp.request(mcp_send_call(
        2,
        &message_id,
        sent_at,
        "réponse liée paritaire",
    ));
    assert_eq!(
        mcp_retry["result"]["structuredContent"]["status"],
        "accepted"
    );
    assert_no_delivery(&mut recipient);

    let duplicate_id = "cli-linked-new-id";
    let duplicate_at = issued_at();
    let duplicate = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--in-reply-to",
            "request-open",
            "--id",
            duplicate_id,
            "--issued-at",
            &duplicate_at.to_string(),
            "réponse liée paritaire",
        ],
    );
    assert!(!duplicate.status.success());
    assert!(
        output_text(&duplicate).contains("REJET: duplicate"),
        "projection CLI du doublon: {}",
        output_text(&duplicate)
    );
    assert_no_delivery(&mut recipient);

    recipient.send(WrapperToDaemon::Availability {
        agent: RECIPIENT.to_string(),
        until_secs: Some((issued_at() + 60) as u64),
    });
    assert!(matches!(recipient.receive(), DaemonToWrapper::Ack { .. }));
    let closed_at = issued_at();
    let closed = run_linked_cli(
        &root,
        &[
            "send",
            "--to",
            RECIPIENT,
            "--in-reply-to",
            "request-open",
            "--id",
            "cli-terminal-request",
            "--issued-at",
            &closed_at.to_string(),
            "message ordinaire après clôture",
        ],
    );
    assert!(!closed.status.success());
    assert!(
        output_text(&closed).contains("REJET: dnd"),
        "D-208 doit conserver DND côté CLI: {}",
        output_text(&closed)
    );
    let mcp_closed = mcp.request(mcp_send_call(
        3,
        "mcp-terminal-request",
        issued_at(),
        "autre message ordinaire après clôture",
    ));
    assert_eq!(mcp_closed["result"]["structuredContent"]["status"], "dnd");
    assert_no_delivery(&mut recipient);
    assert_eq!(
        store
            .get_request("request-open")
            .expect("demande lisible")
            .expect("demande présente")
            .state,
        "answered"
    );

    mcp.stop();
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).expect("nettoyage parité CLI MCP");
}

#[test]
fn recovery_acked_wrapper_finalise_accepted_apres_crash_daemon() {
    let root = test_root("recovery-acked-wrapper");
    let daemon = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-acked-wrapper-instance");
    let issued_at = issued_at();
    let message_id = "recovery-acked-wrapper".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let first = receive_delivery(&mut recipient);
    daemon.crash();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-acked-wrapper-instance");
    let redelivery = receive_delivery(&mut recipient);
    assert_eq!(
        encode(&first).expect("première remise encodable"),
        encode(&redelivery).expect("remise reprise encodable")
    );
    let (delivery_id, delivery_generation) = match redelivery {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage acked wrapper");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_seen_indeterminate_maintient_outcome_unknown_apres_crash_daemon() {
    let root = test_root("recovery-indeterminate");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_indeterminate");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-indeterminate-instance");
    let issued_at = issued_at();
    let message_id = "recovery-indeterminate".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    let _ = client.receive();
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliveryIndeterminate {
        delivery_id,
        delivery_generation,
    });
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-indeterminate-instance");
    assert_no_delivery(&mut recipient);
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::OutcomeUnknown { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage indéterminé");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_absent_redelivre_sans_doublon_apres_crash_daemon() {
    let root = test_root("recovery-absent");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_before_issue");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let recipient = register_recipient_as(&socket_path, "recovery-absent-instance");
    let issued_at = issued_at();
    let message_id = "recovery-absent".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-absent-instance");
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage absent");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_prepared_reprend_le_dispatch_apres_crash_daemon() {
    let root = test_root("recovery-prepared");
    let (sync, marker) = checkpoint_root(&root, "after_prepared");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    let recipient = register_recipient_as(&socket_path, "recovery-prepared-instance");
    let issued_at = issued_at();
    let message_id = "recovery-prepared".to_string();
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    watch_marker(&sync, &marker);
    daemon.crash();
    drop(client);
    drop(recipient);

    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    let mut recipient = register_recipient_as(&socket_path, "recovery-prepared-instance");
    let mut client = negotiate_client(&socket_path);
    client.send(idempotent_send(message_id.clone(), issued_at));
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    let (delivery_id, delivery_generation) = match receive_delivery(&mut recipient) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => (delivery_id, delivery_generation),
        _ => unreachable!(),
    };
    recipient.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    assert!(matches!(
        retry_issue(&socket_path, message_id, issued_at),
        IdempotencyIssue::Accepted { .. }
    ));
    restarted.stop();
    fs::remove_dir_all(root).expect("nettoyage prepared");
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "exige kqueue (Darwin/BSD) ; équivalent inotify hors périmètre SpecKit-030"
)]
fn recovery_terminal_acked_vrai_wrapper_rejoue_sans_second_prompt() {
    let root = test_root("acp-daemon-restart");
    let (sync, marker) = checkpoint_root(&root, "after_delivery_acked");
    let daemon = spawn_daemon(&root, Some(&sync));
    let socket_path = socket(&root);
    // Le fournisseur témoin reste en lecture après son premier prompt : sa
    // disparition volontaire ne doit pas remplacer la preuve de reconnexion.
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(2);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket_path, ACP_AGENT);
    let issued_at = issued_at();
    let message_id = "vrai-wrapper-acp".to_string();
    let mut client = negotiate_client(&socket_path);
    let mut message = BridgetMessage::new(ACTOR, ACP_AGENT, "frame réelle");
    message.id = message_id.clone();
    let command = WrapperToDaemon::SendIdempotent {
        message,
        message_id: message_id.clone(),
        issued_at,
    };
    client.send(command.clone());
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    wait_for_counter(&counter, 1);
    // L'accusé provient du wrapper réel. Le jalon est atteint après son
    // commit durable, mais avant que le daemon puisse poursuivre son cycle.
    watch_marker(&sync, &marker);

    daemon.crash();
    let restarted = spawn_daemon(&root, None);
    let socket_path = socket(&root);
    wait_for_registered_agent(&socket_path, ACP_AGENT);
    wait_for_accepted(&socket_path, &command);
    restarted.stop();
    assert_eq!(wrapper.join(), Ok(()));
    // Compteur final après fermeture et attente du VRAI wrapper : aucun prompt
    // tardif ne peut être caché par une fenêtre arbitraire de 250 ms.
    assert_eq!(fs::read(&counter).expect("compteur ACP final"), b"x");
    fs::remove_dir_all(root).expect("nettoyage daemon ACP");
    fs::remove_dir_all(registry_root).expect("nettoyage registre ACP");
}
