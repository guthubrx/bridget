//! Couture du relais UI : HTTP loopback → projections publiques Bridget → Attach.

use bridget_core::BridgetMessage;
use bridget_daemon::ui::{UiRelay, UiRelayConfig};
use bridget_transport::protocol::{
    AgentInfo, LedgerScope, PresenceMode, StopOutcome, decode, encode,
};
use bridget_transport::{ChannelReport, DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct DaemonProcess(Child);

impl DaemonProcess {
    fn start(home: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("daemon")
            .env_clear()
            .env("HOME", home)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("daemon réel démarré");
        let daemon = Self(child);
        let socket = home.join(".cache/bridget/bridget.sock");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&socket).is_ok() {
                return daemon;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("daemon non prêt");
    }

    fn terminate(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
            let _ = self.0.wait();
        }
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct UiProcess {
    child: Child,
    address: SocketAddr,
    token: String,
}

impl UiProcess {
    fn start(home: &Path, environment_channel: Option<&str>) -> Self {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("port UI éphémère");
        let port = listener.local_addr().expect("adresse UI éphémère").port();
        let endpoint = home.join(".cache/bridget/ui-endpoint.json");
        std::fs::create_dir_all(endpoint.parent().expect("parent endpoint UI")).unwrap();
        std::fs::write(
            &endpoint,
            serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "port": port,
                "token": uuid::Uuid::new_v4().simple().to_string(),
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&endpoint, std::fs::Permissions::from_mode(0o600)).unwrap();
        drop(listener);
        let maicie_config = write_maicie_config(home);
        let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
        command
            .args(["ui", "--maicie-config"])
            .arg(&maicie_config)
            .env_clear()
            .env("HOME", home)
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(channel) = environment_channel {
            command.env("BRIDGET_CHANNEL", channel);
        }
        let mut child = command.spawn().expect("relais UI réel démarré");
        let stdout = child.stdout.take().expect("stdout UI capturé");
        let (line_sender, line_receiver) = std::sync::mpsc::channel();
        thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(stdout).read_line(&mut line);
            let _ = line_sender.send(line);
        });
        let line = line_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("URL du relais UI publiée");
        let url = line
            .split_once("http://")
            .map(|(_, url)| url.trim())
            .unwrap_or_else(|| panic!("URL UI absente de {line:?}"));
        let (address, token) = url
            .split_once("/?token=")
            .unwrap_or_else(|| panic!("URL UI inattendue: {url}"));
        Self {
            child,
            address: address.parse().expect("adresse loopback UI"),
            token: token.to_string(),
        }
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            assert_eq!(
                unsafe { libc::kill(self.child.id() as i32, libc::SIGTERM) },
                0
            );
            let _ = self.child.wait();
        }
    }
}

impl Drop for UiProcess {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct LiveAgent {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
}

impl LiveAgent {
    fn connect(socket: &Path, name: &str) -> Self {
        Self::connect_with_channel_report(socket, name, ChannelReport::Unknown)
    }

    fn connect_with_channel_report(socket: &Path, name: &str, channel: ChannelReport) -> Self {
        Self::connect_with_transport_and_channel_report(socket, name, "cli", channel)
    }

    fn connect_with_transport_and_channel_report(
        socket: &Path,
        name: &str,
        transport: &str,
        channel: ChannelReport,
    ) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        let read_stream = stream.try_clone().unwrap();
        let mut agent = Self {
            writer: BufWriter::new(stream),
            reader: BufReader::new(read_stream),
        };
        agent.send(&WrapperToDaemon::Register {
            agent_type: "ui-test".to_string(),
            identity_version: 2,
            agent_id: name.to_string(),
            host: Some("test".to_string()),
            transport: Some(transport.to_string()),
            channel,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".to_string()),
            instance_id: Some(format!("ui-test-{name}")),
            domain: Some("test".to_string()),
            turn_in_progress: false,
            journal_available: Some(false),
        });
        assert!(matches!(agent.read(), DaemonToWrapper::Registered { .. }));
        agent.send(&WrapperToDaemon::JournalReady);
        agent
    }

    fn unregister_with_barrier(&mut self) {
        self.send(&WrapperToDaemon::Unregister);
        self.send(&WrapperToDaemon::ListAgents);
        assert!(matches!(self.read(), DaemonToWrapper::AgentList { .. }));
    }

    fn listed_channel(&mut self, name: &str) -> Option<String> {
        self.send(&WrapperToDaemon::ListAgents);
        let agents = match self.read() {
            DaemonToWrapper::AgentList { agents } => agents,
            response => panic!("AgentList attendu, reçu {response:?}"),
        };
        agents
            .iter()
            .find(|agent| agent.agent_id == name)
            .unwrap_or_else(|| panic!("présence {name} absente"))
            .channel
            .clone()
    }

