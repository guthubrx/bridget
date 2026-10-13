//! Demandes155 : identité attestée, décision durable et clôture fournisseur.
use super::*;
use crate::delegation::{ChildRequest, Task};
use bridget_transport::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestAnswer, ChildRequestClosedState,
    ChildRequestDecision, ChildRequestVerdict,
};
use log::debug;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// O(b), sortie UTF-8<=limit octets ; texte cité seulement, jamais une autorité.
fn quoted_text(text: &str, limit: usize) -> String {
    let mut output = String::new();
    for character in text.chars() {
        let character = if bridget_core::is_disallowed_control(character) {
            ' '
        } else {
            character
        };
        if output.len() + character.len_utf8() > limit {
            break;
        }
        output.push(character);
    }
    output
}

/// O((a+1) log t+log liens+B), a<=7 ancêtres, B payloads lus/clonés cumulés.
/// Aucune identité fournie par le payload ne fait autorité.
fn linked_task(st: &DaemonState, conn: &str) -> Result<Task, String> {
    let (child, instance) = live_connection_identity(st, conn).ok_or("request_unlinked")?;
    let link = st
        .fleet
        .agent_link_for_child(&instance)
        .map_err(|e| e.to_string())?
        .ok_or("request_unlinked")?;
    let task = st
        .delegation_store
        .get(link.delegation_id.as_deref().ok_or("request_unlinked")?)?
        .ok_or("request_unlinked")?;
    let command = st
        .managed_by_instance
        .get(&instance)
        .ok_or("request_unlinked")?;
    if task.child != child
        || task.child_instance.as_deref() != Some(instance.as_str())
        || task.child_requests.as_deref() != Some("forward")
        || task.definition.protocol != "codex_app_server"
        || link.role != "native_delegate"
        || !link.state.owns_child()
        || command != &task.task_id
        || st
            .fleet
            .resolved_definition_for_command(command)
            .is_none_or(|definition| definition.digest != task.definition.digest)
        || (link.parent_instance_id != task.owner_instance
            && link.parent_instance_id != task.origin_owner_instance)
    {
        return Err("request_unlinked".into());
    }
    let mut visited = HashSet::from([task.task_id.clone()]);
    let mut current = task.clone();
    while let Some(id) = current.parent_task_id.as_deref() {
        if visited.len() >= 8 || !visited.insert(id.to_owned()) {
            return Err("request_unlinked".into());
        }
        let ancestor = st
            .delegation_store
            .get(id)
            .map_err(|_| "request_unlinked")?
            .ok_or("request_unlinked")?;
        if ancestor.root_owner() != task.root_owner()
            || !matches!(
                ancestor.state.as_str(),
                "starting" | "mission_pending" | "working" | "waiting_for_children"
            )
        {
            return Err("request_unlinked".into());
        }
        current = ancestor;
    }
    Ok(task)
}

/// O(log t+log liens+log droits+B) ; FR027 est refait avant chaque émission.
pub(super) fn valid_delivery(
    st: &DaemonState,
    task: &Task,
    row: &ChildRequest,
    now: i64,
) -> Result<String, &'static str> {
    if now >= row.deadline_at
        || task
            .mission_deadline_at
            .is_some_and(|end| now >= end.saturating_sub(30))
    {
        return Err("expired");
    }
    if task.state == "cancelling" {
        return Err("cancelled");
    }
    if !matches!(
        task.state.as_str(),
        "starting" | "mission_pending" | "working" | "waiting_for_children"
    ) {
        return Err("task_ended");
    }
    if crate::referent_control::read(st.store.connection())
        .map_err(|_| "paused")?
        .paused
    {
        return Err("paused");
    }
    if st
        .delegation_store
        .revoked_agent(&task.owner)
        .map_err(|_| "revoked")?
        || st
            .delegation_store
            .revoked_agent(&task.root_owner_agent_id)
            .map_err(|_| "revoked")?
        || st
            .delegation_store
            .revoked(&task.owner_instance)
            .map_err(|_| "revoked")?
    {
        return Err("revoked");
    }
    if task.child_instance.as_deref() != Some(row.child_instance.as_str()) {
        return Err("instance_changed");
    }
    let route = st.router.get_agent(&task.child).ok_or("instance_changed")?;
    let conn = route.connection_id.clone();
    if live_connection_identity(st, &conn) != Some((task.child.clone(), row.child_instance.clone()))
    {
        return Err("instance_changed");
    }
    let proved = linked_task(st, &conn).map_err(|_| "instance_changed")?;
    if proved.task_id != task.task_id {
        return Err("instance_changed");
    }
    if !parent_available(st, task) {
        return Err("revoked");
    }
    if task.remote_placement.is_some() {
        // Les lectures de sources sur destination sont réservées à emit, hors mutex.
        native_delegation::recheck_remote_dispatch(st, task, task).map_err(|_| "revoked")?;
    } else {
        native_delegation::native_launch_registry(st, task).map_err(|_| "revoked")?;
    }
    Ok(conn)
}

