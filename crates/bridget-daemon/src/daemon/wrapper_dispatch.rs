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
