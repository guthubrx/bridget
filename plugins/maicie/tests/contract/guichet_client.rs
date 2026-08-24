use maicie::bridget_client::{
    BridgetClientError, BridgetClientLimits, GuichetClient, GuichetLifecycleEvent,
};
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

const SCOPE: &str = "015_scope_0123456789abcdef0123456789abcdef";
const CLAIM_TOKEN: &str = "Q2xhaW0tdG9rZW4tMTI4LWJpdHM";
const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
    "../../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
);

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

        assert_eq!(
            read_json(&mut reader),
            json!({"type":"guichet_claim_next","v":1})
        );
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
    assert_eq!(
        client
            .reply_exact_bytes(reply)
            .expect("premiere issue")
            .issue,
        "outcome_unknown"
    );
    assert_eq!(
        client.reply_exact_bytes(reply).expect("retry exact").issue,
        "accepted"
    );
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
        let _ = write_json_checked(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_millis(80),
        io_timeout: Duration::from_millis(80),
        max_frame_bytes: 64 * 1024,
    };
    let deadline = Instant::now() + Duration::from_millis(35);
    let error =
        match GuichetClient::connect_with_limits_until(fixture.path(), SCOPE, limits, deadline) {
            Ok(_) => panic!("le role seul doit depasser le budget global"),
            Err(error) => error,
        };
    assert!(matches!(error, BridgetClientError::Timeout { .. }));
    server.join().expect("serveur termine");
}

/// Cette couture lance le vrai daemon et passe exclusivement par le client
/// public Maicie. Si l'un des deux côtés revient à `service_hello`, le client
/// attend indéfiniment une welcome PascalCase et l'échéance transforme le test
/// en échec : la mutation a été vérifiée avant la livraison du correctif.
#[test]
#[ignore = "requiert BRIDGET_MVP_GATE_BIN vers le binaire Bridget de ce worktree"]
fn client_public_et_daemon_reel_partagent_la_negociation_canonique() {
    let bridget = PathBuf::from(
        std::env::var_os("BRIDGET_MVP_GATE_BIN")
            .expect("BRIDGET_MVP_GATE_BIN doit désigner le binaire Bridget réel"),
    );
    let fixture = DaemonFixture::new(&bridget);
    let mut daemon = fixture.start();

    let mut client = GuichetClient::connect(fixture.socket(), SCOPE)
        .expect("RoleHandshake, ServiceHello et ServiceWelcome canoniques");
    assert_eq!(
        client
            .claim_next()
            .expect("claim autorise apres negociation"),
        None,
        "un guichet négocié mais vide ne doit pas être confondu avec un refus de capacité"
    );

    let stream = UnixStream::connect(fixture.socket()).expect("seconde connexion service");
    let (mut reader, mut writer) = split(stream);
    let lines = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
    write_raw(&mut writer, lines[0]);
    assert_eq!(read_raw(&mut reader), lines[1]);
    write_json(
        &mut writer,
        json!({
            "type":"ServiceHello",
            "version":1,
            "service":"maicie",
            "issuer_scope":SCOPE,
            "capabilities":[]
        }),
    );
    let welcome = read_json(&mut reader);
    assert_eq!(welcome["type"], "ServiceWelcome");
    assert_eq!(welcome["capabilities"], json!([]));
    write_json(&mut writer, json!({"type":"guichet_claim_next","v":1}));
    assert_eq!(read_raw(&mut reader), lines[4]);

    stop_daemon(&mut daemon);
}

fn assert_service_handshake(
    reader: &mut BufReader<UnixStream>,
    writer: &mut BufWriter<UnixStream>,
) {
    let fixture = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
    assert_eq!(read_raw(reader), fixture[0]);
    write_raw(writer, fixture[1]);
    assert_eq!(read_raw(reader), fixture[2]);
}

fn write_welcome(writer: &mut BufWriter<UnixStream>) {
    let fixture = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
    write_raw(writer, fixture[3]);
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

fn write_raw(writer: &mut BufWriter<UnixStream>, line: &str) {
    writer.write_all(line.as_bytes()).expect("ligne serveur");
    writer.write_all(b"\n").expect("delimiteur serveur");
    writer.flush().expect("flush serveur");
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

struct DaemonFixture {
    root: PathBuf,
    bridget: PathBuf,
    socket: PathBuf,
}

impl DaemonFixture {
    fn new(bridget: &Path) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        // Le socket Unix est borné (104 octets sur macOS). `temp_dir()` peut
        // contenir le préfixe long `/var/folders/...`, donc le harnais fixe
        // volontairement une racine courte et vérifie la borne avant le spawn.
        let root = PathBuf::from("/tmp").join(format!("mg-{}/{}", std::process::id(), sequence));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("home du daemon");
        let socket = root.join(".cache/bridget/bridget.sock");
        assert!(
            socket.as_os_str().len() < 104,
            "socket de harnais trop longue"
        );
        Self {
            socket,
            root,
            bridget: bridget.to_path_buf(),
        }
    }

    fn socket(&self) -> &Path {
        &self.socket
    }

    fn start(&self) -> Child {
        let child = Command::new(&self.bridget)
            .arg("daemon")
            .env_clear()
            .env("HOME", &self.root)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("daemon réel démarré");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.socket.exists() {
                return child;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let mut child = child;
        stop_daemon(&mut child);
        panic!("daemon réel non prêt sur {}", self.socket.display());
    }
}

impl Drop for DaemonFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stop_daemon(daemon: &mut Child) {
    if daemon.try_wait().expect("état daemon").is_some() {
        return;
    }
    assert_eq!(unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM) }, 0);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if daemon.try_wait().expect("attente daemon").is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("daemon réel ne s'arrête pas dans la borne");
}
