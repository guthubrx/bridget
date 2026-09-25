//! Persistance du `session_id` fournisseur Claude (stream-json).
//!
//! Emplacement : `<session_store_root>/<agent>/claude_provider_session`
//! où `session_store_root` est en production `~/.cache/bridget/sessions`
//! (même racine que le journal d'agent).
//!
//! Pourquoi cet endroit survit aux deux morts mesurées ce soir :
//! - mort du managed-wrapper (et de son enfant) : fichier sur disque, pas
//!   de RAM wrapper ;
//! - redémarrage du daemon : hors mémoire daemon, relu au prochain spawn
//!   géré du même nom d'agent.

use crate::fsutil::{create_private_dir, write_private_file_atomic};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Préfixe imposé pour tout message d'échec de reprise. Un démarrage normal
/// ne l'émet jamais — c'est le distinguo demandé par le collège.
pub const CLAUDE_RESUME_FAILED_PREFIX: &str = "reprise Claude impossible:";

const SESSION_FILE_NAME: &str = "claude_provider_session";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSessionStore {
    root: PathBuf,
    agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeFailureKind {
    /// Les trois cas réels mesurés (UUID fantôme, autre machine, fichier
    /// session fournisseur supprimé) rendent tous
    /// « No conversation found with session ID ».
    Introuvable,
    /// Autre `is_error` pendant une tentative de `--resume`.
    Refusee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeFailure {
    pub kind: ResumeFailureKind,
    pub attempted_id: String,
    pub vendor_message: String,
}

impl ProviderSessionStore {
    pub fn new(root: impl Into<PathBuf>, agent: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            agent: agent.into(),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.root.join(&self.agent).join(SESSION_FILE_NAME)
    }

    pub fn load(&self) -> Option<String> {
        let raw = fs::read_to_string(self.path()).ok()?;
        let id = raw.trim();
        if id.is_empty() {
            None
        } else {
            Some(id.to_string())
        }
    }

    pub fn store(&self, session_id: &str) -> std::io::Result<()> {
        let path = self.path();
        if let Some(parent) = path.parent() {
            create_private_dir(parent)?;
        }
        write_private_file_atomic(&path, session_id.trim().as_bytes())
    }

    pub fn clear(&self) -> std::io::Result<()> {
        let path = self.path();
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

/// Injecte `--resume <id>` en tête des arguments métier (avant les flags
/// stream-json imposés par le pilote). Idempotent si déjà présent.
pub fn inject_resume_arg(args: &mut Vec<String>, session_id: &str) {
    if args.iter().any(|argument| argument == "--resume") {
        return;
    }
    args.insert(0, "--resume".to_string());
    args.insert(1, session_id.to_string());
}

pub fn strip_resume_arg(args: &mut Vec<String>) {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--resume" {
            args.remove(index);
            if index < args.len() {
                args.remove(index);
            }
            continue;
        }
        if let Some(value) = args[index].strip_prefix("--resume=") {
            let _ = value;
            args.remove(index);
            continue;
        }
        index += 1;
    }
}

pub fn session_id_from_system_init(value: &Value) -> Option<String> {
    let is_init = value.get("type").and_then(Value::as_str) == Some("system")
        && value.get("subtype").and_then(Value::as_str) == Some("init");
    if !is_init {
        return None;
    }
    value
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
}

pub fn classify_resume_failure(value: &Value, attempted_id: &str) -> Option<ResumeFailure> {
    if value.get("is_error").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let vendor_message = value
        .get("errors")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|text| !text.trim().is_empty())
        .unwrap_or("échec de reprise sans détail fournisseur")
        .to_string();
    let kind = if vendor_message.contains("No conversation found with session ID") {
        ResumeFailureKind::Introuvable
    } else {
        ResumeFailureKind::Refusee
    };
    Some(ResumeFailure {
        kind,
        attempted_id: attempted_id.to_string(),
        vendor_message,
    })
}

impl ResumeFailure {
    /// Message distinguable d'un démarrage normal — jamais une session neuve
    /// déguisée en continuation.
    pub fn named_message(&self) -> String {
        match self.kind {
            ResumeFailureKind::Introuvable => format!(
                "{CLAUDE_RESUME_FAILED_PREFIX} conversation introuvable pour l'identifiant {} — démarrage d'une session neuve ({})",
                self.attempted_id, self.vendor_message
            ),
            ResumeFailureKind::Refusee => format!(
                "{CLAUDE_RESUME_FAILED_PREFIX} reprise refusée pour l'identifiant {} — démarrage d'une session neuve ({})",
                self.attempted_id, self.vendor_message
            ),
        }
    }
}

/// Prépare les arguments de lancement : charge l'id persisté et injecte
/// `--resume` seulement s'il existe. Absence de fichier = premier lancement.
pub fn prepare_launch_args(
    args: &mut Vec<String>,
    store: Option<&ProviderSessionStore>,
) -> Option<String> {
    let store = store?;
    let session_id = store.load()?;
    inject_resume_arg(args, &session_id);
    Some(session_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-claude-session-{}-{}-{}",
            label,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    /// P1 — un agent relancé retrouve l'identifiant à repasser en `--resume`.
    /// Mutant : ne pas appeler `inject_resume_arg` → cet oracle meurt seul.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_P1_reprise_injecte_resume_quand_session_persistee() {
        let root = temp_root("p1");
        let store = ProviderSessionStore::new(&root, "essai-claude");
        store.store("3c72152f-8a6d-41f8-a00b-e1d46160e4b8").unwrap();
        let mut args = vec!["--model".to_string(), "claude-opus-5".to_string()];
        let loaded = prepare_launch_args(&mut args, Some(&store));
        assert_eq!(
            loaded.as_deref(),
            Some("3c72152f-8a6d-41f8-a00b-e1d46160e4b8")
        );
        assert_eq!(args[0], "--resume");
        assert_eq!(args[1], "3c72152f-8a6d-41f8-a00b-e1d46160e4b8");
        let _ = fs::remove_dir_all(root);
    }

    /// P2 — l'échec de reprise EST NOMMÉ (message présent), pas une simple
    /// session neuve. Mutant : `named_message` vide → oracle meurt seul.
    /// Attentes en dur, jamais un appel au prédicat de production opaque.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_P2_echec_reprise_produit_un_message_nomme() {
        // Les trois cas réels mesurés le 2026-08-26 rendent la même erreur
        // fournisseur ; on fige les trois payloads observés.
        let cases = [
            (
                "00000000-0000-0000-0000-000000000000",
                r#"{"type":"result","subtype":"error_during_execution","is_error":true,"errors":["No conversation found with session ID: 00000000-0000-0000-0000-000000000000"]}"#,
            ),
            (
                "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                r#"{"type":"result","subtype":"error_during_execution","is_error":true,"errors":["No conversation found with session ID: aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"]}"#,
            ),
            (
                "3c72152f-8a6d-41f8-a00b-e1d46160e4b8",
                r#"{"type":"result","subtype":"error_during_execution","is_error":true,"errors":["No conversation found with session ID: 3c72152f-8a6d-41f8-a00b-e1d46160e4b8"]}"#,
            ),
        ];
        for (attempted_id, raw) in cases {
            let value: Value = serde_json::from_str(raw).unwrap();
            let failure = classify_resume_failure(&value, attempted_id)
                .expect("l'échec de reprise doit être classé");
            let notice = failure.named_message();
            assert!(
                notice.starts_with(CLAUDE_RESUME_FAILED_PREFIX),
                "message d'échec absent ou non nommé: {notice}"
            );
            assert!(
                notice.contains("conversation introuvable"),
                "cause absente du message: {notice}"
            );
            assert!(
                notice.contains(attempted_id),
                "identifiant tenté absent du message: {notice}"
            );
            assert!(
                notice.contains("démarrage d'une session neuve"),
                "annonce de démarrage neuf absente: {notice}"
            );
            // Mutant silencieux : une chaîne vide passe un oracle d'absence.
            assert!(
                !notice.trim().is_empty(),
                "oracle d'existence : le message d'échec doit exister"
            );
        }
    }

    /// P3 — premier lancement sans fichier de session : pas de `--resume`,
    /// pas d'échec. Mutant : exiger un id → cet oracle meurt seul.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_P3_premier_lancement_sans_session_ne_tente_pas_resume() {
        let root = temp_root("p3");
        let store = ProviderSessionStore::new(&root, "claude-neuf");
        let mut args = vec!["--model".to_string(), "claude-opus-5".to_string()];
        let loaded = prepare_launch_args(&mut args, Some(&store));
        assert_eq!(loaded, None);
        assert!(!args.iter().any(|argument| argument == "--resume"));
        assert!(!args.iter().any(|argument| argument.contains("resume")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn system_init_extrait_le_session_id() {
        let value = json!({
            "type": "system",
            "subtype": "init",
            "session_id": "3c72152f-8a6d-41f8-a00b-e1d46160e4b8"
        });
        assert_eq!(
            session_id_from_system_init(&value).as_deref(),
            Some("3c72152f-8a6d-41f8-a00b-e1d46160e4b8")
        );
        assert_eq!(
            session_id_from_system_init(&json!({"type":"result","session_id":"x"})),
            None
        );
    }

    #[test]
    fn store_clear_et_relecture() {
        let root = temp_root("roundtrip");
        let store = ProviderSessionStore::new(&root, "agent-a");
        store.store("abc-123").unwrap();
        assert_eq!(store.load().as_deref(), Some("abc-123"));
        store.clear().unwrap();
        assert_eq!(store.load(), None);
        let _ = fs::remove_dir_all(root);
    }
}
