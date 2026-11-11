//! Mandat264 : contrôle d'une boucle sans identité humaine ni filiation empruntée.
use super::*;
use bridget_core::MessageIntent;
use bridget_transport::protocol::{CommunicationProject, LoopControlOutcome, LoopControlRequest};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Grant {
    issuer_scope: String,
    project: CommunicationProject,
    targets: Vec<String>,
    intent: MessageIntent,
}

pub(crate) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS loop_control_grants (
        grant_id TEXT PRIMARY KEY, payload TEXT NOT NULL CHECK(json_valid(payload)),
        revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN (0,1)));",
    )
}

fn token_id(token: &str) -> Result<String, &'static str> {
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid_loop_token");
    }
    Ok(format!("{:x}", Sha256::digest(token.as_bytes())))
}

fn load(conn: &Connection, id: &str) -> Result<Grant, &'static str> {
    load_for_binding(conn, id, false)
}

fn load_for_binding(
    conn: &Connection,
    id: &str,
    allow_revoked: bool,
) -> Result<Grant, &'static str> {
    let row: Option<(String, bool)> = conn
        .query_row(
            "SELECT payload,revoked FROM loop_control_grants WHERE grant_id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(|_| "loop_control_storage_unavailable")?;
    match row {
        Some((payload, revoked)) if !revoked || allow_revoked => {
            serde_json::from_str(&payload).map_err(|_| "loop_control_storage_unavailable")
        }
        Some(_) => Err("loop_control_revoked"),
        None => Err("loop_control_unknown"),
    }
}

fn refusal(code: &str) -> LoopControlOutcome {
    if matches!(
        code,
        "loop_control_audit_unavailable" | "loop_control_storage_unavailable"
    ) {
        return LoopControlOutcome::Refused { code: code.into(), recovery: "Vérifier le stockage du mandat et du journal durable avant de reprendre cette même opération. Aucun changement n'est confirmé ; conserver le fichier privé et ne pas créer de mandat de remplacement. Ce refus concerne cette opération ; les autres travaux autorisés suivent leur propre état.".into() };
    }
    LoopControlOutcome::Refused {
        code: code.into(),
        recovery: format!(
            "Vérifier le mandat privé de cette boucle, son mode, sa portée de remise, son projet et ses cibles ({code}). Une cible ou un mode nouveau exige un nouveau choix explicite de l'utilisateur et un nouveau mandat. Aucun envoi de remplacement n'a été effectué. Ce refus concerne cette demande ; les autres travaux autorisés suivent leur propre état."
        ),
    }
}

pub(super) fn guidance_recovery(code: &str) -> String {
    let action = match code {
        "loop_control_revoked" | "loop_control_unknown" => {
            "Conserver les reçus de cette boucle. Pour reprendre le contrôle, recueillir un nouveau choix de l'utilisateur et créer un nouveau mandat ; le mandat retiré ne se réactive pas par rejeu."
        }
        "loop_target_not_authorized" => {
            "Reprendre une cible explicitement désignée dans le mandat. Si une autre cible est nécessaire, demander ce choix à l'utilisateur et créer un nouveau mandat avant tout envoi."
        }
        "loop_control_mode_mismatch" => {
            "Reprendre le mode inscrit dans l'enveloppe et le mandat d'origine. Pour changer de mode, demander le choix à l'utilisateur avant de créer une nouvelle boucle ; ne pas modifier une remise déjà réservée."
        }
        "loop_control_scope_mismatch"
        | "loop_control_project_changed"
        | "loop_control_project_unavailable"
        | "loop_target_project_unavailable" => {
            "Comparer la portée de remise et le projet attesté de la cible au mandat de cette boucle. Reprendre le contexte d'origine ; un déplacement ou un changement de projet exige une nouvelle autorisation ciblée."
        }
        "loop_target_do_not_disturb" => {
            "Attendre la fin du mode ne pas déranger de cette cible, ou demander à son propriétaire de le lever si cette action est autorisée. Conserver le mode choisi."
        }
        "loop_target_unavailable" => {
            "Vérifier la connexion et l'identité de cette cible avant de reprendre. Cette route n'est plus attestée ; aucune relance d'agent ni autre destinataire n'est autorisé par ce refus."
        }
        "referent_paused" => {
            "Attendre la levée de la pause par le référent. Cette boucle ne peut pas lever la pause ni choisir un autre mode pour la contourner."
        }
        "mission_active" => {
            "Conserver la mission native active. Si son arrêt est nécessaire, demander une décision ciblée à son propriétaire ; ce mandat de boucle ne permet pas de l'interrompre."
        }
        "interrupt_rate_limited" => {
            "Attendre l'échéance retry_after_secs du refus natif avant toute nouvelle demande d'interruption. Une autre clé ou un autre mode ne permet pas de contourner cette limite."
        }
        _ => {
            "Vérifier la lecture du mandat privé et la réponse native. Conserver la même clé, le même fichier et la même enveloppe tant que l'issue n'est pas établie."
        }
    };
    format!(
        "{action} Cause constatée : {code}. Lire le reçu de cette clé : un état rejected est définitif pour cette clé. Après résolution vérifiée de la cause, une nouvelle demande exige de nouveaux id et issued_at ; conserver la cible, le corps et le mode autorisés. Une issue non confirmée exige au contraire la relecture exacte de la même clé et de la même enveloppe. Ce refus concerne cet envoi de boucle ; les autres travaux autorisés suivent leur propre état."
    )
}

