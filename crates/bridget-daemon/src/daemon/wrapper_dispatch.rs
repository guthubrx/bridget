//! Fonctions des branches volumineuses de `handle_wrapper_message_with_peer` (programme 260, lot 5).
//! Chaque fonction reprend le corps de sa branche sans changement.
use super::*;
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::*;

pub(super) fn on_gap(
    subscription_id: String,
    from_seq: u64,
    to_seq: u64,
    reason: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let view = {
        let mut st = state.lock().recover_poison();
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

pub(super) fn on_relaunch_order(
    name: String,
    command_id: String,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    info!("cycle_vie action=relaunch phase=request name={name} command_id={command_id}");
    let mut st = state.lock().recover_poison();
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

pub(super) fn on_cancel_request(
    id: String,
    sender: String,
    reason: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    // Le champ sender désigne la demande, pas une autorité. La route
    // appartient au wrapper lui-même, ou à son client auxiliaire déjà
    // attesté par le même helper que rename et les contenus privés.
    let authorized = st
        .router
        .get_agent(&sender)
        .is_some_and(|route| route.connection_id == conn_id)
        || live_connection_identity(&st, conn_id).is_some_and(|(agent_id, _)| agent_id == sender);
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
                let code = if reason.starts_with(CROSS_PROJECT_REASON_REQUIRED) {
                    CROSS_PROJECT_REASON_REQUIRED
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
                match record_guidance_outcome(&st, facts, &request.target, &id, "admitted") {
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
            let notification = st.router.get_agent(&request.target).and_then(|agent| {
                st.connections.get(&agent.connection_id).map(|writer| {
                    let has_structured_cancel = st
                        .conn_instances
                        .get(&agent.connection_id)
                        .and_then(|instance| st.presences.get(instance))
                        .is_some_and(|presence| {
                            presence.mode == Some(PresenceMode::Acp)
                                || presence.transport == "t3code"
                        });
                    let message = if has_structured_cancel {
                        DaemonToWrapper::CancelDelivery {
                            id: request.id.clone(),
                            reason: request
                                .cancel_reason
                                .clone()
                                .unwrap_or_else(|| "demande annulée".to_string()),
                            interrupt_active: Some(interrupt_active),
                        }
                    } else {
                        DaemonToWrapper::Deliver {
                            message: bridget_core::BridgetMessage::new(
                                "bridget",
                                &request.target,
                                format!(
                                    "Demande #{} annulée par {}. Aucune réponse n'est requise.{}",
                                    request.id,
                                    sender,
                                    request
                                        .cancel_reason
                                        .as_deref()
                                        .map(|reason| format!(" Motif : {reason}"))
                                        .unwrap_or_default()
                                ),
                            ),
                            mid_turn_ok: Some(true),
                        }
                    };
                    (Arc::clone(writer), message)
                })
            });
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

pub(super) fn on_service_hello(
    version: u16,
    service: String,
    issuer_scope: String,
    capabilities: Vec<ServiceCapability>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
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
    let mut st = state.lock().recover_poison();
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

pub(super) fn on_control_execution_reported(
    issuer_scope: String,
    command_id: String,
    execution_id: String,
    accepted: bool,
    refusal_reason: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (controls, response) = {
        let st = state.lock().recover_poison();
        let Some(target) = (match st.execution_store.execution_control_target(&execution_id) {
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
        let refusal_reason =
            (!accepted).then(|| refusal_reason.unwrap_or_else(|| "invalid_command".to_string()));
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

pub(super) fn on_decommission_order(
    name: String,
    command_id: String,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    info!("cycle_vie action=decommission phase=request name={name} command_id={command_id}");
    let (entry, launch_in_flight) = {
        let mut st = state.lock().recover_poison();
        let Some(entry) = st.fleet.desired_entry(&name).ok().flatten() else {
            return Some(DaemonToWrapper::DecommissionResult {
                command_id,
                outcome: if st.router.get_agent(&name).is_some() {
                    DecommissionOutcome::NotManaged
                } else {
                    DecommissionOutcome::NotFound
                },
            });
        };
        if entry.lifecycle_state == DesiredLifecycleState::Decommissioned {
            return Some(DaemonToWrapper::DecommissionResult {
                command_id,
                outcome: DecommissionOutcome::AlreadyDecommissioned,
            });
        }
        if !st.decommissioning_names.insert(name.clone()) {
            return Some(DaemonToWrapper::DecommissionResult {
                command_id,
                outcome: DecommissionOutcome::Timeout {
                    state: "décommissionnement déjà en cours".to_string(),
                },
            });
        }
        let launch_in_flight = st
            .managed_spawns
            .values()
            .any(|spawn| spawn.lease.name == name);
        (entry, launch_in_flight)
    };
    let stop_outcome =
        if entry.lifecycle_state == DesiredLifecycleState::Running || launch_in_flight {
            await_managed_stop(prepare_managed_stop(state, &name), &name)
        } else {
            StopOutcome::Stopped
        };
    let outcome = match stop_outcome {
        StopOutcome::Stopped => {
            let mut st = state.lock().recover_poison();
            let outcome = match st.fleet.decommission(&name) {
                Ok(true) => {
                    prune_agent_presence(&mut st, &name);
                    DecommissionOutcome::Decommissioned
                }
                Ok(false) => DecommissionOutcome::NotFound,
                Err(error) => DecommissionOutcome::Timeout {
                    state: error.to_string(),
                },
            };
            st.decommissioning_names.remove(&name);
            outcome
        }
        StopOutcome::StoppedForced { survivors_killed } => {
            let mut st = state.lock().recover_poison();
            let outcome = match st.fleet.decommission(&name) {
                Ok(true) => {
                    prune_agent_presence(&mut st, &name);
                    DecommissionOutcome::DecommissionedForced { survivors_killed }
                }
                Ok(false) => DecommissionOutcome::NotFound,
                Err(error) => DecommissionOutcome::Timeout {
                    state: error.to_string(),
                },
            };
            st.decommissioning_names.remove(&name);
            outcome
        }
        StopOutcome::NotManaged => {
            state
                .lock()
                .recover_poison()
                .decommissioning_names
                .remove(&name);
            DecommissionOutcome::NotManaged
        }
        StopOutcome::NotFound => {
            state
                .lock()
                .recover_poison()
                .decommissioning_names
                .remove(&name);
            DecommissionOutcome::NotFound
        }
        StopOutcome::Timeout { state: stop_state } => {
            state
                .lock()
                .recover_poison()
                .decommissioning_names
                .remove(&name);
            DecommissionOutcome::Timeout { state: stop_state }
        }
    };
    info!(
        "cycle_vie action=decommission phase=result name={name} command_id={command_id} outcome={outcome:?}"
    );
    Some(DaemonToWrapper::DecommissionResult {
        command_id,
        outcome,
    })
}

pub(super) fn on_register(
    identity_version: u8,
    agent_type: String,
    agent_id: String,
    host: Option<String>,
    transport: Option<String>,
    channel: ChannelReport,
    mode: Option<PresenceMode>,
    location: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    journal_available: Option<bool>,
    turn_attest: bool,
    queue_fallback: bool,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    if instance_id
        .as_ref()
        .is_some_and(|id| st.managed_terminal_instances.contains(id))
    {
        return Some(DaemonToWrapper::Nack {
            id: "register".to_string(),
            reason: "génération gérée déjà terminale".to_string(),
        });
    }
    if st.managed_spawns.values().any(|record| {
        record.lease.remote_placement.is_some()
            && (agent_id == record.lease.name
                || instance_id.as_ref() == Some(&record.lease.instance_id))
            && (agent_id != record.lease.name
                || instance_id.as_ref() != Some(&record.lease.instance_id))
    }) {
        return Some(DaemonToWrapper::Nack {
            id: "register".into(),
            reason: "remote_registration_mismatch".into(),
        });
    }
    let managed_instance = instance_id.clone();
    let response = handle_register_with_channel(
        conn_id,
        identity_version,
        agent_type,
        agent_id,
        host,
        transport,
        channel,
        mode,
        location,
        os,
        instance_id,
        domain,
        turn_in_progress,
        journal_available,
        &mut st,
        turn_attest,
    );
    if let DaemonToWrapper::Registered { agent_id, .. } = &response
        && let Some((owner, instance)) = live_connection_identity(&st, conn_id)
        && owner.as_str() == agent_id.as_str()
        && let Some(presence) = st.presences.get_mut(&instance)
    {
        presence.queue_fallback = queue_fallback;
    }
    if let (
        Some(instance_id),
        DaemonToWrapper::Registered {
            credential: _,
            agent_id: final_agent_id,
        },
    ) = (managed_instance, &response)
        && let Some(command_id) = st.managed_by_instance.get(&instance_id).cloned()
        && let Some(record) = st.managed_spawns.get_mut(&command_id)
    {
        record.wrapper_conn = Some(conn_id.to_string());
        if record.lease.remote_placement.is_some() {
            complete_remote_registration(&mut st, &command_id);
            return Some(response);
        }
        let _ = st.managed_tx.send(ManagedSupervisorCommand::Registered {
            instance_id,
            conn_id: conn_id.to_string(),
            name: final_agent_id.clone(),
        });
    }
    Some(response)
}

pub(super) fn on_coordination_subscribe(
    version: u16,
    after_cursor: Option<u64>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if version != COORDINATION_STREAM_VERSION {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::UnsupportedVersion {
                supported_versions: vec![COORDINATION_STREAM_VERSION],
            },
        });
    }
    let mut st = state.lock().recover_poison();
    let replay = match st.store.guichet_coordination_events_after(after_cursor) {
        Ok(replay) => replay,
        Err(error) => {
            error!("relève de coordination indisponible: {error}");
            st.coordination_subscriptions.insert(
                conn_id.to_string(),
                CoordinationSubscription {
                    fresh: false,
                    next_cursor: after_cursor.unwrap_or(0).saturating_add(1),
                },
            );
            if let Some(writer) = st.connections.get(conn_id).cloned() {
                st.pending_post_response_controls
                    .entry(conn_id.to_string())
                    .or_default()
                    .push(DeferredControl {
                        writer,
                        message: DaemonToWrapper::CoordinationUnavailable {
                            version,
                            reason: "source_unavailable".to_string(),
                        },
                    });
            }
            return None;
        }
    };
    let mut controls = Vec::new();
    let next_cursor = replay
        .through_cursor
        .or(after_cursor)
        .unwrap_or(0)
        .saturating_add(1);
    let fresh = replay.gap.is_none();
    if let Some((from_cursor, to_cursor)) = replay.gap {
        defer_control(
            &st,
            conn_id,
            DaemonToWrapper::CoordinationGap {
                version,
                from_cursor,
                to_cursor,
                reason: "cursor_gap".to_string(),
            },
            &mut controls,
        );
    } else {
        for event in replay.events {
            defer_control(
                &st,
                conn_id,
                guichet_coordination_response(event, version),
                &mut controls,
            );
        }
        defer_control(
            &st,
            conn_id,
            DaemonToWrapper::CoordinationSnapshotCaughtUp {
                version,
                through_cursor: replay.through_cursor,
            },
            &mut controls,
        );
    }
    st.coordination_subscriptions.insert(
        conn_id.to_string(),
        CoordinationSubscription { fresh, next_cursor },
    );
    if !controls.is_empty() {
        st.pending_post_response_controls
            .entry(conn_id.to_string())
            .or_default()
            .extend(controls);
    }
    None
}

pub(super) fn on_client_hello(
    contract_version: u16,
    issuer_scope: String,
    capabilities: Vec<ClientCapability>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if contract_version != CLIENT_CONTRACT_VERSION {
        return Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::UnsupportedVersion {
                supported_versions: vec![CLIENT_CONTRACT_VERSION],
            },
        });
    }
    if crate::idempotency::validate_issuer_scope(&issuer_scope).is_err() {
        return Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::InvalidIssuerScope,
        });
    }
    let mut st = state.lock().recover_poison();
    let active_scopes = st
        .client_negotiations
        .values()
        .map(|negotiated| negotiated.issuer_scope.as_str())
        .collect::<std::collections::HashSet<_>>();
    if !active_scopes.contains(issuer_scope.as_str())
        && active_scopes.len() >= MAX_ACTIVE_ISSUER_SCOPES
    {
        return Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::ActiveScopeLimit,
        });
    }
    let capabilities: Vec<ClientCapability> = capabilities
        .into_iter()
        .filter(|capability| {
            if st.auxiliary_connections.contains(conn_id)
                && st.connection_roles.get(conn_id) == Some(&ConnectionRole::Wrapper)
            {
                return matches!(
                    capability,
                    ClientCapability::CommunicationProjectsV1
                        | ClientCapability::UserInputV1
                        | ClientCapability::ThreadMembersV1
                        | ClientCapability::SpecialistsV1
                );
            }
            matches!(
                capability,
                ClientCapability::MobileAdministrationV1
                    | ClientCapability::MobileOperationsV1
                    | ClientCapability::UserInputV1
                    | ClientCapability::SendIdempotent
                    | ClientCapability::SendInformationV1
                    | ClientCapability::QueueListV1
                    | ClientCapability::QueueEditV1
                    | ClientCapability::ThreadMembersV1
                    | ClientCapability::HumanThreadViewV1
                    | ClientCapability::HumanThreadViewRecentV1
                    | ClientCapability::HumanThreadWatchV1
                    | ClientCapability::HumanLineageViewV1
                    | ClientCapability::HumanLineageWatchV1
                    | ClientCapability::HumanLineageCancelV1
                    | ClientCapability::HumanLineageComposeV1
                    | ClientCapability::CommunicationProjectsV1
                    | ClientCapability::Lookup
                    | ClientCapability::ExecutionControlV1
                    | ClientCapability::ProjectRoundPolicyV1
                    | ClientCapability::ControlStateV1
                    | ClientCapability::RoutingCatalogueV1
                    | ClientCapability::ProviderCatalogueV1
                    | ClientCapability::SpecialistsV1
                    | ClientCapability::AgentTrackingV1
            )
        })
        .collect();
    st.client_negotiations.insert(
        conn_id.to_string(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope,
            capabilities: capabilities.clone(),
        },
    );
    Some(DaemonToWrapper::ClientWelcome {
        version: CLIENT_CONTRACT_VERSION,
        build_id: crate::build_info::BUILD_ID.to_string(),
        horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
        issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        capabilities,
    })
}

pub(super) fn on_delivery_rejected(
    id: String,
    reason: String,
    execution_id: Option<String>,
    generation: Option<u64>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    match native_delegation::rejected(
        &mut st,
        conn_id,
        &id,
        &reason,
        execution_id.as_deref(),
        generation,
    ) {
        Ok(native_delegation::NativeRejection::Pass) => {}
        Ok(native_delegation::NativeRejection::NativeFailed) => {
            st.pending_replies.retain(|pending| pending.msg_id != id);
            return None;
        }
        Ok(_) => return None,
        Err(error) => {
            error!("refus de mission native non qualifié ou non persisté: {error}");
            return None;
        }
    }
    purge_expired_attach_sends(&mut st);
    if let Some(pending) = st.pending_attach_sends.remove(&id) {
        let writer = st.connections.get(&pending.conn_id).cloned();
        drop(st);
        if let Some(writer) = writer {
            let _ =
                push_control_message(&writer, &DaemonToWrapper::DeliveryRejected { id, reason });
        }
        return None;
    }
    let request = st.store.get_request(&id).ok().flatten();
    if let Some(request) = request {
        st.pending_replies.retain(|pending| pending.msg_id != id);
        if (reason.contains("échéance") || reason.contains("timeout ACP"))
            && !claim_timeout(&mut st.store, &id)
        {
            return None;
        }
        if reason.contains("échéance") || reason.contains("timeout ACP") {
            st.delegation_store.wait_epoch().bump();
        }
        let writer = st
            .router
            .get_agent(&request.sender)
            .and_then(|agent| st.connections.get(&agent.connection_id))
            .cloned();
        drop(st);
        if let Some(writer) = writer
            && let Err(error) = deliver_to_agent(
                &writer,
                &request.sender,
                &format!(
                    "{}Échec de livraison de la demande #{id} : {reason}",
                    native_delegation::provider_error_recovery(&reason)
                        .map_or_else(String::new, |recovery| format!("{recovery}\n\n"))
                ),
            )
        {
            error!(
                "impossible de notifier l'échec ACP à {}: {}",
                request.sender, error
            );
        }
    }
    None
}

pub(super) fn on_wait_agent_links(
    after_cursor: Option<u64>,
    timeout_ms: u64,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    const MAX_AGENT_LINK_WAIT: u64 = 30_000;
    if timeout_ms > MAX_AGENT_LINK_WAIT {
        return Some(DaemonToWrapper::Nack {
            id: "agent-link-events".to_string(),
            reason: "délai d'attente de descendance supérieur à 30 s".to_string(),
        });
    }
    let (parent_instance_id, fleet) = {
        let st = state.lock().recover_poison();
        let Some(parent_instance_id) = st.conn_instances.get(conn_id).cloned() else {
            return Some(DaemonToWrapper::Nack {
                id: "agent-link-events".to_string(),
                reason: "wrapper non enregistré".to_string(),
            });
        };
        (parent_instance_id, Arc::clone(&st.fleet))
    };
    match fleet.wait_for_agent_link_events(
        &parent_instance_id,
        after_cursor,
        Duration::from_millis(timeout_ms),
    ) {
        Ok(batch) => Some(DaemonToWrapper::AgentLinkEvents {
            timed_out: batch.events.is_empty(),
            through_cursor: batch.through_cursor,
            events: batch
                .events
                .into_iter()
                .map(|event| AgentLinkEventFrame {
                    cursor: event.cursor,
                    event_id: event.event_id,
                    link_id: event.link_id,
                    child_instance_id: event.child_instance_id,
                    state: event.state.as_str().to_string(),
                    observed_at: event.observed_at,
                    project: event.project,
                })
                .collect(),
        }),
        Err(error) => {
            error!("attente de descendance indisponible: {error}");
            Some(DaemonToWrapper::Nack {
                id: "agent-link-events".to_string(),
                reason: "attente de descendance indisponible".to_string(),
            })
        }
    }
}

pub(super) fn on_t3_thread_binding_fact(
    version: u16,
    t3_thread_id: String,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    let agent = crate::t3code::stable_uuid(&t3_thread_id);
    let instance = crate::t3code::stable_uuid(&format!("instance:{t3_thread_id}"));
    // T3 atteste aussi des identifiants opaques (mcp:…, thread:mcp:…).
    // Borne de SessionProof, filtre de contrôles commun avec lineage.
    // L'identité et le projet restent vérifiés.
    let valid = version == bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION
        && !t3_thread_id.is_empty()
        && t3_thread_id.len() <= 2048
        && !t3_thread_id
            .chars()
            .any(bridget_core::is_disallowed_control)
        && !st.auxiliary_connections.contains(conn_id)
        && st
            .router
            .get_agent(&agent)
            .is_some_and(|route| route.connection_id == conn_id)
        && live_connection_identity(&st, conn_id) == Some((agent, instance.clone()))
        && st
            .presences
            .get(&instance)
            .is_some_and(|p| p.transport == "t3code")
        && st.t3_project_connections.contains(conn_id);
    let contradictory = st
        .t3_thread_bindings
        .get(conn_id)
        .is_some_and(|prior| prior.as_deref() != Some(&t3_thread_id));
    let response = if valid && !contradictory {
        if let Err(code) = agent_tracking::bind_live(
            &mut st,
            &crate::t3code::stable_uuid(&t3_thread_id),
            &t3_thread_id,
        ) {
            warn!(
                "liaison de classement non enregistrée ({code}) ; la liaison de communication reste attestée, les autres travaux peuvent continuer"
            );
        }
        st.t3_thread_bindings
            .insert(conn_id.into(), Some(t3_thread_id));
        None
    } else {
        // Un projet pas encore attesté ne crée pas de fait ; une
        // contradiction avec un binding existant invalide ce lien.
        if st.t3_thread_bindings.contains_key(conn_id) {
            st.t3_thread_bindings.insert(conn_id.into(), None);
        }
        Some(DaemonToWrapper::Nack {
            id: "t3-thread-binding".into(),
            reason: "binding_unavailable".into(),
        })
    };
    invalidate_human_authority(&mut st);
    response
}

pub(super) fn on_daemon_identity_request(
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let st = state.lock().recover_poison();
    Some(DaemonToWrapper::DaemonIdentityReport {
        host: st.host.clone(),
        db_path: st.db_path.display().to_string(),
        instance_id: st.instance_id.clone(),
        features: {
            let mut features = vec![
                "sentinel_status_v1".to_string(),
                "native_reply_outbox_v1".to_string(),
                "wait_v1".to_string(),
                "user_input_v1".to_string(),
                "specialists_v1".to_string(),
                "remote_worker_v1".to_string(),
                "worker_candidate_probe_v1".to_string(),
                "auto_delegate_v1".to_string(),
                "auto_delegate_v2".to_string(),
                "worker_candidate_probe_v2".to_string(),
                "worker_workload_tools_v1".to_string(),
                "worker_release_lease_v2".to_string(),
                "worker_parent_source_proof_v1".to_string(),
                "worker_result_slot_deadline_v1".to_string(),
                "worker_workspace_diagnostics_v1".to_string(),
                "worker_origin_proof_v1".to_string(),
                "worker_origin_project_context_v1".to_string(),
                "worker_workspace_repair_v1".to_string(),
                "task_workspace_repair_v1".to_string(),
                "worker_start_recovery_v1".to_string(),
                "worker_workspace_failure_v1".to_string(),
                "worker_provider_catalogue_v1".to_string(),
                "worker_model_options_v1".to_string(),
                "worker_provider_accounts_v1".to_string(),
                "worker_execution_status_v1".to_string(),
                "worker_program_compatibility_v1".to_string(),
                "worker_workspace_view_v1".to_string(),
                "commit_view_files_v1".to_string(),
                "routing_catalogue_v1".to_string(),
            ];
            if crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED {
                features.push("worker_workspace_view_create_v1".to_string());
            }
            if crate::provider_catalogue::MODEL_OPTIONS_LAUNCH_ENABLED {
                features.push("model_options_v1".to_string());
            }
            features
        },
    })
}

pub(super) fn on_subscribed(
    subscription_id: String,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let control = {
        let st = state.lock().recover_poison();
        let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
        if subscription.wrapper_conn != conn_id {
            return None;
        }
        log::debug!(
            "abonnement attach {} accepté par {}",
            subscription_id,
            subscription.agent
        );
        st.connections
            .get(&subscription.attach_conn)
            .map(|writer| DeferredControl {
                writer: writer.clone(),
                message: DaemonToWrapper::Subscribed {
                    subscription_id: subscription_id.clone(),
                },
            })
    };
    let delivered = control.is_some_and(|control| execute_controls(vec![control]).is_empty());
    if !delivered {
        let (control, view) = {
            let mut st = state.lock().recover_poison();
            let subscription = st.attach_subscriptions.remove(&subscription_id);
            let control = subscription.and_then(|subscription| {
                st.connections
                    .get(&subscription.wrapper_conn)
                    .map(|writer| DeferredControl {
                        writer: writer.clone(),
                        message: DaemonToWrapper::Unsubscribe {
                            subscription_id: subscription_id.clone(),
                        },
                    })
            });
            (control, st.attach_views.remove(&subscription_id))
        };
        if let Some(control) = control {
            let _ = execute_controls(vec![control]);
        }
        if let Some(view) = view {
            view.close_and_join();
        }
    }
    None
}

pub(super) fn on_turn_state(
    in_progress: bool,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    let finished = !in_progress
        && st
            .conn_instances
            .get(conn_id)
            .and_then(|id| st.presences.get(id))
            .is_some_and(|p| p.state == "busy");
    match st.set_turn_state(conn_id, in_progress) {
        Ok(()) => {
            if finished && let Some(notifier) = &st.mobile_notifier {
                notifier.changed();
            }
            // La levée peut précéder le terminal de `turn/interrupt`.
            // Le retour à l'état disponible rejoue alors la reprise
            // durable. Celle-ci reste refusée tant que le contrôle est
            // toujours en pause.
            let controls = if !in_progress {
                resume_executions_after_pause(&mut st)
            } else {
                Vec::new()
            };
            if !in_progress
                && controls.is_empty()
                && let Some((agent, _)) = live_connection_identity(&st, conn_id)
            {
                schedule_held_release(&mut st, &agent, Instant::now());
            }
            drop(st);
            for (target_conn, controls) in controls {
                let retry = execute_controls(controls);
                if !retry.is_empty() {
                    state
                        .lock()
                        .recover_poison()
                        .pending_post_response_controls
                        .entry(target_conn)
                        .or_default()
                        .extend(retry);
                }
            }
            None
        }
        Err(reason) => Some(DaemonToWrapper::Nack {
            id: "turn-state".to_string(),
            reason,
        }),
    }
}

pub(super) fn on_unsubscribe(
    subscription_id: String,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (control, view) = {
        let mut st = state.lock().recover_poison();
        let Some(subscription) = st.attach_subscriptions.get(&subscription_id).cloned() else {
            // Une fin d'ancienne génération n'a pas le droit de toucher
            // une souscription plus récente sur la même connexion.
            return None;
        };
        if subscription.attach_conn != conn_id {
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: Some(subscription_id),
                reason: AttachRefusal::MessageOutsideAttachRole,
                mode: None,
                location: None,
            });
        }
        st.attach_subscriptions.remove(&subscription_id);
        let control = st
            .connections
            .get(&subscription.wrapper_conn)
            .map(|writer| DeferredControl {
                writer: writer.clone(),
                message: DaemonToWrapper::Unsubscribe {
                    subscription_id: subscription_id.clone(),
                },
            });
        (control, st.attach_views.remove(&subscription_id))
    };
    if let Some(control) = control {
        let _ = execute_controls(vec![control]);
    }
    if let Some(view) = view {
        let _ = view.enqueue(DaemonToWrapper::End {
            subscription_id,
            reason: "désabonnement demandé".to_string(),
        });
        view.close_and_join();
    }
    None
}

pub(super) fn on_delivery_indeterminate(
    delivery_id: String,
    delivery_generation: u64,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let Some(instance_id) = state
        .lock()
        .recover_poison()
        .conn_instances
        .get(conn_id)
        .cloned()
    else {
        return Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: "accusé indéterminé émis par une instance inconnue".to_string(),
        });
    };
    let mut st = state.lock().recover_poison();
    match st.idempotency.mark_delivery_indeterminate(
        &delivery_id,
        &instance_id,
        delivery_generation,
    ) {
        Ok(()) => {
            settle_thread_wake_by_delivery(
                &mut st,
                &delivery_id,
                crate::store::threads::WakeOutcome::OutcomeUnknown,
            );
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_delivery_indeterminate");
            None
        }
        Err(error) => Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: error.to_string(),
        }),
    }
}

