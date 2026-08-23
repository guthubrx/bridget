use maicie::bridget_client::{
    BridgetClientError, BridgetClientLimits, GuichetClient, GuichetLifecycleEvent,
};
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

const SCOPE: &str = "scope-guichet-0123456789abcdef";
const CLAIM_TOKEN: &str = "Q2xhaW0tdG9rZW4tMTI4LWJpdHM";

#[test]
fn exige_la_capacite_guichet_avant_toute_releve() {
    let fixture = SocketFixture::new("capability");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("service attendue");
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer);
        write_json(
            &mut writer,
            json!({
                "type":"ServiceWelcome",
                "version":1,
                "horizon_secs":60,
                "issued_at_tolerance_secs":5,
                "capabilities":[]
            }),
        );
    });

    let error = match GuichetClient::connect(fixture.path(), SCOPE) {
        Ok(_) => panic!("la capability guichet doit etre exigee"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        BridgetClientError::CapabilityMissing { capability } if capability == "maicie_guichet"
    ));
    server.join().expect("serveur termine");
}

#[test]
fn releve_fifo_repond_octet_pour_octet_et_lit_evenement_brigde() {
    let fixture = SocketFixture::new("claim-reply");
    let listener = fixture.bind();
    let reply = br#"{"type":"guichet_reply","v":1,"issuer_scope":"scope-guichet-0123456789abcdef","request_id":"req-01","claim_generation":1,"claim_token":"Q2xhaW0tdG9rZW4tMTI4LWJpdHM","response_message_id":"msg-02","in_reply_to":"msg-01","outcome":"accepted","payload":{"kind":"delivery_report","objective_id":"obj-01","delegation_id":"del-01","delivery_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}}"#;
    let expected_reply = String::from_utf8(reply.to_vec()).expect("utf8");
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("service attendue");
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer);
        write_welcome(&mut writer);

        assert_eq!(read_json(&mut reader), json!({"type":"guichet_claim_next","v":1}));
        write_json(&mut writer, claimed());

        assert_eq!(read_raw(&mut reader), expected_reply);
        write_json(&mut writer, result("accepted"));
        write_json(
            &mut writer,
            json!({
                "type":"request_lifecycle_event",
                "v":1,
                "issuer_scope":SCOPE,
                "event_id":"evt-01",
                "request_id":"req-01",
                "state":"answered",
                "observed_at":1_787_500_001_i64,
                "in_reply_to":"msg-01",
                "response_message_id":"msg-02"
            }),
        );
    });

    let mut client = GuichetClient::connect(fixture.path(), SCOPE).expect("negociation");
    let claim = client.claim_next().expect("claim").expect("entree FIFO");
    assert_eq!(claim.request_id, "req-01");
    assert_eq!(claim.claim_generation, 1);
    assert_eq!(claim.claim_token, CLAIM_TOKEN);

    let accepted = client.reply_exact_bytes(reply).expect("reponse exacte");
    assert_eq!(accepted.issue, "accepted");
    assert_eq!(accepted.expires_at, 1_787_500_300);
    assert!(matches!(
        client.next_lifecycle_event().expect("evenement Bridget"),
        GuichetLifecycleEvent { state, request_id, .. }
            if state == "answered" && request_id == "req-01"
    ));
    server.join().expect("serveur termine");
}

#[test]
fn retry_de_reponse_rejoue_les_memes_octets_sans_reserialisation() {
    let fixture = SocketFixture::new("reply-retry");
    let listener = fixture.bind();
    let reply = br#"{"type":"guichet_reply","v":1,"issuer_scope":"scope-guichet-0123456789abcdef","request_id":"req-01","claim_generation":1,"claim_token":"Q2xhaW0tdG9rZW4tMTI4LWJpdHM","response_message_id":"msg-02","in_reply_to":"msg-01","outcome":"accepted","payload":{"kind":"delivery_report"}}"#;
    let expected_reply = String::from_utf8(reply.to_vec()).expect("utf8");
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("service attendue");
        let (mut reader, mut writer) = split(stream);
        assert_service_handshake(&mut reader, &mut writer);
        write_welcome(&mut writer);
        assert_eq!(read_raw(&mut reader), expected_reply);
        write_json(&mut writer, result("outcome_unknown"));
        assert_eq!(read_raw(&mut reader), expected_reply);
        write_json(&mut writer, result("accepted"));
    });

    let mut client = GuichetClient::connect(fixture.path(), SCOPE).expect("negociation");
    assert_eq!(client.reply_exact_bytes(reply).expect("premiere issue").issue, "outcome_unknown");
    assert_eq!(client.reply_exact_bytes(reply).expect("retry exact").issue, "accepted");
    server.join().expect("serveur termine");
}

