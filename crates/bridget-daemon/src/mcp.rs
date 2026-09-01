//! Façade MCP stdio. Le protocole daemon reste le seul transport métier.

use crate::artifact_types::{ARTIFACT_CONTRACT_VERSION, ArtifactPublicationV1, ArtifactReceiptV1};
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, GuichetDelegateMutationStatus,
    GuichetDurationClass, GuichetRegistreAddStatus, GuichetReplyPayload, IdempotencyIssue,
    LedgerScope, PresenceMode, ReviewTarget, SERVICE_CONTRACT_VERSION, ServiceCapability,
    ServiceRefusal, ServiceRequestOperation, ServiceRequestPayload, ServiceSuiteDeclaration,
    decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const DAEMON_BUDGET: Duration = Duration::from_secs(10);
const MAX_IN_FLIGHT_TOOL_CALLS: usize = 8;

/// Consigne de rejeu, point de vérité unique des sept ancrages.
///
/// Les trois formes d'`outcome_unknown` et les deux sorties du binaire disent
/// la même chose parce qu'elles disent LA MÊME CHAÎNE. Elle nomme les trois
/// invariants — un rejeu qui change le corps n'est pas une consultation mais
/// une seconde émission, que le daemon refusera en `envelope_mismatch` — et
/// affirme l'absence de doublon, sans quoi le geste reste redouté et personne
/// ne l'ose.
///
/// Provenance, pour qui voudra la vérifier : la SUBSTANCE vient de
/// `README.md` (« rejouez exactement le même corps avec les valeurs
/// affichées »), qui portait déjà le troisième invariant quand les retours ne
/// nommaient que les deux premiers. La FORMULATION est celle du mandat de
/// dégel, adaptée en ponctuation pour tenir dans un motif. Ce n'est donc pas
/// une recopie littérale du README, et il ne faut pas l'annoncer comme telle.
pub(crate) const REJEU_A_L_IDENTIQUE: &str = "rejouer à l'identique — même id, même issued_at, même corps — lit le sort réel sans jamais dupliquer";

/// Diagnostics des trois formes d'`outcome_unknown`, un par chemin.
///
/// La consigne de rejeu ne suffit pas à rendre un retour honnête : elle peut
/// être portée mot pour mot par un motif qui, juste avant, affirme un dépôt que
/// personne n'a constaté. Un motif est donc COMPOSÉ de ces constantes et de
/// rien d'autre — c'est cette forme close que les oracles verrouillent, et non
/// la seule présence de la consigne.
const DIAGNOSTIC_REMISE_EN_VOL: &str = "remise en vol — le destinataire n'a pas encore accusé";
const DIAGNOSTIC_SORT_INDETERMINE: &str = "sort indéterminé";
const DIAGNOSTIC_ACCUSE_PERDU: &str = "accusé perdu après transmission";
pub(crate) const DIAGNOSTIC_ORPHELIN: &str =
    "remise orpheline — le destinataire a été purgé ; le sort n'est pas inconnu";
/// Conduite pour `orphaned` — distincte de `REJEU_A_L_IDENTIQUE`.
/// L'état est absorbant : rejouer la même clé rend `orphaned` à nouveau.
/// Un agent qui applique le réflexe enseigné pour `in_flight` tourne en rond.
pub(crate) const CONDUITE_ORPHELIN: &str = "le rejeu à l'identique ne sert à rien — cette clé est close ; change de destinataire, ou attends son retour avec une clé neuve";

/// Statuts clients du couple dépôt-réussi / sort-inconnu.
///
/// Ils sont DIFFÉRENTS parce qu'un consommateur branche sur le champ `status`,
/// jamais sur la prose du motif. Tant qu'ils étaient confondus, un lecteur
/// prudent concluait à la panne devant un succès : un relecteur a lu deux
/// `outcome_unknown` sur le même `id`, rejeu à l'identique compris, pour un
/// message en cours d'acheminement — il en a déduit un canal cassé, et un
/// constat BLOQUANT FAUX a été gravé au registre avant rétractation.
pub(crate) const STATUT_IN_FLIGHT: &str = "in_flight";
pub(crate) const STATUT_OUTCOME_UNKNOWN: &str = "outcome_unknown";
pub(crate) const STATUT_ORPHELIN: &str = "orphaned";

/// Preuve de dépôt portée par une issue `OutcomeUnknown`, s'il y en a une.
///
/// Point de vérité UNIQUE du discriminant : le retour MCP et la sortie du
/// binaire l'appellent tous les deux, donc ils ne peuvent pas diverger. Un
/// identifiant vide n'est pas une preuve — le daemon n'en produit jamais, et
/// l'accepter annoncerait un dépôt sur une valeur qu'il refuse lui-même.
pub(crate) fn attestation_de_depot(delivery_id: Option<&str>) -> Option<&str> {
    delivery_id.filter(|delivery_id| !delivery_id.trim().is_empty())
}
static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct Session {
    initialize_seen: bool,
    initialized: bool,
}

/// Exécute le serveur MCP avec un lecteur unique et un writer unique.
///
/// L'ownership exclusif du `BufWriter` sérialise les sorties JSON-RPC : aucun
/// diagnostic ne peut rejoindre stdout, qui est réservé aux réponses MCP.
pub fn serve<R: BufRead, W: Write + Send>(mut input: R, output: W) -> io::Result<()> {
    serve_with(
        &mut input,
        output,
        &crate::mcp_identity::resolve_current_identity,
        &execute_tool,
    )
}

fn serve_with<R, W, I, E>(
    mut input: R,
    output: W,
    resolve_identity: &I,
    execute: &E,
) -> io::Result<()>
where
    R: BufRead,
    W: Write + Send,
    I: Fn() -> Result<crate::mcp_identity::ResolvedIdentity, crate::mcp_identity::IdentityError>
        + Sync,
    E: Fn(&crate::mcp_identity::ResolvedIdentity, &str, &Value) -> Result<Value, ToolError> + Sync,
{
    let output = Arc::new(Mutex::new(BufWriter::new(output)));
    let mut session = Session::default();
    let mut line = String::new();
    let in_flight = Arc::new(Mutex::new(HashSet::new()));
    let cancelled = Arc::new(Mutex::new(HashSet::new()));
    let active = Arc::new(AtomicUsize::new(0));

    std::thread::scope(|scope| -> io::Result<()> {
        loop {
            line.clear();
            if input.read_line(&mut line)? == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            let request = match serde_json::from_str::<Value>(line.trim_end()) {
                Ok(request) => request,
                Err(_) => {
                    write_shared_response(&output, &error(Value::Null, -32700, "JSON malformé"))?;
                    continue;
                }
            };
            if let Some(cancelled_id) = cancellation_id(&request) {
                let is_active = in_flight
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .contains(&cancelled_id);
                if is_active {
                    cancelled
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .insert(cancelled_id);
                }
                continue;
            }
            let call_id = asynchronous_tool_call_id(&request, &session);
            if let Some(call_id) = call_id {
                if !try_reserve_tool_call(&active) {
                    write_shared_response(
                        &output,
                        &result(
                            request.get("id").cloned().unwrap_or(Value::Null),
                            technical_result("busy", "serveur MCP saturé : réessaie plus tard"),
                        ),
                    )?;
                    continue;
                }
                in_flight
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(call_id.clone());
                let output = Arc::clone(&output);
                let in_flight = Arc::clone(&in_flight);
                let cancelled = Arc::clone(&cancelled);
                let active = Arc::clone(&active);
                scope.spawn(move || {
                    let mut tool_session = Session {
                        initialize_seen: true,
                        initialized: true,
                    };
                    let response = dispatch_with_executor(
                        &request,
                        &mut tool_session,
                        resolve_identity,
                        execute,
                    );
                    let cancelled_call = cancelled
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .remove(&call_id);
                    in_flight
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .remove(&call_id);
                    active.fetch_sub(1, Ordering::AcqRel);
                    if !cancelled_call && let Some(response) = response {
                        let _ = write_shared_response(&output, &response);
                    }
                });
                continue;
            }
            if let Some(response) =
                dispatch_with_executor(&request, &mut session, resolve_identity, execute)
            {
                write_shared_response(&output, &response)?;
            }
        }
        Ok(())
    })?;
    output
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .flush()
}

/// Lance la façade MCP depuis la sous-commande `bridget mcp`.
pub fn run_stdio() -> io::Result<()> {
    serve(BufReader::new(io::stdin().lock()), io::stdout())
}

fn write_response(output: &mut impl Write, response: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, response).map_err(io::Error::other)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn write_shared_response(
    output: &Arc<Mutex<BufWriter<impl Write>>>,
    response: &Value,
) -> io::Result<()> {
    let mut writer = output.lock().unwrap_or_else(|error| error.into_inner());
    write_response(&mut *writer, response)
}

fn cancellation_id(request: &Value) -> Option<String> {
    (request.get("method").and_then(Value::as_str) == Some("notifications/cancelled"))
        .then(|| {
            request
                .get("params")?
                .get("requestId")
                .map(Value::to_string)
        })
        .flatten()
}

fn asynchronous_tool_call_id(request: &Value, session: &Session) -> Option<String> {
    (session.initialized && request.get("method").and_then(Value::as_str) == Some("tools/call"))
        .then(|| request.get("id").map(Value::to_string))
        .flatten()
}

fn try_reserve_tool_call(active: &AtomicUsize) -> bool {
    let mut observed = active.load(Ordering::Acquire);
    loop {
        if observed >= MAX_IN_FLIGHT_TOOL_CALLS {
            return false;
        }
        match active.compare_exchange_weak(
            observed,
            observed + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(current) => observed = current,
        }
    }
}

#[cfg(test)]
fn dispatch_with_identity(
    request: &Value,
    session: &mut Session,
    resolve_identity: &impl Fn() -> Result<
        crate::mcp_identity::ResolvedIdentity,
        crate::mcp_identity::IdentityError,
    >,
) -> Option<Value> {
    dispatch_with_executor(request, session, resolve_identity, &execute_tool)
}

fn dispatch_with_executor(
    request: &Value,
    session: &mut Session,
    resolve_identity: &impl Fn() -> Result<
        crate::mcp_identity::ResolvedIdentity,
        crate::mcp_identity::IdentityError,
    >,
    execute: &impl Fn(&crate::mcp_identity::ResolvedIdentity, &str, &Value) -> Result<Value, ToolError>,
) -> Option<Value> {
    let Some(object) = request.as_object() else {
        return Some(error(Value::Null, -32600, "requête JSON-RPC invalide"));
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(error(Value::Null, -32600, "version JSON-RPC invalide"));
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Some(error(Value::Null, -32600, "méthode JSON-RPC absente"));
    };
    let id = match object.get("id") {
        None => None,
        Some(Value::String(_) | Value::Number(_)) => object.get("id").cloned(),
        Some(_) => return Some(error(Value::Null, -32600, "identifiant JSON-RPC invalide")),
    };
    let params = object.get("params");

    match method {
        "initialize" => {
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres initialize invalides"));
            }
            session.initialize_seen = true;
            id.map(|id| result(id, initialize_result()))
        }
        "notifications/initialized" => {
            if session.initialize_seen {
                session.initialized = true;
            } else {
                eprintln!("bridget mcp: notification initialized reçue avant initialize");
            }
            None
        }
        "notifications/cancelled" => None,
        "tools/list" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres tools/list invalides"));
            }
            id.map(|id| result(id, json!({ "tools": tools() })))
        }
        "ping" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres ping invalides"));
            }
            id.map(|id| result(id, json!({})))
        }
        "tools/call" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            let Some(params) = params.and_then(Value::as_object) else {
                return id.map(|id| error(id, -32602, "paramètres tools/call invalides"));
            };
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return id.map(|id| error(id, -32602, "nom d'outil absent"));
            };
            if !tools().iter().any(|tool| tool["name"] == name) {
                return id.map(|id| error(id, -32602, "outil inconnu"));
            }
            let identity = match resolve_identity() {
                Ok(identity) => identity,
                Err(identity_error) => {
                    return id.map(|id| identity_error_result(id, &identity_error));
                }
            };
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            id.map(|id| match execute(&identity, name, &arguments) {
                Ok(payload) => result(id, tool_result(payload)),
                Err(ToolError::InvalidParams(message)) => error(id, -32602, &message),
                Err(ToolError::Technical { code, message }) => {
                    result(id, technical_result(code, &message))
                }
            })
        }
        _ => id.map(|id| error(id, -32601, &format!("méthode inconnue: {method}"))),
    }
}

