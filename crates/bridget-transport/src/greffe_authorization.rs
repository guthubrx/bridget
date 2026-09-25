//! Autorisation fermée des mutations du greffe central.
//!
//! Le client ne fournit jamais son principal ni un jeton dans la requête
//! métier. Bridget centralise le principal depuis la connexion enregistrée,
//! confronte ce fait à une politique privée, puis émet une attestation liée à
//! l'action, au `request_id` et au condensé des octets canoniques exacts.
//! le service compagnon recharge la politique et vérifie cette attestation sur les octets
//! relus, juste avant l'effet : recopier ou modifier une ancienne ligne du
//! guichet ne suffit donc pas à fabriquer une autorisation.
//!
//! L'attestation signe sa version, le principal, l'action, l'`issuer_scope`,
//! le `request_id`, l'instant d'émission, le SHA-256 de la requête canonique,
//! l'expiration du droit et la génération de politique. Elle ne signe pas
//! l'instant local d'observation ni la mécanique de relève (`claim_token`,
//! génération et lease), qui restent des états serveur et ne fondent aucun
//! droit métier.
//!
//! Limite de confiance : le nom et l'instance de cette connexion sont déclarés
//! par `Register`. Le daemon ne les vérifie pas contre la filiation du processus
//! pair. La garde suppose donc des processus non hostiles sous le même compte ;
//! un client local capable de parler le protocole brut peut déclarer l'identité
//! d'un autre agent. Centraliser la déclaration réduit les sources d'identité,
//! mais ne l'authentifie pas.

use crate::greffe_policy_refresh::MarkerSource;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

pub const GREFFE_POLICY_VERSION: u16 = 1;
pub const GREFFE_ATTESTATION_VERSION: u16 = 1;
pub const GREFFE_AUTHORIZATION_PUBLIC_REFUSAL: &str = "greffe_authorization_denied";
pub const GREFFE_POLICY_PATH_ENV: &str = "BRIDGET_GREFFE_POLICY_PATH";
pub const GREFFE_AUDIT_PATH_ENV: &str = "BRIDGET_GREFFE_AUDIT_PATH";

const MAX_POLICY_BYTES: u64 = 1024 * 1024;
const MAX_IDENTITY_BYTES: usize = 256;
const MAX_ISSUER_SCOPE_BYTES: usize = 256;
const MAX_REQUEST_ID_BYTES: usize = 256;
const HMAC_BLOCK_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GreffeMutationAction {
    Delegate,
    RegistreAdd,
    ObjectiveClose,
}

impl GreffeMutationAction {
    pub const ALL: [Self; 3] = [Self::Delegate, Self::RegistreAdd, Self::ObjectiveClose];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Delegate => "delegate",
            Self::RegistreAdd => "registre_add",
            Self::ObjectiveClose => "objective_close",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GreffePrincipal {
    pub name: String,
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GreffeAuthorizationAttestation {
    pub version: u16,
    pub principal: GreffePrincipal,
    pub action: GreffeMutationAction,
    pub issuer_scope: String,
    pub request_id: String,
    pub request_issued_at: i64,
    pub canonical_request_sha256: String,
    pub grant_expires_at: i64,
    pub policy_generation: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GreffeAuthorizationStage {
    Deposit,
    Effect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreffeAuthorizationRefusal {
    PrincipalNameMissing,
    PrincipalInstanceMissing,
    InvalidPrincipal,
    EphemeralCliForbidden,
    DeclaredPrincipalMismatch,
    PolicyUnavailable,
    PolicyInvalid,
    PrincipalUnknown,
    ActionDenied,
    InstanceUnknown,
    GrantExpired,
    GrantRevoked,
    AttestationMissing,
    AttestationMismatch,
    AttestationInvalid,
    AuditUnavailable,
}

impl GreffeAuthorizationRefusal {
    /// Motif détaillé réservé au journal du daemon. Le fil n'expose que
    /// [`GREFFE_AUTHORIZATION_PUBLIC_REFUSAL`].
    pub const fn audit_code(self) -> &'static str {
        match self {
            Self::PrincipalNameMissing => "principal_name_missing",
            Self::PrincipalInstanceMissing => "principal_instance_missing",
            Self::InvalidPrincipal => "invalid_principal",
            Self::EphemeralCliForbidden => "ephemeral_cli_forbidden",
            Self::DeclaredPrincipalMismatch => "declared_principal_mismatch",
            Self::PolicyUnavailable => "policy_unavailable",
            Self::PolicyInvalid => "policy_invalid",
            Self::PrincipalUnknown => "principal_unknown",
            Self::ActionDenied => "action_denied",
            Self::InstanceUnknown => "instance_unknown",
            Self::GrantExpired => "grant_expired",
            Self::GrantRevoked => "grant_revoked",
            Self::AttestationMissing => "attestation_missing",
            Self::AttestationMismatch => "attestation_mismatch",
            Self::AttestationInvalid => "attestation_invalid",
            Self::AuditUnavailable => "audit_unavailable",
        }
    }

    pub const fn public_reason(self) -> &'static str {
        GREFFE_AUTHORIZATION_PUBLIC_REFUSAL
    }
}

impl fmt::Display for GreffeAuthorizationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.audit_code())
    }
}