    fn send(&mut self, message: &WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn read(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }

    /// Un agent peut recevoir une notification Bridget déjà en attente au
    /// moment où le relais ouvre son abonnement Attach. Le client doit donc
    /// continuer à lire jusqu'à la trame d'abonnement, au lieu de confondre
    /// une livraison normale avec une erreur de protocole.
    fn read_subscription_for(&mut self, expected_agent: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        self.reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let subscription_id = loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => panic!("connexion agent fermée avant Subscribe"),
                Ok(_) => match decode(line.trim()).unwrap() {
                    DaemonToWrapper::Subscribe {
                        subscription_id,
                        agent,
                        ..
                    } if agent == expected_agent => break subscription_id,
                    DaemonToWrapper::Deliver(message) if message.from == "bridget" => {
                        assert!(
                            message.body.starts_with("Rappel :"),
                            "notification Bridget inattendue avant Subscribe: {message:?}"
                        );
                    }
                    response => {
                        panic!("Subscribe pour {expected_agent} attendu, reçu {response:?}")
                    }
                },
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    assert!(
                        Instant::now() < deadline,
                        "Subscribe pour {expected_agent} non reçu avant l'échéance"
                    );
                }
                Err(error) => panic!("lecture agent avant Subscribe: {error}"),
            }
        };
        self.reader.get_ref().set_read_timeout(None).unwrap();
        subscription_id
    }

    fn ledger_messages(&mut self) -> Vec<bridget_transport::protocol::LedgerMessage> {
        self.send(&WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Messages,
            limit: 200,
        });
        match self.read() {
            DaemonToWrapper::LedgerProjection { messages, .. } => messages,
            response => panic!("LedgerProjection attendu, reçu {response:?}"),
        }
    }

    fn open_requests(&mut self) -> Vec<bridget_transport::protocol::RequestInfo> {
        self.send(&WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Requests,
            limit: 200,
        });
        match self.read() {
            DaemonToWrapper::LedgerProjection { requests, .. } => requests,
            response => panic!("LedgerProjection attendu, reçu {response:?}"),
        }
    }
}

fn root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bui-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&root).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    root
}

fn write_maicie_config(root: &Path) -> PathBuf {
    let path = root.join("maicie.json");
    let socket = root.join(".cache/bridget/bridget.sock");
    let database = root.join("maicie.sqlite");
    let json = serde_json::json!({
        "version": 1,
        "bridget_socket": socket,
        "database_path": database,
        "durations": {"short_secs": 1, "normal_secs": 2, "long_secs": 3},
        "profiles": []
    });
    std::fs::write(&path, serde_json::to_vec(&json).unwrap()).unwrap();
    path
}

fn write_agent_journal(root: &Path, agent: &str) {
    let journal_dir = root.join(".cache/bridget/sessions").join(agent);
    std::fs::create_dir_all(&journal_dir).unwrap();
    std::fs::write(
        journal_dir.join("fixture.jsonl"),
        "{\"v\":1,\"seq\":1,\"message_id\":\"turn-fixture\",\"event\":\"turn_start\",\"session_id\":\"fixture\"}\n",
    )
    .unwrap();
}

fn observe_ui_presence_channel(
    label: &str,
    environment_channel: Option<&str>,
    federation_config: Option<&str>,
) -> Option<String> {
    observe_ui_presence_channel_after(label, environment_channel, federation_config, None)
}