pub(super) fn on_list_requests(
    sender: String,
    limit: u16,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let st = state.lock().recover_poison();
    match st
        .store
        .requests_for_participant(&sender, usize::from(limit))
    {
        Ok(requests) => match requests
            .into_iter()
            .map(|request| -> Result<_, crate::store::StoreError> {
                let deferred = st.store.latest_deferred_reminder(&request.id)?;
                Ok(bridget_transport::protocol::RequestInfo {
                    id: request.id,
                    sender: request.sender,
                    target: request.target,
                    state: request.state,
                    created_at: request.created_at,
                    deadline_at: request.deadline_at,
                    cancel_reason: request.cancel_reason,
                    deferred_reminder_level: deferred.map(|event| event.0),
                    deferred_reminder_at: deferred.map(|event| event.1),
                })
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(requests) => Some(DaemonToWrapper::RequestList { requests }),
            Err(error) => Some(DaemonToWrapper::Nack {
                id: "requests".to_string(),
                reason: error.to_string(),
            }),
        },
        Err(error) => Some(DaemonToWrapper::Nack {
            id: "requests".to_string(),
            reason: error.to_string(),
        }),
    }
}

pub(super) fn on_adopt_stopped_order(
    name: String,
    command_id: String,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    info!("cycle_vie action=adopt_stopped phase=request name={name} command_id={command_id}");
    let mut st = state.lock().recover_poison();
    let visible = st
        .agent_infos()
        .into_iter()
        .find(|agent| agent.agent_id == name);
    let outcome = match visible {
        Some(agent) if agent.persistent.is_some() => AdoptStoppedOutcome::AlreadyManaged,
        Some(agent) if agent.state != "stopped" => AdoptStoppedOutcome::NotStopped,
        Some(_) | None => match st.fleet.adopt_stopped(&name, unix_timestamp()) {
            Ok(AdoptStoppedResult::Adopted { generation }) => {
                AdoptStoppedOutcome::Adopted { generation }
            }
            Ok(AdoptStoppedResult::AlreadyManaged) => AdoptStoppedOutcome::AlreadyManaged,
            Ok(AdoptStoppedResult::NoManagedHistory) => AdoptStoppedOutcome::NoManagedHistory,
            Err(error) => AdoptStoppedOutcome::IncompleteHistory {
                reason: error.to_string(),
            },
        },
    };
    info!(
        "cycle_vie action=adopt_stopped phase=result name={name} command_id={command_id} outcome={outcome:?}"
    );
    Some(DaemonToWrapper::AdoptStoppedResult {
        command_id,
        outcome,
    })
}

pub(super) fn on_attach_rejected(
    subscription_id: Option<String>,
    reason: AttachRefusal,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (control, view) = {
        let mut st = state.lock().recover_poison();
        let subscription_id = subscription_id?;
        let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
        if subscription.wrapper_conn != conn_id {
            return None;
        }
        let control = st
            .connections
            .get(&subscription.attach_conn)
            .map(|writer| DeferredControl {
                writer: writer.clone(),
                message: DaemonToWrapper::AttachRejected {
                    subscription_id: Some(subscription_id.clone()),
                    reason,
                    mode: None,
                    location: None,
                },
            });
        st.attach_subscriptions.remove(&subscription_id);
        (control, st.attach_views.remove(&subscription_id))
    };
    if let Some(control) = control {
        let _ = execute_controls(vec![control]);
    }
    if let Some(view) = view {
        view.close_and_join();
    }
    None
}

pub(super) fn on_journal_ready(
    receipt_probe: bool,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    let Some(instance_id) = st.conn_instances.get(conn_id).cloned() else {
        return Some(DaemonToWrapper::Nack {
            id: "journal-ready".to_string(),
            reason: "journal annoncé avant l'enregistrement".to_string(),
        });
    };
    let Some(presence) = st.presences.get_mut(&instance_id) else {
        return Some(DaemonToWrapper::Nack {
            id: "journal-ready".to_string(),
            reason: "présence introuvable pour le journal annoncé".to_string(),
        });
    };
    presence.journal_available = true;
    if receipt_probe {
        // La connexion et l'instance sont authentifiées comme pour un
        // ACK normal. Aucun corps ni ordre d'exécution n'est renvoyé.
        match st
            .idempotency
            .unknown_delivery_receipts(&instance_id, unix_now_secs())
        {
            Ok(receipts) => {
                let mut controls = Vec::new();
                for (delivery_id, delivery_generation) in receipts {
                    defer_control(
                        &st,
                        conn_id,
                        DaemonToWrapper::DeliveryReceiptQuery {
                            delivery_id,
                            recipient_instance_id: instance_id.clone(),
                            delivery_generation,
                        },
                        &mut controls,
                    );
                }
                st.pending_post_response_controls
                    .entry(conn_id.into())
                    .or_default()
                    .extend(controls);
            }
            Err(error) => warn!("lecture des reçus indéterminés indisponible : {error}"),
        }
    }
    None
}

pub(super) fn on_live_journal_fragment(
    seq: u64,
    offset: u64,
    final_fragment: bool,
    bytes: Vec<u8>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let views = {
        let st = state.lock().recover_poison();
        st.attach_subscriptions
            .iter()
            .filter(|(_, subscription)| {
                subscription.wrapper_conn == conn_id && subscription.caught_up
            })
            .filter_map(|(subscription_id, _)| {
                st.attach_views
                    .get(subscription_id)
                    .cloned()
                    .map(|view| (subscription_id.clone(), view))
            })
            .collect::<Vec<_>>()
    };
    for (subscription_id, view) in views {
        let _ = view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id,
            seq,
            offset,
            final_fragment,
            bytes: bytes.clone(),
        });
    }
    None
}