#[test]
fn chaque_phase_service_consomme_la_meme_echeance_absolue() {
    let fixture = SocketFixture::new("budget");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("service attendue");
        let (mut reader, mut writer) = split(stream);
        let _role = read_json(&mut reader);
        thread::sleep(Duration::from_millis(70));
        let _ = write_json_checked(
            &mut writer,
            json!({"type":"RoleAccepted","role":"service"}),
        );
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_millis(80),
        io_timeout: Duration::from_millis(80),
        max_frame_bytes: 64 * 1024,
    };
    let deadline = Instant::now() + Duration::from_millis(35);
    let error = match GuichetClient::connect_with_limits_until(fixture.path(), SCOPE, limits, deadline) {
        Ok(_) => panic!("le role seul doit depasser le budget global"),
        Err(error) => error,
    };
    assert!(matches!(error, BridgetClientError::Timeout { .. }));
    server.join().expect("serveur termine");
}

fn assert_service_handshake(reader: &mut BufReader<UnixStream>, writer: &mut BufWriter<UnixStream>) {
    assert_eq!(read_json(reader), json!({"type":"RoleHandshake","role":"service"}));
    write_json(writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["version"], 1);
    assert_eq!(hello["service"], "maicie");
    assert_eq!(hello["issuer_scope"], SCOPE);
    assert_eq!(hello["capabilities"], json!(["maicie_guichet"]));
}

fn write_welcome(writer: &mut BufWriter<UnixStream>) {
    write_json(
        writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":300,
            "issued_at_tolerance_secs":15,
            "capabilities":["maicie_guichet"]
        }),
    );
}

fn claimed() -> Value {
    json!({
        "type":"guichet_claimed",
        "v":1,
        "issuer_scope":SCOPE,
        "request_id":"req-01",
        "canonical_request":"eyJ0eXBlIjoic2VydmljZV9yZXF1ZXN0In0=",
        "claimed_at":1_787_500_000_i64,
        "claim_generation":1,
        "claim_token":CLAIM_TOKEN,
        "claim_lease_expires_at":1_787_500_060_i64,
        "expires_at":1_787_500_300_i64
    })
}

fn result(issue: &str) -> Value {
    json!({
        "type":"guichet_result",
        "v":1,
        "issuer_scope":SCOPE,
        "request_id":"req-01",
        "issue":issue,
        "expires_at":1_787_500_300_i64
    })
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let reader = BufReader::new(stream.try_clone().expect("clone socket"));
    (reader, BufWriter::new(stream))
}

fn read_raw(reader: &mut BufReader<UnixStream>) -> String {
    let mut line = String::new();
    reader.read_line(&mut line).expect("ligne client");
    line.strip_suffix('\n').unwrap_or(&line).to_string()
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    serde_json::from_str(&read_raw(reader)).expect("json client")
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    write_json_checked(writer, value).expect("json serveur")
}

fn write_json_checked(writer: &mut BufWriter<UnixStream>, value: Value) -> std::io::Result<()> {
    serde_json::to_writer(&mut *writer, &value)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

struct SocketFixture {
    path: PathBuf,
}

impl SocketFixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir().join(format!(
                "maicie-guichet-{label}-{}-{sequence}.sock",
                std::process::id()
            )),
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn bind(&self) -> UnixListener {
        let _ = fs::remove_file(&self.path);
        UnixListener::bind(&self.path).expect("socket fixture")
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