fn observe_ui_presence_channel_after(
    label: &str,
    environment_channel: Option<&str>,
    federation_config: Option<&str>,
    previous_channel: Option<&str>,
) -> Option<String> {
    let root = root(label);
    if let Some(config) = federation_config {
        let config_directory = root.join(".config/bridget");
        std::fs::create_dir_all(&config_directory).unwrap();
        std::fs::write(config_directory.join("federation.env"), config).unwrap();
    }
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    if let Some(previous_channel) = previous_channel {
        let mut previous = LiveAgent::connect_with_channel_report(
            &socket,
            "humain",
            ChannelReport::Known(previous_channel.to_string()),
        );
        previous.unregister_with_barrier();
    }
    let mut recipient = LiveAgent::connect(&socket, "destinataire-canal-ui");
    let ui = UiProcess::start(&root, environment_channel);
    let response = read_response(request_http(
        ui.address,
        "POST",
        &format!("/v1/send?token={}", ui.token),
        Some(r#"{"version":1,"to":"destinataire-canal-ui","body":"preuve canal","reply":true}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 202"), "{response}");
    assert!(matches!(
        recipient.read(),
        DaemonToWrapper::DeliverIdempotent { .. }
    ));
    recipient.send(&WrapperToDaemon::ListAgents);
    let agents = match recipient.read() {
        DaemonToWrapper::AgentList { agents } => agents,
        response => panic!("AgentList attendu, reçu {response:?}"),
    };
    let human = agents
        .iter()
        .find(|agent| agent.agent_id == "humain")
        .expect("présence humaine UI enregistrée");
    assert_eq!(human.transport, "cli");
    let channel = human.channel.clone();
    drop(ui);
    drop(recipient);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
    channel
}

fn observe_reconnection_channel(label: &str, report: ChannelReport) -> Option<String> {
    let root = root(label);
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut initial = LiveAgent::connect_with_channel_report(
        &socket,
        "humain-transition",
        ChannelReport::Known("unix".to_string()),
    );
    initial.unregister_with_barrier();
    drop(initial);

    let mut reconnected =
        LiveAgent::connect_with_channel_report(&socket, "humain-transition", report);
    let channel = reconnected.listed_channel("humain-transition");
    drop(reconnected);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
    channel
}

fn observe_fresh_registration_channel(
    label: &str,
    transport: &str,
    report: ChannelReport,
) -> Option<String> {
    let root = root(label);
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let name = format!("humain-{label}");
    let mut agent =
        LiveAgent::connect_with_transport_and_channel_report(&socket, &name, transport, report);
    let channel = agent.listed_channel(&name);
    drop(agent);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
    channel
}

fn request(address: SocketAddr, path: &str) -> TcpStream {
    request_http(address, "GET", path, None)
}

fn request_http(address: SocketAddr, method: &str, path: &str, body: Option<&str>) -> TcpStream {
    let mut stream = TcpStream::connect(address).unwrap();
    let body = body.unwrap_or("");
    stream
        .write_all(
            format!(
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .unwrap();
    stream
}

fn read_response(mut stream: TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn response_json(response: &str) -> serde_json::Value {
    serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap()
}

fn managed_agent_info(name: &str) -> AgentInfo {
    serde_json::from_value(serde_json::json!({
        "name": name,
        "agent_type": "ui-test",
        "connection_id": "managed-connection",
        "host": "test",
        "transport": "cli",
        "state": "idle",
        "last_seen_secs": 0,
        "reconnect_count": 0,
        "persistent": false
    }))
    .unwrap()
}

fn spawn_stop_daemon(
    socket: &Path,
    expected_name: &'static str,
    expected_command_id: &'static str,
    stop_response: DaemonToWrapper,
) -> thread::JoinHandle<()> {
    let listener = UnixListener::bind(socket).unwrap();
    thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim()).unwrap(),
            WrapperToDaemon::Register { .. }
        ));
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                agent_id: "humain".to_string()
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim()).unwrap(),
            WrapperToDaemon::JournalReady
        ));

        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim()).unwrap(),
            WrapperToDaemon::ListAgents
        ));
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::AgentList {
                agents: vec![managed_agent_info(expected_name)]
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(line.trim()).unwrap(),
            WrapperToDaemon::StopOrder { agent_id: ref name, ref command_id }
                if name == expected_name && command_id == expected_command_id
        ));
        writeln!(writer, "{}", encode(&stop_response).unwrap()).unwrap();
        writer.flush().unwrap();
    })
}

fn read_until(stream: &mut TcpStream, expected: &str) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut bytes = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let mut one = [0_u8; 1];
        match stream.read(&mut one) {
            Ok(0) => break,
            Ok(_) => {
                bytes.push(one[0]);
                let text = String::from_utf8_lossy(&bytes);
                if text.contains(expected) {
                    return text.into_owned();
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                continue;
            }
            Err(error) => panic!("lecture HTTP: {error}"),
        }
    }
    String::from_utf8(bytes).unwrap()
}

#[test]
fn spec_024_ui_locale_attestee_projette_unix_dans_agent_info() {
    assert_eq!(
        observe_ui_presence_channel("canal-local", Some("unix"), None).as_deref(),
        Some("unix")
    );
}

#[test]
fn spec_024_reconnexion_inconnue_explicite_efface_le_canal_precedent() {
    assert_eq!(
        observe_reconnection_channel("transition-inconnue", ChannelReport::Unknown),
        None
    );
}

#[test]
fn spec_024_reconnexion_historique_omise_conserve_le_canal_precedent() {
    assert_eq!(
        observe_reconnection_channel("transition-omise", ChannelReport::Omitted).as_deref(),
        Some("unix")
    );
}

#[test]
fn spec_024_inconnu_explicite_interdit_repli_transport_historique() {
    for transport in ["unix", "ssh-unix"] {
        assert_eq!(
            observe_fresh_registration_channel(
                &format!("inconnu-sans-repli-{transport}"),
                transport,
                ChannelReport::Unknown,
            ),
            None,
            "le transport historique {transport} ne doit pas devenir un canal moderne",
        );
    }
}

#[test]
fn spec_024_omission_historique_conserve_repli_transport() {
    for transport in ["unix", "ssh-unix"] {
        assert_eq!(
            observe_fresh_registration_channel(
                &format!("omission-avec-repli-{transport}"),
                transport,
                ChannelReport::Omitted,
            )
            .as_deref(),
            Some(transport),
            "l’omission historique doit conserver le canal {transport}",
        );
    }
}

#[test]
fn spec_024_ui_federee_projette_ssh_unix_dans_agent_info() {
    assert_eq!(
        observe_ui_presence_channel(
            "canal-federe",
            None,
            Some("channel=ssh-unix\ntransport=ssh-unix\n")
        )
        .as_deref(),
        Some("ssh-unix")
    );
}

#[test]
fn spec_024_ui_sans_attestation_reste_inconnue_dans_agent_info() {
    assert_eq!(
        observe_ui_presence_channel("canal-inconnu", None, None),
        None
    );
}

