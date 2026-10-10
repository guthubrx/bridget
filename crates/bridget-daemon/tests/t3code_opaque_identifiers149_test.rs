//! Session149 : un identifiant de fil T3 est opaque. Le client doit l'encoder
//! en UN segment de route, T3 le décode UNE fois, et la valeur d'origine reste
//! celle que `parse_detail` (V2) compare. Serveur HTTP/WebSocket loopback réel,
//! aucune implémentation simulée : le test lit la cible brute de la requête.

use bridget_daemon::t3code_contract::{Client, ContractError};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
enum Protocol {
    V1,
    V2,
}

/// Ce que le « serveur T3 » a réellement reçu pour une route de fil.
#[derive(Debug)]
struct ThreadRequest {
    /// Cible HTTP brute : `/api/orchestration/threads/...`.
    target: String,
    /// Segment brut entre `/threads/` et le suffixe ou la requête.
    raw_segment: String,
    /// Segment après UN décodage pourcentage.
    decoded_once: String,
    /// Reste de la cible après le segment (`/bounded`, `?turnLimit=N`, ``).
    tail: String,
}

struct Mock {
    base_url: String,
    requests: Arc<Mutex<Vec<ThreadRequest>>>,
    connections: Arc<Mutex<usize>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Mock {
    fn start(protocol: Protocol, served_id: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let connections = Arc::new(Mutex::new(0usize));
        let stop = Arc::new(AtomicBool::new(false));
        let (r, c, s, id) = (
            requests.clone(),
            connections.clone(),
            stop.clone(),
            served_id.to_string(),
        );
        let handle = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(30);
            while !s.load(Ordering::SeqCst) && Instant::now() < deadline {
                match listener.accept() {
                    Ok((stream, _)) => {
                        *c.lock().unwrap() += 1;
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(5)))
                            .unwrap();
                        serve(stream, protocol, &id, &r);
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(5)),
                }
            }
        });
        Self {
            base_url,
            requests,
            connections,
            stop,
            handle: Some(handle),
        }
    }

    fn client(&self) -> Client {
        Client::for_base_url(&self.base_url, "fixture")
    }

    fn thread_requests(&self) -> std::sync::MutexGuard<'_, Vec<ThreadRequest>> {
        self.requests.lock().unwrap()
    }

    fn connection_count(&self) -> usize {
        *self.connections.lock().unwrap()
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn percent_decode_once(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = std::str::from_utf8(&bytes[i + 1..(i + 3).min(bytes.len())]).unwrap();
            assert_eq!(hex.len(), 2, "pourcentage tronqué dans {text}");
            out.push(
                u8::from_str_radix(hex, 16).unwrap_or_else(|_| panic!("hex invalide dans {text}")),
            );
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).expect("segment décodé non UTF-8")
}

