use super::*;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;

fn attested_response(name: &'static str, disconnect: bool) {
    let wait = name == "bridget_task_wait";
    let mutation = matches!(name, "bridget_task_cancel" | "bridget_task_ack");
    let socket = crate::mcp_identity::mock_socket("172-attested-deadline");
    let listener = UnixListener::bind(&socket).unwrap();
    crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
    let backend = std::thread::spawn(move || {
        if wait || name == "bridget_task_ack" {
            let (mut stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            writeln!(stream, "{}", json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["wait_v1"]})).unwrap();
        }
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&line).unwrap()["type"],
            "RegisterAuxiliary"
        );
        writeln!(
            stream,
            "{}",
            json!({"type":"Registered","agent_id":"owner","credential":null})
        )
        .unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(request["type"], "NativeDelegationT3");
        assert_eq!(
            request["request"]["operation"],
            name.strip_prefix("bridget_task_").unwrap()
        );
        // Trois échanges HTTP peuvent dépasser 10 s sans dépasser leur budget individuel.
        if !disconnect {
            std::thread::sleep(Duration::from_secs(12));
        }
        let result = if wait {
            json!({"status":"refused","code":"permission_attestation_unavailable"})
        } else {
            json!({"task_id":"task","status":"result_available","result":"résultat".repeat(2048)})
        };
        let sent = writeln!(
            stream,
            "{}",
            if disconnect {
                json!({"type":"Disconnect"})
            } else {
                json!({"type":"NativeDelegationResult","result":result})
            }
        );
        line.clear();
        let remaining = reader.read_line(&mut line).unwrap();
        assert_eq!(
            remaining, 0,
            "aucun rejeu, changement de clé ou deuxième lancement"
        );
        sent.is_ok()
    });
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let proof = bridget_transport::protocol::NativePermissionProof {
        endpoint: "http://127.0.0.1:12345/mcp".into(),
        authorization: bridget_transport::protocol::IdentityCredential::new(format!(
            "Bearer {}",
            "a".repeat(43)
        )),
    };
    let result = execute_with_proof(
        &identity,
        name,
        &if wait {
            json!({"task_ids":["task"],"wait_timeout":1})
        } else if name == "bridget_task_ack" {
            json!({"task_ids":["task"]})
        } else {
            json!({"task_id":"task"})
        },
        &socket,
        Some(proof),
    );
    let sent = backend.join().unwrap();
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    if disconnect {
        let expected = if mutation {
            "outcome_unknown"
        } else {
            "daemon_unreachable"
        };
        assert!(
            matches!(result, Err(ClientError::Technical { code, message })
                if code == expected && message.contains("bridget_task_status") && message.contains("même task_id")
                    && message.contains("ne créez pas une autre demande"))
        );
    } else if wait {
        assert!(
            matches!(result, Err(ClientError::Technical { code:"native_delegation_refused", message }) if message=="permission_attestation_unavailable"),
            "le refus doit rester explicite après la réattestation"
        );
    } else {
        let result = result.expect("le client ne doit pas expirer avant la réattestation");
        assert_eq!(result["task_id"], "task");
        assert_eq!(
            result["result"].as_str().unwrap().len(),
            "résultat".repeat(2048).len()
        );
    }
    assert!(sent, "la réponse complète doit être consommée");
}

#[test]
fn spec172_status_atteste_attend_la_reponse_complete() {
    attested_response("bridget_task_status", false);
}

#[test]
fn spec172_wait_atteste_preserve_le_refus_tardif() {
    attested_response("bridget_task_wait", false);
}

#[test]
fn spec174_disconnect_attente_et_statut_sans_rejeu() {
    attested_response("bridget_task_wait", true);
    attested_response("bridget_task_status", true);
}

#[test]
fn spec174_disconnect_mutations_sans_rejeu() {
    attested_response("bridget_task_cancel", true);
    attested_response("bridget_task_ack", true);
}

#[test]
fn spec172_reprise_attestee_garde_la_cle_et_le_budget() {
    let socket = crate::mcp_identity::mock_socket("172-attested-recovery");
    let listener = UnixListener::bind(&socket).unwrap();
    crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
    let backend = std::thread::spawn(move || {
        let mut first = None;
        for index in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(15)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "RegisterAuxiliary"
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["type"], "NativeDelegationT3");
            assert_eq!(request["request"]["request_id"], "same-key-172");
            if index == 0 {
                first = Some(request);
            } else {
                assert_eq!(
                    Some(request),
                    first,
                    "même enveloppe, même preuve, même clé"
                );
            }
            if index == 1 {
                std::thread::sleep(Duration::from_secs(12));
            }
            let result = if index == 0 {
                json!({"status":"refused","code":"admission_pending"})
            } else {
                json!({"task_id":"same-task","status":"starting"})
            };
            let _ = writeln!(
                stream,
                "{}",
                json!({"type":"NativeDelegationResult","result":result})
            );
        }
    });
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let proof = bridget_transport::protocol::NativePermissionProof {
        endpoint: "http://127.0.0.1:12345/mcp".into(),
        authorization: bridget_transport::protocol::IdentityCredential::new(format!(
            "Bearer {}",
            "a".repeat(43)
        )),
    };
    let result = execute_with_proof(
        &identity,
        "bridget_delegate",
        &json!({"request_id":"same-key-172","agent_type":"codex","model":"fixture","task":"mission","cwd":"/tmp"}),
        &socket,
        Some(proof),
    );
    backend.join().unwrap();
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    assert_eq!(
        result.expect("la reprise doit aussi couvrir la réattestation")["task_id"],
        "same-task"
    );
}

#[test]
fn spec172_sonde_admission_en_panne_reste_transport() {
    let socket = crate::mcp_identity::mock_socket("172-admission-probe");
    let listener = UnixListener::bind(&socket).unwrap();
    crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
    let backend = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        writeln!(
            stream,
            "{}",
            json!({"type":"Registered","agent_id":"owner","credential":null})
        )
        .unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&line).unwrap()["type"],
            "DaemonIdentityRequest"
        );
        // Fermer sans annonce de capacité, avant toute demande de lancement.
    });
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let result = execute_with_proof(
        &identity,
        "bridget_auto_delegate",
        &json!({"request_id":"not-admitted","task":"mission","cwd":"/tmp"}),
        &socket,
        None,
    );
    backend.join().unwrap();
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    assert!(matches!(
        result,
        Err(ClientError::Technical {
            code: "daemon_unreachable",
            ..
        })
    ));
}

#[test]
fn spec172_sonde_en_panne_ne_devient_pas_unsupported() {
    let socket = crate::mcp_identity::mock_socket("172-missing-probe");
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for (name, args) in [
        ("bridget_task_wait", json!({"task_ids":["task"]})),
        ("bridget_task_ack", json!({"task_ids":["task"]})),
    ] {
        let result = execute_with_proof(&identity, name, &args, &socket, None);
        assert!(
            matches!(
                result,
                Err(ClientError::Technical {
                    code: "daemon_unreachable",
                    ..
                })
            ),
            "une panne de transport ne prouve pas l'absence de capacité"
        );
    }
}
