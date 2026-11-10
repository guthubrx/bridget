//! Session 103 — dossier de passation : validation stricte et rendu v1.
//!
//! Un dossier est une valeur JSON rédigée par l'agent, transportée dans le
//! corps d'un envoi idempotent existant. Ce module protège la structure, les
//! bornes, le déterminisme du rendu et l'inertie du contenu ; il n'ouvre ni
//! fichier, ni socket, ni URL, et ne certifie rien de ce que l'auteur déclare.
//!
//! Complexité O(B), B ≤ 16 Kio après rendu ; mémoire O(B) ; aucune E/S.

use serde::Serialize;
use serde_json::{Map, Value};

pub(crate) const HANDOFF_VERSION: u16 = 1;
pub(crate) const MARKER: &str = "[Bridget handoff v1]\n";
pub(crate) const MAX_BODY_BYTES: usize = 16_384;
pub(crate) const MAX_OBJECTIVE_BYTES: usize = 1_024;
pub(crate) const MAX_SUMMARY_BYTES: usize = 8_192;
pub(crate) const MAX_LIST_ITEMS: usize = 12;
pub(crate) const MAX_REFERENCES: usize = 16;
pub(crate) const MAX_ITEM_BYTES: usize = 2_048;
pub(crate) const MAX_LABEL_BYTES: usize = 256;
pub(crate) const MAX_REFERENCE_BYTES: usize = 2_048;

/// Avertissements constants, joints à chaque aperçu et à chaque envoi.
pub(crate) const WARNINGS: [&str; 3] = [
    "sources_not_verified",
    "retention_follows_ledger",
    "ledger_visibility_not_recipient_private",
];

pub(crate) const WARNING_DETAILS: [&str; 3] = [
    "Les références et résultats sont des déclarations de l'auteur : Bridget n'a lu aucune source et ne certifie rien.",
    "Le dossier suit la conservation du journal (sept jours par défaut) ; aucune archive permanente n'est promise.",
    "Le journal général est lisible plus largement que le seul destinataire : aucun secret dans un dossier.",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HandoffError {
    pub field: String,
    pub reason: String,
}

impl std::fmt::Display for HandoffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.field, self.reason)
    }
}

fn err(field: impl Into<String>, reason: impl Into<String>) -> HandoffError {
    HandoffError {
        field: field.into(),
        reason: reason.into(),
    }
}

/// Dossier rendu : corps exact et taille en octets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rendered {
    pub body: String,
    pub bytes: usize,
}

#[derive(Debug, Serialize)]
struct HandoffV1 {
    version: u16,
    objective: String,
    summary: String,
    results: Vec<ResultV1>,
    decisions: Vec<String>,
    questions: Vec<String>,
    next_step: Option<String>,
    references: Vec<ReferenceV1>,
    limitations: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResultV1 {
    text: String,
    evidence: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ReferenceV1 {
    File {
        label: String,
        host: String,
        path: String,
    },
    Url {
        label: String,
        url: String,
    },
    Message {
        label: String,
        id: String,
        target: String,
        source_label: String,
    },
    Journal {
        label: String,
        agent: String,
        from_seq: u64,
        to_seq: u64,
        source_label: String,
    },
    Thread {
        label: String,
        thread_id: String,
        from_seq: u64,
        to_seq: u64,
        source_label: String,
    },
    Artifact {
        label: String,
        artifact_id: String,
        version_id: String,
        source_label: String,
    },
}

fn object<'a>(value: &'a Value, field: &str) -> Result<&'a Map<String, Value>, HandoffError> {
    value
        .as_object()
        .ok_or_else(|| err(field, "objet JSON attendu"))
}

fn reject_unknown(
    map: &Map<String, Value>,
    field: &str,
    allowed: &[&str],
) -> Result<(), HandoffError> {
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(err(format!("{field}.{key}"), "champ inconnu"));
        }
    }
    Ok(())
}

/// Chaîne obligatoire : présente, non nulle, non blanche, bornée en octets UTF-8.
fn required_text(
    map: &Map<String, Value>,
    field: &str,
    max: usize,
) -> Result<String, HandoffError> {
    match map.get(field) {
        None => Err(err(field, "champ obligatoire absent")),
        Some(value) => text(value, field, max),
    }
}

