//! Frontière HTTP volontairement étroite pour le décideur d'admission 163.
//!
//! Aucune admission ne dépend de ce module tant que la politique et le puits
//! durable ne l'ont pas explicitement choisi. Cette isolation conserve le
//! client HTTP historique du daemon et rend visibles les limites d'un appel
//! bloqué sans introduire de runtime, file ou fournisseur supplémentaire.

use crate::registry::DecisionService;
use bridget_transport::protocol::{is_auto_candidate_id, is_routing_candidate_id};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

pub const CALLER_DEADLINE: Duration = Duration::from_secs(2);
const MAX_RESPONSE_BODY_BYTES: usize = 8 * 1024;
const MAX_RESPONSE_HEADERS_BYTES: usize = 8 * 1024;
const MAX_STATUS_LINE_BYTES: usize = 1024;
const MAX_DECISION_JSON_BYTES: usize = 4096;
const MAX_LAYA_STATE_BYTES: usize = 1536;
const MAX_LAYA_INSTRUCTION_BYTES: usize = 128;
const MAX_LAYA_DESCRIPTION_BYTES: usize = 32;
const MAX_LAYA_OUTPUT_BYTES: u64 = 1024;
const MAX_LAYA_HEAD_OUTPUT_BYTES: u64 = 512;
const MAX_LAYA_ROUTING_MODEL_BYTES: usize = 1024;
const MAX_HEALTH_CACHE_ENTRIES: usize = 16;
pub const MAX_ROUTING_DECISION_BYTES: usize = 64 * 1024;
pub const MAX_ROUTING_CANDIDATES: usize = 128;
const MAX_ROUTING_REASON_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionServiceError {
    Disabled,
    UnsafeRuntimeConfiguration,
    EndpointInvalid,
    ProtocolUnsupported,
    RequestInvalid,
    CapacityOccupied,
    CallerDeadline,
    Transport,
    HttpStatus,
    ResponseTooLarge,
    ResponseInvalid,
    CapacityUnavailable,
    HealthUnavailable { reason: &'static str },
}

impl DecisionServiceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Disabled => "decision_service_disabled",
            Self::UnsafeRuntimeConfiguration => "decision_service_unsafe_runtime",
            Self::EndpointInvalid => "decision_service_endpoint_invalid",
            Self::ProtocolUnsupported => "decision_service_protocol_unsupported",
            Self::RequestInvalid => "decision_service_request_invalid",
            Self::CapacityOccupied => "decision_service_capacity_occupied",
            Self::CallerDeadline => "decision_service_caller_deadline",
            Self::Transport => "decision_service_transport",
            Self::HttpStatus => "decision_service_http_status",
            Self::ResponseTooLarge => "decision_service_response_too_large",
            Self::ResponseInvalid => "decision_service_response_invalid",
            Self::CapacityUnavailable => "decision_capacity_unavailable",
            Self::HealthUnavailable { reason } => reason,
        }
    }
}

/// Une deadline de demandeur ne prétend jamais interrompre DNS/TLS ou une
/// lecture distante. Le permis reste donc détenu par le thread jusqu'à son
/// retour réel, même après l'expiration vue par le demandeur.
#[derive(Debug, Clone, Copy)]
pub struct CallerDeadline(Instant);

impl CallerDeadline {
    pub fn new() -> Self {
        Self(Instant::now() + CALLER_DEADLINE)
    }

    /// Échéance absolue de la chaîne168. Une échéance expirée refuse avant I/O.
    pub fn until(deadline: Instant) -> Self {
        Self(deadline)
    }

    fn remaining(self) -> Result<Duration, DecisionServiceError> {
        let remaining = self.0.saturating_duration_since(Instant::now());
        (!remaining.is_zero())
            .then_some(remaining)
            .ok_or(DecisionServiceError::CallerDeadline)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Available,
    Unavailable { reason: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionCandidate {
    pub candidate_id: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionBrief {
    pub objective: String,
    pub summary: String,
}

/// Charge normalisée, sans prompt de projet, chemin, credential, droit ni
/// définition fournisseur. L'adaptateur ne transporte que des IDs opaques.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayaDecisionRequest {
    pub admission_id: String,
    pub policy_revision: u64,
    pub brief: DecisionBrief,
    pub urgent: bool,
    pub deliverable: Option<String>,
    pub criteria: Vec<String>,
    pub state: String,
    pub candidates: Vec<DecisionCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionChoice {
    pub candidate_id: String,
    pub reason: Option<String>,
}

/// Faits publics et agrégés d'une machine ou d'une connexion. Cette frontière
/// exclut chemins, environnements et identités de compte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecisionFact {
    pub id: String,
    pub summary: String,
    pub observed_at: Option<i64>,
}

/// Candidat168 complet : les efforts restent portés par le modèle et ne
/// multiplient donc jamais les candidats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecisionCandidate {
    pub candidate_id: String,
    pub model: String,
    pub engine: String,
    pub machine_id: String,
    pub connection_id: String,
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
    pub score: i32,
    pub facts: Vec<String>,
}

/// Charge figée commune au service et au secours Claude. `brief` est une
/// donnée inerte bornée, jamais une commande ou une configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecisionRequest {
    pub admission_id: String,
    pub policy_revision: u64,
    pub brief: String,
    pub workload: String,
    pub constraints: Vec<String>,
    pub machines: Vec<RoutingDecisionFact>,
    pub connections: Vec<RoutingDecisionFact>,
    pub candidates: Vec<RoutingDecisionCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecisionChoice {
    pub candidate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// O(n log n + b), n<=128 candidats ; b<=64 Kio de charge encodée.
pub fn validate_routing_request(
    request: &RoutingDecisionRequest,
) -> Result<(), DecisionServiceError> {
    if !printable(&request.admission_id, 256)
        || request.policy_revision == 0
        || !printable(&request.brief, 4096)
        || !matches!(request.workload.as_str(), "general" | "compilation")
        || request.constraints.len() > 16
        || request.machines.len() > MAX_ROUTING_CANDIDATES
        || request.connections.len() > MAX_ROUTING_CANDIDATES
        || request.candidates.is_empty()
        || request.candidates.len() > MAX_ROUTING_CANDIDATES
    {
        return Err(DecisionServiceError::RequestInvalid);
    }
    let mut ids = BTreeSet::new();
    if request.candidates.iter().any(|candidate| {
        !is_routing_candidate_id(&candidate.candidate_id)
            || !ids.insert(&candidate.candidate_id)
            || !printable(&candidate.model, 32)
            || !printable(&candidate.engine, 64)
            || !printable(&candidate.machine_id, 256)
            || !printable(&candidate.connection_id, 256)
            || candidate.supported_efforts.len() > 16
            || candidate
                .supported_efforts
                .iter()
                .any(|effort| !printable(effort, 64))
            || (candidate.default_effort.is_none() && !candidate.supported_efforts.is_empty())
            || candidate.default_effort.as_deref().is_some_and(|effort| {
                !candidate
                    .supported_efforts
                    .iter()
                    .any(|supported| supported == effort)
            })
            || candidate.facts.len() > 32
            || candidate.facts.iter().any(|fact| !printable(fact, 256))
    }) {
        return Err(DecisionServiceError::RequestInvalid);
    }
    if request
        .machines
        .iter()
        .chain(&request.connections)
        .any(|fact| !printable(&fact.id, 256) || !printable(&fact.summary, 1024))
        || request
            .constraints
            .iter()
            .any(|constraint| !printable(constraint, 256))
    {
        return Err(DecisionServiceError::RequestInvalid);
    }
    let encoded = serde_json::to_vec(request).map_err(|_| DecisionServiceError::RequestInvalid)?;
    (encoded.len() <= MAX_ROUTING_DECISION_BYTES)
        .then_some(())
        .ok_or(DecisionServiceError::CapacityUnavailable)
}

/// O(n log n + b), même validation fermée et ensemble borné que la requête.
pub fn validate_routing_choice(
    candidate_id: &str,
    effort: Option<&str>,
    reason: Option<&str>,
    request: &RoutingDecisionRequest,
) -> Result<RoutingDecisionChoice, DecisionServiceError> {
    validate_routing_request(request)?;
    let candidate = request
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_id == candidate_id)
        .ok_or(DecisionServiceError::ResponseInvalid)?;
    if !is_routing_candidate_id(candidate_id)
        || effort.is_some_and(|value| {
            !candidate
                .supported_efforts
                .iter()
                .any(|supported| supported == value)
        })
        || reason.is_some_and(|value| !printable(value, MAX_ROUTING_REASON_BYTES))
    {
        return Err(DecisionServiceError::ResponseInvalid);
    }
    Ok(RoutingDecisionChoice {
        candidate_id: candidate_id.to_owned(),
        effort: effort
            .map(ToOwned::to_owned)
            .or_else(|| candidate.default_effort.clone()),
        reason: reason.map(ToOwned::to_owned),
    })
}

#[derive(Default)]
struct DecisionServiceState {
    in_flight: bool,
    routing_in_flight: bool,
    health: BTreeMap<String, HealthCacheEntry>,
}

struct HealthCacheEntry {
    status: HealthStatus,
    expires_at: Instant,
}

struct NetworkPermit {
    state: Arc<Mutex<DecisionServiceState>>,
}

/// Permis de la chaîne168, conservé par l'orchestrateur depuis le service
/// jusqu'au drain confirmé du secours local. Il ne remplace pas le permis HTTP
/// historique, qui couvre toujours DNS/TLS/lecture jusqu'au retour réel.
pub struct RoutingDecisionPermit {
    state: Arc<Mutex<DecisionServiceState>>,
}

impl Drop for RoutingDecisionPermit {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.routing_in_flight = false;
    }
}

impl Drop for NetworkPermit {
    fn drop(&mut self) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.in_flight = false;
    }
}

