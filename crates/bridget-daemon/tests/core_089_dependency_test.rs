//! T013 : graphe Cargo réel et paquet sans les implémentations retirées.
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn assert_graph(metadata: &Value) {
    let expected: BTreeSet<_> = ["bridget-core", "bridget-daemon", "bridget-transport"]
        .into_iter()
        .collect();
    let packages = metadata["packages"].as_array().unwrap();
    let local: BTreeSet<_> = packages
        .iter()
        .filter(|package| package["source"].is_null())
        .map(|package| package["name"].as_str().unwrap())
        .collect();
    assert_eq!(local, expected, "aucune quatrième crate locale cachée");
    assert_eq!(metadata["workspace_members"].as_array().unwrap().len(), 3);
    for package in packages {
        let name = package["name"].as_str().unwrap();
        for forbidden in [
            "guichet",
            "tauri",
            "wry",
            "dioxus",
            "eframe",
            "bollard",
            "docker-api",
        ] {
            assert!(!name.contains(forbidden), "dépendance hors noyau : {name}");
        }
    }
}

#[test]
fn graphe_cargo_reel_limite_aux_trois_crates_de_communication() {
    let log = std::env::temp_dir().join(format!("b089-meta-{}.json", uuid::Uuid::new_v4()));
    let output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&log)
        .unwrap();
    let mut child = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--format-version",
            "1",
            "--no-deps",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .stdout(output)
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            // Fils Cargo créé ici, pas un processus fournisseur ni la flotte.
            let observed = Command::new("/bin/ps")
                .args(["-p", &child.id().to_string(), "-o", "command="])
                .output()
                .expect("identifier Cargo avant arrêt");
            let observed = String::from_utf8_lossy(&observed.stdout);
            assert!(!observed.to_ascii_lowercase().contains("firefox"));
            assert!(observed.is_empty() || observed.contains("cargo"));
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "cargo metadata dépasse son budget ; preuve conservée : {}",
                log.display()
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success());
    assert_graph(&serde_json::from_slice(&fs::read(&log).unwrap()).unwrap());
    fs::remove_file(log).unwrap();
}

#[test]
#[ignore = "gate explicite scripts/package-089-core.sh dans un paquet physiquement isolé"]
fn paquet_sans_sources_metier_ni_interface_et_graphe_transitif_verifie() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    assert_eq!(
        root,
        Path::new(&std::env::var("BRIDGET_CORE_PACKAGE_ROOT").unwrap())
    );
    for absent in [".git", "plugins", "apps", "infra", "node_modules"] {
        assert!(
            !root.join(absent).exists(),
            "source interdite dans le paquet : {absent}"
        );
    }
    for absent in [
        "ui.rs",
        "project_runtime.rs",
        "project_workspace.rs",
        "disk_trend.rs",
    ] {
        assert!(!root.join("crates/bridget-daemon/src").join(absent).exists());
    }
    // Ce fichier vient de cargo metadata SANS --no-deps : ferme aussi le transitif.
    let path = std::env::var("BRIDGET_CORE_PACKAGE_METADATA").unwrap();
    let graph: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert!(graph["resolve"]["nodes"].as_array().unwrap().len() > 3);
    assert_graph(&graph);
}