fn text(value: &Value, field: &str, max: usize) -> Result<String, HandoffError> {
    let Some(text) = value.as_str() else {
        return Err(err(field, "chaîne attendue (null refusé)"));
    };
    if text.chars().all(char::is_whitespace) {
        return Err(err(
            field,
            "chaîne vide ou blanche refusée ; omettre le champ s'il est facultatif",
        ));
    }
    if text.len() > max {
        return Err(err(
            field,
            format!(
                "{} octets UTF-8, maximum {max} ; aucune troncature",
                text.len()
            ),
        ));
    }
    Ok(text.to_string())
}

fn optional_text(
    map: &Map<String, Value>,
    field: &str,
    max: usize,
) -> Result<Option<String>, HandoffError> {
    map.get(field)
        .map(|value| text(value, field, max))
        .transpose()
}

fn string_list(map: &Map<String, Value>, field: &str) -> Result<Vec<String>, HandoffError> {
    let Some(value) = map.get(field) else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(err(field, "tableau attendu (null refusé)"));
    };
    if items.len() > MAX_LIST_ITEMS {
        return Err(err(
            field,
            format!("{} éléments, maximum {MAX_LIST_ITEMS}", items.len()),
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| text(item, &format!("{field}[{index}]"), MAX_ITEM_BYTES))
        .collect()
}

fn results(map: &Map<String, Value>) -> Result<Vec<ResultV1>, HandoffError> {
    let Some(value) = map.get("results") else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(err("results", "tableau attendu (null refusé)"));
    };
    if items.len() > MAX_LIST_ITEMS {
        return Err(err(
            "results",
            format!("{} éléments, maximum {MAX_LIST_ITEMS}", items.len()),
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let field = format!("results[{index}]");
            let item = object(item, &field)?;
            reject_unknown(item, &field, &["text", "evidence"])?;
            Ok(ResultV1 {
                text: required_text(item, "text", MAX_ITEM_BYTES)
                    .map_err(|e| err(format!("{field}.{}", e.field), e.reason))?,
                evidence: optional_text(item, "evidence", MAX_ITEM_BYTES)
                    .map_err(|e| err(format!("{field}.{}", e.field), e.reason))?,
            })
        })
        .collect()
}

fn integer(map: &Map<String, Value>, field: &str) -> Result<u64, HandoffError> {
    match map.get(field) {
        None => Err(err(field, "entier obligatoire absent")),
        Some(Value::Number(number)) if number.is_u64() => Ok(number.as_u64().unwrap_or_default()),
        Some(_) => Err(err(
            field,
            "entier positif attendu (pas de virgule, pas de null)",
        )),
    }
}

fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value)
        .map(|parsed| parsed.hyphenated().to_string() == value)
        .unwrap_or(false)
}

/// Forme lexicale d'un chemin absolu Unix ou Windows ; aucun accès disque.
fn is_absolute_path(path: &str) -> bool {
    if path.starts_with('/') || path.starts_with("\\\\") {
        return true;
    }
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'/' | b'\\')
}

/// URL HTTP(S) sans identifiants ; aucune récupération réseau.
fn is_plain_http_url(url: &str) -> bool {
    // Le schéma est insensible à la casse (RFC 3986) ; l'URL est conservée telle
    // quelle dans le dossier, seule la vérification est normalisée.
    let scheme_end = url.find("://").unwrap_or(0);
    let scheme = url[..scheme_end].to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https") {
        return false;
    }
    let rest = &url[scheme_end + 3..];
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    !authority.is_empty() && !authority.contains('@') && !authority.chars().any(char::is_whitespace)
}

