//! Session 149 r6 - vecteurs SHA-256 connus pour `permission_source_revision`.
//!
//! La feature `asm` de sha2 active le backend matériel ARM64 (instructions
//! SHA256H/SHA256H2, session 149 r6 : latence du hachage du CLI Claude). Ces
//! vecteurs ne viennent pas de l'implémentation testée : FIPS 180-2 pour les
//! trois premiers, `hashlib` + `shasum -a 256` (deux implémentations externes,
//! résultats identiques) pour la table. Les longueurs encadrent les bords de
//! bloc (64 octets), de remplissage (55/56) et de lecture (65536 octets).

use bridget_transport::protocol::{permission_source_revision, recheck_permission_context_sources};
use serde_json::json;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bridget-psr149-{}-{}-{}",
        label,
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn revision_of(root: &std::path::Path, name: &str, bytes: &[u8]) -> String {
    let path = root.join(name);
    fs::write(&path, bytes).unwrap();
    permission_source_revision(&path, 16 * 1024 * 1024).unwrap()
}

#[test]
fn native149_revision_vecteurs_fips_180_2() {
    let root = fixture_root("fips");
    // Message vide, "abc", message de 448 bits (deux blocs) : FIPS 180-2 annexe B.
    assert_eq!(
        revision_of(&root, "vide", b""),
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        revision_of(&root, "abc", b"abc"),
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        revision_of(
            &root,
            "deux-blocs",
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        ),
        "sha256:248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
    // Un million de 'a' : FIPS 180-2 annexe B, plusieurs lectures de 65536 octets.
    assert_eq!(
        revision_of(&root, "million", &vec![b'a'; 1_000_000]),
        "sha256:cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

/// Octet i = (7 i + 3) mod 251 : jamais périodique sur 64 ni sur 65536 octets.
fn pattern(length: usize) -> Vec<u8> {
    (0..length).map(|i| ((i * 7 + 3) % 251) as u8).collect()
}

#[test]
fn native149_revision_vecteurs_externes_aux_bords_de_bloc_et_de_lecture() {
    let root = fixture_root("table");
    let vectors: &[(usize, &str)] = &[
        (
            1,
            "084fed08b978af4d7d196a7446a86b58009e636b611db16211b65a9aadff29c5",
        ),
        (
            55,
            "1deace58c745f3ecadde68a5923f494c3703fa73f0306483ccb898a5826e8d70",
        ),
        (
            56,
            "06dbe23685750e4d3881ded95047abaf93fa8f9c5d3501dc57c717a72ff1398e",
        ),
        (
            63,
            "47fb38b12335c9298d09280515c0666489a189d1554bb0ac1a0740806ce9d8b6",
        ),
        (
            64,
            "dfa798724b1a8014994f363e5da7474ed26ce3757fb29e07aa47ad5a9352d37b",
        ),
        (
            65,
            "dd2eab5a5507d7717c63ce953d9ac61752c21e664425ad1227054722a0d97f69",
        ),
        (
            119,
            "c6e0f435df5d7d265baacca31e0602c00aa22fa6d3819aed664649294c743756",
        ),
        (
            120,
            "17eb8960823a644bde3065620bb9d45931fe8993fd8eb692a17aff0fd725db6a",
        ),
        (
            127,
            "2409a159329f4e770a7d8128130497abb1de01ffb6ad389d97d89ebbeeb7f02c",
        ),
        (
            128,
            "3a9c1e22e20578eebe442238f431befd688bd56698eb7c77d95d3f97d8f58207",
        ),
        (
            65535,
            "e8f3d8d5ca3bbbc3435337aabd9d299b9ed4ebf56d13ae167503259ea24c1c1e",
        ),
        (
            65536,
            "93d1a595bb5828c088e99c53df8dca5511567b7724bc2325cf3e54d725fa069b",
        ),
        (
            65537,
            "1f97534600bc110bc61e3d51829deadef8a1ec24e00bb2becc3f9f75b9527e79",
        ),
        (
            131072,
            "15cfa58b3956aa3c0b306a3e8b4c7ce4fd15d7ee2567628bba5dda60f5264cbb",
        ),
        (
            131073,
            "98b39fc2db8f94e05ed2dbe35f03cdeb0dc8b3db9c976777246820ad32c7ce5c",
        ),
        (
            1000003,
            "cfac01d21a4a2bf8dc11816e3b83d63d19e7f04b1921d5ae47379a08cf856cdd",
        ),
    ];
    for (length, expected) in vectors {
        assert_eq!(
            revision_of(&root, &format!("v{length}"), &pattern(*length)),
            format!("sha256:{expected}"),
            "longueur {length}"
        );
    }
}

#[test]
fn native149_recheck_refuse_un_contenu_de_meme_taille_a_mtime_restaure() {
    // Garde contre un futur cache fondé sur (taille, mtime) : seule la
    // révision de contenu détecte cette altération.
    let root = fixture_root("meme-taille");
    let source = root.join("settings.json");
    fs::write(&source, br#"{"model":"aaaaaaaa"}"#).unwrap();
    let modified = fs::metadata(&source).unwrap().modified().unwrap();
    let revision = permission_source_revision(&source, 16 * 1024 * 1024).unwrap();
    let cli = root.join("cli");
    fs::write(&cli, b"cli-149").unwrap();
    let cli_revision = permission_source_revision(&cli, 512 * 1024 * 1024).unwrap();
    let context = json!({
        "cli_path": cli,
        "cli_revision": cli_revision,
        "resolved_cli_path": cli,
        "resolved_cli_revision": cli_revision,
        "permission_sources": [{"kind": "user", "path": source, "revision": revision}],
    });
    assert_eq!(recheck_permission_context_sources(&context), Ok(()));

    fs::write(&source, br#"{"model":"bbbbbbbb"}"#).unwrap();
    let file = fs::OpenOptions::new().write(true).open(&source).unwrap();
    file.set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    drop(file);
    let meta = fs::metadata(&source).unwrap();
    assert_eq!(
        meta.len(),
        br#"{"model":"aaaaaaaa"}"#.len() as u64,
        "même taille"
    );
    assert_eq!(meta.modified().unwrap(), modified, "mtime restauré");
    assert_eq!(
        recheck_permission_context_sources(&context),
        Err("settings_revision_changed".into())
    );
}
