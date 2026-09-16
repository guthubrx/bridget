//! SC08916 : contenu exact par la socket publique, sans HTTP ni fournisseur.
//! Les accès directs au magasin ci-dessous sont exclusivement des préparations
//! et contre-oracles de test (injection d'un blob, corruption attestée).
use bridget_daemon::artifact_blob_store::ArtifactBlobStore;
use bridget_daemon::artifact_types::{ArtifactPublicationV1, ArtifactReceiptV1, sha256_hex};
use bridget_transport::protocol::{
    ARTIFACT_READ_VERSION, ArtifactReadKind, ArtifactReadOutcome, ArtifactReadRefusal,
    ArtifactReadRequest, MAX_ARTIFACT_READ_BYTES, PresenceMode, decode, encode,
};
use bridget_transport::{ChannelReport, DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const A: &str = "89000000-0000-4000-8000-000000000011";
const B: &str = "89000000-0000-4000-8000-000000000012";

struct Fixture {
    root: PathBuf,
    daemon: Option<Child>,
    deadline: Instant,
}
impl Fixture {
    fn new() -> Self {
        let root = fs::canonicalize("/tmp")
            .unwrap()
            .join(format!("bc-read-{}", uuid::Uuid::new_v4().simple()));
        for path in [
            &root,
            &root.join("state"),
            &root.join("home"),
            &root.join("tmp"),
        ] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("daemon.log"))
            .unwrap();
        let mut fixture = Self {
            root,
            daemon: None,
            deadline: Instant::now() + Duration::from_secs(45),
        };
        fixture.daemon = Some(
            Command::new(env!("CARGO_BIN_EXE_bridget"))
                .arg("daemon")
                .env_clear()
                .env("HOME", fixture.root.join("home"))
                .env("TMPDIR", fixture.root.join("tmp"))
                .env("BRIDGET_HOME", fixture.root.join("state"))
                .env("BRIDGET_SOCKET", fixture.socket())
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if UnixStream::connect(fixture.socket()).is_ok() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "daemon non prêt: {}",
                fs::read_to_string(fixture.root.join("daemon.log")).unwrap_or_default()
            );
            assert!(
                fixture
                    .daemon
                    .as_mut()
                    .unwrap()
                    .try_wait()
                    .unwrap()
                    .is_none()
            );
            std::thread::sleep(Duration::from_millis(5)); // watchdog de readiness, pas oracle métier
        }
        fixture
    }
    fn socket(&self) -> PathBuf {
        self.root.join("state/s")
    }
    fn db(&self) -> PathBuf {
        self.root.join("state/bridget.db")
    }
    fn blobs(&self) -> ArtifactBlobStore {
        ArtifactBlobStore::open(self.root.join("state/artifacts"), Default::default()).unwrap()
    }
    fn blob_path(&self, digest: &str) -> PathBuf {
        self.root
            .join("state/artifacts/blobs")
            .join(&digest[..2])
            .join(digest)
    }
    fn client(&self) -> Client {
        Client::connect(&self.socket(), self.deadline)
    }
    fn install_identity_marker(&self, agent: &str, instance: &str) {
        let dir = self.root.join("state/agent-pids");
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)
            .unwrap();
        let name_file = self.root.join("state/reader-identity");
        private_file(&name_file, agent.as_bytes());
        let pid = std::process::id();
        let marker = bridget_daemon::mcp_identity::AgentPidMarker {
            pid,
            birth: bridget_daemon::managed_process::process_birth(pid).unwrap(),
            instance_id: instance.into(),
            name_file,
        };
        private_file(
            &dir.join(pid.to_string()),
            &serde_json::to_vec(&marker).unwrap(),
        );
    }
    fn run_binary(&self, args: &[&str], input: &[u8]) -> Output {
        // Fichiers privés, pas de pipes saturables pendant l'attente.
        let prefix = self.root.join(uuid::Uuid::new_v4().simple().to_string());
        let stdin = prefix.with_extension("stdin");
        let stdout = prefix.with_extension("stdout");
        let stderr = prefix.with_extension("stderr");
        private_file(&stdin, input);
        let out = private_file(&stdout, &[]);
        let err = private_file(&stderr, &[]);
        let mut guard = BinaryChild(Some(
            Command::new(env!("CARGO_BIN_EXE_bridget"))
                .args(args)
                .env_clear()
                .env("HOME", self.root.join("home"))
                .env("TMPDIR", self.root.join("tmp"))
                .env("BRIDGET_HOME", self.root.join("state"))
                .env("BRIDGET_SOCKET", self.socket())
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .stdin(fs::File::open(stdin).unwrap())
                .stdout(out)
                .stderr(err)
                .spawn()
                .unwrap(),
        ));
        let deadline = self.deadline.min(Instant::now() + Duration::from_secs(12));
        let status = loop {
            if let Some(status) = guard.0.as_mut().unwrap().try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "client réel hors budget global");
            std::thread::sleep(Duration::from_millis(5)); // watchdog, pas oracle métier
        };
        guard.0.take(); // déjà récolté
        assert!(fs::metadata(&stdout).unwrap().len() <= 256 * 1024);
        assert!(fs::metadata(&stderr).unwrap().len() <= 256 * 1024);
        Output {
            status,
            stdout: fs::read(stdout).unwrap(),
            stderr: fs::read(stderr).unwrap(),
        }
    }
    fn stored_manifest(&self, version: &str) -> Vec<u8> {
        rusqlite::Connection::open(self.db())
            .unwrap()
            .query_row(
                "SELECT manifest_json FROM artifact_versions WHERE version_ref=?1",
                [version],
                |row| row.get(0),
            )
            .unwrap()
    }
}

