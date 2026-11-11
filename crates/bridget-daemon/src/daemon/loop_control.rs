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
        "loop_control_scope_mismatch" | "loop_control_project_changed" => {
            "Comparer la portée de remise et le projet attesté de la cible au mandat de cette boucle. Reprendre le contexte d'origine ; un déplacement ou un changement de projet exige une nouvelle autorisation ciblée."
        }
        "loop_target_do_not_disturb" => {
            "Attendre la fin du mode ne pas déranger de cette cible, ou demander à son propriétaire de le lever si cette action est autorisée. Conserver la remise sans choisir un autre mode."
        }
        _ => {
            "Vérifier la lecture du mandat privé et la réponse native. Conserver la même clé, le même fichier et la même enveloppe tant que l'issue n'est pas établie."
        }
    };
    format!(
        "{action} Cause constatée : {code}. Ce refus concerne cet envoi de boucle ; les autres travaux autorisés suivent leur propre état."
    )
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
                db.execute(
                    "INSERT OR IGNORE INTO loop_control_grants(grant_id,payload) VALUES(?1,?2)",
                    params![id, payload],
                )
                .map_err(|_| "loop_control_storage_unavailable")?;
                if load(db, &id)? != grant {
                    return Err("loop_authorization_mismatch");
                }
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
                st.loop_control_connections.insert(conn.into(), id.clone());
                Ok(LoopControlOutcome::Bound { grant_id: id })
            }
            LoopControlRequest::Revoke { .. } => {
                // Le secret de la boucle ne donne que le retrait de ce mandat.
                let changed = st
                    .store
                    .connection()
                    .execute(
                        "UPDATE loop_control_grants SET revoked=1 WHERE grant_id=?1",
                        [&id],
                    )
                    .map_err(|_| "loop_control_storage_unavailable")?;
                if changed == 0 {
                    return Err("loop_control_unknown");
                }
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
    if connection_communication_project(st, conn).as_ref() != Some(&grant.project)
        || agent_communication_project(st, target).as_ref() != Some(&grant.project)
    {
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