pub(super) fn on_send(
    mut bridge_msg: bridget_core::BridgetMessage,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if bridge_msg.intent == Some(bridget_core::MessageIntent::StoreOnly) {
        return Some(DaemonToWrapper::Nack { id: bridge_msg.id.clone(), reason:
                    "information_requires_idempotent_send : utilisez send --intent information ou bridget_send intent=information avec négociation ; aucun envoi normal de remplacement.".into() });
    }
    // Session 102 : aucune notice de fil ne vient d'un client.
    bridge_msg.thread_notice = None;
    bridge_msg.thread_display_title = None;
    eprintln!(
        "[BRIDGET] Send de {}: to={}, body={}",
        conn_id,
        bridge_msg.to,
        bridge_msg.body.chars().take(40).collect::<String>()
    );
    let mut st = state.lock().recover_poison();
    let is_attach = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Attach);
    purge_expired_attach_sends(&mut st);
    if is_attach && st.pending_attach_sends.len() >= MAX_PENDING_ATTACH_SENDS {
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason: "trop d envois humains en attente".to_string(),
        });
    }
    let sender_name = st.conn_names.get(conn_id).cloned().unwrap_or_default();
    let ephemeral_cli = is_ephemeral_cli_route(conn_id, st.router.get_agent(&sender_name));
    if is_attach {
        bridge_msg.from = "humain".to_string();
        bridge_msg.in_reply_to = None;
    } else {
        // Un terminal extérieur a sa propre route temporaire, jamais
        // celle d'un agent dont il connaît simplement l'UUID.
        if ephemeral_cli
            && matches!(bridge_msg.from.as_str(), "" | "human" | "humain")
            && resolve_sender_attribution(
                true,
                bridge_msg.from_declared,
                st.router.get_agent(&bridge_msg.from).is_some(),
            ) == SenderAttribution::UseConnectionName
        {
            bridge_msg.from = sender_name;
        }
        if !sender_is_authorized(&st, conn_id, &bridge_msg.from) {
            return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: "identité expéditeur non attestée : --from ne permet pas d'emprunter une identité".into(),
                    });
        }
    }

    normalize_guidance_origin(&st, conn_id, &mut bridge_msg, false);
    if bridge_msg.reply && bridge_msg.intent == Some(bridget_core::MessageIntent::SteerCurrent) {
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason: "invalid_intent".into(),
        });
    }
    if bridge_msg.id.starts_with("reply244-")
        && bridge_msg.in_reply_to.is_some()
        && let Some((_, instance)) = live_connection_identity(&st, conn_id)
    {
        return Some(retain_wrapper_reply(
            conn_id, bridge_msg, &instance, &mut st,
        ));
    }
    let logical_sender = bridge_msg.from.clone();
    let captured = match native_delegation::capture_reply(&mut st, conn_id, &bridge_msg) {
        Ok(captured) => captured,
        Err(error) => {
            error!("capture native non persistée: {error}");
            return Some(DaemonToWrapper::Nack {
                id: bridge_msg.id.clone(),
                reason: if error == "remote_result_requires_worker"
                    || error.starts_with("retained_reply_")
                {
                    error
                } else {
                    "native_result_not_persisted".into()
                },
            });
        }
    };
    if captured {
        return Some(DaemonToWrapper::Ack {
            id: bridge_msg.id.clone(),
            project_warnings: Vec::new(),
        });
    }
    let content_key = bridge_msg.content_key();
    let message_guard_id = bridge_msg.id.clone();
    let guidance_action = match bridge_msg.intent {
        Some(bridget_core::MessageIntent::SteerCurrent) => {
            Some(crate::guidance::GuidanceAction::Steer)
        }
        Some(bridget_core::MessageIntent::InterruptAndStart) => {
            Some(crate::guidance::GuidanceAction::Interrupt)
        }
        _ => None,
    };
    let prepared = match prepare_dispatch(
        &mut st,
        &mut bridge_msg,
        conn_id,
        logical_sender,
        content_key,
        message_guard_id,
        false,
    ) {
        Ok(prepared) => prepared,
        Err((category, reason)) => {
            record_guidance_refusal(
                &st,
                Some(conn_id),
                &bridge_msg.to,
                &bridge_msg.id,
                guidance_action,
                crate::guidance::GuidancePath::Send,
                category,
            );
            return Some(DaemonToWrapper::Nack {
                id: bridge_msg.id.clone(),
                reason,
            });
        }
    };
    let guidance_admission = match guidance_action {
        Some(action) => {
            match check_guidance(
                &st,
                Some(conn_id),
                &bridge_msg.to,
                action,
                crate::guidance::GuidancePath::Send,
                None,
                bridge_msg.delegated_origin.is_some(),
                &bridge_msg.id,
            ) {
                Ok(facts) => Some(facts),
                Err((code, reason)) => {
                    return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: format!("{code}:{reason}"),
                    });
                }
            }
        }
        _ => None,
    };
    let is_ephemeral_cli_sender =
        ephemeral_cli && prepared.reply_sender_conn.as_deref() == Some(conn_id);
    if bridge_msg.reply && is_ephemeral_cli_sender {
        if let Some(facts) = &guidance_admission
            && let Err(error) = record_guidance_outcome(
                &st,
                facts,
                &bridge_msg.to,
                &bridge_msg.id,
                "refused:reply_sender_unavailable",
            )
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "--reply requiert un agent Bridget connecté ; lance la commande depuis un wrapper actif ou envoie sans --reply".to_string(),
                });
    }
    let target_conn = prepared.target_conn;
    let conv_key = format!("{}|{}", bridge_msg.from, bridge_msg.to);
    if bridge_msg.intent == Some(bridget_core::MessageIntent::ControlOnly) {
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason: "ControlOnly requiert une commande ControlExecution versionnée".to_string(),
        });
    }

    match st.store.record_message(&bridge_msg, &conv_key) {
        Ok(()) => {
            // Même frontière que la voie idempotente : le message est
            // durable avant de rafraîchir la capacité de l'expéditeur.
            touch_message_sender_activity(&mut st, &bridge_msg.from);
        }
        Err(error) => {
            if let Some(facts) = &guidance_admission
                && let Err(error) = record_guidance_outcome(
                    &st,
                    facts,
                    &bridge_msg.to,
                    &bridge_msg.id,
                    "refused:persistence_failed",
                )
            {
                warn!("refus de guidage non journalisé : {error}");
            }
            return Some(DaemonToWrapper::Nack {
                id: bridge_msg.id.clone(),
                reason: format!("message non persisté: {error}"),
            });
        }
    }
    st.circuit_breaker
        .record(&prepared.logical_sender, &bridge_msg.to);
    st.deduplicator
        .mark_sent(&prepared.content_key, &bridge_msg.to);
    st.envelope_guard
        .mark_relayed(&prepared.message_guard_id, &bridge_msg.to);

    let execution_delivery = match bridge_msg.intent {
        Some(bridget_core::MessageIntent::QueueOnly) => {
            match st.execution_store.admit_message_submission(
                &bridge_msg,
                crate::execution_store::ExecutionStore::queue_priority_for(&bridge_msg, 0),
                unix_now_secs(),
            ) {
                Ok(admitted) => {
                    if admitted {
                        get_metrics().record_execution_admitted();
                    }
                    return Some(DaemonToWrapper::Ack {
                        project_warnings: Vec::new(),
                        id: bridge_msg.id.clone(),
                    });
                }
                Err(error) => {
                    return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: format!("admission non persistée: {error}"),
                    });
                }
            }
        }
        Some(
            bridget_core::MessageIntent::TriggerTurn
            | bridget_core::MessageIntent::InterruptAndStart,
        ) => {
            let execution_id = format!("execution-{}", bridge_msg.id);
            match st.execution_store.admit_starting_message(
                &bridge_msg,
                &execution_id,
                unix_now_secs(),
            ) {
                Ok(true) => {
                    get_metrics().record_execution_admitted();
                    Some((execution_id, 1_u64, 0_u64))
                }
                Ok(false) => {
                    return Some(DaemonToWrapper::Ack {
                        project_warnings: Vec::new(),
                        id: bridge_msg.id.clone(),
                    });
                }
                Err(error) => {
                    if let Some(facts) = &guidance_admission
                        && let Err(error) = record_guidance_outcome(
                            &st,
                            facts,
                            &bridge_msg.to,
                            &bridge_msg.id,
                            "refused:persistence_failed",
                        )
                    {
                        warn!("refus de guidage non journalisé : {error}");
                    }
                    return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: format!("démarrage non persisté: {error}"),
                    });
                }
            }
        }
        _ => None,
    };

    // Le daemon est l'autorité de l'échéance de TOUR (pas seulement
    // reply=yes) : relire la valeur courante et la pousser en absolu.
    // Sans cela, worker.notify_timeout figé au spawn ignore agents.json.
    let mut delivered_message = bridge_msg.clone();
    // Ne jamais remplacer l'Agent ID dans le message routé : le
    // wrapper a encore besoin de `from` pour reply. Le display name
    // est une projection d'injection sans sémantique de routage.
    delivered_message.from_display_name = provider_display_name(&st, &bridge_msg.from);
    delivered_message.thread_display_title = provider_thread_display_title(&st, &delivered_message);
    let agent_type = st
        .conn_instances
        .get(&target_conn)
        .and_then(|instance_id| st.presences.get(instance_id))
        .map(|presence| presence.agent_type.as_str())
        .unwrap_or("");
    stamp_turn_deadline_for_delivery(&mut delivered_message, &st.registry, agent_type);
    // Push vers le destinataire
    let mid_turn_ok = Some(guidance_mid_turn_ok(
        &st,
        &bridge_msg,
        &bridge_msg.to,
        None,
        Some(conn_id),
    ));
    let execution_attempt = execution_delivery.clone();
    let dtw = match execution_delivery {
        Some((execution_id, generation, revision)) => DaemonToWrapper::DeliverExecution {
            message: delivered_message,
            execution_id,
            generation,
            revision,
        },
        None => DaemonToWrapper::Deliver {
            message: delivered_message,
            mid_turn_ok,
        },
    };
    // Le destinataire peut répondre dès le premier octet remis : son
    // suivi et la route attach doivent donc exister AVANT le push.
    if let Err(reason) = track_reply_cycle(
        &mut st,
        &bridge_msg,
        prepared.reply_sender_conn.clone(),
        target_conn.clone(),
    ) {
        if let Some(facts) = &guidance_admission
            && let Err(error) = record_guidance_outcome(
                &st,
                facts,
                &bridge_msg.to,
                &bridge_msg.id,
                "refused:persistence_failed",
            )
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason,
        });
    }
    if is_attach {
        st.pending_attach_sends.insert(
            bridge_msg.id.clone(),
            PendingAttachSend {
                conn_id: conn_id.to_string(),
                expires_at: Instant::now() + PENDING_ATTACH_SEND_TTL,
            },
        );
    }
    if let Some(facts) = &guidance_admission
        && let Err(error) =
            record_guidance_outcome(&st, facts, &bridge_msg.to, &bridge_msg.id, "admitted")
    {
        if let Some(facts) = &guidance_admission
            && let Err(error) = record_guidance_outcome(
                &st,
                facts,
                &bridge_msg.to,
                &bridge_msg.id,
                "refused:persistence_failed",
            )
        {
            warn!("refus de guidage non journalisé : {error}");
        }
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason: format!("guidage non persisté: {error}"),
        });
    }
    let target_writer = st.connections.get(&target_conn).cloned();
    drop(st);
    let delivery = target_writer
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotConnected, "cible disparue"))
        .and_then(|writer| {
            push_control_message_until(&writer, &dtw, Instant::now() + CANCEL_NOTIFICATION_BUDGET)
        });
    let delivery_succeeded = delivery.is_ok();
    // Aucun writer n'est détenu à la reprise du verrou global. Le
    // suivi peut déjà être terminal ; ne jamais le recréer ici.
    let mut st = state.lock().recover_poison();
    if !delivery_succeeded && let Some((execution_id, generation, revision)) = execution_attempt {
        get_metrics().record_execution_delivery_failure();
        if let Err(error) = st.execution_store.transition_if_current(
            &execution_id,
            "starting",
            revision,
            generation,
            "unreachable",
            "provider_unavailable",
            unix_now_secs(),
        ) {
            error!("transition exécution {execution_id} après remise échouée: {error}");
        }
    }

    if delivery_succeeded
        && prepared.valid_tracked_reply
        && let Some(request_id) = bridge_msg.in_reply_to.as_deref()
        && st
            .store
            .mark_answered(
                request_id,
                &bridge_msg.from,
                &bridge_msg.to,
                Some(&crate::store::request_reply_summary(&bridge_msg)),
            )
            .unwrap_or(false)
    {
        st.pending_replies
            .retain(|pending| pending.msg_id != request_id);
        info!("demande {} répondue après livraison", request_id);
        st.delegation_store.wait_epoch().bump();
    }

    if let Err(error) = delivery {
        st.pending_attach_sends.remove(&bridge_msg.id);
        return Some(DaemonToWrapper::Nack {
            id: bridge_msg.id.clone(),
            reason: format!(
                "remise indéterminée: écriture vers le destinataire échouée ({error}); vérifier le suivi avant tout rejeu"
            ),
        });
    }
    Some(DaemonToWrapper::Ack {
        id: bridge_msg.id.clone(),
        project_warnings: prepared.project_warnings,
    })
}

