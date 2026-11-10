//! Fonctions des branches volumineuses de `handle_wrapper_message_with_peer` (programme 260, lot 5).
//! Chaque fonction reprend le corps de sa branche sans changement.
use super::*;

pub(super) fn on_gap(subscription_id: String, from_seq: u64, to_seq: u64, reason: Option<String>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let accepted = st
                    .attach_subscriptions
                    .get_mut(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .map(|subscription| {
                        if reason.as_deref() == Some("live_feed_overrun") {
                            subscription.caught_up = false;
                        }
                    })
                    .is_some();
                accepted
                    .then(|| st.attach_views.get(&subscription_id).cloned())
                    .flatten()
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::Gap {
                    subscription_id,
                    from_seq,
                    to_seq,
                    reason,
                });
            }
            None
        }

pub(super) fn on_relaunch_order(name: String, command_id: String, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            info!("cycle_vie action=relaunch phase=request name={name} command_id={command_id}");
            let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
            if st.decommissioning_names.contains(&name) {
                return Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::NotRelaunchable {
                        reason: "décommissionnement en cours".to_string(),
                    },
                });
            }
            let active_command = st
                .managed_spawns
                .values()
                .find(|spawn| spawn.lease.name == name)
                .map(|spawn| spawn.lease.command_id.clone());
            if active_command
                .as_deref()
                .is_some_and(|active| active != command_id)
                || st.router.get_agent(&name).is_some()
            {
                return Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::AlreadyRunning,
                });
            }
            let Some(entry) = st.fleet.desired_entry(&name).ok().flatten() else {
                return Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: if st.router.get_agent(&name).is_some() {
                        RelaunchOutcome::NotManaged
                    } else {
                        RelaunchOutcome::NotFound
                    },
                });
            };
            match entry.lifecycle_state {
                DesiredLifecycleState::Running => {
                    return Some(DaemonToWrapper::RelaunchResult {
                        command_id: command_id.clone(),
                        outcome: if entry.command_id == command_id {
                            RelaunchOutcome::Started {
                                agent_id: name,
                                generation: entry.generation,
                            }
                        } else {
                            RelaunchOutcome::AlreadyRunning
                        },
                    });
                }
                DesiredLifecycleState::Decommissioned => {
                    return Some(DaemonToWrapper::RelaunchResult {
                        command_id,
                        outcome: RelaunchOutcome::NotRelaunchable {
                            reason: "agent décommissionné".to_string(),
                        },
                    });
                }
                DesiredLifecycleState::Stopped => {}
            }
            if entry.project.is_some() || entry.runtime_execution.is_some() {
                let project_id = entry
                    .project
                    .as_ref()
                    .map(|p| p.project_id.clone())
                    .or_else(|| {
                        entry
                            .runtime_execution
                            .as_ref()
                            .map(|e| e.project_id.clone())
                    })
                    .unwrap_or_default();
                return Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::Rejected {
                        reason: SpawnRefusal::DockerRuntimeUnavailable { project_id },
                    },
                });
            }
            let Some(resolved_definition) = entry.resolved_definition.as_ref() else {
                return Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::NotRelaunchable {
                        reason: "définition runtime figée absente".to_string(),
                    },
                });
            };
            let now = unix_timestamp();
            let order = FleetSpawnOrder {
                remote_placement: entry.remote_placement.clone(),
                posture: None,
                agent_type: entry.agent_type.clone(),
                requested_name: Some(name.clone()),
                cwd: entry.cwd.clone(),
                persistent: entry.persistent,
                project: entry.project.clone(),
                command_id: command_id.clone(),
                issued_at: now,
                deadline_at: now.saturating_add(MANAGED_RECOVERY_DEADLINE_SECS),
                ownership: None,
            };
            let decision = submit_relaunch_from_resolved(
                &st.fleet,
                &st.source_env,
                &order,
                now,
                resolved_definition,
                &crate::lifecycle::SpawnHosts::local(),
            );
            match decision {
                Ok(SpawnDecision::Ready(prepared)) => {
                    let stop = Arc::new(ManagedStopControl::new());
                    st.managed_by_instance.insert(
                        prepared.lease.instance_id.clone(),
                        prepared.lease.command_id.clone(),
                    );
                    st.relaunch_requesters
                        .entry(prepared.lease.command_id.clone())
                        .or_default()
                        .push(conn_id.to_string());
                    st.managed_spawns.insert(
                        prepared.lease.command_id.clone(),
                        ManagedSpawnRecord {
                            lease: prepared.lease.clone(),
                            agent_type: prepared.agent_type.clone(),
                            requester_conns: Vec::new(),
                            wrapper_conn: None,
                            stop: Arc::clone(&stop),
                        },
                    );
                    if st
                        .managed_tx
                        .send(ManagedSupervisorCommand::Start {
                            prepared: prepared.clone(),
                            stop,
                        })
                        .is_err()
                    {
                        let _ = st.fleet.fail(
                            &prepared.lease,
                            "negotiation_failed",
                            "superviseur de processus indisponible",
                        );
                        st.relaunch_requesters.remove(&prepared.lease.command_id);
                        st.managed_spawns.remove(&prepared.lease.command_id);
                        st.managed_by_instance.remove(&prepared.lease.instance_id);
                        return Some(DaemonToWrapper::RelaunchResult {
                            command_id,
                            outcome: RelaunchOutcome::Rejected {
                                reason: SpawnRefusal::NegotiationFailed {
                                    detail: "superviseur de processus indisponible".to_string(),
                                },
                            },
                        });
                    }
                    None
                }
                Ok(SpawnDecision::Await(waiter)) => {
                    let requesters = st.relaunch_requesters.entry(waiter.command_id).or_default();
                    if !requesters.iter().any(|requester| requester == conn_id) {
                        requesters.push(conn_id.to_string());
                    }
                    None
                }
                Ok(SpawnDecision::Accepted { name, .. }) => {
                    let generation = st
                        .fleet
                        .desired_entry(&name)
                        .ok()
                        .flatten()
                        .map(|entry| entry.generation)
                        .unwrap_or_default();
                    Some(DaemonToWrapper::RelaunchResult {
                        command_id,
                        outcome: RelaunchOutcome::Started {
                            agent_id: name,
                            generation,
                        },
                    })
                }
                Ok(SpawnDecision::Rejected(reason)) => Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::Rejected { reason },
                }),
                Ok(SpawnDecision::EnvelopeMismatch) => Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::NotRelaunchable {
                        reason: "command_id rejoué avec une enveloppe différente".to_string(),
                    },
                }),
                Err(error) => Some(DaemonToWrapper::RelaunchResult {
                    command_id,
                    outcome: RelaunchOutcome::Rejected {
                        reason: SpawnRefusal::NegotiationFailed {
                            detail: error.to_string(),
                        },
                    },
                }),
            }
        }