/// O(log droits), le parent est le propriétaire direct vivant, pas la racine.
fn parent_available(st: &DaemonState, task: &Task) -> bool {
    st.router.get_agent(&task.owner).is_some_and(|route| {
        live_connection_identity(st, &route.connection_id)
            == Some((task.owner.clone(), task.owner_instance.clone()))
    }) && st.delegation_store.revoked_agent(&task.owner) == Ok(false)
        && st.delegation_store.revoked_agent(&task.root_owner_agent_id) == Ok(false)
        && st.delegation_store.revoked(&task.owner_instance) == Ok(false)
}

/// O(B), tuple JSON sans ambiguïté ; ID fournisseur numérique et chaîne distincts.
fn request_key(task: &Task, request: &ChildProviderRequest) -> Result<String, String> {
    let tuple = serde_json::to_vec(&(
        &task.task_id,
        &task.child_instance,
        &request.provider_session,
        &request.turn_id,
        &request.provider_request_id,
    ))
    .map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(tuple))[..32].to_owned())
}

/// O(B), reconstruction de la seule réponse HISTORIQUE, pas une nouvelle décision.
fn persisted_answer(row: &ChildRequest) -> Result<DaemonToWrapper, String> {
    Ok(DaemonToWrapper::ChildRequestAnswer {
        answer: ChildRequestAnswer {
            request_id: row.request_id.clone(),
            provider_session: row.provider_session.clone(),
            turn_id: row.turn_id.clone(),
            provider_request_id: serde_json::from_str(&row.provider_request_id)
                .map_err(|e| e.to_string())?,
            decision: serde_json::from_value(json!(row.decision.as_deref().unwrap_or("decline")))
                .map_err(|e| e.to_string())?,
            answer: row
                .answer_json
                .as_deref()
                .map(serde_json::from_str)
                .transpose()
                .map_err(|e| e.to_string())?,
            outcome: row.outcome.clone().unwrap_or_else(|| "cancelled".into()),
        },
    })
}

/// O(B), la fermeture est liée au tuple durable exact.
fn cancel_frame(row: &ChildRequest) -> Result<DaemonToWrapper, String> {
    Ok(DaemonToWrapper::ChildRequestCancel {
        request_id: row.request_id.clone(),
        provider_session: row.provider_session.clone(),
        turn_id: row.turn_id.clone(),
        provider_request_id: serde_json::from_str::<ChildProviderRequestId>(
            &row.provider_request_id,
        )
        .map_err(|e| e.to_string())?,
    })
}

/// O(log n), invalider l'accord ne prétend jamais clore le fournisseur.
fn invalidate(st: &DaemonState, row: &ChildRequest, reason: &str, now: i64) -> Result<(), String> {
    if row.state == "open" {
        st.delegation_store.close_child_request(
            &row.request_id,
            if reason == "expired" {
                "expired"
            } else {
                "cancelled"
            },
            if reason == "expired" {
                "expired"
            } else {
                "cancelled"
            },
            now,
        )?;
    }
    st.delegation_store
        .void_pending_child_request(&row.request_id, reason)?;
    st.delegation_store
        .mark_child_closing(&row.request_id, reason)?;
    settle_closed_inbox(st, row, reason, now)
}

/// O(log item+log demande), règlement négatif exact T023, aucune décision positive.
fn settle_closed_inbox(
    st: &DaemonState,
    row: &ChildRequest,
    reason: &str,
    now: i64,
) -> Result<(), String> {
    let Some(id) = row.inbox_item_id.as_deref().filter(|_| !row.inbox_settled) else {
        return Ok(());
    };
    if let Some(item) =
        crate::human_inbox::get(st.store.connection(), id).map_err(|e| e.to_string())?
    {
        match item.state {
            bridget_transport::protocol::HumanInboxState::Open => {
                crate::human_inbox::close_self(st.store.connection(), id, reason, now)
                    .map_err(|e| e.to_string())?;
            }
            bridget_transport::protocol::HumanInboxState::Resolved => {
                if let Some(decision) = item.decision {
                    crate::human_inbox::ack(st.store.connection(), &decision.decision_id, now)
                        .map_err(|e| e.to_string())?;
                }
            }
            bridget_transport::protocol::HumanInboxState::ClosedSelf => {}
        }
    }
    st.delegation_store
        .mark_child_inbox_settled(&row.request_id)?;
    Ok(())
}

/// O(B+log n), clé exacte recherchée même après résolution ; réparation CAS du lien.
fn publish_human(
    st: &DaemonState,
    task: &Task,
    row: &ChildRequest,
) -> Result<
    Option<(
        crate::human_inbox::HumanChannelConfig,
        crate::human_inbox::ChannelSummary,
    )>,
    String,
