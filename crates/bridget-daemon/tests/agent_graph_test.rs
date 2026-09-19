use bridget_daemon::desired_state::DesiredStateStore;
use bridget_daemon::fleet::{
    AgentLinkState, FleetConfig, FleetError, FleetSupervisor, SpawnOrder, SpawnOwnership,
    SpawnSubmission,
};
use std::path::{Path, PathBuf};

const NOW: i64 = 1_788_000_000;

fn root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-agent-graph-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn supervisor(root: &Path) -> FleetSupervisor {
    FleetSupervisor::open(
        &root.join("bridget.db"),
        DesiredStateStore::at_path(root.join("fleet.json")),
        FleetConfig {
            quota: 8,
            ..FleetConfig::default()
        },
    )
    .expect("superviseur")
}

fn ownership(
    parent_instance_id: &str,
    max_children: Option<usize>,
    max_depth: Option<usize>,
) -> SpawnOwnership {
    SpawnOwnership {
        parent_instance_id: parent_instance_id.to_string(),
        parent_execution_id: Some("execution-parent".to_string()),
        objective_id: Some("objective-1".to_string()),
        delegation_id: Some("delegation-1".to_string()),
        project: None,
        role: "verification".to_string(),
        max_children,
        max_depth,
    }
}

fn order(command_id: &str, name: &str, ownership: Option<SpawnOwnership>) -> SpawnOrder {
    // Les libellés ne sont que des repères de fixture ; le graphe utilise
    // l'identité opaque v2, stable pour un même libellé.
    use std::hash::{Hash, Hasher};
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hash);
    let agent_id = format!(
        "89000000-0000-4000-8000-{:012x}",
        hash.finish() & 0xffffffffffff
    );
    SpawnOrder {
        posture: None,
        agent_type: "fixture".to_string(),
        project: None,
        requested_name: Some(agent_id),
        cwd: PathBuf::from("/tmp"),
        persistent: false,
        command_id: command_id.to_string(),
        issued_at: NOW,
        deadline_at: NOW + 60,
        ownership,
    }
}

fn reserve(
    supervisor: &FleetSupervisor,
    command_id: &str,
    name: &str,
    ownership: Option<SpawnOwnership>,
) -> bridget_daemon::fleet::SpawnLease {
    match supervisor
        .request_spawn(&order(command_id, name, ownership), NOW)
        .expect("réservation")
    {
        SpawnSubmission::Start(lease) => lease,
        other => panic!("démarrage attendu : {other:?}"),
    }
}