#[test]
fn spec_024_ui_aux_attestations_divergentes_reste_inconnue_dans_agent_info() {
    assert_eq!(
        observe_ui_presence_channel_after(
            "canal-divergent",
            Some("unix"),
            Some("channel=ssh-unix\ntransport=ssh-unix\n"),
            Some("unix")
        ),
        None
    );
}

#[test]
fn loopback_rend_snapshot_et_relaie_un_fragment_attach_d_un_agent_vivant() {
    let root = root("couteau");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut agent = LiveAgent::connect(&socket, "agent-vivant");
    let mut demandeur = LiveAgent::connect(&socket, "demandeur");
    let mut demande = BridgetMessage::new("demandeur", "agent-vivant", "demande ouverte");
    demande.reply = true;
    let demande_id = demande.id.clone();
    demandeur.send(&WrapperToDaemon::Send(demande));
    assert!(matches!(demandeur.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(agent.read(), DaemonToWrapper::Deliver(_)));
    write_agent_journal(&root, "agent-vivant");
    let ui = UiProcess::start(&root, None);
    let address = ui.address;

    let mut snapshot = request(address, &format!("/v1/snapshot?token={}", ui.token));
    let mut snapshot_text = read_until(&mut snapshot, "agent-vivant");
    let mut snapshot_tail = String::new();
    snapshot
        .read_to_string(&mut snapshot_tail)
        .expect("fin de réponse snapshot");
    snapshot_text.push_str(&snapshot_tail);
    assert!(snapshot_text.starts_with("HTTP/1.1 200"), "{snapshot_text}");
    assert!(
        snapshot_text.contains("\"missions\":{\"version\":1"),
        "{snapshot_text}"
    );
    assert!(
        snapshot_text.contains(&demande_id),
        "la projection globale doit inclure la demande suivie réelle; {snapshot_text}"
    );

    let mut events = request(
        address,
        &format!("/v1/watch?token={}&agent=agent-vivant", ui.token),
    );
    let subscription_id = agent.read_subscription_for("agent-vivant");
    agent.send(&WrapperToDaemon::Subscribed {
        subscription_id: subscription_id.clone(),
    });
    agent.send(&WrapperToDaemon::JournalFragment {
        subscription_id: subscription_id.clone(),
        seq: 1,
        offset: 0,
        final_fragment: true,
        bytes: b"fragment reel".to_vec(),
    });
    agent.send(&WrapperToDaemon::SnapshotCaughtUp {
        subscription_id,
        through_seq: Some(1),
    });
    let events = read_until(&mut events, "ZnJhZ21lbnQgcmVlbA");
    assert!(events.starts_with("HTTP/1.1 200"), "{events}");
    assert!(events.contains("event: snapshot"), "{events}");
    assert!(events.contains("event: journal"), "{events}");
    assert!(events.contains("JournalFragment"), "{events}");
    assert!(events.contains("ZnJhZ21lbnQgcmVlbA"), "{events}");
    let snapshot_position = events.find("event: snapshot").unwrap();
    let fragment_position = events.find("JournalFragment").unwrap();
    assert!(
        snapshot_position < fragment_position,
        "Mutation : publier l'instantané avant Subscribe ferait perdre le fragment entre les deux; {events}"
    );

    drop(ui);
    drop(agent);
    drop(demandeur);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn requete_loopback_sans_jeton_est_refusee() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/ui-inaccessible.sock"),
        maicie_config: PathBuf::from("/tmp/maicie-inaccessible.json"),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "secret".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());
    let mut response = request(address, "/v1/snapshot");
    let response = read_until(&mut response, "403");
    assert!(response.starts_with("HTTP/1.1 403"), "{response}");
}

#[test]
fn get_sur_v1_send_reste_interdit_apres_ouverture_du_post() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/ui-get-send-ne-doit-pas-ouvrir.sock"),
        maicie_config: PathBuf::from("/tmp/ui-get-send-ne-doit-pas-ouvrir.json"),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-get-send".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request(address, "/v1/send?token=jeton-get-send"));
    assert!(response.starts_with("HTTP/1.1 405"), "{response}");
}

