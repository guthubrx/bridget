use maicie::config::{ConfigError, MaicieConfig};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

const VALID_CONFIG: &str = r#"{
  "version": 1,
  "bridget_socket": "/tmp/bridget.sock",
  "database_path": "/tmp/maicie-state.db",
  "durations": {
    "short_secs": 30,
    "normal_secs": 300,
    "long_secs": 3600
  },
  "profiles": [{
    "id": "prospective",
    "display_name": "Prospective",
    "tags": ["architecture", "research"],
    "personality_ref": "profiles/prospective.md",
    "tools": ["bridget_send", "bridget_ledger"],
    "spawn_order_ref": "agents/prospective"
  }]
}"#;

#[test]
fn charge_une_configuration_entierement_declarative() {
    let fixture = Fixture::new("valid", VALID_CONFIG);

    let config = MaicieConfig::load(&fixture.path).unwrap();

    assert_eq!(config.durations.short_secs, 30);
    assert_eq!(config.durations.normal_secs, 300);
    assert_eq!(config.durations.long_secs, 3600);
    assert_eq!(config.profiles[0].tags, ["architecture", "research"]);
    assert_eq!(config.profiles[0].spawn_order_ref, "agents/prospective");
}

#[test]
fn accepte_un_budget_explicit_de_capture_status_sans_en_inventer_un() {
    let enabled = VALID_CONFIG.replace(
        "\"profiles\": [{",
        "\"status_capture_budget_ms\": 250,\n  \"profiles\": [{",
    );
    let enabled_fixture = Fixture::new("capture-enabled", &enabled);
    let disabled_fixture = Fixture::new("capture-disabled", VALID_CONFIG);

    assert_eq!(
        MaicieConfig::load(&enabled_fixture.path)
            .unwrap()
            .status_capture_budget_ms,
        Some(250)
    );
    assert_eq!(
        MaicieConfig::load(&disabled_fixture.path)
            .unwrap()
            .status_capture_budget_ms,
        None
    );
}

#[test]
fn valide_les_politiques_de_reassignation_par_classe_avant_toute_io() {
    let policies = r#"
  "coordination_policies": {
    "courte": {"version": 1, "reminder_threshold": 2, "max_reemissions": 1, "fallback_chain": []},
    "normale": {"version": 2, "reminder_threshold": 3, "max_reemissions": 2, "fallback_chain": [{"participant_id":"bob","membership_version":4}]},
    "longue": {"version": 3, "reminder_threshold": 4, "max_reemissions": 8, "fallback_chain": []}
  },
"#;
    let fixture = Fixture::new(
        "coordination-policies",
        &VALID_CONFIG.replace(
            "\"profiles\": [{",
            &format!("{policies}  \"profiles\": [{{"),
        ),
    );
    let config = MaicieConfig::load(&fixture.path).unwrap();
    assert_eq!(
        config.coordination_policies.unwrap().normale.fallback_chain[0].participant_id,
        "bob"
    );
}

#[test]
fn refuse_les_bornes_et_le_pilote_des_politiques_avant_toute_io() {
    let base = r#"
  "coordination_policies": {
    "courte": {"version": 1, "reminder_threshold": 1, "max_reemissions": 1, "fallback_chain": []},
    "normale": {"version": 1, "reminder_threshold": 1, "max_reemissions": 1, "fallback_chain": []},
    "longue": {"version": 1, "reminder_threshold": 1, "max_reemissions": 1, "fallback_chain": []}
  },
"#;
    for (label, policies, expected) in [
        (
            "reemissions",
            base.replacen("\"max_reemissions\": 1", "\"max_reemissions\": 9", 1),
            "max_reemissions",
        ),
        (
            "pilote",
            base.replacen(
                "\"fallback_chain\": []",
                "\"fallback_chain\": [{\"participant_id\":\"maicie\",\"membership_version\":1}]",
                1,
            ),
            "pilote Maicie",
        ),
    ] {
        let fixture = Fixture::new(
            label,
            &VALID_CONFIG.replace(
                "\"profiles\": [{",
                &format!("{policies}  \"profiles\": [{{"),
            ),
        );
        assert!(
            MaicieConfig::load(&fixture.path)
                .unwrap_err()
                .to_string()
                .contains(expected)
        );
    }
}

