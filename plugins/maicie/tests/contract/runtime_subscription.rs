use base64::Engine;
use maicie::bridget_client::{AttachWindow, BridgetClient};
use maicie::domain::EtatFlux;
use maicie::runtime::{
    MAX_REASSEMBLED_EVENT_BYTES, RuntimeNature, RuntimeSignal, RuntimeSubscription,
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

#[test]
fn consomme_fragments_gap_et_fin_uniquement_via_subscribe_public() {
    let fixture = SocketFixture::new("fragments");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        accept_client(&listener);
        let (stream, _) = listener.accept().expect("connexion Attach attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"RoleHandshake","role":"attach"})
        );
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
        assert_eq!(
            read_json(&mut reader),
            json!({"type":"Subscribe","agent":"prospective","window":{"kind":"Today"}})
        );

        write_json(
            &mut writer,
            json!({"type":"Subscribed","subscription_id":"sub-7"}),
        );
        let line = serde_json::to_vec(&json!({
            "v": 1,
            "seq": 7,
            "ts": "2026-08-23T12:00:00Z",
            "session_id": "acp-session-7",
            "event": "turn_start",
            "message_id": "message-7",
            "payload": {"from":"maicie","reply":false,"body":"secret jamais recopié"}
        }))
        .unwrap();
        let split_at = 23;
        write_fragment(&mut writer, "sub-7", 7, 0, false, &line[..split_at]);
        write_fragment(
            &mut writer,
            "sub-7",
            7,
            split_at as u64,
            true,
            &line[split_at..],
        );
        write_json(
            &mut writer,
            json!({"type":"SnapshotCaughtUp","subscription_id":"sub-7","through_seq":7}),
        );
        write_json(
            &mut writer,
            json!({"type":"Gap","subscription_id":"sub-7","from_seq":8,"to_seq":9,"reason":"vue_lente"}),
        );
        let after_gap = serde_json::to_vec(&json!({
            "v": 1,
            "seq": 10,
            "ts": "2026-08-23T12:00:01Z",
            "session_id": "acp-session-7",
            "event": "turn_end",
            "payload": {"stop_reason":"end_turn"}
        }))
        .unwrap();
        write_fragment(&mut writer, "sub-7", 10, 0, true, &after_gap);
        write_json(
            &mut writer,
            json!({"type":"End","subscription_id":"sub-7","reason":"wrapper_parti"}),
        );
    });

    let client = BridgetClient::connect(fixture.path(), "maicie-instance-runtime").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut runtime = RuntimeSubscription::open_until(
        &client,
        "prospective",
        AttachWindow::Today,
        deadline,
    )
    .unwrap();
    assert_eq!(runtime.subscription_id(), "sub-7");
    assert_eq!(
        runtime.stream_state(),
        EtatFlux::Unavailable,
        "Subscribed seul ne rend pas la vue fraîche"
    );

    let RuntimeSignal::Observation(observation) = runtime.next_signal_until(deadline).unwrap() else {
        panic!("le premier signal doit être une observation factuelle");
    };
    assert_eq!(observation.agent, "prospective");
    assert_eq!(observation.seq, 7);
    assert_eq!(observation.proof_ref, "sub-7:7");
    assert_eq!(observation.nature, RuntimeNature::Tour);
    assert_eq!(observation.observed_at, "2026-08-23T12:00:00Z");
    assert_eq!(
        observation.details,
        json!({"event":"turn_start","from":"maicie","reply":false})
    );
    assert_eq!(
        observation.stream_state,
        EtatFlux::Unavailable,
        "une observation antérieure à SnapshotCaughtUp reste incomplète"
    );
    assert!(
        !observation.details.to_string().contains("secret"),
        "le runtime ne recopie jamais le corps du journal"
    );

    assert!(matches!(
        runtime.next_signal_until(deadline).unwrap(),
        RuntimeSignal::SnapshotCaughtUp {
            through_seq: Some(7),
            ..
        }
    ));
    assert_eq!(runtime.stream_state(), EtatFlux::Fresh);
    assert!(matches!(
        runtime.next_signal_until(deadline).unwrap(),
        RuntimeSignal::Gap {
            from_seq: 8,
            to_seq: 9,
            ..
        }
    ));
    assert_eq!(runtime.stream_state(), EtatFlux::Gap);
    let RuntimeSignal::Observation(after_gap) = runtime.next_signal_until(deadline).unwrap() else {
        panic!("une ligne valide après Gap reste une observation, pas une guérison implicite");
    };
    assert_eq!(after_gap.seq, 10);
    assert_eq!(
        after_gap.stream_state,
        EtatFlux::Gap,
        "chaque fait propage la lacune attestée du flux"
    );
    assert_eq!(
        runtime.stream_state(),
        EtatFlux::Gap,
        "Gap doit rester visible jusqu'à une nouvelle souscription"
    );
    assert!(matches!(
        runtime.next_signal_until(deadline).unwrap(),
        RuntimeSignal::End { ref reason, .. } if reason == "wrapper_parti"
    ));
    assert_eq!(runtime.stream_state(), EtatFlux::Ended);
    assert_eq!(runtime.resume_window().unwrap(), AttachWindow::Seq(11));
    server.join().expect("serveur termine");
}

