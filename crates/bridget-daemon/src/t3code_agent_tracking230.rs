//! Réconciliation du classement ; aucun message métier ni appel de modèle.
use super::*;
use bridget_transport::agent_tracking::{
    AgentTrackingConfirmation, AgentTrackingObservation, AgentTrackingRequest,
    AgentTrackingSyncOutcome,
};

fn save_reports(path: &Path, reports: &[AgentTrackingConfirmation]) -> Result<(), String> {
    let bytes = serde_json::to_vec(reports).map_err(|_| "reçus de classement non sérialisables")?;
    bridget_transport::fsutil::write_private_file_atomic(path, &bytes)
        .map_err(|_| "reçus de classement non enregistrés ; même tentative conservée".into())
}

pub(super) fn sync(
    paths: &Paths,
    session: &SharedSession,
    snapshot: &contract::Snapshot,
) -> Result<(), String> {
    let receipt_path = paths.dir.join("tracking-reports-v1.json");
    let reports: Vec<AgentTrackingConfirmation> = match std::fs::read(&receipt_path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|_| "reçus de classement illisibles ; fichier conservé")?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(_) => return Err("reçus de classement indisponibles ; fichier conservé".into()),
    };
    let observations = snapshot
        .threads
        .iter()
        .filter(|s| s.deleted_at.is_none() && s.archived_at.is_none())
        .map(|s| AgentTrackingObservation {
            thread_id: s.id.clone(),
            source_revision: s.updated_at.clone(),
            source_sequence: snapshot.snapshot_sequence,
            explicit_state: s
                .settled_override
                .as_deref()
                .filter(|v| matches!(*v, "settled" | "active"))
                .map(str::to_owned),
            can_settle: s
                .latest_turn
                .as_ref()
                .is_none_or(|t| turn_is_final(&t.state))
                && s.session
                    .as_ref()
                    .is_none_or(|s| s.active_turn_id.is_none()),
        })
        .collect();
    let result = crate::communication::client::human_agent_tracking(
        &paths.socket,
        AgentTrackingRequest::Sync {
            source_host: crate::wrapper::host_name(),
            observations,
            confirmations: reports,
        },
    )
    .map_err(|e| e.to_string())?;
    if result["status"] != "ok" {
        return Err(result["code"]
            .as_str()
            .unwrap_or("classement non confirmé")
            .into());
    }
    // L'acceptation native précède l'effacement. Une perte de réponse reprend
    // les mêmes reçus ; une perte du cache reprend la tentative gravée au maître.
    save_reports(&receipt_path, &[])?;
    let pending = result["pending"]
        .as_array()
        .filter(|p| p.len() <= 1)
        .ok_or("file native de classement invalide")?;
    let Some(operation) = pending.first() else {
        return Ok(());
    };
    let thread = operation["thread_id"].as_str().ok_or("fil natif absent")?;
    let id = operation["operation_id"]
        .as_str()
        .ok_or("opération native absente")?;
    let command = operation["client_command_id"]
        .as_str()
        .ok_or("commande client absente")?;
    let attempt = operation["attempt"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .ok_or("tentative native absente")?;
    let dormant = operation["dormant"].as_bool().ok_or("état natif absent")?;
    let outcome = match session.call(|client| client.organize_thread(thread, command, dormant)) {
        Ok(sequence) => AgentTrackingSyncOutcome::Applied { sequence },
        Err(ContractError::CommandRejected { command_id }) if command_id == command => {
            AgentTrackingSyncOutcome::Rejected {}
        }
        Err(_) => {
            return Err(format!(
                "issue du classement inconnue ; opération {id}, commande {command}, tentative {attempt} conservées ; aucune nouvelle clé"
            ));
        }
    };
    save_reports(
        &receipt_path,
        &[AgentTrackingConfirmation {
            thread_id: thread.into(),
            operation_id: id.into(),
            attempt,
            outcome,
        }],
    )
}