#[test]
fn refuse_un_budget_status_nul_ou_hors_borne() {
    for (label, budget) in [("zero", "0"), ("large", "30001")] {
        let invalid = VALID_CONFIG.replace(
            "\"profiles\": [{",
            &format!("\"status_capture_budget_ms\": {budget},\n  \"profiles\": [{{"),
        );
        let fixture = Fixture::new(label, &invalid);
        let error = MaicieConfig::load(&fixture.path).unwrap_err();
        assert!(error.to_string().contains("budget de capture"));
    }
}

#[test]
fn accepte_un_nom_d_agent_runtime_distinct_du_slug_de_profil() {
    let body = VALID_CONFIG.replace(
        "\"id\": \"prospective\",",
        "\"id\": \"reviewer\",\n    \"agent_name\": \"coderBridget\",",
    );
    let fixture = Fixture::new("agent-name", &body);

    let config = MaicieConfig::load(&fixture.path).unwrap();

    assert_eq!(config.profiles[0].id, "reviewer");
    assert_eq!(
        config.profiles[0].agent_name.as_deref(),
        Some("coderBridget")
    );
}

#[test]
fn refuse_une_cle_secrete_ou_inconnue_sans_l_ignorer() {
    for field in ["api_key", "OPENAI_API_KEY", "TOKEN"] {
        let with_secret = VALID_CONFIG.replace(
            "\"version\": 1,",
            &format!("\"version\": 1, \"{field}\": \"secret-interdit\","),
        );
        let fixture = Fixture::new(field, &with_secret);

        let error = MaicieConfig::load(&fixture.path).unwrap_err();

        assert!(matches!(error, ConfigError::Parse { .. }));
        assert!(error.to_string().contains(field));
        assert!(error.to_string().contains(fixture.path.to_str().unwrap()));
    }
}

#[test]
fn refuse_les_chemins_relatifs_et_la_base_bridget() {
    let relative = VALID_CONFIG.replace("/tmp/bridget.sock", "bridget.sock");
    let relative_fixture = Fixture::new("relative", &relative);
    let shared = VALID_CONFIG.replace("/tmp/maicie-state.db", "/tmp/bridget.db");
    let shared_fixture = Fixture::new("shared", &shared);

    let relative_error = MaicieConfig::load(&relative_fixture.path).unwrap_err();
    let shared_error = MaicieConfig::load(&shared_fixture.path).unwrap_err();

    assert!(relative_error.to_string().contains("chemin absolu"));
    assert!(shared_error.to_string().contains("distincte de bridget.db"));
}

#[test]
fn refuse_un_chemin_sqlite_hors_borne_avant_toute_ouverture() {
    let long_path = format!("/tmp/{}.db", "x".repeat(1024));
    let invalid = VALID_CONFIG.replace("/tmp/maicie-state.db", &long_path);
    let fixture = Fixture::new("long-database-path", &invalid);

    let error = MaicieConfig::load(&fixture.path).unwrap_err();

    assert!(error.to_string().contains("chemin SQLite"));
    assert!(error.to_string().contains("maximum 1024"));
    assert!(!std::path::Path::new(&long_path).exists());
}

#[test]
fn refuse_des_classes_de_duree_non_croissantes() {
    let invalid = VALID_CONFIG
        .replace("\"short_secs\": 30", "\"short_secs\": 300")
        .replace("\"normal_secs\": 300", "\"normal_secs\": 30");
    let fixture = Fixture::new("durations", &invalid);

    let error = MaicieConfig::load(&fixture.path).unwrap_err();

    assert!(error.to_string().contains("strictement croissants"));
}

#[test]
fn refuse_un_profil_sans_reference_spawn_order() {
    let missing = VALID_CONFIG.replace(
        "\"spawn_order_ref\": \"agents/prospective\"",
        "\"spawn_order_ref\": \"   \"",
    );
    let fixture = Fixture::new("spawn-ref", &missing);

    let error = MaicieConfig::load(&fixture.path).unwrap_err();

    assert!(error.to_string().contains("spawn_order_ref"));
}