> {
    if row.inbox_item_id.is_some() {
        return Ok(None);
    }
    let key = format!("native-child-request:{}", row.request_id);
    let existing = crate::human_inbox::find_by_dedup_key(st.store.connection(), &key)
        .map_err(|e| e.to_string())?;
    let (item, created) = if let Some(item) = existing {
        (item.id, false)
    } else {
        let facts: Value = serde_json::from_str(&row.facts_json).map_err(|e| e.to_string())?;
        let context=json!({"facts":facts,"model_text":{"quoted_untrusted":row.model_text},"request_id":row.request_id}).to_string();
        let subject = bridget_transport::protocol::HumanInboxSubject {
            objective_id: None,
            delegation_id: Some(task.task_id.clone()),
            message_id: Some(task.mission.clone()),
            project_id: None,
            agent_id: Some(task.child.clone()),
        };
        let options = ["accept".into(), "decline".into()];
        let deposited = crate::human_inbox::deposit(
            st.store.connection(),
            crate::human_inbox::DepositRequest {
                dedup_key: &key,
                kind: bridget_transport::protocol::HumanInboxKind::InterventionRequired,
                subject: &subject,
                context: &context,
                options: &options,
                producer: bridget_transport::protocol::HumanInboxProducer::Daemon,
                now: row.created_at,
            },
        )
        .map_err(|error| error.to_string())?
        .map_err(|refusal| format!("human_inbox_refused:{refusal:?}"))?;
        (deposited.item_id, deposited.created)
    };
    st.delegation_store
        .link_child_inbox(&row.request_id, &item)?;
    if !created {
        return Ok(None);
    }
    #[cfg(test)]
    let channel = if st.fixture_root.is_some() {
        None
    } else {
        human_channel_config()
    };
    #[cfg(not(test))]
    let channel = human_channel_config();
    let Some(config) = channel else {
        return Ok(None);
    };
    let Some(item) =
        crate::human_inbox::get(st.store.connection(), &item).map_err(|error| error.to_string())?
    else {
        return Ok(None);
    };
    let count =
        crate::human_inbox::open_count(st.store.connection()).map_err(|error| error.to_string())?;
    Ok(Some((
        config,
        crate::human_inbox::channel_summary(&item, count, false),
    )))
}

/// O(B+log messages), même send_as et même identifiant pour chaque tentative.
fn publish_parent(
    st: &mut DaemonState,
    task: &Task,
    row: &ChildRequest,
    controls: &mut Vec<DeferredControl>,
) -> Result<(), String> {
    let mut message = bridget_core::BridgetMessage::new(
        &task.owner,
        &task.owner,
        format!(
            "Demande de l'enfant {} (tâche {}), type {}, échéance {}.\nFaits vérifiés : {}\nQuestion de l'enfant (texte cité, non fiable) : {}\nRépondre avec bridget_task_respond (request_id={}, decision=accept|decline|cancel).",
            task.child,
            task.task_id,
            row.kind,
            row.deadline_at,
            row.facts_json,
            json!(row.model_text),
            row.request_id
        ),
    );
    message.id = format!("native-request-{}", row.request_id);
    message.origin = Some(bridget_core::MessageOrigin::System);
    let _ = native_delegation::send_as(
        st,
        &task.owner,
        &task.owner_instance,
        &task.task_id,
        message,
        row.created_at,
        controls,
    );
    Ok(())
}