#[derive(Clone, Default)]
pub struct DecisionServiceClient {
    state: Arc<Mutex<DecisionServiceState>>,
}

impl DecisionServiceClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn acquire_routing_permit(&self) -> Result<RoutingDecisionPermit, DecisionServiceError> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.routing_in_flight {
            return Err(DecisionServiceError::CapacityOccupied);
        }
        state.routing_in_flight = true;
        Ok(RoutingDecisionPermit {
            state: Arc::clone(&self.state),
        })
    }

    /// Adaptateur168 vers le contrat Laya163. La charge entière est encodée
    /// dans son état réellement lu ; si elle ne tient pas, ce niveau est
    /// explicitement indisponible plutôt que réduit à huit profils partiels.
    /// Santé/cache/cooldown163 sont consultés seulement après cet encodage
    /// complet. Santé et POST partagent l'échéance absolue du demandeur.
    pub fn decide_routing(
        &self,
        service_id: &str,
        service: &DecisionService,
        request: &RoutingDecisionRequest,
        deadline: CallerDeadline,
    ) -> Result<RoutingDecisionChoice, DecisionServiceError> {
        validate_routing_request(request)?;
        if request.candidates.len() > 8 {
            return Err(DecisionServiceError::CapacityUnavailable);
        }
        let state =
            serde_json::to_string(request).map_err(|_| DecisionServiceError::RequestInvalid)?;
        if !printable(&state, MAX_LAYA_STATE_BYTES) {
            return Err(DecisionServiceError::CapacityUnavailable);
        }
        validate_service(service)?;
        let payload = laya_routing_request_payload(service, request, state)?;
        deadline.remaining()?;
        match self.health(service_id, service, deadline)? {
            HealthStatus::Available => {}
            HealthStatus::Unavailable { reason } => {
                return Err(DecisionServiceError::HealthUnavailable { reason });
            }
        }
        let response = self.send(service, "POST", &service.endpoint, Some(payload), deadline)?;
        if !(200..=299).contains(&response.status) {
            return Err(DecisionServiceError::HttpStatus);
        }
        parse_laya_routing_choice(&response.body, request, &service.model)
    }

    /// Santé demandée seulement par une admission qui a déjà plus d'un
    /// candidat. Il n'y a ni polling, ni découverte, ni démarrage implicite.
    pub fn health(
        &self,
        service_id: &str,
        service: &DecisionService,
        deadline: CallerDeadline,
    ) -> Result<HealthStatus, DecisionServiceError> {
        validate_service(service)?;
        let key = service_cache_key(service_id, service)?;
        if let Some(status) = self.cached_health(&key) {
            return Ok(status);
        }
        let url = health_url(&service.endpoint)?;
        let observation = match self.send(service, "GET", &url, None, deadline) {
            Ok(response) if (200..=299).contains(&response.status) => {
                match parse_laya_health(&response.body) {
                    Ok(()) => HealthStatus::Available,
                    Err(error) => HealthStatus::Unavailable {
                        reason: error.code(),
                    },
                }
            }
            Ok(_) => HealthStatus::Unavailable {
                reason: DecisionServiceError::HttpStatus.code(),
            },
            // Le délai du demandeur n'annule jamais DNS/TLS/lecture. Il est
            // néanmoins une santé négative cachée : après le retour réel du
            // thread, une admission ne contourne pas retry_after_seconds par
            // une nouvelle sonde immédiate.
            Err(DecisionServiceError::CallerDeadline) => HealthStatus::Unavailable {
                reason: DecisionServiceError::CallerDeadline.code(),
            },
            Err(error @ DecisionServiceError::CapacityOccupied)
            | Err(error @ DecisionServiceError::UnsafeRuntimeConfiguration)
            | Err(error @ DecisionServiceError::EndpointInvalid)
            | Err(error @ DecisionServiceError::ProtocolUnsupported) => return Err(error),
            Err(error) => HealthStatus::Unavailable {
                reason: error.code(),
            },
        };
        self.store_health(key, service, observation.clone());
        Ok(observation)
    }

    /// Décision unique, sans retry. Le caller doit partager la même deadline
    /// avec `health`, afin que santé + décision ne dépassent jamais deux secondes.
    pub fn decide(
        &self,
        service: &DecisionService,
        request: &LayaDecisionRequest,
        deadline: CallerDeadline,
    ) -> Result<DecisionChoice, DecisionServiceError> {
        validate_service(service)?;
        let payload = laya_request_payload(service, request)?;
        let response = self.send(service, "POST", &service.endpoint, Some(payload), deadline)?;
        if !(200..=299).contains(&response.status) {
            return Err(DecisionServiceError::HttpStatus);
        }
        parse_laya_choice(&response.body, &request.candidates, &service.model)
    }

    fn cached_health(&self, key: &str) -> Option<HealthStatus> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state
            .health
            .get(key)
            .filter(|entry| entry.expires_at > Instant::now())
            .map(|entry| entry.status.clone())
    }

    fn store_health(&self, key: String, service: &DecisionService, status: HealthStatus) {
        let seconds = match status {
            HealthStatus::Available => service.health_ttl_seconds,
            HealthStatus::Unavailable { .. } => service.retry_after_seconds,
        };
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if !state.health.contains_key(&key) && state.health.len() >= MAX_HEALTH_CACHE_ENTRIES {
            // Cache d'observations seulement : une éviction déterministe ne
            // touche aucune décision ni reçu durable.
            if let Some(oldest) = state.health.keys().next().cloned() {
                state.health.remove(&oldest);
            }
        }
        state.health.insert(
            key,
            HealthCacheEntry {
                status,
                expires_at: Instant::now() + Duration::from_secs(seconds),
            },
        );
    }

    fn send(
        &self,
        service: &DecisionService,
        method: &'static str,
        url: &str,
        body: Option<Vec<u8>>,
        deadline: CallerDeadline,
    ) -> Result<HttpReply, DecisionServiceError> {
        // Présente, vide ou invalide : minreq la lirait globalement. Elle ne
        // désactive donc que ce service, sans muter l'environnement du daemon.
        if std::env::var_os("MINREQ_TIMEOUT").is_some() {
            return Err(DecisionServiceError::UnsafeRuntimeConfiguration);
        }
        let permit = {
            let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
            if state.in_flight {
                return Err(DecisionServiceError::CapacityOccupied);
            }
            deadline.remaining()?;
            state.in_flight = true;
            NetworkPermit {
                state: Arc::clone(&self.state),
            }
        };
        let service = service.clone();
        let url = url.to_string();
        let (sender, receiver) = mpsc::sync_channel(1);
        if std::thread::Builder::new()
            .name("bridget-decision-http".into())
            .spawn(move || {
                let result = send_bounded(&service, method, &url, body);
                // Le retour réseau est le seul point qui libère le permis.
                // Le faire avant la publication évite qu'un appel qui reçoit
                // la réponse observe encore artificiellement `in_flight`.
                drop(permit);
                let _ = sender.send(result);
            })
            .is_err()
        {
            return Err(DecisionServiceError::Transport);
        }
        receiver
            .recv_timeout(deadline.remaining()?)
            .map_err(|_| DecisionServiceError::CallerDeadline)?
    }
}