fn sequences(
    map: &Map<String, Value>,
    field: &str,
    min_from: u64,
) -> Result<(u64, u64), HandoffError> {
    let from_seq =
        integer(map, "from_seq").map_err(|e| err(format!("{field}.{}", e.field), e.reason))?;
    let to_seq =
        integer(map, "to_seq").map_err(|e| err(format!("{field}.{}", e.field), e.reason))?;
    if from_seq < min_from {
        return Err(err(
            format!("{field}.from_seq"),
            format!("minimum {min_from}"),
        ));
    }
    if to_seq < from_seq {
        return Err(err(
            format!("{field}.to_seq"),
            "doit être supérieur ou égal à from_seq",
        ));
    }
    Ok((from_seq, to_seq))
}

fn reference(value: &Value, field: &str) -> Result<ReferenceV1, HandoffError> {
    let map = object(value, field)?;
    let scoped = |e: HandoffError| err(format!("{field}.{}", e.field), e.reason);
    let kind = required_text(map, "kind", 32).map_err(scoped)?;
    let label = required_text(map, "label", MAX_LABEL_BYTES).map_err(scoped)?;
    let source_label = |map: &Map<String, Value>| {
        required_text(map, "source_label", MAX_LABEL_BYTES).map_err(scoped)
    };
    let rendered = match kind.as_str() {
        "file" => {
            reject_unknown(map, field, &["kind", "label", "host", "path"])?;
            let path = required_text(map, "path", MAX_ITEM_BYTES).map_err(scoped)?;
            if !is_absolute_path(&path) {
                return Err(err(
                    format!("{field}.path"),
                    "chemin absolu requis (forme lexicale, aucun accès disque)",
                ));
            }
            ReferenceV1::File {
                label,
                host: required_text(map, "host", MAX_LABEL_BYTES).map_err(scoped)?,
                path,
            }
        }
        "url" => {
            reject_unknown(map, field, &["kind", "label", "url"])?;
            let url = required_text(map, "url", MAX_ITEM_BYTES).map_err(scoped)?;
            if !is_plain_http_url(&url) {
                return Err(err(
                    format!("{field}.url"),
                    "URL http(s) sans identifiants requise ; aucune récupération",
                ));
            }
            ReferenceV1::Url { label, url }
        }
        "message" => {
            reject_unknown(
                map,
                field,
                &["kind", "label", "id", "target", "source_label"],
            )?;
            ReferenceV1::Message {
                label,
                id: required_text(map, "id", MAX_ITEM_BYTES).map_err(scoped)?,
                target: required_text(map, "target", MAX_ITEM_BYTES).map_err(scoped)?,
                source_label: source_label(map)?,
            }
        }
        "journal" => {
            reject_unknown(
                map,
                field,
                &[
                    "kind",
                    "label",
                    "agent",
                    "from_seq",
                    "to_seq",
                    "source_label",
                ],
            )?;
            let agent = required_text(map, "agent", MAX_LABEL_BYTES).map_err(scoped)?;
            if !canonical_uuid(&agent) {
                return Err(err(format!("{field}.agent"), "UUID canonique requis"));
            }
            let (from_seq, to_seq) = sequences(map, field, 0)?;
            ReferenceV1::Journal {
                label,
                agent,
                from_seq,
                to_seq,
                source_label: source_label(map)?,
            }
        }
        "thread" => {
            reject_unknown(
                map,
                field,
                &[
                    "kind",
                    "label",
                    "thread_id",
                    "from_seq",
                    "to_seq",
                    "source_label",
                ],
            )?;
            let thread_id = required_text(map, "thread_id", MAX_LABEL_BYTES).map_err(scoped)?;
            if !canonical_uuid(&thread_id) {
                return Err(err(format!("{field}.thread_id"), "UUID canonique requis"));
            }
            let (from_seq, to_seq) = sequences(map, field, 1)?;
            ReferenceV1::Thread {
                label,
                thread_id,
                from_seq,
                to_seq,
                source_label: source_label(map)?,
            }
        }
        "artifact" => {
            reject_unknown(
                map,
                field,
                &["kind", "label", "artifact_id", "version_id", "source_label"],
            )?;
            ReferenceV1::Artifact {
                label,
                artifact_id: required_text(map, "artifact_id", MAX_ITEM_BYTES).map_err(scoped)?,
                version_id: required_text(map, "version_id", MAX_ITEM_BYTES).map_err(scoped)?,
                source_label: source_label(map)?,
            }
        }
        other => {
            return Err(err(
                format!("{field}.kind"),
                format!(
                    "genre inconnu « {other} » (file, url, message, journal, thread, artifact)"
                ),
            ));
        }
    };
    let serialized = serde_json::to_string(&rendered).map_err(|e| err(field, e.to_string()))?;
    if serialized.len() > MAX_REFERENCE_BYTES {
        return Err(err(
            field,
            format!(
                "référence de {} octets sérialisés, maximum {MAX_REFERENCE_BYTES}",
                serialized.len()
            ),
        ));
    }
    Ok(rendered)
}

