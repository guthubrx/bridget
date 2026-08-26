//! Contrat du vocabulaire des `payload.kind` journalisés.
//!
//! Propriété : un kind écrit dans le journal appartient au vocabulaire commun ;
//! une écriture hors vocabulaire est **refusée** — jamais acceptée en silence
//! puis invisible à la lecture (le défaut qui a masqué page, Claude et Codex).
//!
//! Idée reprise de T3 Code (`providerRuntime` + journaliseur partagé) sans
//! recopier Effect Schema : en Rust, un enum fermé + une garde à l'écriture
//! partagée par tous les pilotes via [`crate::journal::JournalWriter`].

use serde_json::Value;

/// Kinds autorisés sur un événement journal `event == "update"`.
///
/// `text` n'est pas un « acte » UI (bulle de réponse) ; les autres sont des
/// actes projetés. `tool_call` est un héritage borné — voir
/// [`JournalUpdateKind::ToolCallLegacy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JournalUpdateKind {
    Text,
    Command,
    File,
    Tool,
    /// Héritage Cursor / ACP d'avant le kind canonique `tool` (78d57dc).
    ///
    /// **Accepté en écriture** tant que des journaux ou fixtures legacy
    /// l'émettent encore. La vue projette `tool_call` → `tool`.
    ///
    /// **Disparition** : retirer ce variant (et l'entrée vue/témoins) quand
    /// les trois conditions sont vraies ensemble —
    /// (1) tous les daemons déployés écrivent `tool` (post-78d57dc),
    /// (2) attach + fixtures n'émettent plus `tool_call`,
    /// (3) un constat greffe mesure 0 nouvelle écriture `tool_call` sur N jours.
    ToolCallLegacy,
    Plan,
    Approval,
}

impl JournalUpdateKind {
    /// Ensemble fermé écriture : ordre stable, source de vérité unique.
    pub const ALL: &'static [Self] = &[
        Self::Text,
        Self::Command,
        Self::File,
        Self::Tool,
        Self::ToolCallLegacy,
        Self::Plan,
        Self::Approval,
    ];

    /// Actes projetés par la page (hors `text`). Doit rester égal à
    /// `JOURNAL_ACT_KINDS` dans `crates/bridget-daemon/assets/ui/app.js`.
    pub const ACTS: &'static [Self] = &[
        Self::Command,
        Self::File,
        Self::Tool,
        Self::ToolCallLegacy,
        Self::Plan,
        Self::Approval,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Command => "command",
            Self::File => "file",
            Self::Tool => "tool",
            Self::ToolCallLegacy => "tool_call",
            Self::Plan => "plan",
            Self::Approval => "approval",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == raw)
    }

    pub fn is_act(self) -> bool {
        !matches!(self, Self::Text)
    }

    /// Noms d'actes dans l'ordre canonique (pour oracles anti-divergence).
    pub fn act_names() -> Vec<&'static str> {
        Self::ACTS.iter().map(|kind| kind.as_str()).collect()
    }

    /// Tous les noms d'update autorisés à l'écriture.
    pub fn all_names() -> Vec<&'static str> {
        Self::ALL.iter().map(|kind| kind.as_str()).collect()
    }
}

/// Refuse un `payload.kind` d'`update` hors vocabulaire.
pub fn parse_update_kind(raw: &str) -> Result<JournalUpdateKind, String> {
    JournalUpdateKind::parse(raw).ok_or_else(|| {
        format!(
            "kind journal hors vocabulaire: {raw:?} (autorise: {})",
            JournalUpdateKind::all_names().join(", ")
        )
    })
}