#[test]
fn spec_073_route_stop_verrouille_methode_jeton_et_version() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/spec-073-guards-no-daemon.sock"),
        maicie_config: PathBuf::from("/tmp/spec-073-guards-no-maicie.json"),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-stop-guards".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let method = read_response(request(address, "/v1/agents/stop?token=jeton-stop-guards"));
    assert!(method.starts_with("HTTP/1.1 405"), "{method}");

    let token = read_response(request_http(
        address,
        "POST",
        "/v1/agents/stop",
        Some(r#"{"version":1,"name":"managed","command_id":"stop-ui-guards"}"#),
    ));
    assert!(token.starts_with("HTTP/1.1 403"), "{token}");

    let version = read_response(request_http(
        address,
        "POST",
        "/v1/agents/stop?token=jeton-stop-guards",
        Some(r#"{"version":2,"name":"managed","command_id":"stop-ui-guards"}"#),
    ));
    assert!(version.starts_with("HTTP/1.1 400"), "{version}");
    assert_eq!(response_json(&version)["code"], "invalid_request");
}

#[test]
fn spec_073_route_stop_relaie_le_verdict_correle() {
    let root = root("stop-ok");
    let socket = root.join("stop.sock");
    let server = spawn_stop_daemon(
        &socket,
        "managed",
        "stop-ui-http-ok",
        DaemonToWrapper::StopResult {
            command_id: "stop-ui-http-ok".to_string(),
            outcome: StopOutcome::Stopped,
        },
    );
    let config = UiRelayConfig {
        daemon_socket: socket.clone(),
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-stop-ok".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/agents/stop?token=jeton-stop-ok",
        Some(r#"{"version":1,"name":"managed","command_id":"stop-ui-http-ok"}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let payload = response_json(&response);
    assert_eq!(payload["version"], 1);
    assert_eq!(payload["name"], "managed");
    assert_eq!(payload["command_id"], "stop-ui-http-ok");
    assert_eq!(payload["outcome"], "stopped");

    server.join().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec_073_route_stop_ferme_l_erreur_de_protocole() {
    let root = root("stop-protocol");
    let socket = root.join("stop.sock");
    let server = spawn_stop_daemon(
        &socket,
        "managed",
        "stop-ui-http-protocol",
        DaemonToWrapper::AgentList { agents: Vec::new() },
    );
    let config = UiRelayConfig {
        daemon_socket: socket.clone(),
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-stop-protocol".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/agents/stop?token=jeton-stop-protocol",
        Some(r#"{"version":1,"name":"managed","command_id":"stop-ui-http-protocol"}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 503"), "{response}");
    let payload = response_json(&response);
    assert_eq!(payload["code"], "daemon_unavailable");
    assert_eq!(payload["message"], "Le daemon Bridget est indisponible.");

    server.join().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_v1_send_corps_vide_rend_le_code_ferme_invalid_body() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/ui-invalid-body-ne-doit-pas-ouvrir.sock"),
        maicie_config: PathBuf::from("/tmp/ui-invalid-body-ne-doit-pas-ouvrir.json"),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-invalid-body".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/send?token=jeton-invalid-body",
        Some(r#"{"version":1,"to":"agent","body":"  ","reply":false}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 400"), "{response}");
    assert_eq!(response_json(&response)["code"], "invalid_body");
}

#[test]
fn post_v1_send_valide_repond_202_et_livre_un_identifiant_non_vide() {
    let root = root("send-ok");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut recipient = LiveAgent::connect(&socket, "destinataire");
    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-send-ok".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/send?token=jeton-send-ok",
        Some(r#"{"version":1,"to":"destinataire","body":"message UI","reply":false}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 202"), "{response}");
    let payload = response_json(&response);
    assert_eq!(payload["version"], 1);
    assert_eq!(payload["status"], "in_flight");
    assert!(
        payload["delivery_id"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "{payload}"
    );
    assert!(payload["issued_at"].as_i64().is_some_and(|value| value > 0));
    match recipient.read() {
        DaemonToWrapper::DeliverIdempotent { message, .. } => {
            assert_eq!(message.from, "humain");
            assert_eq!(message.to, "destinataire");
            assert_eq!(message.body, "message UI");
            assert!(!message.reply);
        }
        response => panic!("DeliverIdempotent attendu, reçu {response:?}"),
    }

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_v1_send_reply_true_cree_une_demande_suivie() {
    let root = root("send-reply");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut recipient = LiveAgent::connect(&socket, "destinataire-reply");
    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-send-reply".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/send?token=jeton-send-reply",
        Some(r#"{"version":1,"to":"destinataire-reply","body":"question suivie","reply":true}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 202"), "{response}");
    let message_id = match recipient.read() {
        DaemonToWrapper::DeliverIdempotent { message, .. } => {
            assert!(message.reply);
            message.id
        }
        response => panic!("DeliverIdempotent attendu, reçu {response:?}"),
    };
    recipient.send(&WrapperToDaemon::ListAgents);
    let agents = match recipient.read() {
        DaemonToWrapper::AgentList { agents } => agents,
        response => panic!("AgentList attendu, reçu {response:?}"),
    };
    let human = agents
        .iter()
        .find(|agent| agent.agent_id == "humain")
        .expect("présence humaine UI enregistrée");
    assert_eq!(
        human.channel, None,
        "UiRelay::bind sans attestation ne doit pas réinventer unix via transport"
    );
    let requests = recipient.open_requests();
    assert!(
        requests.iter().any(|request| request.id == message_id),
        "reply=true doit créer la demande suivie {message_id}: {requests:?}"
    );
    let mut reply = BridgetMessage::new("destinataire-reply", "humain", "réponse UI");
    reply.in_reply_to = Some(message_id.clone());
    recipient.send(&WrapperToDaemon::Send(reply));
    assert!(matches!(recipient.read(), DaemonToWrapper::Ack { .. }));
    assert!(
        recipient
            .open_requests()
            .iter()
            .all(|request| request.id != message_id),
        "la réponse corrélée doit solder la demande suivie"
    );

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_v1_send_destinataire_inconnu_refuse_sans_archiver() {
    let root = root("send-unknown");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut observer = LiveAgent::connect(&socket, "observateur");
    assert!(observer.ledger_messages().is_empty());
    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-send-unknown".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request_http(
        address,
        "POST",
        "/v1/send?token=jeton-send-unknown",
        Some(r#"{"version":1,"to":"absent","body":"ne pas archiver","reply":false}"#),
    ));
    assert!(response.starts_with("HTTP/1.1 404"), "{response}");
    assert_eq!(response_json(&response)["code"], "unknown_recipient");
    assert!(
        observer.ledger_messages().is_empty(),
        "un refus de destinataire ne doit créer aucune ligne durable"
    );

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_compose_la_ligne_agent_avec_les_faits_du_ledger() {
    let root = root("agent-row");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut agent = LiveAgent::connect(&socket, "agent-ligne");
    let mut human = LiveAgent::connect(&socket, "humain");
    agent.send(&WrapperToDaemon::Send(BridgetMessage::new(
        "agent-ligne",
        "humain",
        "extrait attesté",
    )));
    assert!(matches!(agent.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(human.read(), DaemonToWrapper::Deliver(_)));
    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-agent-row".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request(address, "/v1/snapshot?token=jeton-agent-row"));
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    let payload = response_json(&response);
    let row = payload["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "agent-ligne")
        .expect("ligne de l'agent présent");
    assert_eq!(row["type"], "ui-test");
    assert_eq!(row["host"], "test");
    assert!(matches!(
        row["state"].as_str(),
        Some("alive" | "busy" | "stopped")
    ));
    assert!(row["last_message_at"].as_i64().is_some());
    assert_eq!(row["last_excerpt"], "extrait attesté");
    assert_eq!(row["unread"], 1);

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn relais_sert_les_trois_assets_hors_du_source_rust() {
    let config = UiRelayConfig {
        daemon_socket: PathBuf::from("/tmp/ui-assets-ne-doit-pas-ouvrir.sock"),
        maicie_config: PathBuf::from("/tmp/ui-assets-ne-doit-pas-ouvrir.json"),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-assets".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let index = read_response(request(address, "/?token=jeton-assets"));
    let script = read_response(request(address, "/app.js"));
    let theme = read_response(request(address, "/theme.css"));
    assert!(index.starts_with("HTTP/1.1 200"), "{index}");
    assert!(index.contains("<script type=\"module\" src=\"/app.js\">"));
    assert!(script.starts_with("HTTP/1.1 200"), "{script}");
    assert!(script.contains("application/javascript"), "{script}");
    assert!(theme.starts_with("HTTP/1.1 200"), "{theme}");
    assert!(theme.contains("text/css"), "{theme}");
}

#[test]
fn watch_annonce_reconnecting_puis_connected_apres_coupure_daemon() {
    let root = root("relay-state");
    let mut daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut agent = LiveAgent::connect(&socket, "agent-reprise");
    write_agent_journal(&root, "agent-reprise");
    let ui = UiProcess::start(&root, None);
    let address = ui.address;

    let mut events = request(
        address,
        &format!("/v1/watch?token={}&agent=agent-reprise", ui.token),
    );
    let subscription_id = agent.read_subscription_for("agent-reprise");
    agent.send(&WrapperToDaemon::Subscribed { subscription_id });
    let initial = read_until(&mut events, "\"state\":\"connected\"");
    assert!(initial.contains("event: relay_state"), "{initial}");

    daemon.terminate();
    let cut = read_until(&mut events, "\"state\":\"reconnecting\"");
    assert!(cut.contains("event: relay_state"), "{cut}");

    let daemon_restarted = DaemonProcess::start(&root);
    let mut reconnected = LiveAgent::connect(&socket, "agent-reprise");
    let subscription_id = reconnected.read_subscription_for("agent-reprise");
    reconnected.send(&WrapperToDaemon::Subscribed { subscription_id });
    let restored = read_until(&mut events, "\"state\":\"connected\"");
    assert!(restored.contains("event: relay_state"), "{restored}");

    drop(ui);
    drop(reconnected);
    drop(agent);
    drop(daemon);
    drop(daemon_restarted);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn snapshot_sans_agent_omet_les_pairs_et_watch_agent_les_projette() {
    let root = root("peer-exchange");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut focus = LiveAgent::connect(&socket, "agent-focus");
    let mut peer = LiveAgent::connect(&socket, "agent-pair");

    let mut outgoing = BridgetMessage::new("agent-focus", "agent-pair", "question");
    outgoing.id = "sortant-pair".to_string();
    focus.send(&WrapperToDaemon::Send(outgoing));
    assert!(matches!(focus.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(peer.read(), DaemonToWrapper::Deliver(_)));
    let mut incoming = BridgetMessage::new("agent-pair", "agent-focus", "réponse");
    incoming.id = "entrant-pair".to_string();
    peer.send(&WrapperToDaemon::Send(incoming));
    assert!(matches!(peer.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(focus.read(), DaemonToWrapper::Deliver(_)));

    write_agent_journal(&root, "agent-focus");
    let ui = UiProcess::start(&root, None);
    let address = ui.address;

    let global = response_json(&read_response(request(
        address,
        &format!("/v1/snapshot?token={}", ui.token),
    )));
    assert!(
        global.get("peer_exchanges").is_none(),
        "sans agent, une absence de calcul ne doit pas mentir sous la forme d'une liste vide: {global}"
    );

    let focused = response_json(&read_response(request(
        address,
        &format!("/v1/snapshot?token={}&agent=agent-focus", ui.token),
    )));
    let focused_exchanges = focused["peer_exchanges"]
        .as_array()
        .expect("avec agent, la projection calculée reste toujours présente");
    assert_eq!(focused_exchanges.len(), 1, "{focused}");
    assert_eq!(focused_exchanges[0]["peer"], "agent-pair", "{focused}");
    assert_eq!(focused_exchanges[0]["direction"], "both", "{focused}");
    assert_eq!(focused_exchanges[0]["count"], 2, "{focused}");

    let mut events = request(
        address,
        &format!("/v1/watch?token={}&agent=agent-focus", ui.token),
    );
    let subscription_id = focus.read_subscription_for("agent-focus");
    focus.send(&WrapperToDaemon::Subscribed { subscription_id });

    let snapshot = read_until(&mut events, "event: peer_exchange");
    assert!(snapshot.contains("event: snapshot"), "{snapshot}");
    assert!(snapshot.contains("\"peer\":\"agent-pair\""), "{snapshot}");
    assert!(snapshot.contains("\"direction\":\"both\""), "{snapshot}");
    assert!(snapshot.contains("\"count\":2"), "{snapshot}");
    let pushed = read_until(&mut events, "\"delivery_ids\"");
    assert!(pushed.contains("data: {\"version\":1,\"kind\":\"peer_exchange\""));

    drop(ui);
    drop(focus);
    drop(peer);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn watch_pousse_thread_message_sortant_apres_ouverture() {
    // Témoin du chemin vivant : après l'ouverture du watch, un sortant
    // référent→humain écrit au ledger doit arriver en SSE thread_message
    // SANS reconnexion ni second snapshot. Meurt si push_live_thread_messages
    // est retiré ou vidé.
    // Nom court : sous macOS le chemin de la socket Unix est sévèrement borné ;
    // « thread-live » faisait « daemon non prêt » sans rapport avec le code testé.
    let root = root("tlive");
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut referent = LiveAgent::connect(&socket, "agent-referent");
    // Présence humaine : le Send sortant vers humain doit être accepté au ledger.
    let _humain = LiveAgent::connect(&socket, "humain");

    write_agent_journal(&root, "agent-referent");
    let ui = UiProcess::start(&root, None);
    let address = ui.address;

    let mut events = request(
        address,
        &format!("/v1/watch?token={}&agent=agent-referent", ui.token),
    );
    let subscription_id = referent.read_subscription_for("agent-referent");
    referent.send(&WrapperToDaemon::Subscribed { subscription_id });

    let opened = read_until(&mut events, "event: snapshot");
    assert!(opened.contains("event: snapshot"), "{opened}");
    assert!(
        !opened.contains("live-sortant-apres-ouverture"),
        "le corps live ne doit pas être dans le snapshot initial: {opened}"
    );

    let mut outbound = BridgetMessage::new(
        "agent-referent",
        "humain",
        "réponse vivante live-sortant-apres-ouverture",
    );
    outbound.id = "live-sortant-apres-ouverture".to_string();
    referent.send(&WrapperToDaemon::Send(outbound));
    assert!(matches!(referent.read(), DaemonToWrapper::Ack { .. }));

    let live = read_until(&mut events, "live-sortant-apres-ouverture");
    assert!(
        live.contains("event: thread_message"),
        "le chemin vivant doit pousser thread_message, reçu: {live}"
    );
    assert!(
        live.contains("\"role\":\"agent\""),
        "le sortant doit porter le rôle agent: {live}"
    );
    assert!(
        live.contains("live-sortant-apres-ouverture"),
        "le corps sortant doit apparaître sans recharger: {live}"
    );
    assert!(
        !live.contains("\"state\":\"reconnecting\""),
        "une reconnexion ne compte pas comme chemin vivant: {live}"
    );

    drop(ui);
    drop(referent);
    drop(_humain);
    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn relais_ui_expose_separement_connexion_vitalite_tour_attente_et_file() {
    let root = root("execution-projection");
    let config_directory = root.join(".config/bridget");
    std::fs::create_dir_all(&config_directory).unwrap();
    let registry_path = config_directory.join("agents.json");
    std::fs::write(
        &registry_path,
        r#"{"execution_projection":{"dual_write":true,"legacy_projection":true}}"#,
    )
    .unwrap();
    #[cfg(unix)]
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let daemon = DaemonProcess::start(&root);
    let socket = root.join(".cache/bridget/bridget.sock");
    let mut sender = LiveAgent::connect(&socket, "maicie");
    let mut agent = LiveAgent::connect(&socket, "agent-execution");

    let mut queued = BridgetMessage::new("maicie", "agent-execution", "à conserver");
    queued.id = "queue-execution".to_string();
    queued.intent = Some(bridget_core::MessageIntent::QueueOnly);
    sender.send(&WrapperToDaemon::Send(queued));
    assert!(matches!(sender.read(), DaemonToWrapper::Ack { .. }));

    let mut started = BridgetMessage::new("maicie", "agent-execution", "à démarrer");
    started.id = "turn-execution".to_string();
    started.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    sender.send(&WrapperToDaemon::Send(started));
    assert!(matches!(sender.read(), DaemonToWrapper::Ack { .. }));
    assert!(matches!(
        agent.read(),
        DaemonToWrapper::DeliverExecution {
            ref execution_id,
            generation: 1,
            revision: 0,
            ..
        } if execution_id == "execution-turn-execution"
    ));
    let observed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap();
    agent.send(&WrapperToDaemon::ExecutionStateChanged {
        transition: bridget_transport::protocol::ExecutionStateTransition {
            execution_id: "execution-turn-execution".to_string(),
            generation: 1,
            expected_state: "starting".to_string(),
            expected_revision: 0,
            next_state: "running".to_string(),
            reason: "provider_accepted".to_string(),
            observed_at,
        },
    });

    let config = UiRelayConfig {
        daemon_socket: socket,
        maicie_config: write_maicie_config(&root),
        project_root_policy_path: None,
        bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        token: "jeton-execution".to_string(),
    };
    let relay = UiRelay::bind(config).unwrap();
    let address = relay.local_addr().unwrap();
    thread::spawn(move || relay.serve().unwrap());

    let response = read_response(request(address, "/v1/snapshot?token=jeton-execution"));
    let payload = response_json(&response);
    let row = payload["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "agent-execution")
        .expect("projection de l'agent");
    assert_eq!(row["connection_state"], "busy");
    assert!(row["provider_age_secs"].as_u64().is_some());
    assert_eq!(row["turn_state"], "running");
    assert!(row.get("wait_state").is_none());
    assert!(row["progress_age_secs"].as_u64().is_some(), "row={row}");
    assert_eq!(row["queue_depth"], 1);

    agent.send(&WrapperToDaemon::ExecutionStateChanged {
        transition: bridget_transport::protocol::ExecutionStateTransition {
            execution_id: "execution-turn-execution".to_string(),
            generation: 1,
            expected_state: "running".to_string(),
            expected_revision: 1,
            next_state: "waiting_approval".to_string(),
            reason: "permission_required".to_string(),
            observed_at,
        },
    });
    let response = read_response(request(address, "/v1/snapshot?token=jeton-execution"));
    let payload = response_json(&response);
    let row = payload["agents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "agent-execution")
        .expect("projection après attente");
    assert_eq!(row["connection_state"], "alive");
    assert_eq!(row["turn_state"], "waiting_approval");
    assert_eq!(row["wait_state"], "waiting_approval");
    assert_eq!(row["queue_depth"], 1);

    drop(daemon);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec_074_cli_endpoint_lit_l_etat_sans_demarrer_de_relais_ni_divulguer_en_erreur() {
    let root = root("endpoint-cli");
    let endpoint_path = root.join(".cache/bridget/ui-endpoint.json");
    std::fs::create_dir_all(endpoint_path.parent().unwrap()).unwrap();
    let fixture_token = "fixture-token-074";
    std::fs::write(
        &endpoint_path,
        serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "port": 17888,
            "token": fixture_token,
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&endpoint_path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["ui", "endpoint", "--json"])
        .env_clear()
        .env("HOME", &root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("commande endpoint exécutée");
    assert!(output.status.success(), "stderr={:?}", output.stderr);
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(payload["version"], 1);
    assert_eq!(payload["port"], 17888);
    assert_eq!(payload["token"], fixture_token);
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "la lecture nominale ne produit pas de diagnostic"
    );

    std::fs::remove_file(&endpoint_path).unwrap();
    let missing = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["ui", "endpoint", "--json"])
        .env_clear()
        .env("HOME", &root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("commande endpoint absente exécutée");
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty(), "aucun contrat partiel en erreur");
    assert!(
        !String::from_utf8_lossy(&missing.stderr).contains(fixture_token),
        "un jeton d'état ne doit jamais fuiter dans une erreur"
    );

    std::fs::write(
        &endpoint_path,
        serde_json::to_vec(&serde_json::json!({
            "version": 99,
            "port": 17888,
            "token": fixture_token,
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&endpoint_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["ui", "endpoint", "--json"])
        .env_clear()
        .env("HOME", &root)
        .env("PATH", "/usr/bin:/bin")
        .output()
        .expect("commande endpoint invalide exécutée");
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty(), "aucun contrat partiel en erreur");
    assert!(
        !String::from_utf8_lossy(&invalid.stderr).contains(fixture_token),
        "un jeton d'état invalide ne doit jamais fuiter dans une erreur"
    );

    std::fs::remove_dir_all(root).unwrap();
}