pub(super) fn on_cancel_request(id: String, sender: String, reason: Option<String>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            // Le champ sender désigne la demande, pas une autorité. La route
            // appartient au wrapper lui-même, ou à son client auxiliaire déjà
            // attesté par le même helper que rename et les contenus privés.
            let authorized = st
                .router
                .get_agent(&sender)
                .is_some_and(|route| route.connection_id == conn_id)
                || live_connection_identity(&st, conn_id)
                    .is_some_and(|(agent_id, _)| agent_id == sender);
            if !authorized {
                return Some(DaemonToWrapper::Nack {
                    id,
                    reason: "annulation réservée à la connexion de l'émetteur ou à son client auxiliaire attesté".to_string(),
                });
            }
            let pending_request = match st.store.get_request(&id) {
                Ok(request) => request,
                Err(error) => {
                    return Some(DaemonToWrapper::Nack {
                        id,
                        reason: error.to_string(),
                    });
                }
            };
            let guidance_admission = pending_request
                .as_ref()
                .filter(|request| request.state == "open")
                .and_then(|request| {
                    let project = crate::communication::project_scope(
                        connection_communication_project(&st, conn_id).as_ref(),
                        agent_communication_project(&st, &request.target).as_ref(),
                        &request.target,
                        None,
                    );
                    if let Err(reason) = project {
                        let code = if reason.starts_with("cross_project_reason_required") {
                            "cross_project_reason_required"
                        } else {
                            "invalid_cross_project_reason"
                        };
                        record_guidance_refusal(
                            &st,
                            Some(conn_id),
                            &request.target,
                            &id,
                            Some(crate::guidance::GuidanceAction::Interrupt),
                            crate::guidance::GuidancePath::Cancel,
                            code,
                        );
                        return None;
                    }
                    check_guidance(
                        &st,
                        Some(conn_id),
                        &request.target,
                        crate::guidance::GuidanceAction::Interrupt,
                        crate::guidance::GuidancePath::Cancel,
                        None,
                        false,
                        &id,
                    )
                    .ok()
                });
            match st.store.cancel_request(&id, &sender, reason.as_deref()) {
                Ok(Some(request)) if request.sender != sender => {
                    if let Some(facts) = &guidance_admission
                        && let Err(error) = record_guidance_outcome(
                            &st,
                            facts,
                            &request.target,
                            &id,
                            "refused:not_permitted",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    Some(DaemonToWrapper::Nack {
                        id,
                        reason: "seul l'émetteur peut annuler cette demande".to_string(),
                    })
                }
                Ok(Some(request)) if request.state == "cancelled" => {
                    let interrupt_active = guidance_admission.as_ref().is_some_and(|facts| {
                        match record_guidance_outcome(&st, facts, &request.target, &id, "admitted")
                        {
                            Ok(()) => true,
                            Err(error) => {
                                if let Err(error) = record_guidance_outcome(
                                    &st,
                                    facts,
                                    &request.target,
                                    &id,
                                    "refused:persistence_failed",
                                ) {
                                    warn!("refus de guidage non journalisé : {error}");
                                }
                                warn!("interruption d'annulation non persistée : {error}");
                                false
                            }
                        }
                    });
                    st.delegation_store.wait_epoch().bump();
                    st.pending_replies
                        .retain(|pending| pending.msg_id != request.id);
                    let notification = st.router.get_agent(&request.target)
                        .and_then(|agent| st.connections.get(&agent.connection_id).map(|writer| {
                            let has_structured_cancel = st.conn_instances.get(&agent.connection_id)
                                .and_then(|instance| st.presences.get(instance))
                                .is_some_and(|presence| presence.mode == Some(PresenceMode::Acp)
                                    || presence.transport == "t3code");
                            let message = if has_structured_cancel {
                                DaemonToWrapper::CancelDelivery {
                                    id: request.id.clone(),
                                    reason: request.cancel_reason.clone()
                                        .unwrap_or_else(|| "demande annulée".to_string()),
                                    interrupt_active: Some(interrupt_active),
                                }
                            } else {
                                DaemonToWrapper::Deliver { message: bridget_core::BridgetMessage::new(
                                    "bridget", &request.target,
                                    format!("Demande #{} annulée par {}. Aucune réponse n'est requise.{}",
                                        request.id, sender, request.cancel_reason.as_deref()
                                            .map(|reason| format!(" Motif : {reason}")).unwrap_or_default()),
                                ), mid_turn_ok: Some(true) }
                            };
                            (Arc::clone(writer), message)
                        }));
                    // Commit et pending sont terminés. Aucun accès au writer,
                    // même son verrou, ne peut désormais retenir l'état global.
                    drop(st);
                    if let Some((writer, message)) = notification
                        && let Err(error) = push_control_message_until(
                            &writer,
                            &message,
                            Instant::now() + CANCEL_NOTIFICATION_BUDGET,
                        )
                    {
                        error!(
                            "Annulation durable {}, notification à {} indisponible: {}",
                            request.id, request.target, error
                        );
                    }
                    Some(DaemonToWrapper::RequestCancelled {
                        id: request.id,
                        state: request.state,
                    })
                }
                Ok(Some(request)) => {
                    if let Some(facts) = &guidance_admission
                        && let Err(error) = record_guidance_outcome(
                            &st,
                            facts,
                            &request.target,
                            &id,
                            "refused:terminal_request",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    Some(DaemonToWrapper::Nack {
                        id,
                        reason: format!("demande déjà terminale: {}", request.state),
                    })
                }
                Ok(None) => {
                    if let (Some(facts), Some(request)) = (&guidance_admission, &pending_request)
                        && let Err(error) = record_guidance_outcome(
                            &st,
                            facts,
                            &request.target,
                            &id,
                            "refused:request_unavailable",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    Some(DaemonToWrapper::Nack {
                        id,
                        reason: "demande introuvable".to_string(),
                    })
                }
                Err(error) => {
                    if let (Some(facts), Some(request)) = (&guidance_admission, &pending_request)
                        && let Err(error) = record_guidance_outcome(
                            &st,
                            facts,
                            &request.target,
                            &id,
                            "refused:persistence_failed",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    Some(DaemonToWrapper::Nack {
                        id,
                        reason: error.to_string(),
                    })
                }
            }
        }

pub(super) fn on_service_hello(version: u16, service: String, issuer_scope: String, capabilities: Vec<ServiceCapability>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![SERVICE_CONTRACT_VERSION],
                    },
                });
            }
            if service != "guichet" {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::ReservedServiceRequired,
                });
            }
            if crate::idempotency::validate_issuer_scope(&issuer_scope).is_err() {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidIssuerScope,
                });
            }
            let canonical_capabilities = matches!(
                capabilities.as_slice(),
                [] | [ServiceCapability::GuichetV1]
                    | [ServiceCapability::HumanInboxV1]
                    | [ServiceCapability::ProjectRegistryV1]
                    | [
                        ServiceCapability::ProjectRegistryV1,
                        ServiceCapability::ProjectProfilesV1
                    ]
                    | [ServiceCapability::ProjectProfilesV1]
                    | [
                        ServiceCapability::GuichetV1,
                        ServiceCapability::ProjectRegistryV1,
                    ]
                    | [
                        ServiceCapability::GuichetV1,
                        ServiceCapability::CoordinationEventsV1,
                    ]
                    | [
                        ServiceCapability::GuichetV1,
                        ServiceCapability::CoordinationEventsV1,
                        ServiceCapability::ProjectRegistryV1,
                    ]
                    | [
                        ServiceCapability::GuichetV1,
                        ServiceCapability::CoordinationEventsV2,
                    ]
                    | [
                        ServiceCapability::GuichetV1,
                        ServiceCapability::CoordinationEventsV2,
                        ServiceCapability::ProjectRegistryV1,
                    ]
            );
            if !canonical_capabilities {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.service_negotiations.insert(
                conn_id.to_string(),
                NegotiatedService {
                    version: SERVICE_CONTRACT_VERSION,
                    capabilities: capabilities.clone(),
                },
            );
            if capabilities.contains(&ServiceCapability::GuichetV1) {
                match st.store.guichet_lifecycle_events() {
                    Ok(events) => {
                        let controls = events
                            .into_iter()
                            .filter_map(|event| {
                                st.connections.get(conn_id).map(|writer| DeferredControl {
                                    writer: writer.clone(),
                                    message: guichet_lifecycle_response(event),
                                })
                            })
                            .collect::<Vec<_>>();
                        if !controls.is_empty() {
                            st.pending_post_response_controls
                                .insert(conn_id.to_string(), controls);
                        }
                    }
                    Err(error) => error!("lecture des événements guichet: {error}"),
                }
            }
            // La v1 conserve son rejeu initial historique. La v2 ne pousse
            // rien avant `coordination_subscribe`, sinon un fait live pourrait
            // être confondu avec une observation fraîche avant le snapshot.
            if capabilities.contains(&ServiceCapability::CoordinationEventsV1) {
                match st.store.guichet_coordination_events() {
                    Ok(events) => {
                        let controls = events
                            .into_iter()
                            .filter_map(|event| {
                                st.connections.get(conn_id).map(|writer| DeferredControl {
                                    writer: writer.clone(),
                                    message: guichet_coordination_response(
                                        event,
                                        COORDINATION_EVENTS_VERSION,
                                    ),
                                })
                            })
                            .collect::<Vec<_>>();
                        if !controls.is_empty() {
                            st.pending_post_response_controls
                                .entry(conn_id.to_string())
                                .or_default()
                                .extend(controls);
                        }
                    }
                    Err(error) => error!("lecture des événements de coordination: {error}"),
                }
            }
            Some(DaemonToWrapper::ServiceWelcome {
                version: SERVICE_CONTRACT_VERSION,
                horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
                issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                capabilities,
            })
        }