fn audit(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    action: &str,
    actor: &str,
) -> Result<(), &'static str> {
    let target = format!("loop:{id}");
    let body =
        serde_json::json!({"event":"loop_control", "action":action, "grant_id":id}).to_string();
    let mut message = bridget_core::BridgetMessage::new(actor, &target, &body);
    message.origin = Some(bridget_core::MessageOrigin::System);
    message.intent = Some(MessageIntent::StoreOnly);
    // Journal existant, aucune remise ni notification. Même transaction que le mandat.
    crate::store::record_message_in_transaction(tx, &message, &format!("loop-control:{id}"))
        .map_err(|_| "loop_control_audit_unavailable")
}

pub(super) fn handle(
    conn: &str,
    request: LoopControlRequest,
    st: &mut DaemonState,
) -> LoopControlOutcome {
    let result = (|| -> Result<LoopControlOutcome, &'static str> {
        let token = match &request {
            LoopControlRequest::Authorize { token, .. }
            | LoopControlRequest::Use { token }
            | LoopControlRequest::Revoke { token } => token,
        };
        let id = token_id(token)?;
        match request {
            LoopControlRequest::Authorize {
                issuer_scope,
                project,
                targets,
                intent,
                ..
            } => {
                if guidance_source(st, Some(conn)).0
                    != crate::guidance::GuidanceActor::AttachedHuman
                {
                    return Err("loop_authorization_requires_human");
                }
                if crate::idempotency::validate_issuer_scope(&issuer_scope).is_err()
                    || !matches!(
                        intent,
                        MessageIntent::SteerCurrent | MessageIntent::InterruptAndStart
                    )
                    || targets.is_empty()
                    || targets.len() > 64
                    || targets.iter().any(|target| {
                        crate::threads::canonical_uuid(target).as_deref() != Some(target)
                    })
                    || project.root.is_empty()
                    || !std::path::Path::new(&project.root).is_absolute()
                    || project.root.len() > 4096
                    || project.root.chars().any(char::is_control)
                    || !bridget_core::host_is_attested(&project.host)
                {
                    return Err("invalid_loop_authorization");
                }
                if targets.iter().any(|target| {
                    agent_communication_project(st, target).as_ref() != Some(&project)
                }) {
                    return Err("loop_target_project_unavailable");
                }
                let mut targets = targets;
                targets.sort();
                targets.dedup();
                let grant = Grant {
                    issuer_scope,
                    project,
                    targets,
                    intent,
                };
                let payload =
                    serde_json::to_string(&grant).map_err(|_| "invalid_loop_authorization")?;
                let db = st.store.connection();
                let tx = db
                    .unchecked_transaction()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                tx.execute(
                    "INSERT OR IGNORE INTO loop_control_grants(grant_id,payload) VALUES(?1,?2)",
                    params![id, payload],
                )
                .map_err(|_| "loop_control_storage_unavailable")?;
                if load(&tx, &id)? != grant {
                    return Err("loop_authorization_mismatch");
                }
                audit(&tx, &id, "authorize", "human")?;
                tx.commit()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                Ok(LoopControlOutcome::Authorized { grant_id: id })
            }
            LoopControlRequest::Use { .. } => {
                if st.connection_roles.get(conn) != Some(&ConnectionRole::Client)
                    || st.conn_names.contains_key(conn)
                {
                    return Err("loop_control_requires_autonomous_client");
                }
                // Le retrait interdit les nouveaux effets, pas la lecture d'un ancien reçu.
                let grant = load_for_binding(st.store.connection(), &id, true)?;
                let negotiation = st
                    .client_negotiations
                    .get(conn)
                    .ok_or("loop_control_not_negotiated")?;
                if !negotiation
                    .capabilities
                    .contains(&ClientCapability::LoopControlV1)
                    || negotiation.issuer_scope != grant.issuer_scope
                    || connection_communication_project(st, conn).as_ref() != Some(&grant.project)
                {
                    return Err("loop_control_scope_mismatch");
                }
                let tx = st
                    .store
                    .connection()
                    .unchecked_transaction()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                audit(&tx, &id, "bind", &format!("loop:{id}"))?;
                tx.commit()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                st.loop_control_connections.insert(conn.into(), id.clone());
                Ok(LoopControlOutcome::Bound { grant_id: id })
            }
            LoopControlRequest::Revoke { .. } => {
                // Le secret de la boucle ne donne que le retrait de ce mandat.
                let tx = st
                    .store
                    .connection()
                    .unchecked_transaction()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                let changed = tx
                    .execute(
                        "UPDATE loop_control_grants SET revoked=1 WHERE grant_id=?1",
                        [&id],
                    )
                    .map_err(|_| "loop_control_storage_unavailable")?;
                if changed == 0 {
                    return Err("loop_control_unknown");
                }
                audit(&tx, &id, "revoke", &format!("loop:{id}"))?;
                tx.commit()
                    .map_err(|_| "loop_control_storage_unavailable")?;
                Ok(LoopControlOutcome::Revoked { grant_id: id })
            }
        }
    })();
    result.unwrap_or_else(refusal)
}

