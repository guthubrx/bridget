use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-cli-arguments-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    ))
}

fn run_cli(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .stdin(Stdio::null())
        .output()
        .expect("exécuter le vrai binaire bridget")
}

#[test]
fn daemon_stop_est_refuse_avant_tout_effet_de_bord() {
    let root = fixture_root("daemon");
    let cache = root.join(".cache/bridget");
    fs::create_dir_all(&cache).unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o500)).unwrap();

    let output = run_cli(&root, &["daemon", "stop"]);
    let pid_exists = cache.join("bridget.pid").exists();
    let socket_exists = cache.join("bridget.sock").exists();
    let database_exists = cache.join("bridget.db").exists();

    fs::set_permissions(&cache, fs::Permissions::from_mode(0o700)).unwrap();
    fs::remove_dir_all(&root).unwrap();

    assert_eq!(output.status.code(), Some(2), "sortie réelle: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("daemon"), "commande absente de {stderr}");
    assert!(stderr.contains("stop"), "argument absent de {stderr}");
    assert!(!pid_exists, "la validation ne doit pas créer le PID file");
    assert!(!socket_exists, "la validation ne doit pas créer la socket");
    assert!(!database_exists, "la validation ne doit pas créer la base");
}

#[test]
fn toutes_les_commandes_fermees_refusent_le_surplus_dans_le_vrai_binaire() {
    let root = fixture_root("matrice");
    fs::create_dir_all(&root).unwrap();
    let cases = vec![
        vec!["mcp", "SURPLUS"],
        vec!["discover", "SURPLUS"],
        vec!["status", "SURPLUS"],
        vec!["ledger", "SURPLUS"],
        vec!["version", "SURPLUS"],
        vec!["help", "SURPLUS"],
        vec!["cancel", "--SURPLUS"],
        vec!["cancel", "demande", "SURPLUS"],
        vec!["hook", "claude-runtime", "SURPLUS"],
        vec!["hook", "claude-statusline", "SURPLUS"],
        vec!["install-hooks", "--remove", "SURPLUS"],
        vec!["domain", "revue", "SURPLUS"],
        vec!["dnd", "off", "SURPLUS"],
        vec!["agents", "--json", "SURPLUS"],
        vec!["who", "--domain", "revue", "SURPLUS"],
        vec!["cleanup", "--dry-run", "SURPLUS"],
    ];

    for args in cases {
        let output = run_cli(&root, &args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{} n'a pas refusé le surplus: {output:?}",
            args.join(" ")
        );
        assert!(
            stderr.contains("SURPLUS"),
            "{} n'a pas nommé le surplus dans {stderr}",
            args.join(" ")
        );
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nom_de_hook_inconnu_reste_une_exception_fail_soft() {
    let root = fixture_root("hook-inconnu");
    fs::create_dir_all(&root).unwrap();
    let output = run_cli(&root, &["hook", "extension-future", "payload-opaque"]);
    fs::remove_dir_all(root).unwrap();

    assert!(output.status.success(), "sortie réelle: {output:?}");
    assert!(output.stdout.is_empty(), "stdout doit rester vide");
}