impl std::error::Error for GreffeAuthorizationRefusal {}

pub struct GreffeDepositAuthorization<'a> {
    pub canonical_name: Option<&'a str>,
    pub canonical_instance_id: Option<&'a str>,
    pub declared_from: Option<&'a str>,
    pub action: GreffeMutationAction,
    pub issuer_scope: &'a str,
    pub request_id: &'a str,
    pub request_issued_at: i64,
    pub canonical_request: &'a [u8],
    pub observed_at: i64,
}

pub struct GreffeEffectAuthorization<'a> {
    pub attestation: Option<&'a GreffeAuthorizationAttestation>,
    pub action: GreffeMutationAction,
    pub issuer_scope: &'a str,
    pub request_id: &'a str,
    pub request_issued_at: i64,
    pub canonical_request: &'a [u8],
    pub observed_at: i64,
}

#[derive(Debug, Clone)]
pub struct GreffeAuthorizationGate {
    policy_path: PathBuf,
    audit_path: PathBuf,
}

impl GreffeAuthorizationGate {
    pub fn new(policy_path: impl Into<PathBuf>, audit_path: impl Into<PathBuf>) -> Self {
        Self {
            policy_path: policy_path.into(),
            audit_path: audit_path.into(),
        }
    }

    pub fn from_environment() -> Self {
        Self::new(default_policy_path(), default_audit_path())
    }

    pub fn policy_path(&self) -> &Path {
        &self.policy_path
    }

    pub fn audit_path(&self) -> &Path {
        &self.audit_path
    }

    /// Autorise le dépôt et produit une attestation serveur. Tout échec de
    /// journalisation transforme aussi un succès potentiel en refus.
    pub fn authorize_deposit(
        &self,
        attempt: GreffeDepositAuthorization<'_>,
    ) -> Result<GreffeAuthorizationAttestation, GreffeAuthorizationRefusal> {
        let principal = canonical_principal(attempt.canonical_name, attempt.canonical_instance_id);
        let audit_principal = principal.as_ref().ok().cloned();
        let canonical_request_sha256 = sha256_hex(attempt.canonical_request);
        let result = principal.and_then(|principal| {
            if attempt.declared_from != Some(principal.name.as_str()) {
                return Err(GreffeAuthorizationRefusal::DeclaredPrincipalMismatch);
            }
            validate_request(
                attempt.issuer_scope,
                attempt.request_id,
                attempt.request_issued_at,
            )?;
            let policy = GreffePolicy::load(&self.policy_path)?;
            policy.attest(
                principal,
                attempt.action,
                attempt.issuer_scope,
                attempt.request_id,
                attempt.request_issued_at,
                &canonical_request_sha256,
                attempt.observed_at,
            )
        });
        self.finish_with_audit(
            GreffeAuthorizationStage::Deposit,
            audit_principal.as_ref(),
            attempt.action,
            attempt.issuer_scope,
            attempt.request_id,
            attempt.observed_at,
            result,
        )
    }

    /// Exécute l'écriture durable seulement après l'autorisation et son audit.
    /// Cette forme rend l'ordre « garde puis effet » structurel pour l'appelant.
    pub fn authorize_deposit_then<T>(
        &self,
        attempt: GreffeDepositAuthorization<'_>,
        deposit: impl FnOnce(&GreffeAuthorizationAttestation) -> T,
    ) -> Result<T, GreffeAuthorizationRefusal> {
        let attestation = self.authorize_deposit(attempt)?;
        Ok(deposit(&attestation))
    }

    /// Revérifie l'attestation et la politique courante juste avant l'effet.
    /// Une révocation ou expiration postérieure au dépôt est donc effective.
    pub fn authorize_effect(
        &self,
        attempt: GreffeEffectAuthorization<'_>,
    ) -> Result<GreffePrincipal, GreffeAuthorizationRefusal> {
        let canonical_request_sha256 = sha256_hex(attempt.canonical_request);
        let result = (|| {
            let attestation = attempt
                .attestation
                .ok_or(GreffeAuthorizationRefusal::AttestationMissing)?;
            validate_request(
                attempt.issuer_scope,
                attempt.request_id,
                attempt.request_issued_at,
            )?;
            let policy = GreffePolicy::load(&self.policy_path)?;
            policy.verify(
                attestation,
                attempt.action,
                attempt.issuer_scope,
                attempt.request_id,
                attempt.request_issued_at,
                &canonical_request_sha256,
                attempt.observed_at,
            )
        })();
        let principal = attempt.attestation.map(|value| &value.principal);
        self.finish_with_audit(
            GreffeAuthorizationStage::Effect,
            principal,
            attempt.action,
            attempt.issuer_scope,
            attempt.request_id,
            attempt.observed_at,
            result,
        )
    }