#[test]
fn reservations_bornent_enfants_profondeur_cycle_et_rollback() {
    let root = root("reservation");
    std::fs::create_dir_all(&root).unwrap();
    let supervisor = supervisor(&root);

    let first = reserve(
        &supervisor,
        "spawn-first",
        "first",
        Some(ownership("parent-root", Some(1), Some(3))),
    );
    assert!(matches!(
        supervisor.agent_link_for_child(&first.instance_id).unwrap(),
        Some(link) if link.state == AgentLinkState::Reserved
            && link.parent_instance_id == "parent-root"
            && link.parent_execution_id.as_deref() == Some("execution-parent")
            && link.objective_id.as_deref() == Some("objective-1")
            && link.delegation_id.as_deref() == Some("delegation-1")
    ));

    let quota = supervisor
        .request_spawn(
            &order(
                "spawn-quota",
                "quota",
                Some(ownership("parent-root", Some(1), Some(3))),
            ),
            NOW,
        )
        .unwrap();
    assert!(matches!(
        quota,
        SpawnSubmission::Terminal(bridget_daemon::idempotency::SpawnCommandIssue::Failed {
            category,
            ..
        }) if category == "children_quota_exceeded"
    ));
    assert_eq!(
        supervisor
            .agent_links_for_parent("parent-root")
            .unwrap()
            .len(),
        1
    );

    let depth = supervisor
        .request_spawn(
            &order(
                "spawn-depth",
                "depth",
                Some(ownership(&first.instance_id, Some(4), Some(1))),
            ),
            NOW,
        )
        .unwrap();
    assert!(matches!(
        depth,
        SpawnSubmission::Terminal(bridget_daemon::idempotency::SpawnCommandIssue::Failed {
            category,
            ..
        }) if category == "depth_exceeded"
    ));

    let second = reserve(
        &supervisor,
        "spawn-second",
        "second",
        Some(ownership(&first.instance_id, Some(4), Some(3))),
    );
    let first_link = first.link_id.as_deref().expect("lien premier");
    assert!(matches!(
        supervisor.transfer_agent_link(first_link, &second.instance_id, NOW + 1),
        Err(FleetError::Ownership("cycle d'ascendance"))
    ));

    supervisor
        .fail(&second, "rollback", "préparation refusée")
        .unwrap();
    assert!(matches!(
        supervisor.agent_link_for_child(&second.instance_id).unwrap(),
        Some(link) if link.state == AgentLinkState::Closed && link.closed_at.is_some()
    ));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn disparition_parent_conserve_lien_et_transfert_explicite_gouverne_resultat_tardif() {
    let root = root("orphan");
    std::fs::create_dir_all(&root).unwrap();
    let supervisor = supervisor(&root);
    let child = reserve(
        &supervisor,
        "spawn-child",
        "child",
        Some(ownership("parent-disparu", Some(2), Some(3))),
    );
    let link_id = child.link_id.as_deref().expect("lien durable").to_string();

    assert_eq!(
        supervisor
            .orphan_agent_links_for_parent("parent-disparu", NOW + 1)
            .unwrap(),
        1
    );
    assert!(matches!(
        supervisor.agent_link_for_child(&child.instance_id).unwrap(),
        Some(link) if link.state == AgentLinkState::Orphaned
            && link.closed_at.is_none()
            && link.parent_instance_id == "parent-disparu"
    ));

    supervisor
        .transfer_agent_link(&link_id, "parent-repreneur", NOW + 2)
        .unwrap();
    assert!(matches!(
        supervisor.agent_link_for_child(&child.instance_id).unwrap(),
        Some(link) if link.state == AgentLinkState::Transferred
            && link.parent_instance_id == "parent-repreneur"
            && link.revision == 2
    ));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn evenements_de_lien_sont_curses_rejouables_et_reveillent_le_parent() {
    let root = root("events");
    std::fs::create_dir_all(&root).unwrap();
    let supervisor = supervisor(&root);
    let child = reserve(
        &supervisor,
        "spawn-events",
        "events",
        Some(ownership("parent-events", Some(2), Some(3))),
    );

    let initial = supervisor
        .wait_for_agent_link_events("parent-events", None, std::time::Duration::ZERO)
        .unwrap();
    assert_eq!(initial.events.len(), 1);
    assert_eq!(initial.events[0].state, AgentLinkState::Reserved);
    let cursor = initial.through_cursor.expect("curseur de réservation");

    let empty = supervisor
        .wait_for_agent_link_events(
            "parent-events",
            Some(cursor),
            std::time::Duration::from_millis(10),
        )
        .unwrap();
    assert!(empty.events.is_empty());

    std::thread::scope(|scope| {
        let waiting = scope.spawn(|| {
            supervisor
                .wait_for_agent_link_events(
                    "parent-events",
                    Some(cursor),
                    std::time::Duration::from_secs(1),
                )
                .unwrap()
        });
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert_eq!(
            supervisor
                .orphan_agent_links_for_parent("parent-events", NOW + 1)
                .unwrap(),
            1
        );
        let awakened = waiting.join().unwrap();
        assert!(matches!(
            awakened.events.as_slice(),
            [event] if event.state == AgentLinkState::Orphaned
                && event.child_instance_id == child.instance_id
                && event.cursor > cursor
        ));
        assert_eq!(awakened.through_cursor, Some(awakened.events[0].cursor));
    });

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn lien_et_journal_survivent_au_redemarrage_du_superviseur() {
    let root = root("restart");
    std::fs::create_dir_all(&root).unwrap();
    let child = {
        let supervisor = supervisor(&root);
        let child = reserve(
            &supervisor,
            "spawn-restart",
            "restart",
            Some(ownership("parent-restart", Some(2), Some(3))),
        );
        assert!(
            supervisor
                .agent_link_for_child(&child.instance_id)
                .unwrap()
                .is_some()
        );
        child
    };

    let reopened = supervisor(&root);
    assert!(matches!(
        reopened.agent_link_for_child(&child.instance_id).unwrap(),
        Some(link) if link.state == AgentLinkState::Reserved
            && link.parent_instance_id == "parent-restart"
    ));
    let replay = reopened
        .wait_for_agent_link_events("parent-restart", None, std::time::Duration::ZERO)
        .unwrap();
    assert!(matches!(
        replay.events.as_slice(),
        [event] if event.child_instance_id == child.instance_id
            && event.state == AgentLinkState::Reserved
    ));

    std::fs::remove_dir_all(root).unwrap();
}