fn require_initialized(id: Option<Value>, session: &Session) -> Option<Value> {
    (!session.initialized)
        .then(|| id.map(|id| error(id, -32002, "initialisation MCP requise")))
        .flatten()
}

fn result(id: Value, payload: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": payload })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn identity_error_result(id: Value, identity_error: &crate::mcp_identity::IdentityError) -> Value {
    result(
        id,
        json!({
            "content": [{
                "type": "text",
                "text": format!("{}: {}", identity_error.code(), identity_error.remediation())
            }],
            "isError": true,
            "code": identity_error.code()
        }),
    )
}

#[derive(Debug)]
enum ToolError {
    InvalidParams(String),
    Technical { code: &'static str, message: String },
}

struct DaemonConnection {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
    deadline: Instant,
}

impl DaemonConnection {
    fn connect(socket: &Path) -> Result<Self, ToolError> {
        let deadline = Instant::now() + DAEMON_BUDGET;
        let stream =
            connect_nonblocking(socket, deadline).map_err(|error| ToolError::Technical {
                code: "daemon_unreachable",
                message: format!("daemon Bridget injoignable : {error}"),
            })?;
        let reader_stream = stream.try_clone().map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("impossible de dupliquer le socket daemon : {error}"),
        })?;
        Ok(Self {
            writer: BufWriter::new(stream),
            reader: BufReader::new(reader_stream),
            deadline,
        })
    }

    fn exchange(&mut self, command: &WrapperToDaemon) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ToolError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer.flush().map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("flush daemon impossible : {error}"),
        })?;
        self.read_response("daemon_unreachable")
    }

    fn send_then_wait(&mut self, command: &WrapperToDaemon) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ToolError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ToolError::Technical {
            code: "outcome_unknown",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer.flush().map_err(|error| ToolError::Technical {
            code: "outcome_unknown",
            message: format!("flush daemon impossible : {error}"),
        })?;
        self.read_response("outcome_unknown")
    }

    fn read_response(&mut self, failure_code: &'static str) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout(failure_code)?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|error| ToolError::Technical {
                code: failure_code,
                message: format!("réponse daemon indisponible : {error}"),
            })?;
        if line.is_empty() {
            return Err(ToolError::Technical {
                code: failure_code,
                message: "daemon Bridget a fermé la connexion sans réponse".to_string(),
            });
        }
        decode(line.trim_end()).map_err(|error| ToolError::Technical {
            code: failure_code,
            message: format!("réponse daemon invalide : {error}"),
        })
    }

    fn apply_remaining_timeout(&self, code: &'static str) -> Result<(), ToolError> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| ToolError::Technical {
                code,
                message: "budget total de 10 s dépassé".to_string(),
            })?;
        self.writer
            .get_ref()
            .set_write_timeout(Some(remaining))
            .map_err(|error| ToolError::Technical {
                code,
                message: format!("impossible de borner l'écriture daemon : {error}"),
            })?;
        self.reader
            .get_ref()
            .set_read_timeout(Some(remaining))
            .map_err(|error| ToolError::Technical {
                code,
                message: format!("impossible de borner la lecture daemon : {error}"),
            })
    }
}

fn connect_nonblocking(socket: &Path, deadline: Instant) -> io::Result<UnixStream> {
    if deadline <= Instant::now() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "budget connexion dépassé",
        ));
    }
    let path = socket.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "socket Unix trop long",
        ));
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        address.sun_len = (std::mem::size_of::<libc::sa_family_t>() + path.len() + 1) as u8;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            path.as_ptr().cast(),
            address.sun_path.as_mut_ptr(),
            path.len(),
        );
    }
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let close = |fd: libc::c_int| unsafe { libc::close(fd) };
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        close(fd);
        return Err(io::Error::last_os_error());
    }
    let result = unsafe {
        libc::connect(
            fd,
            (&address as *const libc::sockaddr_un).cast(),
            std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            close(fd);
            return Err(error);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "budget connexion dépassé"))?;
        let timeout = remaining.as_millis().min(i32::MAX as u128) as libc::c_int;
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };
        if unsafe { libc::poll(&mut pollfd, 1, timeout) } <= 0 {
            close(fd);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "connexion daemon expirée",
            ));
        }
        let mut so_error: libc::c_int = 0;
        let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut so_error as *mut libc::c_int).cast(),
                &mut length,
            )
        } < 0
            || so_error != 0
        {
            close(fd);
            return Err(io::Error::from_raw_os_error(so_error));
        }
    }
    let stream = unsafe { UnixStream::from_raw_fd(fd) };
    stream.set_nonblocking(false)?;
    Ok(stream)
}

fn execute_tool(
    identity: &crate::mcp_identity::ResolvedIdentity,
    name: &str,
    arguments: &Value,
) -> Result<Value, ToolError> {
    let arguments = arguments
        .as_object()
        .ok_or_else(|| ToolError::InvalidParams("arguments d'outil invalides".to_string()))?;
    let socket = crate::daemon::DaemonConfig::default().socket_path;
    execute_tool_at_with_scope(
        &identity.name,
        &identity.instance_id,
        name,
        arguments,
        &socket,
    )
}

#[cfg(test)]
fn execute_tool_at(
    identity: &str,
    name: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    execute_tool_at_with_scope(identity, "test-instance", name, arguments, socket)
}

fn execute_tool_at_with_scope(
    identity: &str,
    instance_id: &str,
    name: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    match name {
        "bridget_send" => execute_send(identity, instance_id, arguments, socket),
        "bridget_publish_artifact" => {
            execute_publish_artifact(identity, instance_id, arguments, socket)
        }
        "bridget_who" => execute_who(identity, instance_id, arguments, socket),
        "bridget_ledger" => execute_ledger(identity, instance_id, arguments, socket),
        "maicie_delegate" => execute_maicie_delegate(identity, instance_id, arguments, socket),
        "maicie_registre_add" => {
            execute_maicie_registre_add(identity, instance_id, arguments, socket)
        }
        "maicie_objective_close" => {
            execute_maicie_objective_close(identity, instance_id, arguments, socket)
        }
        "maicie_request_status" => execute_maicie_request_status(instance_id, arguments, socket),
        _ => Err(ToolError::InvalidParams("outil inconnu".to_string())),
    }
}

fn execute_publish_artifact(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    let publication: ArtifactPublicationV1 =
        serde_json::from_value(Value::Object(arguments.clone())).map_err(|error| {
            ToolError::InvalidParams(format!("contrat d'artefact invalide : {error}"))
        })?;
    let canonical_publication = publication.canonical_bytes();
    let mut connection = registered_connection(identity, instance_id, socket)?;
    match connection.exchange(&WrapperToDaemon::ArtifactPublish {
        contract_version: ARTIFACT_CONTRACT_VERSION,
        canonical_publication,
    })? {
        DaemonToWrapper::ArtifactPublicationResult {
            contract_version,
            replayed,
            receipt_json: Some(receipt_json),
            refusal_code: None,
            refusal_message: None,
        } if contract_version == ARTIFACT_CONTRACT_VERSION => {
            let receipt: ArtifactReceiptV1 =
                serde_json::from_slice(&receipt_json).map_err(|_| ToolError::Technical {
                    code: "invalid_artifact_receipt",
                    message: "Bridget a renvoyé un reçu d'artefact illisible".to_string(),
                })?;
            Ok(json!({
                "status": if replayed { "replayed" } else { "published" },
                "receipt": receipt,
            }))
        }
        DaemonToWrapper::ArtifactPublicationResult {
            contract_version,
            refusal_code: Some(code),
            refusal_message: Some(message),
            ..
        } if contract_version == ARTIFACT_CONTRACT_VERSION => {
            Ok(json!({ "status": "refused", "code": code, "message": message }))
        }
        DaemonToWrapper::ArtifactPublicationResult { .. } => Err(ToolError::Technical {
            code: "artifact_protocol",
            message: "Bridget a renvoyé une issue d'artefact incohérente".to_string(),
        }),
        other => unexpected_response(other),
    }
}

fn execute_send(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(
        arguments,
        &[
            "to",
            "body",
            "reply",
            "reply_timeout",
            "in_reply_to",
            "id",
            "issued_at",
        ],
    )?;
    let to = required_non_empty_string(arguments, "to")?;
    let body = required_non_empty_string(arguments, "body")?;
    let reply = optional_bool(arguments, "reply")?.unwrap_or(false);
    let reply_timeout = optional_positive_u64(arguments, "reply_timeout")?;
    let in_reply_to = optional_non_empty_string(arguments, "in_reply_to")?;
    if !reply && reply_timeout.is_some() {
        return Err(ToolError::InvalidParams(
            "reply_timeout est réservé à reply=true".to_string(),
        ));
    }
    let supplied_id = arguments.get("id");
    let supplied_issued_at = arguments.get("issued_at");
    if supplied_id.is_some() != supplied_issued_at.is_some() {
        return Err(ToolError::InvalidParams(
            "id et issued_at doivent être fournis ensemble pour un retry".to_string(),
        ));
    }
    let id = match supplied_id {
        Some(value) => value
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                ToolError::InvalidParams("id doit être une chaîne non vide".to_string())
            })?,
        None => new_message_id(),
    };
    let issued_at = match supplied_issued_at {
        Some(value) => value.as_i64().filter(|value| *value > 0).ok_or_else(|| {
            ToolError::InvalidParams("issued_at doit être un entier positif".to_string())
        })?,
        None => now_secs(),
    };
    let mut message = BridgetMessage::new(identity, to, body);
    message.id = id.clone();
    message.reply = reply;
    message.reply_timeout = reply_timeout;
    message.in_reply_to = in_reply_to;
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        other => return unexpected_response(other),
    }
    let issuer_scope = issuer_scope(instance_id);
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope,
        capabilities: vec![ClientCapability::SendIdempotent],
    })? {
        DaemonToWrapper::ClientWelcome {
            capabilities,
            build_id,
            ..
        } if capabilities.contains(&ClientCapability::SendIdempotent) => {
            if let Some(warning) = crate::build_info::stale_daemon_warning(&build_id) {
                eprintln!("{warning}");
            }
        }
        DaemonToWrapper::ClientRejected { reason } => {
            return Err(ToolError::Technical {
                code: "daemon_protocol",
                message: format!("négociation client refusée : {reason:?}"),
            });
        }
        other => return unexpected_response(other),
    }
    match connection.send_then_wait(&WrapperToDaemon::SendIdempotent {
        message,
        message_id: id.clone(),
        issued_at,
    }) {
        Ok(DaemonToWrapper::IdempotencyResult { issue, .. }) => {
            Ok(send_issue_result(&id, issued_at, issue))
        }
        Ok(other) => unexpected_response(other),
        Err(ToolError::Technical {
            code: "outcome_unknown",
            message,
        }) => Ok(json!({
            "status": "outcome_unknown",
            "id": id,
            "issued_at": issued_at,
            "reason": format!("{DIAGNOSTIC_ACCUSE_PERDU} — {REJEU_A_L_IDENTIQUE} ({message})")
        })),
        Err(error) => Err(error),
    }
}

