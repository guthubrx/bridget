use super::*;

/// Contrôle positif d'abord : identifiants égaux → silence. Sans lui, les
/// assertions suivantes ne prouveraient pas que l'instrument sait se taire.
#[test]
fn ecart_de_build_id_observe_les_identifiants_pas_un_libelle() {
    let local = "build-local";
    assert!(stale_daemon_warning_for(local, "cartae", local, Some("cartae")).is_none());

    let warning = stale_daemon_warning_for(local, "cartae", "build-daemon", Some("cartae"))
        .expect("écart signalé");
    assert!(warning.contains("build-daemon"));
    assert!(warning.contains(local));

    // Surface production : même binaire → silence.
    assert!(stale_daemon_warning_at(BUILD_ID, Some(&local_host())).is_none());
    assert!(stale_daemon_warning(BUILD_ID).is_none());
}

/// POINT 3 — l'attribution inversée. Un client sans build-id ne doit PAS
/// produire une affirmation sur le daemon.
/// Mutant qui tue ce test : retirer la branche `local_build_id == "unknown"`
/// → le message retombe sur « daemon périmé » et l'assertion suivante meurt.
#[test]
fn un_client_non_identifiable_n_accuse_pas_le_daemon() {
    let warning = stale_daemon_warning_for("unknown", "cartae", "24e8003", Some("monordinateur"))
        .expect("écart signalé");
    assert!(
        !warning.contains("daemon périmé"),
        "le client inconnu ne doit pas accuser le daemon : {warning}"
    );
    assert!(
        warning.contains("client non identifiable sur cartae"),
        "le fait doit porter la machine où il vaut : {warning}"
    );
    // Contrôle positif du même instrument : un client identifiable face au
    // même daemon rend bien, lui, un verdict de péremption.
    let vrai_ecart =
        stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
            .expect("écart signalé");
    assert!(vrai_ecart.contains("build du daemon différent"));
    assert!(!vrai_ecart.contains("daemon périmé"));
}

/// POINT 4 — le fait est attaché à la machine où il vaut, pas à la mienne.
/// Mutant qui tue ce test : rendre `machine` = `local_host` → l'assertion
/// « build du daemon différent sur monordinateur » meurt.
#[test]
fn le_verdict_nomme_la_machine_du_daemon_et_non_la_mienne() {
    let warning =
        stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
            .expect("écart signalé");
    assert!(
        warning.contains("build du daemon différent sur monordinateur"),
        "le verdict doit être attaché à la machine du daemon : {warning}"
    );
    assert!(
        warning.contains("client client-neuf sur cartae"),
        "le build-id local doit être attaché à MA machine : {warning}"
    );
}

/// POINT 4 — un daemon distant ne se relance pas par une commande locale.
/// Mutant qui tue ce test : retirer le bras `Some(host) if host != local_host`
/// → la remédiation redevient une commande locale et l'assertion meurt.
#[test]
fn un_daemon_distant_ne_propose_aucune_commande_locale() {
    let warning =
        stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
            .expect("écart signalé");
    assert!(
        warning.contains("vérifier les versions actives sur monordinateur"),
        "la remédiation doit nommer la machine à traiter : {warning}"
    );
    assert!(
        !warning.contains("launchctl"),
        "commande locale : {warning}"
    );
    assert!(
        !warning.contains("systemctl"),
        "commande locale : {warning}"
    );
}

/// Une remédiation du noyau indépendant ne doit jamais cibler le service
/// installé par l'ancien produit, quelle que soit la plateforme.
#[test]
fn la_remediation_locale_reste_dans_le_namespace_independant() {
    let warning = stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("cartae"))
        .expect("écart signalé");
    assert!(
        warning.contains("namespace BRIDGET_HOME/BRIDGET_SOCKET vérifié"),
        "{warning}"
    );
    assert!(
        warning.contains("ne pas relancer le service historique"),
        "{warning}"
    );
    assert!(!warning.contains("launchctl"), "{warning}");
    assert!(!warning.contains("systemctl"), "{warning}");
    assert!(!warning.contains("com.bridget.daemon"), "{warning}");
}