#[derive(Debug)]
struct HttpReply {
    status: i32,
    body: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaHealth {
    status: String,
    // Ces champs décrivent seulement l'instance Laya qui répond. Ils ne
    // participent ni au choix, ni au cache, ni à l'autorisation : `status`
    // reste le seul verdict de santé. Les connaître explicitement conserve le
    // schéma fermé pour toute extension non revue du serveur.
    #[serde(default, rename = "loaded")]
    _loaded: Option<Vec<String>>,
    #[serde(default, rename = "revisions")]
    _revisions: Option<BTreeMap<String, String>>,
    #[serde(default, rename = "device")]
    _device: Option<String>,
}

fn parse_laya_health(body: &[u8]) -> Result<(), DecisionServiceError> {
    let health: LayaHealth =
        serde_json::from_slice(body).map_err(|_| DecisionServiceError::ResponseInvalid)?;
    (health.status == "ok")
        .then_some(())
        .ok_or(DecisionServiceError::ResponseInvalid)
}

fn validate_service(service: &DecisionService) -> Result<(), DecisionServiceError> {
    if !service.enabled {
        return Err(DecisionServiceError::Disabled);
    }
    if service.protocol != "laya_systemone_v1" {
        return Err(DecisionServiceError::ProtocolUnsupported);
    }
    if !decision_endpoint_allowed(&service.endpoint) {
        return Err(DecisionServiceError::EndpointInvalid);
    }
    if std::env::var_os("MINREQ_TIMEOUT").is_some() {
        return Err(DecisionServiceError::UnsafeRuntimeConfiguration);
    }
    Ok(())
}

/// Diagnostic de démarrage sans état persistant : la configuration globale de
/// minreq ne doit désactiver que le décideur, jamais le daemon ni Delegate.
pub(crate) fn startup_diagnostic() -> Option<&'static str> {
    std::env::var_os("MINREQ_TIMEOUT")
        .is_some()
        .then_some("service de décision désactivé : MINREQ_TIMEOUT est présent")
}

fn decision_endpoint_allowed(endpoint: &str) -> bool {
    if endpoint.trim() != endpoint
        || endpoint.is_empty()
        || endpoint.len() > 2048
        || endpoint.chars().any(char::is_control)
        || endpoint.contains('@')
        || endpoint.contains('#')
    {
        return false;
    }
    if endpoint.starts_with("https://") {
        return endpoint[8..]
            .split(['/', '?'])
            .next()
            .is_some_and(|authority| !authority.is_empty());
    }
    let Some(rest) = endpoint.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    matches!(authority, "localhost" | "127.0.0.1" | "[::1]")
        || authority
            .strip_prefix("localhost:")
            .or_else(|| authority.strip_prefix("127.0.0.1:"))
            .or_else(|| authority.strip_prefix("[::1]:"))
            .is_some_and(|port| port.parse::<u16>().is_ok_and(|port| port != 0))
}

fn health_url(endpoint: &str) -> Result<String, DecisionServiceError> {
    let (scheme, rest) = endpoint
        .split_once("://")
        .ok_or(DecisionServiceError::EndpointInvalid)?;
    let authority = rest
        .split(['/', '?'])
        .next()
        .filter(|authority| !authority.is_empty())
        .ok_or(DecisionServiceError::EndpointInvalid)?;
    Ok(format!("{scheme}://{authority}/health"))
}

fn service_cache_key(
    service_id: &str,
    service: &DecisionService,
) -> Result<String, DecisionServiceError> {
    if service_id.is_empty() || service_id.len() > 256 || service_id.chars().any(char::is_control) {
        return Err(DecisionServiceError::RequestInvalid);
    }
    let encoded = serde_json::to_vec(service).map_err(|_| DecisionServiceError::RequestInvalid)?;
    Ok(format!("{service_id}:{:x}", Sha256::digest(encoded)))
}

fn send_bounded(
    _service: &DecisionService,
    method: &str,
    url: &str,
    body: Option<Vec<u8>>,
) -> Result<HttpReply, DecisionServiceError> {
    let mut request = match method {
        "GET" => minreq::get(url),
        "POST" => minreq::post(url),
        _ => return Err(DecisionServiceError::RequestInvalid),
    }
    .with_header("Accept", "application/json")
    .with_max_redirects(0)
    .with_follow_redirects(false)
    .with_max_headers_size(MAX_RESPONSE_HEADERS_BYTES)
    .with_max_status_line_length(MAX_STATUS_LINE_BYTES);
    if let Some(body) = body {
        request = request
            .with_header("Content-Type", "application/json")
            .with_body(body);
    }
    let mut response = request
        .send_lazy()
        .map_err(|_| DecisionServiceError::Transport)?;
    let status = response.status_code;
    let mut body = Vec::with_capacity(MAX_RESPONSE_BODY_BYTES + 1);
    let mut limited = std::io::Read::take(&mut response, (MAX_RESPONSE_BODY_BYTES + 1) as u64);
    std::io::Read::read_to_end(&mut limited, &mut body)
        .map_err(|_| DecisionServiceError::Transport)?;
    if body.len() > MAX_RESPONSE_BODY_BYTES {
        return Err(DecisionServiceError::ResponseTooLarge);
    }
    Ok(HttpReply { status, body })
}

fn printable(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && !value.chars().any(bridget_core::is_disallowed_control)
}

