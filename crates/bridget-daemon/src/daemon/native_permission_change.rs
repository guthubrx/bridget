//! Diagnostic fermé des mêmes comparaisons strictes. Aucune valeur de droits n'est publiée.
use super::*;
use bridget_transport::protocol::ProviderPermissions;
const STAGES: &[&str] = &["after_inspection", "after_workspace", "before_start",
    "recovery_before_start", "before_release_attestation", "before_release_parent"];
const FIELDS: &[&str] = &["version", "source", "run_id", "provider_session_id",
    "provider_instance_id", "revision", "driver", "cwd", "runtime_mode",
    "interaction_mode", "provider_policy", "parent_missing", "thread",
    "connection_identity", "child_policy", "definition", "permission_sources"];
const OTHER_WORK: &str = "Ce refus ne vous demande pas de suspendre vos autres chantiers. Continuez leurs travaux autorisés selon leur propre état. Leur lancement reste soumis aux contrôles habituels de Bridget.";
const HUMAN: &str = "Vous pouvez actualiser l’attestation de votre propre session avec bridget_session. Si un nouveau lancement autorisé échoue encore avec ce code après cette actualisation, transmettez task_id, request_id et detail.permission_change au responsable Bridget. Relire l’erreur d’une ancienne tâche ne constitue pas un nouvel échec. N’attendez pas une maintenance globale pour ce seul refus. Demandez une décision à l’utilisateur seulement si un changement réel de droits ou de session sort de votre mandat.";

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
fn message(code: &str) -> String {
    let Some((_, changes)) = detail(code) else {
        return "Le lancement de cette mission a été refusé pendant le contrôle des permissions. Le détail de la différence n’a pas été conservé. Ce message ne déclare pas tout Bridget ou le serveur indisponible.".into();
    };
    let labels: Vec<_> = changes.iter().map(|field| match *field {
        "version" => "format de la preuve",
        "source" => "origine de la preuve",
        "run_id" => "tour de travail du demandeur",
        "provider_session_id" => "session du demandeur",
        "provider_instance_id" => "identité du demandeur",
        "revision" => "version de l’attestation",
        "driver" => "pilote de la session",
        "cwd" => "dossier du demandeur",
        "runtime_mode" => "mode de permissions",
        "interaction_mode" => "mode de travail",
        "provider_policy" => "règles de permissions",
        "parent_missing" => "preuve du demandeur absente",
        "thread" => "conversation du demandeur",
        "connection_identity" => "identité de connexion",
        "child_policy" => "permissions prévues pour l’agent",
        "definition" => "définition de l’agent",
        "permission_sources" => "sources des permissions",
        _ => "preuve du demandeur",
    }).collect();
    format!("Le lancement de cette mission a été refusé. Différence constatée : {}. Ce refus concerne cette demande ; il ne constate pas une panne générale.", labels.join(", "))
}
pub(super) fn failure(code: &str) -> Value {
    json!({"status":"refused","code":"permission_changed","message":message(code),
        "detail":{"permission_change":projection(code),
        "recovery":format!("Pour cette demande : conservez sa request_id et ses paramètres. Lisez son reçu et, si une tâche existe, son statut. Une réponse perdue ou une admission pending peut encore aboutir : ne créez pas de doublon. Une tâche failed ou cancelled ne redémarre pas avec son ancienne clé ; suivez le conseil de son statut. Actualisez l’attestation de CE demandeur avec bridget_session. {OTHER_WORK}"),
        "human_action":HUMAN}})
}
pub(super) fn project_task(task: &Task, view: &mut Value) {
    let Some(raw) = task.error.as_deref().and_then(|e| e.strip_prefix("remote_start_failed:")) else { return; };
    if !recognizes(raw) { return; }
    view["error"] = json!("remote_start_failed:permission_changed");
    view["error_message"] = json!(if task.terminal() {
        format!("{} Ce message décrit l’échec de l’ancienne tentative ; il ne vérifie pas vos droits actuels.", message(raw))
    } else { message(raw) });
    if !view["detail"].is_object() { view["detail"] = json!({}); }
    view["detail"]["permission_change"] = projection(raw);
    let action = if task.terminal() && task.cleanup_done && workspace_report_ready(task) {
        "Cette tentative est arrêtée. Pour reprendre SON travail : 1. Conservez les fichiers et vérifiez le dossier ; report_unavailable impose une inspection du dossier et de ses usages. 2. Clôturez la garde depuis le parent propriétaire, puis actualisez son attestation avec bridget_session. 3. Faites une nouvelle admission autorisée, avec nouvelle request_id et passation du seul travail restant. L’ancienne tâche ne redémarrera ni par Continue ni par rejeu de sa clé."
    } else if task.terminal() && task.cleanup_done {
        "Cette tentative est arrêtée, mais son rapport de dossier n’est pas encore disponible. Conservez les fichiers et demandez le statut de CETTE tâche avant de reprendre son travail. Ne créez pas de remplacement tant que le dossier n’est pas vérifié."
    } else {
        "Pour CETTE mission, Bridget n’a pas encore confirmé la fin de la tentative et la vérification du dossier. Conservez sa task_id, sa request_id et ses fichiers. Consultez son statut avant de reprendre SON travail. Ne rejouez pas ce refus comme un Start et ne créez pas de remplacement pendant cette vérification. Une attestation fraîche ne redémarre pas l’ancienne tâche."
    };
    view["error_recovery"] = json!(format!("{action} {OTHER_WORK}"));
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
    #[test]
    fn spec206_scope_and_next_step_are_explicit_without_global_permission_claim() {
        for raw in ["permission_changed", "permission_changed:before_start:revision"] {
            let view = failure(raw);
            let advice = view["detail"]["recovery"].as_str().unwrap();
            assert!(view["message"].as_str().unwrap().contains("cette mission"));
            assert!(advice.contains("ne vous demande pas de suspendre vos autres chantiers"));
            assert!(advice.contains("selon leur propre état"));
            assert!(advice.contains("contrôles habituels"));
            assert!(!advice.contains("tout est disponible"));
            assert!(!advice.contains("aucun blocage"));
        }
        let legacy=failure("permission_changed");
        assert!(legacy["message"].as_str().unwrap().contains("n’a pas été conservé"));
        let known=failure("permission_changed:before_start:revision");
        assert!(known["message"].as_str().unwrap().contains("version de l’attestation"));
        assert!(!known["message"].as_str().unwrap().contains("droits révoqués"));
    }

}