    /// Exécute la mutation métier seulement après la seconde vérification et
    /// son audit durable.
    pub fn authorize_effect_then<T>(
        &self,
        attempt: GreffeEffectAuthorization<'_>,
        effect: impl FnOnce(&GreffePrincipal) -> T,
    ) -> Result<T, GreffeAuthorizationRefusal> {
        let principal = self.authorize_effect(attempt)?;
        Ok(effect(&principal))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_with_audit<T>(
        &self,
        stage: GreffeAuthorizationStage,
        principal: Option<&GreffePrincipal>,
        action: GreffeMutationAction,
        issuer_scope: &str,
        request_id: &str,
        observed_at: i64,
        result: Result<T, GreffeAuthorizationRefusal>,
    ) -> Result<T, GreffeAuthorizationRefusal> {
        let (allowed, reason) = match &result {
            Ok(_) => (true, "authorized"),
            Err(refusal) => (false, refusal.audit_code()),
        };
        if let Err(error) = append_audit(
            &self.audit_path,
            GreffeAuditEvent {
                version: GREFFE_POLICY_VERSION,
                observed_at,
                stage,
                principal,
                action,
                issuer_scope,
                request_id,
                allowed,
                reason,
            },
        ) {
            log::error!(
                "journal d'autorisation du greffe indisponible ({}): {error}",
                self.audit_path.display()
            );
            return Err(GreffeAuthorizationRefusal::AuditUnavailable);
        }
        result
    }
}

pub fn default_policy_path() -> PathBuf {
    explicit_or_home_path(
        GREFFE_POLICY_PATH_ENV,
        ".config/bridget/greffe-authorization.json",
    )
}

pub fn default_audit_path() -> PathBuf {
    explicit_or_home_path(
        GREFFE_AUDIT_PATH_ENV,
        ".cache/bridget/greffe-authorization.jsonl",
    )
}

fn explicit_or_home_path(variable: &str, suffix: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(suffix)))
        .unwrap_or_else(|| PathBuf::from("/nonexistent/bridget").join(suffix))
}

fn canonical_principal(
    name: Option<&str>,
    instance_id: Option<&str>,
) -> Result<GreffePrincipal, GreffeAuthorizationRefusal> {
    let name = name.ok_or(GreffeAuthorizationRefusal::PrincipalNameMissing)?;
    let instance_id = instance_id.ok_or(GreffeAuthorizationRefusal::PrincipalInstanceMissing)?;
    if !is_valid_greffe_identity_component(name) || !is_valid_greffe_identity_component(instance_id)
    {
        return Err(GreffeAuthorizationRefusal::InvalidPrincipal);
    }
    if name.starts_with("cli-send-") {
        return Err(GreffeAuthorizationRefusal::EphemeralCliForbidden);
    }
    Ok(GreffePrincipal {
        name: name.to_string(),
        instance_id: instance_id.to_string(),
    })
}

pub fn is_valid_greffe_identity_component(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_IDENTITY_BYTES && !value.chars().any(char::is_control)
}

