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
pub(super) fn view_error_code(reason:WorkerRefusal,failure:bridget_transport::protocol::CommitViewFailure)->String {
    if failure.validate(reason).is_err() {return "remote_commit_view_prepare:workspace_prepare_failed:cause=unknown".into();}
    format!("remote_commit_view_prepare:{}:cause={}",serde_json::to_value(reason).unwrap().as_str().unwrap(),
        serde_json::to_value(failure).unwrap().as_str().unwrap())
}

fn parse_view(error:&str)->Option<(WorkerRefusal,bridget_transport::protocol::CommitViewFailure)> {
    let suffix=error.strip_prefix("remote_start_failed:remote_commit_view_prepare:")?;
    if suffix.len()>160 {return None;}
    let (reason,cause)=suffix.split_once(":cause=")?;
    let reason:WorkerRefusal=serde_json::from_value(json!(reason)).ok()?;
    let failure:bridget_transport::protocol::CommitViewFailure=serde_json::from_value(json!(cause)).ok()?;
    failure.validate(reason).ok()?;Some((reason,failure))
}

pub(super) fn view_failure_action(cause:bridget_transport::protocol::CommitViewFailure)->&'static str {
    use bridget_transport::protocol::CommitViewFailure as V;
    match cause {
        V::ObjectUnavailable=>"Faire publier ou rétablir le commit exact et ses objets dans le dépôt cible ou son miroir configuré. Ne pas substituer HEAD ou une autre base.",
        V::DiskFull=>"Faire vérifier l'espace et le quota du volume choisi pour la vue, puis ranger seulement les données dont la clôture et les usages sont établis.",
        V::AccessDenied=>"Faire vérifier l'accès du compte worker au volume choisi. Ne pas élargir les droits pour forcer la préparation.",
        V::OperationCapacity=>"Une capacité de préparation ou de lecture de vue est déjà occupée sur ce worker. Attendre la fin attestée de l'opération déjà en cours, puis relire l'état de la même tâche. Si l'arrêt reste inconnu ou l'attente persiste, faire inventorier les opérations et marqueurs de ce worker par le responsable ; demander à l'utilisateur l'accès de diagnostic manquant. Ne pas interrompre un autre travail ni changer de machine pour contourner sa garde.",
        V::MetadataCapacity=>"Faire inventorier les reçus et vues conservés de ce worker. Clôturer seulement les préparations dont les preuves le permettent ; ne pas augmenter le quota pour masquer une conservation inconnue.",
        V::DestinationInsideRepository=>"Faire corriger le placement de la vue hors du dépôt source. Le checkout partagé et ses fichiers restent à préserver.",
        V::DestinationVolumeMismatch=>"Faire préparer un emplacement privé hors du dépôt, sur son volume autorisé. Ne pas déplacer une tâche active.",
        V::DestinationUnavailable=>"Faire préparer par le responsable un parent privé appartenant au compte worker, hors du dépôt et sur le même volume.",
        V::DestinationUnsafe=>"Faire inspecter le parent et la réserve de vues : propriétaire, accès et liens doivent respecter le placement privé. Ne pas remplacer ni modifier un dossier existant à l'aveugle.",
        V::StorageSpaceUnknown=>"Faire vérifier pourquoi l'espace libre du volume n'a pas pu être observé. Ce constat ne prouve pas un disque plein.",
        V::StopUnconfirmed=>"Faire vérifier le marqueur privé, l'identité et le groupe du helper de cette préparation. Conserver les fichiers et la preuve tant que son arrêt reste inconnu.",
        V::Cancelled=>"Cette opération a reçu une annulation. Vérifier sa clôture native et conserver ses fichiers ; ne pas la remettre en marche.",
        V::PreparationExpired|V::TimedOut=>"Faire vérifier la durée de l'export ou de la lecture et de son audit, puis constater l'arrêt du helper avant toute autre admission de ce travail.",
        V::ByteLimit=>"Faire qualifier le nombre et le volume des fichiers sélectionnés pour cette vue. Ne pas relever les bornes ou créer un worktree pour contourner le refus.",
        V::LinkOutsideView|V::LinkCycle|V::UnsupportedPath=>"Faire examiner les chemins et liens du commit demandé. Préserver le dépôt ; corriger la source ou décider avec son propriétaire d'un parcours compatible.",
        V::IntegrityFailed=>"Conserver la vue et ses preuves. Faire examiner les fichiers divergents avant toute décision de rangement ; aucun retrait automatique n'est autorisé.",
        V::SourceIdentityChanged=>"Faire vérifier la correspondance du projet et l'identité Git de la source. Ne pas accepter une autre source sous la même clé.",
        V::StartupUnavailable=>"Faire vérifier le binaire Bridget et le bootstrap géré du worker. Ne pas en déduire une panne du fournisseur ou un défaut de compte.",
        V::SourceUnavailable=>"Faire vérifier la disponibilité du dépôt source et des objets du commit exact. Aucun changement de base n'est autorisé.",
        V::ProofUnavailable=>"Conserver les fichiers et demander une inspection corrélée du reçu, de la graine et des preuves privées de cette préparation.",
        V::Unknown=>"La cause précise reste inconnue. Transmettre au responsable Bridget la tâche, sa clé et sa machine pour lire le reçu corrélé ; ne pas déduire un problème de compte, de réseau ou de stockage.",
    }
}

