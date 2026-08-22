//! Gate manuelle T1006 : exécute les vrais harness Codex et Claude à travers
//! `bridget <type>`, avec les options construites par le wrapper de production.
//!
//! Ce banc est ignoré par défaut : il requiert une session authentifiée de
//! chaque harness et peut consommer du quota. Exécution explicite :
//! `BRIDGET_MCP_REAL_SMOKE=1 cargo test -p bridget-daemon --features
//! test-support --test mcp_injection_smoke_test -- --ignored --test-threads=1`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PROBE_PROMPT: &str =
    "Appelle exactement une fois l'outil MCP probe puis réponds seulement avec son résultat.";

fn config_files(home: &Path) -> [PathBuf; 4] {
    [
        home.join(".claude/settings.json"),
        home.join(".claude/settings.local.json"),
        home.join(".codex/config.toml"),
        home.join(".gemini/settings.json"),
    ]
}

fn config_snapshot(home: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    config_files(home)
        .into_iter()
        .map(|path| {
            let contents = match fs::read(&path) {
                Ok(contents) => Some(contents),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("lecture impossible {}: {error}", path.display()),
            };
            (path, contents)
        })
        .collect()
}

fn fake_server() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/010-mcp/spike/fake-mcp-server.py")
}

fn run_harness(home: &Path, agent: &str, arguments: &[&str]) {
    let log = std::env::temp_dir().join(format!(
        "bridget-t1006-{agent}-{}-{}.log",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let before = config_snapshot(home);
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg(agent)
        .args(arguments)
        .env("BRIDGET_TEST_MCP_SERVER_COMMAND", "python3")
        .env(
            "BRIDGET_TEST_MCP_SERVER_ARGS",
            serde_json::to_string(&vec![fake_server().display().to_string()]).unwrap(),
        )
        .env("BRIDGET_MCP_SMOKE_LOG", &log)
        .output()
        .unwrap_or_else(|error| panic!("lancement {agent} impossible: {error}"));
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{agent} a échoué : {combined}");
    assert!(combined.contains("PROBE_OK"), "{agent} n'a pas retourné PROBE_OK : {combined}");

    let log_contents = fs::read_to_string(&log)
        .unwrap_or_else(|error| panic!("journal MCP {agent} absent: {error}"));
    assert!(
        log_contents.contains("method=tools/list"),
        "{agent} n'a pas listé les outils : {log_contents}"
    );
    assert!(
        log_contents.contains("method=tools/call") && log_contents.contains("probe"),
        "{agent} n'a pas appelé probe : {log_contents}"
    );
    assert_eq!(
        config_snapshot(home),
        before,
        "{agent} a modifié une configuration utilisateur persistante"
    );
    fs::remove_file(log).unwrap();
}

#[test]
#[ignore = "requiert les profils Codex et Claude authentifiés ; gate manuelle T1006"]
fn wrapper_de_production_injecte_le_serveur_mcp_ephemere_dans_les_harness_disponibles() {
    assert_eq!(
        std::env::var("BRIDGET_MCP_REAL_SMOKE").as_deref(),
        Ok("1"),
        "fixez BRIDGET_MCP_REAL_SMOKE=1 pour confirmer l'exécution volontaire"
    );
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME requis"));

    run_harness(
        &home,
        "codex",
        &["exec", "--ephemeral", "--skip-git-repo-check", PROBE_PROMPT],
    );
    run_harness(&home, "claude", &["-p", PROBE_PROMPT, "--output-format", "text"]);

    // Gemini reste volontairement `unsupported`, conformément au constat
    // documenté T708/T1001 ; aucune écriture de configuration ne lui est
    // attribuée par le wrapper.
}
