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
