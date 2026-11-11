//! Annulation du reçu avant création de Task, sans toucher aux messages suivis.
use super::*;

pub(super) fn cancel(st: &DaemonState, owner: &str, request_id: &str) -> Result<Value, String> {
    if request_id.is_empty()
        || request_id.len() > 128
        || request_id.chars().any(bridget_core::is_disallowed_control)
    {
        return Err("auto_admission_invalid".into());
    }
    let Some(receipt) = st
        .delegation_store
        .auto_admission_receipt(owner, request_id)?
    else {
        return Ok(
            json!({"version":1,"status":"not_recorded","request_id":request_id,"task_id":null,
            "code":"admission_not_recorded","detail":{"scope":"this_admission",
            "recovery":"Aucun reçu n’a été trouvé pour cette clé et ce propriétaire. Aucune annulation n’est enregistrée : un appel concurrent encore en précontrôle peut créer son reçu plus tard. Attendre la fin de ces appels puis vérifier la même clé ; ne pas rejouer l’admission ni créer une clé neuve pour annuler. Les autres travaux autorisés suivent leur propre état."}}),
        );
    };
    if receipt.state == AutoAdmissionState::Admitted {
        let id = receipt
            .task_id
            .as_deref()
            .ok_or("admission_task_unavailable")?;
        let task = st
            .delegation_store
            .get(id)?
            .filter(|task| task.owner == owner)
            .ok_or("admission_task_unavailable")?;
        return Ok(
            json!({"version":1,"status":"task_exists","code":"admission_already_admitted",
            "request_id":request_id,"task_id":task.task_id,"detail":{"scope":"this_admission",
            "recovery":"La création de tâche a précédé cette annulation d’admission. Pour arrêter ce travail devenu périmé, le propriétaire utilise bridget_task_cancel avec ce task_id, puis vérifie sa clôture native et conserve ses fichiers. Cette commande n’a annulé ni la tâche ni une autre demande. Les autres travaux autorisés suivent leur propre état."}}),
        );
    }
    // Le mutex du coordinateur est tenu. Le store reprend la révision dans sa
    // transaction Immediate ; admit_auto_task refuse un reçu devenu Refused.
    let receipt = if receipt.state == AutoAdmissionState::Refused {
        receipt
    } else {
        st.delegation_store.refuse_auto_admission(
            &receipt,
            "admission_cancelled",
            unix_timestamp(),
        )?
    };
    let cancelled = receipt.refusal_code.as_deref() == Some("admission_cancelled");
    Ok(
        json!({"version":1,"status":if cancelled {"cancelled"} else {"already_refused"},
        "code":receipt.refusal_code,"request_id":request_id,"task_id":null,
        "detail":{"scope":"this_admission","admission_state":"refused",
        "recovery":if cancelled {
            "L’admission est annulée durablement avant création de tâche. Un choix ou contrôle déjà en cours peut finir, mais son résultat ne peut plus admettre une tâche pour cette clé. Conserver la clé et les éventuels dossiers préparés : cette annulation ne les efface pas. Un rejeu identique ne relance pas l’admission ; une nouvelle mission exige une nouvelle clé et son autorisation propre. Les autres travaux autorisés suivent leur propre état."
        } else {
            "Cette admission était déjà refusée ; sa cause et son reçu sont conservés. Cette commande ne la reclasse pas comme annulée et ne crée aucune tâche. Les autres travaux autorisés suivent leur propre état."
        }}}),
    )
}
