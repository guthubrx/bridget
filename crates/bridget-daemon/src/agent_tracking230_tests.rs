use super::*;
use crate::store::Store;
use bridget_transport::agent_tracking::AgentTrackingSyncOutcome;

const AGENT: &str = "23000000-0000-4000-8000-000000000001";
const THREAD: &str = "thread-230";
fn agent() -> ScopedAgentInfo {
    serde_json::from_value(json!({"agent_id":AGENT,"display_name":"Témoin sommeil","agent_type":"fixture",
        "connection_id":"test","host":"test-host","transport":"fixture","state":"busy","last_seen_secs":0,
        "reconnect_count":0,"communication_project":{"host":"test-host","root":"/project"},"project_relation":"same"})).unwrap()
}
struct Fixture {
    store: Store,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("bridget-tracking230-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let store = Store::open(&path.join("state.db")).unwrap();
        Self { store, path }
    }
    fn set(&self, sleep: bool, sync: Option<bool>) -> Value {
        let id = uuid::Uuid::new_v4().to_string();
        set(
            self.store.connection(),
            "human",
            &id,
            id.as_bytes(),
            &agent(),
            sleep,
            sync,
            None,
            100,
        )
        .unwrap()
    }
    fn observe(&self, revision: &str, state: Option<&str>, confirm: Option<&str>) -> Value {
        sync(
            self.store.connection(),
            "test-host",
            &[AgentTrackingObservation {
                thread_id: THREAD.into(),
                source_revision: revision.into(),
                source_sequence: revision
                    .chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .unwrap(),
                explicit_state: state.map(str::to_owned),
                can_settle: true,
            }],
            &confirm
                .into_iter()
                .map(|id| AgentTrackingConfirmation {
                    thread_id: THREAD.into(),
                    operation_id: id.into(),
                    attempt: 0,
                    outcome: AgentTrackingSyncOutcome::Applied { sequence: 0 },
                })
                .collect::<Vec<_>>(),
            101,
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn tracking230_persiste_rejoue_et_refuse_enveloppe_differente() {
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    let conn = f.store.connection();
    let first = set(
        conn,
        "human",
        &id,
        b"same",
        &agent(),
        true,
        None,
        Some(0),
        100,
    )
    .unwrap();
    assert_eq!(first["tracking"]["synchronize"], true);
    assert!(dormant(conn, AGENT).unwrap());
    let changes = conn.total_changes();
    assert_eq!(
        set(
            conn,
            "human",
            &id,
            b"same",
            &agent(),
            true,
            None,
            Some(0),
            102
        )
        .unwrap(),
        first
    );
    assert_eq!(conn.total_changes(), changes);
    assert_eq!(
        set(
            conn,
            "human",
            &id,
            b"other",
            &agent(),
            false,
            None,
            None,
            103
        ),
        Err("envelope_mismatch")
    );
    let read = Store::open_read_only(&f.path.join("state.db")).unwrap();
    assert!(dormant(&read, AGENT).unwrap());
    assert_eq!(retained(&read).unwrap()[AGENT].agent.state, "unobserved");
    assert_eq!(
        conn.query_row(
            "SELECT MAX(version) FROM idempotency_schema_migrations",
            [],
            |r| r.get::<_, u64>(0)
        )
        .unwrap(),
        8
    );
}

#[test]
fn tracking230_synchronise_dans_les_deux_sens_sans_echo_ni_reveil_par_message() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.observe("2026-11-03T08:00:00.000Z", Some("active"), None);
    let result = f.set(true, None);
    let id = result["request_id"].as_str().unwrap();
    let pending = f.observe("2026-11-03T08:00:00.000Z", Some("active"), None);
    assert_eq!(pending["pending"][0]["operation_id"], id);
    assert!(
        dormant(c, AGENT).unwrap(),
        "l'ancien client ne défait pas le choix natif"
    );
    f.observe("2026-11-03T08:01:00.000Z", Some("settled"), Some(id));
    assert!(!view(c, AGENT).unwrap().unwrap().synchronization_pending);
    let revision = view(c, AGENT).unwrap().unwrap().revision;
    f.observe("2026-11-03T08:01:00.000Z", Some("settled"), Some(id));
    assert_eq!(
        view(c, AGENT).unwrap().unwrap().revision,
        revision,
        "aucun écho"
    );
    f.observe("2026-11-03T08:02:00.000Z", None, None);
    assert!(
        dormant(c, AGENT).unwrap(),
        "un message client ne réveille pas Bridget"
    );
    f.observe("2026-11-03T08:03:00.000Z", Some("active"), None);
    assert!(
        !dormant(c, AGENT).unwrap(),
        "la restauration explicite réveille"
    );
    f.observe("2026-11-03T08:04:00.000Z", Some("settled"), None);
    assert!(dormant(c, AGENT).unwrap(), "le rangement explicite endort");
    f.observe("2026-11-03T08:03:00.000Z", Some("active"), None);
    assert!(
        dormant(c, AGENT).unwrap(),
        "une ancienne observation est ignorée"
    );
}

#[test]
fn tracking230_mode_natif_et_fiche_absente_du_snapshot() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.set(true, Some(false));
    f.observe("1", Some("active"), None);
    assert!(dormant(c, AGENT).unwrap());
    assert!(!view(c, AGENT).unwrap().unwrap().synchronization_pending);
    f.set(false, Some(true));
    let result = sync(c, "test-host", &[], &[], 102).unwrap();
    assert_eq!(
        result["pending"].as_array().unwrap().len(),
        1,
        "le client peut réveiller un fil sorti de ses pages"
    );
    assert!(
        sync(c, "another-host", &[], &[], 102).unwrap()["pending"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let prior = view(c, AGENT).unwrap().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    assert_eq!(
        set(
            c,
            "human",
            &id,
            b"stale",
            &agent(),
            false,
            None,
            Some(prior.revision - 1),
            103
        ),
        Err("revision_conflict")
    );
    assert_eq!(view(c, AGENT).unwrap().unwrap(), prior);
}

#[test]
fn tracking230_liaison_tardive_ne_defait_pas_le_sommeil() {
    let f = Fixture::new();
    let c = f.store.connection();
    f.set(true, None);
    bind(c, &agent(), THREAD).unwrap();
    let pending = f.observe("1", Some("active"), None);
    assert!(dormant(c, AGENT).unwrap());
    assert_eq!(pending["pending"].as_array().unwrap().len(), 1);
}

#[test]
fn tracking230_un_reveil_client_posterieur_a_notre_commande_est_respecte() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.observe("1", Some("active"), None);
    let first = f.set(true, None);
    f.observe("1", Some("active"), None);
    f.observe("3", Some("active"), first["request_id"].as_str());
    assert!(
        !dormant(c, AGENT).unwrap(),
        "le reçu de notre rangement précède le nouveau réveil humain"
    );
}

fn observation(
    revision: &str,
    state: Option<&str>,
    can_settle: bool,
    sequence: u64,
) -> AgentTrackingObservation {
    AgentTrackingObservation {
        thread_id: THREAD.into(),
        source_revision: revision.into(),
        source_sequence: sequence
            + revision
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u64>()
                .unwrap(),
        explicit_state: state.map(str::to_owned),
        can_settle,
    }
}
fn report(operation: &Value, outcome: AgentTrackingSyncOutcome) -> AgentTrackingConfirmation {
    AgentTrackingConfirmation {
        thread_id: operation["thread_id"].as_str().unwrap().into(),
        operation_id: operation["operation_id"].as_str().unwrap().into(),
        attempt: operation["attempt"].as_u64().unwrap() as u32,
        outcome,
    }
}

#[test]
fn tracking230_ordre_incertain_reconcilie_avant_le_choix_suivant() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    let first = f.set(true, None);
    let a = f.observe("1", Some("active"), None)["pending"][0].clone();
    assert_eq!(a["operation_id"], first["request_id"]);
    let second = f.set(false, None);
    let retry = f.observe("2", None, None)["pending"][0].clone();
    assert_eq!(
        retry["client_command_id"], a["client_command_id"],
        "aucune nouvelle clé pendant l'incertitude"
    );
    let next = sync(
        c,
        "test-host",
        &[observation("3", Some("settled"), true, 1000)],
        &[report(
            &a,
            AgentTrackingSyncOutcome::Applied { sequence: 10 },
        )],
        103,
    )
    .unwrap();
    let b = next["pending"][0].clone();
    assert_eq!(b["operation_id"], second["request_id"]);
    assert!(
        !dormant(c, AGENT).unwrap(),
        "l'écho du premier ordre ne défait pas le choix suivant"
    );
    sync(
        c,
        "test-host",
        &[observation("4", Some("active"), true, 1000)],
        &[report(
            &b,
            AgentTrackingSyncOutcome::Applied { sequence: 11 },
        )],
        104,
    )
    .unwrap();
    assert!(!view(c, AGENT).unwrap().unwrap().synchronization_pending);
    assert_eq!(
        c.query_row(
            "SELECT COUNT(*) FROM agent_tracking_attempt_receipts",
            [],
            |r| r.get::<_, u64>(0)
        )
        .unwrap(),
        2
    );
}

#[test]
fn tracking230_refus_certain_exige_fait_nouveau_et_borne_les_essais() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.set(true, None);
    let mut previous = Value::Null;
    for attempt in 0..3 {
        let revision = (attempt * 2 + 1).to_string();
        let next = sync(
            c,
            "test-host",
            &[observation(&revision, Some("active"), true, 1000)],
            &[],
            100 + attempt,
        )
        .unwrap();
        let operation = next["pending"][0].clone();
        assert_eq!(operation["attempt"], attempt);
        if attempt > 0 {
            assert_ne!(
                previous["client_command_id"],
                operation["client_command_id"]
            );
        }
        let result = sync(
            c,
            "test-host",
            &[observation(&revision, Some("active"), true, 1000)],
            &[report(&operation, AgentTrackingSyncOutcome::Rejected {})],
            104 + attempt,
        )
        .unwrap();
        assert!(result["pending"].as_array().unwrap().is_empty());
        assert!(
            sync(
                c,
                "test-host",
                &[observation(&revision, Some("active"), true, 1000)],
                &[],
                107 + attempt
            )
            .unwrap()["pending"]
                .as_array()
                .unwrap()
                .is_empty(),
            "pas de nouvel essai sur le même fait"
        );
        let busy = (attempt * 2 + 2).to_string();
        assert!(
            sync(
                c,
                "test-host",
                &[observation(&busy, Some("active"), false, 1000)],
                &[],
                110 + attempt
            )
            .unwrap()["pending"]
                .as_array()
                .unwrap()
                .is_empty(),
            "aucun rangement pendant le travail"
        );
        previous = operation;
    }
    assert!(
        sync(
            c,
            "test-host",
            &[observation("9", Some("active"), true, 1000)],
            &[],
            120
        )
        .unwrap()["pending"]
            .as_array()
            .unwrap()
            .is_empty(),
        "plafond trois départs"
    );
    assert_eq!(
        c.query_row("SELECT phase FROM agent_tracking_outbox", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "refused"
    );
    let retry = f.set(true, None);
    assert_ne!(retry["request_id"], previous["operation_id"]);
    assert_eq!(
        sync(
            c,
            "test-host",
            &[observation("10", Some("active"), true, 1000)],
            &[],
            121
        )
        .unwrap()["pending"][0]["attempt"],
        0,
        "nouvelle confirmation humaine"
    );
}

#[test]
fn tracking230_refus_d_une_ancienne_commande_ne_rejoue_pas_un_choix_remplace() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.set(true, None);
    let a = f.observe("1", Some("active"), None)["pending"][0].clone();
    let latest = f.set(false, None);
    let result = sync(
        c,
        "test-host",
        &[observation("2", Some("active"), true, 1000)],
        &[report(&a, AgentTrackingSyncOutcome::Rejected {})],
        102,
    )
    .unwrap();
    assert_eq!(result["pending"][0]["operation_id"], latest["request_id"]);
    assert_eq!(
        c.query_row(
            "SELECT phase FROM agent_tracking_outbox WHERE operation_id=?1",
            [a["operation_id"].as_str().unwrap()],
            |r| r.get::<_, String>(0)
        )
        .unwrap(),
        "superseded"
    );
}

