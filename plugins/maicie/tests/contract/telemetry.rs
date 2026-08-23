use maicie::{
    domain::{EtatFlux, SourceSnapshot},
    telemetry::{TelemetryEvent, TelemetryJournal, TelemetryKind},
};
use serde_json::json;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
};
use uuid::Uuid;

fn event() -> TelemetryEvent {
    TelemetryEvent {
        observed_at: 1_787_464_900,
        objective_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        delegation_id: Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap(),
        message_id: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
        source: SourceSnapshot::AcpSubscription,
        freshness: EtatFlux::Gap,
        event: TelemetryKind::Correlation,
    }
}

#[test]
fn journalise_une_ligne_structuree_et_correlee() {
    let root = temporary_root("ligne-correlee");
    let path = root.join("events.jsonl");
    let mut journal = TelemetryJournal::open(&path).unwrap();
    journal.append(&event()).unwrap();
    drop(journal);

    let line = fs::read_to_string(&path).unwrap();
    assert_eq!(
        line,
        concat!(
            r#"{"observed_at":1787464900,"objective_id":"11111111-1111-1111-1111-111111111111","delegation_id":"22222222-2222-2222-2222-222222222222","message_id":"33333333-3333-3333-3333-333333333333","source":"acp_subscription","freshness":"gap","event":"correlation"}"#,
            "\n",
        )
    );

    let decoded: serde_json::Value = serde_json::from_str(line.trim_end()).unwrap();
    assert_eq!(
        decoded,
        json!({
            "observed_at": 1_787_464_900,
            "objective_id": "11111111-1111-1111-1111-111111111111",
            "delegation_id": "22222222-2222-2222-2222-222222222222",
            "message_id": "33333333-3333-3333-3333-333333333333",
            "source": "acp_subscription",
            "freshness": "gap",
            "event": "correlation",
        })
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn refuse_un_corps_non_prevu_par_le_schema_de_telemetrie() {
    let mut value = serde_json::to_value(event()).unwrap();
    value["body"] = json!("contenu qui ne doit pas etre journalise");

    assert!(serde_json::from_value::<TelemetryEvent>(value).is_err());
}

#[test]
fn reouvre_le_journal_durable_prive_apres_un_append() {
    let root = temporary_root("reouverture");
    let path = root.join("telemetry.jsonl");
    {
        let mut journal = TelemetryJournal::open(&path).unwrap();
        assert_eq!(journal.path(), path);
        journal.append(&event()).unwrap();
    }

    let mut second = event();
    second.event = TelemetryKind::Decision;
    let mut journal = TelemetryJournal::open(&path).unwrap();
    journal.append(&second).unwrap();
    drop(journal);

    let lines = fs::read_to_string(&path).unwrap();
    let decoded = lines
        .lines()
        .map(serde_json::from_str::<TelemetryEvent>)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(decoded, vec![event(), second]);
    assert_eq!(
        fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn refuse_les_liens_symboliques_vers_repertoire_ou_fichier_prive() {
    let root = temporary_root("symlinks");
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();

    let real_directory = root.join("real");
    fs::create_dir(&real_directory).unwrap();
    fs::set_permissions(&real_directory, fs::Permissions::from_mode(0o700)).unwrap();
    let linked_directory = root.join("linked");
    symlink(&real_directory, &linked_directory).unwrap();
    assert!(TelemetryJournal::open(linked_directory.join("events.jsonl")).is_err());

    let real_file = real_directory.join("real.jsonl");
    fs::write(&real_file, b"").unwrap();
    fs::set_permissions(&real_file, fs::Permissions::from_mode(0o600)).unwrap();
    let linked_file = real_directory.join("linked.jsonl");
    symlink(&real_file, &linked_file).unwrap();
    assert!(TelemetryJournal::open(&linked_file).is_err());
    fs::remove_dir_all(root).unwrap();
}

fn temporary_root(suffix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "maicie-telemetry-{suffix}-{}-{}",
        std::process::id(),
        Uuid::new_v4()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}
