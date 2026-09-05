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

#[cfg(test)]
use std::sync::Mutex;

/// Forçage test-only du `payload.kind` émis par un pilote.
///
/// Mutex (pas thread-local) : Claude/Codex journalisent depuis des threads
/// worker — un TLS serait invisible sur le chemin réel bout-en-bout.
#[cfg(test)]
static FORCED_PILOT_UPDATE_KIND: Mutex<Option<&'static str>> = Mutex::new(None);

/// Sérialise les fixtures qui empruntent `pilot_kind_str` avec le forçage
/// bout-en-bout — sinon un TEMOIN_TOOL parallèle verrait `intent`.
#[cfg(test)]
static PILOT_KIND_SUITE_LOCK: Mutex<()> = Mutex::new(());

/// Verrou partagé : tout tour qui journalise un acte via `pilot_kind_str`.
#[cfg(test)]
pub fn pilot_kind_suite_lock() -> std::sync::MutexGuard<'static, ()> {
    PILOT_KIND_SUITE_LOCK
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

/// Exécute `f` en forçant le kind que les pilotes écrivent via
/// [`pilot_kind_str`]. Sert aux oracles bout-en-bout : le pilote emprunte
/// son vrai chemin (tool_use / commandExecution) mais pose un kind interdit.
#[cfg(test)]
pub fn with_forced_pilot_update_kind<R>(kind: &'static str, f: impl FnOnce() -> R) -> R {
    let _suite = pilot_kind_suite_lock();
    {
        let mut slot = FORCED_PILOT_UPDATE_KIND
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        *slot = Some(kind);
    }
    let result = f();
    {
        let mut slot = FORCED_PILOT_UPDATE_KIND
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        *slot = None;
    }
    result
}

/// Kind effectif posé par un pilote pour un update.
///
/// En production : toujours le canonique. En test : peut être forcé pour
/// prouver qu'un kind refusé devient un événement terminal visible.
pub fn pilot_kind_str(canonical: JournalUpdateKind) -> &'static str {
    #[cfg(test)]
    {
        if let Some(forced) = FORCED_PILOT_UPDATE_KIND
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_ref()
            .copied()
        {
            return forced;
        }
    }
    canonical.as_str()
}

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
    /// Publication attestée d'un artefact Bridget.
    ///
    /// Les pilotes qui ne savent journaliser qu'un appel d'outil continuent
    /// d'émettre `tool`. La projection UI reconnaît également ce chemin et
    /// l'affiche comme une publication, sans inventer une seconde timeline.
    Artifact,
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
    /// SPEC-088 : refus reconnu sur la sortie brute d'une commande (sandbox du
    /// fournisseur). Attribué à Bridget par la vue, jamais à l'agent.
    Refusal,
}

impl JournalUpdateKind {
    /// Ensemble fermé écriture : ordre stable, source de vérité unique.
    pub const ALL: &'static [Self] = &[
        Self::Text,
        Self::Command,
        Self::File,
        Self::Tool,
        Self::Artifact,
        Self::ToolCallLegacy,
        Self::Plan,
        Self::Approval,
        Self::Refusal,
    ];

    /// Sous-ensemble fermé des actes du journal (hors contenu `text`).
    pub const ACTS: &'static [Self] = &[
        Self::Command,
        Self::File,
        Self::Tool,
        Self::Artifact,
        Self::ToolCallLegacy,
        Self::Plan,
        Self::Approval,
        Self::Refusal,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Command => "command",
            Self::File => "file",
            Self::Tool => "tool",
            Self::Artifact => "artifact",
            Self::ToolCallLegacy => "tool_call",
            Self::Plan => "plan",
            Self::Approval => "approval",
            Self::Refusal => "refusal",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|kind| kind.as_str() == raw)
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
    let parsed = parse_update_kind(kind)?;
    if parsed == JournalUpdateKind::Refusal {
        validate_refusal_payload(payload)?;
    }
    Ok(())
}

/// Un acte `refusal` porte une couche fermée et une ligne brute bornée :
/// sans elles, la vue ne pourrait ni attribuer ni permettre de contredire.
fn validate_refusal_payload(payload: &Value) -> Result<(), String> {
    let layer = payload
        .get("layer")
        .and_then(Value::as_str)
        .ok_or_else(|| "update refusal sans payload.layer — écriture refusée".to_string())?;
    if crate::refusals::RefusalLayer::parse(layer).is_none() {
        return Err(format!(
            "update refusal : couche hors vocabulaire {layer:?}"
        ));
    }
    let raw = payload
        .get("raw")
        .and_then(Value::as_str)
        .ok_or_else(|| "update refusal sans payload.raw — écriture refusée".to_string())?;
    if raw.chars().count() > crate::refusals::MAX_RAW_CHARS {
        return Err("update refusal : payload.raw dépasse la borne".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
            validate_journal_write("update", &json!({"kind": "tool_call", "title": "legacy"}))
                .is_ok(),
            "tool_call legacy encore accepté jusqu'à sa disparition documentée"
        );
        assert!(
            validate_journal_write("error", &json!({"reason": "x"})).is_ok(),
            "les événements non-update ne sont pas soumis au vocabulaire d'actes"
        );

        let rejected =
            validate_journal_write("update", &json!({"kind": "intent", "text": "fantôme"}));
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
            healthy
                .as_ref()
                .is_err_and(|detail| detail.contains("hors vocabulaire")),
            "TEMOIN_ecriture_hors_vocabulaire: {healthy:?}"
        );
    }
}