#[test]
fn tracking230_snapshot_anterieur_au_commit_client_est_ignore() {
    let f = Fixture::new();
    let c = f.store.connection();
    bind(c, &agent(), THREAD).unwrap();
    f.set(true, None);
    let a = f.observe("1", Some("active"), None)["pending"][0].clone();
    sync(
        c,
        "test-host",
        &[],
        &[report(
            &a,
            AgentTrackingSyncOutcome::Applied { sequence: 10 },
        )],
        102,
    )
    .unwrap();
    sync(
        c,
        "test-host",
        &[observation("3", Some("active"), true, 6)],
        &[],
        103,
    )
    .unwrap();
    assert!(dormant(c, AGENT).unwrap());
    sync(
        c,
        "test-host",
        &[observation("4", Some("active"), true, 7)],
        &[],
        104,
    )
    .unwrap();
    // Le reçu prouve le rangement intermédiaire, même sans snapshot de cet état.
    assert!(!dormant(c, AGENT).unwrap());
}

#[test]
fn tracking230_desactivation_sync_reconcilie_uniquement_la_commande_deja_partie() {
    let f=Fixture::new(); let c=f.store.connection(); bind(c,&agent(),THREAD).unwrap();
    f.set(true,None);
    let command=f.observe("1",Some("active"),None)["pending"][0].clone();
    f.set(false,Some(false));
    let retry=f.observe("2",None,None)["pending"][0].clone();
    assert_eq!(retry["client_command_id"],command["client_command_id"],"seul le reçu incertain est rapproché");
    assert_eq!(view(c,AGENT).unwrap().unwrap().synchronization_status,"awaiting_confirmation");
    let result=sync(c,"test-host",&[observation("3",Some("settled"),true,1000)],&[report(&command,AgentTrackingSyncOutcome::Applied{sequence:10})],104).unwrap();
    assert!(result["pending"].as_array().unwrap().is_empty(),"pas de nouvelle commande après désactivation");
    let view=view(c,AGENT).unwrap().unwrap(); assert!(!view.dormant); assert!(!view.synchronize); assert_eq!(view.synchronization_status,"local_only");
}
