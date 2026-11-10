//! Adaptation d'identité T3. Aucun lancement ni état de tâche ne dépend de ce module.
use crate::mcp_identity::{IdentityError, ResolvedIdentity};
use crate::t3code_contract::{ServerRuntime, base_dir, read_runtime};
use serde_json::{Value, json};
use std::io::Read;
use std::time::Duration;

const MAX_RESPONSE: u64 = 64 * 1024;
// Même preuve stricte ; budget réseau indépendant de la validité des droits.
const ATTESTATION_HTTP_TIMEOUT_SECS: u64 = 10;
const ATTESTATION_CLEANUP_TIMEOUT_SECS: u64 = 3;
// initialize, initialized, bridget_session, puis fermeture du transport.
// Le client natif doit couvrir toute cette vérification, pas un seul POST.
pub(crate) const ATTESTATION_MAX_BUDGET: Duration = Duration::from_secs(
    3 * ATTESTATION_HTTP_TIMEOUT_SECS + ATTESTATION_CLEANUP_TIMEOUT_SECS,
);

pub(crate) type SessionProof = bridget_transport::protocol::OriginSessionProof;

pub(crate) fn private_proof() -> Option<bridget_transport::protocol::NativePermissionProof> {
    Some(bridget_transport::protocol::NativePermissionProof {
        endpoint:std::env::var("BRIDGET_T3_MCP_ENDPOINT").ok()?,
        authorization:bridget_transport::protocol::IdentityCredential::new(std::env::var("BRIDGET_T3_MCP_AUTHORIZATION").ok()?),
    })
}

pub(crate) fn reattest(proof:&bridget_transport::protocol::NativePermissionProof)->Result<SessionProof,IdentityError> {
    let runtime=read_runtime(&base_dir().map_err(|_|IdentityError::T3SessionUnavailable)?).map_err(|_|IdentityError::T3SessionUnavailable)?;
    let authorization=proof.authorization.expose_for_attestation();
    let url=validate_endpoint(&runtime,&proof.endpoint,authorization)?;
    attest_http(&url,authorization)
}

/// La présence d'une seule variable sélectionne aussi ce chemin : une preuve
/// partielle ou refusée ne doit jamais retomber sur une identité de processus.
pub(crate) fn resolve_current() -> Option<Result<ResolvedIdentity, IdentityError>> {
    let endpoint = std::env::var_os("BRIDGET_T3_MCP_ENDPOINT");
    let authorization = std::env::var_os("BRIDGET_T3_MCP_AUTHORIZATION");
    if endpoint.is_none() && authorization.is_none() {
        return None;
    }
    Some((|| {
        let denied = || IdentityError::T3SessionUnavailable;
        let endpoint = endpoint
            .and_then(|v| v.into_string().ok())
            .ok_or_else(denied)?;
        let authorization = authorization
            .and_then(|v| v.into_string().ok())
            .ok_or_else(denied)?;
        let runtime = read_runtime(&base_dir().map_err(|_| denied())?).map_err(|_| denied())?;
        let url = validate_endpoint(&runtime, &endpoint, &authorization)?;
        let proof = attest_http(&url, &authorization)?;
        let namespace = crate::environment::Namespace::from_environment().map_err(|_| denied())?;
        resolve_binding(&proof.thread_id, &namespace.socket)
    })())
}

fn resolve_binding(
    thread_id: &str,
    socket: &std::path::Path,
) -> Result<ResolvedIdentity, IdentityError> {
    let name = crate::t3code::stable_uuid(thread_id);
    let instance_id = crate::t3code::stable_uuid(&format!("instance:{thread_id}"));
    crate::mcp_identity::auxiliary_registration(&name, &instance_id, socket)
        .map_err(|_| IdentityError::T3SessionUnavailableAt("binding_credential_unavailable : rattachement Bridget local absent ou invalide"))?;
    // La preuve locale seule ne suffit pas : le daemon doit reconnaître
    // cette instance maintenant, même pour un outil de diagnostic.
    crate::communication::client::registered_connection(&name, &instance_id, socket)
        .map_err(binding_failure)?;
    Ok(ResolvedIdentity {
        name,
        instance_id,
        delegated_origin: None,
    })
}