#[test]
fn html_au_plafond_entree_reste_lisible_apres_enrichissement_canonique_historique() {
    let f = Fixture::new();
    let mut owner = Client::agent(&f, A, "limit-html", "codex");
    let mut html = chart();
    html.kind = bridget_daemon::artifact_types::ArtifactKind::Html;
    html.idempotency_key = "html-limit-exact".into();
    html.payload = serde_json::json!({"html":""});
    let limit = bridget_daemon::artifact_policy::ArtifactPolicy::default().manifest_max_bytes;
    let padding = limit.checked_sub(html.canonical_bytes().len()).unwrap();
    html.payload["html"] = serde_json::json!("x".repeat(padding));
    assert_eq!(html.canonical_bytes().len(), limit);
    let receipt = owner.publish(&html);
    let stored = f.stored_manifest(&receipt.version_ref);
    assert!(
        stored.len() > limit,
        "enrichissement APRÈS plafond d'entrée réellement exercé"
    );
    assert!(
        stored.len() <= limit + 256,
        "borne des trois métadonnées historiques"
    );
    // Mutation : rétablir le plafond d'entrée comme plafond de lecture refuse
    // ce contenu légitimement publié. On ne recanonise jamais son histoire.
    all_bytes(&mut owner, &receipt, ArtifactReadKind::Manifest, &stored);
}

