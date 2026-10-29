//! Diagnostic fermé des mêmes comparaisons strictes. Aucune valeur de droits n'est publiée.
use super::*;
use bridget_transport::protocol::ProviderPermissions;
const STAGES: &[&str] = &["after_inspection", "after_workspace", "before_start",
    "recovery_before_start", "before_release_attestation", "before_release_parent"];
const FIELDS: &[&str] = &["version", "source", "run_id", "provider_session_id",
    "provider_instance_id", "revision", "driver", "cwd", "runtime_mode",
    "interaction_mode", "provider_policy", "parent_missing", "thread",
    "connection_identity", "child_policy", "definition", "permission_sources"];
const MESSAGE: &str = "La preuve de permissions ou l’identité du parent a changé pendant le lancement. Le permis de remise de mission a été refusé ; cela ne prouve pas une panne de compte ou de serveur.";
const HUMAN: &str = "Si le refus revient après une attestation fraîche dans le même tour, transmettez task_id, request_id et detail.permission_change au responsable Bridget. Il doit comparer les catégories constatées ; ne changez ni droits, moteur ni machine pour contourner le refus. Demandez à l’utilisateur une décision uniquement si les droits ou la session doivent réellement être changés.";

pub(super) fn code(stage: &str, fields: &[&str]) -> String {
    let code = format!("permission_changed:{stage}:{}", fields.join(","));
    if detail(&code).is_some() { code } else { "permission_changed".into() }
}
pub(super) fn recognizes(code: &str) -> bool {
    code == "permission_changed" || code.starts_with("permission_changed:")
}
fn detail(code: &str) -> Option<(&str, Vec<&str>)> {
    if code.len() > 512 { return None; }
    let (stage, fields) = code.strip_prefix("permission_changed:")?.split_once(':')?;
    if !STAGES.contains(&stage) { return None; }
    let fields: Vec<_> = fields.split(',').collect();
    if fields.is_empty() || fields.iter().enumerate().any(|(i, field)|
        !FIELDS.contains(field) || fields[..i].contains(field)) { return None; }
    Some((stage, fields))
}
pub(super) fn parent_changes(expected: &ProviderPermissions, current: Option<&ProviderPermissions>) -> Vec<&'static str> {
    let Some(current) = current else { return vec!["parent_missing"]; };
    let mut changes = Vec::new();
    macro_rules! field { ($name:ident) => {
        if expected.$name != current.$name { changes.push(stringify!($name)); }
    }; }
    field!(version); field!(source); field!(run_id); field!(provider_session_id);
    field!(provider_instance_id); field!(revision); field!(driver); field!(cwd);
    field!(runtime_mode); field!(interaction_mode); field!(provider_policy);
    changes
}
pub(super) fn check_parent(stage: &str, expected: &ProviderPermissions,
    current: Option<&ProviderPermissions>, thread_changed: bool) -> Result<(), String> {
    let mut changes = parent_changes(expected, current);
    if thread_changed { changes.push("thread"); }
    if changes.is_empty() { Ok(()) } else { Err(code(stage, &changes)) }
}
fn projection(code: &str) -> Value {
    let (stage, changes) = detail(code).unwrap_or(("unobserved", Vec::new()));
    json!({"stage":stage,"changes":changes,"evidence_available":stage!="unobserved",
        "comparison":"strict","historical_cause":if stage=="unobserved" {"not_recorded_or_invalid"} else {"observed_difference"}})
}
pub(super) fn failure(code: &str) -> Value {
    json!({"status":"refused","code":"permission_changed","message":MESSAGE,
        "detail":{"permission_change":projection(code),
        "recovery":"Conservez request_id et enveloppe. Consultez l’état durable de cette admission avant toute reprise : une réponse perdue ou une tâche connue interdit un remplacement. Réattestez CE parent avec bridget_session. Un reçu admitted conserve sa tâche ; une tâche failed ne repart ni par Continue ni par rejeu. Une issue pending sans tâche garde sa clé et son enveloppe ; ne créez pas de doublon.",
        "human_action":HUMAN}})
}
pub(super) fn project_task(task: &Task, view: &mut Value) {
    let Some(raw) = task.error.as_deref().and_then(|e| e.strip_prefix("remote_start_failed:")) else { return; };
    if !recognizes(raw) { return; }
    view["error"] = json!("remote_start_failed:permission_changed");
    view["error_message"] = json!(MESSAGE);
    if !view["detail"].is_object() { view["detail"] = json!({}); }
    view["detail"]["permission_change"] = projection(raw);
    view["error_recovery"] = json!(if task.terminal() && task.cleanup_done && workspace_report_ready(task) {
        "L’arrêt natif est confirmé. Vérifiez le dernier rapport du dossier : report_unavailable impose une inspection du dossier et de ses usages ; préservez fichiers, commits et sorties. Après cette vérification et la clôture de la garde propriétaire, réattestez CE parent avec bridget_session avant une nouvelle admission autorisée, avec nouvelle request_id et passation du seul travail restant. Cette tâche failed ne reprend ni par Continue ni par rejeu de sa clé."
    } else {
        "Conservez cette tâche, sa request_id et son dossier. Consultez son statut natif jusqu’à la confirmation d’arrêt, de nettoyage et du rapport du dossier. Ne rejouez pas ce refus comme un Start et ne créez pas de remplacement pendant cette vérification. Une attestation fraîche du parent n’annule pas le refus historique."
    });
    view["error_human_action"] = json!(HUMAN);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parent() -> ProviderPermissions {
        ProviderPermissions {version:1,source:"provider_turn".into(),run_id:"run-private".into(),
            provider_session_id:"session-private".into(),provider_instance_id:"instance-private".into(),
            revision:3,driver:"claude_stream_json".into(),cwd:"/private-parent".into(),
            runtime_mode:"full-access".into(),interaction_mode:"default".into(),
            provider_policy:json!({"private":"secret"})}
    }
    #[test]
    fn spec206_strict_comparison_keeps_revision_run_and_policy_guards() {
        let p=parent();
        assert!(check_parent("before_start",&p,Some(&p),false).is_ok());
        let mut other=p.clone(); other.revision+=1; other.run_id="new-private-run".into();
        let err=check_parent("before_release_attestation",&p,Some(&other),false).unwrap_err();
        assert_eq!(detail(&err),Some(("before_release_attestation",vec!["run_id","revision"])));
        assert!(!err.contains("private"));
        other=p.clone();other.provider_policy=json!({"secret":"different"});
        assert_eq!(parent_changes(&p,Some(&other)),vec!["provider_policy"]);
        assert_eq!(parent_changes(&p,None),vec!["parent_missing"]);
        assert_eq!(check_parent("before_start",&p,Some(&p),true),
            Err("permission_changed:before_start:thread".into()));
        // Une différence de chaque champ doit conserver exactement le même refus.
        let base=serde_json::to_value(&p).unwrap();
        for field in &FIELDS[..11] {
            let mut value=base.clone();
            value[*field]=if *field=="version" || *field=="revision" {json!(7)}
                else if *field=="provider_policy" {json!({"different":true})}
                else {json!("different")};
            let changed:ProviderPermissions=serde_json::from_value(value).unwrap();
            assert_ne!(p,changed);
            assert_eq!(parent_changes(&p,Some(&changed)),vec![*field]);
        }
    }
    #[test]
    fn spec206_closed_projection_excludes_private_values_and_legacy_inference() {
        for stage in STAGES { for field in FIELDS {
            let code=code(stage,&[*field]);
            let projected=failure(&code);
            assert_eq!(projected["code"],"permission_changed");
            assert_eq!(projected["detail"]["permission_change"]["stage"],*stage);
            assert_eq!(projected["detail"]["permission_change"]["changes"],json!([field]));
        }}
        for invalid in ["permission_changed", "permission_changed:private:revision",
            "permission_changed:before_start:private-secret",
            "permission_changed:before_start:revision,revision",
            "permission_changed:before_start:"] {
            let out=failure(invalid);
            assert_eq!(out["detail"]["permission_change"]["stage"],"unobserved");
            assert_eq!(out["detail"]["permission_change"]["changes"],json!([]));
            assert!(!out.to_string().contains("private-secret"));
        }
        assert!(detail(&format!("permission_changed:before_start:{}", "x".repeat(600))).is_none());
    }
}
