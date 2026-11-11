//! Construction canonique commune : aucune dépendance aux façades CLI/MCP.
//!
//! Le scope identifie un namespace stable ; ce hash n'est pas une preuve
//! d'authentification. L'autorisation appartient à la connexion négociée.

pub(crate) mod client;

use bridget_transport::protocol::{
    CommunicationProject, CommunicationProjectSource, ProjectRelation, ProjectWarning,
};

pub(crate) fn validate_cross_project_reason(
    reason: Option<&str>,
) -> Result<Option<String>, String> {
    reason.map(|value| {
        if value.chars().any(char::is_control) { return Err("invalid_cross_project_reason : caractères de contrôle interdits".into()); }
        let value = value.trim();
        if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
            return Err("invalid_cross_project_reason : motif requis de 1 à 512 octets UTF-8, sans contrôles".into());
        }
        Ok(value.to_owned())
    }).transpose()
}

pub(crate) fn project_relation(
    sender: Option<&CommunicationProject>,
    recipient: Option<&CommunicationProject>,
) -> ProjectRelation {
    match (sender, recipient) {
        (Some(a), Some(b)) if a == b => ProjectRelation::Same,
        (Some(_), Some(_)) => ProjectRelation::Other,
        _ => ProjectRelation::Unknown,
    }
}

pub(crate) fn project_scope(
    sender: Option<&CommunicationProject>,
    recipient: Option<&CommunicationProject>,
    recipient_id: &str,
    reason: Option<&str>,
) -> Result<Vec<ProjectWarning>, String> {
    let reason = validate_cross_project_reason(reason)?;
    let relation = project_relation(sender, recipient);
    if relation == ProjectRelation::Other && reason.is_none() {
        return Err("cross_project_reason_required : autre projet, préciser --cross-project-reason avec un motif volontaire. Ne modifiez pas le contenu d’un envoi déjà identifié sous la même clé. Si le reçu atteste un refus définitif sans remise, le motif corrigé appartient à une nouvelle demande avec sa propre clé. Une issue inconnue se relit avec la même clé et la même enveloppe.".into());
    }
    if relation == ProjectRelation::Same {
        return Ok(Vec::new());
    }
    Ok(vec![ProjectWarning {
        code: if relation == ProjectRelation::Other {
            "cross_project"
        } else {
            "project_unknown"
        }
        .into(),
        sender_project: sender.cloned(),
        recipient_project: recipient.cloned(),
        recipient: recipient_id.into(),
        reason,
    }])
}

/// Git est résolu une seule fois, hors verrou daemon. Aucun nom de domaine
/// ni basename ne prouve l'appartenance ; les worktrees partagent common-dir.
pub(crate) fn resolve_communication_project(
    root: &str,
    host: &str,
    source: CommunicationProjectSource,
    worktree_root: Option<&str>,
) -> Option<CommunicationProject> {
    if !bridget_core::host_is_attested(host)
        || root.is_empty()
        || root.len() > 4096
        || root.chars().any(char::is_control)
    {
        return None;
    }
    let path = std::path::Path::new(root);
    if !path.is_absolute() {
        return None;
    }
    let path = path.canonicalize().ok()?;
    if !path.is_dir() {
        return None;
    }
    fn git_common(path: &std::path::Path) -> Option<std::path::PathBuf> {
        let output = std::process::Command::new("git")
            .args(["-C"])
            .arg(path)
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_CEILING_DIRECTORIES")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let value = String::from_utf8(output.stdout).ok()?;
        std::path::Path::new(value.trim()).canonicalize().ok()
    }
    let common = git_common(&path);
    let resolved = match (source, common) {
        (_, Some(common)) => common,
        (CommunicationProjectSource::T3, None) => path.clone(),
        _ => return None,
    };
    if let Some(worktree) = worktree_root {
        let candidate = std::path::Path::new(worktree);
        if !candidate.is_absolute() {
            return None;
        }
        let candidate = candidate.canonicalize().ok()?;
        let candidate_root = git_common(&candidate).unwrap_or(candidate);
        if candidate_root != resolved {
            return None;
        }
    }
    Some(CommunicationProject {
        host: host.to_owned(),
        root: resolved.to_string_lossy().into_owned(),
    })
}

pub(crate) fn issuer_scope(identity: &str) -> String {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x9e3779b97f4a7c15_u64;
    for byte in identity.bytes() {
        first = (first ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        second = second.rotate_left(5) ^ u64::from(byte);
        second = second.wrapping_mul(0x9e3779b185ebca87);
    }
    format!("012_scope_{first:016x}{second:016x}")
}

fn canonical_field(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value);
}

fn canonical_option<T: ToString>(bytes: &mut Vec<u8>, value: Option<T>) {
    match value {
        Some(value) => {
            bytes.push(1);
            canonical_field(bytes, value.to_string().as_bytes());
        }
        None => bytes.push(0),
    }
}

