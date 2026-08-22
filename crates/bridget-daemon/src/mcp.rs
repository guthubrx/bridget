//! Façade MCP stdio. Le protocole daemon reste le seul transport métier.

use serde_json::{Value, json};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

pub const PROTOCOL_VERSION: &str = "2025-06-18";

#[derive(Default)]
struct Session {
    initialize_seen: bool,
    initialized: bool,
}

/// Exécute le serveur MCP avec un lecteur unique et un writer unique.
///
/// L'ownership exclusif du `BufWriter` sérialise les sorties JSON-RPC : aucun
/// diagnostic ne peut rejoindre stdout, qui est réservé aux réponses MCP.
pub fn serve<R: BufRead, W: Write>(mut input: R, output: W) -> io::Result<()> {
    let mut output = BufWriter::new(output);
    let mut session = Session::default();
    let mut line = String::new();

    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return output.flush();
        }
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<Value>(line.trim_end()) {
            Ok(request) => dispatch(&request, &mut session),
            Err(_) => Some(error(Value::Null, -32700, "JSON malformé")),
        };
        if let Some(response) = response {
            write_response(&mut output, &response)?;
        }
    }
}

/// Lance la façade MCP depuis la sous-commande `bridget mcp`.
pub fn run_stdio() -> io::Result<()> {
    serve(BufReader::new(io::stdin().lock()), io::stdout().lock())
}

fn write_response(output: &mut impl Write, response: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, response).map_err(io::Error::other)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn dispatch(request: &Value, session: &mut Session) -> Option<Value> {
    dispatch_with_identity(request, session, &crate::mcp_identity::resolve_current)
}

fn dispatch_with_identity(
    request: &Value,
    session: &mut Session,
    resolve_identity: &impl Fn() -> Result<String, crate::mcp_identity::IdentityError>,
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
                Err(identity_error) => return id.map(|id| identity_error_result(id, &identity_error)),
            };
            id.map(|id| {
                result(
                    id,
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Outil Bridget indisponible avant la connexion au daemon pour {identity}.")
                        }],
                        "isError": true
                    }),
                )
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
            "name": "bridget_send",
            "description": "Envoyer un message Bridget à un équipier.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": { "type": "string", "minLength": 1 },
                    "body": { "type": "string", "minLength": 1 },
                    "reply": { "type": "boolean", "default": false },
                    "reply_timeout": { "type": "integer", "minimum": 1 }
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
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 20 }
                },
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

    const FIXTURES: &str = include_str!("../tests/fixtures/mcp/fr009.jsonl");

    fn run(lines: &[Value]) -> Vec<Value> {
        let input = lines
            .iter()
            .map(|line| line.as_str().map_or_else(|| line.to_string(), str::to_string))
            .collect::<Vec<_>>()
            .join("\n");
        let mut stdout = Vec::new();
        serve(input.as_bytes(), &mut stdout).unwrap();
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
                    assert_eq!(responses[0]["result"]["capabilities"], json!({ "tools": {} }));
                }
                "tools" => assert_eq!(responses.last().unwrap()["result"]["tools"].as_array().unwrap().len(), 3),
                "tools_twice" => assert_eq!(responses[1]["result"], responses[2]["result"]),
                "ping" => assert_eq!(responses.last().unwrap()["result"], json!({})),
                "tool_unavailable" => assert_eq!(responses.last().unwrap()["result"]["isError"], true),
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
        let mut session = Session { initialize_seen: true, initialized: true };
        let response = dispatch_with_identity(&request, &mut session, &|| {
            Err(crate::mcp_identity::IdentityError::LegacyMarker)
        }).unwrap();
        assert_eq!(response["result"]["code"], "legacy_marker");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn deux_appels_outil_ne_partagent_pas_l_identite_resolue() {
        let mut session = Session { initialize_seen: true, initialized: true };
        let calls = Cell::new(0);
        let resolver = || {
            calls.set(calls.get() + 1);
            Ok("agent".to_string())
        };
        for id in [3, 4] {
            let request = json!({
                "jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": { "name": "bridget_who", "arguments": {} }
            });
            let response = dispatch_with_identity(&request, &mut session, &resolver).unwrap();
            assert_eq!(response["result"]["isError"], true);
        }
        assert_eq!(calls.get(), 2);
    }
}