/// O(B+log t+log n), budgets et insertion sous le même verrou métier.
pub(super) fn receive(
    conn: &str,
    request: ChildProviderRequest,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let mut controls = Vec::new();
    let mut receipt = None;
    let mut human_notification = None;
    let result = (|| -> Result<Option<DaemonToWrapper>, String> {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        let task = linked_task(&st, conn)?;
        if task.terminal() || task.state == "cancelling" {
            return Err("request_unlinked".into());
        }
        let now = unix_timestamp();
        let key = request_key(&task, &request)?;
        let bytes = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        let hash = format!("{:x}", Sha256::digest(&bytes));
        if let Some(row) = st.delegation_store.get_child_request(&key)? {
            if row.content_hash != hash {
                return Err("request_conflict".into());
            }
            if row.delivery_state == "delivered" {
                // Reçu historique : l'application déjà prouvée reste delivered.
                // Ce n'est ni une nouvelle décision, ni un nouveau claim de droit.
                if valid_delivery(&st, &task, &row, now).is_ok() && row.sent_at.is_some() {
                    if let Some(writer) = st.connections.get(conn) {
                        receipt = Some((
                            row.request_id.clone(),
                            conn.to_owned(),
                            writer.clone(),
                            persisted_answer(&row)?,
                        ));
                    }
                    return Ok(None);
                }
                return Ok(Some(cancel_frame(&row)?));
            }
            if row.state == "open" {
                if row.verdict == "human" {
                    human_notification = publish_human(&st, &task, &row)?;
                } else if row.verdict == "parent" {
                    publish_parent(&mut st, &task, &row, &mut controls)?;
                }
            }
            // Remettre un accord seulement par tick/FR027. Les décisions closes
            // sont relues sans les réappliquer ; les invalidations demandent Closed.
            return Ok(
                if matches!(row.delivery_state.as_str(), "void" | "unknown")
                    || row.void_reason.is_some()
                {
                    Some(cancel_frame(&row)?)
                } else {
                    None
                },
            );
        }
        let snapshot = task
            .permission_snapshot
            .as_ref()
            .ok_or("request_unlinked")?;
        let (mut verdict, mut code) = crate::native_permissions::classify_request(
            snapshot,
            &request,
            parent_available(&st, &task),
        );
        if bytes.len() > 16384
            || request.provider_session.is_empty()
            || request.turn_id.is_empty()
            || request.provider_session.len() > 256
            || request.turn_id.len() > 256
        {
            verdict = ChildRequestVerdict::Forbidden;
            code = Some("details_incomplete");
        }
        let counts = st.delegation_store.child_request_counts(&task.task_id)?;
        if counts.0 >= 4 || counts.1 >= 32 {
            verdict = ChildRequestVerdict::Forbidden;
            code = Some("request_budget_exhausted");
        }
        let window = if verdict == ChildRequestVerdict::Human {
            native_delegation::CHILD_REQUEST_HUMAN_WINDOW_SECS
        } else {
            native_delegation::CHILD_REQUEST_PARENT_WINDOW_SECS
        };
        let deadline = task
            .mission_deadline_at
            .map(|end| end.saturating_sub(30))
            .unwrap_or(now + window)
            .min(now + window);
        if deadline <= now {
            verdict = ChildRequestVerdict::Forbidden;
            code = Some("deadline_too_close");
        }
        let kind = serde_json::to_value(request.kind)
            .map_err(|e| e.to_string())?
            .as_str()
            .unwrap()
            .to_owned();
        let mut text = request.details["reason"].as_str().unwrap_or("").to_owned();
        if let Some(questions) = request.details["questions"].as_array() {
            for question in questions {
                if let Some(value) = question["question"].as_str() {
                    text.push(' ');
                    text.push_str(value);
                }
            }
        }
        let mut paths = Vec::new();
        let mut path_budget = 1024usize;
        let mut paths_cited_truncated = false;
        let mut paths_cited_changed = false;
        if let Some(values) = request.details["paths_cited"].as_array() {
            paths_cited_truncated = values.len() > 64;
            for (index, value) in values.iter().take(64).enumerate() {
                let Some(path) = value.as_str() else {
                    paths_cited_changed = true;
                    continue;
                };
                let lexical = std::path::Path::new(path).components().collect::<PathBuf>();
                let lexical = lexical.to_string_lossy();
                let limit = path_budget.min(256);
                paths_cited_changed |= lexical.as_ref() != path
                    || lexical.chars().any(bridget_core::is_disallowed_control);
                let display_bytes: usize = lexical
                    .chars()
                    .map(|character| {
                        if bridget_core::is_disallowed_control(character) {
                            1
                        } else {
                            character.len_utf8()
                        }
                    })
                    .sum();
                paths_cited_truncated |= display_bytes > limit;
                let clean = quoted_text(&lexical, limit);
                path_budget = path_budget.saturating_sub(clean.len() + 8);
                paths.push(clean);
                if path_budget == 0 {
                    paths_cited_truncated |= index + 1 < values.len();
                    break;
                }
            }
        }
        let raw_command = request.details["command"].as_str().unwrap_or("");
        // Debug rend les contrôles et caractères de direction visibles, sans
        // modifier ni couper la commande qui sera exécutée par le fournisseur.
        let command = if raw_command.is_empty() {
            String::new()
        } else {
            format!("{raw_command:?}")
        };
        let cwd = request.details["cwd"]
            .as_str()
            .map(|cwd| format!("{cwd:?}"));
        let mut facts=json!({"method":quoted_text(request.details["method"].as_str().unwrap_or("unsupported"),128),
            "duration":if request.details["duration"]=="once"{"once"}else{"unsupported"},
            "command":command,
            "cwd":cwd,"cwd_verified":false,
            "paths_cited":paths,"paths_verified":false,"paths_cited_truncated":paths_cited_truncated,"paths_cited_changed":paths_cited_changed,
            "effects":{"command":request.details["effects"]["command"].as_bool(),"file_write":request.details["effects"]["file_write"].as_bool(),"network":request.details["effects"]["network"].as_bool()},
            "network":request.details["effects"]["network"].as_bool(),
            "child_posture":if snapshot.child_policy["sandbox_policy"]["type"]=="readOnly"{"read_only"}else{"development"},
            "parent_sandbox":snapshot.parent.provider_policy["sandbox_policy"]["type"],"verdict_code":code,
            "question_ids":if verdict==ChildRequestVerdict::Parent{request.question_ids.clone()}else{Vec::new()}}).to_string();
        if raw_command.len() > 1024 || facts.len() > 4096 {
            verdict = ChildRequestVerdict::Forbidden;
            code = Some("details_incomplete");
            facts=json!({"method":"unsupported","verdict_code":"details_incomplete","question_ids":[]}).to_string();
        }
        if code == Some("secret_not_forwardable") {
            text.clear();
        }
        let refused = verdict == ChildRequestVerdict::Forbidden;
        let outcome = code.map(|code| format!("refused:{code}"));
        let row = ChildRequest {
            request_id: key,
            task_id: task.task_id.clone(),
            child_instance: task.child_instance.clone().unwrap(),
            provider_session: request.provider_session.clone(),
            turn_id: request.turn_id.clone(),
            provider_request_id: serde_json::to_string(&request.provider_request_id)
                .map_err(|e| e.to_string())?,
            content_hash: hash,
            kind,
            verdict: serde_json::to_value(verdict)
                .map_err(|e| e.to_string())?
                .as_str()
                .unwrap()
                .into(),
            state: if refused { "refused" } else { "open" }.into(),
            outcome,
            summary: format!(
                "Demande {} de l'enfant",
                serde_json::to_value(request.kind)
                    .unwrap()
                    .as_str()
                    .unwrap()
            ),
            facts_json: facts,
            model_text: quoted_text(&text, 1024),
            offered_json: serde_json::to_string(&request.offered).map_err(|e| e.to_string())?,
            created_at: now,
            deadline_at: deadline,
            decision: if refused {
                Some("decline".into())
            } else {
                None
            },
            answer_json: None,
            decided_by: if refused { Some("system".into()) } else { None },
            decided_at: if refused { Some(now) } else { None },
            delivery_state: if refused { "sent" } else { "none" }.into(),
            void_reason: None,
            sent_at: if refused { Some(now) } else { None },
            delivered_at: None,
            inbox_item_id: None,
            inbox_settled: false,
        };
        // Le refus33 est rendu au fournisseur sans dépasser le plafond durable.
        if counts.1 >= 32 {
            return Ok(Some(persisted_answer(&row)?));
        }
        let row = st.delegation_store.insert_child_request(&row)?;
        if refused {
            return Ok(Some(persisted_answer(&row)?));
        }
        if verdict == ChildRequestVerdict::Human {
            human_notification = publish_human(&st, &task, &row)?;
        } else {
            publish_parent(&mut st, &task, &row, &mut controls)?;
            st.delegation_store.wait_epoch().bump();
        }
        Ok(None)
    })();
    if let Some((config, summary)) = human_notification {
        // Le programme externe ne détient jamais le verrou métier. Seule la
        // consignation locale de sa tentative reprend brièvement state.
        let pushed = crate::human_inbox::push_to_channel(&config, &summary);
        let attempt = crate::human_inbox::NotifyAttempt {
            at: unix_timestamp(),
            ok: pushed.is_ok(),
            error: pushed.err(),
        };
        let st = state.lock().unwrap_or_else(|error| error.into_inner());
        if let Err(error) = crate::human_inbox::record_notify_attempt(
            st.store.connection(),
            &summary.item_id,
            &attempt,
        ) {
            warn!("native_child_request_notify_attempt_unavailable : {error}");
        }
    }
    // Un lecteur connecté peut cesser de lire. Avis idempotent conservé, mais
    // aucun flush non borné : budget1s TOTAL après libération de state.
    let deadline = Instant::now() + Duration::from_secs(1);
    for control in controls {
        if let Err(error) = push_control_message_until(&control.writer, &control.message, deadline)
        {
            warn!("native_child_request_notice_unavailable : {error}");
        }
    }
    if let Some((id, conn, writer, frame)) = receipt {
        emit(state, &id, &conn, &writer, &frame, true);
    }
    match result {
        Ok(reply) => reply,
        Err(reason) => Some(DaemonToWrapper::Nack {
            id: "child-request".into(),
            reason,
        }),
    }
}

