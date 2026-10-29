use super::*;
use bridget_transport::protocol::{WorkspacePrepareCause as C, WorkspacePrepareFailure as F, WorkspacePrepareStage as S, WorkerRefusal};

pub(super) fn error_code(reason: WorkerRefusal, failure: F) -> String {
    // Le contrat a déjà été validé dans la réponse corrélée ; défense locale.
    if failure.validate(reason).is_err() { return "remote_workspace_prepare:WorkspacePrepareFailed".into(); }
    let value=serde_json::to_value(failure).unwrap();
    format!("remote_workspace_prepare:{reason:?}:stage={}:cause={}",
        value["stage"].as_str().unwrap(), value["cause"].as_str().unwrap())
}
fn parse(error: &str) -> Option<F> {
    let suffix = error.strip_prefix("remote_start_failed:remote_workspace_prepare:")?;
    let mut fields = suffix.split(':');
    let reason = match fields.next()? {
        "WorkspacePrepareFailed" => WorkerRefusal::WorkspacePrepareFailed,
        "DiskFull" => WorkerRefusal::DiskFull,
        "FilesystemAccessDenied" => WorkerRefusal::FilesystemAccessDenied,
        _ => return None,
    };
    let stage = fields.next()?.strip_prefix("stage=")?;
    let cause = fields.next()?.strip_prefix("cause=")?;
    if fields.next().is_some() || suffix.len() > 160 { return None; }
    let failure = F {
        stage: serde_json::from_value(json!(stage)).ok()?,
        cause: serde_json::from_value(json!(cause)).ok()?,
    };
    failure.validate(reason).ok()?;
    Some(failure)
}
pub(super) fn project(task: &Task, view: &mut Value) {
    let Some(error) = task.projected_error() else { return; };
    if !error.starts_with("remote_start_failed:remote_workspace_prepare:") { return; }
    let suffix=error.strip_prefix("remote_start_failed:remote_workspace_prepare:").unwrap();
    let recognized = matches!(suffix, "WorkspacePrepareFailed" | "WorkspacePrepareIncomplete"
        | "DiskFull" | "FilesystemAccessDenied") || parse(error).is_some();
    if !recognized { return; }
    let failure = parse(error).unwrap_or(F {stage:S::Unknown,cause:match suffix {
        "DiskFull"=>C::DiskFull, "FilesystemAccessDenied"=>C::AccessDenied, _=>C::Unknown
    }});
    let action = match failure.cause {
        C::GitTimeout => "Faire vérifier la durée de la lecture ou de la commande Git indiquée et la disponibilité du stockage sur la machine cible. Vérifier aussi la fin du processus Git avant une nouvelle préparation. Ne pas déduire un disque plein de ce délai.",
        C::GitOutputTruncated => "Faire qualifier la taille de l’arbre ou de l’index Git et le budget de lecture du profil compact sur la machine cible.",
        C::RepoUnsupported => "Faire vérifier le profil compact et la configuration Git du dépôt cible ; corriger le réglage refusé avant une future admission.",
        C::BranchExists => "Faire inspecter la branche déjà présente et ses usages. Ne pas l’effacer ou la réutiliser à l’aveugle.",
        C::PathUnsafe => "Faire vérifier le chemin préparé, ses liens et son inclusion dans le placement autorisé. Ne pas élargir les droits pour passer.",
        C::WorktreeLimit => "Faire inventorier les worktrees enregistrés et ranger uniquement ceux dont le travail est préservé et la clôture confirmée.",
        C::GitUnavailable => "Faire vérifier le programme Git utilisé par le worker et sa version requise sur la machine cible.",
        C::DiskFull => "Faire vérifier l’espace et le quota du volume réellement ciblé, puis libérer seulement les fichiers dont la conservation et les usages ont été contrôlés.",
        C::AccessDenied => "Faire vérifier l’accès du compte worker au dépôt cible et l’état lecture seule du volume. Ne pas changer les permissions sans diagnostic.",
        C::GitCommandFailed | C::Unknown => "Transmettre la tâche et sa clé au responsable Bridget pour une lecture corrélée du reçu et du journal worker. Le détail Git historique peut manquer ; ne pas inventer de cause ni lancer une autre mission pour diagnostiquer.",
    };
    let unknown = if failure.cause == C::Unknown { " La cause précise reste inconnue." } else { "" };
    view["error_message"] = json!(format!("{action}{unknown} Portée : la préparation de cette tâche. Ce diagnostic ne qualifie pas la machine entière ni un arrêt global. Les autres travaux autorisés suivent leur propre état."));
    let closure = if task.cleanup_done && matches!(task.state.as_str(),"cancelled"|"failed") {
        "La clôture native est acquise. Inspecter et préserver le dossier éventuel, puis corriger la cause avant une nouvelle admission autorisée avec une nouvelle clé."
    } else {
        "Conserver la tâche, sa clé et le dossier éventuel. Le parent propriétaire doit annuler cette tâche, puis vérifier sa clôture native et inspecter le dossier avant une nouvelle admission autorisée après correction de la cause."
    };
    view["error_recovery"] = json!(format!("{closure} Le rejeu de cette clé ne relance pas la préparation."));
    view["error_human_action"] = json!(format!("Si cette intervention dépasse vos droits, demandez à l’utilisateur de l’autoriser sur la machine indiquée, avec cette tâche et sa clé : {action} Si la cause reste inconnue, demander d’abord l’accès au diagnostic manquant. Ne pas demander une remise à zéro générale ou un autre modèle. Ne pas lancer de nouvelle mission pour diagnostiquer ou remplacer une tâche dont l’issue reste inconnue."));
    if !view["detail"].is_object() { view["detail"]=json!({}); }
    let request_id = match &task.request { NativeDelegationRequest::Delegate {request_id,..} => Some(request_id), _ => None };
    view["detail"]["workspace_preparation"] = json!({
        "stage":failure.stage,"cause":failure.cause,"task_id":task.task_id,
        "request_id":request_id,"machine_id":task.remote_placement.as_ref().map(|p|&p.machine_id),
        "historical_detail_available":parse(error).is_some(),
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec210_closed_diagnostic_roundtrip_and_unknown_legacy() {
        let f=F{stage:S::CompactPlan,cause:C::GitTimeout};
        let code=format!("remote_start_failed:{}",error_code(WorkerRefusal::WorkspacePrepareFailed,f));
        assert_eq!(parse(&code),Some(f));
        assert!(parse("remote_start_failed:remote_workspace_prepare:WorkspacePrepareFailed").is_none());
        assert!(parse(&(code.clone()+":private")).is_none());
        assert!(parse(&code.replace("git_timeout","private/path")).is_none());
        assert!(parse(&code.replace("WorkspacePrepareFailed","DiskFull")).is_none());
    }
}