fn binding_failure(error: crate::communication::client::ClientError) -> IdentityError {
    use crate::communication::client::ClientError;
    IdentityError::T3SessionUnavailableAt(match error {
        ClientError::Technical {
            code: "auxiliary_identity_unproven",
            message,
        } => match message.as_str() {
            "auxiliary_identity_unproven : primary_absent" => {
                "binding_registration_refused : primary_absent : le lien Bridget du fil est absent. Attendez sa reconnexion, puis reprenez la lecture avec bridget_task_status et le même task_id. Une attestation bridget_session ne prouve pas que ce lien est revenu. Si le refus persiste, conservez la demande et signalez ce code ; ne recréez pas la mission."
            }
            "auxiliary_identity_unproven : credential_mismatch" => {
                "binding_registration_refused : credential_mismatch : la preuve locale ne correspond plus au lien du fil"
            }
            "auxiliary_identity_unproven : instance_mismatch" => {
                "binding_registration_refused : instance_mismatch : le lien appartient à une autre instance"
            }
            "auxiliary_identity_unproven : primary_not_live" => {
                "binding_registration_refused : primary_not_live : le lien du fil n’est plus actif. Attendez sa reconnexion, puis reprenez la lecture avec bridget_task_status et le même task_id. Si le refus persiste, conservez la demande et signalez ce code ; ne recréez pas la mission."
            }
            "auxiliary_identity_unproven : connection_already_registered" => {
                "binding_registration_refused : connection_already_registered : la connexion est déjà enregistrée"
            }
            _ => "binding_registration_refused : le démon refuse le rattachement de cette session",
        },
        ClientError::Technical { code: "auxiliary_credential_required", .. } =>
            "binding_credential_unavailable : rattachement Bridget local absent ou invalide",
        ClientError::Technical { code: "daemon_protocol", .. } =>
            "binding_protocol_invalid : réponse du démon invalide",
        _ => "binding_transport_unavailable : connexion au démon interrompue ou délai dépassé ; aucune révocation de session établie. Attendez le retour du lien, puis reprenez la lecture avec bridget_task_status et le même task_id. Sans task_id, conservez request_id et tous les paramètres pour retrouver le sort de la demande. Si le refus persiste, signalez ce code ; ne recréez pas la mission et ne changez pas les permissions.",
    })
}

fn validate_endpoint(
    runtime: &ServerRuntime,
    endpoint: &str,
    authorization: &str,
) -> Result<String, IdentityError> {
    let allowed = [
        format!("http://127.0.0.1:{}/mcp", runtime.port),
        format!("http://localhost:{}/mcp", runtime.port),
        format!("http://[::1]:{}/mcp", runtime.port),
    ];
    let token = authorization.strip_prefix("Bearer ").unwrap_or("");
    if !allowed.iter().any(|value| value == endpoint)
        || !(32..=256).contains(&token.len())
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(IdentityError::T3SessionUnavailable);
    }
    Ok(format!("{}/mcp", runtime.base_url()))
}

fn attest_http(url: &str, authorization: &str) -> Result<SessionProof, IdentityError> {
    let mut session_id = None;
    let proof = attest(|body| post(url, authorization, &mut session_id, body));
    // Une session de transport par attestation. Ne jamais conserver le jeton
    // ni accumuler des sessions HTTP lors des appels successifs de la façade.
    if let Some(session_id) = session_id {
        let cleanup = minreq::delete(url)
            .with_header("Authorization", authorization)
            .with_header("MCP-Protocol-Version", "2025-06-18")
            .with_header("Mcp-Session-Id", session_id)
            .with_max_redirects(0)
            .with_timeout(ATTESTATION_CLEANUP_TIMEOUT_SECS)
            .send_lazy()
            .map_err(|_| IdentityError::T3SessionUnavailable)
            .and_then(|response| {
                if (200..300).contains(&response.status_code) {
                    Ok(())
                } else {
                    Err(IdentityError::T3SessionUnavailable)
                }
            });
        // La fermeture du transport ne révoque pas une preuve déjà vérifiée.
        // Le prochain appel refait son attestation ; aucun jeton n'est conservé.
        if cleanup.is_err() {
            eprintln!("bridget_t3_session cleanup=failed attestation_preserved={}", proof.is_ok());
        }
    }
    proof
}