/// O(log n+log liens), ACK d'une autre connexion ignoré sans mutation.
pub(super) fn acknowledged(
    conn: &str,
    id: &str,
    applied: bool,
    reason: Option<&str>,
    st: &DaemonState,
) {
    let Ok(Some(row)) = st.delegation_store.get_child_request(id) else {
        return;
    };
    let Ok(Some(task)) = st.delegation_store.get(&row.task_id) else {
        return;
    };
    let link = st
        .fleet
        .agent_link_for_child(&row.child_instance)
        .ok()
        .flatten();
    if live_connection_identity(st, conn) != Some((task.child.clone(), row.child_instance.clone()))
        || !link.is_some_and(|link| {
            link.delegation_id.as_deref() == Some(row.task_id.as_str())
                && link.role == "native_delegate"
        })
    {
        warn!("ACK155 d'une autre connexion ignoré");
        return;
    }
    if row.delivery_state == "sent" {
        let _ = st.delegation_store.settle_sent_child_request(
            id,
            if applied { "delivered" } else { "void" },
            if applied {
                None
            } else {
                Some("closed_before_apply")
            },
            unix_timestamp(),
        );
        if !applied {
            debug!(
                "réponse155 non appliquée : {}",
                quoted_text(reason.unwrap_or("closed_before_apply"), 256)
            );
        }
        let _ = settle_closed_inbox(st, &row, "request_closed", unix_timestamp());
    }
}