#[test]
fn refuse_les_identifiants_et_tags_dupliques() {
    let duplicated_tag = VALID_CONFIG.replace(
        "\"architecture\", \"research\"",
        "\"architecture\", \"architecture\"",
    );
    let tag_fixture = Fixture::new("duplicate-tag", &duplicated_tag);
    let duplicated_profile = VALID_CONFIG.replace(
        "]\n}",
        ", {\n    \"id\": \"prospective\",\n    \"display_name\": \"Autre\",\n    \"tags\": [],\n    \"personality_ref\": \"profiles/autre.md\",\n    \"tools\": [],\n    \"spawn_order_ref\": \"agents/autre\"\n  }]\n}",
    );
    let profile_fixture = Fixture::new("duplicate-profile", &duplicated_profile);

    let tag_error = MaicieConfig::load(&tag_fixture.path).unwrap_err();
    let profile_error = MaicieConfig::load(&profile_fixture.path).unwrap_err();

    assert!(tag_error.to_string().contains("valeur dupliquee"));
    assert!(profile_error.to_string().contains("profil duplique"));
}

#[test]
fn refuse_deux_profils_distincts_pour_le_meme_agent_runtime() {
    let first = VALID_CONFIG.replace(
        "\"id\": \"prospective\",",
        "\"id\": \"coder-profile\",\n    \"agent_name\": \"coderBridget\",",
    );
    let duplicated_agent = first.replace(
        "]\n}",
        ", {\n    \"id\": \"cx-profile\",\n    \"agent_name\": \"coderBridget\",\n    \"display_name\": \"CX\",\n    \"tags\": [],\n    \"personality_ref\": \"profiles/cx.md\",\n    \"tools\": [],\n    \"spawn_order_ref\": \"agents/cx\"\n  }]\n}",
    );
    let fixture = Fixture::new("duplicate-agent-name", &duplicated_agent);

    let error = MaicieConfig::load(&fixture.path).unwrap_err();

    assert!(error.to_string().contains("nom d'agent duplique"));
}

