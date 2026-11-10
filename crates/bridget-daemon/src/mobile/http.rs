//! Frontière HTTP humaine ; limites appliquées avant lecture et travail bloquant.
use super::{
    Config,
    store::{Device, Operation, Store, StoreError},
};
use axum::{
    Router,
    body::to_bytes,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::Semaphore;

const BODY_LIMIT: usize = 64 * 1024;
const UPLOAD_LIMIT: usize = (8usize * 1024 * 1024).div_ceil(3) * 4 + 16 * 1024;
const REQUEST_BUDGET: Duration = Duration::from_secs(20);
const CONNECTION_BUDGET: Duration = Duration::from_secs(30);
const MAX_CONNECTIONS: usize = 32;
const MAX_BLOCKING: usize = 8;
const FOLLOW_BUDGET: Duration = Duration::from_secs(25);
const FOLLOW_PER_DEVICE: usize = 4;

#[derive(Clone)]
struct Gateway {
    root: PathBuf,
    config: Config,
    socket: PathBuf,
    store: Arc<Mutex<Store>>,
    workers: Arc<Semaphore>,
    bootstrap_rate: Arc<Mutex<BTreeMap<String,(Instant,u16)>>>,
    renewal_rate: Arc<Mutex<BTreeMap<String,(Instant,u16)>>>,
    followers: Arc<Mutex<BTreeMap<String, Arc<Semaphore>>>>,
    follow_slots: Arc<Semaphore>,
    uploads: Arc<Semaphore>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FollowCursor {
    version: u8,
    device_id: String,
    agent_id: String,
    after_seq: u64,
}

fn failure(code: &str) -> Response {
    let (status, message, recovery) = match code {
        "authorization_required" => (
            401,
            "Téléphone non authentifié.",
            "Associez le téléphone avec un QR valide.",
        ),
        "device_revoked" => (
            401,
            "Téléphone révoqué.",
            "Demandez une nouvelle association depuis l'installation de confiance.",
        ),
        "credential_expired" => (
            401,
            "Autorisation expirée.",
            "Associez à nouveau ce téléphone.",
        ),
        "pairing_unavailable" => (
            409,
            "Association indisponible.",
            "Réconciliez la tentative conservée ; sinon demandez un nouveau QR.",
        ),
        "command_origin_changed" => (
            409,
            "Cette commande appartient à une autre association.",
            "Conservez son enveloppe et consultez son installation d’origine ; ne la renvoyez pas avec une nouvelle association.",
        ),
        "envelope_mismatch" => (
            409,
            "Cette clé désigne une autre commande.",
            "Conservez la clé et l'enveloppe d'origine.",
        ),
        "forbidden_scope" => (
            403,
            "Action hors des droits de ce téléphone.",
            "Choisissez une machine, un projet ou une action autorisés.",
        ),
        "payload_too_large" => (
            413,
            "Requête trop volumineuse.",
            "Respectez la limite annoncée.",
        ),
        "busy" => (
            429,
            "Accès momentanément saturé.",
            "Attendez puis conservez la même clé pour réconcilier.",
        ),
        "update_in_progress" => (
            503,
            "Mise à jour Bridget en cours. Cet appel n’a pas été exécuté.",
            "Attendez le retour de la connexion, puis consultez les reçus existants. Conservez les mêmes identifiants et paramètres.",
        ),
        "outcome_unknown" => (
            504,
            "Réponse non confirmée.",
            "Lisez le reçu avec la clé conservée avant tout nouvel envoi.",
        ),
        "resync_required" => (
            409,
            "Ce repère ne correspond plus à cette vue.",
            "Relisez l’historique de cet agent depuis un repère valide ; conservez les commandes enregistrées.",
        ),
        "history_scope_unavailable" => (
            409,
            "La portée de cet historique ne peut pas être attestée.",
            "Consultez cet historique depuis le poste d'origine ; les droits du téléphone restent inchangés.",
        ),
        "journal_unavailable" => (
            503,
            "Historique momentanément indisponible.",
            "Conservez le dernier repère et réessayez la lecture plus tard.",
        ),
        "notifications_not_configured" => (
            503,
            "Notifications pas encore configurées sur cette installation.",
            "Conservez l’association ; l’administrateur doit configurer le service de notification gratuit.",
        ),
        "not_found" => (
            404,
            "Objet indisponible.",
            "Relisez la liste des objets autorisés.",
        ),
        "invalid_file" | "file_type_unavailable" => (
            400,
            "Type ou contenu du fichier refusé.",
            "Utilisez un PDF, du texte UTF-8, une image PNG ou JPEG, sans chemin et de 8 Mio maximum.",
        ),
        "choice_not_offered" => (
            409,
            "Ce choix n’est pas proposé pour cette question.",
            "Relisez les options actuelles ; conservez la commande d’origine.",
        ),
        "question_already_resolved" => (
            409,
            "Cette question a déjà reçu une décision.",
            "Relisez la décision native et son acquittement ; ne recréez pas une réponse.",
        ),
        "question_unavailable" => (
            409,
            "Cette question n’est plus disponible.",
            "Relisez la boîte humaine ; aucun nouveau droit n’a été accordé.",
        ),
        "resource_limit" => (
            409,
            "La limite de cette vue est atteinte.",
            "Conservez votre repère et consultez l’installation de confiance pour les objets hors limite.",
        ),
        "artifact_unavailable" => (
            503,
            "Fichier momentanément indisponible.",
            "Conservez sa référence et reprenez la lecture ; aucun chemin de disque ne doit être substitué.",
        ),
        "publication_capacity_reached" => (
            409,
            "Quota de fichiers publiés atteint.",
            "Examinez les contenus conservés avec l’administrateur ; aucun fichier existant n’a été supprimé.",
        ),
        "capability_unavailable" => (
            409,
            "Action non disponible.",
            "Utilisez un agent enregistré pour demander du travail.",
        ),
        "endpoint_unavailable" => (
            503,
            "Maître Bridget momentanément inaccessible.",
            "Conservez vos commandes et réessayez la lecture plus tard.",
        ),
        "storage_unavailable" => (
            503,
            "Autorisations momentanément indisponibles.",
            "Conservez votre tentative et contactez l'administrateur.",
        ),
        "device_limit" => (
            409,
            "Limite d'appareils atteinte.",
            "Révoquez un ancien téléphone depuis l'installation de confiance.",
        ),
        _ => (
            400,
            "Requête refusée.",
            "Vérifiez la version et les champs de la requête.",
        ),
    };
    let mut response = (
        StatusCode::from_u16(status).unwrap(),
        axum::Json(json!({
            "version":1,"status":"refused","code":code,"message":message,"recovery":recovery
        })),
    )
        .into_response();
    if status == 429 {
        response
            .headers_mut()
            .insert("retry-after", "10".parse().unwrap());
    }
    response
}

fn result(value: Value) -> Response {
    axum::Json(json!({"version":1,"status":"ok","result":value})).into_response()
}

fn strict<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Response> {
    serde_json::from_slice(bytes).map_err(|_| failure("invalid_request"))
}

fn single_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let mut values = headers.get_all(name).iter();
    let first = values.next()?.to_str().ok()?;
    if values.next().is_some() {
        None
    } else {
        Some(first)
    }
}

fn bearer(headers: &HeaderMap) -> Result<&str, Response> {
    single_header(headers, "authorization")
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|key| {
            key.len() == 64
                && key
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
        .ok_or_else(|| failure("authorization_required"))
}

impl Gateway {
    fn follow_permit(
        &self,
        device: &str,
    ) -> Result<
        (
            tokio::sync::OwnedSemaphorePermit,
            tokio::sync::OwnedSemaphorePermit,
        ),
        Response,
    > {
        let global = self
            .follow_slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| failure("busy"))?;
        let mut followers = self.followers.lock().map_err(|_| failure("busy"))?;
        followers.retain(|_, permits| {
            Arc::strong_count(permits) > 1 || permits.available_permits() != FOLLOW_PER_DEVICE
        });
        if !followers.contains_key(device) && followers.len() >= MAX_CONNECTIONS {
            return Err(failure("busy"));
        }
        followers
            .entry(device.into())
            .or_insert_with(|| Arc::new(Semaphore::new(FOLLOW_PER_DEVICE)))
            .clone()
            .try_acquire_owned()
            .map(|local| (global, local))
            .map_err(|_| failure("busy"))
    }
    fn bootstrap_allowed(&self, source: &str) -> bool {
        rate_allowed(&self.bootstrap_rate, source, 1024)
    }

}

// O(S), S borné ; aucune adresse ni empreinte n’est conservée au-delà d’une minute.
fn rate_allowed(rates:&Mutex<BTreeMap<String,(Instant,u16)>>,subject:&str,capacity:usize)->bool {
    let Ok(mut rates)=rates.lock() else {return false;};
    rates.retain(|_,(start,_)|start.elapsed()<Duration::from_secs(60));
    if !rates.contains_key(subject) && rates.len()>=capacity {return false;}
    let rate=rates.entry(subject.to_owned()).or_insert((Instant::now(),0));
    if rate.1>=30 {return false;}
    rate.1+=1;true
}

async fn handle(State(gateway): State<Gateway>, request: Request) -> Response {
    let budget = if request.method() == Method::GET && request.uri().path() == "/v1/events" {
        FOLLOW_BUDGET + Duration::from_secs(3)
    } else {
        REQUEST_BUDGET
    };
    let mut response = match tokio::time::timeout(budget, dispatch(gateway, request)).await {
        Ok(response) => response,
        Err(_) => failure("outcome_unknown"),
    };
    let headers = response.headers_mut();
    headers.insert("cache-control", "no-store".parse().unwrap());
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("referrer-policy", "no-referrer".parse().unwrap());
    // Une seule requête par connexion ; la durée totale du flux est bornée.
    headers.insert("connection", "close".parse().unwrap());
    response
}

async fn dispatch(gateway: Gateway, request: Request) -> Response {
    let expected_host = gateway
        .config
        .public_url
        .strip_prefix("https://")
        .unwrap_or("");
    if single_header(request.headers(), "host") != Some(expected_host)
        || (request.headers().contains_key("origin")
            && single_header(request.headers(), "origin")
                != Some(gateway.config.public_url.as_str()))
    {
        return failure("invalid_request");
    }
    let query = if matches!(
        request.uri().path(),
        "/v1/agents"
            | "/v1/projects"
            | "/v1/federation"
            | "/v1/events"
            | "/v1/devices"
            | "/v1/tasks"
            | "/v1/questions"
            | "/v1/user-inputs"
    ) || request.uri().path().starts_with("/v1/questions/")
        || (request.uri().path().starts_with("/v1/tasks/")
            && !request.uri().path().contains("/cancel/"))
        || (request.uri().path().starts_with("/v1/agents/")
            && (request.uri().path().ends_with("/events")
                || request.uri().path().ends_with("/artifacts")))
    {
        match axum::extract::Query::<super::client::DirectoryQuery>::try_from_uri(request.uri()) {
            Ok(query) if request.uri().to_string().len() <= 8192 => query.0,
            _ => return failure("invalid_request"),
        }
    } else if request.uri().query().is_some() {
        return failure("invalid_request");
    } else {
        super::client::DirectoryQuery::default()
    };
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    if method == Method::GET && path == "/v1/status" {
        return result(json!({"protocol_version":1,"minimum_client_version":1}));
    }
    if method==Method::POST && path=="/v1/pair" {
        // Seul le proxy loopback installé écrit ce champ ; il remplace toute valeur cliente.
        let source=match single_header(request.headers(),"x-bridget-client-ip")
            .and_then(|v|v.parse::<std::net::IpAddr>().ok()) {
            Some(value)=>value.to_string(),None=>return failure("invalid_request")
        };
        if !gateway.bootstrap_allowed(&source) {return failure("busy");}
    }
    let credential = if path == "/v1/pair" && method == Method::POST {
        None
    } else {
        match bearer(request.headers()) {
            Ok(value) => Some(value.to_owned()),
            Err(response) => return response,
        }
    };
    if method==Method::POST && path=="/v1/session/renew" {
        let subject=match gateway.store.lock() {
            Ok(store)=>match store.renewal_subject(credential.as_deref().unwrap_or(""),unix_now()) {
                Ok(device)=>device.device_id,Err(error)=>return failure(error.code())
            },Err(_)=>return failure("storage_unavailable")
        };
        if !rate_allowed(&gateway.renewal_rate,&subject,256) {return failure("busy");}
    }
    let permit = match gateway.workers.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => return failure("busy"),
    };
    if matches!(method, Method::POST | Method::PUT | Method::PATCH)
        && single_header(request.headers(), "content-type")
            .and_then(|v| v.split(';').next())
            .map(str::trim)
            != Some("application/json")
    {
        return failure("invalid_request");
    }
    let is_upload = method == Method::POST && path == "/v1/artifacts";
    let upload_permit = if is_upload {
        let current = match gateway.store.lock() {
            Ok(store) => {
                match store.authenticate(credential.as_deref().unwrap_or(""), unix_now()) {
                    Ok(device) => device,
                    Err(error) => return failure(error.code()),
                }
            }
            Err(_) => return failure("storage_unavailable"),
        };
        if !current.scope.operations.contains(&Operation::Message) {
            return failure("forbidden_scope");
        }
        match gateway.uploads.clone().try_acquire_owned() {
            Ok(permit) => Some(permit),
            Err(_) => return failure("busy"),
        }
    } else {
        None
    };
    let bytes = match to_bytes(
        request.into_body(),
        if is_upload { UPLOAD_LIMIT } else { BODY_LIMIT },
    )
    .await
    {
        Ok(bytes) => bytes,
        Err(_) => return failure("payload_too_large"),
    };
    let deadline = Instant::now()
        + if path == "/v1/events" {
            FOLLOW_BUDGET
        } else {
            REQUEST_BUDGET
        };
    // Le permis reste dans le travail lancé même si la réponse HTTP est perdue.
    match tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let _upload_permit = upload_permit;
        if Instant::now() >= deadline {
            return failure("outcome_unknown");
        }
        execute(
            &gateway,
            method,
            &path,
            credential.as_deref(),
            &bytes,
            &query,
            deadline,
        )
    })
    .await
    {
        Ok(response) => response,
        Err(_) => failure("storage_unavailable"),
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum CommandRequest {
    Message {
        device_id: String,
        command_id: String,
        issued_at: i64,
        agent_id: String,
        body: String,
        #[serde(default)]
        guide_now: bool,
    },
    TaskCancel {
        device_id: String,
        command_id: String,
        task_id: String,
    },
    UserInputRespond { device_id:String, command_id:String, request_id:String, revision:String, answers:Value },
    QuestionRespond {
        device_id: String,
        command_id: String,
        item_id: String,
        choice: String,
    },
    CreateAgent,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueueChangeRequest {
    command_id: String,
    request_id: String,
    action: String,
    expected_body: String,
    body: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PairRequest {
    credential: String,
    attempt_id: String,
    device_key: String,
    label: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenewRequest {
    request_id: String,
    device_key: String,
}

fn full_management(device: &Device) -> bool {
    device.scope.all_machines
        && device.scope.all_projects
        && device.scope.operations.contains(&Operation::DevicesManage)
}

fn execute(
    gateway: &Gateway,
    method: Method,
    path: &str,
    key: Option<&str>,
    bytes: &[u8],
    query: &super::client::DirectoryQuery,
    deadline: Instant,
) -> Response {
    if (query.choice.is_some()
        && !(method == Method::GET
            && path.starts_with("/v1/questions/")
            && path.contains("/responses/")))
        || ((path.starts_with("/v1/questions") || path.starts_with("/v1/user-inputs"))
            && (query.machine.is_some()
                || query.project_host.is_some()
                || query.agent_id.is_some()
                || query.offset.is_some()
                || query.after_seq.is_some()))
        || ((path.starts_with("/v1/questions/") || path.starts_with("/v1/user-inputs/")) && (query.after.is_some() || query.limit.is_some()))
        || query.validate().is_err()
        || (!matches!(path, "/v1/events" | "/v1/tasks") && query.agent_id.is_some())
        || (matches!(path, "/v1/federation" | "/v1/devices" | "/v1/events")
            && (query.machine.is_some()
                || query.project_host.is_some()
                || query.project_root.is_some()))
        || (query.tail.is_some() && !(method==Method::GET && path.starts_with("/v1/agents/") && path.ends_with("/events")))
        || (query.tail==Some(true) && (query.after_seq.is_some() || query.after.is_some()))
        || (!path.ends_with("/events") && query.after_seq.is_some())
        || (!path.starts_with("/v1/tasks/") && query.offset.is_some())
        || (path.starts_with("/v1/tasks/")
            && (query.after.is_some()
                || query.agent_id.is_some()
                || query.limit.is_some()
                || query.machine.is_some()
                || query.project_host.is_some()))
    {
        return failure("invalid_request");
    }
    let now = unix_now();
    let mut store = match gateway.store.lock() {
        Ok(store) => store,
        Err(_) => return failure("storage_unavailable"),
    };
    if method == Method::POST && path == "/v1/pair" {
        let request: PairRequest = match strict(bytes) {
            Ok(v) => v,
            Err(r) => return r,
        };
        return match store.pair(
            &request.credential,
            &request.attempt_id,
            &request.device_key,
            &request.label,
            now,
        ) {
            Ok(device) => result(json!(device)),
            Err(error) => failure(error.code()),
        };
    }
    let Some(key) = key else {
        return failure("authorization_required");
    };
    // Seul ce reçu de rotation peut réutiliser une ancienne clé, pour le même échange.
    if method == Method::POST && path == "/v1/session/renew" {
        let request: RenewRequest = match strict(bytes) {
            Ok(v) => v,
            Err(r) => return r,
        };
        return match store.renew(key, &request.device_key, &request.request_id, now) {
            Ok(device) => result(json!(device)),
            Err(error) => failure(error.code()),
        };
    }
    let device = match store.authenticate(key, now) {
        Ok(device) => device,
        Err(error) => return failure(error.code()),
    };
    if method == Method::GET && path == "/v1/session" {
        return result(json!(device));
    }
    if method == Method::GET && path == "/v1/sentinel" {
        if !device.scope.permits_federation(Operation::Read) { return failure("forbidden_scope"); }
        drop(store);
        return match crate::sentinel244::read_remote(&gateway.socket) {
            Ok(report)=>result(report),Err(_)=>failure("endpoint_unavailable"),
        };
    }
    if path == "/v1/notifications" && matches!(method, Method::GET | Method::PUT) {
        if !device.scope.permits_federation(Operation::Read)
            || !device.scope.permits_federation(Operation::TaskRead)
        {
            return failure("forbidden_scope");
        }
        let root = &gateway.root;
        let configured = super::notifications::configured(root);
        if method == Method::PUT {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Subscription {
                enabled: bool,
                token: Option<String>,
            }
            let subscription: Subscription = match strict(bytes) {
                Ok(v) => v,
                Err(r) => return r,
            };
            if subscription.enabled && !configured {
                return failure("notifications_not_configured");
            }
            if subscription.enabled != subscription.token.is_some() {
                return failure("invalid_request");
            }
            if let Err(error) = store.set_notifications(&device, subscription.token.as_deref(), now)
            {
                return failure(error.code());
            }
        }
        let enabled = match store.notification_targets(now) {
            Ok(rows) => rows.iter().any(|(id, _)| id == &device.device_id),
            Err(error) => return failure(error.code()),
        };
        return result(json!({"configured":configured,"enabled":enabled}));
    }
    if method == Method::GET && path == "/v1/devices" {
        let devices = if full_management(&device) {
            match store.devices() {
                Ok(devices) => devices,
                Err(error) => return failure(error.code()),
            }
        } else {
            vec![device]
        };
        let mut devices = devices
            .into_iter()
            .filter(|d| {
                query
                    .after
                    .as_ref()
                    .is_none_or(|after| &d.device_id > after)
            })
            .collect::<Vec<_>>();
        devices.sort_by(|a, b| a.device_id.cmp(&b.device_id));
        let limit = query.limit.unwrap_or(50);
        let next = (devices.len() > limit).then(|| devices[limit - 1].device_id.clone());
        devices.truncate(limit);
        return result(json!({"items":devices,"next_cursor":next}));
    }
    if method == Method::DELETE
        && let Some(id) = path.strip_prefix("/v1/devices/")
    {
        if id != device.device_id && !full_management(&device) {
            return failure("forbidden_scope");
        }
        return match store.revoke(id, now) {
            Ok(found) => result(json!({"device_id":id,"revoked":found})),
            Err(error) => failure(error.code()),
        };
    }
    drop(store);
    if method == Method::POST && path == "/v1/agents/tracking" {
        let command:bridget_transport::agent_tracking::AgentTrackingRequest=match serde_json::from_slice(bytes) {
            Ok(command)=>command,Err(_)=>return failure("invalid_request"),
        };
        return match super::client::mobile_operation(&gateway.socket,key,bridget_transport::protocol::MobileOperation::AgentTracking{request:command}) {
            Ok(value)=>result(value),Err(code)=>failure(code),
        };
    }
    if method == Method::POST && path == "/v1/specialists" {
        let command:bridget_transport::specialists::SpecialistRequest=match serde_json::from_slice(bytes) {
            Ok(command)=>command,Err(_)=>return failure("invalid_request"),
        };
        if command.validate().is_err(){return failure("invalid_request");}
        return match super::client::mobile_operation(&gateway.socket,key,bridget_transport::protocol::MobileOperation::Specialists{request:command}) {
            Ok(value)=>result(value),Err(code)=>failure(code),
        };
    }
    if method == Method::GET && path == "/v1/events" {
        return follow(gateway, key, &device, query, deadline);
    }
    if method == Method::GET && matches!(path, "/v1/agents" | "/v1/projects") {
        if !device.scope.operations.contains(&Operation::Read) {
            return failure("forbidden_scope");
        }
        let agents = match super::client::directory(&gateway.socket) {
            Ok(items) => items,
            Err(code) => return failure(code),
        };
        // L'autorisation peut avoir changé pendant l'aller-retour Unix.
        let current = match gateway.store.lock() {
            Ok(store) => match store.authenticate(
                key,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64,
            ) {
                Ok(device) => device,
                Err(error) => return failure(error.code()),
            },
            Err(_) => return failure("storage_unavailable"),
        };
        let view = match path {
            "/v1/agents" => super::client::agents_page(&current, &agents, query),
            "/v1/projects" => super::client::projects_page(&current, &agents, query),
            _ => Err("invalid_request"),
        };
        return match view {
            Ok(value) => result(value),
            Err(code) => failure(code),
        };
    }

    let operation = if method == Method::GET && path == "/v1/updates" {
        Some(bridget_transport::protocol::MobileOperation::UpdatesStatus)
    } else if method == Method::POST && path == "/v1/updates/check" && bytes == b"{}" {
        Some(bridget_transport::protocol::MobileOperation::UpdatesCheck)
    } else if method == Method::POST && path == "/v1/updates/install" {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Install {
            version: String,
        }
        match serde_json::from_slice::<Install>(bytes) {
            Ok(value) => Some(
                bridget_transport::protocol::MobileOperation::UpdatesInstall {
                    version: value.version,
                },
            ),
            Err(_) => return failure("invalid_request"),
        }
    } else if method == Method::PUT && path == "/v1/updates" {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Setting {
            automatic: bool,
        }
        match serde_json::from_slice::<Setting>(bytes) {
            Ok(value) => Some(
                bridget_transport::protocol::MobileOperation::UpdatesConfigure {
                    automatic: value.automatic,
                },
            ),
            Err(_) => return failure("invalid_request"),
        }
    } else if method == Method::GET && path.starts_with("/v1/agents/") && path.ends_with("/events")
    {
        Some(bridget_transport::protocol::MobileOperation::AgentJournal {
            agent_id: path
                .trim_start_matches("/v1/agents/")
                .trim_end_matches("/events")
                .into(),
            after_seq: query.after_seq.unwrap_or(0),
            tail: query.tail.unwrap_or(false),
            limit: match u32::try_from(query.limit.unwrap_or(50)) {
                Ok(v) => v,
                Err(_) => return failure("invalid_request"),
            },
        })
    } else if method == Method::GET
        && path.starts_with("/v1/agents/")
        && path.ends_with("/artifacts")
    {
        Some(bridget_transport::protocol::MobileOperation::ArtifactList {
            agent_id: path
                .trim_start_matches("/v1/agents/")
                .trim_end_matches("/artifacts")
                .into(),
            after: query.after.clone(),
            limit: query.limit.unwrap_or(50) as u16,
        })
    } else if method == Method::POST && path == "/v1/artifacts" {
        match strict::<bridget_transport::protocol::MobileFileUpload>(bytes) {
            Ok(upload) => {
                Some(bridget_transport::protocol::MobileOperation::ArtifactUpload { upload })
            }
            Err(response) => return response,
        }
    } else if method == Method::POST && path == "/v1/artifacts/read" {
        match strict::<bridget_transport::protocol::MobileOperation>(bytes) {
            Ok(op @ bridget_transport::protocol::MobileOperation::ArtifactRead { .. }) => Some(op),
            _ => return failure("invalid_request"),
        }
    } else if method == Method::GET && path.starts_with("/v1/artifacts/uploads/") {
        Some(
            bridget_transport::protocol::MobileOperation::ArtifactLookup {
                command_id: path.trim_start_matches("/v1/artifacts/uploads/").into(),
            },
        )
    } else if path == "/v1/user-inputs" && method == Method::GET {
        Some(bridget_transport::protocol::MobileOperation::UserInputs {after:query.after.clone(),limit:query.limit.unwrap_or(50) as u32})
    } else if path.starts_with("/v1/user-inputs/") && method == Method::GET {
        match path.trim_start_matches("/v1/user-inputs/").split('/').collect::<Vec<_>>().as_slice() {
            [id]=>Some(bridget_transport::protocol::MobileOperation::UserInputShow{request_id:(*id).into()}),
            [id,"responses",command]=>Some(bridget_transport::protocol::MobileOperation::UserInputReceipt{device_id:device.device_id.clone(),request_id:(*id).into(),command_id:(*command).into()}),
            _=>return failure("invalid_request"),
        }
    } else if path == "/v1/questions" && method == Method::GET {
        Some(bridget_transport::protocol::MobileOperation::Questions {
            after: query.after.clone(),
            limit: query.limit.unwrap_or(50) as u32,
        })
    } else if path.starts_with("/v1/questions/") && method == Method::GET {
        let parts = path
            .trim_start_matches("/v1/questions/")
            .split('/')
            .collect::<Vec<_>>();
        match parts.as_slice() {
            [item_id] => Some(bridget_transport::protocol::MobileOperation::QuestionShow {
                item_id: (*item_id).into(),
            }),
            [item_id, "responses", command_id] => {
                let Some(choice) = query.choice.clone() else {
                    return failure("invalid_request");
                };
                Some(
                    bridget_transport::protocol::MobileOperation::QuestionReceipt {
                        device_id: device.device_id.clone(),
                        item_id: (*item_id).into(),
                        command_id: (*command_id).into(),
                        choice,
                    },
                )
            }
            _ => return failure("invalid_request"),
        }
    } else if path == "/v1/tasks" && method == Method::GET {
        let Some(agent_id) = query.agent_id.clone() else {
            return failure("invalid_request");
        };
        if query.machine.is_some() || query.project_host.is_some() {
            return failure("invalid_request");
        }
        Some(bridget_transport::protocol::MobileOperation::TaskList {
            agent_id,
            after: query.after.clone(),
            limit: query.limit.unwrap_or(50) as u32,
        })
    } else if method == Method::GET && path.starts_with("/v1/tasks/") {
        let parts = path
            .trim_start_matches("/v1/tasks/")
            .split('/')
            .collect::<Vec<_>>();
        match parts.as_slice() {
            [task_id] => Some(bridget_transport::protocol::MobileOperation::TaskShow {
                task_id: (*task_id).into(),
                offset: query.offset.unwrap_or(0),
            }),
            [task_id, "cancel", command_id] => Some(
                bridget_transport::protocol::MobileOperation::TaskCancelReceipt {
                    device_id: device.device_id.clone(),
                    task_id: (*task_id).into(),
                    command_id: (*command_id).into(),
                },
            ),
            _ => return failure("invalid_request"),
        }
    } else if path == "/v1/federation" && method == Method::GET {
        Some(bridget_transport::protocol::MobileOperation::Federation {
            after: query.after.clone(),
            limit: match u16::try_from(query.limit.unwrap_or(50)) {
                Ok(v) => v,
                Err(_) => return failure("invalid_request"),
            },
        })
    } else if path.starts_with("/v1/message-queue/") {
        let agent_id = path.trim_start_matches("/v1/message-queue/").to_string();
        if agent_id.contains('/') { return failure("invalid_request"); }
        if method == Method::GET {
            Some(bridget_transport::protocol::MobileOperation::MessageQueue { agent_id })
        } else if method == Method::POST {
            match strict::<QueueChangeRequest>(bytes) {
                Ok(q) => Some(bridget_transport::protocol::MobileOperation::MessageQueueChange {
                    agent_id, command_id:q.command_id, request_id:q.request_id, change:q.action,
                    expected_body:q.expected_body, body:q.body,
                }),
                Err(response) => return response,
            }
        } else { return failure("invalid_request"); }
    } else if path == "/v1/commands" && method == Method::POST {
        match strict::<CommandRequest>(bytes) {
            Ok(CommandRequest::Message {
                device_id,
                command_id,
                issued_at,
                agent_id,
                body,
                guide_now,
            }) => Some(bridget_transport::protocol::MobileOperation::Message {
                device_id,
                command_id,
                issued_at,
                agent_id,
                body,
                guide_now,
            }),
            Ok(CommandRequest::TaskCancel {
                device_id,
                command_id,
                task_id,
            }) => Some(bridget_transport::protocol::MobileOperation::TaskCancel {
                device_id,
                command_id,
                task_id,
            }),
            Ok(CommandRequest::UserInputRespond {device_id,command_id,request_id,revision,answers})=>Some(bridget_transport::protocol::MobileOperation::UserInputRespond {device_id,command_id,request_id,revision,answers}),
            Ok(CommandRequest::QuestionRespond {
                device_id,
                command_id,
                item_id,
                choice,
            }) => Some(
                bridget_transport::protocol::MobileOperation::QuestionRespond {
                    device_id,
                    command_id,
                    item_id,
                    choice,
                },
            ),
            Ok(CommandRequest::CreateAgent) => return failure("capability_unavailable"),
            Err(response) => return response,
        }
    } else if method == Method::GET && path.starts_with("/v1/commands/") {
        Some(bridget_transport::protocol::MobileOperation::Lookup {
            command_id: path.trim_start_matches("/v1/commands/").into(),
        })
    } else {
        None
    };
    if let Some(operation) = operation {
        return match super::client::mobile_operation(&gateway.socket, key, operation) {
            Ok(value) if value.get("status").and_then(Value::as_str) == Some("ok") => result(value),
            Ok(value) => failure(
                value
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("outcome_unknown"),
            ),
            Err(code) => failure(code),
        };
    }
    failure("not_found")
}

fn follow(
    gateway: &Gateway,
    key: &str,
    device: &Device,
    query: &super::client::DirectoryQuery,
    deadline: Instant,
) -> Response {
    use bridget_transport::protocol::{HumanLineageError as E, MobileOperation};
    let _permit = match gateway.follow_permit(&device.device_id) {
        Ok(permit) => permit,
        Err(response) => return response,
    };
    let Some(agent) = query.agent_id.as_deref() else {
        return failure("invalid_request");
    };
    if uuid::Uuid::parse_str(agent).is_err() || query.after_seq.is_some() && query.after.is_some() {
        return failure("invalid_request");
    }
    let after = if let Some(cursor) = &query.after {
        if cursor.len() > 512 {
            return failure("invalid_request");
        }
        match serde_json::from_str::<FollowCursor>(cursor) {
            Ok(cursor)
                if cursor.version == 1
                    && cursor.device_id == device.device_id
                    && cursor.agent_id == agent =>
            {
                cursor.after_seq
            }
            _ => return failure("resync_required"),
        }
    } else {
        query.after_seq.unwrap_or(0)
    };
    if !device.scope.all_projects || !device.scope.all_machines {
        return failure("history_scope_unavailable");
    }
    let mut page = match super::client::mobile_operation(
        &gateway.socket,
        key,
        MobileOperation::AgentJournal {
            agent_id: agent.into(),
            after_seq: after,
            tail: false,
            limit: query.limit.unwrap_or(50) as u32,
        },
    ) {
        Ok(page) if page["status"] == "ok" => page,
        Ok(page) => return failure(page["code"].as_str().unwrap_or("journal_unavailable")),
        Err(code) => return failure(code),
    };
    let mut idle = false;
    if page["items"].as_array().is_some_and(Vec::is_empty)
        && page["gap"].is_null()
        && page["caught_up"] == true
        && Instant::now() < deadline
    {
        let agent_state=page["agent_state"].clone();
        let mut received = None;
        let mut stop = None;
        let mut timer = None;
        let store = gateway.store.clone();
        let credential = key.to_owned();
        let expected_device = device.device_id.clone();
        let native = crate::attach::lineage_journal_follow(
            &gateway.socket,
            agent,
            agent,
            page["next_seq"].as_u64().unwrap_or(after),
            query.limit.unwrap_or(50) as u32,
            |socket| {
                let socket = socket.try_clone().map_err(|_| E::JournalUnavailable)?;
                let (done, receiver) = std::sync::mpsc::sync_channel(1);
                stop = Some(done);
                let store = store.clone();
                let key = credential.clone();
                let expected = expected_device.clone();
                timer = Some(std::thread::spawn(move || {
                    loop {
                        let remaining = deadline.saturating_duration_since(Instant::now());
                        if remaining.is_zero() {
                            break;
                        }
                        match receiver.recv_timeout(remaining.min(Duration::from_secs(1))) {
                            Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                        }
                        let authorized = store
                            .lock()
                            .ok()
                            .and_then(|s| s.authenticate(&key, unix_now()).ok())
                            .is_some_and(|d| {
                                d.device_id == expected
                                    && d.scope.all_projects
                                    && d.scope.all_machines
                                    && d.scope.operations.contains(&Operation::Read)
                            });
                        if !authorized {
                            break;
                        }
                    }
                    let _ = socket.shutdown(std::net::Shutdown::Both);
                }));
                Ok(())
            },
            |value| {
                if !value["events"].as_array().is_some_and(Vec::is_empty) || !value["gap"].is_null()
                {
                    received = Some(
                        json!({"status":"ok","agent_id":agent,"items":value["events"],
                        "gap":value["gap"],"next_seq":value["next_seq"],"caught_up":value["caught_up"]}),
                    );
                    return Err(E::BindingUnavailable); // Cette page termine le cycle HTTP borné.
                }
                Ok(())
            },
        );
        if let Some(stop) = stop {
            let _ = stop.send(());
        }
        if let Some(timer) = timer {
            let _ = timer.join();
        }
        if let Some(received) = received {
            page = received;
            page["agent_state"]=agent_state;
        } else if Instant::now() >= deadline {
            idle = true;
        } else if native.is_err() {
            return recheck_or_failure(gateway, key, "journal_unavailable");
        }
    }
    match gateway.store.lock() {
        Ok(store) => match store.authenticate(key, unix_now()) {
            Ok(current)
                if current.device_id == device.device_id
                    && current.scope.operations.contains(&Operation::Read)
                    && current.scope.all_machines
                    && current.scope.all_projects => {}
            Ok(_) => return failure("forbidden_scope"),
            Err(error) => return failure(error.code()),
        },
        Err(_) => return failure("storage_unavailable"),
    }
    let cursor = FollowCursor {
        version: 1,
        device_id: device.device_id.clone(),
        agent_id: agent.into(),
        after_seq: page["next_seq"].as_u64().unwrap_or(after),
    };
    page["next_cursor"] =
        match serde_json::to_value(&cursor).and_then(|v| serde_json::to_string(&v)) {
            Ok(cursor) => json!(cursor),
            Err(_) => return failure("storage_unavailable"),
        };
    page["idle"] = json!(idle);
    result(page)
}

pub(super) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn recheck_or_failure(gateway: &Gateway, key: &str, fallback: &str) -> Response {
    match gateway.store.lock() {
        Ok(store) => match store.authenticate(key, unix_now()) {
            Ok(_) => failure(fallback),
            Err(error) => failure(error.code()),
        },
        Err(_) => failure("storage_unavailable"),
    }
}

/// Socket du maître local obligatoire ; une socket SSH ne suffit pas à déplacer l'autorité.
fn require_local_master(root: &Path, socket: &Path) -> Result<(), &'static str> {
    use crate::communication::client::DaemonConnection;
    use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
    let mut connection = DaemonConnection::connect(socket).map_err(|_| "daemon_unavailable")?;
    let response = connection
        .exchange(&WrapperToDaemon::DaemonIdentityRequest)
        .map_err(|_| "daemon_unavailable")?;
    let DaemonToWrapper::DaemonIdentityReport { db_path, host, .. } = response else {
        return Err("mobile_master_unconfirmed");
    };
    if host != crate::wrapper::host_name()
        || std::fs::canonicalize(&db_path).ok()
            != std::fs::canonicalize(root.join("bridget.db")).ok()
        || !root.join("bridget.db").is_file()
    {
        return Err("mobile_requires_local_master");
    }
    Ok(())
}

/// Connexions et durée totale bornées, y compris avant le décodage des en-têtes HTTP.
struct LimitedListener {
    inner: tokio::net::TcpListener,
    permits: Arc<Semaphore>,
}
struct LimitedStream {
    inner: tokio::net::TcpStream,
    _permit: tokio::sync::OwnedSemaphorePermit,
    deadline: std::pin::Pin<Box<tokio::time::Sleep>>,
}
impl LimitedStream {
    fn expired(&mut self, cx: &mut std::task::Context<'_>) -> bool {
        std::future::Future::poll(self.deadline.as_mut(), cx).is_ready()
    }
}
impl tokio::io::AsyncRead for LimitedStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if this.expired(cx) {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::TimedOut.into()));
        }
        std::pin::Pin::new(&mut this.inner).poll_read(cx, buf)
    }
}
impl tokio::io::AsyncWrite for LimitedStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        if this.expired(cx) {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::TimedOut.into()));
        }
        std::pin::Pin::new(&mut this.inner).poll_write(cx, buf)
    }
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        if this.expired(cx) {
            return std::task::Poll::Ready(Err(std::io::ErrorKind::TimedOut.into()));
        }
        std::pin::Pin::new(&mut this.inner).poll_flush(cx)
    }
    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
