use super::*;
use std::io::Write;

/// PID d'un processus terminé et récolté : garanti absent à l'instant.
fn pid_disparu() -> u32 {
    let mut child = std::process::Command::new("/usr/bin/true").spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}

/// Session 116 : un processus qui s'arrête pendant la collecte ne rend
/// plus l'inventaire des survivants « incomplet ».
#[test]
fn spec116_processus_disparu_n_invalide_pas_l_inventaire_des_vivants() {
    let path = fixture("spec116-vivant", "{}\n");
    let file = std::fs::File::open(&path).unwrap();
    let vivant = std::process::id();
    let disparu = pid_disparu();
    let inventory = open_session_files(&[vivant, disparu], is_codex_rollout)
        .expect("les survivants sont listés entièrement");
    let canonique = std::fs::canonicalize(&path).unwrap();
    assert!(
        inventory
            .get(&vivant)
            .is_some_and(|files| files.contains(&canonique)),
        "le rollout du survivant est bien inventorié"
    );
    assert!(!inventory.contains_key(&disparu));
    drop(file);
    let _ = std::fs::remove_file(path);
}

#[test]
fn spec116_seule_une_disparition_prouvee_excuse_l_echec() {
    let vivant = std::process::id();
    let disparu = pid_disparu();
    let listes = |pids: &[u32]| {
        pids.iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
    };
    assert!(only_vanished(&[vivant, disparu], &listes(&[vivant])));
    assert!(
        !only_vanished(&[vivant], &listes(&[])),
        "un processus vivant absent de l'inventaire reste un échec"
    );
    assert!(
        !only_vanished(&[vivant], &listes(&[vivant])),
        "un échec sans disparition garde sa cause inconnue"
    );
}