fn execute_who(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["domain"])?;
    let domain = match arguments.get("domain") {
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| ToolError::InvalidParams("domain doit être une chaîne".to_string()))?
                .to_string(),
        ),
        None => None,
    };
    let mut connection = registered_connection(identity, instance_id, socket)?;
    match connection.exchange(&WrapperToDaemon::ListAgents)? {
        DaemonToWrapper::AgentList { agents } => Ok(json!({
            "agents": agents.into_iter().filter(|agent| {
                domain.as_deref().is_none_or(|wanted| agent.domain.as_deref() == Some(wanted))
            }).collect::<Vec<_>>()
        })),
        other => unexpected_response(other),
    }
}

fn execute_ledger(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["view", "limit", "requests_scope"])?;
    let view = arguments
        .get("view")
        .and_then(Value::as_str)
        .unwrap_or("both");
    let scope = match view {
        "messages" => LedgerScope::Messages,
        "requests" => LedgerScope::Requests,
        "both" => LedgerScope::Both,
        _ => {
            return Err(ToolError::InvalidParams(
                "view doit valoir messages, requests ou both".to_string(),
            ));
        }
    };
    let requests_scope = parse_requests_scope(arguments)?;
    let limit = match arguments.get("limit") {
        Some(value) => value
            .as_u64()
            .filter(|value| (1..=200).contains(value))
            .ok_or_else(|| {
                ToolError::InvalidParams("limit doit être compris entre 1 et 200".to_string())
            })? as u16,
        None => 20,
    };
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let mut messages = Vec::new();
    let mut requests = Vec::new();
    if matches!(scope, LedgerScope::Messages | LedgerScope::Both) {
        match connection.exchange(&WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Messages,
            limit,
        })? {
            DaemonToWrapper::LedgerProjection {
                messages: result, ..
            } => messages = result,
            other => return unexpected_response(other),
        }
    }
    if matches!(scope, LedgerScope::Requests | LedgerScope::Both) {
        requests = fetch_ledger_requests(&mut connection, identity, requests_scope, limit)?;
    }
    Ok(json!({
        "messages": messages.into_iter().map(ledger_message_dto).collect::<Vec<_>>(),
        "requests": requests.into_iter().map(request_dto).collect::<Vec<_>>(),
        "requests_scope": match requests_scope {
            RequestsScope::Mine => "mine",
            RequestsScope::All => "all",
        },
    }))
}

fn execute_maicie_delegate(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(
        arguments,
        &[
            "goal",
            "explicit_target",
            "required_tags",
            "duration",
            "suite_objective_id",
            "depends_on",
            "references",
            "review_ref",
            "expected_head",
            "request_id",
            "issued_at",
        ],
    )?;
    let goal = required_non_empty_string(arguments, "goal")?;
    let explicit_target = optional_non_empty_string(arguments, "explicit_target")?;
    let required_tags = optional_string_array(arguments, "required_tags", 32)?;
    let depends_on = optional_string_array(arguments, "depends_on", 100)?;
    let references = optional_string_array(arguments, "references", 100)?;
    let review_target = optional_review_target(arguments)?;
    let duration = match arguments
        .get("duration")
        .and_then(Value::as_str)
        .unwrap_or("normale")
    {
        "courte" => GuichetDurationClass::Courte,
        "normale" => GuichetDurationClass::Normale,
        "longue" => GuichetDurationClass::Longue,
        _ => {
            return Err(ToolError::InvalidParams(
                "duration doit valoir courte, normale ou longue".to_string(),
            ));
        }
    };
    let suite = optional_non_empty_string(arguments, "suite_objective_id")?
        .map_or(ServiceSuiteDeclaration::Aucune, |objective_id| {
            ServiceSuiteDeclaration::Objectif { objective_id }
        });
    execute_maicie_mutation(
        identity,
        instance_id,
        arguments,
        socket,
        ServiceRequestOperation::Delegate,
        ServiceRequestPayload::Delegate {
            goal,
            review_target,
            explicit_target,
            required_tags,
            duration,
            suite,
            depends_on,
            references,
        },
    )
}

fn execute_maicie_registre_add(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["line", "request_id", "issued_at"])?;
    execute_maicie_mutation(
        identity,
        instance_id,
        arguments,
        socket,
        ServiceRequestOperation::RegistreAdd,
        ServiceRequestPayload::RegistreAdd {
            line: required_non_empty_string(arguments, "line")?,
        },
    )
}

fn execute_maicie_objective_close(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(
        arguments,
        &["objective_id", "reason", "request_id", "issued_at"],
    )?;
    execute_maicie_mutation(
        identity,
        instance_id,
        arguments,
        socket,
        ServiceRequestOperation::ObjectiveClose,
        ServiceRequestPayload::ObjectiveClose {
            objective_id: required_non_empty_string(arguments, "objective_id")?,
            reason: required_non_empty_string(arguments, "reason")?,
        },
    )
}

fn execute_maicie_mutation(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
    operation: ServiceRequestOperation,
    payload: ServiceRequestPayload,
) -> Result<Value, ToolError> {
    let (request_id, issued_at) = mutation_coordinates(arguments)?;
    let scope = issuer_scope(instance_id);
    let mut connection = registered_connection(identity, instance_id, socket)?;
    let version = payload.required_contract_version();
    let request = WrapperToDaemon::ServiceRequest {
        version,
        issuer_scope: scope.clone(),
        request_id: request_id.clone(),
        issued_at,
        from: identity.to_string(),
        to: "maicie".to_string(),
        operation,
        payload,
    };
    match connection.send_then_wait(&request) {
        Ok(DaemonToWrapper::GuichetResult {
            issuer_scope,
            request_id: returned_request_id,
            issue,
            payload,
            ..
        }) if issuer_scope == scope && returned_request_id == request_id => {
            guichet_result_value(&request_id, Some(issued_at), &scope, &issue, payload)
        }
        Ok(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::GreffeAuthorizationDenied,
        }) => Err(ToolError::Technical {
            code: "authorization_denied",
            message: "mutation du greffe refusée".to_string(),
        }),
        Ok(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::UnsupportedVersion { .. },
        }) => Err(ToolError::Technical {
            code: "unsupported_version",
            message: "le daemon ne prend pas en charge ce contrat de délégation".to_string(),
        }),
        Ok(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CanonicalBytesMismatch,
        }) => Err(ToolError::Technical {
            code: "canonical_bytes_mismatch",
            message: "le daemon a refusé une extension de requête qu'il ne conserve pas"
                .to_string(),
        }),
        Ok(response) => unexpected_response(response),
        Err(ToolError::Technical {
            code: "outcome_unknown",
            message,
        }) => Ok(json!({
            "status": "outcome_unknown",
            "terminal": false,
            "applied": false,
            "request_id": request_id,
            "issued_at": issued_at,
            "issuer_scope": scope,
            "reason": message,
            "next": "maicie_request_status"
        })),
        Err(error) => Err(error),
    }
}

fn execute_maicie_request_status(
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["request_id"])?;
    let request_id = required_non_empty_string(arguments, "request_id")?;
    let scope = issuer_scope(instance_id);
    let mut connection = guichet_lookup_connection(&scope, socket)?;
    match connection.exchange(&WrapperToDaemon::GuichetLookup {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: scope.clone(),
        request_id: request_id.clone(),
    })? {
        DaemonToWrapper::GuichetResult {
            issuer_scope,
            request_id: returned_request_id,
            issue,
            payload,
            ..
        } if issuer_scope == scope && returned_request_id == request_id => {
            guichet_result_value(&request_id, None, &scope, &issue, payload)
        }
        response => unexpected_response(response),
    }
}

fn mutation_coordinates(
    arguments: &serde_json::Map<String, Value>,
) -> Result<(String, i64), ToolError> {
    let request_id = arguments.get("request_id");
    let issued_at = arguments.get("issued_at");
    if request_id.is_some() != issued_at.is_some() {
        return Err(ToolError::InvalidParams(
            "request_id et issued_at doivent être fournis ensemble pour un retry".to_string(),
        ));
    }
    match (request_id, issued_at) {
        (Some(request_id), Some(issued_at)) => {
            let request_id = request_id
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    ToolError::InvalidParams("request_id doit être une chaîne non vide".to_string())
                })?
                .to_string();
            let issued_at = issued_at
                .as_i64()
                .filter(|value| *value > 0)
                .ok_or_else(|| {
                    ToolError::InvalidParams("issued_at doit être un entier positif".to_string())
                })?;
            Ok((request_id, issued_at))
        }
        (None, None) => Ok((new_message_id(), now_secs())),
        _ => unreachable!("présence contrôlée ensemble"),
    }
}

fn optional_string_array(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<Vec<String>, ToolError> {
    let Some(value) = arguments.get(key) else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .filter(|values| values.len() <= max)
        .ok_or_else(|| {
            ToolError::InvalidParams(format!(
                "{key} doit être un tableau de {max} éléments au plus"
            ))
        })?;
    let mut unique = HashSet::new();
    values
        .iter()
        .map(|value| {
            let value = value
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    ToolError::InvalidParams(format!("{key} doit contenir des chaînes non vides"))
                })?
                .to_string();
            if !unique.insert(value.clone()) {
                return Err(ToolError::InvalidParams(format!(
                    "{key} contient une valeur dupliquée"
                )));
            }
            Ok(value)
        })
        .collect()
}

fn guichet_lookup_connection(
    issuer_scope: &str,
    socket: &Path,
) -> Result<DaemonConnection, ToolError> {
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Service,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Service,
        } => {}
        response => return unexpected_response(response),
    }
    match connection.exchange(&WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "maicie".to_string(),
        issuer_scope: issuer_scope.to_string(),
        capabilities: vec![ServiceCapability::MaicieGuichet],
    })? {
        DaemonToWrapper::ServiceWelcome { capabilities, .. }
            if capabilities.contains(&ServiceCapability::MaicieGuichet) =>
        {
            Ok(connection)
        }
        response => unexpected_response(response),
    }
}

