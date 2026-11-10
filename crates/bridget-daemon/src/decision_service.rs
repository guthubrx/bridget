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
    status: u16,
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
    if let Some(rest) = endpoint.strip_prefix("https://") {
        return rest
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
#[path = "decision_service_tests.rs"]
pub(crate) mod tests;
