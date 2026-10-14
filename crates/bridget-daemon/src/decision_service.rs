//! Frontière HTTP volontairement étroite pour le décideur d'admission 163.
//!
//! Aucune admission ne dépend de ce module tant que la politique et le puits
//! durable ne l'ont pas explicitement choisi. Cette isolation conserve le
//! client HTTP historique du daemon et rend visibles les limites d'un appel
//! bloqué sans introduire de runtime, file ou fournisseur supplémentaire.

use crate::registry::DecisionService;
use bridget_transport::protocol::is_auto_candidate_id;
use serde::Deserialize;
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

#[derive(Default)]
struct DecisionServiceState {
    in_flight: bool,
    health: BTreeMap<String, HealthCacheEntry>,
}

struct HealthCacheEntry {
    status: HealthStatus,
    expires_at: Instant,
}

struct NetworkPermit {
    state: Arc<Mutex<DecisionServiceState>>,
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
            }
        }

        pub(crate) fn requests(&self) -> Vec<String> {
            self.requests.lock().unwrap().clone()
        }

        pub(crate) fn join(self) {
            self.worker.join().unwrap();
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