/// O(log n+log liens), Closed confirme aussi les attentes none/pending/void.
pub(super) fn closed(conn: &str, id: &str, result: ChildRequestClosedState, st: &DaemonState) {
    let Ok(Some(row)) = st.delegation_store.get_child_request(id) else {
        return;
    };
    let Ok(Some(task)) = st.delegation_store.get(&row.task_id) else {
        return;
    };
    let link = st
        .fleet
        .agent_link_for_child(&row.child_instance)
        .ok()
        .flatten();
    if live_connection_identity(st, conn) != Some((task.child.clone(), row.child_instance.clone()))
        || !link.is_some_and(|link| {
            link.delegation_id.as_deref() == Some(row.task_id.as_str())
                && link.role == "native_delegate"
        })
    {
        return;
    }
    let _ = st.delegation_store.confirm_child_closed(
        id,
        result == ChildRequestClosedState::AlreadyApplied,
        unix_timestamp(),
    );
    let _ = settle_closed_inbox(st, &row, "request_closed", unix_timestamp());
}

/// O(n+B), au plus512 demandes suivies ; aucune relève humaine globale.
pub(super) fn tick(state: &Arc<Mutex<DaemonState>>) {
    let mut outgoing = Vec::new();
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        // T029 : au plus512 liens155, jamais les premières décisions globales
        // du producteur. Le puits humain087 a déjà attesté le décideur.
        let Ok(human_rows) = st.delegation_store.child_inbox_followups() else {
            return;
        };
        for row in human_rows {
            let now = unix_timestamp();
            if row.verdict != "human"
                || row.state != "open"
                || row.delivery_state != "none"
                || row.void_reason.is_some()
            {
                // Une question mal liée ou un replay après CAS ne devient pas
                // une approbation. Réparer l'ACK sans créer de nouvelle décision.
                if let Err(error) = settle_closed_inbox(&st, &row, "request_closed", now) {
                    warn!("règlement item155 impossible: {error}");
                }
                continue;
            }
            let Ok(Some(task)) = st.delegation_store.get(&row.task_id) else {
                let _ = invalidate(&st, &row, "task_ended", now);
                continue;
            };
            if let Err(reason) = valid_delivery(&st, &task, &row, now) {
                // Pause/révocation/échéance : fermer et acquitter sans stocker
                // un accord qui serait exécuté après la reprise.
                let _ = invalidate(&st, &row, reason, unix_timestamp());
                continue;
            }
            let Some(item_id) = row.inbox_item_id.as_deref() else {
                continue;
            };
            let item = match crate::human_inbox::get(st.store.connection(), item_id) {
                Ok(Some(item)) => item,
                Ok(None) => {
                    let _ = invalidate(&st, &row, "request_closed", unix_timestamp());
                    continue;
                }
                Err(error) => {
                    warn!("item155 illisible: {error}");
                    continue;
                }
            };
            // Source/registre et item sont relus avant l'horloge du CAS. Le
            // timestamp initial ne survit pas à un contrôle source lent.
            let decision_at = unix_timestamp();
            if decision_at >= row.deadline_at
                || task
                    .mission_deadline_at
                    .is_some_and(|end| decision_at >= end.saturating_sub(30))
            {
                let _ = invalidate(&st, &row, "expired", decision_at);
                continue;
            }
            match item.state {
                bridget_transport::protocol::HumanInboxState::Open => continue,
                bridget_transport::protocol::HumanInboxState::ClosedSelf => {
                    let _ = invalidate(&st, &row, "request_closed", decision_at);
                    continue;
                }
                bridget_transport::protocol::HumanInboxState::Resolved => {}
            }
            let Some(decision) = item.decision.as_ref() else {
                let _ = invalidate(&st, &row, "request_closed", decision_at);
                continue;
            };
            let offered = serde_json::from_str::<Vec<ChildRequestDecision>>(&row.offered_json)
                .unwrap_or_default();
            let choice = match decision.choice.as_str() {
                "accept" => ChildRequestDecision::Accept,
                "decline" => ChildRequestDecision::Decline,
                _ => {
                    let _ = invalidate(&st, &row, "decision_not_offered", decision_at);
                    continue;
                }
            };
            if !offered.contains(&choice) {
                let _ = invalidate(&st, &row, "decision_not_offered", decision_at);
                continue;
            }
            // Même verrou métier pour recontrôle/CAS ; le clock du CAS est
            // relu après le décodage. Une panne ACK rejoue la décision durable,
            // pas l'accord : la prochaine lecture voit state=answered.
            let decision_at = unix_timestamp();
            if decision_at >= row.deadline_at
                || task
                    .mission_deadline_at
                    .is_some_and(|end| decision_at >= end.saturating_sub(30))
            {
                let _ = invalidate(&st, &row, "expired", decision_at);
                continue;
            }
            match st.delegation_store.decide_child_request(
                &row.request_id,
                &decision.choice,
                None,
                "human",
                "answered_by_human",
                decision_at,
            ) {
                Ok(true) => {
                    if let Err(error) =
                        settle_closed_inbox(&st, &row, "request_answered", decision_at)
                    {
                        warn!("ACK humain155 impossible: {error}");
                    }
                }
                Ok(false) => {}
                Err(error) => warn!("décision humaine155 non persistée: {error}"),
            }
        }
        let Ok(rows) = st.delegation_store.child_request_followups() else {
            return;
        };
        for row in rows {
            let Ok(Some(task)) = st.delegation_store.get(&row.task_id) else {
                continue;
            };
            let now = unix_timestamp();
            let validity = valid_delivery(&st, &task, &row, now);
            let closing = row.void_reason.is_some()
                || row.state == "refused"
                || matches!(row.state.as_str(), "expired" | "cancelled");
            if closing || validity.is_err() {
                let reason = validity.as_ref().err().copied().unwrap_or("cancelled");
                if invalidate(&st, &row, reason, now).is_err() {
                    continue;
                }
                // Fermer sur l'instance originale, jamais sur une nouvelle instance.
                let conn = st
                    .router
                    .get_agent(&task.child)
                    .map(|route| route.connection_id.clone());
                if let Some(conn) = conn.filter(|conn| {
                    live_connection_identity(&st, conn)
                        == Some((task.child.clone(), row.child_instance.clone()))
                }) {
                    if let (Some(writer), Ok(frame)) =
                        (st.connections.get(&conn), cancel_frame(&row))
                    {
                        outgoing.push((row.request_id.clone(), conn, writer.clone(), frame));
                    }
                } else if row.delivery_state == "sent" {
                    let _ = st.delegation_store.settle_sent_child_request(
                        &row.request_id,
                        "unknown",
                        Some(reason),
                        now,
                    );
                }
                continue;
            }
            if row.state == "open" {
                continue;
            }
            if row.delivery_state == "pending"
                && !st
                    .delegation_store
                    .claim_child_request(&row.request_id, now)
                    .unwrap_or(false)
            {
                continue;
            }
            if !matches!(row.delivery_state.as_str(), "pending" | "sent") {
                continue;
            }
            if let (Ok(conn), Ok(frame)) = (validity, persisted_answer(&row)) {
                if let Some(writer) = st.connections.get(&conn) {
                    outgoing.push((row.request_id.clone(), conn, writer.clone(), frame));
                }
            }
        }
    }
    for (id, conn, writer, frame) in outgoing {
        emit(state, &id, &conn, &writer, &frame, false);
    }
}