#[test]
fn cli_et_mcp_reels_lisent_les_memes_octets_sans_droit_supplementaire() {
    let f = Fixture::new();
    let mut owner = Client::agent(&f, A, "reader-instance", "codex");
    let mut other = Client::agent(&f, B, "other-instance", "codex");
    f.install_identity_marker(A, "reader-instance");
    let mine = owner.publish(&chart());
    let theirs = other.publish(&chart());
    for (receipt, expected_reason) in [
        (&mine, None),
        (&theirs, Some(ArtifactReadRefusal::NotFound)),
    ] {
        let output = f.run_binary(
            &[
                "artifact",
                "read",
                "--artifact-ref",
                &receipt.artifact_ref,
                "--version-ref",
                &receipt.version_ref,
                "--limit",
                "4096",
            ],
            &[],
        );
        assert_eq!(
            output.status.success(),
            expected_reason.is_none(),
            "CLI: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let cli: DaemonToWrapper = serde_json::from_slice(&output.stdout).unwrap();
        let query = request(receipt, ArtifactReadKind::Manifest, 0, 4096);
        let frames = [
            serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
            serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_read_artifact","arguments":query}}),
        ];
        let input = frames
            .iter()
            .map(|frame| format!("{frame}\n"))
            .collect::<String>();
        let output = f.run_binary(&["mcp"], input.as_bytes());
        assert!(
            output.status.success(),
            "MCP: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let replies: Vec<serde_json::Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let response = replies.iter().find(|frame| frame["id"] == 2).unwrap();
        assert!(response.get("error").is_none(), "{response}");
        let mcp: DaemonToWrapper =
            serde_json::from_value(response["result"]["structuredContent"].clone()).unwrap();
        assert_eq!(
            encode(&cli).unwrap(),
            encode(&mcp).unwrap(),
            "façades réelles : même contrat, même refus, mêmes octets"
        );
        let DaemonToWrapper::ArtifactReadResult { outcome, .. } = cli else {
            panic!("lecture attendue")
        };
        match expected_reason {
            Some(reason) => refused(outcome, reason),
            None => {
                let ArtifactReadOutcome::Chunk { bytes, digest, .. } = outcome else {
                    panic!("contenu attendu")
                };
                assert_eq!(bytes, f.stored_manifest(&mine.version_ref));
                assert_eq!(digest, sha256_hex(&bytes));
            }
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(child) = self.daemon.as_mut() {
            if matches!(child.try_wait(), Ok(None)) {
                let observed = Command::new("/bin/ps")
                    .args(["-p", &child.id().to_string(), "-o", "command="])
                    .output();
                let owned = observed.is_ok_and(|out| {
                    out.status.success()
                        && String::from_utf8_lossy(&out.stdout).trim()
                            == format!("{} daemon", env!("CARGO_BIN_EXE_bridget"))
                });
                if owned {
                    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
                    let deadline = Instant::now() + Duration::from_secs(8);
                    while matches!(child.try_wait(), Ok(None)) && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
            }
            if !matches!(child.try_wait(), Ok(Some(_))) {
                eprintln!(
                    "enfant de test non récolté : {} — racine conservée {}",
                    child.id(),
                    self.root.display()
                );
                return;
            }
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn private_file(path: &Path, bytes: &[u8]) -> fs::File {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file
}
struct BinaryChild(Option<Child>);
impl Drop for BinaryChild {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if !matches!(child.try_wait(), Ok(None)) {
            return;
        }
        // Handle non récolté : PID non réutilisable. Vérifier aussi la commande.
        let owned = Command::new("/bin/ps")
            .args(["-p", &child.id().to_string(), "-o", "command="])
            .output()
            .is_ok_and(|out| {
                out.status.success()
                    && String::from_utf8_lossy(&out.stdout)
                        .trim()
                        .starts_with(&format!("{} ", env!("CARGO_BIN_EXE_bridget")))
            });
        if !owned {
            eprintln!("identité du fils invérifiable : {}", child.id());
            return;
        }
        let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
        let deadline = Instant::now() + Duration::from_secs(3);
        while matches!(child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if matches!(child.try_wait(), Ok(None)) {
            // Autorisation utilisateur limitée aux enfants isolés de ces tests.
            let _ = unsafe { libc::kill(child.id() as i32, libc::SIGKILL) };
            let _ = child.wait();
        }
    }
}
struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
    deadline: Instant,
    socket: PathBuf,
}
impl Client {
    fn connect(socket: &Path, deadline: Instant) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        Self {
            reader: BufReader::new(stream.try_clone().unwrap()),
            writer: BufWriter::new(stream),
            deadline,
            socket: socket.to_path_buf(),
        }
    }
    fn raw(&mut self, line: &str) -> DaemonToWrapper {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .expect("budget global de recette épuisé")
            .min(Duration::from_secs(5));
        self.reader
            .get_ref()
            .set_read_timeout(Some(remaining))
            .unwrap();
        self.writer
            .get_ref()
            .set_write_timeout(Some(remaining))
            .unwrap();
        writeln!(self.writer, "{line}").unwrap();
        self.writer.flush().unwrap();
        let mut reply = String::new();
        self.reader.read_line(&mut reply).unwrap();
        decode(reply.trim_end()).unwrap_or_else(|error| panic!("réponse {reply:?}: {error}"))
    }
    fn request(&mut self, message: WrapperToDaemon) -> DaemonToWrapper {
        self.raw(&encode(&message).unwrap())
    }
    fn register(&mut self, agent: &str, instance: &str, kind: &str) -> DaemonToWrapper {
        let proof_path = self
            .socket
            .parent()
            .unwrap()
            .join("agent-names")
            .join(format!("proof-{}.json", sha256_hex(instance.as_bytes())));
        if kind == "mcp" {
            let credential = fs::read(&proof_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                .and_then(|proof| serde_json::from_value(proof["credential"].clone()).ok())
                // Le scénario d'intrusion présente une preuve invalide au
                // vrai daemon ; il ne simule jamais le résultat du refus.
                .unwrap_or_else(|| {
                    bridget_transport::protocol::IdentityCredential::new(
                        "invalid-fixture-proof".into(),
                    )
                });
            return self.request(WrapperToDaemon::RegisterAuxiliary {
                agent_id: agent.into(),
                instance_id: instance.into(),
                credential,
            });
        }
        let response = self.request(WrapperToDaemon::Register {
            identity_version: 2,
            agent_type: kind.into(),
            agent_id: agent.into(),
            host: Some("fixture".into()),
            transport: Some("unix".into()),
            channel: ChannelReport::Known("unix".into()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".into()),
            instance_id: Some(instance.into()),
            domain: None,
            turn_in_progress: false,
            journal_available: Some(false),
        });
        if let DaemonToWrapper::Registered {
            credential: Some(credential),
            ..
        } = &response
        {
            bridget_transport::fsutil::write_private_file_atomic(
                &proof_path,
                &serde_json::to_vec(&serde_json::json!({
                    "agent_id": agent, "instance_id": instance, "credential": credential,
                }))
                .unwrap(),
            )
            .unwrap();
        }
        response
    }
    fn agent(fixture: &Fixture, agent: &str, instance: &str, kind: &str) -> Self {
        let mut client = fixture.client();
        let response = client.register(agent, instance, kind);
        assert!(
            matches!(response, DaemonToWrapper::Registered { ref agent_id, .. } if agent_id == agent),
            "{response:?}"
        );
        client
    }
    fn publish(&mut self, publication: &ArtifactPublicationV1) -> ArtifactReceiptV1 {
        let response = self.request(WrapperToDaemon::ArtifactPublish {
            contract_version: 1,
            canonical_publication: publication.canonical_bytes(),
        });
        let DaemonToWrapper::ArtifactPublicationResult {
            receipt_json: Some(receipt),
            refusal_code: None,
            ..
        } = response
        else {
            panic!("publication: {response:?}")
        };
        serde_json::from_slice(&receipt).unwrap()
    }
    fn read(&mut self, request: ArtifactReadRequest) -> ArtifactReadOutcome {
        let response = self.request(WrapperToDaemon::ArtifactRead {
            request: request.clone(),
        });
        let DaemonToWrapper::ArtifactReadResult {
            version,
            artifact_ref,
            version_ref,
            outcome,
        } = response
        else {
            panic!("lecture: {response:?}")
        };
        assert_eq!(version, ARTIFACT_READ_VERSION);
        assert_eq!(artifact_ref, request.artifact_ref);
        assert_eq!(version_ref, request.version_ref);
        outcome
    }
}
fn request(
    receipt: &ArtifactReceiptV1,
    kind: ArtifactReadKind,
    offset: u64,
    limit: u32,
) -> ArtifactReadRequest {
    ArtifactReadRequest {
        version: 1,
        artifact_ref: receipt.artifact_ref.clone(),
        version_ref: receipt.version_ref.clone(),
        kind,
        offset,
        limit,
    }
}
fn chart() -> ArtifactPublicationV1 {
    serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json")).unwrap()
}
fn file_publication(fixture: &Fixture, bytes: &[u8]) -> (ArtifactPublicationV1, String) {
    let digest = fixture.blobs().put(bytes).unwrap();
    let mut file: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/file-v1.json")).unwrap();
    file.payload["blob_digest"] = serde_json::json!(digest);
    file.payload["byte_length"] = serde_json::json!(bytes.len());
    file.sources[0].content_digest = Some(digest.clone());
    (file, digest)
}
fn all_bytes(
    client: &mut Client,
    receipt: &ArtifactReceiptV1,
    kind: ArtifactReadKind,
    expected: &[u8],
) {
    let mut collected = Vec::new();
    let mut offset = 0;
    loop {
        let ArtifactReadOutcome::Chunk {
            bytes,
            total_len,
            digest,
            offset: observed_offset,
            next_offset,
        } = client.read(request(receipt, kind.clone(), offset, 4096))
        else {
            panic!("fragment attendu")
        };
        assert_eq!(total_len, expected.len() as u64);
        assert_eq!(digest, sha256_hex(expected));
        assert_eq!(observed_offset, offset);
        assert!(bytes.len() <= 4096);
        collected.extend_from_slice(&bytes);
        match next_offset {
            Some(next) => {
                assert!(next > offset);
                offset = next;
            }
            None => break,
        }
    }
    assert_eq!(
        collected, expected,
        "octets originaux, pas seulement JSON équivalent"
    );
}
fn refused(outcome: ArtifactReadOutcome, reason: ArtifactReadRefusal) {
    assert_eq!(outcome, ArtifactReadOutcome::Rejected { reason });
}

#[test]
fn socket_reelle_preserve_manifest_provenance_blob_et_html_inerte() {
    let fixture = Fixture::new();
    let mut owner = Client::agent(&fixture, A, "instance-a", "codex");
    let mut auxiliary = Client::agent(&fixture, A, "instance-a", "mcp");
    let publication = chart();
    let receipt = auxiliary.publish(&publication); // même autorité pour les deux façades
    assert_eq!(
        fixture.stored_manifest(&receipt.version_ref),
        publication.canonical_bytes()
    );
    all_bytes(
        &mut auxiliary,
        &receipt,
        ArtifactReadKind::Manifest,
        &publication.canonical_bytes(),
    );
    let bytes: Vec<u8> = (0..40_123).map(|index| (index % 256) as u8).collect();
    let (file, digest) = file_publication(&fixture, &bytes);
    let file_receipt = owner.publish(&file);
    all_bytes(
        &mut auxiliary,
        &file_receipt,
        ArtifactReadKind::Blob { digest },
        &bytes,
    );
    let mut html = chart();
    html.idempotency_key = "html-read".into();
    html.kind = bridget_daemon::artifact_types::ArtifactKind::Html;
    html.payload = serde_json::json!({"html":"<main>Texte\n<script>alert('inerte')</script></main>","data":{}});
    let html_receipt = owner.publish(&html);
    let stored = fixture.stored_manifest(&html_receipt.version_ref);
    assert!(
        String::from_utf8_lossy(&stored).contains("runtime_policy"),
        "canon historique gardé"
    );
    all_bytes(
        &mut auxiliary,
        &html_receipt,
        ArtifactReadKind::Manifest,
        &stored,
    );
}

#[test]
fn lecture_refuse_autre_portee_version_et_digest_non_lie() {
    let fixture = Fixture::new();
    let mut owner = Client::agent(&fixture, A, "instance-a", "codex");
    let mut other = Client::agent(&fixture, B, "instance-b", "codex");
    let receipt = owner.publish(&chart());
    let other_receipt = other.publish(&chart());
    refused(
        other.read(request(&receipt, ArtifactReadKind::Manifest, 0, 100)),
        ArtifactReadRefusal::NotFound,
    );
    let mut mismatch = request(&receipt, ArtifactReadKind::Manifest, 0, 100);
    mismatch.version_ref = other_receipt.version_ref;
    refused(owner.read(mismatch), ArtifactReadRefusal::NotFound);
    let digest = fixture.blobs().put(b"present mais pas lie").unwrap();
    refused(
        owner.read(request(&receipt, ArtifactReadKind::Blob { digest }, 0, 100)),
        ArtifactReadRefusal::BlobNotLinked,
    );
}

#[test]
fn pagination_est_bornee_et_contrat_ferme() {
    let fixture = Fixture::new();
    let mut owner = Client::agent(&fixture, A, "instance-a", "codex");
    let receipt = owner.publish(&chart());
    for limit in [0, MAX_ARTIFACT_READ_BYTES + 1, u32::MAX] {
        refused(
            owner.read(request(&receipt, ArtifactReadKind::Manifest, 0, limit)),
            ArtifactReadRefusal::InvalidRequest,
        );
    }
    let length = chart().canonical_bytes().len() as u64;
    assert!(
        matches!(owner.read(request(&receipt, ArtifactReadKind::Manifest, length, 1)),
        ArtifactReadOutcome::Chunk { bytes, next_offset: None, .. } if bytes.is_empty())
    );
    for offset in [length + 1, u64::MAX] {
        refused(
            owner.read(request(&receipt, ArtifactReadKind::Manifest, offset, 1)),
            ArtifactReadRefusal::OffsetOutOfRange,
        );
    }
    let base = serde_json::to_value(WrapperToDaemon::ArtifactRead {
        request: request(&receipt, ArtifactReadKind::Manifest, 0, 100),
    })
    .unwrap();
    for position in ["root", "request", "kind"] {
        let mut value = base.clone();
        match position {
            "root" => value["future"] = true.into(),
            "request" => value["request"]["future"] = true.into(),
            _ => value["request"]["kind"] = serde_json::json!({"kind":"executable"}),
        }
        assert!(matches!(
            owner.raw(&value.to_string()),
            DaemonToWrapper::ArtifactReadResult {
                outcome: ArtifactReadOutcome::Rejected {
                    reason: ArtifactReadRefusal::InvalidRequest
                },
                ..
            }
        ));
    }
    let mut newer = request(&receipt, ArtifactReadKind::Manifest, 0, 100);
    newer.version = 2;
    refused(owner.read(newer), ArtifactReadRefusal::UnsupportedVersion);
}

#[test]
fn disparition_et_corruption_ne_sont_jamais_cachees_par_un_succes_precedent() {
    let fixture = Fixture::new();
    let mut owner = Client::agent(&fixture, A, "instance-a", "codex");
    let bytes = b"contenu originel";
    let (file, digest) = file_publication(&fixture, bytes);
    let receipt = owner.publish(&file);
    let req = request(
        &receipt,
        ArtifactReadKind::Blob {
            digest: digest.clone(),
        },
        0,
        100,
    );
    all_bytes(&mut owner, &receipt, req.kind.clone(), bytes);
    fs::write(fixture.blob_path(&digest), b"contenu modifie!").unwrap();
    refused(owner.read(req.clone()), ArtifactReadRefusal::CorruptContent);
    fs::remove_file(fixture.blob_path(&digest)).unwrap();
    refused(owner.read(req), ArtifactReadRefusal::ContentUnavailable);
    let connection = rusqlite::Connection::open(fixture.db()).unwrap();
    connection
        .execute(
            "UPDATE artifact_versions SET manifest_json=?1 WHERE version_ref=?2",
            rusqlite::params![b"{}".as_slice(), receipt.version_ref],
        )
        .unwrap();
    refused(
        owner.read(request(&receipt, ArtifactReadKind::Manifest, 0, 100)),
        ArtifactReadRefusal::CorruptContent,
    );
}

#[test]
fn auxiliaire_ne_survit_pas_a_sa_source_et_instance_divergente_refusee() {
    let fixture = Fixture::new();
    let owner = Client::agent(&fixture, A, "instance-a", "codex");
    let mut auxiliary = Client::agent(&fixture, A, "instance-a", "mcp");
    let receipt = auxiliary.publish(&chart());
    let mut wrong = fixture.client();
    assert!(matches!(
        wrong.register(A, "instance-intruse", "mcp"),
        DaemonToWrapper::Nack { .. }
    ));
    refused(
        wrong.read(request(&receipt, ArtifactReadKind::Manifest, 0, 100)),
        ArtifactReadRefusal::IdentityUnavailable,
    );
    drop(owner);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match auxiliary.read(request(&receipt, ArtifactReadKind::Manifest, 0, 100)) {
            ArtifactReadOutcome::Rejected {
                reason: ArtifactReadRefusal::IdentityUnavailable,
            } => break,
            ArtifactReadOutcome::Chunk { .. } => assert!(
                Instant::now() < deadline,
                "source disparue mais droit conservé"
            ),
            other => panic!("{other:?}"),
        }
    }
}