/// Garde d'écriture partagée. Ne s'applique qu'aux événements `update` :
/// les autres (`turn_start`, `error`, `permission`, …) ont d'autres formes.
pub fn validate_journal_write(event: &str, payload: &Value) -> Result<(), String> {
    if event != "update" {
        return Ok(());
    }
    let Some(kind_value) = payload.get("kind") else {
        return Err("update journal sans payload.kind — écriture refusée".to_string());
    };
    let Some(kind) = kind_value.as_str() else {
        return Err("update journal : payload.kind doit être une chaîne".to_string());
    };
    parse_update_kind(kind).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_ecriture_valide_passe_puis_hors_vocabulaire_est_refusee() {
        // Preuve d'abord qu'une écriture valide passe — sinon un oracle
        // d'absence passerait sur une projection vide et ne garderait rien.
        assert!(
            validate_journal_write("update", &json!({"kind": "tool", "text": "Read"})).is_ok(),
            "une écriture tool valide doit passer"
        );
        assert!(
            validate_journal_write("update", &json!({"kind": "text", "content": "ok"})).is_ok()
        );
        assert!(
            validate_journal_write(
                "update",
                &json!({"kind": "tool_call", "title": "legacy"})
            )
            .is_ok(),
            "tool_call legacy encore accepté jusqu'à sa disparition documentée"
        );
        assert!(
            validate_journal_write("error", &json!({"reason": "x"})).is_ok(),
            "les événements non-update ne sont pas soumis au vocabulaire d'actes"
        );

        let rejected = validate_journal_write(
            "update",
            &json!({"kind": "intent", "text": "fantôme"}),
        );
        assert!(
            rejected.is_err(),
            "intent hors vocabulaire doit être refusé"
        );
        let detail = rejected.unwrap_err();
        assert!(
            detail.contains("hors vocabulaire"),
            "le refus doit être signalé explicitement, pas silencieux: {detail}"
        );
        assert!(
            validate_journal_write("update", &json!({"kind": "quantum_wrench", "text": "x"}))
                .is_err(),
            "un kind inventé par un nouveau pilote doit échouer"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_accepte_kind_inconnu_tue_TEMOIN_ecriture_hors_vocabulaire() {
        // Simule le défaut : accepter n'importe quel kind (comme avant le contrat).
        fn broken_validate(event: &str, payload: &Value) -> Result<(), String> {
            if event != "update" {
                return Ok(());
            }
            let _ = payload.get("kind");
            Ok(())
        }
        assert!(
            broken_validate("update", &json!({"kind": "intent"})).is_ok(),
            "le mutant laisse passer"
        );
        let healthy = validate_journal_write("update", &json!({"kind": "intent"}));
        assert!(
            healthy.as_ref().is_err_and(|detail| detail.contains("hors vocabulaire")),
            "TEMOIN_ecriture_hors_vocabulaire: {healthy:?}"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_vocabulaire_vue_et_ecriture_ne_divergent_pas() {
        let write_acts: BTreeSet<&str> = JournalUpdateKind::ACTS
            .iter()
            .map(|kind| kind.as_str())
            .collect();

        let ui_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../bridget-daemon/assets/ui/app.js");
        let source = fs::read_to_string(&ui_path)
            .unwrap_or_else(|err| panic!("lire {}: {err}", ui_path.display()));
        let marker = "const JOURNAL_ACT_KINDS = new Set([";
        let start = source
            .find(marker)
            .unwrap_or_else(|| panic!("JOURNAL_ACT_KINDS introuvable dans {}", ui_path.display()));
        let after = &source[start + marker.len()..];
        let end = after
            .find(']')
            .unwrap_or_else(|| panic!("JOURNAL_ACT_KINDS non fermé"));
        let block = &after[..end];
        let view_acts: BTreeSet<String> = block
            .split(',')
            .filter_map(|piece| {
                let trimmed = piece.trim();
                let start = trimmed.find('"')?;
                let rest = &trimmed[start + 1..];
                let end = rest.find('"')?;
                Some(rest[..end].to_string())
            })
            .collect();

        assert_eq!(
            view_acts,
            write_acts.iter().map(|s| (*s).to_string()).collect(),
            "divergence vue↔écriture : write={write_acts:?} view={view_acts:?}"
        );
    }
}