pub(super) fn on_service_request(
    version: u16,
    issuer_scope: String,
    request_id: String,
    issued_at: i64,
    from: String,
    to: String,
    operation: bridget_transport::protocol::ServiceRequestOperation,
    payload: bridget_transport::protocol::ServiceRequestPayload,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if !matches!(
        version,
        SERVICE_CONTRACT_VERSION | REVIEW_DELEGATE_CONTRACT_VERSION
    ) {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::UnsupportedVersion {
                supported_versions: vec![
                    SERVICE_CONTRACT_VERSION,
                    REVIEW_DELEGATE_CONTRACT_VERSION,
                ],
            },
        });
    }
    if to != "guichet" {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::ReservedTargetRequired,
        });
    }
    if !guichet_request_is_valid(version, &request_id, operation, &payload)
        || crate::idempotency::validate_issuer_scope(&issuer_scope).is_err()
    {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::InvalidEnvelope,
        });
    }
    let (declared_name, declared_instance_id, declared_sender_matches) = {
        let st = state.lock().recover_poison();
        let name = st.conn_names.get(conn_id).cloned();
        let instance_id = st.conn_instances.get(conn_id).cloned();
        let matches = name.as_ref().is_some_and(|registered| {
            registered == &from
                || (is_ephemeral_cli_route(conn_id, st.router.get_agent(registered))
                    && st.router.get_agent(&from).is_some())
        });
        (name, instance_id, matches)
    };
    let mutation_action = greffe_mutation_action(operation);
    if !declared_sender_matches {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: if mutation_action.is_some() {
                ServiceRefusal::GreffeAuthorizationDenied
            } else {
                ServiceRefusal::DeclaredSenderMismatch
            },
        });
    }
    // SPEC-087 : seul le principal humain (relais UI enregistré sous
    // l'identité humaine) peut porter une origine humaine ou un focus,
    // et c'est le daemon qui fabrique l'origine, jamais le client.
    let human_principal = {
        let st = state.lock().recover_poison();
        declared_name.as_deref() == Some(UI_HUMAN_AGENT_ID)
            && declared_instance_id.as_deref().is_some_and(|instance| {
                st.presences
                    .get(instance)
                    .is_some_and(|presence| presence.agent_type == "ui")
            })
    };
    let mut payload = payload;
    let carries_human_claims = matches!(
        &payload,
        bridget_transport::protocol::ServiceRequestPayload::Delegate {
            origin: Some(_),
            ..
        } | bridget_transport::protocol::ServiceRequestPayload::Delegate { focus: Some(_), .. }
    );
    if carries_human_claims && !human_principal {
        warn!("dépôt {request_id}: origine humaine ou focus refusés à {from}");
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::HumanOriginForbidden,
        });
    }
    if human_principal
        && let bridget_transport::protocol::ServiceRequestPayload::Delegate { origin, goal, .. } =
            &mut payload
    {
        // Tout `origin` reçu est ignoré : le hash canonique se calcule
        // sur le dépôt SANS origine (ADR-014, pas de hash récursif).
        *origin = None;
        let goal = goal.clone();
        let without_origin = match encode(&WrapperToDaemon::ServiceRequest {
            version,
            issuer_scope: issuer_scope.clone(),
            request_id: request_id.clone(),
            issued_at,
            from: from.clone(),
            to: to.clone(),
            operation,
            payload: payload.clone(),
        }) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
        };
        // Déterministe par (périmètre, request_id) : un rejeu du même
        // dépôt refabrique la même origine et reste idempotent.
        let message_id = format!(
            "hmo-{}",
            &bridget_transport::protocol::sha256_hex(
                format!("{issuer_scope}\n{request_id}").as_bytes()
            )[..32]
        );
        let observed = bridget_transport::protocol::ObservedHumanMessageFrame {
            message_id: message_id.clone(),
            ts: issued_at,
            sender: UI_HUMAN_SENDER.to_string(),
            target: "guichet".to_string(),
            body: goal.clone(),
        };
        let attestation = bridget_transport::protocol::HumanOriginAttestationFrame {
            version: 1,
            issuer_scope: issuer_scope.clone(),
            canonical_request_sha256: bridget_transport::protocol::sha256_hex(
                without_origin.as_bytes(),
            ),
            signature: bridget_transport::protocol::human_message_content_seal(&observed),
        };
        {
            let st = state.lock().recover_poison();
            let mut human_message =
                bridget_core::BridgetMessage::new(UI_HUMAN_SENDER, "guichet", goal);
            human_message.id = message_id.clone();
            human_message.origin = Some(bridget_core::MessageOrigin::Human);
            let conv_key = format!("{}|{}", human_message.from, human_message.to);
            if let Err(error) = st.store.record_message(&human_message, &conv_key) {
                error!("ledger du message humain d'origine: {error}");
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::TransitionInvalid,
                });
            }
        }
        if let bridget_transport::protocol::ServiceRequestPayload::Delegate { origin, .. } =
            &mut payload
        {
            *origin = Some(bridget_transport::protocol::DelegateOrigin::Human {
                message_id: message_id.clone(),
                observed,
                attestation,
            });
        }
        info!("dépôt {request_id}: origine humaine attestée ({message_id})");
    }
    let canonical = match encode(&WrapperToDaemon::ServiceRequest {
        version,
        issuer_scope: issuer_scope.clone(),
        request_id: request_id.clone(),
        issued_at,
        from: from.clone(),
        to,
        operation,
        payload: payload.clone(),
    }) {
        Ok(bytes) => bytes.into_bytes(),
        Err(_) => {
            return Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::InvalidEnvelope,
            });
        }
    };
    let now = unix_timestamp();
    let mut deposit = GuichetDeposit {
        issuer_scope: issuer_scope.clone(),
        request_id: request_id.clone(),
        issued_at,
        from,
        operation,
        payload,
        canonical_bytes: canonical,
        authorization_attestation: None,
    };
    let authorization_declared_from = deposit.from.clone();
    let authorization_request_id = deposit.request_id.clone();
    let authorization_canonical_request = deposit.canonical_bytes.clone();
    // SPEC-087 : pour le principal humain, l'attestation d'origine EST
    // l'autorisation ; la garde du greffe, faite pour les agents, ne
    // s'applique pas (ADR 027).
    let deposit_result = match mutation_action.filter(|_| !human_principal) {
        Some(action) => {
            // `declared_name` et `declared_instance_id` viennent du
            // Register de cette connexion. Ils réduisent les sources
            // d'identité sans authentifier le processus pair.
            let gate = GreffeAuthorizationGate::from_environment();
            match gate.authorize_deposit_then(
                GreffeDepositAuthorization {
                    canonical_name: declared_name.as_deref(),
                    canonical_instance_id: declared_instance_id.as_deref(),
                    declared_from: Some(authorization_declared_from.as_str()),
                    action,
                    issuer_scope: &issuer_scope,
                    request_id: authorization_request_id.as_str(),
                    request_issued_at: deposit.issued_at,
                    canonical_request: &authorization_canonical_request,
                    observed_at: now,
                },
                |attestation| {
                    deposit.authorization_attestation = Some(attestation.clone());
                    state.lock().recover_poison().store.deposit_guichet(
                        &deposit,
                        CLIENT_IDEMPOTENCY_HORIZON_SECS,
                        CLIENT_ISSUED_AT_TOLERANCE_SECS,
                        now,
                    )
                },
            ) {
                Ok(result) => result,
                Err(refusal) => {
                    warn!(
                        "mutation du greffe refusée au dépôt: {}",
                        refusal.audit_code()
                    );
                    return Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::GreffeAuthorizationDenied,
                    });
                }
            }
        }
        None => state.lock().recover_poison().store.deposit_guichet(
            &deposit,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
            now,
        ),
    };
    match deposit_result {
        Ok(result) => Some(guichet_result_response(issuer_scope, request_id, result)),
        Err(StoreError::FrameTooLarge { .. }) => Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::FrameTooLarge,
        }),
        Err(error) => {
            error!("dépôt guichet: {error}");
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::TransitionInvalid,
            })
        }
    }
}

pub(super) fn on_spawn_order(
    posture: Option<bridget_transport::protocol::SpawnPosture>,
    agent_type: String,
    project: Option<bridget_transport::protocol::ProjectReference>,
    agent_id: Option<String>,
    cwd: String,
    persistent: bool,
    command_id: String,
    issued_at: i64,
    deadline_at: i64,
    ownership: Option<bridget_transport::protocol::SpawnOwnership>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture,
        agent_type,
        requested_name: agent_id,
        cwd: PathBuf::from(cwd),
        persistent,
        project,
        command_id: command_id.clone(),
        issued_at,
        ownership,
        deadline_at,
    };
    // Seul endroit du programme où les deux machines coexistent : le
    // daemon cherche le `cwd` chez LUI, le demandeur peut être au bout
    // du tunnel. `conn_hosts` porte l'hôte attesté à l'enregistrement.
    let hosts = crate::lifecycle::SpawnHosts {
        searched_on: st.host.clone(),
        // Absence de connexion connue OU hote non atteste : meme fait,
        // meme valeur. Le rendu s en chargera, pas deux litteraux.
        requested_from: st
            .conn_hosts
            .get(conn_id)
            .filter(|host| bridget_core::host_is_attested(host))
            .cloned()
            .unwrap_or_else(|| bridget_core::HOTE_NON_ATTESTE.to_string()),
    };
    // Un projet retiré est refusé avant toute résolution de posture ou
    // de fournisseur. Une clé connue reste traitée par son canon durable.
    if !st.fleet.knows_command(&command_id)
        && let Some(project) = &order.project
    {
        return Some(DaemonToWrapper::SpawnRejected {
            command_id,
            reason: SpawnRefusal::DockerRuntimeUnavailable {
                project_id: project.project_id.clone(),
            },
        });
    }
    // La capacité seule ne vaut pas attribution humaine. Le choix de
    // développement est autorisé sur cette connexion et cet ordre,
    // jamais en modifiant l'état global du référent.
    if posture == Some(bridget_transport::protocol::SpawnPosture::Development)
        && !(st.connection_roles.get(conn_id) == Some(&ConnectionRole::Client)
            && control_client_actor(&st, conn_id).is_some()
            && st.client_negotiations.get(conn_id).is_some_and(|client| {
                client.version == CLIENT_CONTRACT_VERSION
                    && client
                        .capabilities
                        .contains(&ClientCapability::ControlStateV1)
            }))
    {
        return Some(DaemonToWrapper::SpawnRejected {
            command_id,
            reason: SpawnRefusal::UnsupportedCapability {
                agent_type: order.agent_type,
                model: String::new(),
                capability: "posture_development_human_principal_required".to_string(),
            },
        });
    }
    let scoped_registry = match posture {
        // Le canon et la définition durable sont l'autorité du rejeu,
        // même si le registre a changé depuis le premier envoi.
        Some(_) if st.fleet.knows_command(&command_id) => None,
        Some(posture) => match st.registry.for_spawn_posture(&order.agent_type, posture) {
            Ok(registry) => Some(registry),
            Err(detail) => {
                return Some(DaemonToWrapper::SpawnRejected {
                    command_id,
                    reason: SpawnRefusal::UnsupportedCapability {
                        agent_type: order.agent_type,
                        model: String::new(),
                        capability: detail,
                    },
                });
            }
        },
        None => None,
    };
    // Sans choix explicite, conserver la politique globale existante.
    let resolved_type = if posture.is_some() {
        Ok(order.agent_type.clone())
    } else {
        resolve_spawn_agent_type_for_posture(&st, &order.agent_type)
    };
    let order = match resolved_type {
        Ok(agent_type) => FleetSpawnOrder {
            agent_type,
            ..order
        },
        Err(reason) => {
            return Some(DaemonToWrapper::SpawnRejected { command_id, reason });
        }
    };
    let decision = crate::lifecycle::submit_spawn(
        &st.fleet,
        scoped_registry.as_ref().unwrap_or(&st.registry),
        &st.source_env,
        &order,
        unix_timestamp(),
        st.recovering,
        &hosts,
    );
    match decision {
        Ok(SpawnDecision::Ready(prepared)) => {
            let stop = Arc::new(ManagedStopControl::new());
            let supervisor_command = ManagedSupervisorCommand::Start {
                prepared: prepared.clone(),
                stop: Arc::clone(&stop),
            };
            st.managed_by_instance.insert(
                prepared.lease.instance_id.clone(),
                prepared.lease.command_id.clone(),
            );
            st.managed_spawns.insert(
                prepared.lease.command_id.clone(),
                ManagedSpawnRecord {
                    lease: prepared.lease.clone(),
                    agent_type: prepared.agent_type.clone(),
                    requester_conns: vec![conn_id.to_string()],
                    wrapper_conn: None,
                    stop: Arc::clone(&stop),
                },
            );
            if st.managed_tx.send(supervisor_command).is_err() {
                let _ = st.fleet.fail(
                    &prepared.lease,
                    "negotiation_failed",
                    "superviseur de processus indisponible",
                );
                st.managed_spawns.remove(&prepared.lease.command_id);
                st.managed_by_instance.remove(&prepared.lease.instance_id);
                st.managed_terminal_instances
                    .insert(prepared.lease.instance_id.clone());
                return Some(DaemonToWrapper::SpawnRejected {
                    command_id,
                    reason: SpawnRefusal::NegotiationFailed {
                        detail: "superviseur de processus indisponible".to_string(),
                    },
                });
            }
            None
        }
        Ok(SpawnDecision::Await(waiter)) => {
            if let Some(record) = st.managed_spawns.get_mut(&waiter.command_id)
                && !record.requester_conns.iter().any(|id| id == conn_id)
            {
                record.requester_conns.push(conn_id.to_string());
            }
            None
        }
        Ok(SpawnDecision::Accepted { name, definition }) => Some(DaemonToWrapper::SpawnAccepted {
            command_id,
            agent_id: name,
            definition,
        }),
        Ok(SpawnDecision::Rejected(reason)) => {
            Some(DaemonToWrapper::SpawnRejected { command_id, reason })
        }
        Ok(SpawnDecision::EnvelopeMismatch) => Some(DaemonToWrapper::IdempotencyResult {
            project_warnings: Vec::new(),
            operation_kind: "spawn".to_string(),
            idempotency_key: command_id,
            issue: IdempotencyIssue::EnvelopeMismatch,
        }),
        Err(error) => Some(DaemonToWrapper::SpawnRejected {
            command_id,
            reason: SpawnRefusal::NegotiationFailed {
                detail: error.to_string(),
            },
        }),
    }
}