fn laya_request_payload(
    service: &DecisionService,
    request: &LayaDecisionRequest,
) -> Result<Vec<u8>, DecisionServiceError> {
    if !printable(&request.admission_id, 256)
        || request.policy_revision == 0
        || !printable(&request.brief.objective, MAX_LAYA_INSTRUCTION_BYTES)
        || !printable(&request.brief.summary, MAX_LAYA_INSTRUCTION_BYTES)
        || request
            .deliverable
            .as_deref()
            .is_some_and(|value| !printable(value, MAX_LAYA_INSTRUCTION_BYTES))
        || request.criteria.len() > 8
        || request
            .criteria
            .iter()
            .any(|criterion| !printable(criterion, MAX_LAYA_INSTRUCTION_BYTES))
        || !printable(&request.state, MAX_LAYA_STATE_BYTES)
        || request.candidates.is_empty()
        || request.candidates.len() > 8
    {
        return Err(DecisionServiceError::RequestInvalid);
    }
    let mut ids = BTreeSet::new();
    if request.candidates.iter().any(|candidate| {
        !is_auto_candidate_id(&candidate.candidate_id)
            || !printable(&candidate.description, MAX_LAYA_DESCRIPTION_BYTES)
            || !ids.insert(&candidate.candidate_id)
    }) {
        return Err(DecisionServiceError::RequestInvalid);
    }
    if !printable(&service.model, MAX_LAYA_ROUTING_MODEL_BYTES) {
        return Err(DecisionServiceError::RequestInvalid);
    }
    // Laya ne lit pas les champs contextuels racine. Les informations métier
    // explicites sont donc transportées dans les deux champs réellement lus,
    // avec refus plutôt que troncature si la borne est dépassée.
    let instructions = format!(
        "Choisir un candidat autorisé. Critères: {}",
        request.criteria.join("; ")
    );
    let state = format!(
        "admission_id={}; policy_revision={}; objective={}; summary={}; urgent={}; deliverable={}; state={}",
        request.admission_id,
        request.policy_revision,
        request.brief.objective,
        request.brief.summary,
        request.urgent,
        request.deliverable.as_deref().unwrap_or("none"),
        request.state,
    );
    if !printable(&instructions, MAX_LAYA_INSTRUCTION_BYTES)
        || !printable(&state, MAX_LAYA_STATE_BYTES)
    {
        return Err(DecisionServiceError::RequestInvalid);
    }
    let criteria = request
        .candidates
        .iter()
        .map(|candidate| {
            (
                candidate.candidate_id.clone(),
                candidate.description.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let payload = serde_json::json!({
        "model": service.model,
        "max_len": MAX_LAYA_OUTPUT_BYTES,
        "head_max_len": MAX_LAYA_HEAD_OUTPUT_BYTES,
        "state": state,
        "questions": {
            "candidate": {
                "type": "choice",
                "instructions": instructions,
                "criteria": criteria,
            }
        },
    });
    let encoded = serde_json::to_vec(&payload).map_err(|_| DecisionServiceError::RequestInvalid)?;
    (encoded.len() <= MAX_DECISION_JSON_BYTES)
        .then_some(encoded)
        .ok_or(DecisionServiceError::RequestInvalid)
}

/// Variante168 du wire Laya : elle conserve les bornes du serveur mais valide
/// ses IDs v2 séparément, sans jamais élargir `is_auto_candidate_id` (163).
fn laya_routing_request_payload(
    service: &DecisionService,
    request: &RoutingDecisionRequest,
    state: String,
) -> Result<Vec<u8>, DecisionServiceError> {
    if !printable(&service.model, MAX_LAYA_ROUTING_MODEL_BYTES)
        || !printable(&state, MAX_LAYA_STATE_BYTES)
    {
        return Err(DecisionServiceError::CapacityUnavailable);
    }
    let criteria = request
        .candidates
        .iter()
        .map(|candidate| {
            (
                candidate.candidate_id.clone(),
                candidate.candidate_id.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let payload = serde_json::json!({
        "model": service.model,
        "max_len": MAX_LAYA_OUTPUT_BYTES,
        "head_max_len": MAX_LAYA_HEAD_OUTPUT_BYTES,
        "state": state,
        "questions": {"candidate": {
            "type":"choice", "instructions":"Choisir un candidat autorisé.", "criteria":criteria
        }}
    });
    let encoded = serde_json::to_vec(&payload).map_err(|_| DecisionServiceError::RequestInvalid)?;
    (encoded.len() <= MAX_DECISION_JSON_BYTES)
        .then_some(encoded)
        .ok_or(DecisionServiceError::CapacityUnavailable)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaResponse {
    #[serde(default)]
    model: Option<String>,
    answers: LayaAnswers,
    routing: LayaRouting,
    usage: LayaUsage,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaAnswers {
    candidate: LayaCandidateAnswer,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaCandidateAnswer {
    #[serde(rename = "type")]
    kind: String,
    choice: String,
    #[serde(default, rename = "probabilities")]
    _probabilities: Option<BTreeMap<String, f64>>,
    #[serde(default, rename = "confidence")]
    _confidence: Option<f64>,
    #[serde(default, rename = "answer_confidence")]
    _answer_confidence: Option<f64>,
    #[serde(default, rename = "action")]
    _action: Option<LayaAction>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaAction {
    #[serde(rename = "act_probability")]
    _act_probability: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaRouting {
    model: String,
    #[serde(default, rename = "repo")]
    _repo: Option<String>,
    #[serde(default, rename = "reason")]
    _reason: Option<String>,
    #[serde(default, rename = "detection")]
    _detection: Option<serde_json::Value>,
    #[serde(default, rename = "workflow")]
    _workflow: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayaUsage {
    input_tokens: u64,
    #[serde(default)]
    output_tokens: Option<u64>,
}

fn parse_laya_choice(
    response: &[u8],
    candidates: &[DecisionCandidate],
    expected_model: &str,
) -> Result<DecisionChoice, DecisionServiceError> {
    let response: LayaResponse =
        serde_json::from_slice(response).map_err(|_| DecisionServiceError::ResponseInvalid)?;
    if response
        .model
        .as_deref()
        .is_some_and(|model| !printable(model, MAX_LAYA_ROUTING_MODEL_BYTES))
        || !printable(&response.routing.model, MAX_LAYA_ROUTING_MODEL_BYTES)
        || response.routing.model != expected_model
        || !(1..MAX_LAYA_OUTPUT_BYTES).contains(&response.usage.input_tokens)
        || response
            .usage
            .output_tokens
            .is_some_and(|tokens| tokens > MAX_LAYA_OUTPUT_BYTES)
        || response.answers.candidate.kind != "choice"
        || !is_auto_candidate_id(&response.answers.candidate.choice)
        || !candidates
            .iter()
            .any(|candidate| candidate.candidate_id == response.answers.candidate.choice)
        || response
            .routing
            ._repo
            .as_deref()
            .is_some_and(|value| !printable(value, MAX_LAYA_ROUTING_MODEL_BYTES))
        || response
            .routing
            ._reason
            .as_deref()
            .is_some_and(|value| !printable(value, MAX_LAYA_ROUTING_MODEL_BYTES))
    {
        return Err(DecisionServiceError::ResponseInvalid);
    }
    Ok(DecisionChoice {
        candidate_id: response.answers.candidate.choice,
        // `routing.reason` décrit le routage du serveur, jamais le candidat.
        // Laya ne fournit pas de raison de candidat dans ce contrat : l'absence
        // est donc projetée explicitement comme `None`.
        reason: None,
    })
}

fn parse_laya_routing_choice(
    response: &[u8],
    request: &RoutingDecisionRequest,
    expected_model: &str,
) -> Result<RoutingDecisionChoice, DecisionServiceError> {
    let response: LayaResponse =
        serde_json::from_slice(response).map_err(|_| DecisionServiceError::ResponseInvalid)?;
    if response
        .model
        .as_deref()
        .is_some_and(|model| !printable(model, MAX_LAYA_ROUTING_MODEL_BYTES))
        || !printable(&response.routing.model, MAX_LAYA_ROUTING_MODEL_BYTES)
        || response.routing.model != expected_model
        || response.answers.candidate.kind != "choice"
        || !is_routing_candidate_id(&response.answers.candidate.choice)
        || !(1..MAX_LAYA_OUTPUT_BYTES).contains(&response.usage.input_tokens)
        || response
            .usage
            .output_tokens
            .is_some_and(|tokens| tokens > MAX_LAYA_OUTPUT_BYTES)
        || response
            .routing
            ._repo
            .as_deref()
            .is_some_and(|value| !printable(value, MAX_LAYA_ROUTING_MODEL_BYTES))
        || response
            .routing
            ._reason
            .as_deref()
            .is_some_and(|value| !printable(value, MAX_LAYA_ROUTING_MODEL_BYTES))
    {
        return Err(DecisionServiceError::ResponseInvalid);
    }
    validate_routing_choice(&response.answers.candidate.choice, None, None, request)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::thread;

    pub(crate) static MINREQ_TIMEOUT_LOCK: Mutex<()> = Mutex::new(());

    pub(crate) struct MinreqTimeoutRestore(Option<std::ffi::OsString>);

    impl MinreqTimeoutRestore {
        pub(crate) fn set(value: Option<&str>) -> Self {
            let previous = std::env::var_os("MINREQ_TIMEOUT");
            // Les tests vérifient le refus local sans jamais laisser cette
            // variable globale contaminer une autre exécution concurrente.
            unsafe {
                match value {
                    Some(value) => std::env::set_var("MINREQ_TIMEOUT", value),
                    None => std::env::remove_var("MINREQ_TIMEOUT"),
                }
            }
            Self(previous)
        }
    }

    impl Drop for MinreqTimeoutRestore {
        fn drop(&mut self) {
            unsafe {
                match self.0.take() {
                    Some(value) => std::env::set_var("MINREQ_TIMEOUT", value),
                    None => std::env::remove_var("MINREQ_TIMEOUT"),
                }
            }
        }
    }

    pub(crate) struct ControlledServer {
        pub(crate) endpoint: String,
        calls: Arc<AtomicUsize>,
        requests: Arc<Mutex<Vec<String>>>,
        worker: thread::JoinHandle<()>,
        listener: TcpListener,
    }

    impl ControlledServer {
        pub(crate) fn start(
            replies: Vec<(Vec<u8>, Duration)>,
            accepted: Option<mpsc::Sender<()>>,
        ) -> Self {
            Self::start_gated(replies, accepted, None)
        }

        pub(crate) fn start_gated(
            replies: Vec<(Vec<u8>, Duration)>,
            accepted: Option<mpsc::Sender<()>>,
            response_gate: Option<mpsc::Receiver<()>>,
        ) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/v1/systemone", listener.local_addr().unwrap());
            // Garder les connexions inattendues observables après la dernière
            // réponse : elles restent en file au lieu de recevoir un refus.
            let witness = listener.try_clone().unwrap();
            let calls = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&calls);
            let requests = Arc::new(Mutex::new(Vec::new()));
            let observed_requests = Arc::clone(&requests);
            let worker = thread::spawn(move || {
                for (reply, delay) in replies {
                    let (mut stream, _) = listener.accept().unwrap();
                    observed.fetch_add(1, Ordering::SeqCst);
                    if let Some(accepted) = &accepted {
                        let _ = accepted.send(());
                    }
                    // Le test inspecte le POST réel : TCP peut séparer les
                    // headers du corps. La lecture reste bornée et temporisée,
                    // sans changer les limites du client de production.
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let maximum = MAX_DECISION_JSON_BYTES + 8 * 1024;
                    let mut request = Vec::with_capacity(1024);
                    let mut chunk = [0_u8; 1024];
                    loop {
                        if let Some(headers_end) =
                            request.windows(4).position(|window| window == b"\r\n\r\n")
                        {
                            let headers = std::str::from_utf8(&request[..headers_end]).unwrap();
                            let content_length = headers
                                .split("\r\n")
                                .skip(1)
                                .find_map(|line| {
                                    let (name, value) = line.split_once(':')?;
                                    name.eq_ignore_ascii_case("content-length")
                                        .then(|| value.trim().parse::<usize>().unwrap())
                                })
                                .unwrap_or(0);
                            if headers.starts_with("GET ") {
                                assert_eq!(content_length, 0, "GET contrôlé sans corps");
                            }
                            let total = headers_end + 4 + content_length;
                            assert!(total <= maximum, "requête contrôlée trop grande");
                            if request.len() >= total {
                                request.truncate(total);
                                break;
                            }
                        }
                        assert!(request.len() < maximum, "headers contrôlés trop grands");
                        let remaining = (maximum - request.len()).min(chunk.len());
                        let bytes = stream.read(&mut chunk[..remaining]).unwrap();
                        assert_ne!(bytes, 0, "requête contrôlée incomplète");
                        request.extend_from_slice(&chunk[..bytes]);
                    }
                    let is_post = request.starts_with(b"POST ");
                    observed_requests
                        .lock()
                        .unwrap()
                        .push(String::from_utf8_lossy(&request).into_owned());
                    // T021 : aucun octet de réponse avant le CAS/recovery
                    // observé par l'appelant. Le POST est déjà lu en entier.
                    if is_post && let Some(gate) = &response_gate {
                        gate.recv_timeout(Duration::from_secs(1)).unwrap();
                    }
                    thread::sleep(delay);
                    stream.write_all(&reply).unwrap();
                    stream.flush().unwrap();
                }
            });
            Self {
                endpoint,
                calls,
                requests,
                worker,
                listener: witness,
            }
        }

        pub(crate) fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }

        pub(crate) fn join(self) {
            self.worker.join().unwrap();
        }

        pub(crate) fn join_assert_no_extra_requests(self) {
            self.worker.join().unwrap();
            self.listener.set_nonblocking(true).unwrap();
            assert_eq!(
                self.listener.accept().unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock,
                "aucune connexion supplémentaire ne doit rester en file"
            );
            // Le dernier témoin est fermé après l'observation, même en panic.
            drop(self.listener);
        }
    }

    pub(crate) fn response(status: &str, body: &[u8]) -> Vec<u8> {
        [
            format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes(),
            body.to_vec(),
        ]
        .concat()
    }

    fn service(endpoint: &str) -> DecisionService {
        DecisionService {
            enabled: true,
            machine_id: "decision-host".into(),
            protocol: "laya_systemone_v1".into(),
            model: "typed-decisions".into(),
            endpoint: endpoint.into(),
            health_ttl_seconds: 30,
            retry_after_seconds: 30,
        }
    }

    fn decision_request() -> LayaDecisionRequest {
        LayaDecisionRequest {
            admission_id: "admission-163".into(),
            policy_revision: 1,
            brief: DecisionBrief {
                objective: "Choisir un candidat".into(),
                summary: "Une sélection bornée".into(),
            },
            urgent: false,
            deliverable: None,
            criteria: vec!["droits valides".into()],
            state: "candidats validés".into(),
            candidates: vec![DecisionCandidate {
                candidate_id: "c01".into(),
                description: "local".into(),
            }],
        }
    }

    fn maximum_decision_request() -> LayaDecisionRequest {
        LayaDecisionRequest {
            admission_id: "admission-163-max8".into(),
            policy_revision: 163,
            brief: DecisionBrief {
                objective: "Choisir un candidat".into(),
                summary: "Une sélection bornée".into(),
            },
            urgent: false,
            deliverable: Some("rapport".into()),
            criteria: vec!["droits valides".into()],
            state: "synthetic_state_163".into(),
            candidates: (1..=8)
                .map(|index| DecisionCandidate {
                    candidate_id: format!("c{index:02}"),
                    description: format!("candidate-description-profile-{index:02}"),
                })
                .collect(),
        }
    }

    fn routing_request(count: usize) -> RoutingDecisionRequest {
        RoutingDecisionRequest {
            admission_id: "routing168".into(),
            policy_revision: 168,
            brief: "Choisir avec les faits fournis".into(),
            workload: "general".into(),
            constraints: vec!["machine=m".into()],
            machines: vec![RoutingDecisionFact {
                id: "m".into(),
                summary: "cpu=unknown;ram=unknown".into(),
                observed_at: None,
            }],
            connections: vec![RoutingDecisionFact {
                id: "a".into(),
                summary: "quota=unknown".into(),
                observed_at: None,
            }],
            candidates: (1..=count)
                .map(|index| RoutingDecisionCandidate {
                    candidate_id: format!("c{index:03}"),
                    model: format!("model{index}"),
                    engine: "claude".into(),
                    machine_id: "m".into(),
                    connection_id: "a".into(),
                    supported_efforts: vec!["low".into(), "high".into()],
                    default_effort: Some("low".into()),
                    score: index as i32,
                    facts: vec![],
                })
                .collect(),
        }
    }

    fn routing_laya_response(candidate_id: &str) -> serde_json::Value {
        serde_json::json!({
            "model":"laya-rl-agent",
            "answers":{"candidate":{"type":"choice","choice":candidate_id,
                "probabilities":{candidate_id:0.5},"confidence":0.0,"answer_confidence":0.5,
                "action":{"act_probability":0.5}}},
            "routing":{"model":"typed-decisions","repo":"convaiinnovations/laya/typed-decisions",
                "reason":"explicit model=typed-decisions","detection":null,"workflow":null},
            "usage":{"input_tokens":12,"output_tokens":0}
        })
    }

    #[test]
    fn spec168_laya_post_choisit_sur_l_ensemble_entier_et_garde_ses_faits() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let decision = serde_json::to_vec(&routing_laya_response("c004")).unwrap();
        let server = ControlledServer::start(
            vec![
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                (response("200 OK", &decision), Duration::ZERO),
            ],
            None,
        );
        let request = routing_request(4);
        let configured = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let choice = client
            .decide_routing("laya168", &configured, &request, CallerDeadline::new())
            .unwrap();
        assert_eq!(
            choice,
            RoutingDecisionChoice {
                candidate_id: "c004".into(),
                effort: Some("low".into()),
                reason: None
            }
        );
        assert_eq!(server.calls.load(Ordering::SeqCst), 2);
        let requests = server.requests();
        assert!(requests[0].starts_with("GET /health HTTP/1.1\r\n"));
        assert!(requests[1].starts_with("POST /v1/systemone HTTP/1.1\r\n"));
        let (_, body) = requests[1].split_once("\r\n\r\n").unwrap();
        assert!(body.len() <= MAX_DECISION_JSON_BYTES);
        let payload: serde_json::Value = serde_json::from_str(body).unwrap();
        assert_eq!(payload["model"], configured.model);
        assert_eq!(payload["max_len"], MAX_LAYA_OUTPUT_BYTES);
        assert_eq!(payload["head_max_len"], MAX_LAYA_HEAD_OUTPUT_BYTES);
        let state = payload["state"].as_str().unwrap();
        assert!(state.len() <= MAX_LAYA_STATE_BYTES);
        let sent: RoutingDecisionRequest = serde_json::from_str(state).unwrap();
        assert_eq!(sent, request, "aucun fait, candidat ou contrainte omis");
        let criteria = payload["questions"]["candidate"]["criteria"]
            .as_object()
            .unwrap();
        assert_eq!(criteria.len(), request.candidates.len());
        for candidate in &request.candidates {
            assert_eq!(criteria[&candidate.candidate_id], candidate.candidate_id);
        }
        server.join();
    }

    #[test]
    fn spec168_laya_refuse_capacite_et_deadline_avant_tout_reseau() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let configured = service(&format!(
            "http://{}/v1/systemone",
            listener.local_addr().unwrap()
        ));
        let client = DecisionServiceClient::new();
        let nine = routing_request(9);
        assert_eq!(
            client.decide_routing("laya168", &configured, &nine, CallerDeadline::new()),
            Err(DecisionServiceError::CapacityUnavailable)
        );
        let mut large_state = routing_request(1);
        large_state.brief = "x".repeat(MAX_LAYA_STATE_BYTES);
        assert!(validate_routing_request(&large_state).is_ok());
        assert_eq!(
            client.decide_routing("laya168", &configured, &large_state, CallerDeadline::new()),
            Err(DecisionServiceError::CapacityUnavailable)
        );
        let mut large_payload = routing_request(1);
        large_payload.brief = "\"".repeat(400);
        let state = serde_json::to_string(&large_payload).unwrap();
        assert!(
            state.len() <= MAX_LAYA_STATE_BYTES,
            "le refus porte sur le JSON complet"
        );
        let mut escaped_model = configured.clone();
        escaped_model.model = "\"".repeat(MAX_LAYA_ROUTING_MODEL_BYTES);
        assert_eq!(
            client.decide_routing(
                "laya168",
                &escaped_model,
                &large_payload,
                CallerDeadline::new()
            ),
            Err(DecisionServiceError::CapacityUnavailable)
        );
        assert_eq!(
            client.decide_routing(
                "laya168",
                &configured,
                &routing_request(1),
                CallerDeadline::until(Instant::now() - Duration::from_millis(1)),
            ),
            Err(DecisionServiceError::CallerDeadline)
        );
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "aucun GET ni POST ne doit avoir atteint le socket"
        );
        assert!(client.acquire_routing_permit().is_ok());
    }

    #[test]
    fn spec168_laya_default_natif_null_et_transport_legacy163_restent_distincts() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let native = serde_json::to_vec(&routing_laya_response("c001")).unwrap();
        let legacy = serde_json::to_vec(&routing_laya_response("c01")).unwrap();
        let server = ControlledServer::start(
            vec![
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                (response("200 OK", &native), Duration::ZERO),
                (response("200 OK", &legacy), Duration::ZERO),
            ],
            None,
        );
        let configured = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let mut request = routing_request(1);
        request.candidates[0].supported_efforts.clear();
        request.candidates[0].default_effort = None;
        assert_eq!(
            client
                .decide_routing("laya168", &configured, &request, CallerDeadline::new())
                .unwrap(),
            RoutingDecisionChoice {
                candidate_id: "c001".into(),
                effort: None,
                reason: None
            }
        );
        assert_eq!(
            client
                .decide(&configured, &decision_request(), CallerDeadline::new())
                .unwrap(),
            DecisionChoice {
                candidate_id: "c01".into(),
                reason: None
            }
        );
        assert_eq!(server.calls.load(Ordering::SeqCst), 3);
        assert_eq!(
            validate_routing_choice("c001", Some("high"), None, &request),
            Err(DecisionServiceError::ResponseInvalid)
        );
        assert_eq!(
            validate_routing_choice("c01", None, None, &request),
            Err(DecisionServiceError::ResponseInvalid)
        );
        let mut legacy_request = decision_request();
        legacy_request.candidates[0].candidate_id = "c001".into();
        assert_eq!(
            laya_request_payload(&configured, &legacy_request),
            Err(DecisionServiceError::RequestInvalid)
        );
        server.join();
    }

    #[test]
    fn spec168_laya_http_refuse_choix_effort_et_metadonnees_invalides() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let mut invalid = Vec::new();
        invalid.push(routing_laya_response("c128"));
        invalid.push(routing_laya_response("c01"));
        let base = routing_laya_response("c001");
        let mut value = base.clone();
        value["answers"]["candidate"]["effort"] = serde_json::json!("high");
        invalid.push(value);
        let mut value = base.clone();
        value["answers"]["candidate"]["type"] = serde_json::json!("text");
        invalid.push(value);
        let mut value = base.clone();
        value["routing"]["model"] = serde_json::json!("other-model");
        invalid.push(value);
        let mut value = base.clone();
        value["model"] = serde_json::json!("");
        invalid.push(value);
        for field in ["repo", "reason"] {
            let mut value = base.clone();
            value["routing"][field] = serde_json::json!("");
            invalid.push(value);
        }
        for tokens in [0, MAX_LAYA_OUTPUT_BYTES] {
            let mut value = base.clone();
            value["usage"]["input_tokens"] = serde_json::json!(tokens);
            invalid.push(value);
        }
        let mut value = base.clone();
        value["usage"]["output_tokens"] = serde_json::json!(MAX_LAYA_OUTPUT_BYTES + 1);
        invalid.push(value);
        let mut value = base;
        value["unknown"] = serde_json::json!(true);
        invalid.push(value);
        for value in invalid {
            let body = serde_json::to_vec(&value).unwrap();
            let server = ControlledServer::start(
                vec![
                    (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                    (response("200 OK", &body), Duration::ZERO),
                ],
                None,
            );
            assert_eq!(
                DecisionServiceClient::new().decide_routing(
                    "laya168",
                    &service(&server.endpoint),
                    &routing_request(1),
                    CallerDeadline::new()
                ),
                Err(DecisionServiceError::ResponseInvalid)
            );
            assert_eq!(server.calls.load(Ordering::SeqCst), 2);
            server.join();
        }
    }

    #[test]
    fn spec168_laya_reutilise_sante_cachee_par_identifiant_service() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let decision = serde_json::to_vec(&routing_laya_response("c001")).unwrap();
        let server = ControlledServer::start(
            vec![
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                (response("200 OK", &decision), Duration::ZERO),
                (response("200 OK", &decision), Duration::ZERO),
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                (response("200 OK", &decision), Duration::ZERO),
            ],
            None,
        );
        let configured = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let request = routing_request(1);
        for service_id in ["laya168", "laya168", "autre-service168"] {
            assert_eq!(
                client
                    .decide_routing(service_id, &configured, &request, CallerDeadline::new())
                    .unwrap()
                    .candidate_id,
                "c001"
            );
        }
        let methods: Vec<_> = server
            .requests()
            .iter()
            .map(|request| request.split_whitespace().next().unwrap().to_owned())
            .collect();
        assert_eq!(
            methods,
            ["GET", "POST", "POST", "GET", "POST"],
            "le même ID réemploie sa santé ; un autre ID garde sa propre clé"
        );
        assert_eq!(server.calls.load(Ordering::SeqCst), 5);
        server.join();
    }

    #[test]
    fn spec168_laya_sante_negative_et_cooldown_interdisent_post_et_gardent_motif() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        for (unavailable, cause) in [
            (
                response("503 Service Unavailable", b"failure"),
                DecisionServiceError::HttpStatus,
            ),
            (
                response("200 OK", b"{"),
                DecisionServiceError::ResponseInvalid,
            ),
        ] {
            let decision = serde_json::to_vec(&routing_laya_response("c001")).unwrap();
            let server = ControlledServer::start(
                vec![
                    (unavailable, Duration::ZERO),
                    (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                    (response("200 OK", &decision), Duration::ZERO),
                ],
                None,
            );
            let mut configured = service(&server.endpoint);
            configured.retry_after_seconds = 1;
            let client = DecisionServiceClient::new();
            let request = routing_request(1);
            for _ in 0..2 {
                let error = client
                    .decide_routing("laya168", &configured, &request, CallerDeadline::new())
                    .unwrap_err();
                assert_eq!(
                    error,
                    DecisionServiceError::HealthUnavailable {
                        reason: cause.code()
                    }
                );
                assert_eq!(
                    error.code(),
                    cause.code(),
                    "le reçu garde le motif de santé initial"
                );
            }
            assert_eq!(
                server.calls.load(Ordering::SeqCst),
                1,
                "la santé négative est cachée ; aucun POST ni nouvelle sonde en cooldown"
            );
            assert!(server.requests()[0].starts_with("GET /health HTTP/1.1\r\n"));
            thread::sleep(Duration::from_millis(1100));
            assert_eq!(
                client
                    .decide_routing("laya168", &configured, &request, CallerDeadline::new())
                    .unwrap()
                    .candidate_id,
                "c001"
            );
            let requests = server.requests();
            assert_eq!(requests.len(), 3);
            assert!(requests[1].starts_with("GET /health HTTP/1.1\r\n"));
            assert!(requests[2].starts_with("POST /v1/systemone HTTP/1.1\r\n"));
            server.join_assert_no_extra_requests();
        }
    }

    #[test]
    fn spec168_laya_sante_post_partagent_deadline_et_permis_reseau_jusqu_au_drain() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let decision = serde_json::to_vec(&routing_laya_response("c001")).unwrap();
        let (release, response_gate) = mpsc::channel();
        let server = ControlledServer::start_gated(
            vec![
                (
                    response("200 OK", br#"{"status":"ok"}"#),
                    Duration::from_millis(200),
                ),
                (response("200 OK", &decision), Duration::ZERO),
            ],
            None,
            Some(response_gate),
        );
        let configured = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let request = routing_request(1);
        let started = Instant::now();
        let deadline = CallerDeadline::until(started + Duration::from_millis(700));
        let outcome = client.decide_routing("laya168", &configured, &request, deadline);
        let elapsed = started.elapsed();
        let occupied =
            client.decide_routing("laya168", &configured, &request, CallerDeadline::new());
        // Le POST reste physiquement en attente derrière ce verrou, même
        // après la deadline du demandeur. Lever avant toute assertion.
        let released = release.send(());
        let requests = server.requests();
        server.join();
        let drained_at = Instant::now();
        while client.state.lock().unwrap().in_flight
            && drained_at.elapsed() < Duration::from_secs(1)
        {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(outcome, Err(DecisionServiceError::CallerDeadline));
        assert!(
            elapsed >= Duration::from_millis(700) && elapsed < Duration::from_secs(1),
            "GET et POST doivent consommer une seule échéance absolue"
        );
        assert_eq!(occupied, Err(DecisionServiceError::CapacityOccupied));
        assert!(released.is_ok());
        assert!(
            !client.state.lock().unwrap().in_flight,
            "le seul permis HTTP se libère au retour physique, après ouverture du verrou"
        );
        assert_eq!(requests.len(), 2);
        assert!(requests[0].starts_with("GET /health HTTP/1.1\r\n"));
        assert!(requests[1].starts_with("POST /v1/systemone HTTP/1.1\r\n"));
    }

    #[test]
    fn spec168_effort_valide_ou_defaut_atteste_sans_niveau_invente() {
        let request = routing_request(1);
        assert_eq!(
            validate_routing_choice("c001", Some("high"), None, &request)
                .unwrap()
                .effort,
            Some("high".into())
        );
        assert_eq!(
            validate_routing_choice("c001", None, None, &request)
                .unwrap()
                .effort,
            Some("low".into())
        );
        assert_eq!(
            validate_routing_choice("c001", Some("ultra"), None, &request),
            Err(DecisionServiceError::ResponseInvalid)
        );
        let mut missing_default = request;
        missing_default.candidates[0].default_effort = None;
        assert_eq!(
            validate_routing_request(&missing_default),
            Err(DecisionServiceError::RequestInvalid)
        );
        missing_default.candidates[0].supported_efforts.clear();
        assert_eq!(
            validate_routing_choice("c001", None, None, &missing_default)
                .unwrap()
                .effort,
            None
        );
        assert!(!is_auto_candidate_id("c001"));
        assert!(!is_routing_candidate_id("c01"));
    }

    #[test]
    fn spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel() {
        use bridget_transport::{
            ClaudeIsolatedDecisionError, ClaudeIsolatedDecisionOptions,
            run_isolated_claude_decision,
        };
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("bridget-permit168-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let executable = root.join("claude-fixture");
        // Le leader et son descendant ignorent TERM/INT jusqu'à la levée du
        // verrou de test. Le leader récolte son descendant avant de sortir.
        std::fs::write(
            &executable,
            r#"#!/usr/bin/python3
import os,pathlib,signal,sys,time
root=pathlib.Path(__file__).parent
signal.signal(signal.SIGTERM,signal.SIG_IGN)
signal.signal(signal.SIGINT,signal.SIG_IGN)
pid=os.fork()
if pid==0:
    (root/'descendant').write_text(str(os.getpid()))
    while not (root/'release').exists():
        time.sleep(.005)
    os._exit(0)
else:
    (root/'leader').write_text(str(os.getpid()))
    while not (root/'release').exists():
        time.sleep(.005)
    os.waitpid(pid,0)
"#,
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
        let options = ClaudeIsolatedDecisionOptions {
            command: executable.to_string_lossy().into_owned(),
            command_args: vec![],
            model: "claude-fixture168".into(),
            environment: vec![],
            request_json:
                "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"choisir\"}}".into(),
            deadline: Instant::now() + Duration::from_millis(100),
        };
        let client = DecisionServiceClient::new();
        let permit = client.acquire_routing_permit().unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let result = run_isolated_claude_decision(&options);
            drop(permit);
            let _ = sender.send(result);
        });
        let started = Instant::now();
        while !root.join("descendant").exists() && started.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(5));
        }
        let ready = root.join("descendant").exists();
        let caller_timed_out = matches!(
            receiver.recv_timeout(Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Timeout)
        );
        let occupied = matches!(
            client.acquire_routing_permit(),
            Err(DecisionServiceError::CapacityOccupied)
        );
        // Nettoyer même si un oracle ci-dessus devient faux : aucune assertion
        // ne doit laisser les vrais processus figés sur le verrou de fixture.
        std::fs::write(root.join("release"), b"release").unwrap();
        let outcome = receiver.recv_timeout(Duration::from_secs(4));
        worker.join().unwrap();
        assert!(ready);
        assert!(
            caller_timed_out,
            "la deadline demandeur n'abandonne pas le propriétaire"
        );
        assert!(
            occupied,
            "aucune seconde chaîne tant qu'un descendant reste vivant"
        );
        assert_eq!(outcome.unwrap(), Err(ClaudeIsolatedDecisionError::Timeout));
        let leader: i32 = std::fs::read_to_string(root.join("leader"))
            .unwrap()
            .parse()
            .unwrap();
        assert!(unsafe { libc::kill(-leader, 0) } < 0);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        assert!(
            client.acquire_routing_permit().is_ok(),
            "permis libéré après arrêt confirmé seulement"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec163_laya_normalise_sans_prompt_ni_candidat_hors_snapshot() {
        let request = decision_request();
        let configured = service("https://decision.internal/v1");
        let encoded = laya_request_payload(&configured, &request).unwrap();
        assert!(encoded.len() <= MAX_DECISION_JSON_BYTES);
        let text = String::from_utf8(encoded).unwrap();
        assert!(!text.contains("/tmp"));
        assert!(!text.contains("credential"));
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(payload["model"], "typed-decisions");
        assert_eq!(payload["questions"]["candidate"]["type"], "choice");
        assert!(
            payload["state"]
                .as_str()
                .unwrap()
                .contains("objective=Choisir un candidat")
        );
        let mut generated = laya_request_payload(&configured, &maximum_decision_request()).unwrap();
        // L'artefact versionné est la sortie exacte du constructeur, suivie de
        // son unique fin de ligne de fichier. Toute dérive de wire est donc
        // visible au byte près, sans POST de modèle.
        generated.push(b'\n');
        assert_eq!(
            generated.as_slice(),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../specs/163-decision-admission/validation/decision-request-max8.json"
            )),
            "l'artefact huit candidats provient du constructeur Laya fermé"
        );
        let response = br#"{"model":"laya-rl-agent","answers":{"candidate":{"type":"choice","choice":"c01","probabilities":{"c01":0.5},"confidence":0.0,"answer_confidence":0.5,"action":{"act_probability":0.5}}},"routing":{"model":"typed-decisions","repo":"convaiinnovations/laya/typed-decisions","reason":"explicit model=typed-decisions","detection":null,"workflow":null},"usage":{"input_tokens":12,"output_tokens":0}}"#;
        assert_eq!(
            parse_laya_choice(response, &request.candidates, &configured.model).unwrap(),
            DecisionChoice {
                candidate_id: "c01".into(),
                reason: None,
            }
        );
        let mut wrong_model: serde_json::Value = serde_json::from_slice(response).unwrap();
        wrong_model["routing"]["model"] = serde_json::json!("another-model");
        assert_eq!(
            parse_laya_choice(
                &serde_json::to_vec(&wrong_model).unwrap(),
                &request.candidates,
                &configured.model,
            )
            .unwrap_err(),
            DecisionServiceError::ResponseInvalid
        );
        let mut wrong_type: serde_json::Value = serde_json::from_slice(response).unwrap();
        wrong_type["answers"]["candidate"]["type"] = serde_json::json!("text");
        assert_eq!(
            parse_laya_choice(
                &serde_json::to_vec(&wrong_type).unwrap(),
                &request.candidates,
                &configured.model,
            )
            .unwrap_err(),
            DecisionServiceError::ResponseInvalid
        );
        let mut too_many_input: serde_json::Value = serde_json::from_slice(response).unwrap();
        too_many_input["usage"]["input_tokens"] = serde_json::json!(MAX_LAYA_OUTPUT_BYTES);
        assert_eq!(
            parse_laya_choice(
                &serde_json::to_vec(&too_many_input).unwrap(),
                &request.candidates,
                &configured.model,
            )
            .unwrap_err(),
            DecisionServiceError::ResponseInvalid
        );
        let mut too_many_output: serde_json::Value = serde_json::from_slice(response).unwrap();
        too_many_output["usage"]["output_tokens"] = serde_json::json!(MAX_LAYA_OUTPUT_BYTES + 1);
        assert_eq!(
            parse_laya_choice(
                &serde_json::to_vec(&too_many_output).unwrap(),
                &request.candidates,
                &configured.model,
            )
            .unwrap_err(),
            DecisionServiceError::ResponseInvalid
        );
        let hostile = br#"{"answers":{"candidate":{"type":"choice","choice":"c08"}},"routing":{"model":"typed-decisions"},"usage":{"input_tokens":12}}"#;
        assert_eq!(
            parse_laya_choice(hostile, &request.candidates, &configured.model).unwrap_err(),
            DecisionServiceError::ResponseInvalid
        );
    }

    #[test]
    fn spec163_sante_laya_accepte_les_metadonnees_connues_sans_leur_donner_autorite() {
        let master = br#"{"status":"ok","loaded":["typed-decisions","english","multilingual"],"revisions":{"english":"7b928d828b7b0e022f929d9bd2e44165aa270148","multilingual":"7b928d828b7b0e022f929d9bd2e44165aa270148","typed-decisions":"7b928d828b7b0e022f929d9bd2e44165aa270148"},"device":"auto"}"#;
        assert_eq!(parse_laya_health(br#"{"status":"ok"}"#), Ok(()));
        assert_eq!(parse_laya_health(master), Ok(()));
        for invalid in [
            br#"{"status":"degraded"}"#.as_slice(),
            br#"{"status":"ok","loaded":"typed-decisions"}"#.as_slice(),
            br#"{"status":"ok","revisions":["typed-decisions"]}"#.as_slice(),
            br#"{"status":"ok","device":false}"#.as_slice(),
            br#"{"status":"ok","unknown":true}"#.as_slice(),
            br#"{"status":"ok","loaded":["typed-decisions"],"unknown":true}"#.as_slice(),
        ] {
            assert_eq!(
                parse_laya_health(invalid),
                Err(DecisionServiceError::ResponseInvalid),
                "corps santé fermé attendu: {}",
                String::from_utf8_lossy(invalid)
            );
        }
    }

    #[test]
    fn spec163_endpoint_et_runtime_sont_refuses_avant_reseau() {
        assert!(decision_endpoint_allowed("https://decision.internal/v1"));
        assert!(decision_endpoint_allowed("http://127.0.0.1:8792/v1"));
        assert!(!decision_endpoint_allowed("http://decision.internal/v1"));
        assert!(!decision_endpoint_allowed(
            "https://user@decision.internal/v1"
        ));
        assert_eq!(
            health_url("https://decision.internal/v1").unwrap(),
            "https://decision.internal/health"
        );
        let mut disabled = service("https://decision.internal/v1");
        disabled.enabled = false;
        assert_eq!(
            validate_service(&disabled),
            Err(DecisionServiceError::Disabled)
        );
    }

    #[test]
    fn spec163_minreq_timeout_present_vide_ou_invalide_refuse_le_service() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        for value in ["", "not-a-duration", "999999"] {
            let _timeout = MinreqTimeoutRestore::set(Some(value));
            assert!(startup_diagnostic().is_some());
            assert_eq!(
                DecisionServiceClient::new()
                    .health(
                        "laya",
                        &service("https://decision.internal/v1"),
                        CallerDeadline::new(),
                    )
                    .unwrap_err(),
                DecisionServiceError::UnsafeRuntimeConfiguration
            );
        }
    }

    #[test]
    fn spec163_sante_laya_fermee_cache_les_refus_sans_post() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        for body in [br#"{"status":"error"}"#.as_slice(), b"{".as_slice()] {
            let server =
                ControlledServer::start(vec![(response("200 OK", body), Duration::ZERO)], None);
            let mut configured = service(&server.endpoint);
            configured.retry_after_seconds = 30;
            let client = DecisionServiceClient::new();
            let expected = HealthStatus::Unavailable {
                reason: DecisionServiceError::ResponseInvalid.code(),
            };
            assert_eq!(
                client
                    .health("laya", &configured, CallerDeadline::new())
                    .unwrap(),
                expected
            );
            assert_eq!(
                client
                    .health("laya", &configured, CallerDeadline::new())
                    .unwrap(),
                expected
            );
            assert_eq!(server.calls.load(Ordering::SeqCst), 1);
            let requests = server.requests();
            assert_eq!(requests.len(), 1);
            assert!(requests[0].starts_with("GET /health HTTP/1.1\r\n"));
            server.join();
        }
    }

    #[test]
    fn spec163_statut_et_entetes_bornes_restent_des_refus_visibles() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let unavailable = ControlledServer::start(
            vec![(
                response("503 Service Unavailable", b"failure"),
                Duration::ZERO,
            )],
            None,
        );
        let client = DecisionServiceClient::new();
        let configured = service(&unavailable.endpoint);
        let expected = HealthStatus::Unavailable {
            reason: DecisionServiceError::HttpStatus.code(),
        };
        assert_eq!(
            client
                .health("laya", &configured, CallerDeadline::new())
                .unwrap(),
            expected
        );
        assert_eq!(
            client
                .health("laya", &configured, CallerDeadline::new())
                .unwrap(),
            expected
        );
        assert_eq!(unavailable.calls.load(Ordering::SeqCst), 1);
        unavailable.join();

        let headers = format!("X-Fill: {}\r\n", "x".repeat(MAX_RESPONSE_HEADERS_BYTES + 1));
        let oversized_headers = [
            b"HTTP/1.1 200 OK\r\n".as_slice(),
            headers.as_bytes(),
            b"Content-Length: 0\r\nConnection: close\r\n\r\n".as_slice(),
        ]
        .concat();
        let server = ControlledServer::start(vec![(oversized_headers, Duration::ZERO)], None);
        assert_eq!(
            send_bounded(&service(&server.endpoint), "GET", &server.endpoint, None).unwrap_err(),
            DecisionServiceError::Transport
        );
        server.join();
    }

    #[test]
    fn spec163_sante_et_decision_partagent_un_budget_unique() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let decision = br#"{"model":"laya-rl-agent","answers":{"candidate":{"type":"choice","choice":"c01","probabilities":{"c01":0.5},"confidence":0.0,"answer_confidence":0.5,"action":{"act_probability":0.5}}},"routing":{"model":"typed-decisions","repo":"convaiinnovations/laya/typed-decisions","reason":"explicit model=typed-decisions","detection":null,"workflow":null},"usage":{"input_tokens":12,"output_tokens":0}}"#;
        let server = ControlledServer::start(
            vec![
                (
                    response("200 OK", br#"{"status":"ok"}"#),
                    Duration::from_millis(1_500),
                ),
                (response("200 OK", decision), Duration::from_secs(1)),
            ],
            None,
        );
        let configured = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let deadline = CallerDeadline::new();
        assert_eq!(
            client.health("laya", &configured, deadline).unwrap(),
            HealthStatus::Available
        );
        assert_eq!(
            client
                .decide(&configured, &decision_request(), deadline)
                .unwrap_err(),
            DecisionServiceError::CallerDeadline
        );
        server.join();
    }

    #[test]
    fn spec163_sante_cachee_et_decision_bornee_utilisent_le_vrai_transport() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let decision = br#"{"model":"laya-rl-agent","answers":{"candidate":{"type":"choice","choice":"c01","probabilities":{"c01":0.5},"confidence":0.0,"answer_confidence":0.5,"action":{"act_probability":0.5}}},"routing":{"model":"typed-decisions","repo":"convaiinnovations/laya/typed-decisions","reason":"explicit model=typed-decisions","detection":null,"workflow":null},"usage":{"input_tokens":12,"output_tokens":0}}"#;
        let server = ControlledServer::start(
            vec![
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
                (response("200 OK", decision), Duration::ZERO),
            ],
            None,
        );
        let service = service(&server.endpoint);
        let client = DecisionServiceClient::new();
        let deadline = CallerDeadline::new();
        assert_eq!(
            client.health("laya", &service, deadline).unwrap(),
            HealthStatus::Available
        );
        assert_eq!(
            client
                .health("laya", &service, CallerDeadline::new())
                .unwrap(),
            HealthStatus::Available
        );
        let choice = client
            .decide(&service, &decision_request(), deadline)
            .unwrap();
        assert_eq!(choice.candidate_id, "c01");
        assert_eq!(server.calls.load(Ordering::SeqCst), 2);
        server.join();
    }

    #[test]
    fn spec163_corps_excessif_et_permis_occupe_restent_visibles() {
        let _lock = MINREQ_TIMEOUT_LOCK.lock().unwrap();
        let _timeout = MinreqTimeoutRestore::set(None);
        let oversized = vec![b'x'; MAX_RESPONSE_BODY_BYTES + 1];
        let server =
            ControlledServer::start(vec![(response("200 OK", &oversized), Duration::ZERO)], None);
        assert_eq!(
            send_bounded(&service(&server.endpoint), "GET", &server.endpoint, None).unwrap_err(),
            DecisionServiceError::ResponseTooLarge
        );
        server.join();

        let (accepted_tx, accepted_rx) = mpsc::channel();
        let slow = ControlledServer::start(
            vec![
                (
                    response("200 OK", br#"{"status":"ok"}"#),
                    Duration::from_secs(4),
                ),
                (response("200 OK", br#"{"status":"ok"}"#), Duration::ZERO),
            ],
            Some(accepted_tx),
        );
        let mut service = service(&slow.endpoint);
        service.retry_after_seconds = 1;
        let client = DecisionServiceClient::new();
        let pending = {
            let client = client.clone();
            let service = service.clone();
            thread::spawn(move || client.health("laya", &service, CallerDeadline::new()))
        };
        accepted_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(
            client
                .health("laya", &service, CallerDeadline::new())
                .unwrap_err(),
            DecisionServiceError::CapacityOccupied
        );
        assert_eq!(
            pending.join().unwrap().unwrap(),
            HealthStatus::Unavailable {
                reason: DecisionServiceError::CallerDeadline.code(),
            }
        );
        // Le cache négatif a expiré, mais le thread réseau lent conserve le
        // permis jusqu'au retour réel : aucune deuxième sonde ne part.
        thread::sleep(Duration::from_millis(1_100));
        assert_eq!(
            client
                .health("laya", &service, CallerDeadline::new())
                .unwrap_err(),
            DecisionServiceError::CapacityOccupied
        );
        thread::sleep(Duration::from_millis(1_100));
        assert_eq!(
            client
                .health("laya", &service, CallerDeadline::new())
                .unwrap(),
            HealthStatus::Available
        );
        assert_eq!(slow.calls.load(Ordering::SeqCst), 2);
        slow.join();
    }
}