fn serve(
    mut stream: TcpStream,
    protocol: Protocol,
    served_id: &str,
    requests: &Arc<Mutex<Vec<ThreadRequest>>>,
) {
    let mut probe = [0u8; 8];
    let seen = stream.peek(&mut probe).unwrap_or(0);
    if probe[..seen].starts_with(b"GET /ws?") {
        let mut socket = tungstenite::accept(stream).unwrap();
        let frame: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(frame["tag"], "server.getConfig");
        socket
            .send(tungstenite::Message::Text(
                json!([{"_tag":"Exit","requestId":"bridget","exit":{"_tag":"Success",
                    "value":{"providers":[{"instanceId":"claude_glm","driver":"claude"}]}}}])
                .to_string()
                .into(),
            ))
            .unwrap();
        let _ = socket.close(None);
        return;
    }
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
            break;
        }
        headers.push_str(&line);
    }
    let target = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string();
    let (status, body) = match target.as_str() {
        "/.well-known/t3/environment" => (
            200,
            match protocol {
                Protocol::V1 => json!({"environmentId":"fixture","serverVersion":"fixture"}),
                Protocol::V2 => json!({"environmentId":"fixture","serverVersion":"fixture",
                    "orchestrationProtocolVersion":2}),
            }
            .to_string(),
        ),
        "/api/orchestration/snapshot" => (
            200,
            json!({"snapshotSequence":1,"projects":[],"threads":[]}).to_string(),
        ),
        "/api/orchestration/shell" => (
            200,
            json!({"schemaVersion":2,"snapshotSequence":17,"projects":[],"threads":[]}).to_string(),
        ),
        t if t.starts_with("/api/orchestration/threads/") => {
            assert!(
                headers
                    .to_lowercase()
                    .contains("authorization: bearer fixture"),
                "jeton absent d'une lecture de fil"
            );
            let rest = &t["/api/orchestration/threads/".len()..];
            let end = rest.find(['/', '?']).unwrap_or(rest.len());
            let raw_segment = rest[..end].to_string();
            requests.lock().unwrap().push(ThreadRequest {
                target: t.to_string(),
                decoded_once: percent_decode_once(&raw_segment),
                raw_segment,
                tail: rest[end..].to_string(),
            });
            // T3 sert le fil dont il a décodé l'identifiant UNE fois.
            let id = percent_decode_once(&rest[..end]);
            if id != served_id {
                (404, "{}".to_string())
            } else {
                (
                    200,
                    match protocol {
                        Protocol::V1 => json!({"thread":{"messages":[
                            {"id":"m","role":"user","text":id,"streaming":false}]}}),
                        Protocol::V2 => json!({"snapshotSequence":18,"projection":{
                            "thread":{"id":served_id},
                            "runs":[{"id":"r","threadId":served_id,"ordinal":1,"userMessageId":"u",
                                "rootNodeId":"root","status":"completed",
                                "requestedAt":"2026-10-09T00:00:00.000Z"}],
                            "nodes":[{"id":"root","kind":"root_turn","parentNodeId":null}],
                            "messages":[{"id":"u","threadId":served_id,"runId":"r","nodeId":null,
                                "role":"user","text":"question","streaming":false,
                                "createdAt":"2026-10-09T00:00:00.000Z"}],
                            "turnItems":[],"runtimeRequests":[],"visibleTurnItems":[]}}),
                    }
                    .to_string(),
                )
            }
        }
        _ => (404, "{}".to_string()),
    };
    write!(
        stream,
        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
}

/// Identifiants qui cassaient ou pouvaient casser la route : tous doivent
/// arriver intacts après un seul décodage.
fn opaque_ids() -> Vec<String> {
    let mut ids: Vec<String> = [
        "0b1f3c1e-8a4d-4c1e-9a54-2f6b0d7e9c11",
        "thread:delegated-task:command%3Amcp%3Ax",
        "delegated%3Aabc",
        "delegated%3A",
        "a%2Fb",
        "a%2fb",
        "a%25b",
        "a%zz",
        "a%",
        "a/b",
        "a/../b",
        "a?b=c",
        "a#frag",
        "a b",
        "a+b",
        "a\\b",
        "a\"b",
        "é日本語😀",
        "..%2F..%2Fetc",
        "...",
        ".hidden",
        "thread:delegated-task:",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    ids.push("x".repeat(2048));
    ids
}

fn assert_single_segment(request: &ThreadRequest, original: &str) {
    assert_eq!(
        request.decoded_once, original,
        "l'identifiant ne survit pas à un décodage unique : {}",
        request.target
    );
    assert!(
        !request.raw_segment.contains(['/', '?', '#', ' ', '\\']),
        "segment non unique : {}",
        request.target
    );
    assert!(!request.raw_segment.is_empty());
    assert!(
        request.raw_segment.is_ascii(),
        "octet non ASCII brut dans la route : {}",
        request.target
    );
    // Chaque `%` brut de l'original doit être lui-même encodé (`%25`).
    if original.contains('%') {
        assert!(request.raw_segment.contains("%25"), "{}", request.target);
    }
}

#[test]
fn spec149_v1_thread_id_is_one_opaque_route_segment() {
    for id in opaque_ids() {
        let mock = Mock::start(Protocol::V1, &id);
        let detail = mock.client().thread_detail(&id, 40);
        let detail = detail.unwrap_or_else(|e| panic!("refus inattendu de {id:?} : {e:?}"));
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].text, id);
        let requests = mock.thread_requests();
        assert_eq!(requests.len(), 1, "une seule lecture de fil pour {id:?}");
        assert_single_segment(&requests[0], &id);
        assert_eq!(requests[0].tail, "?turnLimit=40");
    }
}