pub(super) fn on_artifact_publish(
    contract_version: u8,
    canonical_publication: Vec<u8>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if contract_version != ARTIFACT_CONTRACT_VERSION {
        return Some(artifact_publication_refusal(
            "invalid_contract",
            "Version de contrat d'artefact non prise en charge.",
        ));
    }
    let publication = match serde_json::from_slice::<ArtifactPublicationV1>(&canonical_publication)
    {
        Ok(publication) if publication.canonical_bytes() == canonical_publication => publication,
        Ok(_) => {
            return Some(artifact_publication_refusal(
                "invalid_payload",
                "La publication d'artefact doit être sérialisée canoniquement.",
            ));
        }
        Err(_) => {
            return Some(artifact_publication_refusal(
                "invalid_payload",
                "La publication d'artefact ne respecte pas le contrat attendu.",
            ));
        }
    };
    let (context, database_path, artifact_root) = {
        let st = state.lock().recover_poison();
        let (agent_name, instance_id, project_id) = match artifact_access_scope(&st, conn_id) {
            Ok(scope) => scope,
            Err(reason) => {
                return Some(artifact_publication_refusal(
                    match reason {
                        ArtifactReadRefusal::ScopeUnavailable => "project_binding_unavailable",
                        ArtifactReadRefusal::StorageUnavailable => "storage_unavailable",
                        _ => "identity_unavailable",
                    },
                    "La connexion ne porte pas une identité, une instance et une portée actives.",
                ));
            }
        };
        let context = ArtifactPublicationContext {
            conversation_reference: format!("conversation:{project_id}:{agent_name}"),
            project_id,
            turn_reference: format!("turn:{}:{}", instance_id, publication.idempotency_key),
            created_by: format!("agent:{agent_name}"),
            origin_instance: format!("instance:{instance_id}"),
            observed_at: unix_timestamp(),
        };
        (context, st.db_path.clone(), st.artifact_root.clone())
    };
    let result = ArtifactService::open(&database_path, &artifact_root, ArtifactPolicy::default())
        .and_then(|mut service| service.publish(context, publication));
    Some(match result {
        Ok(ArtifactPersistResult::Created(receipt)) => artifact_publication_success(receipt, false),
        Ok(ArtifactPersistResult::Replayed(receipt)) => artifact_publication_success(receipt, true),
        Err(ArtifactServiceError::PolicyBlocked) => artifact_publication_refusal(
            "publication_capacity_reached",
            "Le quota de contenus publiés est atteint. Aucun contenu existant n'a été supprimé.",
        ),
        Err(ArtifactServiceError::BlobUnavailable) => artifact_publication_refusal(
            "blob_unavailable",
            "Le contenu binaire annoncé n'est pas présent dans le magasin privé Bridget.",
        ),
        Err(ArtifactServiceError::Interrupted) => artifact_publication_refusal(
            "publication_interrupted",
            "La publication a été interrompue avant validation. Aucun artefact partiel n'a été créé.",
        ),
        Err(ArtifactServiceError::Validation(_)) => artifact_publication_refusal(
            "publication_refused",
            "La publication ne respecte pas les règles d'artefact et de provenance.",
        ),
        Err(ArtifactServiceError::Store(_) | ArtifactServiceError::BlobStore(_)) => {
            artifact_publication_refusal(
                "storage_unavailable",
                "Bridget ne peut pas enregistrer cet artefact pour le moment.",
            )
        }
    })
}

pub(super) fn on_guichet_reply(
    version: u16,
    issuer_scope: String,
    request_id: String,
    claim_generation: u64,
    claim_token: String,
    response_message_id: String,
    in_reply_to: String,
    outcome: bridget_transport::protocol::GuichetOutcome,
    payload: bridget_transport::protocol::GuichetReplyPayload,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if version != SERVICE_CONTRACT_VERSION
        || !guichet_reply_is_valid(&request_id, &response_message_id, &in_reply_to, &payload)
    {
        return Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::InvalidEnvelope,
        });
    }
    let canonical = match encode(&WrapperToDaemon::GuichetReply {
        version,
        issuer_scope: issuer_scope.clone(),
        request_id: request_id.clone(),
        claim_generation,
        claim_token: claim_token.clone(),
        response_message_id: response_message_id.clone(),
        in_reply_to: in_reply_to.clone(),
        outcome,
        payload,
    }) {
        Ok(bytes) => bytes.into_bytes(),
        Err(_) => {
            return Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::InvalidEnvelope,
            });
        }
    };
    let mut st = state.lock().recover_poison();
    match st.store.reply_guichet(
        conn_id,
        GuichetReplyInput {
            issuer_scope: &issuer_scope,
            request_id: &request_id,
            generation: claim_generation,
            token: &claim_token,
            response_message_id: &response_message_id,
            reply_bytes: &canonical,
            in_reply_to: &in_reply_to,
            outcome,
        },
        unix_timestamp(),
    ) {
        Ok(result) => {
            if matches!(&result, GuichetResult::Terminal { issue, newly_finalized: true, .. } if issue == "accepted")
            {
                st.delegation_store.wait_epoch().bump();
                match st.store.guichet_lifecycle_events() {
                    Ok(events) => {
                        if let Some(event) = events.into_iter().find(|event| {
                            event.issuer_scope == issuer_scope && event.request_id == request_id
                        }) && let Some(writer) = st.connections.get(conn_id).cloned()
                        {
                            st.pending_post_response_controls
                                .entry(conn_id.to_string())
                                .or_default()
                                .push(DeferredControl {
                                    writer,
                                    message: guichet_lifecycle_response(event),
                                });
                        }
                    }
                    Err(error) => error!("lecture événement guichet: {error}"),
                }
            }
            Some(guichet_result_response(issuer_scope, request_id, result))
        }
        Err(error) => {
            error!("réponse guichet: {error}");
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::TransitionInvalid,
            })
        }
    }
}

pub(super) fn on_queue_edit_request(
    request_id: String,
    target: String,
    action: bridget_transport::protocol::QueueEditAction,
    message_id: String,
    before_message_id: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut controls = Vec::new();
    let (response, notice) = {
        let mut st = state.lock().recover_poison();
        handle_queue_edit(
            &mut st,
            conn_id,
            request_id,
            target,
            action,
            message_id,
            before_message_id,
            &mut controls,
        )
    };
    // Le COMMIT et le relâchement de State précèdent toute poussée. Rejeu : zéro contrôle.
    let failed_controls = execute_controls(controls);
    if !failed_controls.is_empty() {
        let mut st = state.lock().recover_poison();
        for control in failed_controls {
            if let DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                recipient_instance_id,
                delivery_generation,
                ..
            } = control.message
                && let Err(error) = st.idempotency.mark_delivery_indeterminate(
                    &delivery_id,
                    &recipient_instance_id,
                    delivery_generation,
                )
            {
                warn!("poussée de promotion non confirmée: {error}");
            }
        }
    }
    // Le retrait est committé et State relâché avant le writer. Rejeu: aucune notice fresh.
    if let Some(notice) = notice {
        use rusqlite::OptionalExtension;
        let pending = {
            let st = state.lock().recover_poison();
            let body = st.store.connection().query_row(
                        "SELECT body FROM orphan_emitter_notices WHERE delivery_id=?1 AND notified_at IS NULL",
                        [&notice.delivery_id], |row| row.get::<_, String>(0),
                    ).optional();
            match body {
                Ok(Some(body)) => st
                    .router
                    .get_agent(&notice.sender)
                    .and_then(|route| st.connections.get(&route.connection_id))
                    .cloned()
                    .map(|writer| (writer, body)),
                Ok(None) => None,
                Err(error) => {
                    warn!("notice d'édition reste durable: {error}");
                    None
                }
            }
        };
        if let Some((writer, body)) = pending {
            if let Err(error) = deliver_to_agent(&writer, &notice.sender, &body) {
                warn!("notice d'édition différée: {error}");
            } else if let Err(error) = state
                .lock()
                .recover_poison()
                .idempotency
                .mark_orphan_emitter_notified(&notice.delivery_id, unix_now_secs())
            {
                warn!("accusé de notice d'édition refusé: {error}");
            }
        }
    }
    Some(response)
}

pub(super) fn on_observation_request(
    request: bridget_transport::protocol::ObservationRequest,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    let Some((owner, _)) = live_connection_identity(&st, conn_id) else {
        return Some(DaemonToWrapper::Nack {
            id: "observation".into(),
            reason: "identité active requise".into(),
        });
    };
    if let bridget_transport::protocol::ObservationRequest::Sub {
        agent: Some(ref agent),
        ..
    } = request
    {
        let reason = if st.router.get_agent(agent).is_none() {
            Some(if st.presences.values().any(|p| p.name == *agent) {
                "source_unavailable"
            } else {
                "agent_not_found"
            })
        } else {
            None
        };
        if let Some(reason) = reason {
            return Some(DaemonToWrapper::ObservationResult {
                result: serde_json::json!({"status":"rejected","reason":reason}),
            });
        }
    }
    if let bridget_transport::protocol::ObservationRequest::Missed { limit, after_seq } = &request {
        return Some(DaemonToWrapper::ObservationResult {
            result: st
                .store
                .observation_missed_page(&owner, *limit, *after_seq, unix_now_secs())
                .unwrap_or_else(
                    |_| serde_json::json!({"status":"error","code":"storage_unavailable"}),
                ),
        });
    }
    let before = st.observations.subscriptions.clone();
    let revision = st.observations.revision;
    let mut result = st.observations.request(
        &owner,
        request,
        Instant::now(),
        unix_now_secs().max(0) as u64,
    );
    if st.observations.revision != revision
        && st
            .observations
            .snapshot()
            .ok()
            .is_none_or(|bytes| st.store.save_observation_snapshot(&bytes).is_err())
    {
        st.observations.subscriptions = before;
        st.observations.revision = revision;
        return Some(DaemonToWrapper::ObservationResult {
            result: serde_json::json!({"status":"rejected","reason":"subscription_persistence_failed"}),
        });
    }
    result["sources"] = st.observations.catalogue();
    result["facts_lost"] = serde_json::json!(st.observations.facts_lost);
    result["observation_gaps"] = serde_json::json!(st.observations.observation_gaps);
    result["coverage"] = serde_json::json!(
        "partial: declared compatible live sources only; no historical replay; T3: latest turn only, activity window 500 before compression, at most 12 paths per activity; Claude T3 file writes unsupported"
    );
    result["daemon_instance"] = serde_json::json!(st.instance_id);
    result["lifetime"] = serde_json::json!(
        "subscriptions resumed automatically after restart: facts during the outage are lost, not replayed; no durable delivery, DND respected"
    );
    result["notifications_lost"] = serde_json::json!(
        st.observation_lost
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    result["evicted_writes"] = serde_json::json!(st.observations.evicted_writes);
    Some(DaemonToWrapper::ObservationResult { result })
}

pub(super) fn on_subscribe(
    agent: String,
    window: bridget_transport::protocol::AttachWindow,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (subscription_id, control) = {
        let mut st = state.lock().recover_poison();
        if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Attach) {
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::MessageOutsideAttachRole,
                mode: None,
                location: None,
            });
        }
        let wrapper_conn = match attach_refusal_for_subscription(&st, &agent) {
            Ok(connection_id) => connection_id,
            Err(refusal) => {
                return Some(DaemonToWrapper::AttachRejected {
                    subscription_id: None,
                    reason: refusal.reason,
                    mode: refusal.mode,
                    location: refusal.location,
                });
            }
        };
        let subscription_id = format!("attach-{}", uuid::Uuid::new_v4());
        let Some(view) = st.connections.get(conn_id).and_then(|writer| {
            AttachView::start(subscription_id.clone(), writer, st.view_closed_tx.clone())
        }) else {
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: Some(subscription_id),
                reason: AttachRefusal::WrapperUnavailable,
                mode: None,
                location: None,
            });
        };
        let writer = st.connections.get(&wrapper_conn).cloned();
        st.attach_subscriptions.insert(
            subscription_id.clone(),
            AttachSubscription {
                agent: agent.clone(),
                attach_conn: conn_id.to_string(),
                wrapper_conn,
                caught_up: false,
            },
        );
        st.attach_views.insert(subscription_id.clone(), view);
        let control = writer.map(|writer| DeferredControl {
            writer,
            message: DaemonToWrapper::Subscribe {
                subscription_id: subscription_id.clone(),
                agent,
                window,
            },
        });
        (subscription_id, control)
    };
    let accepted = control.is_some_and(|control| execute_controls(vec![control]).is_empty());
    if accepted {
        None
    } else {
        let view = {
            let mut st = state.lock().recover_poison();
            st.attach_subscriptions.remove(&subscription_id);
            st.attach_views.remove(&subscription_id)
        };
        if let Some(view) = view {
            view.close_and_join();
        }
        Some(DaemonToWrapper::AttachRejected {
            subscription_id: Some(subscription_id),
            reason: AttachRefusal::WrapperUnavailable,
            mode: None,
            location: None,
        })
    }
}

pub(super) fn on_display_name_set(
    request: bridget_transport::protocol::DisplayNameRequest,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    use crate::agent_profile::{AgentProfileError, AgentProfileStore};
    use bridget_transport::protocol::{
        DisplayNameOutcome as Outcome, DisplayNameRefusal as Refusal,
    };
    let mut st = state.lock().recover_poison();
    let mut renamed = HashSet::new();
    let result = (|| {
        if request.version != 1
            || request.display_name.is_empty()
            || request.display_name.chars().count() > crate::agent_profile::MAX_DISPLAY_NAME_CHARS
            || request.display_name.chars().any(char::is_control)
        {
            return Err(Refusal::InvalidRequest);
        }
        let (agent, _) =
            live_connection_identity(&st, conn_id).ok_or(Refusal::IdentityUnavailable)?;
        let mut profiles =
            AgentProfileStore::open(&st.db_path).map_err(|_| Refusal::StorageUnavailable)?;
        let previous = profiles
            .profile_for_agent_id(&agent)
            .map_err(|_| Refusal::StorageUnavailable)?
            .ok_or(Refusal::IdentityUnavailable)?;
        let displaced = profiles
            .agent_id_for_display_name(&request.display_name)
            .map_err(|_| Refusal::StorageUnavailable)?
            .filter(|holder| holder != &agent)
            .map(|holder| profiles.profile_for_agent_id(&holder))
            .transpose()
            .map_err(|_| Refusal::StorageUnavailable)?
            .flatten();
        let profile = profiles
            // Session 110 : un nom détenu par une identité absente de
            // l'annuaire vivant est transféré ; un détenteur vivant
            // conserve le sien.
            .rename_display_name(&agent, &request.display_name, |holder| {
                st.router
                    .get_agent(holder)
                    .is_some_and(|route| st.connections.contains_key(&route.connection_id))
            })
            .map_err(|error| match error {
                AgentProfileError::Invalid(_) => Refusal::InvalidRequest,
                AgentProfileError::NotFound => Refusal::IdentityUnavailable,
                AgentProfileError::DisplayNameConflict => Refusal::NameConflict,
                AgentProfileError::RevisionConflict => Refusal::RevisionConflict,
                AgentProfileError::Sqlite(_) => Refusal::StorageUnavailable,
            })?;
        if previous.display_name != profile.display_name {
            renamed.insert(agent.clone());
        }
        if let Some(before) = displaced
            && profiles
                .profile_for_agent_id(&before.agent_id)
                .map_err(|_| Refusal::StorageUnavailable)?
                .is_some_and(|after| after.display_name != before.display_name)
        {
            renamed.insert(before.agent_id);
        }
        Ok(Outcome::Applied {
            agent_id: profile.agent_id,
            display_name: profile.display_name,
            revision: profile.revision,
        })
    })();
    if !renamed.is_empty() {
        publish_human_name_change(&mut st, &renamed);
    }
    Some(DaemonToWrapper::DisplayNameResult {
        outcome: result.unwrap_or_else(|reason| Outcome::Rejected { reason }),
    })
}

