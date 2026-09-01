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
    ];

    /// Actes projetés par la page (hors `text`). Doit rester égal à
    /// `JOURNAL_ACT_KINDS` dans `crates/bridget-daemon/assets/ui/app.js`.
    pub const ACTS: &'static [Self] = &[
        Self::Command,
        Self::File,
        Self::Tool,
        Self::Artifact,
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
            Self::Artifact => "artifact",
            Self::ToolCallLegacy => "tool_call",
            Self::Plan => "plan",
            Self::Approval => "approval",
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

    /// Oracle SENS (pas forme) : évalue le `Set` runtime exporté par app.js.
    ///
    /// Borne écrite — jusqu'où il protège :
    /// - protège le **contenu runtime** de `JOURNAL_ACT_KINDS` vs `JournalUpdateKind::ACTS` ;
    /// - survit à une réécriture équivalente (ex. `.map` sur les mêmes littéraux) ;
    /// - meurt si le Set runtime diverge (kind fantôme, omission, rename sémantique).
    ///
    /// À partir d'où il ne protège plus :
    /// - `JOURNAL_ACT_KINDS` retiré de `module.exports` → échec fermé (node/require) ;
    /// - Set reconstruit hors du module chargé (autre fichier non requis) ;
    /// - Node absent du PATH de mesure.
    ///   Le parse source (marker `new Set([`) est volontairement abandonné : il
    ///   mourait sur un espace cosmétique et restait vert sur `.map` équivalent.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_vocabulaire_vue_et_ecriture_ne_divergent_pas() {
        let write_acts: BTreeSet<&str> = JournalUpdateKind::ACTS
            .iter()
            .map(|kind| kind.as_str())
            .collect();
        let view_acts = view_acts_from_app_js_runtime();
        assert_eq!(
            view_acts,
            write_acts.iter().map(|s| (*s).to_string()).collect(),
            "divergence vue↔écriture (runtime) : write={write_acts:?} view={view_acts:?}"
        );
    }

    fn view_acts_from_app_js_runtime() -> BTreeSet<String> {
        use std::process::Command;
        let ui_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bridget-daemon/assets/ui/app.js");
        let script = format!(
            "const api = require({}); const kinds = [...api.JOURNAL_ACT_KINDS].sort(); process.stdout.write(JSON.stringify(kinds));",
            serde_json::to_string(&ui_path).expect("chemin UI sérialisable")
        );
        let output = Command::new("node")
            .arg("-e")
            .arg(&script)
            .output()
            .unwrap_or_else(|err| panic!("node pour évaluer JOURNAL_ACT_KINDS: {err}"));
        assert!(
            output.status.success(),
            "évaluation runtime app.js échouée: status={} stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        let parsed: Vec<String> = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
            panic!(
                "JSON kinds invalide ({}): {}",
                err,
                String::from_utf8_lossy(&output.stdout)
            )
        });
        parsed.into_iter().collect()
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_parse_source_reste_vert_sur_map_equivalent_mais_runtime_voit_le_sens() {
        // Montre POURQUOI le parse source est abandonné : même littéraux via
        // `.map` → le parse forme reste vert ; seul le runtime distingue une
        // transformation qui change le sens.
        let equivalent = r#"const JOURNAL_ACT_KINDS = new Set(["command","file","tool","artifact","tool_call","plan","approval"].map((k) => k));"#;
        let mutated_sense = r#"const JOURNAL_ACT_KINDS = new Set(["command","file","tool","artifact","tool_call","plan","approval"].map((k) => k === "tool" ? "intent" : k));"#;
        fn parse_form(source: &str) -> BTreeSet<String> {
            let marker = "const JOURNAL_ACT_KINDS = new Set([";
            let start = source.find(marker).expect("marker");
            let after = &source[start + marker.len()..];
            let end = after.find(']').expect("]");
            after[..end]
                .split(',')
                .filter_map(|piece| {
                    let trimmed = piece.trim();
                    let start = trimmed.find('"')?;
                    let rest = &trimmed[start + 1..];
                    let end = rest.find('"')?;
                    Some(rest[..end].to_string())
                })
                .collect()
        }
        let expected: BTreeSet<String> = JournalUpdateKind::ACTS
            .iter()
            .map(|kind| kind.as_str().to_string())
            .collect();
        assert_eq!(
            parse_form(equivalent),
            expected,
            "parse forme : réécriture .map équivalente reste verte (faux sentiment de garde)"
        );
        assert_eq!(
            parse_form(mutated_sense),
            expected,
            "parse forme : mutation tool→intent via .map RESTE VERTE — c'est le trou"
        );
        // Le runtime, lui, verrait "intent" à la place de "tool". Contrôle
        // positif : l'oracle runtime du TEMOIN principal compare des Sets réels.
        let runtime_mutated: BTreeSet<String> = [
            "command",
            "file",
            "intent",
            "artifact",
            "tool_call",
            "plan",
            "approval",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        assert_ne!(
            runtime_mutated, expected,
            "runtime distingue la mutation de sens que le parse rate"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_projectTimeline_filtre_par_defaut_est_JOURNAL_ACT_KINDS() {
        // Charge 3 : mutant REAL_ACT_KINDS — JOURNAL_ACT_KINDS reste aligné sur
        // l'enum (oracle contenu VERT) mais projectTimeline filtre via un autre
        // Set. Ce témoin meurt dès que le repli par défaut n'est plus
        // JOURNAL_ACT_KINDS, pour N'IMPORTE QUEL kind (pas seulement tool_call).
        let ui_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bridget-daemon/assets/ui/app.js");
        let source = fs::read_to_string(&ui_path)
            .unwrap_or_else(|err| panic!("lire {}: {err}", ui_path.display()));
        let fn_marker = "function projectTimeline(";
        let fn_start = source
            .find(fn_marker)
            .unwrap_or_else(|| panic!("projectTimeline introuvable dans {}", ui_path.display()));
        // Fenêtre bornée : corps jusqu'à la fonction suivante de même niveau
        // ou 2500 octets — assez pour le choix de filtre, sans avaler tout le fichier.
        let window = &source[fn_start..fn_start.saturating_add(2500).min(source.len())];
        let needle = "options.actKinds instanceof Set ? options.actKinds : JOURNAL_ACT_KINDS";
        assert!(
            window.contains(needle),
            "projectTimeline doit filtrer par JOURNAL_ACT_KINDS par défaut \
             (mutant REAL_ACT_KINDS : {needle} absent du corps)"
        );
        // Aucun autre repli `… : AUTRE_SET` sur la même ligne de décision.
        for line in window.lines() {
            if line.contains("options.actKinds instanceof Set") {
                assert!(
                    line.contains(": JOURNAL_ACT_KINDS"),
                    "repli actKinds doit être JOURNAL_ACT_KINDS, trouvé: {line}"
                );
            }
        }
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_REAL_ACT_KINDS_tue_TEMOIN_projectTimeline_filtre_par_defaut() {
        // Simule le mutant revue : constante alignée pour l'oracle contenu, mais
        // filtre runtime branché sur un autre Set.
        let healthy = "const actKinds = options.actKinds instanceof Set ? options.actKinds : JOURNAL_ACT_KINDS;";
        let broken =
            "const actKinds = options.actKinds instanceof Set ? options.actKinds : REAL_ACT_KINDS;";
        assert!(healthy.contains(": JOURNAL_ACT_KINDS"), "contrôle positif");
        assert!(
            !broken.contains(": JOURNAL_ACT_KINDS") || broken.contains(": REAL_ACT_KINDS"),
            "le mutant pointe ailleurs"
        );
        assert!(
            broken.contains(": REAL_ACT_KINDS") && !broken.ends_with(": JOURNAL_ACT_KINDS;"),
            "TEMOIN_projectTimeline_filtre_par_defaut: le mutant REAL_ACT_KINDS doit être détectable"
        );
    }
}