pub(super) fn on_control_execution_reported(issuer_scope: String, command_id: String, execution_id: String, accepted: bool, refusal_reason: Option<String>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let (controls, response) = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                let Some(target) = (match st.execution_store.execution_control_target(&execution_id)
                {
                    Ok(target) => target,
                    Err(error) => {
                        error!("lecture cible accusé de contrôle: {error}");
                        return Some(DaemonToWrapper::Nack {
                            id: command_id,
                            reason: "cible de contrôle introuvable".to_string(),
                        });
                    }
                }) else {
                    return Some(DaemonToWrapper::Nack {
                        id: command_id,
                        reason: "cible de contrôle introuvable".to_string(),
                    });
                };
                let is_target_wrapper = st
                    .router
                    .get_agent(&target.target_agent)
                    .is_some_and(|agent| agent.connection_id == conn_id);
                if !is_target_wrapper {
                    warn!(
                        "accusé de contrôle refusé: wrapper non propriétaire execution={} conn={}",
                        execution_id, conn_id
                    );
                    return Some(DaemonToWrapper::Nack {
                        id: command_id,
                        reason: "wrapper non propriétaire de l'exécution".to_string(),
                    });
                }
                let stored = match st.execution_store.lookup_control_command(
                    &issuer_scope,
                    &command_id,
                    unix_timestamp(),
                ) {
                    Ok(Some(stored)) if stored.execution_id == execution_id => stored,
                    Ok(_) => {
                        return Some(DaemonToWrapper::Nack {
                            id: command_id,
                            reason: "commande de contrôle inconnue ou expirée".to_string(),
                        });
                    }
                    Err(error) => {
                        error!("lecture commande de contrôle: {error}");
                        return Some(DaemonToWrapper::Nack {
                            id: command_id,
                            reason: "lecture de commande de contrôle impossible".to_string(),
                        });
                    }
                };
                if !matches!(stored.status, ControlCommandStatus::Dispatched) {
                    return Some(DaemonToWrapper::Nack {
                        id: command_id,
                        reason: "commande de contrôle déjà résolue".to_string(),
                    });
                }
                let refusal_reason = (!accepted)
                    .then(|| refusal_reason.unwrap_or_else(|| "invalid_command".to_string()));
                let now = unix_timestamp();
                if !st
                    .execution_store
                    .resolve_control_command(
                        &issuer_scope,
                        &command_id,
                        accepted,
                        refusal_reason.as_deref(),
                        now,
                    )
                    .unwrap_or(false)
                {
                    return Some(DaemonToWrapper::Nack {
                        id: command_id,
                        reason: "résolution concurrente de commande de contrôle".to_string(),
                    });
                }
                let outcome = if accepted {
                    ExecutionControlOutcome::Accepted
                } else {
                    ExecutionControlOutcome::Refused(control_refusal_from_reason(
                        refusal_reason.as_deref().unwrap_or("invalid_command"),
                    ))
                };
                let mut controls = Vec::new();
                for (client_conn, negotiation) in &st.client_negotiations {
                    if negotiation.issuer_scope == issuer_scope {
                        defer_control(
                            &st,
                            client_conn,
                            DaemonToWrapper::ControlExecutionResult {
                                command_id: command_id.clone(),
                                execution_id: execution_id.clone(),
                                outcome: outcome.clone(),
                                project_warnings: stored.project_warnings.clone(),
                            },
                            &mut controls,
                        );
                    }
                }
                (controls, None)
            };
            let _ = execute_controls(controls);
            response
        }