fn guichet_result_value(
    request_id: &str,
    issued_at: Option<i64>,
    issuer_scope: &str,
    issue: &str,
    payload: Option<GuichetReplyPayload>,
) -> Result<Value, ToolError> {
    if let Some(payload) = payload {
        let mut result = serde_json::to_value(&payload).map_err(|error| ToolError::Technical {
            code: "daemon_protocol",
            message: format!("payload terminal non sérialisable : {error}"),
        })?;
        let Value::Object(fields) = &mut result else {
            return Err(ToolError::Technical {
                code: "daemon_protocol",
                message: "payload terminal non structuré".to_string(),
            });
        };
        let status = match &payload {
            GuichetReplyPayload::Delegate {
                status: GuichetDelegateMutationStatus::Created,
                ..
            } => "created",
            GuichetReplyPayload::Delegate {
                status: GuichetDelegateMutationStatus::SelectionRequired,
                ..
            } => "selection_required",
            GuichetReplyPayload::RegistreAdd {
                status: GuichetRegistreAddStatus::Appended,
                ..
            } => "appended",
            GuichetReplyPayload::RegistreAdd {
                status: GuichetRegistreAddStatus::IdempotentNoop,
                ..
            } => "idempotent_noop",
            GuichetReplyPayload::ObjectiveClose { .. } => "closed",
            GuichetReplyPayload::Refused { .. } => "refused",
            GuichetReplyPayload::DeliveryReport { .. }
            | GuichetReplyPayload::MissionStatus { .. }
            | GuichetReplyPayload::DeadlineQuestion { .. } => "terminal",
        };
        let applied = matches!(
            &payload,
            GuichetReplyPayload::Delegate {
                status: GuichetDelegateMutationStatus::Created,
                ..
            } | GuichetReplyPayload::RegistreAdd { .. }
                | GuichetReplyPayload::ObjectiveClose { .. }
        ) && issue == "accepted";
        fields.insert("status".to_string(), json!(status));
        fields.insert("terminal".to_string(), json!(true));
        fields.insert("applied".to_string(), json!(applied));
        fields.insert("issue".to_string(), json!(issue));
        fields.insert("request_id".to_string(), json!(request_id));
        fields.insert("issuer_scope".to_string(), json!(issuer_scope));
        if let Some(issued_at) = issued_at {
            fields.insert("issued_at".to_string(), json!(issued_at));
        }
        return Ok(result);
    }
    if matches!(
        issue,
        "accepted" | "refused" | "request_already_terminal" | "recipient_unavailable"
    ) {
        return Err(ToolError::Technical {
            code: "terminal_payload_missing",
            message: "issue terminale sans résultat métier relu du maître".to_string(),
        });
    }
    let pending = matches!(issue, "queued" | "outcome_unknown");
    Ok(json!({
        "status": issue,
        "terminal": !pending,
        "applied": false,
        "request_id": request_id,
        "issued_at": issued_at,
        "issuer_scope": issuer_scope,
        "next": pending.then_some("maicie_request_status")
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestsScope {
    Mine,
    All,
}

fn parse_requests_scope(
    arguments: &serde_json::Map<String, Value>,
) -> Result<RequestsScope, ToolError> {
    match arguments
        .get("requests_scope")
        .and_then(Value::as_str)
        .unwrap_or("mine")
    {
        "mine" => Ok(RequestsScope::Mine),
        "all" => Ok(RequestsScope::All),
        _ => Err(ToolError::InvalidParams(
            "requests_scope doit valoir mine ou all".to_string(),
        )),
    }
}

fn fetch_ledger_requests(
    connection: &mut DaemonConnection,
    identity: &str,
    scope: RequestsScope,
    limit: u16,
) -> Result<Vec<bridget_transport::protocol::RequestInfo>, ToolError> {
    match scope {
        RequestsScope::Mine => match connection.exchange(&WrapperToDaemon::ListRequests {
            sender: identity.to_string(),
            limit,
        })? {
            DaemonToWrapper::RequestList { requests } => Ok(requests),
            other => unexpected_response(other),
        },
        RequestsScope::All => match connection.exchange(&WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Requests,
            limit,
        })? {
            DaemonToWrapper::LedgerProjection { requests, .. } => Ok(requests),
            other => unexpected_response(other),
        },
    }
}

fn registered_connection(
    identity: &str,
    instance_id: &str,
    socket: &Path,
) -> Result<DaemonConnection, ToolError> {
    let mut connection = DaemonConnection::connect(socket)?;
    let registration = WrapperToDaemon::Register {
        agent_type: "mcp".to_string(),
        identity_version: 2,
        agent_id: identity.to_string(),
        host: None,
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: Some(instance_id.to_string()),
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    };
    match connection.exchange(&registration)? {
        DaemonToWrapper::Registered { .. } => Ok(connection),
        other => unexpected_response(other),
    }
}

fn send_issue_result(id: &str, issued_at: i64, issue: IdempotencyIssue) -> Value {
    match issue {
        IdempotencyIssue::Accepted { .. } => {
            json!({ "status": "accepted", "id": id, "issued_at": issued_at, "hops": 4 })
        }
        IdempotencyIssue::Rejected {
            category, reason, ..
        } => {
            json!({ "status": public_refusal_category(&category), "id": id, "issued_at": issued_at, "reason": reason })
        }
        // Le daemon répond AVANT l'accusé du destinataire : sur un premier envoi
        // nominal, l'issue est donc toujours `OutcomeUnknown`. Un `delivery_id`
        // atteste que la remise est en vol — rien n'est perdu. Sans lui, le sort
        // est réellement indéterminé.
        //
        // Les deux cas portent des STATUTS DIFFÉRENTS, pas seulement des motifs
        // différents : le lecteur d'un retour MCP branche sur `status`, jamais
        // sur la prose. Tant que le cas nominal s'annonçait `outcome_unknown`,
        // chaque agent revérifiait le ledger à la main — c'est le défaut mesuré
        // sur ~100 envois d'une seule journée.
        IdempotencyIssue::OutcomeUnknown { delivery_id, .. } => {
            match attestation_de_depot(delivery_id.as_deref()) {
                Some(delivery_id) => json!({
                    "status": STATUT_IN_FLIGHT,
                    "id": id,
                    "issued_at": issued_at,
                    "delivery_id": delivery_id,
                    "reason": format!("{DIAGNOSTIC_REMISE_EN_VOL} ; {REJEU_A_L_IDENTIQUE}")
                }),
                None => json!({
                    "status": STATUT_OUTCOME_UNKNOWN,
                    "id": id,
                    "issued_at": issued_at,
                    "reason": format!("{DIAGNOSTIC_SORT_INDETERMINE} ; {REJEU_A_L_IDENTIQUE}")
                }),
            }
        }
        IdempotencyIssue::Orphaned {
            delivery_id,
            reason,
            ..
        } => json!({
            "status": STATUT_ORPHELIN,
            "id": id,
            "issued_at": issued_at,
            "delivery_id": delivery_id,
            "reason": format!("{DIAGNOSTIC_ORPHELIN} ; {CONDUITE_ORPHELIN} ({reason})")
        }),
        IdempotencyIssue::EnvelopeMismatch => json!({
            "status": "envelope_mismatch",
            "id": id,
            "issued_at": issued_at,
            "reason": "enveloppe différente pour le même id"
        }),
        IdempotencyIssue::IdempotencyExpired => json!({
            "status": "idempotency_expired",
            "id": id,
            "issued_at": issued_at,
            "reason": "clé d'idempotence expirée"
        }),
        IdempotencyIssue::InvalidIssuedAt => json!({
            "status": "invalid_issued_at",
            "id": id,
            "issued_at": issued_at,
            "reason": "horodatage d'émission invalide"
        }),
    }
}

pub(crate) fn public_refusal_category(category: &str) -> &str {
    match category {
        "dnd" | "circuit_breaker" | "hops_exhausted" | "queue_full" => category,
        "duplicate_content" | "quarantined" => "duplicate",
        "routing" | "recipient_unavailable" => "unknown_recipient",
        "reply_sender_unavailable" => "reply_requires_agent",
        _ => category,
    }
}

fn tool_result(payload: Value) -> Value {
    let text = serde_json::to_string(&payload).expect("un résultat MCP est toujours sérialisable");
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": payload
    })
}

fn technical_result(code: &str, message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "code": code
    })
}

fn unexpected_response<T>(response: DaemonToWrapper) -> Result<T, ToolError> {
    Err(ToolError::Technical {
        code: "daemon_protocol",
        message: format!("réponse daemon inattendue : {response:?}"),
    })
}

fn reject_unknown_arguments(
    arguments: &serde_json::Map<String, Value>,
    allowed: &[&str],
) -> Result<(), ToolError> {
    if let Some(unknown) = arguments
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
    {
        return Err(ToolError::InvalidParams(format!(
            "argument inconnu : {unknown}"
        )));
    }
    Ok(())
}

fn required_non_empty_string(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, ToolError> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ToolError::InvalidParams(format!("{key} doit être une chaîne non vide")))
}

fn optional_non_empty_string(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    ToolError::InvalidParams(format!("{key} doit être une chaîne non vide"))
                })
        })
        .transpose()
}

fn optional_review_target(
    arguments: &serde_json::Map<String, Value>,
) -> Result<Option<ReviewTarget>, ToolError> {
    let review_ref = optional_non_empty_string(arguments, "review_ref")?;
    let expected_head = optional_non_empty_string(arguments, "expected_head")?;
    let target = match (review_ref, expected_head) {
        (None, None) => return Ok(None),
        (Some(target_ref), Some(expected_head)) => ReviewTarget {
            target_ref,
            expected_head,
        },
        _ => {
            return Err(ToolError::InvalidParams(
                "review_ref et expected_head doivent être fournis ensemble".to_string(),
            ));
        }
    };
    if !target.is_valid() {
        return Err(ToolError::InvalidParams(
            "review_ref attend <remote>/<branche> valide et expected_head exactement 40 hexadécimaux minuscules"
                .to_string(),
        ));
    }
    Ok(Some(target))
}

fn optional_bool(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| ToolError::InvalidParams(format!("{key} doit être booléen")))
        })
        .transpose()
}

fn optional_positive_u64(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<u64>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value.as_u64().filter(|value| *value > 0).ok_or_else(|| {
                ToolError::InvalidParams(format!("{key} doit être un entier positif"))
            })
        })
        .transpose()
}

fn new_message_id() -> String {
    let sequence = NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
    format!("mcp-{}-{:x}-{:x}", std::process::id(), now_secs(), sequence)
}

fn ledger_message_dto(message: bridget_transport::protocol::LedgerMessage) -> Value {
    let mut payload = json!({
        "id": message.id,
        "from": message.sender,
        "to": message.target,
        "body": message.body,
        "ts": message.ts,
    });
    if let Some(status) = message.delivery_status {
        payload["delivery_status"] = json!(status);
    }
    payload
}

fn request_dto(request: bridget_transport::protocol::RequestInfo) -> Value {
    json!({
        "id": request.id,
        "from": request.sender,
        "to": request.target,
        "state": request.state,
        "deadline": request.deadline_at,
        "created": request.created_at,
    })
}

