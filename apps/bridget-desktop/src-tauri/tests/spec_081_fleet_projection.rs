use bridget_desktop::fleet::{
    DesktopFleetSnapshotV1, FleetSourceInput, SourceKind, project_source, unavailable_source,
};

fn source(id: &str, kind: SourceKind) -> FleetSourceInput {
    FleetSourceInput {
        source_id: id.to_owned(),
        label: if kind == SourceKind::Local {
            "Cet ordinateur".to_owned()
        } else {
            format!("Serveur {id}")
        },
        kind,
        connection_state: "connected".to_owned(),
    }
}

const SNAPSHOT: &[u8] = br#"{"version": 1,
  "agents": [{
    "name": "coordinateur",
    "profile": {"display_name": "Coordinateur"},
    "project_id": "project-a",
    "state": "idle",
    "wait_state": "waiting",
    "alerts": [],
    "last_message_at": 42,
    "unread": 2,
    "agent_link": {"role": "coordinator"}
  }]
}"#;

const PROJECTS: &[u8] = br#"{"version": 1,
  "projects": [{
    "project_id": "project-a",
    "display_name": "Projet A",
    "state": "active",
    "canonical_path": "/secret/project"
  }]
}"#;

#[test]
fn homonymes_restent_distincts_par_cle_composee() {
    let first = project_source(source("source-a", SourceKind::Ssh), SNAPSHOT, PROJECTS).unwrap();
    let second = project_source(source("source-b", SourceKind::Ssh), SNAPSHOT, PROJECTS).unwrap();
    assert_ne!(first.agents[0].key, second.agents[0].key);
    assert_eq!(first.agents[0].key, "source-a:coordinateur");
    assert_eq!(second.agents[0].key, "source-b:coordinateur");
}

#[test]
fn projection_publique_ne_transporte_ni_chemin_ni_secret() {
    let projection =
        project_source(source("source-a", SourceKind::Ssh), SNAPSHOT, PROJECTS).unwrap();
    let snapshot = DesktopFleetSnapshotV1::from_sources(vec![projection]);
    let encoded = serde_json::to_string(&snapshot).unwrap();
    assert!(!encoded.contains("canonical_path"));
    assert!(!encoded.contains("/secret/project"));
    assert!(!encoded.contains("token"));
    assert!(!encoded.contains("127.0.0.1"));
}

#[test]
fn source_illisible_est_isolee_sans_effacer_les_autres_sources() {
    let healthy = project_source(source("source-a", SourceKind::Ssh), SNAPSHOT, PROJECTS).unwrap();
    let unavailable = unavailable_source(
        source("source-b", SourceKind::Ssh),
        "La lecture de cette source est indisponible.",
    );
    let snapshot = DesktopFleetSnapshotV1::from_sources(vec![healthy, unavailable]);
    assert_eq!(snapshot.agents.len(), 1);
    assert_eq!(snapshot.sources[1].connection_state, "failed");
    assert!(snapshot.sources[1].error.is_some());
}

#[test]
fn source_locale_est_dynamique_et_ne_se_distingue_pas_par_un_profil() {
    let projection =
        project_source(source("local", SourceKind::Local), SNAPSHOT, PROJECTS).unwrap();
    assert_eq!(projection.source.label, "Cet ordinateur");
    assert_eq!(projection.source.kind, SourceKind::Local);
    assert_eq!(projection.agents[0].source_id, "local");
}

#[test]
fn projet_sans_agent_reste_visible_dans_sa_source() {
    let projection = project_source(
        source("source-a", SourceKind::Ssh),
        br#"{ "version": 1, "agents": [] }"#,
        PROJECTS,
    )
    .unwrap();
    assert!(projection.agents.is_empty());
    assert_eq!(projection.source.projects[0].display_name, "Projet A");
}

#[test]
fn reponse_relais_illisible_ne_produit_ni_agent_ni_donnee_partielle() {
    let result = project_source(source("source-a", SourceKind::Ssh), b"not-json", PROJECTS);
    assert!(result.is_err());
}

#[test]
fn version_relais_inconnue_est_traitee_comme_source_illisible() {
    let result = project_source(
        source("source-a", SourceKind::Ssh),
        br#"{ "version": 2, "agents": [] }"#,
        PROJECTS,
    );
    assert!(result.is_err());
}

#[test]
fn projection_consomme_des_octets_possedes_sans_verrou_externe() {
    let snapshot = SNAPSHOT.to_vec();
    let projects = PROJECTS.to_vec();
    let projection =
        project_source(source("source-a", SourceKind::Ssh), &snapshot, &projects).unwrap();
    drop(snapshot);
    drop(projects);
    assert_eq!(
        projection.agents[0].project_name.as_deref(),
        Some("Projet A")
    );
}