#[test]
fn ignore_generation_obsolete_et_reprend_la_fenetre_initiale_apres_snapshot_vide() {
    let fixture = SocketFixture::new("empty-snapshot");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        accept_client(&listener);
        let (stream, _) = listener.accept().expect("connexion Attach attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(read_json(&mut reader)["type"], "RoleHandshake");
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
        assert_eq!(
            read_json(&mut reader)["window"],
            json!({"kind":"Date","value":"2026-08-23"})
        );
        write_json(
            &mut writer,
            json!({"type":"Subscribed","subscription_id":"sub-actif"}),
        );
        write_fragment(
            &mut writer,
            "sub-obsolete",
            4,
            0,
            true,
            br#"{"v":1,"seq":4,"ts":"2026-08-23T12:00:00Z","session_id":"old","event":"error","payload":{"reason":"ignore"}}"#,
        );
        write_json(
            &mut writer,
            json!({"type":"SnapshotCaughtUp","subscription_id":"sub-actif"}),
        );
        write_json(
            &mut writer,
            json!({"type":"End","subscription_id":"sub-actif","reason":"reconnexion"}),
        );
    });

    let client = BridgetClient::connect(fixture.path(), "maicie-instance-runtime").unwrap();
    let initial = AttachWindow::Date("2026-08-23".to_string());
    let mut runtime = RuntimeSubscription::open(&client, "prospective", initial.clone()).unwrap();
    assert!(matches!(
        runtime.next_signal().unwrap(),
        RuntimeSignal::SnapshotCaughtUp {
            through_seq: None,
            ..
        }
    ));
    assert!(matches!(
        runtime.next_signal().unwrap(),
        RuntimeSignal::End { .. }
    ));
    assert_eq!(runtime.resume_window().unwrap(), initial);
    server.join().expect("serveur termine");
}

#[test]
fn journal_read_error_reste_un_diagnostic_de_flux_sans_inference() {
    let fixture = SocketFixture::new("journal-read-error");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        accept_client(&listener);
        let (stream, _) = listener.accept().expect("connexion Attach attendue");
        let (mut reader, mut writer) = split(stream);
        let _ = read_json(&mut reader);
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
        let _ = read_json(&mut reader);
        write_json(
            &mut writer,
            json!({"type":"Subscribed","subscription_id":"sub-err"}),
        );
        write_json(
            &mut writer,
            json!({
                "type":"JournalReadError",
                "subscription_id":"sub-err",
                "line":12,
                "offset":448,
                "reason":"json_invalide"
            }),
        );
    });

    let client = BridgetClient::connect(fixture.path(), "maicie-instance-runtime").unwrap();
    let mut runtime =
        RuntimeSubscription::open(&client, "prospective", AttachWindow::Seq(12)).unwrap();
    assert!(matches!(
        runtime.next_signal().unwrap(),
        RuntimeSignal::JournalReadError { line: 12, offset: 448, ref reason, .. }
            if reason == "json_invalide"
    ));
    assert_eq!(runtime.stream_state(), EtatFlux::Gap);
    server.join().expect("serveur termine");
}

#[test]
fn abandonne_un_evenement_trop_grand_puis_consomme_jusqu_au_fragment_final() {
    let fixture = SocketFixture::new("oversized");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        accept_client(&listener);
        let (stream, _) = listener.accept().expect("connexion Attach attendue");
        let (mut reader, mut writer) = split(stream);
        let _ = read_json(&mut reader);
        write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
        let _ = read_json(&mut reader);
        write_json(
            &mut writer,
            json!({"type":"Subscribed","subscription_id":"sub-large"}),
        );

        let chunk = vec![b'x'; 64 * 1024];
        let mut offset = 0_u64;
        while offset as usize <= MAX_REASSEMBLED_EVENT_BYTES {
            write_fragment(&mut writer, "sub-large", 41, offset, false, &chunk);
            offset += chunk.len() as u64;
        }
        write_fragment(&mut writer, "sub-large", 41, offset, true, b"x");
        let next = serde_json::to_vec(&json!({
            "v":1,
            "seq":42,
            "ts":"2026-08-23T12:00:01Z",
            "session_id":"next",
            "event":"turn_end",
            "payload":{"stop_reason":"end_turn"}
        }))
        .unwrap();
        write_fragment(&mut writer, "sub-large", 42, 0, true, &next);
    });

    let client = BridgetClient::connect(fixture.path(), "maicie-instance-runtime").unwrap();
    let mut runtime =
        RuntimeSubscription::open(&client, "prospective", AttachWindow::Seq(41)).unwrap();
    assert!(matches!(
        runtime.next_signal().unwrap(),
        RuntimeSignal::Gap {
            from_seq: 41,
            to_seq: 41,
            reason: Some(ref reason),
            ..
        } if reason == "event_too_large"
    ));
    let RuntimeSignal::Observation(observation) = runtime.next_signal().unwrap() else {
        panic!("le flux doit progresser après le fragment final abandonné");
    };
    assert_eq!(observation.seq, 42);
    assert_eq!(runtime.resume_window().unwrap(), AttachWindow::Seq(43));
    server.join().expect("serveur termine");
}

fn write_fragment(
    writer: &mut BufWriter<UnixStream>,
    subscription_id: &str,
    seq: u64,
    offset: u64,
    final_fragment: bool,
    bytes: &[u8],
) {
    write_json(
        writer,
        json!({
            "type":"JournalFragment",
            "subscription_id":subscription_id,
            "seq":seq,
            "offset":offset,
            "final":final_fragment,
            "bytes":base64::engine::general_purpose::STANDARD.encode(bytes)
        }),
    );
}

fn accept_client(listener: &UnixListener) {
    let (stream, _) = listener.accept().expect("client public attendu");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().unwrap()),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

struct SocketFixture {
    path: PathBuf,
}

impl SocketFixture {
    fn new(label: &str) -> Self {
        let serial = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir().join(format!(
                "maicie-runtime-{label}-{}-{serial}.sock",
                std::process::id()
            )),
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn bind(&self) -> UnixListener {
        let _ = fs::remove_file(&self.path);
        UnixListener::bind(&self.path).unwrap()
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