/// O(log n+B+sources149), unique point d'écriture contrôlée après writer acquis.
/// receipt ne crée aucun droit : delivered/sent_at restent des preuves historiques.
fn emit(
    state: &Arc<Mutex<DaemonState>>,
    id: &str,
    conn: &str,
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    frame: &DaemonToWrapper,
    receipt: bool,
) {
    let answer = matches!(frame, DaemonToWrapper::ChildRequestAnswer { .. });
    let remote = if answer {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let Ok(Some(row)) = st.delegation_store.get_child_request(id) else {
            return;
        };
        let Ok(Some(task)) = st.delegation_store.get(&row.task_id) else {
            return;
        };
        if let Some(placement) = &task.remote_placement {
            let Some(worker) = st.remote_workers.get(&placement.machine_id) else {
                return;
            };
            Some((task.clone(), worker.epoch.clone()))
        } else {
            None
        }
    } else {
        None
    };
    // Réponse parent, réponse humaine et rejeu de reçu empruntent ce seul puits.
    // Une indisponibilité ne vaut jamais une autorisation et garde le suivi borné155.
    let remote_proof = match remote {
        Some((task, epoch)) => match native_delegation::remote_dispatch_proof(state, &task) {
            Ok(proof) => Some((proof, epoch)),
            Err(error) => {
                warn!("preuve distante155 indisponible: {error}");
                return;
            }
        },
        None => None,
    };
    // Le callback n'attend pas state sous writer ; après acquisition, le
    // verrou métier reste détenu jusqu'au write pour figer pause/révocation.
    let mut permit = None;
    let result = push_control_message_until_checked(
        writer,
        frame,
        Instant::now() + Duration::from_secs(1),
        || {
            let st = state
                .try_lock()
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::WouldBlock))?;
            let fail = |message: String| std::io::Error::other(message);
            let row = st
                .delegation_store
                .get_child_request(id)
                .map_err(fail)?
                .ok_or_else(|| fail("request_unknown".into()))?;
            let task = st
                .delegation_store
                .get(&row.task_id)
                .map_err(fail)?
                .ok_or_else(|| fail("task_unavailable".into()))?;
            if live_connection_identity(&st, conn)
                != Some((task.child.clone(), row.child_instance.clone()))
            {
                return Err(fail("instance_changed".into()));
            }
            if row.delivery_state == "unknown"
                || (!receipt && row.delivery_state == "delivered")
                || row.void_reason.as_deref() == Some("closed_before_apply")
            {
                return Err(fail("request_closed".into()));
            }
            if answer {
                if let Some(placement) = &task.remote_placement {
                    let Some((proof, epoch)) = &remote_proof else {
                        return Err(fail("remote_permission_proof_required".into()));
                    };
                    if st
                        .remote_workers
                        .get(&placement.machine_id)
                        .is_none_or(|worker| {
                            &worker.epoch != epoch
                                || worker.contact
                                    != bridget_transport::protocol::WorkerContactState::Connected
                        })
                    {
                        return Err(fail("worker_epoch_changed".into()));
                    }
                    native_delegation::recheck_remote_dispatch(&st, &task, proof).map_err(fail)?;
                }
                if !(row.delivery_state == "sent"
                    || receipt && row.delivery_state == "delivered" && row.sent_at.is_some())
                {
                    return Err(fail("request_closed".into()));
                }
                if let Err(reason) = valid_delivery(&st, &task, &row, unix_timestamp()) {
                    invalidate(&st, &row, reason, unix_timestamp()).map_err(fail)?;
                    let cancel = cancel_frame(&row).map_err(fail)?;
                    permit = Some(st);
                    return Ok(Some(cancel));
                }
                // Horloge relue après les recontrôles de sources, juste avant emit.
                let now = unix_timestamp();
                if now >= row.deadline_at
                    || task
                        .mission_deadline_at
                        .is_some_and(|end| now >= end.saturating_sub(30))
                    || row.void_reason.is_some()
                {
                    invalidate(&st, &row, "expired", now).map_err(fail)?;
                    let cancel = cancel_frame(&row).map_err(fail)?;
                    permit = Some(st);
                    return Ok(Some(cancel));
                }
            }
            permit = Some(st);
            Ok(None)
        },
    );
    // Sans permis, le callback n'a pas autorisé le premier write. Le suivi
    // existant peut retenter sous FR027 ; une écriture autorisée reste incertaine.
    let had_emit_permit = permit.is_some();
    drop(permit);
    if result.is_err() && had_emit_permit {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(Some(row)) = st.delegation_store.get_child_request(id) {
            if row.delivery_state == "sent" && !receipt {
                let _ = st.delegation_store.settle_sent_child_request(
                    id,
                    "unknown",
                    Some("instance_changed"),
                    unix_timestamp(),
                );
            }
        }
    }
}