fn canonical_message_control(bytes: &mut Vec<u8>, message: &bridget_core::BridgetMessage) {
    // Les anciens clients n'avaient pas ces champs. Ne rien ajouter pour leur
    // forme vide conserve leurs rejeux exacts ; une sémantique nouvelle ajoute
    // un suffixe distinct qui ne peut jamais réutiliser leur même identité.
    if message.origin.is_none() && message.intent.is_none() && message.references.is_empty() {
        return;
    }
    bytes.push(0xff);
    bytes.push(match message.origin {
        None => 0,
        Some(bridget_core::MessageOrigin::Human) => 1,
        Some(bridget_core::MessageOrigin::Agent) => 2,
        Some(bridget_core::MessageOrigin::Routine) => 3,
        Some(bridget_core::MessageOrigin::System) => 4,
    });
    bytes.push(match message.intent {
        None => 0,
        Some(bridget_core::MessageIntent::QueueOnly) => 1,
        Some(bridget_core::MessageIntent::TriggerTurn) => 2,
        Some(bridget_core::MessageIntent::SteerCurrent) => 3,
        Some(bridget_core::MessageIntent::InterruptAndStart) => 4,
        Some(bridget_core::MessageIntent::ControlOnly) => 5,
        Some(bridget_core::MessageIntent::StoreOnly) => 6,
    });
    canonical_field(bytes, &(message.references.len() as u64).to_be_bytes());
    for reference in &message.references {
        canonical_field(bytes, reference.as_bytes());
    }
}
/// Sérialisation binaire fermée et sans ambiguïté de l'enveloppe publiée.
/// Elle ne dépend ni de l'ordre JSON ni des valeurs mutées lors du routage.
pub(crate) fn canonical_send(
    issuer_scope: &str,
    message_id: &str,
    message: &bridget_core::BridgetMessage,
    issued_at: i64,
) -> Vec<u8> {
    let mut bytes = b"bridget/client-send/v1\0".to_vec();
    canonical_field(&mut bytes, issuer_scope.as_bytes());
    canonical_field(&mut bytes, message_id.as_bytes());
    canonical_field(&mut bytes, message.to.as_bytes());
    canonical_field(&mut bytes, message.body.as_bytes());
    bytes.push(u8::from(message.reply));
    canonical_field(&mut bytes, &message.hops.to_be_bytes());
    canonical_option(&mut bytes, message.deadline_at);
    canonical_option(&mut bytes, message.in_reply_to.as_deref());
    canonical_message_control(&mut bytes, message);
    canonical_field(&mut bytes, &issued_at.to_be_bytes());
    canonical_thread_notice(&mut bytes, message);
    if let Some(reason) = &message.cross_project_reason {
        bytes.extend_from_slice(b"bridget/cross-project/v1\0");
        canonical_field(&mut bytes, reason.as_bytes());
    }
    bytes
}

/// Session 102 : une alerte de fil ajoute un domaine distinct à la FIN du canon.
/// `None` conserve exactement les octets historiques ; chaque champ de la
/// notice modifie le canon, donc un rejeu avec une autre notice est refusé.
fn canonical_thread_notice(bytes: &mut Vec<u8>, message: &bridget_core::BridgetMessage) {
    let Some(notice) = &message.thread_notice else {
        return;
    };
    bytes.extend_from_slice(b"bridget/thread-notice/v1\0");
    canonical_field(bytes, &notice.version.to_be_bytes());
    canonical_field(bytes, notice.thread_id.as_bytes());
    canonical_field(bytes, &notice.through_seq.to_be_bytes());
    canonical_field(bytes, &notice.generation.to_be_bytes());
}

#[cfg(test)]
#[path = "communication_tests.rs"]
mod tests;

#[cfg(test)]
mod coordination247_tests {
    use super::*;
    #[test]
    fn spec247_motif_manquant_explique_la_cle_close_sans_casser_le_code() {
        let from = CommunicationProject {
            host: "mac".into(),
            root: "/a".into(),
        };
        let to = CommunicationProject {
            host: "sys".into(),
            root: "/b".into(),
        };
        let error = project_scope(Some(&from), Some(&to), "dest", None).unwrap_err();
        assert!(error.starts_with("cross_project_reason_required :"));
        assert!(error.contains("nouvelle demande") && error.contains("même enveloppe"));
        assert!(
            project_scope(
                Some(&from),
                Some(&to),
                "dest",
                Some("coordination autorisée")
            )
            .is_ok()
        );
    }
}

#[cfg(test)]
#[test]
fn spec251_canon_information_distinct_sans_changer_ancien() {
    let mut message = bridget_core::BridgetMessage::new("a", "b", "avis");
    let old = canonical_send("scope", "id", &message, 123);
    message.intent = Some(bridget_core::MessageIntent::StoreOnly);
    assert_ne!(old, canonical_send("scope", "id", &message, 123));
    let wire = serde_json::to_value(&message).unwrap();
    assert_eq!(wire["intent"], "store_only");
    assert_eq!(
        serde_json::from_value::<bridget_core::BridgetMessage>(wire).unwrap(),
        message
    );
}