#[test]
fn spec149_v2_thread_id_is_encoded_and_original_reaches_parse_detail() {
    for id in opaque_ids() {
        let mock = Mock::start(Protocol::V2, &id);
        let client = mock.client();
        // 20 : route bornée ; 21 : route complète. Le fil servi garde l'id d'origine.
        for (limit, tail) in [(1u32, "/bounded"), (20, "/bounded"), (21, ""), (200, "")] {
            let detail = client
                .thread_detail(&id, limit)
                .unwrap_or_else(|e| panic!("refus inattendu de {id:?} limite {limit} : {e:?}"));
            assert_eq!(detail.messages.len(), 1);
            let requests = mock.thread_requests();
            let last = requests.last().unwrap();
            assert_single_segment(last, &id);
            assert_eq!(last.tail, tail, "limite {limit}");
        }
    }
}

#[test]
fn spec149_encoded_form_in_the_id_is_not_decoded_by_the_client() {
    // Cause 2 : `%3A` dans un childThreadId doit devenir `%253A` sur le fil.
    let id = "thread:delegated-task:command%3Amcp%3Ax";
    for protocol in [Protocol::V1, Protocol::V2] {
        let mock = Mock::start(protocol, id);
        mock.client().thread_detail(id, 20).unwrap();
        let requests = mock.thread_requests();
        assert!(
            requests[0].raw_segment.contains("%253Amcp%253Ax"),
            "{}",
            requests[0].target
        );
        assert!(
            !requests[0].raw_segment.contains("%3Amcp"),
            "{}",
            requests[0].target
        );
        // Tout `%` brut non suivi de deux chiffres hexadécimaux serait une route invalide.
        let raw = requests[0].raw_segment.as_bytes();
        for (i, b) in raw.iter().enumerate() {
            if *b == b'%' {
                assert!(raw[i + 1..].len() >= 2);
                assert!(raw[i + 1..i + 3].iter().all(u8::is_ascii_hexdigit));
            }
        }
    }
}

#[test]
fn spec149_rejected_ids_never_reach_the_network() {
    let too_long = "x".repeat(2049);
    let rejected: Vec<&str> = vec![
        "",
        ".",
        "..",
        "a\nb",
        "a\rb",
        "a\0b",
        "a\tb",
        "a\u{7f}b",
        "a\u{85}b",
        "a\u{202e}b",
        "a\u{200b}b",
        "a\u{2066}b",
        &too_long,
    ];
    for protocol in [Protocol::V1, Protocol::V2] {
        let mock = Mock::start(protocol, "unused");
        let client = mock.client();
        for id in &rejected {
            for limit in [20, 40] {
                match client.thread_detail(id, limit) {
                    Err(ContractError::InvalidShape { source, field }) => {
                        assert_eq!(source, "thread_detail", "{id:?}");
                        assert_eq!(field, "threadId", "{id:?}");
                    }
                    other => panic!("{id:?} aurait dû être refusé : {:?}", other.map(|_| ())),
                }
            }
        }
        assert_eq!(
            mock.connection_count(),
            0,
            "une requête est partie pour un id refusé"
        );
    }
}

#[test]
fn spec149_dot_segments_stay_literal_when_embedded() {
    // `.` et `..` seuls sont refusés ; embarqués, ils restent des octets littéraux
    // d'un seul segment (jamais un chemin relatif réinterprété).
    for id in ["a/..", "../a", "a/./b", "%2e%2e", "..a", "a.."] {
        let mock = Mock::start(Protocol::V1, id);
        mock.client().thread_detail(id, 20).unwrap();
        let requests = mock.thread_requests();
        assert_single_segment(&requests[0], id);
        assert_ne!(requests[0].raw_segment, ".");
        assert_ne!(requests[0].raw_segment, "..");
    }
}

#[test]
fn spec149_unknown_thread_is_an_http_refusal_not_a_masked_success() {
    let mock = Mock::start(Protocol::V1, "other-thread");
    match mock.client().thread_detail("thread:missing%3A1", 20) {
        Err(ContractError::Http { status: 404, .. }) => {}
        other => panic!("404 attendu, reçu {:?}", other.map(|_| ())),
    }
}
