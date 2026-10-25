//! Administration mobile locale : rôle, capacité et UID avant tout accès privé.
use super::*;
use bridget_transport::protocol::{MobileAdministrationRequest, MobileAdministrationResult};

pub(super) fn handle(
    conn_id: &str,
    request: MobileAdministrationRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let root = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let valid = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Client)
            && !st.auxiliary_connections.contains(conn_id)
            && !st.conn_names.contains_key(conn_id)
            && st.peer_uids.get(conn_id).copied() == Some(unsafe { libc::geteuid() })
            && st.client_negotiations.get(conn_id).is_some_and(|n| {
                n.version == CLIENT_CONTRACT_VERSION
                    && n.capabilities.as_slice() == [ClientCapability::MobileAdministrationV1]
            });
        if !valid {
            return DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Error {
                    code: "human_principal_required".into(),
                },
            };
        }
        let Some(root) = st.db_path.parent().map(PathBuf::from) else {
            return DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Error {
                    code: "storage_unavailable".into(),
                },
            };
        };
        root
    };
    if let MobileAdministrationRequest::Updates { request } = request {
        use bridget_transport::protocol::UpdateRequest as Q;
        let result = (|| -> Result<serde_json::Value, &'static str> {
            match &request {
                Q::Prepare { token } => {
                    // L'état et le compteur sont observés sous le même verrou métier.
                    let st = state.lock().unwrap_or_else(|e| e.into_inner());
                    if !st
                        .delegation_store
                        .pending()
                        .map_err(|_| "update_activity_unknown")?
                        .is_empty()
                        || st
                            .delegation_store
                            .update_has_pending_admissions()
                            .map_err(|_| "update_activity_unknown")?
                        || st.fleet.active_count() != 0
                        || st.recovering
                        || !st.managed_spawns.is_empty()
                        || st.presences.values().any(|p| p.state == "busy")
                        || st.remote_result_reservation.is_some()
                        || !st.pending_replies.is_empty()
                        || !st.pending_post_response_controls.is_empty()
                        || !st.pending_worker_responses.is_empty()
                        || !st.pending_origin_proofs.is_empty()
                        || !st.pending_candidate_probes_v2.is_empty()
                        || !st.pending_attach_sends.is_empty()
                        || st.active_waits.load(Ordering::Acquire)!=0
                        || crate::user_input::has_active(st.store.connection(),unix_now_secs()).map_err(|_|"update_activity_unknown")?
                    {
                        return Err("update_missions_active_or_unconfirmed");
                    }
                    st.update_gate.prepare(token)
                }
                Q::Finish { token } => state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .update_gate
                    .finish(token),
                _ => crate::updates::policy(&root, &request),
            }
        })();
        return DaemonToWrapper::MobileAdministrationResult {
            result: match result {
                Ok(value) => MobileAdministrationResult::Updates { value },
                Err(code) => MobileAdministrationResult::Error { code: code.into() },
            },
        };
    }
    if let MobileAdministrationRequest::GrantUpdates { device_id } = request {
        let result = crate::updates::grant_device(&root, &device_id);
        return DaemonToWrapper::MobileAdministrationResult {
            result: match result {
                Ok(found) => MobileAdministrationResult::Updates {
                    value: serde_json::json!({"status":"ok","granted":found,"device_id":device_id}),
                },
                Err(e) => MobileAdministrationResult::Error {
                    code: e.into(),
                },
            },
        };
    }
    // Aucun appel réseau ni écriture du magasin mobile sous le mutex métier.
    let reload = matches!(
        &request,
        MobileAdministrationRequest::ConfigureNotifications { .. }
    );
    let result = crate::mobile::administer(&root, request, unix_now_secs());
    if reload
        && matches!(
            &result,
            MobileAdministrationResult::NotificationsConfigured { configured: true }
        )
    {
        let notifier = crate::mobile::notifications::Notifier::start(&root);
        state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .mobile_notifier = notifier;
    }
    DaemonToWrapper::MobileAdministrationResult { result }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mobile177_bare_and_agent_connections_cannot_administer_devices() {
        let root = std::env::temp_dir().join(format!("bridget177-admin-{}", Uuid::new_v4()));
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let config = DaemonConfig {
            socket_path: root.join("s"),
            db_path: root.join("b.db"),
            log_path: root.join("log"),
            ..DaemonConfig::default()
        };
        let (tx, _) = mpsc::channel();
        let state = Arc::new(Mutex::new(DaemonState::new(&config, tx).unwrap()));
        let request = MobileAdministrationRequest::Configure {
            public_url: "https://bridget.example.net".into(),
            listen_addr: "127.0.0.1:9467".into(),
        };
        {
            let mut st = state.lock().unwrap();
            st.connection_roles
                .insert("agent".into(), ConnectionRole::Client);
            st.client_negotiations.insert(
                "agent".into(),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: crate::communication::issuer_scope("bridget-mobile-admin"),
                    capabilities: vec![ClientCapability::MobileAdministrationV1],
                },
            );
            st.peer_uids
                .insert("agent".into(), unsafe { libc::geteuid() });
            st.conn_names
                .insert("agent".into(), Uuid::new_v4().to_string());
        }
        for id in ["bare", "agent"] {
            let response = handle(id, request.clone(), &state);
            assert!(
                matches!(response,DaemonToWrapper::MobileAdministrationResult {
                result:MobileAdministrationResult::Error {code}
            } if code=="human_principal_required")
            );
        }
        assert!(!root.join("mobile.json").exists());
        drop(state);
        let _ = std::fs::remove_dir_all(root);
    }
    #[test]
    fn mobile177_admin_negotiates_on_real_unix_socket_and_rejects_unknown_fields() {
        use crate::communication::client::DaemonConnection;
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let root = std::env::temp_dir()
            .canonicalize()
            .unwrap()
            .join(format!("bridget177-wire-{}", Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let config = DaemonConfig {
            socket_path: root.join("s"),
            db_path: root.join("b.db"),
            log_path: root.join("log"),
            ..DaemonConfig::default()
        };
        let (tx, _) = mpsc::channel();
        let state = Arc::new(Mutex::new(DaemonState::new(&config, tx).unwrap()));
        // Le lancement réel fixe umask 077 ; le test parallèle ne change pas l'umask global.
        for name in ["b.db", "b.db-shm", "b.db-wal"] {
            let path = root.join(name);
            if path.exists() {
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
            }
        }
        let listener = UnixListener::bind(&config.socket_path).unwrap();
        std::fs::set_permissions(&config.socket_path, std::fs::Permissions::from_mode(0o600))
            .unwrap();

        let shared = state.clone();
        let server = thread::spawn(move || {
            let (peer, _) = listener.accept().unwrap();
            handle_connection(peer, shared).unwrap();
        });
        let mut client = DaemonConnection::connect(&config.socket_path).unwrap();
        assert!(matches!(
            client
                .exchange(&WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                })
                .unwrap(),
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            }
        ));
        assert!(
            matches!(client.exchange(&WrapperToDaemon::ClientHello {contract_version:1,
            issuer_scope:crate::communication::issuer_scope("bridget-mobile-admin"),
            capabilities:vec![ClientCapability::MobileAdministrationV1]}).unwrap(),
            DaemonToWrapper::ClientWelcome {capabilities,..} if capabilities==[ClientCapability::MobileAdministrationV1])
        );
        let status = client
            .exchange(&WrapperToDaemon::MobileAdministration {
                request: MobileAdministrationRequest::Status,
            })
            .unwrap();
        assert!(matches!(
            status,
            DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Configuration {
                    configured: false,
                    ..
                }
            }
        ));
        assert!(!root.join("mobile-devices.db").exists());
        let applied = client
            .exchange(&WrapperToDaemon::MobileAdministration {
                request: MobileAdministrationRequest::Configure {
                    public_url: "https://bridget.example.net".into(),
                    listen_addr: "127.0.0.1:9467".into(),
                },
            })
            .unwrap();
        assert!(matches!(
            applied,
            DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Configuration {
                    configured: true,
                    ..
                }
            }
        ));
        assert_eq!(
            std::fs::metadata(root.join("mobile.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert!(
            serde_json::from_value::<MobileAdministrationRequest>(serde_json::json!({
                "action":"pair","scope":{},"actor":"human"
            }))
            .is_err()
        );
        drop(client);
        server.join().unwrap();
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn updates179_human_activation_blocks_delivery_and_sql_unknown_is_closed() {
        use bridget_transport::protocol::{
            IdentityCredential, MobileOperation, MobileOperationRequest, UpdateRequest as Q,
        };
        let root = std::env::temp_dir().join(format!("bridget179-gate-{}", Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let config = DaemonConfig {
            socket_path: root.join("s"),
            db_path: root.join("b.db"),
            log_path: root.join("log"),
            ..DaemonConfig::default()
        };
        let (tx, _) = mpsc::channel();
        let state = Arc::new(Mutex::new(DaemonState::new(&config, tx).unwrap()));
        {
            let mut st = state.lock().unwrap();
            st.connection_roles
                .insert("human".into(), ConnectionRole::Client);
            st.peer_uids
                .insert("human".into(), unsafe { libc::geteuid() });
            st.client_negotiations.insert(
                "human".into(),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: crate::communication::issuer_scope("bridget-mobile-admin"),
                    capabilities: vec![ClientCapability::MobileAdministrationV1],
                },
            );
        }
        let token = Uuid::new_v4().to_string();
        let prepare = || MobileAdministrationRequest::Updates {
            request: Q::Prepare {
                token: token.clone(),
            },
        };
        assert!(
            matches!(handle("agent",prepare(),&state),DaemonToWrapper::MobileAdministrationResult{result:MobileAdministrationResult::Error{code}} if code=="human_principal_required")
        );
        assert!(
            matches!(handle("human",prepare(),&state),DaemonToWrapper::MobileAdministrationResult{result:MobileAdministrationResult::Updates{value}} if value["status"]=="ok")
        );
        let delivery = || WrapperToDaemon::MobileOperation {
            request: MobileOperationRequest {
                credential: IdentityCredential::new("invalid".into()),
                operation: MobileOperation::Questions {
                    after: None,
                    limit: 10,
                },
            },
        };
        assert!(matches!(
            handle_wrapper_message("other", delivery(), &state),
            Some(DaemonToWrapper::UpdateInProgress)
        ));
        assert!(matches!(
            handle(
                "human",
                MobileAdministrationRequest::Updates {
                    request: Q::Finish { token }
                },
                &state
            ),
            DaemonToWrapper::MobileAdministrationResult {
                result: MobileAdministrationResult::Updates { .. }
            }
        ));
        assert!(!matches!(
            handle_wrapper_message("other", delivery(), &state),
            Some(DaemonToWrapper::UpdateInProgress)
        ));
        state
            .lock()
            .unwrap()
            .store
            .connection()
            .execute_batch("DROP TABLE auto_admission_receipts")
            .unwrap();
        assert!(
            matches!(handle("human",MobileAdministrationRequest::Updates{request:Q::Prepare{token:Uuid::new_v4().to_string()}},&state),DaemonToWrapper::MobileAdministrationResult{result:MobileAdministrationResult::Error{code}} if code=="update_activity_unknown")
        );
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }
}