pub(super) fn actor(st: &DaemonState, conn: &str) -> Option<crate::guidance::GuidanceActor> {
    // Même un mandat révoqué reste identifié : le refus nomme sa vraie cause.
    let id = st.loop_control_connections.get(conn)?;
    Some(crate::guidance::GuidanceActor::Loop {
        id: format!("loop:{id}"),
    })
}

pub(super) fn check(
    st: &DaemonState,
    conn: &str,
    target: &str,
    action: crate::guidance::GuidanceAction,
    path: crate::guidance::GuidancePath,
) -> Result<(), &'static str> {
    let Some(id) = st.loop_control_connections.get(conn) else {
        return Ok(());
    };
    let grant = load(st.store.connection(), id)?;
    if st.client_negotiations.get(conn).is_none_or(|n| {
        n.issuer_scope != grant.issuer_scope
            || !n.capabilities.contains(&ClientCapability::LoopControlV1)
    }) {
        return Err("loop_control_scope_mismatch");
    }
    let expected = match grant.intent {
        MessageIntent::SteerCurrent => crate::guidance::GuidanceAction::Steer,
        MessageIntent::InterruptAndStart => crate::guidance::GuidanceAction::Interrupt,
        _ => return Err("invalid_loop_authorization"),
    };
    if action != expected || path != crate::guidance::GuidancePath::SendIdempotent {
        return Err("loop_control_mode_mismatch");
    }
    if grant
        .targets
        .binary_search_by(|id| id.as_str().cmp(target))
        .is_err()
    {
        return Err("loop_target_not_authorized");
    }
    let route = st
        .router
        .get_agent(target)
        .ok_or("loop_target_unavailable")?;
    if live_connection_identity(st, &route.connection_id).is_none() {
        return Err("loop_target_unavailable");
    }
    let source_project =
        connection_communication_project(st, conn).ok_or("loop_control_project_unavailable")?;
    let target_project =
        agent_communication_project(st, target).ok_or("loop_target_project_unavailable")?;
    if source_project != grant.project || target_project != grant.project {
        return Err("loop_control_project_changed");
    }
    if let Some(instance) = st
        .router
        .get_agent(target)
        .and_then(|r| st.conn_instances.get(&r.connection_id))
        && st
            .presences
            .get(instance)
            .is_some_and(|p| p.dnd_until.is_some_and(|until| Instant::now() < until))
    {
        return Err("loop_target_do_not_disturb");
    }
    Ok(())
}

pub(super) fn check_message(
    st: &DaemonState,
    conn: &str,
    message: &bridget_core::BridgetMessage,
) -> Result<(), &'static str> {
    if !st.loop_control_connections.contains_key(conn) {
        return Ok(());
    }
    let action = match message.intent {
        Some(MessageIntent::SteerCurrent) => crate::guidance::GuidanceAction::Steer,
        Some(MessageIntent::InterruptAndStart) => crate::guidance::GuidanceAction::Interrupt,
        _ => return Err("loop_control_mode_mismatch"),
    };
    check(
        st,
        conn,
        routing_agent_id(&message.to),
        action,
        crate::guidance::GuidancePath::SendIdempotent,
    )
}