fn references(map: &Map<String, Value>) -> Result<Vec<ReferenceV1>, HandoffError> {
    let Some(value) = map.get("references") else {
        return Ok(Vec::new());
    };
    let Some(items) = value.as_array() else {
        return Err(err("references", "tableau attendu (null refusé)"));
    };
    if items.len() > MAX_REFERENCES {
        return Err(err(
            "references",
            format!("{} références, maximum {MAX_REFERENCES}", items.len()),
        ));
    }
    items
        .iter()
        .enumerate()
        .map(|(index, item)| reference(item, &format!("references[{index}]")))
        .collect()
}

/// Valide le brouillon (objet `draft`) puis rend le corps v1 exact :
/// marqueur, JSON indenté à deux espaces, champs dans l'ordre canonique, sans
/// saut de ligne final. Le contenu est conservé octet pour octet.
pub(crate) fn validate_and_render(draft: &Value) -> Result<Rendered, HandoffError> {
    let map = object(draft, "draft")?;
    reject_unknown(
        map,
        "draft",
        &[
            "objective",
            "summary",
            "results",
            "decisions",
            "questions",
            "next_step",
            "references",
            "limitations",
        ],
    )?;
    let handoff = HandoffV1 {
        version: HANDOFF_VERSION,
        objective: required_text(map, "objective", MAX_OBJECTIVE_BYTES)?,
        summary: required_text(map, "summary", MAX_SUMMARY_BYTES)?,
        results: results(map)?,
        decisions: string_list(map, "decisions")?,
        questions: string_list(map, "questions")?,
        next_step: optional_text(map, "next_step", MAX_ITEM_BYTES)?,
        references: references(map)?,
        limitations: string_list(map, "limitations")?,
    };
    let json = serde_json::to_string_pretty(&handoff).map_err(|e| err("draft", e.to_string()))?;
    let body = format!("{MARKER}{json}");
    let bytes = body.len();
    if bytes > MAX_BODY_BYTES {
        return Err(err(
            "draft",
            format!(
                "dossier rendu de {bytes} octets, maximum {MAX_BODY_BYTES} ; sélectionner le contenu plutôt que découper"
            ),
        ));
    }
    Ok(Rendered { body, bytes })
}

/// Action demandée : l'aperçu refuse tout paramètre de transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HandoffAction {
    Preview,
    Send,
}

impl HandoffAction {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Preview => "preview",
            Self::Send => "send",
        }
    }
}

/// Paramètres de transport d'un envoi, validés avant toute connexion.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct HandoffTransport {
    pub to: String,
    pub reply: bool,
    pub reply_timeout: Option<u64>,
    pub in_reply_to: Option<String>,
    pub id: Option<String>,
    pub issued_at: Option<i64>,
    pub cross_project_reason: Option<String>,
}

/// Requête commune aux façades MCP et CLI : mêmes règles, mêmes refus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HandoffRequest {
    pub action: HandoffAction,
    pub rendered: Rendered,
    pub transport: Option<HandoffTransport>,
}

const TRANSPORT_FIELDS: [&str; 7] = [
    "to",
    "reply",
    "reply_timeout",
    "in_reply_to",
    "id",
    "issued_at",
    "cross_project_reason",
];