#[test]
fn accepte_un_catalogue_path_absolu_declare_sans_en_inventer_un() {
    let catalogue = std::env::temp_dir().join(format!(
        "maicie-declared-catalogue-{}-{}.jsonl",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&catalogue, "").unwrap();
    let body = VALID_CONFIG.replace(
        "\"profiles\": [{",
        &format!(
            "\"catalogue_path\": \"{}\",\n  \"profiles\": [{{",
            catalogue.display()
        ),
    );
    let fixture = Fixture::new("catalogue-ok", &body);
    let config = MaicieConfig::load(&fixture.path).unwrap();
    assert_eq!(config.catalogue_path.as_deref(), Some(catalogue.as_path()));
    let _ = fs::remove_file(&catalogue);

    let without = Fixture::new("catalogue-absent", VALID_CONFIG);
    assert_eq!(
        MaicieConfig::load(&without.path).unwrap().catalogue_path,
        None
    );
}

#[test]
fn refuse_catalogue_path_vers_tasks_md_ou_symlink() {
    let root = std::env::temp_dir().join(format!(
        "maicie-config-cat-bad-{}",
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let tasks = root.join("tasks.md");
    fs::write(&tasks, "x").unwrap();
    let body_tasks = VALID_CONFIG.replace(
        "\"profiles\": [{",
        &format!(
            "\"catalogue_path\": \"{}\",\n  \"profiles\": [{{",
            tasks.display()
        ),
    );
    let fixture_tasks = Fixture::new("catalogue-tasks", &body_tasks);
    let err_tasks = MaicieConfig::load(&fixture_tasks.path).unwrap_err();
    assert!(
        err_tasks.to_string().contains("catalogue_path")
            || err_tasks.to_string().contains("artefact"),
        "{err_tasks}"
    );

    let real = root.join("real.jsonl");
    fs::write(&real, "").unwrap();
    let link = root.join("link.jsonl");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let body_link = VALID_CONFIG.replace(
        "\"profiles\": [{",
        &format!(
            "\"catalogue_path\": \"{}\",\n  \"profiles\": [{{",
            link.display()
        ),
    );
    let fixture_link = Fixture::new("catalogue-link", &body_link);
    let err_link = MaicieConfig::load(&fixture_link.path).unwrap_err();
    assert!(
        err_link.to_string().contains("symlink") || err_link.to_string().contains("catalogue_path"),
        "{err_link}"
    );
    let _ = fs::remove_dir_all(&root);
}

fn with_review_project(project_id: &str, repository_root: &str, referent_id: &str) -> String {
    VALID_CONFIG.replace(
        "\"profiles\": [{",
        &format!(
            "\"review_project\": {{\"project_id\":\"{project_id}\",\"repository_root\":\"{repository_root}\",\"referent_id\":\"{referent_id}\"}},\n  \"profiles\": [{{"
        ),
    )
}

#[test]
fn t2511d_projet_revue_est_explicite_sans_liste_critique_ni_valeur_inventee() {
    let configured = Fixture::new(
        "review-project",
        &with_review_project("bridget", "/srv/bridget", "referent-1"),
    );
    let config = MaicieConfig::load(&configured.path).unwrap();
    let project = config.review_project.unwrap();

    assert_eq!(project.project_id, "bridget");
    assert_eq!(project.repository_root, PathBuf::from("/srv/bridget"));
    assert_eq!(project.referent_id, "referent-1");

    let absent = Fixture::new("review-project-absent", VALID_CONFIG);
    assert_eq!(
        MaicieConfig::load(&absent.path).unwrap().review_project,
        None
    );

    let with_manual_map = with_review_project("bridget", "/srv/bridget", "referent-1").replace(
        "\"referent_id\":\"referent-1\"",
        "\"referent_id\":\"referent-1\",\"critical_paths\":[\"plugins/maicie/src/store.rs\"]",
    );
    let forbidden = Fixture::new("review-critical-paths", &with_manual_map);
    assert!(matches!(
        MaicieConfig::load(&forbidden.path),
        Err(ConfigError::Parse { .. })
    ));
}

#[test]
fn t2511d_racine_inexistante_reste_un_fait_de_soumission_comptable() {
    let missing = std::env::temp_dir().join(format!(
        "maicie-review-repository-absent-{}-{}",
        std::process::id(),
        NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
    ));
    assert!(
        !missing.exists(),
        "le contrôle positif exige une racine absente"
    );
    let configured = Fixture::new(
        "review-project-missing",
        &with_review_project("bridget", missing.to_str().unwrap(), "referent-1"),
    );

    let loaded = MaicieConfig::load(&configured.path);
    assert!(
        loaded.is_ok(),
        "l'existence du dépôt ne se déclare pas au chargement : {loaded:?}"
    );
    let project = loaded.unwrap().review_project.unwrap();
    assert_eq!(project.repository_root, missing);
}

#[test]
fn t2511d_refuse_identites_ou_racine_non_canoniques_avant_usage() {
    let long_root = format!("/{}", "x".repeat(1_024));
    for (label, project_id, root, referent_id, expected) in [
        (
            "review-project-id",
            "Bridget",
            "/srv/bridget",
            "referent-1",
            "project_id",
        ),
        (
            "review-project-relative",
            "bridget",
            "srv/bridget",
            "referent-1",
            "chemin absolu",
        ),
        (
            "review-project-parent",
            "bridget",
            "/srv/../bridget",
            "referent-1",
            "normalisee",
        ),
        (
            "review-project-current",
            "bridget",
            "/srv/./bridget",
            "referent-1",
            "normalisee",
        ),
        (
            "review-project-double-separator",
            "bridget",
            "/srv//bridget",
            "referent-1",
            "normalisee",
        ),
        (
            "review-project-long",
            "bridget",
            &long_root,
            "referent-1",
            "1024",
        ),
        (
            "review-project-empty-referent",
            "bridget",
            "/srv/bridget",
            "",
            "referent_id",
        ),
        (
            "review-project-self-referent",
            "bridget",
            "/srv/bridget",
            "maicie",
            "ne peut pas etre son propre referent",
        ),
    ] {
        let fixture = Fixture::new(label, &with_review_project(project_id, root, referent_id));
        let loaded = MaicieConfig::load(&fixture.path);
        assert!(loaded.is_err(), "{label} doit être refusé");
        let error = loaded.unwrap_err();
        assert!(error.to_string().contains(expected), "{label}: {error}");
    }
}

struct Fixture {
    path: PathBuf,
}

impl Fixture {
    fn new(label: &str, body: &str) -> Self {
        let suffix = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "maicie-config-{}-{label}-{suffix}.json",
            std::process::id()
        ));
        fs::write(&path, body).unwrap();
        Self { path }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
