//! Observations de catalogue, jamais autorisations de compte ou de lancement.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MAX_CATALOGUE_MODELS: usize = 256;
pub const MAX_CATALOGUE_CONNECTIONS: usize = 32;
pub const MAX_CATALOGUE_BYTES: usize = 64 * 1024;
pub const CATALOGUE_TTL_SECS: u64 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogueState {
    Refreshing,
    Observed,
    SignInRequired,
    Unavailable,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogueCause {
    ProgramUnavailable,
    ProtocolUnavailable,
    TimedOut,
    OutputLimit,
    AccountChanged,
    AccountUnidentified,
    StopUnconfirmed,
    DefinitionChanged,
    SourceUnavailable,
    UnsupportedProvider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogueModel {
    pub model: String,
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderCatalogue {
    pub connection_id: String,
    pub definition_digest: String,
    pub observed_at: i64,
    pub state: CatalogueState,
    /// Nonce aléatoire local ; aucune adresse ni empreinte dérivée n’est publiée.
    pub account_scope: Option<String>,
    pub models: Vec<CatalogueModel>,
    pub cause: Option<CatalogueCause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerCatalogueUpdate {
    pub version: u16,
    pub worker: crate::protocol::WorkerIdentity,
    pub connection_epoch: String,
    pub sequence: u64,
    pub refresh_id: String,
    pub catalogue: ProviderCatalogue,
}

pub fn catalogue_token(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._:/-".contains(&c))
}
fn digest(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

impl CatalogueModel {
    pub fn valid(&self) -> bool {
        catalogue_token(&self.model)
            && self.supported_efforts.len() <= 16
            && self.supported_efforts.iter().all(|e| catalogue_token(e))
            && self.supported_efforts.iter().collect::<BTreeSet<_>>().len()
                == self.supported_efforts.len()
            && self
                .default_effort
                .as_ref()
                .is_none_or(|e| self.supported_efforts.contains(e))
    }
}
impl ProviderCatalogue {
    pub fn valid(&self) -> bool {
        serde_json::to_vec(self).is_ok_and(|v| v.len() <= MAX_CATALOGUE_BYTES)
            && catalogue_token(&self.connection_id)
            && digest(&self.definition_digest)
            && self.observed_at > 0
            && self.models.len() <= MAX_CATALOGUE_MODELS
            && self.models.iter().all(CatalogueModel::valid)
            && self
                .models
                .iter()
                .map(|m| &m.model)
                .collect::<BTreeSet<_>>()
                .len()
                == self.models.len()
            && self
                .account_scope
                .as_deref()
                .is_none_or(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()))
            && match self.state {
                CatalogueState::Observed => {
                    !self.models.is_empty()
                        && self
                            .cause
                            .is_none_or(|c| c == CatalogueCause::AccountUnidentified)
                }
                _ => self.models.is_empty() && self.account_scope.is_none(),
            }
    }
}
impl WorkerCatalogueUpdate {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.worker.validate().is_ok()
            && catalogue_token(&self.connection_epoch)
            && catalogue_token(&self.refresh_id)
            && self.sequence > 0
            && self.catalogue.valid()
    }
}

/// Clé de comparaison PRIVÉE à la sonde, jamais transmise. La projection
/// publique emploie un nonce aléatoire, sans dérivation de l’adresse.
pub fn account_comparison(value: &Value, salt: &[u8]) -> Result<Option<String>, CatalogueCause> {
    let required = value
        .get("requiresOpenaiAuth")
        .and_then(Value::as_bool)
        .ok_or(CatalogueCause::ProtocolUnavailable)?;
    let Some(account) = value.get("account").filter(|v| !v.is_null()) else {
        return if required {
            Ok(None)
        } else {
            Err(CatalogueCause::AccountUnidentified)
        };
    };
    let kind = account
        .get("type")
        .and_then(Value::as_str)
        .ok_or(CatalogueCause::ProtocolUnavailable)?;
    // apiKey ne donne aucun identifiant public stable : ne pas en inventer un.
    if kind != "chatgpt" {
        return Err(CatalogueCause::AccountUnidentified);
    }
    let email = account
        .get("email")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or(CatalogueCause::AccountUnidentified)?;
    let plan = account
        .get("planType")
        .and_then(Value::as_str)
        .ok_or(CatalogueCause::ProtocolUnavailable)?;
    if email.len() > 1024 || plan.len() > 128 || salt.len() < 16 {
        return Err(CatalogueCause::ProtocolUnavailable);
    }
    let mut hash = Sha256::new();
    hash.update(salt);
    hash.update([0]);
    hash.update(kind);
    hash.update([0]);
    hash.update(email);
    hash.update([0]);
    hash.update(plan);
    Ok(Some(format!("{:x}", hash.finalize())))
}

/// Une page entière est acceptée ou refusée ; jamais de liste silencieusement tronquée.
pub fn model_page(value: &Value) -> Result<(Vec<CatalogueModel>, Option<String>), CatalogueCause> {
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or(CatalogueCause::ProtocolUnavailable)?;
    if data.len() > MAX_CATALOGUE_MODELS {
        return Err(CatalogueCause::OutputLimit);
    }
    let mut models = Vec::new();
    for item in data {
        let model = item
            .get("model")
            .and_then(Value::as_str)
            .ok_or(CatalogueCause::ProtocolUnavailable)?
            .to_owned();
        let raw = item
            .get("supportedReasoningEfforts")
            .and_then(Value::as_array)
            .ok_or(CatalogueCause::ProtocolUnavailable)?;
        if raw.len() > 16 {
            return Err(CatalogueCause::OutputLimit);
        }
        let efforts = raw
            .iter()
            .map(|e| {
                e.get("reasoningEffort")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or(CatalogueCause::ProtocolUnavailable)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let default_effort = item
            .get("defaultReasoningEffort")
            .filter(|v| !v.is_null())
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or(CatalogueCause::ProtocolUnavailable)
            })
            .transpose()?;
        let model = CatalogueModel {
            model,
            supported_efforts: efforts,
            default_effort,
        };
        if !model.valid() {
            return Err(CatalogueCause::ProtocolUnavailable);
        }
        models.push(model);
    }
    let next = value
        .get("nextCursor")
        .filter(|v| !v.is_null())
        .map(|v| {
            v.as_str()
                .filter(|s| !s.is_empty() && s.len() <= 1024)
                .map(str::to_owned)
                .ok_or(CatalogueCause::ProtocolUnavailable)
        })
        .transpose()?;
    Ok((models, next))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn spec220_account_scopes_are_opaque_local_and_change_with_account() {
        let a = json!({"account":{"type":"chatgpt","email":"PRIVATE_ACCOUNT","planType":"pro"},"requiresOpenaiAuth":true});
        let mut b = a.clone();
        b["account"]["email"] = json!("OTHER_ACCOUNT");
        let salt = b"0123456789abcdef";
        let scope = account_comparison(&a, salt).unwrap().unwrap();
        assert!(!scope.contains("PRIVATE"));
        assert_ne!(Some(scope.clone()), account_comparison(&b, salt).unwrap());
        assert_ne!(
            Some(scope),
            account_comparison(&a, b"fedcba9876543210").unwrap()
        );
        assert_eq!(
            account_comparison(&json!({"account":null,"requiresOpenaiAuth":true}), salt),
            Ok(None)
        );
        assert_eq!(
            account_comparison(
                &json!({"account":{"type":"apiKey"},"requiresOpenaiAuth":true}),
                salt
            ),
            Err(CatalogueCause::AccountUnidentified)
        );
    }
    #[test]
    fn spec220_model_page_preserves_efforts_and_refuses_unbounded_or_foreign_shapes() {
        let page = json!({"data":[{"model":"model-a","supportedReasoningEfforts":[{"reasoningEffort":"low"}],"defaultReasoningEffort":"low"}],"nextCursor":"next"});
        let (models, next) = model_page(&page).unwrap();
        assert_eq!(next.as_deref(), Some("next"));
        assert_eq!(models[0].supported_efforts, vec!["low"]);
        let mut bad = page.clone();
        bad["data"][0]["model"] = json!("unsafe\nname");
        assert!(model_page(&bad).is_err());
        bad = page.clone();
        bad["data"][0]["defaultReasoningEffort"] = json!("other");
        assert!(model_page(&bad).is_err());
        bad = page.clone();
        bad["nextCursor"] = json!(42);
        assert!(model_page(&bad).is_err());
        bad = page.clone();
        bad["data"] = json!(vec![page["data"][0].clone(); 257]);
        assert_eq!(model_page(&bad), Err(CatalogueCause::OutputLimit));
    }
}
