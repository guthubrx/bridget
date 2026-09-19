//! Recette du binaire Codex installé : jamais remplacé par une fausse TUI.
//! Le fournisseur HTTP synthétique ne sert qu'à rendre ses réponses répétables.
use std::process::{Command, Stdio};

#[test]
fn codex_interactif_refuse_un_pipe_avant_presence_ou_fournisseur() {
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("codex")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("stdin et stdout sur un terminal"));
}

#[test]
#[ignore = "recette Codex local explicite, BRIDGET_CODEX_090_BIN requis"]
fn vraie_tui_et_daemon_partagent_fil_messages_et_permissions() {
    recipe(&["--human-busy", "--reconnect"]);
}

fn recipe(options: &[&str]) {
    let codex =
        std::env::var("BRIDGET_CODEX_090_BIN").expect("chemin explicite du Codex de recette");
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/codex_interactive_090.py"
    );
    let output = Command::new("/usr/bin/python3")
        .arg(script)
        .arg(env!("CARGO_BIN_EXE_bridget"))
        .arg(codex)
        .args(options)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}

macro_rules! native_recipe {
    ($name:ident, $mode:literal) => {
        #[test]
        #[ignore = "recette Codex local explicite, BRIDGET_CODEX_090_BIN requis"]
        fn $name() {
            recipe(&[$mode]);
        }
    };
}
native_recipe!(
    permission_native_refusee_sans_reponse_et_session_reutilisable,
    "--deny-permission"
);
native_recipe!(
    fermeture_terminal_pendant_permission_bornee,
    "--hangup-permission"
);
native_recipe!(eof_fournisseur_pendant_permission_borne, "--provider-eof");
native_recipe!(
    nouveau_fil_reste_non_destructif_et_messages_bridget_ciblent_le_parent,
    "--new-thread"
);
native_recipe!(
    reprise_historique_reste_non_destructive_et_messages_bridget_ciblent_le_parent,
    "--resume-thread"
);
native_recipe!(
    reprise_initiale_yolo_nom_et_historique_reels,
    "--initial-resume"
);
native_recipe!(reprise_absente_refusee_avant_presence, "--missing-resume");
native_recipe!(
    reprise_nom_codex_reel_conserve_historique_et_messages,
    "--named-resume"
);
native_recipe!(menu_reprise_reel_sans_fil_provisoire, "--menu-resume");
native_recipe!(menu_annule_nettoie_serveur_avant_presence, "--menu-cancel");
native_recipe!(nom_codex_absent_refuse_sans_presence, "--name-missing");
native_recipe!(
    nom_codex_ambigu_refuse_sans_selection_arbitraire,
    "--name-ambiguous"
);
native_recipe!(
    reprise_humaine_par_nom_et_fil_sans_uuid_bridget,
    "--human-resume"
);
native_recipe!(
    reprise_historique_pagine_rendu_par_la_tui_native,
    "--paginated-resume"
);