/// O(log n+32+B). UNIQUEMENT après preuve physique Stopped/StoppedForced/NotFound
/// du chemin149, ou groupes152 fermés au coldrestart. Aucune trame Closed inventée.
pub(super) fn settle_after_stop(st: &DaemonState, task: &Task) -> Result<(), String> {
    let now = unix_timestamp();
    for row in st.delegation_store.child_requests_for_task(&task.task_id)? {
        if matches!(row.delivery_state.as_str(), "delivered" | "unknown")
            || row.void_reason.as_deref() == Some("closed_before_apply")
        {
            settle_closed_inbox(st, &row, "task_ended", now)?;
            continue;
        }
        let reason = row.void_reason.as_deref().unwrap_or(
            if matches!(task.state.as_str(), "cancelling" | "cancelled") {
                "cancelled"
            } else {
                "task_ended"
            },
        );
        invalidate(st, &row, reason, now)?;
        if row.delivery_state == "sent" {
            // Le groupe disparu ne prouve pas si un accord envoyé fut appliqué.
            st.delegation_store.settle_sent_child_request(
                &row.request_id,
                "unknown",
                Some(reason),
                now,
            )?;
        } else if row.sent_at.is_none()
            && matches!(row.delivery_state.as_str(), "none" | "pending" | "void")
        {
            // Aucun envoi ne fut revendiqué. L'arrêt OS prouvé interdit désormais
            // toute application future de ce callback, sans ACK fournisseur fictif.
            st.delegation_store
                .confirm_child_closed(&row.request_id, false, now)?;
        }
    }
    Ok(())
}

/// O(t(log n+32+B)), coldrestart appelé APRÈS groupes152 fermés, avant reprise.
pub(super) fn prepare_restart(st: &DaemonState) -> Result<(), String> {
    for task in st.delegation_store.tasks()? {
        for row in st.delegation_store.child_requests_for_task(&task.task_id)? {
            if matches!(row.delivery_state.as_str(), "delivered" | "unknown")
                || row.void_reason.as_deref() == Some("closed_before_apply")
            {
                continue;
            }
            invalidate(st, &row, "restart", unix_timestamp())?;
        }
        settle_after_stop(st, &task)?;
    }
    Ok(())
}