pub(crate) fn issuer_scope(identity: &str) -> String {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x9e3779b97f4a7c15_u64;
    for byte in identity.bytes() {
        first = (first ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        second = second.rotate_left(5) ^ u64::from(byte);
        second = second.wrapping_mul(0x9e3779b185ebca87);
    }
    format!("012_scope_{first:016x}{second:016x}")
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "bridget", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "bridget_publish_artifact",
            "description": "Publier un unique artefact structuré et sourcé. Bridget atteste le projet, la conversation et le tour depuis l'identité connectée. Utiliser kind=html pour une visualisation HTML/JavaScript interactive : payload.html contient le document complet, payload.data les seules données déclarées et payload.inline_height_hint une hauteur souhaitée entre 0 et 1200. Bridget stocke le HTML comme blob canonique et le rend inline dans une sandbox sans réseau, fichiers, cookies, Tauri ni accès à la conversation. Après une publication HTML réussie, répondre brièvement que l’artefact est publié : ne jamais recopier le HTML dans Markdown.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "idempotency_key": { "type": "string", "minLength": 1, "maxLength": 128 },
                    "kind": { "enum": ["chart", "kpi", "table", "timeline", "image", "file", "html"] },
                    "title": { "type": "string", "minLength": 1, "maxLength": 240 },
                    "payload": { "type": "object", "description": "Données structurées de l'artefact. Pour kind=html, inclure html (document complet autonome), data (données injectées) et inline_height_hint (0..1200) ; aucune URL, CDN ou dépendance externe. Pour image/fichier avec blob, inclure blob_digest SHA-256 et media_type." },
                    "sources": {
                        "type": "array", "minItems": 1, "maxItems": 100,
                        "items": {
                            "type": "object",
                            "properties": {
                                "source_kind": { "enum": ["remote", "local_project", "user_supplied", "agent_computed", "restored"] },
                                "locator": { "type": "string", "minLength": 1 },
                                "fetched_at": { "type": ["integer", "null"] },
                                "content_digest": { "type": ["string", "null"], "description": "SHA-256 du contenu source lorsque disponible." },
                                "citation": { "type": "string", "minLength": 1 },
                                "units": { "type": ["string", "null"] },
                                "transformations": { "type": "array", "items": { "type": "string" } },
                                "access_status": { "enum": ["available", "expired", "unavailable", "blocked", "unknown"] }
                            },
                            "required": ["source_kind", "locator", "citation"],
                            "additionalProperties": false
                        }
                    },
                    "quality_notices": { "type": "array", "items": { "type": "string" } },
                    "parent_artifact_ref": { "type": ["string", "null"] },
                    "publication_reason": { "enum": ["initial", "refresh", "restore_changed", "save_interaction"] }
                },
                "required": ["idempotency_key", "kind", "title", "payload", "sources", "publication_reason"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "bridget_send",
            "description": "Envoyer un message Bridget à un équipier.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": { "type": "string", "minLength": 1 },
                    "body": { "type": "string", "minLength": 1 },
                    "reply": { "type": "boolean", "default": false },
                    "reply_timeout": { "type": "integer", "minimum": 1 },
                    "in_reply_to": { "type": "string", "minLength": 1, "description": "Identifiant de la demande Bridget à résoudre par cette réponse." },
                    "id": { "type": "string", "minLength": 1, "description": "Clé métier à réutiliser pour un retry explicite." },
                    "issued_at": { "type": "integer", "minimum": 1, "description": "Horodatage renvoyé par le premier appel ; requis avec id pour rejouer le même contrat." }
                },
                "required": ["to", "body"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "bridget_who",
            "description": "Lister les équipiers Bridget visibles.",
            "inputSchema": {
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "bridget_ledger",
            "description": "Lire les messages et demandes Bridget récents.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "view": { "enum": ["messages", "requests", "both"] },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 20 },
                    "requests_scope": {
                        "enum": ["mine", "all"],
                        "default": "mine",
                        "description": "mine = demandes où l'appelant est participant (défaut) ; all = toutes les demandes ouvertes."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "maicie_delegate",
            "description": "Créer une délégation dans le greffe Maicie central. Une réponse queued exige maicie_request_status.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "goal": { "type": "string", "minLength": 1 },
                    "explicit_target": { "type": "string", "minLength": 1 },
                    "required_tags": { "type": "array", "maxItems": 32, "items": { "type": "string", "minLength": 1 } },
                    "duration": { "enum": ["courte", "normale", "longue"], "default": "normale" },
                    "suite_objective_id": { "type": "string", "minLength": 1 },
                    "depends_on": { "type": "array", "maxItems": 100, "items": { "type": "string", "minLength": 1 } },
                    "references": { "type": "array", "maxItems": 100, "items": { "type": "string", "minLength": 1 } },
                    "review_ref": { "type": "string", "minLength": 3, "description": "Référence distante <remote>/<branche>, atomique avec expected_head." },
                    "expected_head": { "type": "string", "pattern": "^[0-9a-f]{40}$", "description": "SHA complet gelé, atomique avec review_ref." },
                    "request_id": { "type": "string", "minLength": 1, "description": "Clé à réutiliser avec issued_at pour un retry exact." },
                    "issued_at": { "type": "integer", "minimum": 1 }
                },
                "required": ["goal"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "maicie_registre_add",
            "description": "Ajouter une ligne fermée au registre central. Aucun chemin de registre ne vient de l'appelant.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "line": { "type": "string", "minLength": 1 },
                    "request_id": { "type": "string", "minLength": 1 },
                    "issued_at": { "type": "integer", "minimum": 1 }
                },
                "required": ["line"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "maicie_objective_close",
            "description": "Clore un objectif dans le greffe central avec un motif explicite.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "objective_id": { "type": "string", "minLength": 1 },
                    "reason": { "type": "string", "minLength": 1 },
                    "request_id": { "type": "string", "minLength": 1 },
                    "issued_at": { "type": "integer", "minimum": 1 }
                },
                "required": ["objective_id", "reason"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "maicie_request_status",
            "description": "Relire depuis le daemon maître l'issue terminale et ses identifiants durables.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "request_id": { "type": "string", "minLength": 1 }
                },
                "required": ["request_id"],
                "additionalProperties": false
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::cell::Cell;
    use std::collections::BTreeSet;
    use std::io::Cursor;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::{Barrier, mpsc};
    use std::thread;

    const FIXTURES: &str = include_str!("../tests/fixtures/mcp/fr009.jsonl");

    struct EmptyProcessTree;

    impl crate::mcp_identity::ProcessTree for EmptyProcessTree {
        fn birth(&self, _pid: u32) -> Option<u64> {
            None
        }

        fn parent(&self, _pid: u32) -> Option<u32> {
            None
        }
    }

    fn run(lines: &[Value]) -> Vec<Value> {
        let input = lines
            .iter()
            .map(|line| {
                line.as_str()
                    .map_or_else(|| line.to_string(), str::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut stdout = Vec::new();
        let identity = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "fixture-agent".to_string(),
                instance_id: "fixture-instance".to_string(),
            })
        };
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            Err(ToolError::Technical {
                code: "daemon_unreachable",
                message: "daemon de fixture absent".to_string(),
            })
        };
        serve_with(input.as_bytes(), &mut stdout, &identity, &execute).unwrap();
        String::from_utf8(stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn matrice_fr009_couvre_les_quinze_cas() {
        let cases: Vec<Value> = FIXTURES
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(cases.len(), 15, "la matrice FR-009 doit rester complète");

        for case in cases {
            let responses = run(case["stdin"].as_array().unwrap());
            let expected_count = case["responses"].as_u64().unwrap() as usize;
            assert_eq!(responses.len(), expected_count, "{}", case["name"]);
            match case["expect"].as_str().unwrap() {
                "initialize" => {
                    assert_eq!(responses[0]["id"], case["id"], "{}", case["name"]);
                    assert_eq!(responses[0]["result"]["protocolVersion"], PROTOCOL_VERSION);
                    assert_eq!(
                        responses[0]["result"]["capabilities"],
                        json!({ "tools": {} })
                    );
                }
                "tools" => assert_eq!(
                    responses.last().unwrap()["result"]["tools"]
                        .as_array()
                        .unwrap()
                        .len(),
                    8
                ),
                "tools_twice" => assert_eq!(responses[1]["result"], responses[2]["result"]),
                "ping" => assert_eq!(responses.last().unwrap()["result"], json!({})),
                "tool_unavailable" => {
                    assert_eq!(responses.last().unwrap()["result"]["isError"], true)
                }
                "error" => assert!(responses.last().unwrap().get("error").is_some()),
                "none" => assert!(responses.is_empty()),
                "mixed_ids" => {
                    assert_eq!(responses[1]["id"], json!(9));
                    assert_eq!(responses[2]["id"], json!("neuf"));
                }
                other => panic!("expectation inconnue: {other}"),
            }
        }
    }

    #[test]
    fn catalogue_mcp_expose_exactement_les_quatre_verbes_du_greffe() {
        let names = tools()
            .into_iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        let expected = [
            "bridget_ledger",
            "bridget_publish_artifact",
            "bridget_send",
            "bridget_who",
            "maicie_delegate",
            "maicie_objective_close",
            "maicie_registre_add",
            "maicie_request_status",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
        assert_eq!(names, expected);
        for forbidden in ["profile_approve", "routine_approve", "command"] {
            assert!(
                names.iter().all(|name| !name.contains(forbidden)),
                "l'action humaine {forbidden} ne doit jamais être un outil MCP"
            );
        }
    }

    #[test]
    fn publication_html_est_annoncee_au_moteur_comme_un_artefact_sandboxe() {
        let publication = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_publish_artifact")
            .expect("outil de publication présent");
        let kinds = publication["inputSchema"]["properties"]["kind"]["enum"]
            .as_array()
            .expect("énumération des types présente");
        assert!(kinds.iter().any(|kind| kind == "html"));
        let description = publication["description"]
            .as_str()
            .expect("description présente");
        assert!(description.contains("kind=html"));
        assert!(description.contains("ne jamais recopier le HTML dans Markdown"));
    }

    #[test]
    fn publication_d_artefact_transporte_un_contrat_canonique_et_un_recu_atteste() {
        let socket = test_socket("artifact-publish");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let read_stream = stream.try_clone().unwrap();
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::Register { agent_type, .. } if agent_type == "mcp")
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "agent-fixture".to_string(),
                },
            );
            let WrapperToDaemon::ArtifactPublish {
                contract_version,
                canonical_publication,
            } = read_command(&mut reader)
            else {
                panic!("publication d'artefact attendue");
            };
            assert_eq!(contract_version, ARTIFACT_CONTRACT_VERSION);
            let publication: ArtifactPublicationV1 =
                serde_json::from_slice(&canonical_publication).unwrap();
            assert_eq!(publication.canonical_bytes(), canonical_publication);
            let receipt = ArtifactReceiptV1 {
                artifact_ref: "artifact:fixture".to_string(),
                version_ref: "artifact-version:fixture".to_string(),
                state: crate::artifact_types::ArtifactState::Published,
                content_digest: publication.content_digest(),
                warnings: Vec::new(),
                conversation_reference: "conversation:project:agent".to_string(),
                storage_state: crate::artifact_types::ArtifactStorageState::Canonical,
            };
            write_command(
                &mut writer,
                DaemonToWrapper::ArtifactPublicationResult {
                    contract_version: ARTIFACT_CONTRACT_VERSION,
                    replayed: false,
                    receipt_json: Some(serde_json::to_vec(&receipt).unwrap()),
                    refusal_code: None,
                    refusal_message: None,
                },
            );
        });
        let arguments = serde_json::json!({
            "idempotency_key": "fixture-artifact-v1",
            "kind": "kpi",
            "title": "Latence médiane",
            "payload": {"value": 17.5, "unit": "ms"},
            "sources": [{
                "source_kind": "agent_computed",
                "locator": "calculation:p50",
                "citation": "Calcul attesté",
                "content_digest": "f6c9a81b91f3329113d5b0ab57c2ca9cbbc96cd6b6f4ea8434fb829432533a7a",
                "transformations": ["médiane p50"]
            }],
            "publication_reason": "initial"
        });
        let result = execute_tool_at(
            "agent-fixture",
            "bridget_publish_artifact",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "published");
        assert_eq!(result["receipt"]["artifact_ref"], "artifact:fixture");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn publication_d_artefact_refuse_les_champs_non_contractuels_avant_connexion() {
        let socket = test_socket("artifact-invalid");
        let arguments = serde_json::json!({
            "idempotency_key": "fixture-artifact-v1",
            "kind": "kpi",
            "title": "Latence médiane",
            "payload": {},
            "sources": [],
            "publication_reason": "initial",
            "project_id": "projet-impose-par-l-appelant"
        });
        assert!(matches!(
            execute_tool_at(
                "agent-fixture",
                "bridget_publish_artifact",
                arguments.as_object().unwrap(),
                &socket,
            ),
            Err(ToolError::InvalidParams(_))
        ));
    }

    #[test]
    fn stdout_ne_contient_que_des_reponses_json_rpc() {
        let fixture: Value = serde_json::from_str(FIXTURES.lines().next().unwrap()).unwrap();
        let mut input = fixture["stdin"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        input.push('\n');
        input.push_str("{ JSON invalide\n");
        let mut stdout = Vec::new();
        serve(input.as_bytes(), &mut stdout).unwrap();
        for line in String::from_utf8(stdout).unwrap().lines() {
            let response: Value = serde_json::from_str(line).expect("stdout doit être JSON");
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response.get("result").is_some() || response.get("error").is_some());
        }
    }

    #[test]
    fn tools_call_resout_l_identite_a_chaque_appel() {
        let request = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let response = dispatch_with_identity(&request, &mut session, &|| {
            Err(crate::mcp_identity::IdentityError::LegacyMarker)
        })
        .unwrap();
        assert_eq!(response["result"]["code"], "legacy_marker");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn identite_mcp_unicode_est_refusee_avant_toute_execution() {
        let root = test_socket("identity-unicode").with_extension("identity");
        std::fs::create_dir_all(&root).unwrap();
        let name_file = root.join("agent-name");
        std::fs::write(&name_file, "分析\n").unwrap();
        let marker_directory = root.join("agent-pids");
        let resolver = || {
            crate::mcp_identity::resolve_identity_with(
                Some(&name_file),
                &marker_directory,
                Some("fixture-instance"),
                42,
                &EmptyProcessTree,
            )
        };
        let calls = Cell::new(0);
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            calls.set(calls.get() + 1);
            Ok(json!({ "agents": [] }))
        };
        let request = json!({
            "jsonrpc": "2.0", "id": 39, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };

        let response = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        let execution_count = calls.get();
        std::fs::remove_dir_all(root).unwrap();

        assert_eq!(execution_count, 0, "le refus doit précéder tout outil");
        assert_eq!(response["result"]["code"], "identity_not_found");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn identite_mcp_ascii_declenche_exactement_une_execution() {
        let root = test_socket("identity-ascii").with_extension("identity");
        std::fs::create_dir_all(&root).unwrap();
        let name_file = root.join("agent-name");
        std::fs::write(&name_file, "a3d27a89-80d5-4e0f-9b84-cf5523ecb026\n").unwrap();
        let marker_directory = root.join("agent-pids");
        let resolver = || {
            crate::mcp_identity::resolve_identity_with(
                Some(&name_file),
                &marker_directory,
                Some("fixture-instance"),
                42,
                &EmptyProcessTree,
            )
        };
        let calls = Cell::new(0);
        let execute =
            |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, arguments: &Value| {
                calls.set(calls.get() + 1);
                assert_eq!(identity.name, "a3d27a89-80d5-4e0f-9b84-cf5523ecb026");
                assert_eq!(identity.instance_id, "fixture-instance");
                assert_eq!(name, "bridget_who");
                assert_eq!(arguments, &json!({}));
                Ok(json!({ "agents": [] }))
            };
        let request = json!({
            "jsonrpc": "2.0", "id": 40, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };

        let response = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        let execution_count = calls.get();
        std::fs::remove_dir_all(root).unwrap();

        assert_eq!(
            response["result"]["structuredContent"],
            json!({ "agents": [] })
        );
        assert_eq!(execution_count, 1, "le contrôle sain doit exécuter l'outil");
    }

    #[test]
    fn deux_appels_outil_ne_partagent_pas_l_identite_resolue() {
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let calls = Cell::new(0);
        let resolver = || {
            calls.set(calls.get() + 1);
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "agent".to_string(),
                instance_id: "instance".to_string(),
            })
        };
        for id in [3, 4] {
            let request = json!({
                "jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": { "name": "bridget_who", "arguments": {} }
            });
            let response = dispatch_with_identity(&request, &mut session, &resolver).unwrap();
            assert_eq!(response["id"], id);
        }
        assert_eq!(calls.get(), 2);
    }

    fn test_socket(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-mcp-{label}-{}-{}.sock",
            std::process::id(),
            NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn read_command(reader: &mut BufReader<UnixStream>) -> WrapperToDaemon {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim_end()).unwrap()
    }

    fn write_command(writer: &mut BufWriter<UnixStream>, response: DaemonToWrapper) {
        writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
        writer.flush().unwrap();
    }

    #[test]
    fn mutation_mcp_declare_l_identite_une_fois_et_ne_vend_pas_queued_comme_un_effet() {
        let socket = test_socket("greffe-queued");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_scope = issuer_scope("instance-greffe-1");
        let server_scope = expected_scope.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::Register {
                    agent_type,
                    identity_version: 2,
                    agent_id: name,
                    instance_id: Some(instance_id),
                    ..
                } if agent_type == "mcp"
                    && name == "jc2"
                    && instance_id == "instance-greffe-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    issuer_scope,
                    request_id,
                    issued_at: 1_787_824_100,
                    from,
                    operation: ServiceRequestOperation::RegistreAdd,
                    payload: ServiceRequestPayload::RegistreAdd { line },
                    ..
                } if issuer_scope == server_scope
                    && request_id == "request-registre-1"
                    && from == "jc2"
                    && line == "kind=add id=constat-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: server_scope,
                    request_id: "request-registre-1".to_string(),
                    issue: "queued".to_string(),
                    expires_at: 1_787_824_160,
                    payload: None,
                },
            );
        });

        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-1",
            "maicie_registre_add",
            json!({
                "line": "kind=add id=constat-1",
                "request_id": "request-registre-1",
                "issued_at": 1_787_824_100_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "queued");
        assert_eq!(result["terminal"], false);
        assert_eq!(result["applied"], false);
        assert_eq!(result["next"], "maicie_request_status");
        assert_eq!(result["issuer_scope"], expected_scope);
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn delegate_mcp_transporte_atomiquement_la_cible_de_revue_en_v2() {
        let socket = test_socket("delegate-review-target");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::Register { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    version: bridget_transport::protocol::REVIEW_DELEGATE_CONTRACT_VERSION,
                    operation: ServiceRequestOperation::Delegate,
                    payload: ServiceRequestPayload::Delegate {
                        review_target: Some(ReviewTarget { target_ref, expected_head }),
                        ..
                    },
                    ..
                } if target_ref == "origin/session-047-verdict-tete-reecrite"
                    && expected_head == "a".repeat(40)
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: issuer_scope("instance-review-target"),
                    request_id: "request-review-target".to_string(),
                    issue: "queued".to_string(),
                    expires_at: 1_787_824_560,
                    payload: None,
                },
            );
        });

        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-review-target",
            "maicie_delegate",
            json!({
                "goal": "relire le lot",
                "review_ref": "origin/session-047-verdict-tete-reecrite",
                "expected_head": "a".repeat(40),
                "request_id": "request-review-target",
                "issued_at": 1_787_824_500_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "queued");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();

        for isolated in [
            json!({ "goal": "relire", "review_ref": "origin/main" }),
            json!({ "goal": "relire", "expected_head": "a".repeat(40) }),
        ] {
            assert!(matches!(
                execute_tool_at_with_scope(
                    "jc2",
                    "instance-review-target",
                    "maicie_delegate",
                    isolated.as_object().unwrap(),
                    Path::new("/socket/ne-doit-pas-etre-ouverte"),
                ),
                Err(ToolError::InvalidParams(reason))
                    if reason.contains("doivent être fournis ensemble")
            ));
        }
    }

    #[test]
    fn coupure_apres_depot_mcp_reste_outcome_unknown_sans_succes_invente() {
        let socket = test_socket("greffe-cut");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::Register { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    operation: ServiceRequestOperation::ObjectiveClose,
                    ..
                }
            ));
            // Fermeture volontaire après lecture : le dépôt peut avoir eu lieu,
            // mais aucun résultat terminal n'est attesté à l'appelant.
        });
        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-2",
            "maicie_objective_close",
            json!({
                "objective_id": "objective-1",
                "reason": "objectif atteint",
                "request_id": "request-close-1",
                "issued_at": 1_787_824_200_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "outcome_unknown");
        assert_eq!(result["terminal"], false);
        assert_eq!(result["applied"], false);
        assert_eq!(result["next"], "maicie_request_status");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn request_status_relit_les_identifiants_terminaux_du_maitre() {
        let socket = test_socket("greffe-status");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_scope = issuer_scope("instance-greffe-3");
        let server_scope = expected_scope.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Service
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Service,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceHello {
                    service,
                    issuer_scope,
                    capabilities,
                    ..
                } if service == "maicie"
                    && issuer_scope == server_scope
                    && capabilities == vec![ServiceCapability::MaicieGuichet]
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ServiceWelcome {
                    version: SERVICE_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ServiceCapability::MaicieGuichet],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::GuichetLookup {
                    issuer_scope,
                    request_id,
                    ..
                } if issuer_scope == server_scope && request_id == "request-delegate-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: server_scope,
                    request_id: "request-delegate-1".to_string(),
                    issue: "accepted".to_string(),
                    expires_at: 1_787_824_360,
                    payload: Some(GuichetReplyPayload::Delegate {
                        status: GuichetDelegateMutationStatus::Created,
                        objective_id: Some("objective-1".to_string()),
                        delegation_id: Some("delegation-1".to_string()),
                        message_id: Some("message-1".to_string()),
                        participant: Some("cursor-1".to_string()),
                        candidates: Vec::new(),
                        waiting_on_prerequisites: false,
                        replayed: false,
                    }),
                },
            );
        });
        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-3",
            "maicie_request_status",
            json!({ "request_id": "request-delegate-1" })
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "created");
        assert_eq!(result["terminal"], true);
        assert_eq!(result["applied"], true);
        assert_eq!(result["objective_id"], "objective-1");
        assert_eq!(result["delegation_id"], "delegation-1");
        assert_eq!(result["message_id"], "message-1");
        assert_eq!(result["issuer_scope"], expected_scope);
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn mutation_mcp_refuse_toute_seconde_source_de_principal_ou_de_registre() {
        let socket = Path::new("/socket/inutile");
        for forbidden in ["from", "principal", "database_path", "catalogue_path"] {
            let mut arguments = json!({ "line": "kind=add id=constat-1" });
            arguments[forbidden] = json!("forged");
            assert!(matches!(
                execute_tool_at_with_scope(
                    "jc2",
                    "instance-greffe-4",
                    "maicie_registre_add",
                    arguments.as_object().unwrap(),
                    socket,
                ),
                Err(ToolError::InvalidParams(reason)) if reason.contains(forbidden)
            ));
        }
    }

    #[test]
    fn send_transmet_un_corps_riche_octet_pour_octet_et_negocie_le_client() {
        let socket = test_socket("send-rich");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_body =
            "l'apostrophe, \"guillemets\", $VAR, `backticks`,\net l'emoji é".to_string();
        let server = thread::spawn({
            let expected_body = expected_body.clone();
            move || {
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Client
                    }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ClientHello { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                match read_command(&mut reader) {
                    WrapperToDaemon::SendIdempotent {
                        message,
                        message_id,
                        ..
                    } => {
                        assert_eq!(message.body, expected_body);
                        assert_eq!(message.in_reply_to, None);
                        assert_eq!(message_id, "retry-me");
                    }
                    other => panic!("commande inattendue: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-me".to_string(),
                        issue: IdempotencyIssue::Accepted { expires_at: 60 },
                    },
                );
            }
        });
        let result = execute_tool_at(
            "codex-1",
            "bridget_send",
            json!({
            "to": "claude-1", "body": expected_body, "id": "retry-me"
            , "issued_at": 1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "accepted");
        assert_eq!(result["id"], "retry-me");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn send_transmet_in_reply_to_dans_l_enveloppe_idempotente() {
        let socket = test_socket("send-reply");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent { message, .. } => {
                    assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
                    assert!(!message.reply);
                }
                other => panic!("commande inattendue: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "reply-1".to_string(),
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                },
            );
        });
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "bridget_send",
                "arguments": {
                    "to": "bridget",
                    "body": "réponse liée",
                    "in_reply_to": "request-open",
                    "id": "reply-1",
                    "issued_at": 1_700_000_000
                }
            }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let resolver = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "codex-1".to_string(),
                instance_id: "test-instance".to_string(),
            })
        };
        let execute =
            |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, arguments: &Value| {
                execute_tool_at_with_scope(
                    &identity.name,
                    &identity.instance_id,
                    name,
                    arguments.as_object().expect("arguments objet"),
                    &socket,
                )
            };
        let result = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        assert_eq!(result["result"]["structuredContent"]["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn schema_send_expose_in_reply_to_non_vide() {
        let send = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_send")
            .unwrap();
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["type"],
            "string"
        );
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["minLength"],
            1
        );
    }

    #[test]
    fn resultats_metier_conservent_les_categories_de_refus_fermees() {
        for (category, expected) in [
            ("dnd", "dnd"),
            ("circuit_breaker", "circuit_breaker"),
            ("duplicate_content", "duplicate"),
            ("routing", "unknown_recipient"),
        ] {
            let result = send_issue_result(
                "id-1",
                1_700_000_000,
                IdempotencyIssue::Rejected {
                    category: category.to_string(),
                    reason: "motif exact".to_string(),
                    expires_at: 1_700_000_100,
                },
            );
            assert_eq!(result["status"], expected);
            assert_eq!(result["reason"], "motif exact");
        }
    }

    #[test]
    fn retry_rejoue_scope_et_horodatage_malgre_rename() {
        let socket = test_socket("retry-scope");
        let listener = UnixListener::bind(&socket).unwrap();
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                let hello = read_command(&mut reader);
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                let send = read_command(&mut reader);
                seen_tx.send((hello, send)).unwrap();
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-1".to_string(),
                        issue: IdempotencyIssue::Accepted {
                            expires_at: 1_700_000_060,
                        },
                    },
                );
            }
        });
        let arguments = json!({
            "to": "claude-1", "body": "même prompt", "id": "retry-1", "issued_at": 1_700_000_000
        });
        for (index, identity) in ["avant-rename", "apres-rename"].into_iter().enumerate() {
            if index == 1 {
                thread::sleep(Duration::from_secs(1));
            }
            let result = execute_tool_at_with_scope(
                identity,
                "instance-stable-1",
                "bridget_send",
                arguments.as_object().unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(result["issued_at"], 1_700_000_000);
        }
        let first = seen_rx.recv().unwrap();
        let second = seen_rx.recv().unwrap();
        let scope = |command: WrapperToDaemon| match command {
            WrapperToDaemon::ClientHello { issuer_scope, .. } => issuer_scope,
            other => panic!("hello attendu: {other:?}"),
        };
        assert_eq!(scope(first.0), scope(second.0));
        for command in [first.1, second.1] {
            match command {
                WrapperToDaemon::SendIdempotent {
                    message, issued_at, ..
                } => {
                    assert_eq!(issued_at, 1_700_000_000);
                    assert_eq!(message.body, "même prompt");
                }
                other => panic!("send attendu: {other:?}"),
            }
        }
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    /// Le cas nominal ne doit plus s'annoncer comme un incident : le daemon
    /// répond avant l'accusé du destinataire, donc TOUT premier envoi passe
    /// par là. Annoncer « accusé perdu » sur un succès, c'est inviter au
    /// double envoi — le défaut mesuré sur ~100 envois d'une seule journée.
    ///
    /// Le statut lui-même doit changer : un lecteur branche sur `status`, pas
    /// sur la prose du motif.
    #[test]
    fn une_remise_en_vol_ne_s_annonce_pas_comme_un_accuse_perdu() {
        let issue = IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: Some("livraison-7".to_string()),
        };
        let rendered = send_issue_result("msg-1", 1_700_000_000, issue);
        assert_eq!(rendered["status"], "in_flight");
        assert_eq!(rendered["delivery_id"], "livraison-7");
        // Forme CLOSE : diagnostic puis consigne, rien avant, rien après. Un
        // `contains` laissait passer tout préfixe ajouté — dont un préfixe qui
        // affirme un dépôt que personne n'a constaté.
        assert_eq!(
            rendered["reason"].as_str().unwrap(),
            format!("{DIAGNOSTIC_REMISE_EN_VOL} ; {REJEU_A_L_IDENTIQUE}")
        );
    }

    /// Gardien du point de vérité : les autres oracles vérifient que chaque
    /// ancrage porte CETTE chaîne, celui-ci vérifie ce que la chaîne dit. Sans
    /// lui, la consigne pourrait se vider de son sens sans faire rougir un
    /// seul test — c'est exactement ainsi que le troisième invariant avait
    /// disparu de la version précédente.
    #[test]
    fn la_consigne_de_rejeu_nomme_ses_trois_invariants_et_l_absence_de_doublon() {
        // Chercher « id » ne gardait RIEN : « id » est déjà contenu dans
        // « à l'identique ». Amputer la consigne de « même id, » laissait donc
        // ce gardien vert — sur l'invariant précisément qui avait disparu de la
        // version précédente, celui qu'il était censé protéger. Chaque
        // invariant se vérifie sous la forme qui l'énonce, pas sous un fragment
        // que le reste de la phrase fournit déjà.
        for invariant in ["même id", "même issued_at", "même corps"] {
            assert!(
                REJEU_A_L_IDENTIQUE.contains(invariant),
                "la consigne doit nommer l'invariant « {invariant} »"
            );
        }
        assert!(
            REJEU_A_L_IDENTIQUE.contains("dupliquer"),
            "sans la promesse de non-duplication, le rejeu reste redouté et personne ne l'ose"
        );
    }

    /// Gardien des diagnostics : la forme close verrouille la COMPOSITION du
    /// motif, celui-ci verrouille ce que chaque morceau affirme. Sans lui, il
    /// suffirait de réécrire une constante pour qu'un sort inconnu s'annonce
    /// comme un dépôt attesté — les oracles de forme resteraient verts, car ils
    /// compareraient le rendu à la constante mensongère elle-même.
    ///
    /// Une seule des trois formes atteste une remise. Les deux autres portent
    /// sur des cas où rien n'est constaté : elles ne doivent rien promettre.
    #[test]
    fn seul_le_diagnostic_de_remise_en_vol_atteste_quelque_chose() {
        assert!(
            DIAGNOSTIC_REMISE_EN_VOL.contains("remise en vol"),
            "le seul cas où le daemon a pris la remise doit le dire"
        );
        for (diagnostic, nom) in [
            (DIAGNOSTIC_SORT_INDETERMINE, "sort indéterminé"),
            (DIAGNOSTIC_ACCUSE_PERDU, "accusé perdu"),
        ] {
            for promesse in ["remise en vol", "attesté", "atteste", "réussi", "déposé"] {
                assert!(
                    !diagnostic.contains(promesse),
                    "le diagnostic « {nom} » porte sur un sort NON constaté : \
                     il ne doit rien promettre, or il contient « {promesse} »"
                );
            }
        }
    }

    /// Contre-épreuve : sans `delivery_id`, le sort est vraiment inconnu et le
    /// retour ne doit pas rassurer. Si ce test tombe, le correctif a effacé la
    /// distinction qu'il avait pour but d'établir — c'est-à-dire qu'il aurait
    /// remplacé un mensonge pessimiste par un mensonge optimiste, bien pire
    /// dans un système d'attestation.
    #[test]
    fn un_sort_indetermine_ne_promet_pas_une_remise() {
        let issue = IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: None,
        };
        let rendered = send_issue_result("msg-2", 1_700_000_000, issue);
        assert_eq!(rendered["status"], "outcome_unknown");
        assert!(rendered.get("delivery_id").is_none());
        assert_eq!(
            rendered["reason"].as_str().unwrap(),
            format!("{DIAGNOSTIC_SORT_INDETERMINE} ; {REJEU_A_L_IDENTIQUE}")
        );
    }

    /// L'invariant qui porte tout le correctif : les deux cas ne partagent pas
    /// leur statut. Les deux tests précédents pourraient rester verts alors que
    /// les statuts auraient reconvergé sur une valeur commune ; celui-ci le
    /// constate directement.
    #[test]
    fn la_remise_en_vol_et_le_sort_inconnu_ne_partagent_pas_leur_statut() {
        let en_vol = send_issue_result(
            "msg-3",
            1_700_000_000,
            IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: Some("livraison-8".to_string()),
            },
        );
        let inconnu = send_issue_result(
            "msg-3",
            1_700_000_000,
            IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: None,
            },
        );
        assert_ne!(
            en_vol["status"], inconnu["status"],
            "un dépôt réussi et un sort inconnu doivent se lire sur le statut seul"
        );
    }

    /// Le discriminant vit à UN seul endroit, donc les deux surfaces ne peuvent
    /// pas diverger. Sans cet oracle, rien n'empêche le retour MCP et la sortie
    /// du binaire de répondre différemment à la même issue — et un agent qui
    /// lit les deux n'aurait aucun moyen de savoir laquelle croire.
    ///
    /// Le cas `Some("")` est celui qui les faisait déjà diverger : la ligne du
    /// binaire annonçait « en vol » quand son propre code de sortie refusait le
    /// dépôt.
    #[test]
    fn le_statut_mcp_et_le_depot_du_binaire_ne_peuvent_pas_se_contredire() {
        for (delivery_id, statut_attendu, depot_attendu) in [
            (Some("livraison-9"), STATUT_IN_FLIGHT, true),
            (None, STATUT_OUTCOME_UNKNOWN, false),
            (Some(""), STATUT_OUTCOME_UNKNOWN, false),
            (Some("   "), STATUT_OUTCOME_UNKNOWN, false),
        ] {
            let issue = IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: delivery_id.map(str::to_string),
            };
            let rendu = send_issue_result("msg-4", 1_700_000_000, issue.clone());
            assert_eq!(
                rendu["status"], statut_attendu,
                "statut MCP pour delivery_id={delivery_id:?}"
            );
            assert_eq!(
                crate::cli::send_deposited(&issue),
                depot_attendu,
                "le binaire doit conclure comme le statut MCP pour delivery_id={delivery_id:?}"
            );
            // La preuve n'est publiée que lorsqu'elle atteste quelque chose.
            assert_eq!(
                rendu.get("delivery_id").is_some(),
                depot_attendu,
                "un delivery_id sans valeur probante ne doit pas être publié"
            );
        }
    }

    /// Oracle : `orphaned` n'est ni `in_flight` ni `outcome_unknown`.
    #[test]
    fn statut_orphelin_distinct_de_in_flight_et_outcome_unknown() {
        let issue = IdempotencyIssue::Orphaned {
            expires_at: 1_700_000_060,
            delivery_id: "delivery-orphelin".to_string(),
            reason: "destinataire purgé".to_string(),
        };
        let rendu = send_issue_result("msg-orphelin", 1_700_000_000, issue.clone());
        assert_eq!(rendu["status"], STATUT_ORPHELIN);
        assert_ne!(rendu["status"], STATUT_IN_FLIGHT);
        assert_ne!(rendu["status"], STATUT_OUTCOME_UNKNOWN);
        assert_eq!(rendu["delivery_id"], "delivery-orphelin");
        assert!(
            !crate::cli::send_deposited(&issue),
            "un orphelin n'est pas un dépôt réussi à conclure en rc=0"
        );
        let reason = rendu["reason"].as_str().unwrap_or("");
        assert!(
            reason.contains(CONDUITE_ORPHELIN),
            "orphaned doit porter la conduite, pas seulement le constat: {reason}"
        );
        assert!(
            !reason.contains(REJEU_A_L_IDENTIQUE),
            "orphaned ne doit PAS enseigner le rejeu à l'identique (absorbant): {reason}"
        );
    }

    #[test]
    fn coupure_apres_transmission_devient_outcome_unknown_et_le_retry_reste_identique() {
        let socket = test_socket("cut-after-send");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::SendIdempotent { .. }
            ));
            drop(writer);
            drop(reader);

            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent {
                    message_id,
                    issued_at,
                    ..
                } => {
                    assert_eq!(message_id, "retry-cut");
                    assert_eq!(issued_at, 1_700_000_000);
                }
                other => panic!("send attendu: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "retry-cut".to_string(),
                    issue: IdempotencyIssue::Accepted {
                        expires_at: 1_700_000_060,
                    },
                },
            );
        });
        let arguments = json!({
            "to":"claude-1", "body":"retry", "id":"retry-cut", "issued_at":1_700_000_000
        });
        let first = execute_tool_at_with_scope(
            "agent",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(first["status"], "outcome_unknown");
        let retry = execute_tool_at_with_scope(
            "agent-renamed",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(retry["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    /// L'oracle du mandat : un envoi dont l'accusé aval est retardé rend un
    /// statut HONNÊTE (remise en vol, avec sa preuve), et le rejeu de la même
    /// clé lit le sort réel — en portant EXACTEMENT la même enveloppe, jamais
    /// une nouvelle émission. C'est le geste que le libellé d'origine faisait
    /// redouter alors qu'il est le seul chemin correct.
    #[test]
    fn accuse_retarde_rend_un_statut_honnete_puis_le_rejeu_lit_le_sort_sans_dupliquer() {
        let socket = test_socket("ack-differe");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let mut envelopes = Vec::new();
            for issue in [
                IdempotencyIssue::OutcomeUnknown {
                    expires_at: 1_700_000_060,
                    delivery_id: Some("livraison-differee".to_string()),
                },
                IdempotencyIssue::Accepted {
                    expires_at: 1_700_000_060,
                },
            ] {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ClientHello { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                match read_command(&mut reader) {
                    WrapperToDaemon::SendIdempotent {
                        message_id,
                        issued_at,
                        message,
                        ..
                    } => envelopes.push((message_id, issued_at, message.body)),
                    other => panic!("send attendu: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "ack-differe-1".to_string(),
                        issue,
                    },
                );
            }
            envelopes
        });
        let arguments = json!({
            "to":"bridget", "body":"livraison", "id":"ack-differe-1", "issued_at":1_700_000_000
        });
        let first = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(first["status"], "in_flight");
        assert_eq!(first["delivery_id"], "livraison-differee");
        assert!(!first["reason"].as_str().unwrap().contains("perdu"));

        let replay = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(replay["status"], "accepted");

        let envelopes = server.join().unwrap();
        assert_eq!(
            envelopes[0], envelopes[1],
            "le rejeu doit porter la même enveloppe, sinon il duplique au lieu de consulter"
        );
        std::fs::remove_file(socket).unwrap();
    }

    /// ORACLE C4 — la TROISIÈME forme d'`outcome_unknown`, celle qui naît côté
    /// client sans aucune issue du daemon : la connexion tombe APRÈS l'écriture
    /// de la commande, donc l'outil ne lira jamais la réponse.
    ///
    /// Ce chemin était le seul des trois à n'avoir aucun filet sur son
    /// contenu : le banc de coupure voisin ne vérifie que le `status`, si bien
    /// qu'une mutation du corps du retour y survivait. Or c'est justement le
    /// cas où l'appelant a le plus besoin de la consigne de rejeu — le message
    /// a pu partir, et lui seul l'ignore.
    ///
    /// L'oracle verrouille donc le contrat ENTIER de ce bras : le statut, les
    /// deux clés sans lesquelles aucun rejeu n'est possible, et les trois
    /// invariants du rejeu à l'identique.
    #[test]
    fn la_coupure_apres_ecriture_rend_les_cles_de_rejeu_et_les_trois_invariants() {
        let socket = test_socket("coupure-contrat");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::SendIdempotent { .. }
            ));
            // La coupure : la commande est écrite et lue, aucune réponse ne
            // vient. C'est ce qui distingue ce cas des deux autres.
            drop(writer);
            drop(reader);
        });
        let rendered = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            json!({
                "to":"bridget", "body":"coupure", "id":"coupure-1", "issued_at":1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();

        assert_eq!(rendered["status"], "outcome_unknown");
        assert!(
            rendered.get("delivery_id").is_none(),
            "aucune issue n'a été lue : rien n'atteste une remise, et rien ne doit le prétendre"
        );
        // Sans ces deux clés, la consigne de rejeu est irréalisable.
        assert_eq!(rendered["id"], "coupure-1");
        assert_eq!(rendered["issued_at"], 1_700_000_000i64);

        // Forme CLOSE. Porter la consigne ne suffit pas : un motif peut la
        // citer mot pour mot et affirmer juste avant un dépôt que personne n'a
        // constaté. Sur CE chemin le mensonge optimiste est le pire de tous —
        // aucune issue n'a été lue, le message a pu ne jamais partir.
        //
        // Le seul ajout tolérable est le détail technique final entre
        // parenthèses : il vient de l'erreur d'entrée-sortie, il n'est pas
        // rédigé, et il varie d'une plateforme à l'autre. Tout le reste est
        // verrouillé au caractère près.
        let reason = rendered["reason"].as_str().unwrap();
        let attendu = format!("{DIAGNOSTIC_ACCUSE_PERDU} — {REJEU_A_L_IDENTIQUE} (");
        assert!(
            reason.starts_with(&attendu),
            "le motif doit être exactement le diagnostic puis la consigne, sans rien avant ni entre: {reason}"
        );
        assert!(
            reason.ends_with(')'),
            "seul le détail technique entre parenthèses peut suivre la consigne: {reason}"
        );

        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn issues_idempotentes_deterministes_restent_metier_et_inconnues_sont_preservees() {
        for (issue, status) in [
            (IdempotencyIssue::EnvelopeMismatch, "envelope_mismatch"),
            (IdempotencyIssue::IdempotencyExpired, "idempotency_expired"),
            (IdempotencyIssue::InvalidIssuedAt, "invalid_issued_at"),
        ] {
            assert_eq!(send_issue_result("id", 7, issue)["status"], status);
        }
        let unknown = send_issue_result(
            "id",
            7,
            IdempotencyIssue::Rejected {
                category: "future_refusal".to_string(),
                reason: "raison".to_string(),
                expires_at: 8,
            },
        );
        assert_eq!(unknown["status"], "future_refusal");
    }

    #[test]
    fn resultat_mcp_duplique_le_payload_dans_textcontent() {
        let payload = json!({ "status": "accepted", "id": "m-1", "issued_at": 8 });
        let result = tool_result(payload.clone());
        assert_eq!(result["structuredContent"], payload);
        assert_eq!(
            serde_json::from_str::<Value>(result["content"][0]["text"].as_str().unwrap()).unwrap(),
            result["structuredContent"]
        );
    }

    #[test]
    fn huit_connexions_simultanees_gardent_le_principal_resolu_et_la_neuvieme_est_busy() {
        let started = Arc::new(Barrier::new(MAX_IN_FLIGHT_TOOL_CALLS + 1));
        let (started_tx, started_rx) = mpsc::channel();
        let socket = test_socket("eight-registers");
        let listener = UnixListener::bind(&socket).unwrap();
        let (principals_tx, principals_rx) = mpsc::channel();
        let daemon = thread::spawn(move || {
            for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                match read_command(&mut reader) {
                    WrapperToDaemon::Register {
                        identity_version: 2,
                        agent_id: name,
                        instance_id: Some(instance_id),
                        ..
                    } => principals_tx.send((name, instance_id)).unwrap(),
                    other => panic!("Register MCP attendu: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id: "mcp".to_string(),
                    },
                );
            }
        });
        let input = [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        ]
        .into_iter()
        .chain((2..=10).map(|id| {
            json!({
                "jsonrpc":"2.0", "id": id, "method":"tools/call",
                "params":{"name":"bridget_who","arguments":{}}
            })
        }))
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let barrier = Arc::clone(&started);
        let socket_for_calls = socket.clone();
        let server = thread::spawn(move || {
            let resolver = || {
                Ok(crate::mcp_identity::ResolvedIdentity {
                    name: "agent".to_string(),
                    instance_id: "instance".to_string(),
                })
            };
            let execute = move |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                let _connection =
                    registered_connection("fixture-agent", "fixture-instance", &socket_for_calls)
                        .unwrap();
                started_tx.send(()).unwrap();
                barrier.wait();
                Ok(json!({ "agents": [] }))
            };
            let mut output = Vec::new();
            serve_with(Cursor::new(input), &mut output, &resolver, &execute).unwrap();
            output
        });
        for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        started.wait();
        let output = String::from_utf8(server.join().unwrap()).unwrap();
        let responses = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            responses
                .iter()
                .filter(|response| response["result"]["code"] == "busy")
                .count(),
            1
        );

        let principals = (0..MAX_IN_FLIGHT_TOOL_CALLS)
            .map(|_| principals_rx.recv_timeout(Duration::from_secs(2)).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(principals.len(), MAX_IN_FLIGHT_TOOL_CALLS);
        assert!(principals.iter().all(|(name, instance_id)| {
            name == "fixture-agent" && instance_id == "fixture-instance"
        }));
        daemon.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn connexion_non_bloquante_refuse_un_budget_expire_avant_toute_attente() {
        let socket = test_socket("expired-connect");
        let _listener = UnixListener::bind(&socket).unwrap();
        let error =
            connect_nonblocking(&socket, Instant::now() - Duration::from_millis(1)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn dto_ledger_respectent_le_contrat_outil() {
        let source = bridget_transport::protocol::LedgerMessage {
            id: "m-1".to_string(),
            ts: 4,
            sender: "alice".to_string(),
            target: "bob".to_string(),
            body: "riche".to_string(),
            delivery_status: Some(bridget_transport::protocol::LedgerDeliveryStatus::EnVol),
        };
        let message = ledger_message_dto(source.clone());
        assert_eq!(
            message,
            json!({"id":"m-1","from":"alice","to":"bob","body":"riche","ts":4,"delivery_status":"en_vol"})
        );
        assert_eq!(
            crate::cli::render_ledger(&[source]),
            "Derniers 1 messages :\n  [4] alice → bob [en vol]: riche\n"
        );
        let request = request_dto(bridget_transport::protocol::RequestInfo {
            id: "r-1".to_string(),
            sender: "alice".to_string(),
            target: "bob".to_string(),
            state: "open".to_string(),
            created_at: 2,
            deadline_at: 9,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        });
        assert_eq!(
            request,
            json!({"id":"r-1","from":"alice","to":"bob","state":"open","deadline":9,"created":2})
        );
    }

    #[test]
    fn ledger_requests_scope_mine_par_defaut_et_all_accepte() {
        let schema = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_ledger")
            .unwrap();
        assert_eq!(
            schema["inputSchema"]["properties"]["requests_scope"]["default"],
            "mine"
        );
        assert_eq!(
            schema["inputSchema"]["properties"]["requests_scope"]["enum"],
            json!(["mine", "all"])
        );

        let mut args = serde_json::Map::new();
        args.insert("view".into(), json!("requests"));
        assert_eq!(parse_requests_scope(&args).unwrap(), RequestsScope::Mine);
        args.insert("requests_scope".into(), json!("all"));
        assert_eq!(parse_requests_scope(&args).unwrap(), RequestsScope::All);
        args.insert("requests_scope".into(), json!("everyone"));
        assert!(parse_requests_scope(&args).is_err());
    }

    #[test]
    fn daemon_coupe_est_un_is_error_technique_immediat() {
        let missing = test_socket("missing");
        let error = execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &missing)
            .unwrap_err();
        assert!(matches!(
            error,
            ToolError::Technical {
                code: "daemon_unreachable",
                ..
            }
        ));
    }

    #[test]
    fn limite_d_appels_simultanes_refuse_avant_toute_connexion() {
        let active = AtomicUsize::new(0);
        for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            assert!(try_reserve_tool_call(&active));
        }
        assert!(!try_reserve_tool_call(&active));
        assert_eq!(active.load(Ordering::Acquire), MAX_IN_FLIGHT_TOOL_CALLS);
    }

    #[test]
    fn who_ouvre_une_connexion_ephemere_register_puis_commande() {
        let socket = test_socket("who");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let read_stream = stream.try_clone().unwrap();
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::Register { agent_type, .. } if agent_type == "mcp")
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "mcp-test".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ListAgents
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::AgentList { agents: Vec::new() },
            );
        });
        let result =
            execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &socket).unwrap();
        assert_eq!(result, json!({ "agents": [] }));
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }
}
