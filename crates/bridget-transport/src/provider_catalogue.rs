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
    /// Une métadonnée facultative invalide ferme ses options, pas les modèles.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub service_tiers_unavailable: bool,
    pub model: String,
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub service_tiers: Vec<String>,
    /// Métadonnée annoncée seulement ; ne sélectionne jamais un niveau.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_service_tier: Option<String>,
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
            && (!self.service_tiers_unavailable
                || (self.service_tiers.is_empty() && self.default_service_tier.is_none()))
            && self.service_tiers.len() <= 16
            && self.service_tiers.iter().all(|v| catalogue_token(v))
            && self.service_tiers.iter().collect::<BTreeSet<_>>().len() == self.service_tiers.len()
            && self
                .default_service_tier
                .as_deref()
                .is_none_or(catalogue_token)
    }
}
impl ProviderCatalogue {
    /// Projection de compatibilité d'un pair sans ModelOptionsV1.
    /// La source conservée par le collecteur n'est jamais réécrite.
    pub fn legacy_projection(mut self) -> Self {
        for model in &mut self.models {
            model.service_tiers_unavailable = false;
            model.service_tiers.clear();
            model.default_service_tier = None;
        }
        self
    }

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
        let options = (|| -> Result<(Vec<String>, Option<String>), ()> {
            let tiers = match item.get("serviceTiers") {
                None => Vec::new(),
                Some(value) => {
                    let rows = value.as_array().ok_or(())?;
                    if rows.len() > 16 {
                        return Err(());
                    }
                    rows.iter()
                        .map(|tier| {
                            tier.get("id")
                                .and_then(Value::as_str)
                                .filter(|v| catalogue_token(v))
                                .map(str::to_owned)
                                .ok_or(())
                        })
                        .collect::<Result<Vec<_>, _>>()?
                }
            };
            if tiers.iter().collect::<BTreeSet<_>>().len() != tiers.len() {
                return Err(());
            }
            let default = item
                .get("defaultServiceTier")
                .filter(|v| !v.is_null())
                .map(|v| {
                    v.as_str()
                        .filter(|v| catalogue_token(v))
                        .map(str::to_owned)
                        .ok_or(())
                })
                .transpose()?;
            Ok((tiers, default))
        })();
        let service_tiers_unavailable = options.is_err();
        let (service_tiers, default_service_tier) = options.unwrap_or_default();
        let model = CatalogueModel {
            service_tiers_unavailable,
            model,
            supported_efforts: efforts,
            default_effort,
            service_tiers,
            default_service_tier,
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
    fn spec221_c05_model_page_preserves_service_tiers_and_never_applies_a_default() {
        let page = json!({"data":[{"model":"model-a",
            "supportedReasoningEfforts":[{"reasoningEffort":"low"}],"defaultReasoningEffort":"low",
            "serviceTiers":[{"id":"default","name":"Standard","description":"PRIVATE_PROVIDER_TEXT"},
                {"id":"priority","name":"Priority","description":"PRIVATE_PROVIDER_TEXT"}],
            "defaultServiceTier":"default"}],"nextCursor":null});
        let (models, next) = model_page(&page).unwrap();
        assert!(next.is_none());
        let value = serde_json::to_value(&models[0]).unwrap();
        assert_eq!(value["service_tiers"], json!(["default", "priority"]));
        assert_eq!(value["default_service_tier"], "default");
        assert!(!value.to_string().contains("PRIVATE_PROVIDER_TEXT"));
        let legacy = json!({"data":[{"model":"legacy",
            "supportedReasoningEfforts":[],"defaultReasoningEffort":null}],
            "nextCursor":null});
        let old = serde_json::to_value(&model_page(&legacy).unwrap().0[0]).unwrap();
        assert_eq!(
            old,
            json!({"model":"legacy","supported_efforts":[],"default_effort":null})
        );
        let mut deprecated = legacy;
        deprecated["data"][0]["additionalSpeedTiers"] = json!(["priority"]);
        assert_eq!(
            serde_json::to_value(&model_page(&deprecated).unwrap().0[0]).unwrap(),
            old
        );
    }

    #[test]
    fn spec221_c05_model_page_preserves_models_when_optional_service_tiers_are_invalid() {
        let page = json!({"data":[{"model":"model-a",
            "supportedReasoningEfforts":[],"defaultReasoningEffort":null,
            "serviceTiers":[{"id":"default","name":"Standard","description":"standard"}],
            "defaultServiceTier":"default"}],"nextCursor":null});
        for bad in [
            json!(true),
            json!([{}]),
            json!([{"id":"bad\nvalue"}]),
            json!([{"id":"default"},{"id":"default"}]),
        ] {
            let mut value = page.clone();
            value["data"][0]["serviceTiers"] = bad;
            let (models, _) = model_page(&value).unwrap();
            assert_eq!(models[0].model, "model-a");
            assert!(models[0].service_tiers_unavailable);
            assert!(models[0].service_tiers.is_empty());
            assert!(models[0].default_service_tier.is_none());
        }
        let mut large = page.clone();
        large["data"][0]["serviceTiers"] = json!(
            (0..17)
                .map(|n| json!({"id":format!("tier-{n}")}))
                .collect::<Vec<_>>()
        );
        let (models, _) = model_page(&large).unwrap();
        assert!(models[0].service_tiers_unavailable);
        let mut invalid = page;
        invalid["data"][0]["defaultServiceTier"] = json!(true);
        let (models, _) = model_page(&invalid).unwrap();
        assert!(models[0].service_tiers_unavailable);
        assert_eq!(models[0].model, "model-a");
    }

    #[test]
    fn spec221_c05_legacy_projection_keeps_the_original_wire_and_bounds() {
        let value = json!({"connection_id":"main","definition_digest":"a".repeat(64),
            "observed_at":1,"state":"observed","account_scope":"b".repeat(32),"cause":null,
            "models":[{"model":"model-a","supported_efforts":["low"],"default_effort":"low",
                "service_tiers":["default","priority"],"default_service_tier":"default"}]});
        let catalogue: ProviderCatalogue = serde_json::from_value(value).unwrap();
        assert!(catalogue.valid());
        let original = serde_json::to_value(&catalogue).unwrap();
        let legacy = catalogue.clone().legacy_projection();
        assert!(legacy.valid());
        assert_eq!(
            serde_json::to_string(&legacy.models[0]).unwrap(),
            r#"{"model":"model-a","supported_efforts":["low"],"default_effort":"low"}"#
        );
        assert_eq!(serde_json::to_value(&catalogue).unwrap(), original);
        let mut invalid = catalogue.clone();
        invalid.models[0].service_tiers = vec!["default".into(), "default".into()];
        assert!(!invalid.valid());
        invalid = catalogue.clone();
        invalid.models[0].default_service_tier = Some("bad\nvalue".into());
        assert!(!invalid.valid());
        let mut large = catalogue;
        large.models = (0..MAX_CATALOGUE_MODELS)
            .map(|n| CatalogueModel {
                service_tiers_unavailable: false,
                model: format!("model-{n}"),
                supported_efforts: vec![],
                default_effort: None,
                service_tiers: (0..16)
                    .map(|n| format!("tier-{n}-{}", "x".repeat(200)))
                    .collect(),
                default_service_tier: None,
            })
            .collect();
        assert!(serde_json::to_vec(&large).unwrap().len() > MAX_CATALOGUE_BYTES);
        assert!(
            !large.valid(),
            "le nouveau catalogue ne relève pas la borne64Kio"
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