/// Analyse stricte de l'objet complet (`action`, `draft`, transport pour `send`).
pub(crate) fn parse_request(value: &Value) -> Result<HandoffRequest, HandoffError> {
    let map = object(value, "request")?;
    reject_unknown(
        map,
        "request",
        &[
            "action",
            "draft",
            "to",
            "reply",
            "reply_timeout",
            "in_reply_to",
            "id",
            "issued_at",
            "cross_project_reason",
        ],
    )?;
    let action = match map.get("action").and_then(Value::as_str) {
        Some("preview") => HandoffAction::Preview,
        Some("send") => HandoffAction::Send,
        _ => return Err(err("action", "preview ou send requis")),
    };
    let draft = map
        .get("draft")
        .ok_or_else(|| err("draft", "champ obligatoire absent"))?;
    if action == HandoffAction::Preview {
        if let Some(field) = TRANSPORT_FIELDS
            .iter()
            .find(|field| map.contains_key(**field))
        {
            return Err(err(
                *field,
                "paramètre de transport refusé en aperçu : preview n'envoie rien",
            ));
        }
        return Ok(HandoffRequest {
            action,
            rendered: validate_and_render(draft)?,
            transport: None,
        });
    }
    let to = required_text(map, "to", MAX_LABEL_BYTES)?;
    if !canonical_uuid(&to) {
        return Err(err(
            "to",
            "UUID canonique du destinataire requis (résoudre le nom par l'annuaire avant l'envoi)",
        ));
    }
    let reply = match map.get("reply") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(err("reply", "booléen attendu (null refusé)")),
    };
    let reply_timeout = match map.get("reply_timeout") {
        None => None,
        Some(Value::Number(number)) if number.as_u64().is_some_and(|value| value > 0) => {
            number.as_u64()
        }
        Some(_) => return Err(err("reply_timeout", "entier strictement positif attendu")),
    };
    if reply_timeout.is_some() && !reply {
        return Err(err("reply_timeout", "réservé à reply=true"));
    }
    let in_reply_to = optional_text(map, "in_reply_to", MAX_ITEM_BYTES)?;
    let cross_project_reason = map
        .get("cross_project_reason")
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| err("cross_project_reason", "chaîne attendue (null refusé)"))
        })
        .transpose()?;
    let cross_project_reason =
        crate::communication::validate_cross_project_reason(cross_project_reason)
            .map_err(|reason| err("cross_project_reason", reason))?;
    let id = optional_text(map, "id", MAX_LABEL_BYTES)?;
    let issued_at = match map.get("issued_at") {
        None => None,
        Some(Value::Number(number)) if number.as_i64().is_some_and(|value| value > 0) => {
            number.as_i64()
        }
        Some(_) => return Err(err("issued_at", "instant Unix entier positif attendu")),
    };
    if id.is_some() != issued_at.is_some() {
        return Err(err(
            "id",
            "id et issued_at doivent être fournis ensemble pour rejouer le même envoi",
        ));
    }
    Ok(HandoffRequest {
        action,
        rendered: validate_and_render(draft)?,
        transport: Some(HandoffTransport {
            to,
            reply,
            reply_timeout,
            in_reply_to,
            id,
            issued_at,
            cross_project_reason,
        }),
    })
}

/// Résultat d'aperçu : aucun identifiant ni date d'envoi n'est fabriqué.
pub(crate) fn preview_result(rendered: &Rendered) -> Value {
    serde_json::json!({
        "status": "preview_valid",
        "handoff_version": HANDOFF_VERSION,
        "body": rendered.body,
        "bytes": rendered.bytes,
        "warnings": WARNINGS,
        "warning_details": WARNING_DETAILS,
    })
}

/// Complète un reçu d'envoi 099 avec les faits constants du dossier, sans le corps.
pub(crate) fn decorate_send_result(mut receipt: Value, rendered: &Rendered) -> Value {
    if let Value::Object(map) = &mut receipt {
        map.insert("handoff_version".into(), Value::from(HANDOFF_VERSION));
        map.insert("bytes".into(), Value::from(rendered.bytes as u64));
        map.insert("warnings".into(), serde_json::json!(WARNINGS));
    }
    receipt
}

#[cfg(test)]
#[path = "handoff_tests.rs"]
mod tests;
