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
        return Err("cross_project_reason_required : autre projet, préciser --cross-project-reason avec un motif volontaire".into());
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
mod tests {
    use super::*;

    #[test]
    fn spec138_scope_matrix_and_reason_bounds() {
        let a = CommunicationProject {
            host: "host".into(),
            root: "/a/.git".into(),
        };
        let b = CommunicationProject {
            host: "host".into(),
            root: "/b/.git".into(),
        };
        assert!(
            project_scope(Some(&a), Some(&a), "b", None)
                .unwrap()
                .is_empty()
        );
        assert!(
            project_scope(Some(&a), Some(&b), "b", None)
                .unwrap_err()
                .starts_with("cross_project_reason_required")
        );
        let warnings = project_scope(Some(&a), Some(&b), "b", Some("  shared review  ")).unwrap();
        assert_eq!(warnings[0].code, "cross_project");
        assert_eq!(warnings[0].reason.as_deref(), Some("shared review"));
        assert_eq!(
            project_scope(Some(&a), None, "b", None).unwrap()[0].code,
            "project_unknown"
        );
        for invalid in ["", "  ", "reason\n", "\treason", "rea\0son"] {
            assert!(validate_cross_project_reason(Some(invalid)).is_err());
        }
        assert!(validate_cross_project_reason(Some(&"é".repeat(256))).is_ok());
        assert!(validate_cross_project_reason(Some(&"é".repeat(257))).is_err());
    }