impl axum::serve::Listener for LimitedListener {
    type Io = LimitedStream;
    type Addr = std::net::SocketAddr;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let permit = self
                .permits
                .clone()
                .acquire_owned()
                .await
                .expect("listener permit");
            match self.inner.accept().await {
                Ok((inner, addr)) => {
                    return (
                        LimitedStream {
                            inner,
                            _permit: permit,
                            deadline: Box::pin(tokio::time::sleep(CONNECTION_BUDGET)),
                        },
                        addr,
                    );
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }
    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.inner.local_addr()
    }
}

pub(super) fn serve(root: &Path, socket: &Path) -> Result<(), &'static str> {
    require_local_master(root, socket)?;
    let config = Config::load(root)?.ok_or("mobile_not_configured")?;
    let gateway = Gateway {
        root: root.to_path_buf(),
        config: config.clone(),
        socket: socket.to_path_buf(),
        store: Arc::new(Mutex::new(
            Store::open(&root.join("mobile-devices.db")).map_err(StoreError::code)?,
        )),
        workers: Arc::new(Semaphore::new(MAX_BLOCKING)),
        bootstrap_rate: Arc::new(Mutex::new(BTreeMap::new())),
        renewal_rate: Arc::new(Mutex::new(BTreeMap::new())),
        followers: Arc::new(Mutex::new(BTreeMap::new())),
        follow_slots: Arc::new(Semaphore::new(MAX_BLOCKING - 2)),
        uploads: Arc::new(Semaphore::new(2)),
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(MAX_BLOCKING)
        .enable_all()
        .build()
        .map_err(|_| "mobile_runtime_unavailable")?;
    runtime.block_on(async move {
        let inner = tokio::net::TcpListener::bind(config.listen_addr)
            .await
            .map_err(|_| "mobile_listener_unavailable")?;
        let listener = LimitedListener {
            inner,
            permits: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        };
        axum::serve(listener, Router::new().fallback(handle).with_state(gateway))
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await
            .map_err(|_| "mobile_listener_failed")
    })
}

#[cfg(test)]
#[path = "http_tests.rs"]
pub(crate) mod tests;