/// Machine du daemon non attestée : le message le DIT au lieu de laisser
/// croire qu'il parle de la machine locale.
/// Mutant qui tue ce test : `unwrap_or(local_host)` → l'assertion meurt.
#[test]
fn une_machine_non_attestee_est_nommee_comme_telle() {
    let warning =
        stale_daemon_warning_for("client-neuf", "cartae", "24e8003", None).expect("écart");
    assert!(warning.contains(MACHINE_NON_ATTESTEE), "{warning}");
    assert!(
        !warning.contains("build du daemon différent sur cartae"),
        "une machine inconnue ne doit pas être supposée locale : {warning}"
    );
}

/// La chaîne complète doit rendre UNE SEULE valeur.
///
/// `wrapper::host_name` -> `build_info::local_host` -> `bridget_core::local_host`.
/// CE QUE CET ORACLE PROUVE ET CE QU'IL NE PROUVE PAS, je le dis ici plutôt
/// que de laisser croire : il constate l'égalité des valeurs rendues. Il ne
/// PROTÈGE pas contre une copie fidèle. Ce qui protège, c'est que
/// `build_info::local_host` est un `pub use` — il n'existe qu'un seul item,
/// et le langage interdit qu'un second en diverge.
#[test]
fn les_deux_chemins_rendent_la_meme_machine() {
    assert_eq!(local_host(), bridget_core::local_host());
    assert_eq!(
        local_host(),
        bridget_core::host::local_host(),
        "le ré-export et le chemin complet désignent le même item"
    );
}

/// QUATRIEME ORACLE — le seul des trois points qui produit un GESTE.
///
/// Quand aucune machine n'est attestee, `remediation` proposait une commande
/// LOCALE parce que les deux cotes portaient la meme valeur de repli. On
/// envoyait l'exploitant relancer un service sur SA machine alors que le
/// daemon est peut-etre ailleurs : il execute, rien ne se passe, et rien ne
/// lui dit pourquoi.
///
/// Mutant qui tue ce test : retirer la garde d'attestation en tete de
/// `remediation` -> une commande locale reapparait et l'assertion meurt en
/// affichant le message complet.
#[test]
fn aucune_commande_locale_quand_la_machine_n_est_pas_attestee() {
    let sentinelle = bridget_core::HOTE_NON_ATTESTE;
    let warning = stale_daemon_warning_for("client", sentinelle, "daemon", Some(sentinelle))
        .expect("écart signalé");
    assert!(
        !warning.contains("launchctl") && !warning.contains("systemctl"),
        "aucune commande locale ne doit être proposée : {warning}"
    );
    assert!(
        warning.contains("on ne sait pas où vérifier les versions"),
        "le message doit dire qu'on ignore où agir : {warning}"
    );
    // L'hôte LOCAL aussi passe par le rendu unique : sans cela le message
    // disait « sur inconnu » d'un côté et « machine non attestée » de
    // l'autre — les deux vocabulaires, dans la MÊME phrase.
    assert!(
        !warning.contains(&format!("sur {sentinelle}")),
        "la machine locale non attestée doit se lire comme telle : {warning}"
    );

    // Cas mixte : un seul côté attesté ne suffit pas non plus.
    let mixte = stale_daemon_warning_for("client", "cartae", "daemon", Some(sentinelle))
        .expect("écart signalé");
    assert!(
        !mixte.contains("systemctl") && !mixte.contains("launchctl"),
        "{mixte}"
    );

    // CONTRÔLE POSITIF : deux machines attestées et identiques → une
    // relance manuelle du namespace local, jamais du service historique.
    let local = stale_daemon_warning_for("client", "cartae", "daemon", Some("cartae"))
        .expect("écart signalé");
    assert!(
        local.contains("consulter bridget updates status"),
        "une machine attestée et locale doit recevoir sa remédiation : {local}"
    );
}

/// Les DEUX formes de l'absence se rendent PAREIL.
///
/// Mutant qui tue ce test : faire rendre `host` tel quel par `describe_host`
/// pour la sentinelle -> la première assertion meurt en affichant « inconnu ».
#[test]
fn les_deux_formes_de_l_absence_se_rendent_pareil() {
    assert_eq!(
        describe_host(Some(bridget_core::HOTE_NON_ATTESTE)),
        MACHINE_NON_ATTESTEE,
        "un champ portant le repli doit se lire comme un champ absent"
    );
    assert_eq!(describe_host(None), MACHINE_NON_ATTESTEE);
    assert_eq!(describe_host(Some("   ")), MACHINE_NON_ATTESTEE);
    // Contrôle positif : un vrai nom passe intact.
    assert_eq!(describe_host(Some("cartae")), "cartae");
}