pub(super) fn on_thread_request(
    request: bridget_transport::protocol::ThreadRequest,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    if matches!(
        request.request,
        bridget_transport::protocol::ThreadAction::AddMembers { .. }
    ) {
        let st = state.lock().recover_poison();
        if !st
            .client_negotiations
            .get(conn_id)
            .is_some_and(|negotiated| {
                negotiated
                    .capabilities
                    .contains(&ClientCapability::ThreadMembersV1)
            })
        {
            return Some(DaemonToWrapper::ThreadResult {
                result: bridget_transport::protocol::ThreadResult {
                    version: bridget_transport::protocol::THREAD_CONTRACT_VERSION,
                    result: crate::threads::error(
                        "thread_members_unsupported",
                        "Capacité d'ajout de membres non négociée.",
                        false,
                    ),
                },
            });
        }
    }
    // L'ajout de lecteurs invalide seulement la vue humaine après
    // commit ; il ne dispatch même pas une ancienne intention pending.
    let mutates = matches!(
        request.request,
        bridget_transport::protocol::ThreadAction::Post { .. }
            | bridget_transport::protocol::ThreadAction::Ack { .. }
            | bridget_transport::protocol::ThreadAction::Close { .. }
    );
    let (result, controls) = {
        let mut st = state.lock().recover_poison();
        let Some((owner, _)) = live_connection_identity(&st, conn_id) else {
            return Some(DaemonToWrapper::ThreadResult {
                result: bridget_transport::protocol::ThreadResult {
                    version: bridget_transport::protocol::THREAD_CONTRACT_VERSION,
                    result: crate::threads::error(
                        "identity_unavailable",
                        "identité active requise",
                        false,
                    ),
                },
            });
        };
        let now = unix_now_secs();
        let (result, changed) = {
            let directory = DaemonDirectory { state: &st };
            crate::threads::handle_with_change(&st.store, &directory, &owner, request, now)
        };
        if let Some(thread_id) = changed {
            publish_human_thread_change(&mut st, &thread_id);
        }
        let mut controls = Vec::new();
        // Un dépôt, une confirmation ou une clôture réévaluent les
        // sollicitations tout de suite ; la poussée part hors verrou.
        if mutates && result.result["status"] != "error" {
            dispatch_thread_wakes(&mut st, now, Instant::now(), &mut controls);
        }
        (result, controls)
    };
    let _ = execute_controls(controls);
    Some(DaemonToWrapper::ThreadResult { result })
}

pub(super) fn on_delegated_runtime_event(
    execution_id: String,
    kind: bridget_transport::protocol::DelegatedRuntimeEventKind,
    code: String,
    reference: String,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (child_instance_id, fleet) = {
        let st = state.lock().recover_poison();
        let Some(child_instance_id) = st.conn_instances.get(conn_id).cloned() else {
            return Some(DaemonToWrapper::Nack {
                id: "delegated-runtime-event".to_string(),
                reason: "wrapper non enregistré".to_string(),
            });
        };
        (child_instance_id, Arc::clone(&st.fleet))
    };
    let event = match fleet.record_delegated_runtime_event(DelegatedRuntimeEventInput {
        child_instance_id,
        child_execution_id: execution_id,
        kind,
        code,
        reference,
        observed_at: unix_now_secs(),
    }) {
        Ok(event) => event,
        Err(error) => {
            warn!("fait runtime délégué ignoré: {error}");
            return Some(DaemonToWrapper::Nack {
                id: "delegated-runtime-event".to_string(),
                reason: "fait runtime délégué invalide".to_string(),
            });
        }
    };
    let parent_writer = {
        let st = state.lock().recover_poison();
        st.conn_instances
            .iter()
            .find(|(candidate_conn, candidate_instance)| {
                **candidate_instance == event.parent_instance_id
                    && !st.auxiliary_connections.contains(*candidate_conn)
            })
            .and_then(|(candidate_conn, _)| st.connections.get(candidate_conn))
            .cloned()
    };
    if let Some(writer) = parent_writer
        && !push_control_message(
            &writer,
            &DaemonToWrapper::DelegatedRuntimeEvent {
                event: DelegatedRuntimeEventFrame {
                    cursor: event.cursor,
                    event_id: event.event_id,
                    link_id: event.link_id,
                    child_instance_id: event.child_instance_id,
                    child_execution_id: event.child_execution_id,
                    kind: event.kind,
                    code: event.code,
                    reference: event.reference,
                    observed_at: event.observed_at,
                    project: event.project,
                },
            },
        )
    {
        warn!("remise du fait runtime délégué différée: parent inaccessible");
    }
    None
}

pub(super) fn on_observed_activity(
    seq: u64,
    event: bridget_transport::protocol::ObservationKind,
    file: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::ObservationKind;
    let mut st = state.lock().recover_poison();
    let (agent, instance) = live_connection_identity(&st, conn_id)?;
    if st.auxiliary_connections.contains(conn_id)
        || !st.observations.accepts(conn_id, event)
        || seq == 0
        || st
            .observation_sequences
            .get(conn_id)
            .is_some_and(|last| seq <= *last)
        || !matches!(
            event,
            ObservationKind::FileWritten
                | ObservationKind::PermissionRequired
                | ObservationKind::TurnEnded
        )
        || (event == ObservationKind::FileWritten && file.is_none())
        || (event != ObservationKind::FileWritten && file.is_some())
    {
        return None;
    }
    st.observation_sequences.insert(conn_id.into(), seq);
    if event == ObservationKind::TurnEnded {
        let agent = agent.clone();
        let instance = instance.clone();
        drop(st);
        specialist_requests::observe_history(state, &agent, &instance, seq);
        st = state.lock().recover_poison();
        if live_connection_identity(&st, conn_id) != Some((agent, instance)) {
            return None;
        }
    }
    let host = st
        .presences
        .get(&instance)
        .map(|p| p.host.clone())
        .unwrap_or_else(|| st.host.clone());
    observe_fact(
        &mut st,
        crate::observation::Fact {
            event,
            agent,
            host,
            file,
            other_agent: None,
        },
    );
    None
}

pub(super) fn on_display_name_resolve(
    request: bridget_transport::protocol::DisplayNameRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    use crate::agent_profile::AgentProfileStore;
    use bridget_transport::protocol::{
        DisplayNameRefusal as Refusal, DisplayNameResolution as Outcome,
    };
    let st = state.lock().recover_poison();
    let outcome = if request.version != 1
        || request.display_name.chars().any(char::is_control)
        || request.display_name.trim().is_empty()
        || request.display_name.chars().count() > crate::agent_profile::MAX_DISPLAY_NAME_CHARS
    {
        Outcome::Rejected {
            reason: Refusal::InvalidRequest,
        }
    } else {
        match AgentProfileStore::open(&st.db_path)
            .and_then(|profiles| profiles.agent_id_for_display_name(&request.display_name))
        {
            Ok(Some(agent_id)) => {
                let active = st
                    .router
                    .list_agents()
                    .iter()
                    .any(|agent| agent.agent_id == agent_id);
                Outcome::Found { agent_id, active }
            }
            Ok(None) => Outcome::NotFound,
            Err(_) => Outcome::Rejected {
                reason: Refusal::StorageUnavailable,
            },
        }
    };
    Some(DaemonToWrapper::DisplayNameResolved { outcome })
}

pub(super) fn on_directory_scoped(
    scope: bridget_transport::protocol::CommunicationDirectoryScope,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    use bridget_transport::protocol::{CommunicationDirectoryScope, ProjectRelation};
    let mut st = state.lock().recover_poison();
    let project = connection_communication_project(&st, conn_id);
    let all = match agent_tracking::directory(&mut st) {
        Ok(all) => all,
        Err(reason) => {
            return Some(DaemonToWrapper::Nack {
                id: "directory".into(),
                reason: reason.into(),
            });
        }
    };
    let agents = all
        .into_iter()
        .filter_map(|mut scoped| {
            let communication_project = scoped.communication_project.clone();
            let project_relation = crate::communication::project_relation(
                project.as_ref(),
                communication_project.as_ref(),
            );
            (scope == CommunicationDirectoryScope::Global
                || project_relation == ProjectRelation::Same)
                .then(|| {
                    scoped.project_relation = project_relation;
                    scoped
                })
        })
        .collect();
    let project_warnings = if project.is_none() {
        crate::communication::project_scope(None, None, "", None).unwrap_or_default()
    } else {
        Vec::new()
    };
    Some(DaemonToWrapper::CommunicationDirectory {
        agents,
        project,
        project_warnings,
    })
}

pub(super) fn on_native_permission_fact(
    fact: Option<ProviderPermissions>,
    request_id: Option<String>,
    observation_id: Option<String>,
    unavailable_code: Option<String>,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut st = state.lock().recover_poison();
    if st.auxiliary_connections.contains(conn_id)
        || live_connection_identity(&st, conn_id).is_none()
    {
        return None;
    }
    let allowed = fact.as_ref().is_some_and(|fact| {
        live_connection_identity(&st, conn_id).is_some_and(|(_, instance)| {
            fact.provider_instance_id == instance
                && st.presences.get(&instance).is_some_and(|p| {
                    matches!(
                        p.transport.as_str(),
                        "codex_app_server" | "claude_stream_json" | "stdio" | "claude_pty" | "tmux"
                    ) && st
                        .registry
                        .get(&p.agent_type)
                        .is_ok_and(|definition| definition.protocol == fact.driver)
                })
        }) && request_id
            .as_ref()
            .is_none_or(|id| !id.is_empty() && id.len() <= 128 && !id.chars().any(char::is_control))
            && st
                .native_permission_facts
                .get(conn_id)
                .is_none_or(|(previous, _)| {
                    fact.provider_session_id != previous.provider_session_id
                        || fact.revision >= previous.revision
                })
            && bridget_transport::protocol::validate_permissions(fact, false).is_ok()
    });
    let accepted = allowed && unavailable_code.is_none();
    if accepted {
        st.native_permission_facts
            .insert(conn_id.into(), (fact.unwrap(), request_id));
        st.native_permission_refusals.remove(conn_id);
    } else {
        st.native_permission_facts.remove(conn_id);
        let code = unavailable_code
            .filter(|code| {
                matches!(
                    code.as_str(),
                    "permission_source_unavailable" | "settings_revision_changed"
                )
            })
            .unwrap_or_else(|| "permission_attestation_unavailable".into());
        st.native_permission_refusals.insert(conn_id.into(), code);
    }
    observation_id
        .filter(|id| Uuid::parse_str(id).is_ok())
        .map(
            |observation_id| DaemonToWrapper::NativePermissionAcknowledged {
                observation_id,
                accepted,
            },
        )
}