#[test]
fn spec133_inventaire_lsof_vide_le_tube_sans_bloquer_l_identite() {
    let root = std::env::temp_dir().join(format!(
        "bridget-runtime-many-open-files-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&root).unwrap();
    let mut open = Vec::new();
    let mut expected = Vec::new();
    for index in 0..192 {
        let path = root.join(format!(
            "rollout-{index:03}-nom-volontairement-long-pour-depasser-le-tube.jsonl"
        ));
        open.push(std::fs::File::create(&path).unwrap());
        expected.push(path.canonicalize().unwrap());
    }
    let started = std::time::Instant::now();
    let inventory = open_session_files(&[std::process::id()], is_codex_rollout).unwrap();
    assert!(started.elapsed() < std::time::Duration::from_secs(3));
    for path in expected {
        assert!(inventory[&std::process::id()].contains(&path));
    }
    drop(open);
    std::fs::remove_dir_all(root).unwrap();
}

fn fixture(name: &str, content: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bridget-runtime-{}-{}.jsonl",
        std::process::id(),
        name
    ));
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

#[test]
fn claude_transcript_nominal() {
    let path = fixture(
        "claude-nominal",
        concat!(
            r#"{"type":"user","message":{"role":"user"}}"#,
            "\n",
            r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
            "\n"
        ),
    );
    let observed = parse_claude_transcript(&path).unwrap();
    assert_eq!(observed.model, "claude-opus-5");
    assert_eq!(observed.effort.as_deref(), Some("high"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn claude_ignore_le_sous_agent_et_les_lignes_sans_modele() {
    // La ligne la plus récente est un sous-agent sur un autre modèle, la
    // précédente une ligne assistant synthétique sans modèle : ni l'une ni
    // l'autre ne décrit le modèle de la session principale.
    let path = fixture(
        "claude-sidechain",
        concat!(
            r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
            "\n",
            r#"{"type":"assistant","isSidechain":false,"message":{"role":"assistant"}}"#,
            "\n",
            r#"{"type":"assistant","isSidechain":true,"message":{"model":"claude-haiku-4-5"}}"#,
            "\n"
        ),
    );
    let observed = parse_claude_transcript(&path).unwrap();
    assert_eq!(observed.model, "claude-opus-5");
    let _ = std::fs::remove_file(path);
}

#[test]
fn claude_effort_absent_reste_absent() {
    // Cas Haiku observé en Phase 0 : aucun champ `effort`.
    let path = fixture(
        "claude-sans-effort",
        concat!(
            r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-haiku-4-5-20251001"}}"#,
            "\n"
        ),
    );
    let observed = parse_claude_transcript(&path).unwrap();
    assert_eq!(observed.model, "claude-haiku-4-5-20251001");
    assert_eq!(observed.effort, None);
    let _ = std::fs::remove_file(path);
}

#[test]
fn codex_rollout_nominal() {
    let path = fixture(
        "codex-nominal",
        concat!(
            r#"{"type":"session_meta","payload":{"cli_version":"0.147.0"}}"#,
            "\n",
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"xhigh"}}"#,
            "\n"
        ),
    );
    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    assert_eq!(observed.effort.as_deref(), Some("xhigh"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn codex_dernier_turn_context_sans_modele_est_ignore() {
    let path = fixture(
        "codex-sans-modele",
        concat!(
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"high"}}"#,
            "\n",
            r#"{"type":"turn_context","payload":{"cwd":"/tmp"}}"#,
            "\n"
        ),
    );
    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    let _ = std::fs::remove_file(path);
}

#[test]
fn fenetre_tronquee_aux_deux_bornes() {
    // Fichier plus grand que la fenêtre initiale : la première ligne lue
    // est coupée, et la dernière est en cours d'écriture (sans \n final).
    // Aucune des deux ne doit produire d'observation ni faire échouer le
    // scan. Cas soulevé par la contre-revue « agent-1 ».
    let mut content = String::new();
    content.push_str(&format!(
        "{{\"type\":\"turn_context\",\"payload\":{{\"model\":\"ancien\",\"filler\":\"{}\"}}}}\n",
        "x".repeat(300 * 1024)
    ));
    content
        .push_str(r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"low"}}"#);
    content.push('\n');
    content.push_str(r#"{"type":"turn_context","payload":{"model":"tronq"#);
    let path = fixture("codex-tronque", &content);

    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    assert_eq!(observed.effort.as_deref(), Some("low"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn fenetre_agrandie_quand_l_observation_est_plus_haut() {
    // L'unique turn_context est au début d'un fichier dépassant la fenêtre
    // initiale : l'agrandissement doit le retrouver.
    let mut content =
        String::from(r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex"}}"#);
    content.push('\n');
    for _ in 0..40 {
        content.push_str(&format!(
            "{{\"type\":\"response_item\",\"payload\":{{\"filler\":\"{}\"}}}}\n",
            "y".repeat(10 * 1024)
        ));
    }
    let path = fixture("codex-loin", &content);

    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    let _ = std::fs::remove_file(path);
}

#[test]
fn fichier_vide_ou_absent_ne_produit_rien() {
    let path = fixture("vide", "");
    assert!(parse_codex_rollout(&path).is_none());
    assert!(parse_claude_transcript(&path).is_none());
    let _ = std::fs::remove_file(&path);

    let absent = std::env::temp_dir().join("bridget-runtime-inexistant.jsonl");
    assert!(parse_codex_rollout(&absent).is_none());
}

#[test]
fn derniere_ligne_complete_sans_saut_final_est_lue() {
    // Un fichier bien formé peut ne pas se terminer par un saut de ligne.
    // Sa dernière ligne porte pourtant l'observation la plus récente.
    let path = fixture(
        "sans-saut-final",
        r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"medium"}}"#,
    );
    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    assert_eq!(observed.effort.as_deref(), Some("medium"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn derniere_ligne_incomplete_est_rejetee_au_profit_de_la_precedente() {
    // Écriture en cours : le JSON final est tronqué. Le parseur doit
    // l'écarter et retenir l'observation complète qui précède.
    let path = fixture(
        "ecriture-en-cours",
        concat!(
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"low"}}"#,
            "\n",
            r#"{"type":"turn_context","payload":{"model":"gpt-5.4-cod"#
        ),
    );
    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    assert_eq!(observed.effort.as_deref(), Some("low"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn rollout_actif_est_le_plus_recemment_ecrit() {
    // Après un `codex resume`, deux rollouts sont ouverts par le même
    // processus. Le plus récemment écrit est celui où l'agent travaille,
    // quel que soit l'ordre de leur nom ou de leur date de création.
    // Fixture demandée par la contre-revue « agent-1 ».
    let ancien = fixture("rollout-ancien", "{}\n");
    let recent = fixture("rollout-recent", "{}\n");
    let hier = std::time::SystemTime::now() - std::time::Duration::from_secs(86_400);
    File::open(&ancien)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(hier))
        .unwrap();

    let choisi = most_recently_written(&[ancien.clone(), recent.clone()]).unwrap();
    assert_eq!(choisi, recent);
    // L'ordre des candidats ne change pas le résultat.
    let choisi = most_recently_written(&[recent.clone(), ancien.clone()]).unwrap();
    assert_eq!(choisi, recent);

    assert!(most_recently_written(&[]).is_none());
    let _ = std::fs::remove_file(ancien);
    let _ = std::fs::remove_file(recent);
}

#[test]
fn json_invalide_est_ignore_sans_erreur() {
    let path = fixture(
        "json-casse",
        concat!(
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex"}}"#,
            "\n",
            "pas du json du tout\n",
            r#"{"type":"turn_context","payload":"#,
            "\n"
        ),
    );
    let observed = parse_codex_rollout(&path).unwrap();
    assert_eq!(observed.model, "gpt-5.3-codex");
    let _ = std::fs::remove_file(path);
}