fn project_view(task:&Task,view:&mut Value,error:&str) {
    use bridget_transport::protocol::CommitViewFailure as V;
    let known=parse_view(error);
    let cause=known.map(|(_,cause)|cause).unwrap_or(V::Unknown);
    let action=view_failure_action(cause);
    view["error_message"]=json!(format!("{action} Portée : la préparation de cette tâche. Ce refus ne qualifie ni toute la machine ni un arrêt global. Les autres travaux autorisés suivent leur propre état."));
    view["error_recovery"]=json!(if task.cleanup_done && task.terminal() {
        "La conservation native est confirmée. Inspecter et préserver les fichiers éventuels, puis corriger la cause avant une nouvelle admission autorisée avec une nouvelle clé. Le rejeu de cette clé ne relance pas la préparation."
    } else {
        "Conserver cette tâche, sa clé et les fichiers éventuels. Son propriétaire doit demander l'annulation puis attendre la clôture native et inspecter les preuves avant une nouvelle admission autorisée après correction. Une absence de dossier ne prouve pas l'arrêt. Le rejeu de cette clé ne relance pas la préparation."
    });
    view["error_human_action"]=json!(format!("Si cette action dépasse vos droits, demander à l'utilisateur de l'autoriser uniquement pour cette tâche et cette machine : {action} Ne pas créer une mission de remplacement tant que l'issue et la conservation restent inconnues."));
    if !view["detail"].is_object(){view["detail"]=json!({});}
    view["detail"]["commit_view_failure"]=json!({"cause":cause,"diagnostic_available":known.is_some(),
        "task_id":task.task_id,"request_id":match &task.request {NativeDelegationRequest::Delegate{request_id,..}=>Some(request_id),_=>None},
        "machine_id":task.remote_placement.as_ref().map(|placement|&placement.machine_id)});
}

pub(super) fn project(task: &Task, view: &mut Value) {
    let Some(error) = task.projected_error() else { return; };
    if error.starts_with("remote_start_failed:remote_commit_view_prepare:") {
        project_view(task,view,error);return;
    }
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
    fn spec221_c19_failure_code_is_closed_and_never_imports_private_text() {
        use bridget_transport::protocol::CommitViewFailure as V;
        for cause in [V::ObjectUnavailable,V::DiskFull,V::StopUnconfirmed,V::DestinationUnavailable,V::Unknown] {
            let code=format!("remote_start_failed:{}",view_error_code(WorkerRefusal::WorkspacePrepareFailed,cause));
            assert_eq!(parse_view(&code),Some((WorkerRefusal::WorkspacePrepareFailed,cause)));
            assert!(parse_view(&(code+":PRIVATE/path")).is_none());
        }
        assert!(parse_view("remote_start_failed:remote_commit_view_prepare:workspace_prepare_failed:cause=PRIVATE").is_none());
        assert!(parse_view("remote_start_failed:remote_commit_view_prepare:unauthorized:cause=disk_full").is_none());
    }

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