pub(super) fn service_refusal(
    msg: &WrapperToDaemon,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<ServiceRefusal> {
    let st = state.lock().recover_poison();
    let is_service_message = matches!(
        &msg,
        WrapperToDaemon::ServiceHello { .. }
            | WrapperToDaemon::ProjectRegistryRequest { .. }
            | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
            | WrapperToDaemon::ProjectSystemRequest { .. }
            | WrapperToDaemon::ProjectProfileRequest { .. }
            | WrapperToDaemon::CoordinationSubscribe { .. }
            | WrapperToDaemon::GuichetClaimNext { .. }
            | WrapperToDaemon::GuichetClaim { .. }
            | WrapperToDaemon::GuichetLookup { .. }
            | WrapperToDaemon::GuichetReply { .. }
            | WrapperToDaemon::HumanInboxDeposit { .. }
            | WrapperToDaemon::ControlFocusPublish { .. }
            | WrapperToDaemon::HumanInboxDecisions { .. }
            | WrapperToDaemon::HumanInboxAck { .. }
            | WrapperToDaemon::HumanInboxClose { .. }
    );
    match st.connection_roles.get(conn_id) {
        Some(ConnectionRole::Service) => match &msg {
                WrapperToDaemon::AgentTracking { .. } | WrapperToDaemon::Specialists { .. } | WrapperToDaemon::MobileAdministration { .. } | WrapperToDaemon::MobileOperation { .. } => Some(ServiceRefusal::MessageOutsideServiceRole),
                WrapperToDaemon::WorkerHello { .. }
                | WrapperToDaemon::WorkerHelloWithCapabilities { .. }
                | WrapperToDaemon::WorkerResponse { .. }
                | WrapperToDaemon::CandidateProbeResult { .. }
                | WrapperToDaemon::CandidateProbeResultV2 { .. }
                | WrapperToDaemon::ProviderCatalogueUpdate { .. }
                | WrapperToDaemon::OriginProofResponse { .. } =>
                    Some(ServiceRefusal::MessageOutsideServiceRole),
                WrapperToDaemon::SelectRuntime { .. } | WrapperToDaemon::RuntimeSelectionReported { .. } | WrapperToDaemon::RuntimeCatalogueReported { .. } => Some(ServiceRefusal::MessageOutsideServiceRole),
                WrapperToDaemon::HumanInboxDeposit { .. }
                | WrapperToDaemon::ControlFocusPublish { .. }
                | WrapperToDaemon::HumanInboxDecisions { .. }
                | WrapperToDaemon::HumanInboxAck { .. }
                | WrapperToDaemon::HumanInboxClose { .. }
                    if !st.service_negotiations.contains_key(conn_id) =>
                {
                    Some(ServiceRefusal::NegotiationRequired)
                }
                WrapperToDaemon::HumanInboxDeposit { .. }
                | WrapperToDaemon::ControlFocusPublish { .. }
                | WrapperToDaemon::HumanInboxDecisions { .. }
                | WrapperToDaemon::HumanInboxAck { .. }
                | WrapperToDaemon::HumanInboxClose { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated
                                .capabilities
                                .contains(&ServiceCapability::HumanInboxV1)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::HumanInboxDeposit { .. }
                | WrapperToDaemon::ControlFocusPublish { .. }
                | WrapperToDaemon::HumanInboxDecisions { .. }
                | WrapperToDaemon::HumanInboxAck { .. }
                | WrapperToDaemon::HumanInboxClose { .. } => None,
                WrapperToDaemon::ServiceHello { .. }
                    if st.service_negotiations.contains_key(conn_id) =>
                {
                    Some(ServiceRefusal::AlreadyNegotiated)
                }
                WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::ProjectRegistryRequest { .. }
                | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
                | WrapperToDaemon::ProjectSystemRequest { .. }
                | WrapperToDaemon::ProjectProfileRequest { .. }
                | WrapperToDaemon::ProjectRuntimeRequest { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                    if !st.service_negotiations.contains_key(conn_id) =>
                {
                    Some(ServiceRefusal::NegotiationRequired)
                }
                WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::GuichetV1)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::CoordinationSubscribe { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::CoordinationEventsV2)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::ProjectRegistryRequest { .. }
                | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
                | WrapperToDaemon::ProjectSystemRequest { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::ProjectRegistryV1)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::ProjectProfileRequest { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::ProjectProfilesV1)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                // MATRICE EXHAUSTIVE — même raison que pour le rôle Client, et
                // conséquence PIRE ici. Un message non classé y était refusé en
                // silence : une sonde d'identité posée sur ce rôle rendrait
                // « non attesté », une garde en conclurait « ne pas écrire », et
                // le refus serait PERMANENT. Une garde qui refuse toujours
                // ressemble à une garde qui marche — elle passe la revue verte
                // et ne protège rien.
                WrapperToDaemon::ServiceHello { .. }
                | WrapperToDaemon::ProjectRegistryRequest { .. }
                | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
                | WrapperToDaemon::ProjectSystemRequest { .. }
                | WrapperToDaemon::ProjectProfileRequest { .. }
                | WrapperToDaemon::ProjectRuntimeRequest { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                | WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::Heartbeat
                // Sonde d'identité : lecture seule, aucune écriture durable. Le
                // greffe en a besoin pour savoir SUR QUELLE MACHINE il écrirait.
                | WrapperToDaemon::SentinelStatusRead
                | WrapperToDaemon::DaemonIdentityRequest => None,
                WrapperToDaemon::RoleHandshake { .. }
                | WrapperToDaemon::ChildRequest { .. }
                | WrapperToDaemon::ChildRequestAnswered { .. }
                | WrapperToDaemon::ChildRequestClosed { .. }
                | WrapperToDaemon::UserInput { .. }
                | WrapperToDaemon::NativeDelegation { .. }
                | WrapperToDaemon::NativeDelegationT3 { .. }
                | WrapperToDaemon::RequestWait { .. }
                | WrapperToDaemon::NativePermissionFact { .. }
                | WrapperToDaemon::CommunicationProjectFact { .. }
                | WrapperToDaemon::DirectoryScoped { .. }
                | WrapperToDaemon::ObservationRequest { .. }
                | WrapperToDaemon::ThreadRequest { .. }
                | WrapperToDaemon::HumanLineage { .. }
                | WrapperToDaemon::HumanThreadViewV1 { .. }
                | WrapperToDaemon::HumanThreadWatchV1 { .. }
                | WrapperToDaemon::T3ThreadBindingFact { .. }
                | WrapperToDaemon::ThreadNoticeCapability { .. }
                | WrapperToDaemon::LedgerSearch { .. }
                | WrapperToDaemon::LedgerRead { .. }
                | WrapperToDaemon::ObservedActivity { .. }
                | WrapperToDaemon::ObservationCapabilities { .. }
                | WrapperToDaemon::ObservationGap { .. }
                | WrapperToDaemon::ClientHello { .. }
                | WrapperToDaemon::ArtifactPublish { .. }
                | WrapperToDaemon::ArtifactRead { .. }
                | WrapperToDaemon::ProjectRoundRequest { .. }
                | WrapperToDaemon::ProjectRoundDispatch { .. }
                | WrapperToDaemon::RuntimeIngressHello { .. }
                | WrapperToDaemon::RuntimeIngressPreflight { .. }
                // Refusé AVANT ce lot aussi : il tombait dans le tiret bas.
                // La forme change, le sort de ce message ne change pas.
                | WrapperToDaemon::ServiceRequest { .. }
                | WrapperToDaemon::SendIdempotent { .. }
                | WrapperToDaemon::Lookup { .. }
                | WrapperToDaemon::QueueList { .. }
                | WrapperToDaemon::QueueEditRequest { .. }
                | WrapperToDaemon::ControlExecution { .. }
                | WrapperToDaemon::ControlExecutionReported { .. }
                | WrapperToDaemon::DelegatedRuntimeEvent { .. }
                | WrapperToDaemon::DelegatedRuntimeEventAcknowledged { .. }
                | WrapperToDaemon::DeliverAcked { .. }
                | WrapperToDaemon::DeliveryIndeterminate { .. }
                | WrapperToDaemon::SpawnOrder { .. }
                | WrapperToDaemon::WaitAgentLinks { .. }
                | WrapperToDaemon::StopOrder { .. }
                | WrapperToDaemon::RelaunchOrder { .. }
                | WrapperToDaemon::DecommissionOrder { .. }
                | WrapperToDaemon::AdoptStoppedOrder { .. }
                | WrapperToDaemon::Subscribe { .. }
                | WrapperToDaemon::Unsubscribe { .. }
                | WrapperToDaemon::Subscribed { .. }
                | WrapperToDaemon::JournalFragment { .. }
                | WrapperToDaemon::LiveJournalFragment { .. }
                | WrapperToDaemon::SnapshotCaughtUp { .. }
                | WrapperToDaemon::Gap { .. }
                | WrapperToDaemon::JournalReadError { .. }
                | WrapperToDaemon::End { .. }
                | WrapperToDaemon::AttachRejected { .. }
                | WrapperToDaemon::RegisterAuxiliary { .. }
                | WrapperToDaemon::Register { .. }
                | WrapperToDaemon::DiskSpace { .. }
                | WrapperToDaemon::JournalReady { .. }
                | WrapperToDaemon::TerminalSessionReady
                | WrapperToDaemon::Unregister
                | WrapperToDaemon::Send { .. }
                | WrapperToDaemon::DeliveryRejected { .. }
                | WrapperToDaemon::TurnState { .. }
                | WrapperToDaemon::ExecutionStateChanged { .. }
                | WrapperToDaemon::WorkspaceRepairResult { .. }
                | WrapperToDaemon::ExecutionProviderObserved { .. }
                | WrapperToDaemon::CancelRequest { .. }
                | WrapperToDaemon::ListRequests { .. }
                | WrapperToDaemon::LedgerProjection { .. }
                | WrapperToDaemon::ListAgents
                | WrapperToDaemon::DisplayNameSet { .. }
                | WrapperToDaemon::DisplayNameResolve { .. }
                | WrapperToDaemon::Runtime { .. }
                | WrapperToDaemon::ServedModel { .. }
                | WrapperToDaemon::RateLimit { .. }
                | WrapperToDaemon::Usage { .. }
                | WrapperToDaemon::UsageWindow { .. }
                | WrapperToDaemon::Domain { .. }
                | WrapperToDaemon::Availability { .. }
                | WrapperToDaemon::ControlStateRead { .. }
                | WrapperToDaemon::DelegationOverviewRead { .. }
                | WrapperToDaemon::DelegationHaltSet { .. }
                | WrapperToDaemon::DelegationCancel { .. }
                | WrapperToDaemon::WakeRequest { .. }

                | WrapperToDaemon::ControlFocusRead { .. }
                | WrapperToDaemon::ControlHistory { .. }
                | WrapperToDaemon::ControlStateSet { .. }
                | WrapperToDaemon::HumanInboxList { .. }
                | WrapperToDaemon::HumanInboxResolve { .. } => {
                    Some(ServiceRefusal::MessageOutsideServiceRole)
                }
            },
        Some(ConnectionRole::Wrapper) | None if is_service_message => {
            Some(ServiceRefusal::ServiceRoleRequired)
        }
        _ => None,
    }
}

pub(super) fn client_refusal(
    msg: &WrapperToDaemon,
    conn_id: &str,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<ClientRefusal> {
    let st = state.lock().recover_poison();
    match st.connection_roles.get(conn_id) {
        Some(ConnectionRole::Attach)
            if matches!(&msg, WrapperToDaemon::ClientHello { .. })
                && st.client_negotiations.contains_key(conn_id) =>
        {
            Some(ClientRefusal::AlreadyNegotiated)
        }
        Some(ConnectionRole::Attach)
            if matches!(&msg, WrapperToDaemon::QueueList { .. })
                && !st.client_negotiations.contains_key(conn_id) =>
        {
            Some(ClientRefusal::NegotiationRequired)
        }
        Some(ConnectionRole::Attach)
            if matches!(&msg, WrapperToDaemon::QueueList { .. })
                && !st
                    .client_negotiations
                    .get(conn_id)
                    .is_some_and(|n| n.capabilities.contains(&ClientCapability::QueueListV1)) =>
        {
            Some(ClientRefusal::CapabilityNotNegotiated)
        }
        Some(ConnectionRole::Attach)
            if matches!(&msg, WrapperToDaemon::QueueEditRequest { .. })
                && !st.client_negotiations.contains_key(conn_id) =>
        {
            Some(ClientRefusal::NegotiationRequired)
        }
        Some(ConnectionRole::Attach)
            if matches!(&msg, WrapperToDaemon::QueueEditRequest { .. })
                && !st
                    .client_negotiations
                    .get(conn_id)
                    .is_some_and(|n| n.capabilities.contains(&ClientCapability::QueueEditV1)) =>
        {
            Some(ClientRefusal::CapabilityNotNegotiated)
        }
        Some(ConnectionRole::Client) => match &msg {
                WrapperToDaemon::UserInput { .. } => None,
                WrapperToDaemon::AgentTracking { .. } | WrapperToDaemon::Specialists { .. } | WrapperToDaemon::MobileAdministration { .. } | WrapperToDaemon::MobileOperation { .. } => Some(ClientRefusal::MessageOutsideClientRole),
                WrapperToDaemon::WorkerHello { .. }
                | WrapperToDaemon::WorkerHelloWithCapabilities { .. }
                | WrapperToDaemon::WorkerResponse { .. }
                | WrapperToDaemon::CandidateProbeResult { .. }
                | WrapperToDaemon::CandidateProbeResultV2 { .. }
                | WrapperToDaemon::ProviderCatalogueUpdate { .. }
                | WrapperToDaemon::OriginProofResponse { .. } =>
                    Some(ClientRefusal::MessageOutsideClientRole),
                WrapperToDaemon::QueueEditRequest { .. } if !st.client_negotiations.contains_key(conn_id) => Some(ClientRefusal::NegotiationRequired),
                WrapperToDaemon::QueueEditRequest { .. } if !st.client_negotiations.get(conn_id).is_some_and(|n| n.capabilities.contains(&ClientCapability::QueueEditV1)) => Some(ClientRefusal::CapabilityNotNegotiated),
                WrapperToDaemon::QueueEditRequest { .. } => None,
                WrapperToDaemon::WakeRequest { .. } if !st.client_negotiations.contains_key(conn_id) => Some(ClientRefusal::NegotiationRequired),
                WrapperToDaemon::WakeRequest { .. } => None,
                WrapperToDaemon::QueueList { .. } if !st.client_negotiations.contains_key(conn_id) => Some(ClientRefusal::NegotiationRequired),
                WrapperToDaemon::QueueList { .. } if !st.client_negotiations.get(conn_id).is_some_and(|n| n.capabilities.contains(&ClientCapability::QueueListV1)) => Some(ClientRefusal::CapabilityNotNegotiated),
                WrapperToDaemon::QueueList { .. } => None,
                WrapperToDaemon::CommunicationProjectFact { .. } | WrapperToDaemon::DirectoryScoped { .. } if !st.client_negotiations.get(conn_id).is_some_and(|n| n.capabilities.contains(&ClientCapability::CommunicationProjectsV1)) => Some(ClientRefusal::CapabilityNotNegotiated),
                WrapperToDaemon::CommunicationProjectFact { .. } | WrapperToDaemon::DirectoryScoped { .. } => None,
                WrapperToDaemon::RegisterAuxiliary { .. } => None,
                WrapperToDaemon::SelectRuntime { .. } | WrapperToDaemon::RuntimeSelectionReported { .. } | WrapperToDaemon::RuntimeCatalogueReported { .. } => Some(ClientRefusal::MessageOutsideClientRole),
                WrapperToDaemon::ClientHello { .. }
                    if st.client_negotiations.contains_key(conn_id) =>
                {
                    Some(ClientRefusal::AlreadyNegotiated)
                }
                WrapperToDaemon::ControlStateRead { .. }
                | WrapperToDaemon::ControlFocusRead { .. }
                | WrapperToDaemon::ControlHistory { .. }
                | WrapperToDaemon::ControlStateSet { .. }
                | WrapperToDaemon::HumanInboxList { .. }
                | WrapperToDaemon::HumanInboxResolve { .. }
                | WrapperToDaemon::DelegationOverviewRead { .. }
                | WrapperToDaemon::DelegationHaltSet { .. }
                | WrapperToDaemon::DelegationCancel { .. }

                    if !st.client_negotiations.contains_key(conn_id) =>
                {
                    Some(ClientRefusal::NegotiationRequired)
                }
                WrapperToDaemon::ControlStateRead { .. } | WrapperToDaemon::ControlHistory { .. }
                | WrapperToDaemon::ControlFocusRead { .. }
                    if st.client_negotiations.get(conn_id).is_some_and(|negotiated| {
                        negotiated.version != CLIENT_CONTRACT_VERSION
                            || !(negotiated
                                .capabilities
                                .contains(&ClientCapability::ControlStateV1)
                                || negotiated.capabilities.contains(&ClientCapability::Lookup))
                    }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::ControlStateSet { .. }
                | WrapperToDaemon::HumanInboxList { .. }
                | WrapperToDaemon::HumanInboxResolve { .. }
                | WrapperToDaemon::DelegationOverviewRead { .. }
                | WrapperToDaemon::DelegationHaltSet { .. }
                | WrapperToDaemon::DelegationCancel { .. }

                    if st.client_negotiations.get(conn_id).is_some_and(|negotiated| {
                        negotiated.version != CLIENT_CONTRACT_VERSION
                            || !negotiated
                                .capabilities
                                .contains(&ClientCapability::ControlStateV1)
                    }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::ControlStateRead { .. }
                | WrapperToDaemon::ControlFocusRead { .. }
                | WrapperToDaemon::ControlHistory { .. }
                | WrapperToDaemon::ControlStateSet { .. }
                | WrapperToDaemon::HumanInboxList { .. }
                | WrapperToDaemon::HumanInboxResolve { .. }
                | WrapperToDaemon::DelegationOverviewRead { .. }
                | WrapperToDaemon::DelegationHaltSet { .. }
                | WrapperToDaemon::DelegationCancel { .. }
 => None,
                WrapperToDaemon::SendIdempotent { .. }
                | WrapperToDaemon::Lookup { .. }
                | WrapperToDaemon::ControlExecution { .. }
                | WrapperToDaemon::ProjectRoundRequest { .. }
                | WrapperToDaemon::ProjectRoundDispatch { .. }
                    if !st.client_negotiations.contains_key(conn_id) =>
                {
                    Some(ClientRefusal::NegotiationRequired)
                }
                WrapperToDaemon::SendIdempotent { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated
                                    .capabilities
                                    .contains(&ClientCapability::SendIdempotent)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::Lookup { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated.capabilities.contains(&ClientCapability::Lookup)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::ControlExecution { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated
                                    .capabilities
                                    .contains(&ClientCapability::ExecutionControlV1)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::ProjectRoundRequest { .. }
                | WrapperToDaemon::ProjectRoundDispatch { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated
                                    .capabilities
                                    .contains(&ClientCapability::ProjectRoundPolicyV1)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::SpawnOrder {
                    posture: Some(_), ..
                } => None, // Attribution humaine et capacité contrôlées au puits.
                WrapperToDaemon::NativeDelegation { .. } | WrapperToDaemon::NativeDelegationT3 { .. } | WrapperToDaemon::RequestWait { .. } => None, // Identité et droits au puits natif.
                // MATRICE EXHAUSTIVE — aucun `_`, et c'est délibéré.
                //
                // Le tiret bas précédent classait trois variantes et renvoyait
                // les quarante-deux autres au refus, EN SILENCE. Ajouter une
                // variante au protocole compilait sans rien dire, et le message
                // neuf était rejeté en production sans qu'aucun test unitaire ne
                // puisse le voir : c'est ainsi que `DaemonIdentityRequest` a été
                // livré inatteignable. Désormais, ajouter une variante NE COMPILE
                // PAS tant qu'elle n'est pas classée ici — le compilateur pose la
                // question à la place du relecteur.
                WrapperToDaemon::ClientHello { .. }
                | WrapperToDaemon::SendIdempotent { .. }
                | WrapperToDaemon::Lookup { .. }
                | WrapperToDaemon::ControlExecution { .. }
                | WrapperToDaemon::ProjectRoundRequest { .. }
                | WrapperToDaemon::ProjectRoundDispatch { .. }
                // Sonde d'identité : lecture seule, aucune écriture durable, et
                // c'est le rôle Client qui l'emprunte (`daemon_identity`).
                | WrapperToDaemon::SentinelStatusRead
                | WrapperToDaemon::DaemonIdentityRequest => None,
                WrapperToDaemon::RoleHandshake { .. }
                | WrapperToDaemon::ChildRequest { .. }
                | WrapperToDaemon::ChildRequestAnswered { .. }
                | WrapperToDaemon::ChildRequestClosed { .. }
                | WrapperToDaemon::NativePermissionFact { .. }
                | WrapperToDaemon::ObservationRequest { .. }
                | WrapperToDaemon::ThreadRequest { .. }
                | WrapperToDaemon::HumanLineage { .. }
                | WrapperToDaemon::HumanThreadViewV1 { .. }
                | WrapperToDaemon::HumanThreadWatchV1 { .. }
                | WrapperToDaemon::T3ThreadBindingFact { .. }
                | WrapperToDaemon::ThreadNoticeCapability { .. }
                | WrapperToDaemon::LedgerSearch { .. }
                | WrapperToDaemon::LedgerRead { .. }
                | WrapperToDaemon::ObservedActivity { .. }
                | WrapperToDaemon::ObservationCapabilities { .. }
                | WrapperToDaemon::ObservationGap { .. }
                | WrapperToDaemon::ServiceHello { .. }
                | WrapperToDaemon::ArtifactPublish { .. }
                | WrapperToDaemon::ArtifactRead { .. }
                | WrapperToDaemon::RuntimeIngressHello { .. }
                | WrapperToDaemon::RuntimeIngressPreflight { .. }
                | WrapperToDaemon::ProjectRegistryRequest { .. }
                | WrapperToDaemon::ProjectRegistryAdminRequest { .. }
                | WrapperToDaemon::ProjectSystemRequest { .. }
                | WrapperToDaemon::ProjectProfileRequest { .. }
                | WrapperToDaemon::ProjectRuntimeRequest { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                | WrapperToDaemon::ServiceRequest { .. }
                | WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::ControlExecutionReported { .. }
                | WrapperToDaemon::DelegatedRuntimeEvent { .. }
                | WrapperToDaemon::DelegatedRuntimeEventAcknowledged { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::DeliverAcked { .. }
                | WrapperToDaemon::DeliveryIndeterminate { .. }
                | WrapperToDaemon::SpawnOrder { .. }
                | WrapperToDaemon::WaitAgentLinks { .. }
                | WrapperToDaemon::StopOrder { .. }
                | WrapperToDaemon::RelaunchOrder { .. }
                | WrapperToDaemon::DecommissionOrder { .. }
                | WrapperToDaemon::AdoptStoppedOrder { .. }
                | WrapperToDaemon::Subscribe { .. }
                | WrapperToDaemon::Unsubscribe { .. }
                | WrapperToDaemon::Subscribed { .. }
                | WrapperToDaemon::JournalFragment { .. }
                | WrapperToDaemon::LiveJournalFragment { .. }
                | WrapperToDaemon::SnapshotCaughtUp { .. }
                | WrapperToDaemon::Gap { .. }
                | WrapperToDaemon::JournalReadError { .. }
                | WrapperToDaemon::End { .. }
                | WrapperToDaemon::AttachRejected { .. }
                | WrapperToDaemon::Register { .. }
                | WrapperToDaemon::DiskSpace { .. }
                | WrapperToDaemon::JournalReady { .. }
                | WrapperToDaemon::TerminalSessionReady
                | WrapperToDaemon::Unregister
                | WrapperToDaemon::Send { .. }
                | WrapperToDaemon::DeliveryRejected { .. }
                | WrapperToDaemon::TurnState { .. }
                | WrapperToDaemon::ExecutionStateChanged { .. }
                | WrapperToDaemon::WorkspaceRepairResult { .. }
                | WrapperToDaemon::ExecutionProviderObserved { .. }
                | WrapperToDaemon::CancelRequest { .. }
                | WrapperToDaemon::ListRequests { .. }
                | WrapperToDaemon::LedgerProjection { .. }
                | WrapperToDaemon::Heartbeat
                | WrapperToDaemon::ListAgents
                | WrapperToDaemon::DisplayNameSet { .. }
                | WrapperToDaemon::DisplayNameResolve { .. }
                | WrapperToDaemon::Runtime { .. }
                | WrapperToDaemon::ServedModel { .. }
                | WrapperToDaemon::RateLimit { .. }
                | WrapperToDaemon::Usage { .. }
                | WrapperToDaemon::UsageWindow { .. }
                | WrapperToDaemon::Domain { .. }
                | WrapperToDaemon::Availability { .. }
                | WrapperToDaemon::ControlFocusPublish { .. }
                | WrapperToDaemon::HumanInboxDeposit { .. }
                | WrapperToDaemon::HumanInboxDecisions { .. }
                | WrapperToDaemon::HumanInboxAck { .. }
                | WrapperToDaemon::HumanInboxClose { .. } => {
                    Some(ClientRefusal::MessageOutsideClientRole)
                }
            },
        Some(ConnectionRole::Wrapper) | None
            if matches!(msg, WrapperToDaemon::ClientHello { .. })
                && st.auxiliary_connections.contains(conn_id) =>
        {
            None
        }
        Some(ConnectionRole::Wrapper) | None
            if matches!(
                msg,
                WrapperToDaemon::ClientHello { .. }
                    | WrapperToDaemon::SendIdempotent { .. }
                    | WrapperToDaemon::Lookup { .. }
                    | WrapperToDaemon::QueueList { .. }
                    | WrapperToDaemon::QueueEditRequest { .. }
                    | WrapperToDaemon::ControlExecution { .. }
                    | WrapperToDaemon::ProjectRoundRequest { .. }
                    | WrapperToDaemon::ProjectRoundDispatch { .. }
                    | WrapperToDaemon::ControlStateRead { .. }
                    | WrapperToDaemon::ControlFocusRead { .. }
                    | WrapperToDaemon::ControlHistory { .. }
                    | WrapperToDaemon::ControlStateSet { .. }
                    | WrapperToDaemon::HumanInboxList { .. }
                    | WrapperToDaemon::HumanInboxResolve { .. }
                    | WrapperToDaemon::DelegationOverviewRead { .. }
                    | WrapperToDaemon::DelegationHaltSet { .. }
                    | WrapperToDaemon::DelegationCancel { .. }
            ) =>
        {
            Some(ClientRefusal::ClientRoleRequired)
        }
        _ => None,
    }
}

pub(super) fn on_observation_capabilities(events: Vec<bridget_transport::protocol::ObservationKind>, runtime_catalogue: bool, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let mut st = state.lock().recover_poison();
            let Some((agent, instance)) = live_connection_identity(&st, conn_id) else {
                return Some(DaemonToWrapper::Nack {
                    id: "observation-capabilities".into(),
                    reason: "primary_identity_required".into(),
                });
            };
            if st.auxiliary_connections.contains(conn_id)
                || events.len() > 3
                || events.iter().enumerate().any(|(i, event)| {
                    *event == bridget_transport::protocol::ObservationKind::FileCollision
                        || events[..i].contains(event)
                })
            {
                return Some(DaemonToWrapper::Nack {
                    id: "observation-capabilities".into(),
                    reason: "invalid_primary_capabilities".into(),
                });
            }
            if runtime_catalogue {
                st.runtime_catalogue_connections
                    .insert(conn_id.into(), (agent.clone(), instance));
            } else {
                st.runtime_catalogue_connections.remove(conn_id);
            }
            st.observations
                .set_source(conn_id, &agent, events, Instant::now());
            None
        }

pub(super) fn on_guichet_claim(version: u16, issuer_scope: String, request_id: String, claim_token: String, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let st = state.lock().recover_poison();
            match st.store.claim_guichet(
                conn_id,
                &issuer_scope,
                &request_id,
                &claim_token,
                unix_timestamp(),
            ) {
                Ok(Ok(claim)) => Some(guichet_claim_response(claim)),
                Ok(Err(result)) => Some(guichet_result_response(issuer_scope, request_id, result)),
                Err(error) => {
                    error!("rejeu claim guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }

pub(super) fn on_usage(agent: String, input_tokens: u64, execution_id: Option<String>, execution_generation: Option<u64>, output_tokens: u64, cache_creation_input_tokens: u64, cache_read_input_tokens: u64, provider_kind: Option<String>, source: bridget_transport::protocol::UsageSource, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let mut st = state.lock().recover_poison();
            Some(handle_usage(
                &agent,
                bridget_transport::protocol::UsageTokens {
                    input_tokens,
                    output_tokens,
                    cache_creation_input_tokens,
                    cache_read_input_tokens,
                },
                source,
                provider_kind,
                execution_id,
                execution_generation,
                &mut st,
            ))
        }

pub(super) fn on_send_idempotent(mut message: bridget_core::BridgetMessage, message_id: String, issued_at: i64, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let (response, controls) = {
                let mut st = state.lock().recover_poison();
                let mut controls = Vec::new();
                normalize_guidance_origin(&st, conn_id, &mut message, false);
                let response = handle_idempotent_send(
                    conn_id,
                    message,
                    message_id,
                    issued_at,
                    IdempotentSendAdmission {
                        project: None,
                        issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                    },
                    &mut st,
                    &mut controls,
                );
                (response, controls)
            };
            let _ = execute_controls(controls);
            Some(response)
        }

pub(super) fn on_journal_fragment(subscription_id: String, seq: u64, offset: u64, final_fragment: bool, bytes: Vec<u8>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let st = state.lock().recover_poison();
                st.attach_subscriptions
                    .get(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .and_then(|_| st.attach_views.get(&subscription_id).cloned())
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::JournalFragment {
                    subscription_id,
                    seq,
                    offset,
                    final_fragment,
                    bytes,
                });
            }
            None
        }

pub(super) fn on_snapshot_caught_up(subscription_id: String, through_seq: Option<u64>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let mut st = state.lock().recover_poison();
                let accepted = st
                    .attach_subscriptions
                    .get_mut(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .map(|subscription| subscription.caught_up = true)
                    .is_some();
                accepted
                    .then(|| st.attach_views.get(&subscription_id).cloned())
                    .flatten()
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::SnapshotCaughtUp {
                    subscription_id,
                    through_seq,
                });
            }
            None
        }

pub(super) fn on_runtime(agent: String, model: String, effort: Option<String>, source: bridget_transport::protocol::RuntimeSource, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let mut st = state.lock().recover_poison();
            if source == bridget_transport::protocol::RuntimeSource::Declared {
                let instance_id = match self_mutation_instance(&st, conn_id, &agent, "runtime") {
                    Ok(instance_id) => instance_id,
                    Err(response) => return Some(*response),
                };
                Some(handle_runtime_for_instance(
                    &agent,
                    model,
                    effort,
                    source,
                    Some(&instance_id),
                    &mut st,
                ))
            } else {
                Some(handle_runtime(&agent, model, effort, source, &mut st))
            }
        }

pub(super) fn on_journal_read_error(subscription_id: String, line: u64, offset: u64, reason: String, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let st = state.lock().recover_poison();
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                st.attach_views.get(&subscription_id).cloned()
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::JournalReadError {
                    subscription_id,
                    line,
                    offset,
                    reason,
                });
            }
            None
        }

pub(super) fn on_guichet_lookup(version: u16, issuer_scope: String, request_id: String, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let st = state.lock().recover_poison();
            match st
                .store
                .lookup_guichet(&issuer_scope, &request_id, unix_timestamp())
            {
                Ok(result) => Some(guichet_result_response(issuer_scope, request_id, result)),
                Err(error) => {
                    error!("lookup guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }

pub(super) fn on_unregister(conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let (controls, views) = {
                let mut st = state.lock().recover_poison();
                let (controls, views) = close_attach_subscriptions(&mut st, conn_id);
                st.router.unregister_by_conn(conn_id);
                st.mark_stopped(conn_id);
                st.conn_names.remove(conn_id);
                st.observation_sequences.remove(conn_id);
                st.runtime_catalogue_connections.remove(conn_id);
                st.observations.remove_source(conn_id, Instant::now());
                st.conn_hosts.remove(conn_id);
                st.communication_projects.remove(conn_id);
                st.conn_operating_systems.remove(conn_id);
                st.service_negotiations.remove(conn_id);
                st.terminal_sessions.remove(conn_id);
                (controls, views)
            };
            let _ = execute_controls(controls);
            for view in views {
                view.close_and_join();
            }
            None
        }

pub(super) fn on_guichet_claim_next(version: u16, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![SERVICE_CONTRACT_VERSION],
                    },
                });
            }
            let mut st = state.lock().recover_poison();
            match st.store.claim_next_guichet(conn_id, unix_timestamp()) {
                Ok(GuichetNext::Claimed(claim)) => Some(guichet_claim_response(claim)),
                Ok(GuichetNext::Empty) => Some(DaemonToWrapper::GuichetEmpty {
                    version: SERVICE_CONTRACT_VERSION,
                }),
                Err(error) => {
                    error!("claim guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }

pub(super) fn on_end(subscription_id: String, reason: String, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let mut st = state.lock().recover_poison();
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                st.attach_subscriptions.remove(&subscription_id);
                st.attach_views.remove(&subscription_id)
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::End {
                    subscription_id,
                    reason,
                });
                view.close_and_join();
            }
            None
        }

pub(super) fn on_delegated_runtime_event_acknowledged(event_id: String, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let (parent_instance_id, fleet) = {
                let st = state.lock().recover_poison();
                let Some(parent_instance_id) = st.conn_instances.get(conn_id).cloned() else {
                    return Some(DaemonToWrapper::Nack {
                        id: event_id,
                        reason: "wrapper non enregistré".to_string(),
                    });
                };
                (parent_instance_id, Arc::clone(&st.fleet))
            };
            match fleet.acknowledge_delegated_runtime_event(&event_id, &parent_instance_id) {
                Ok(_) => Some(DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: event_id,
                }),
                Err(_) => Some(DaemonToWrapper::Nack {
                    id: event_id,
                    reason: "accusé runtime délégué refusé".to_string(),
                }),
            }
        }
