//! Cause de la préparation courante. Le worker est sériel ; les threads de sonde
//! ont leur propre emplacement. Aucun texte libre ne sort de ce module.
use bridget_transport::protocol::{WorkspacePrepareFailure, WorkspacePrepareCause as C, WorkspacePrepareStage as S};
thread_local! {
    static FAILURE: std::cell::Cell<Option<WorkspacePrepareFailure>> = const { std::cell::Cell::new(None) };
}
pub(crate) fn clear() { FAILURE.with(|v| v.set(None)); }
pub(crate) fn take() -> Option<WorkspacePrepareFailure> { FAILURE.with(|v| v.take()) }
pub(crate) fn stage<T, E: std::fmt::Display>(stage: S, result: Result<T,E>) -> Result<T,E> {
    if let Err(error) = &result {
        let text = error.to_string();
        let generated=text.strip_prefix("workspace_create_failed: ").unwrap_or(&text);
        let cause = if generated.starts_with("délai Git dépassé") { C::GitTimeout }
        else if generated.starts_with("sortie Git tronquée")
            || generated.starts_with("workspace_repo_unsupported: sortie Git tronquée") { C::GitOutputTruncated }
        else {
            match text.split(':').next().unwrap_or("") {
                "workspace_disk_full" => C::DiskFull,
                "workspace_access_denied" => C::AccessDenied,
                "workspace_repo_unsupported" => C::RepoUnsupported,
                "workspace_branch_exists" => C::BranchExists,
                "workspace_path_unsafe" => C::PathUnsafe,
                "workspace_limit" => C::WorktreeLimit,
                "workspace_git_unsupported" => C::GitUnavailable,
                _ if generated.starts_with("Git ") => C::GitCommandFailed,
                _ => C::Unknown,
            }
        };
        FAILURE.with(|v| if v.get().is_none() { v.set(Some(WorkspacePrepareFailure {stage,cause})); });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec210_failure_is_closed_and_first_inner_stage_wins() {
        clear();
        let result:Result<(),String>=stage(S::CompactPlan,Err("délai Git dépassé; private-token".into()));
        let _=stage(S::Create,result);
        assert_eq!(take(),Some(WorkspacePrepareFailure {stage:S::CompactPlan,cause:C::GitTimeout}));
        assert_eq!(take(),None);
        let _:Result<(),String>=stage(S::Create,Err("Git Some(1) : private/path secret-token".into()));
        let wire=serde_json::to_string(&take().unwrap()).unwrap();
        assert_eq!(wire,r#"{"stage":"create","cause":"git_command_failed"}"#);
        let _:Result<(),String>=stage(S::Create,Err("Git Some(1) : filename délai Git dépassé".into()));
        assert_eq!(take().unwrap().cause,C::GitCommandFailed);
        clear();
        let _=stage(S::Report,Ok::<_,String>(()));
        assert!(take().is_none());
    }
}