fn validate_request(
    issuer_scope: &str,
    request_id: &str,
    request_issued_at: i64,
) -> Result<(), GreffeAuthorizationRefusal> {
    if issuer_scope.is_empty()
        || issuer_scope.len() > MAX_ISSUER_SCOPE_BYTES
        || issuer_scope.chars().any(char::is_control)
        || request_id.is_empty()
        || request_id.len() > MAX_REQUEST_ID_BYTES
        || request_id.chars().any(char::is_control)
        || request_issued_at <= 0
    {
        return Err(GreffeAuthorizationRefusal::AttestationMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GreffePolicyFile {
    pub(crate) version: u16,
    pub(crate) generation: u64,
    pub(crate) attestation_key: String,
    pub(crate) principals: Vec<PrincipalPolicyFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrincipalPolicyFile {
    pub(crate) principal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) marker_source: Option<MarkerSource>,
    pub(crate) actions: Vec<GreffeMutationAction>,
    pub(crate) instances: Vec<InstancePolicyFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstancePolicyFile {
    pub(crate) instance_id: String,
    pub(crate) expires_at: i64,
    pub(crate) revoked: bool,
}

struct GreffePolicy {
    generation: u64,
    attestation_key: [u8; 32],
    principals: BTreeMap<String, PrincipalPolicy>,
}

struct PrincipalPolicy {
    actions: BTreeSet<GreffeMutationAction>,
    instances: BTreeMap<String, InstancePolicy>,
}

#[derive(Clone, Copy)]
struct InstancePolicy {
    expires_at: i64,
    revoked: bool,
}

impl GreffePolicy {
    fn load(path: &Path) -> Result<Self, GreffeAuthorizationRefusal> {
        let file = load_policy_file(path)?;
        Self::from_file(&file)
    }

    fn from_file(file: &GreffePolicyFile) -> Result<Self, GreffeAuthorizationRefusal> {
        if file.version != GREFFE_POLICY_VERSION || file.generation == 0 {
            return Err(GreffeAuthorizationRefusal::PolicyInvalid);
        }
        let attestation_key = decode_key(&file.attestation_key)?;
        let mut principals = BTreeMap::new();
        for entry in &file.principals {
            if !is_valid_greffe_identity_component(&entry.principal)
                || entry.actions.is_empty()
                || entry.instances.is_empty()
            {
                return Err(GreffeAuthorizationRefusal::PolicyInvalid);
            }
            let action_count = entry.actions.len();
            let actions = entry.actions.iter().copied().collect::<BTreeSet<_>>();
            if actions.len() != action_count {
                return Err(GreffeAuthorizationRefusal::PolicyInvalid);
            }
            let mut instances = BTreeMap::new();
            for instance in &entry.instances {
                if !is_valid_greffe_identity_component(&instance.instance_id)
                    || instance.expires_at <= 0
                {
                    return Err(GreffeAuthorizationRefusal::PolicyInvalid);
                }
                if instances
                    .insert(
                        instance.instance_id.clone(),
                        InstancePolicy {
                            expires_at: instance.expires_at,
                            revoked: instance.revoked,
                        },
                    )
                    .is_some()
                {
                    return Err(GreffeAuthorizationRefusal::PolicyInvalid);
                }
            }
            if principals
                .insert(
                    entry.principal.clone(),
                    PrincipalPolicy { actions, instances },
                )
                .is_some()
            {
                return Err(GreffeAuthorizationRefusal::PolicyInvalid);
            }
        }
        Ok(Self {
            generation: file.generation,
            attestation_key,
            principals,
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn attest(
        &self,
        principal: GreffePrincipal,
        action: GreffeMutationAction,
        issuer_scope: &str,
        request_id: &str,
        request_issued_at: i64,
        canonical_request_sha256: &str,
        now: i64,
    ) -> Result<GreffeAuthorizationAttestation, GreffeAuthorizationRefusal> {
        let grant = self.active_grant(&principal, action, now)?;
        let mut attestation = GreffeAuthorizationAttestation {
            version: GREFFE_ATTESTATION_VERSION,
            principal,
            action,
            issuer_scope: issuer_scope.to_string(),
            request_id: request_id.to_string(),
            request_issued_at,
            canonical_request_sha256: canonical_request_sha256.to_string(),
            grant_expires_at: grant.expires_at,
            policy_generation: self.generation,
            signature: String::new(),
        };
        attestation.signature = signature(&self.attestation_key, &attestation);
        Ok(attestation)
    }

    #[allow(clippy::too_many_arguments)]
    fn verify(
        &self,
        attestation: &GreffeAuthorizationAttestation,
        action: GreffeMutationAction,
        issuer_scope: &str,
        request_id: &str,
        request_issued_at: i64,
        canonical_request_sha256: &str,
        now: i64,
    ) -> Result<GreffePrincipal, GreffeAuthorizationRefusal> {
        if attestation.version != GREFFE_ATTESTATION_VERSION
            || attestation.action != action
            || attestation.issuer_scope != issuer_scope
            || attestation.request_id != request_id
            || attestation.request_issued_at != request_issued_at
            || attestation.canonical_request_sha256 != canonical_request_sha256
            || attestation.grant_expires_at <= now
            || attestation.policy_generation == 0
        {
            return Err(GreffeAuthorizationRefusal::AttestationMismatch);
        }
        let expected = signature(&self.attestation_key, attestation);
        if !constant_time_eq(expected.as_bytes(), attestation.signature.as_bytes()) {
            return Err(GreffeAuthorizationRefusal::AttestationInvalid);
        }
        let current = self.active_grant(&attestation.principal, action, now)?;
        if attestation.grant_expires_at > current.expires_at {
            return Err(GreffeAuthorizationRefusal::GrantExpired);
        }
        Ok(attestation.principal.clone())
    }

    fn active_grant(
        &self,
        principal: &GreffePrincipal,
        action: GreffeMutationAction,
        now: i64,
    ) -> Result<InstancePolicy, GreffeAuthorizationRefusal> {
        let policy = self
            .principals
            .get(&principal.name)
            .ok_or(GreffeAuthorizationRefusal::PrincipalUnknown)?;
        if !policy.actions.contains(&action) {
            return Err(GreffeAuthorizationRefusal::ActionDenied);
        }
        let instance = policy
            .instances
            .get(&principal.instance_id)
            .copied()
            .ok_or(GreffeAuthorizationRefusal::InstanceUnknown)?;
        if instance.revoked {
            return Err(GreffeAuthorizationRefusal::GrantRevoked);
        }
        if instance.expires_at <= now {
            return Err(GreffeAuthorizationRefusal::GrantExpired);
        }
        Ok(instance)
    }
}

pub(crate) fn load_policy_file(
    path: &Path,
) -> Result<GreffePolicyFile, GreffeAuthorizationRefusal> {
    load_policy_file_detailed(path).map_err(|error| match error {
        PolicyFileLoadError::Unavailable => GreffeAuthorizationRefusal::PolicyUnavailable,
        PolicyFileLoadError::UnsupportedType | PolicyFileLoadError::Invalid => {
            GreffeAuthorizationRefusal::PolicyInvalid
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyFileLoadError {
    Unavailable,
    UnsupportedType,
    Invalid,
}

pub(crate) fn load_policy_file_detailed(
    path: &Path,
) -> Result<GreffePolicyFile, PolicyFileLoadError> {
    let content = read_private_file_detailed(path)?;
    let file: GreffePolicyFile =
        serde_json::from_str(&content).map_err(|_| PolicyFileLoadError::Invalid)?;
    GreffePolicy::from_file(&file).map_err(|_| PolicyFileLoadError::Invalid)?;
    Ok(file)
}

fn read_private_file_detailed(path: &Path) -> Result<String, PolicyFileLoadError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| PolicyFileLoadError::Unavailable)?;
    let metadata = file
        .metadata()
        .map_err(|_| PolicyFileLoadError::Unavailable)?;
    if !metadata.is_file() {
        return Err(PolicyFileLoadError::UnsupportedType);
    }
    if metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > MAX_POLICY_BYTES
    {
        return Err(PolicyFileLoadError::Invalid);
    }
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|_| PolicyFileLoadError::Unavailable)?;
    Ok(content)
}

fn decode_key(value: &str) -> Result<[u8; 32], GreffeAuthorizationRefusal> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GreffeAuthorizationRefusal::PolicyInvalid);
    }
    let mut decoded = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let raw =
            std::str::from_utf8(pair).map_err(|_| GreffeAuthorizationRefusal::PolicyInvalid)?;
        decoded[index] =
            u8::from_str_radix(raw, 16).map_err(|_| GreffeAuthorizationRefusal::PolicyInvalid)?;
    }
    if decoded.iter().all(|byte| *byte == 0) {
        return Err(GreffeAuthorizationRefusal::PolicyInvalid);
    }
    Ok(decoded)
}

#[derive(Serialize)]
struct UnsignedAttestation<'a> {
    version: u16,
    principal: &'a GreffePrincipal,
    action: GreffeMutationAction,
    issuer_scope: &'a str,
    request_id: &'a str,
    request_issued_at: i64,
    canonical_request_sha256: &'a str,
    grant_expires_at: i64,
    policy_generation: u64,
}

fn signature(key: &[u8; 32], attestation: &GreffeAuthorizationAttestation) -> String {
    let canonical = serde_json::to_vec(&UnsignedAttestation {
        version: attestation.version,
        principal: &attestation.principal,
        action: attestation.action,
        issuer_scope: &attestation.issuer_scope,
        request_id: &attestation.request_id,
        request_issued_at: attestation.request_issued_at,
        canonical_request_sha256: &attestation.canonical_request_sha256,
        grant_expires_at: attestation.grant_expires_at,
        policy_generation: attestation.policy_generation,
    })
    .expect("attestation composée uniquement de valeurs sérialisables");
    hex(&hmac_sha256(key, &canonical))
}

fn hmac_sha256(key: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut block = [0_u8; HMAC_BLOCK_BYTES];
    if key.len() > HMAC_BLOCK_BYTES {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut inner_pad = [0x36_u8; HMAC_BLOCK_BYTES];
    let mut outer_pad = [0x5c_u8; HMAC_BLOCK_BYTES];
    for index in 0..HMAC_BLOCK_BYTES {
        inner_pad[index] ^= block[index];
        outer_pad[index] ^= block[index];
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(bytes);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

#[derive(Serialize)]
struct GreffeAuditEvent<'a> {
    version: u16,
    observed_at: i64,
    stage: GreffeAuthorizationStage,
    principal: Option<&'a GreffePrincipal>,
    action: GreffeMutationAction,
    issuer_scope: &'a str,
    request_id: &'a str,
    allowed: bool,
    reason: &'a str,
}

fn append_audit(path: &Path, event: GreffeAuditEvent<'_>) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    verify_private_regular_file(&file)?;
    let fd = file.as_raw_fd();
    if unsafe { libc::flock(fd, libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let result = (|| {
        serde_json::to_writer(&mut file, &event).map_err(std::io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_data()
    })();
    let unlock = unsafe { libc::flock(fd, libc::LOCK_UN) };
    result?;
    if unlock != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Vérifie sur le descripteur ouvert qu'un fichier sensible est régulier,
/// appartient à l'utilisateur effectif et n'est accessible ni au groupe ni aux autres.
pub fn verify_private_regular_file(file: &File) -> std::io::Result<()> {
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(std::io::Error::other("fichier régulier requis"));
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(std::io::Error::other("propriétaire inattendu"));
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(std::io::Error::other(format!(
            "permissions {mode:04o}, attendu 0600 ou plus restrictif"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW: i64 = 1_788_000_000;
    const ISSUER_SCOPE: &str = "026_scope_0123456789abcdef0123456789abcdef";
    const CANONICAL_REQUEST: &[u8] = br#"{"operation":"delegate","goal":"A"}"#;
    const CANONICAL_REQUEST_SHA256: &str =
        "beff6e8b9bca6da6bbaa448dc70686181c56f485c44544f61dca46fe690d5d59";
    const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        policy: PathBuf,
        audit: PathBuf,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "bridget-greffe-authorization-{label}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let fixture = Self {
                policy: root.join("policy.json"),
                audit: root.join("audit.jsonl"),
                root,
            };
            fixture.write_policy(false, NOW + 300);
            fixture
        }

        fn write_policy(&self, revoked: bool, expires_at: i64) {
            let content = json!({
                "version": 1,
                "generation": 7,
                "attestation_key": KEY,
                "principals": [
                    {
                        "principal": "agent-autorise",
                        "actions": ["delegate", "registre_add", "objective_close"],
                        "instances": [{
                            "instance_id": "instance-autorisee",
                            "expires_at": expires_at,
                            "revoked": revoked
                        }]
                    },
                    {
                        "principal": "cli-send-123",
                        "actions": ["delegate", "registre_add", "objective_close"],
                        "instances": [{
                            "instance_id": "instance-autorisee",
                            "expires_at": expires_at,
                            "revoked": revoked
                        }]
                    }
                ]
            });
            fs::write(&self.policy, serde_json::to_vec(&content).unwrap()).unwrap();
            fs::set_permissions(&self.policy, fs::Permissions::from_mode(0o600)).unwrap();
        }

        fn gate(&self) -> GreffeAuthorizationGate {
            GreffeAuthorizationGate::new(&self.policy, &self.audit)
        }

        fn deposit(
            &self,
            name: Option<&str>,
            instance_id: Option<&str>,
            declared_from: Option<&str>,
            action: GreffeMutationAction,
        ) -> Result<GreffeAuthorizationAttestation, GreffeAuthorizationRefusal> {
            self.gate().authorize_deposit(GreffeDepositAuthorization {
                canonical_name: name,
                canonical_instance_id: instance_id,
                declared_from,
                action,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            })
        }

        fn effect(
            &self,
            attestation: Option<&GreffeAuthorizationAttestation>,
            action: GreffeMutationAction,
        ) -> Result<GreffePrincipal, GreffeAuthorizationRefusal> {
            self.gate().authorize_effect(GreffeEffectAuthorization {
                attestation,
                action,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            })
        }

        fn audit_lines(&self) -> Vec<serde_json::Value> {
            fs::read_to_string(&self.audit)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }

        fn assert_deposit_refused_before_mutation(
            &self,
            name: Option<&str>,
            instance_id: Option<&str>,
            declared_from: Option<&str>,
            expected: GreffeAuthorizationRefusal,
        ) {
            // Cet oracle prouve le traitement du principal fourni à la garde,
            // pas l'authenticité du `Register` qui alimente les cartes daemon.
            let durable_state = self.root.join("durable-business-state");
            fs::write(&durable_state, b"unchanged").unwrap();
            let result = self.gate().authorize_deposit_then(
                GreffeDepositAuthorization {
                    canonical_name: name,
                    canonical_instance_id: instance_id,
                    declared_from,
                    action: GreffeMutationAction::Delegate,
                    issuer_scope: ISSUER_SCOPE,
                    request_id: "request-1",
                    request_issued_at: NOW - 1,
                    canonical_request: CANONICAL_REQUEST,
                    observed_at: NOW,
                },
                |_| fs::write(&durable_state, b"mutated").unwrap(),
            );

            assert_eq!(
                fs::read_to_string(&durable_state).unwrap(),
                "unchanged",
                "la mutation durable doit rester en aval de la garde"
            );
            assert_eq!(result.unwrap_err(), expected);
            assert_eq!(
                expected.public_reason(),
                GREFFE_AUTHORIZATION_PUBLIC_REFUSAL
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn hmac_sha256_est_confronte_a_un_vecteur_rfc4231_litteral() {
        let key = [0x0b_u8; 20];
        assert_eq!(
            hex(&hmac_sha256(&key, b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn agent_autorise_est_atteste_au_depot_et_reverifie_avant_effet() {
        let fixture = Fixture::new("nominal");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let principal = fixture
            .effect(Some(&attestation), GreffeMutationAction::Delegate)
            .unwrap();

        assert_eq!(principal.name, "agent-autorise");
        assert_eq!(principal.instance_id, "instance-autorisee");
        assert_eq!(attestation.request_id, "request-1");
        assert_eq!(
            attestation.canonical_request_sha256,
            CANONICAL_REQUEST_SHA256
        );
        assert_eq!(attestation.signature.len(), 64);
        let audit = fixture.audit_lines();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0]["stage"], "deposit");
        assert_eq!(audit[1]["stage"], "effect");
        assert_eq!(audit[0]["principal"]["name"], "agent-autorise");
        assert_eq!(audit[0]["action"], "delegate");
        assert_eq!(audit[0]["request_id"], "request-1");
        assert_eq!(audit[0]["allowed"], true);
    }

    #[test]
    fn exemple_de_politique_adapte_autorise_exactement_les_trois_mutations() {
        let fixture = Fixture::new("documented-policy");
        let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/greffe-authorization.example.json");
        let mut example: serde_json::Value =
            serde_json::from_slice(&fs::read(example_path).unwrap()).unwrap();
        example["attestation_key"] = json!(KEY);
        example["principals"][0]["principal"] = json!("agent-autorise");
        example["principals"][0]["instances"][0]["instance_id"] = json!("instance-autorisee");
        example["principals"][0]["instances"][0]["expires_at"] = json!(NOW + 300);
        fs::write(&fixture.policy, serde_json::to_vec(&example).unwrap()).unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();

        for action in GreffeMutationAction::ALL {
            let attestation = fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    action,
                )
                .unwrap();
            assert_eq!(attestation.action, action);
        }
        assert_eq!(fixture.audit_lines().len(), GreffeMutationAction::ALL.len());
    }

    #[test]
    fn nom_canonique_absent_est_refuse_avant_mutation_durable() {
        Fixture::new("missing-name").assert_deposit_refused_before_mutation(
            None,
            Some("instance-autorisee"),
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::PrincipalNameMissing,
        );
    }

    #[test]
    fn instance_canonique_absente_est_refusee_avant_mutation_durable() {
        Fixture::new("missing-instance").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            None,
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::PrincipalInstanceMissing,
        );
    }

    #[test]
    fn from_forge_est_refuse_avant_mutation_durable() {
        Fixture::new("forged-from").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            Some("instance-autorisee"),
            Some("agent-forge"),
            GreffeAuthorizationRefusal::DeclaredPrincipalMismatch,
        );
    }

    #[test]
    fn connexion_cli_send_est_refusee_avant_mutation_durable() {
        Fixture::new("cli-send").assert_deposit_refused_before_mutation(
            Some("cli-send-123"),
            Some("instance-autorisee"),
            Some("cli-send-123"),
            GreffeAuthorizationRefusal::EphemeralCliForbidden,
        );
    }

    #[test]
    fn instance_remplacee_est_refusee_avant_mutation_durable() {
        Fixture::new("replaced-instance").assert_deposit_refused_before_mutation(
            Some("agent-autorise"),
            Some("instance-remplacee"),
            Some("agent-autorise"),
            GreffeAuthorizationRefusal::InstanceUnknown,
        );
    }

    #[test]
    fn defaut_deny_couvre_principal_et_action_non_declares() {
        let fixture = Fixture::new("default-deny");
        let unknown = fixture
            .deposit(
                Some("autre-agent"),
                Some("autre-instance"),
                Some("autre-agent"),
                GreffeMutationAction::Delegate,
            )
            .unwrap_err();
        assert_eq!(unknown, GreffeAuthorizationRefusal::PrincipalUnknown);

        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        policy["principals"][0]["actions"] = json!(["delegate"]);
        fs::write(&fixture.policy, serde_json::to_vec(&policy).unwrap()).unwrap();
        let denied = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::ObjectiveClose,
            )
            .unwrap_err();
        assert_eq!(denied, GreffeAuthorizationRefusal::ActionDenied);
    }

    #[test]
    fn expiration_et_revocation_serveur_sont_reverifiees_avant_effet() {
        let fixture = Fixture::new("server-state");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::RegistreAdd,
            )
            .unwrap();

        fixture.write_policy(true, NOW + 300);
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::RegistreAdd)
                .unwrap_err(),
            GreffeAuthorizationRefusal::GrantRevoked
        );
        fixture.write_policy(false, NOW);
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::RegistreAdd)
                .unwrap_err(),
            GreffeAuthorizationRefusal::GrantExpired
        );
    }

    #[test]
    fn attestation_absente_fausse_ou_rejouee_sur_une_autre_action_est_refusee() {
        let fixture = Fixture::new("attestation-negatives");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        assert_eq!(
            fixture
                .effect(None, GreffeMutationAction::Delegate)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMissing
        );

        let mut forged = attestation.clone();
        forged.signature = "signature-disjointe".to_string();
        assert_eq!(
            fixture
                .effect(Some(&forged), GreffeMutationAction::Delegate)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationInvalid
        );
        assert_eq!(
            fixture
                .effect(Some(&attestation), GreffeMutationAction::ObjectiveClose)
                .unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn issuer_scope_copie_est_refuse_avant_mutation_durable() {
        let fixture = Fixture::new("copied-issuer-scope");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let durable_state = fixture.root.join("durable-effect-state");
        fs::write(&durable_state, b"unchanged").unwrap();

        let result = fixture.gate().authorize_effect_then(
            GreffeEffectAuthorization {
                attestation: Some(&attestation),
                action: GreffeMutationAction::Delegate,
                issuer_scope: "026_scope_ffffffffffffffffffffffffffffffff",
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: CANONICAL_REQUEST,
                observed_at: NOW,
            },
            |_| fs::write(&durable_state, b"mutated").unwrap(),
        );

        assert_eq!(
            fs::read_to_string(&durable_state).unwrap(),
            "unchanged",
            "une attestation ne s'étend jamais à une seconde portée idempotente"
        );
        assert_eq!(
            result.unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn octet_canonique_substitue_est_refuse_avant_mutation_durable() {
        let fixture = Fixture::new("canonical-request-substitution");
        let attestation = fixture
            .deposit(
                Some("agent-autorise"),
                Some("instance-autorisee"),
                Some("agent-autorise"),
                GreffeMutationAction::Delegate,
            )
            .unwrap();
        let mut substituted = CANONICAL_REQUEST.to_vec();
        let changed = substituted
            .iter()
            .position(|byte| *byte == b'A')
            .expect("le témoin contient l'octet discriminant A");
        substituted[changed] = b'B';
        assert_eq!(
            CANONICAL_REQUEST
                .iter()
                .zip(&substituted)
                .filter(|(left, right)| left != right)
                .count(),
            1,
            "le témoin substitue exactement un octet canonique"
        );
        let durable_state = fixture.root.join("canonical-request-effect-state");
        fs::write(&durable_state, b"unchanged").unwrap();

        let result = fixture.gate().authorize_effect_then(
            GreffeEffectAuthorization {
                attestation: Some(&attestation),
                action: GreffeMutationAction::Delegate,
                issuer_scope: ISSUER_SCOPE,
                request_id: "request-1",
                request_issued_at: NOW - 1,
                canonical_request: &substituted,
                observed_at: NOW,
            },
            |_| fs::write(&durable_state, b"mutated").unwrap(),
        );

        assert_eq!(
            fs::read_to_string(&durable_state).unwrap(),
            "unchanged",
            "la substitution des octets relus doit être refusée avant l'effet"
        );
        assert_eq!(
            result.unwrap_err(),
            GreffeAuthorizationRefusal::AttestationMismatch
        );
    }

    #[test]
    fn politique_absente_invalide_ou_non_privee_refuse_et_journalise() {
        let fixture = Fixture::new("policy-negatives");
        fs::remove_file(&fixture.policy).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyUnavailable
        );

        fixture.write_policy(false, NOW + 300);
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyInvalid
        );
        let audit = fixture.audit_lines();
        assert_eq!(audit.len(), 2);
        assert_eq!(audit[0]["allowed"], false);
        assert_eq!(audit[0]["reason"], "policy_unavailable");
        assert_eq!(audit[1]["allowed"], false);
        assert_eq!(audit[1]["reason"], "policy_invalid");
    }

    #[test]
    fn politique_et_audit_refusent_les_liens_symboliques() {
        let fixture = Fixture::new("nofollow");
        let real_policy = fixture.root.join("real-policy.json");
        fs::rename(&fixture.policy, &real_policy).unwrap();
        symlink(&real_policy, &fixture.policy).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::PolicyUnavailable
        );

        fs::remove_file(&fixture.policy).unwrap();
        fs::rename(&real_policy, &fixture.policy).unwrap();
        let target = fixture.root.join("real-audit.jsonl");
        fs::write(&target, b"").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        fs::remove_file(&fixture.audit).unwrap();
        symlink(&target, &fixture.audit).unwrap();
        assert_eq!(
            fixture
                .deposit(
                    Some("agent-autorise"),
                    Some("instance-autorisee"),
                    Some("agent-autorise"),
                    GreffeMutationAction::Delegate,
                )
                .unwrap_err(),
            GreffeAuthorizationRefusal::AuditUnavailable
        );
        assert!(fs::read(&target).unwrap().is_empty());
    }
}