    #[test]
    fn spec138_git_environment_does_not_override_project() {
        const CHILD: &str = "BRIDGET_138_GIT_ENV_CHILD";
        if std::env::var_os(CHILD).is_some() {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .parent()
                .unwrap();
            assert!(
                resolve_communication_project(
                    root.to_str().unwrap(),
                    "test-host",
                    CommunicationProjectSource::Git,
                    None
                )
                .is_some()
            );
            return;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "communication::tests::spec138_git_environment_does_not_override_project",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env("GIT_DIR", "/nonexistent-bridget-138")
            .env("GIT_WORK_TREE", "/nonexistent-bridget-138-worktree")
            .env("GIT_COMMON_DIR", "/nonexistent-bridget-138-common")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn spec138_real_worktree_and_symlink_share_common_root() {
        let worktree = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let repository = std::process::Command::new("git")
            .arg("-C")
            .arg(worktree)
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .output()
            .unwrap();
        let common = std::path::PathBuf::from(String::from_utf8(repository.stdout).unwrap().trim());
        let primary = common.parent().unwrap();
        let a = resolve_communication_project(
            worktree.to_str().unwrap(),
            "test-host",
            CommunicationProjectSource::Git,
            None,
        )
        .unwrap();
        let b = resolve_communication_project(
            primary.to_str().unwrap(),
            "test-host",
            CommunicationProjectSource::Git,
            None,
        )
        .unwrap();
        assert_eq!(a, b);
        let link =
            std::env::temp_dir().join(format!("b138-link-{}", uuid::Uuid::new_v4().simple()));
        std::os::unix::fs::symlink(worktree, &link).unwrap();
        let result = resolve_communication_project(
            link.to_str().unwrap(),
            "test-host",
            CommunicationProjectSource::Git,
            None,
        );
        std::fs::remove_file(&link).unwrap();
        assert_eq!(result, Some(a));
        assert!(
            resolve_communication_project(
                primary.to_str().unwrap(),
                "inconnu",
                CommunicationProjectSource::Git,
                None
            )
            .is_none()
        );
    }

    #[test]
    fn spec138_same_basename_repositories_remain_other_projects() {
        use std::os::unix::fs::DirBuilderExt;
        let root =
            std::env::temp_dir().join(format!("b138-homonyms-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let mut projects = Vec::new();
        for parent in ["first", "second"] {
            let repository = root.join(parent).join("same-name");
            std::fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(&repository)
                .unwrap();
            assert!(
                std::process::Command::new("git")
                    .args(["init", "--quiet"])
                    .arg(&repository)
                    .env_remove("GIT_DIR")
                    .env_remove("GIT_WORK_TREE")
                    .env_remove("GIT_COMMON_DIR")
                    .status()
                    .unwrap()
                    .success()
            );
            projects.push(
                resolve_communication_project(
                    repository.to_str().unwrap(),
                    "test-host",
                    CommunicationProjectSource::Git,
                    None,
                )
                .unwrap(),
            );
        }
        assert_ne!(projects[0].root, projects[1].root);
        assert_eq!(
            project_relation(Some(&projects[0]), Some(&projects[1])),
            ProjectRelation::Other
        );
        assert!(
            project_scope(Some(&projects[0]), Some(&projects[1]), "recipient", None)
                .unwrap_err()
                .starts_with("cross_project_reason_required")
        );
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn spec138_reason_is_part_of_canonical_envelope() {
        let base = bridget_core::BridgetMessage::new("a", "b", "exact body");
        let mut wire = serde_json::to_value(&base).unwrap();
        wire["cross_project_reason"] = serde_json::json!("shared review");
        let with_reason: bridget_core::BridgetMessage = serde_json::from_value(wire).unwrap();
        assert_ne!(
            canonical_send("scope", "id", &base, 1),
            canonical_send("scope", "id", &with_reason, 1)
        );
    }

    #[test]
    fn spec138_explicit_null_reason_is_not_legacy_absence() {
        let mut wire =
            serde_json::to_value(bridget_core::BridgetMessage::new("a", "b", "body")).unwrap();
        wire["cross_project_reason"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<bridget_core::BridgetMessage>(wire).is_err());
    }

    #[test]
    fn portee_stable_reste_identique_a_la_reference() {
        assert_eq!(
            issuer_scope("instance-stable-089"),
            "012_scope_37419e46a921ee300d273a45c16ebe1a"
        );
        assert_ne!(
            issuer_scope("instance-stable-089"),
            issuer_scope("instance-other")
        );
    }

    #[test]
    fn canon_historique_garde_ses_octets_et_ses_frontieres() {
        let mut message: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte\n  suite","reply":true,"hops":4,"deadline_at":123060,"in_reply_to":"request-0"}"#
        ).unwrap();
        let bytes = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000);
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "627269646765742f636c69656e742d73656e642f76310000000000000000163031325f73636f70655f61616161616161616161616100000000000000096d6573736167652d3100000000000000076167656e742d62000000000000000d74657874650a2020737569746501000000000000000400000004010000000000000006313233303630010000000000000009726571756573742d300000000000000008000000000001e078"
        );
        message.from_display_name = Some("Nouveau nom".into());
        assert_eq!(
            bytes,
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
        );
        let mut titled = serde_json::to_value(&message).unwrap();
        titled["thread_display_title"] = serde_json::json!("Titre de présentation");
        let titled: bridget_core::BridgetMessage = serde_json::from_value(titled).unwrap();
        assert_eq!(
            bytes,
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &titled, 123000)
        );
        assert_eq!(message.content_key(), titled.content_key());
        message.in_reply_to = Some("request-other".into());
        assert_ne!(
            bytes,
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
        );
    }

    #[test]
    fn spec102_v20_canon_sans_notice_inchange_et_chaque_champ_de_notice_compte() {
        let base: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte","reply":false,"hops":4}"#,
        )
        .unwrap();
        let reference = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &base, 123000);
        let mut same = base.clone();
        same.thread_notice = None;
        assert_eq!(
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &same, 123000),
            reference
        );
        let notice = bridget_core::ThreadNotice {
            version: 1,
            thread_id: "33333333-3333-4333-8333-333333333333".into(),
            through_seq: 20,
            generation: 2,
        };
        let mut with_notice = base.clone();
        with_notice.thread_notice = Some(notice.clone());
        let canon = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &with_notice, 123000);
        assert_ne!(canon, reference);
        assert!(
            canon.starts_with(&reference),
            "le préfixe historique est conservé"
        );
        for variant in [
            bridget_core::ThreadNotice {
                version: 2,
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                thread_id: "44444444-4444-4444-8444-444444444444".into(),
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                through_seq: 21,
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                generation: 3,
                ..notice.clone()
            },
        ] {
            let mut other = base.clone();
            other.thread_notice = Some(variant);
            assert_ne!(
                canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &other, 123000),
                canon
            );
        }
    }

    #[test]
    fn spec133_provenance_ne_change_pas_le_rejeu_parental() {
        let mut message = bridget_core::BridgetMessage::new("parent", "cible", "texte");
        message.id = "message-enfant".into();
        let reference = canonical_send("scope-parent", "message-enfant", &message, 123_000);
        message.delegated_origin = Some(bridget_core::DelegatedOrigin {
            provider: "codex".into(),
            child_ref: "0123456789abcdef".into(),
        });
        assert_eq!(
            canonical_send("scope-parent", "message-enfant", &message, 123_000),
            reference,
            "la provenance visible ne change pas le contrat de rejeu du parent"
        );
    }

    #[test]
    fn les_facades_ne_sont_plus_une_dependance_du_noyau() {
        // Oracle d'architecture : remettre un appel via MCP recrée le cycle.
        for source in [
            include_str!("daemon.rs"),
            include_str!("store.rs"),
            include_str!("store/ledger_requests.rs"),
            include_str!("store/service_events.rs"),
            include_str!("store/project_compat.rs"),
            include_str!("idempotency.rs"),
            include_str!("idempotency/send_delivery.rs"),
            include_str!("idempotency/spawn_commands.rs"),
            include_str!("idempotency/agent_links.rs"),
            include_str!("referent_control.rs"),
            include_str!("cli.rs"),
        ] {
            assert!(!source.contains("crate::mcp::issuer_scope"));
        }
    }
}