fn post(
    url: &str,
    authorization: &str,
    session_id: &mut Option<String>,
    body: &Value,
) -> Result<Value, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    let mut request = minreq::post(url)
        .with_header("Authorization", authorization)
        .with_header("Accept", "application/json, text/event-stream")
        .with_header("Content-Type", "application/json")
        .with_header("MCP-Protocol-Version", "2025-06-18")
        .with_max_redirects(0)
        .with_timeout(ATTESTATION_HTTP_TIMEOUT_SECS)
        .with_body(body.to_string());
    if let Some(session_id) = session_id.as_ref() {
        request = request.with_header("Mcp-Session-Id", session_id);
    }
    let response = request.send_lazy().map_err(|_| IdentityError::T3SessionUnavailableAt(
        "attestation_transport_unavailable : transport T3 indisponible ou délai HTTP de 10 s dépassé"))?;
    if !(200..300).contains(&response.status_code) {
        return Err(IdentityError::T3SessionUnavailableAt(match response.status_code {
            401 | 403 => "attestation_http_unauthorized : T3 refuse l’authentification de cette session",
            404 => "attestation_http_not_found : session ou port T3 introuvable",
            _ => "attestation_http_refused : T3 refuse la requête HTTP",
        }));
    }
    if body.get("method").and_then(Value::as_str) == Some("initialize") {
        let mut issued_values = response
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("mcp-session-id"))
            .map(|(_, value)| value);
        let issued = issued_values.next().ok_or_else(denied)?;
        if issued_values.next().is_some() {
            return Err(denied());
        }
        if issued.is_empty()
            || issued.len() > 256
            || !issued
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(denied());
        }
        *session_id = Some(issued.clone());
    }
    let mut bytes = Vec::new();
    Read::take(response, MAX_RESPONSE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| denied())?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err(denied());
    }
    if body.get("id").is_none() && bytes.is_empty() {
        return Ok(Value::Null);
    }
    decode_response(&bytes)
}

fn decode_response(bytes: &[u8]) -> Result<Value, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    if let Ok(value) = serde_json::from_slice(bytes) {
        return Ok(value);
    }
    // Streamable HTTP peut rendre l'unique réponse sous forme d'événement SSE.
    let text = std::str::from_utf8(bytes).map_err(|_| denied())?;
    let data: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .collect();
    if data.len() != 1 {
        return Err(denied());
    }
    serde_json::from_str(data[0]).map_err(|_| denied())
}

fn attest(
    mut exchange: impl FnMut(&Value) -> Result<Value, IdentityError>,
) -> Result<SessionProof, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    let init = exchange(
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"bridget-session","version":"1"}
        }}),
    )?;
    if init.get("id") != Some(&json!(1))
        || init
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str)
            != Some("2025-06-18")
        || init.get("error").is_some()
    {
        return Err(denied());
    }
    exchange(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let response = exchange(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_session","arguments":{}}}),
    )?;
    if response.pointer("/result/isError") == Some(&Value::Bool(true)) {
        return Err(attestation_failure(&response));
    }
    if response.get("id") != Some(&json!(2))
        || response.get("error").is_some()
        || response.pointer("/result/isError") == Some(&Value::Bool(true))
    {
        return Err(denied());
    }
    let proof: SessionProof = serde_json::from_value(
        response
            .pointer("/result/structuredContent")
            .cloned()
            .ok_or_else(denied)?,
    )
    .map_err(|_| denied())?;
    proof.validate().map_err(|_|denied())?;
    Ok(proof)
}

fn attestation_failure(response: &Value) -> IdentityError {
    // T3 exposes typed failures in structuredContent or a JSON text block.
    // Never pass through its free-form message, credentials or arbitrary fields.
    let mut values = vec![response.pointer("/result/structuredContent").cloned().unwrap_or(Value::Null)];
    if let Some(content) = response.pointer("/result/content").and_then(Value::as_array) {
        for item in content.iter().take(8) {
            if let Some(text) = item.get("text").and_then(Value::as_str).filter(|s| s.len() <= 4096)
                && let Ok(value) = serde_json::from_str::<Value>(text) { values.push(value); }
        }
    }
    for value in values {
        match value.get("code").and_then(Value::as_str) {
            Some("permission_attestation_unavailable") => return IdentityError::T3SessionUnavailableAt(
                "permission_attestation_unavailable : T3 refuse le fait de droits du tour courant ; il est absent, modifié ou ne lui appartient plus"),
            Some("parent_not_active") => return IdentityError::T3SessionUnavailableAt(
                "parent_not_active : ce fournisseur ne possède plus le tour actif"),
            _ => {}
        }
    }
    IdentityError::T3SessionUnavailableAt("attestation_tool_refused : T3 a refusé l’attestation sans motif reconnu")
}

#[cfg(test)]
#[path = "t3code_mcp_tests.rs"]
mod tests;
