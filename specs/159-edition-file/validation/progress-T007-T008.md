# Progression159A T007/T008/T010

Dernière preuve close : **32PASS/0FAIL/0ignored, RC0,30,88s**. T007 (20 release + ACK1) et T008 (11 sweep) ont tous passé, aux noms exacts identiques aux cohortes antérieures. T010 : quatre cas encore en cours : GREEN pending. Aucun cochage ni GREEN global.

Les résultats ci-dessous restent historiques.

Reçus ciblés réels. Aucun cochage. Aucun GREEN global.

| Preuve | Résultat | État |
|---|---|---|
| T007 release + ACK | 21PASS/0FAIL, RC0,31,93s | GREEN ciblé historique acquis |
| T008 fixture initiale | 1PASS/10FAIL, RC101,20,96s | Historique conservé |
| T008 vrai SendIdempotent + supports fermés | 3PASS/10FAIL, RC101,23,55s ;11 T008=1P10F +2 supportsPASS | RED assertions réel |
| Tentative T008 GREEN/T007 régression | RC101,6,29s ;9 erreursE0308 de mutabilité ;0 test | Compilation échouée, GREEN pending |
| T010 initial crash/SC004 | 0PASS/2FAIL, RC101,72,25s | Nonvalidé |
| T010 diagnostics bornés | Crash0P1F/27,58s puisSC0040P1F/37,22s | Cause de fixture prouvée ; GREEN pending |

T008 fixture corrigée : vrai handler, held_ids et delivery_id réels, catégorie/code et raison/texte alignés. Les10 échecs restent consignés. Aucune cause métier globale n’est déduite.

T010 : le log privé atteste held_queue=true et turn_attest=true, mais la présence est connected lors de l’admission. La fixture doit attendre le busy réellement attesté par le daemon. Le signal fournisseur seul ne prouve pas cette précondition. Aucun FULL ni résultat SC004 réussi.

Chaque run possède sa propre garde AV/AP de 222 sources stable. Ne pas additionner leurs PASS ni les attribuer au prochain SHA.

## T007-release-ACK-GREEN

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T007-release-ACK-GREEN.json
SHA reçu : `68be92da4692d5b22db9d91f4ddbf65ad5badbcce7b4c0270f014f4d6d1d0865`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T007-release-ACK-GREEN.log
SHA journal : `5ac2954bd16a0b9a0186711ed0d4886024c05aa612e22cfdc6f19872c0f9d539`

Noms réellement exécutés :

- `daemon::presence_tests::held159_release_balayage_libere_si_repos_reattesté` : ok
- `daemon::presence_tests::held159_release_ack_puis_faux_signal_terminal_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_declencheurs_concurrents_une_seule_liberation` : ok
- `daemon::presence_tests::held159_release_demande_close_juste_avant_ne_libere_pas_et_retire` : ok
- `daemon::presence_tests::held159_release_echeance_de_tour_posee_a_la_liberation` : ok
- `daemon::presence_tests::held159_release_echeance_juste_avant_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_fin_d_execution_libere` : ok
- `daemon::presence_tests::held159_release_levee_de_pause_reprise_d_abord_puis_liberation` : ok
- `daemon::presence_tests::held159_release_pas_de_liberation_si_busy` : ok
- `daemon::presence_tests::held159_release_pause_interruption_fin_de_tour_aucune_liberation` : ok
- `daemon::presence_tests::held159_release_pause_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_reconnexion_libere` : ok
- `daemon::presence_tests::held159_release_refus_de_guidage_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_repos_atteste_libere_le_plus_ancien` : ok
- `daemon::presence_tests::held159_release_reservation_levee_par_remise_indeterminee` : ok
- `daemon::presence_tests::held159_release_reservation_levee_apres_ack_et_30s_sans_busy` : ok
- `daemon::presence_tests::held159_release_reservation_levee_par_true` : ok
- `daemon::presence_tests::held159_release_reservation_survit_au_redemarrage_du_daemon` : ok
- `daemon::presence_tests::held159_release_un_seul_par_cycle_de_tour` : ok
- `idempotency::send_delivery::held159_acked_at_premier_ack_et_rejeu` : ok
- `daemon::presence_tests::held159_release_wrapper_muet_attend_la_sortie_forcee` : ok

## T008-sweep-RED

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-RED.json
SHA reçu : `acf08bc5ee4be91f25fdab80162217d9c60c052386505cc9f78efef6ee4afe94`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-RED.log
SHA journal : `5eab48b79959e0c46760275dc3dfbdd1b19b75cf060b51c1150a445af34ece86`

Noms réellement exécutés :

- `daemon::presence_tests::held159_sweep_aucun_rappel_palier_1_2_pour_une_demande_retenue` : FAILED
- `daemon::presence_tests::held159_sweep_demande_close_retire` : FAILED
- `daemon::presence_tests::held159_sweep_demande_expiree_retire` : FAILED
- `daemon::presence_tests::held159_sweep_enregistrement_expire_retire` : FAILED
- `daemon::presence_tests::held159_sweep_auteur_prevenu` : FAILED
- `daemon::presence_tests::held159_sweep_palier_3_ferme_la_demande_et_le_balayage_retire_le_message` : FAILED
- `daemon::presence_tests::held159_sweep_held_ids_reconstruit_au_demarrage` : FAILED
- `daemon::presence_tests::held159_sweep_parite_avec_les_causes_du_pont` : FAILED
- `daemon::presence_tests::held159_sweep_pending_replies_retire_apres_commit` : FAILED
- `daemon::presence_tests::held159_sweep_retenu_non_expire_conserve` : ok
- `daemon::presence_tests::held159_sweep_retrait_resultat_envoi_rejected_avec_raison` : FAILED

## T008-real-handler-RED-registry-GREEN

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T008-real-handler-RED-registry-GREEN.json
SHA reçu : `647d8a4e71a44a1c4e4ae8d6927b058ca397d297778d8333ffd4b6697f658c49`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T008-real-handler-RED-registry-GREEN.log
SHA journal : `363c527e0e4d784f4e1bc05d1bf77b9da64c0ed24ed752b3a52380fce6f8b55d`

Noms réellement exécutés :

- `daemon::presence_tests::held159_sweep_aucun_rappel_palier_1_2_pour_une_demande_retenue` : FAILED
- `daemon::presence_tests::held159_hold_ferme_identique` : ok
- `daemon::presence_tests::held159_sweep_demande_close_retire` : FAILED
- `daemon::presence_tests::held159_sweep_demande_expiree_retire` : FAILED
- `daemon::presence_tests::held159_sweep_enregistrement_expire_retire` : FAILED
- `daemon::presence_tests::held159_sweep_auteur_prevenu` : FAILED
- `daemon::presence_tests::held159_sweep_palier_3_ferme_la_demande_et_le_balayage_retire_le_message` : FAILED
- `daemon::presence_tests::held159_sweep_held_ids_reconstruit_au_demarrage` : FAILED
- `daemon::presence_tests::held159_sweep_parite_avec_les_causes_du_pont` : FAILED
- `daemon::presence_tests::held159_sweep_pending_replies_retire_apres_commit` : FAILED
- `daemon::presence_tests::held159_sweep_retenu_non_expire_conserve` : ok
- `registry::held159_hold_ferme_identique` : ok
- `daemon::presence_tests::held159_sweep_retrait_resultat_envoi_rejected_avec_raison` : FAILED

## T008-sweep-GREEN-T007-regression

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-GREEN-T007-regression.json
SHA reçu : `0e6cab26e46ea6557808794b78b7f3160f9a74583a607e5236ee663b5e4d8b2a`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-GREEN-T007-regression.log
SHA journal : `2c0622658433d6d5c0c0f263652d55605e03f1a1c9892da343916e5242076dbc`

Noms réellement exécutés :

Aucun test exécuté.

Neuf appels existants passent une référence immutable à maintain_stores qui exige &mut DaemonState. Détails fichier/ligne dans le JSON.

## Diagnostic T010

Preuve : /Users/moi/.cache/bridget-tests159A-20261012/T010-busy-precondition-diagnostic.json
SHA : `b1c93633e6a6d5d3448d3e5fb367bb338b0c780c841c88f3b6f81950a7edc6a4`
Log privé source vérifié : /private/tmp/b.nhyjmphi/tmp/h159-4b4bcd07/daemon.log
SHA : `faec68ecd025634f5338fbea6a48fb23107482c07624457d6369d87176ba6e40`

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T010-first-crash-SC004.json
SHA : `c9cf8325f6f09221efa5c498b982466f607de966f51752091c4c0aac3cc32c55`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T010-first-crash-SC004.log
SHA : `6a342806568341d3bc395281fd44b44fc18ac8efe363a5461806147931706c1c`
- `held159_daemon_arrete_apres_retention_conserve_la_remise` : FAILED, nom tiré de la panique réelle (nocapture intercalé)
- `held159_latency_retenue_codex_simule` : FAILED, nom tiré de la panique réelle (nocapture intercalé)

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T010-actual-presence-diagnostic.json
SHA : `3f510e088d60ce1d61378bbc7ec37ddb044ee0766b80baaa9c31e13a5a73177a`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T010-actual-presence-diagnostic.log
SHA : `204284b02145ccbc9ce0286263ea5250c71beb9951fdf4e98499742289f8b5bc`
- `held159_daemon_arrete_apres_retention_conserve_la_remise` : FAILED, nom tiré de la panique réelle (nocapture intercalé)

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T010-actual-presence-SC004-diagnostic.json
SHA : `4012b8debad5eaff2ee328ef8c26dd9af3d692d3301cf5d657b3bae7291d721a`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T010-actual-presence-SC004-diagnostic.log
SHA : `60d60071f7569a97ab04b824cedafa0a55789c2c9bf1da663e55349ff12301a1`
- `held159_latency_retenue_codex_simule` : FAILED, nom tiré de la panique réelle (nocapture intercalé)

JSON complet : /Users/moi/.cache/bridget-validation159A-T007-T008-20261012/progress-T007-T008.json

Aucun test, build, service, écriture WT ou Git exécuté par cet agent.

## Dernier GREEN T008 et régression T007

Reçu : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-GREEN-T007-regression-mutable-calls.json
SHA reçu : `e1119dbcb1bf4aa1f86b1f11cda2f15c232f66dca8a83c167669c38c80630898`
Journal : /Users/moi/.cache/bridget-tests159A-20261012/T008-sweep-GREEN-T007-regression-mutable-calls.log
SHA journal : `a362141551794139c9109f87ed5391735f84517c9f68a1ec7f3a728d6bebedde`

222 sources identiques avant/après. Source daemon : `a3ba97f2c1d1897beef098b20edddb0c128c8e582dd8dbeef503eb8c2fca85f2`.

T007_release : 20 noms PASS exacts.

- `daemon::presence_tests::held159_release_ack_puis_faux_signal_terminal_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_balayage_libere_si_repos_reattesté` : ok
- `daemon::presence_tests::held159_release_declencheurs_concurrents_une_seule_liberation` : ok
- `daemon::presence_tests::held159_release_demande_close_juste_avant_ne_libere_pas_et_retire` : ok
- `daemon::presence_tests::held159_release_echeance_de_tour_posee_a_la_liberation` : ok
- `daemon::presence_tests::held159_release_echeance_juste_avant_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_fin_d_execution_libere` : ok
- `daemon::presence_tests::held159_release_levee_de_pause_reprise_d_abord_puis_liberation` : ok
- `daemon::presence_tests::held159_release_pas_de_liberation_si_busy` : ok
- `daemon::presence_tests::held159_release_pause_interruption_fin_de_tour_aucune_liberation` : ok
- `daemon::presence_tests::held159_release_pause_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_reconnexion_libere` : ok
- `daemon::presence_tests::held159_release_refus_de_guidage_ne_libere_pas` : ok
- `daemon::presence_tests::held159_release_repos_atteste_libere_le_plus_ancien` : ok
- `daemon::presence_tests::held159_release_reservation_levee_par_remise_indeterminee` : ok
- `daemon::presence_tests::held159_release_reservation_levee_apres_ack_et_30s_sans_busy` : ok
- `daemon::presence_tests::held159_release_reservation_levee_par_true` : ok
- `daemon::presence_tests::held159_release_reservation_survit_au_redemarrage_du_daemon` : ok
- `daemon::presence_tests::held159_release_un_seul_par_cycle_de_tour` : ok
- `daemon::presence_tests::held159_release_wrapper_muet_attend_la_sortie_forcee` : ok

T007_ACK : 1 noms PASS exacts.

- `idempotency::send_delivery::held159_acked_at_premier_ack_et_rejeu` : ok

T008_sweep : 11 noms PASS exacts.

- `daemon::presence_tests::held159_sweep_aucun_rappel_palier_1_2_pour_une_demande_retenue` : ok
- `daemon::presence_tests::held159_sweep_auteur_prevenu` : ok
- `daemon::presence_tests::held159_sweep_demande_close_retire` : ok
- `daemon::presence_tests::held159_sweep_demande_expiree_retire` : ok
- `daemon::presence_tests::held159_sweep_enregistrement_expire_retire` : ok
- `daemon::presence_tests::held159_sweep_palier_3_ferme_la_demande_et_le_balayage_retire_le_message` : ok
- `daemon::presence_tests::held159_sweep_held_ids_reconstruit_au_demarrage` : ok
- `daemon::presence_tests::held159_sweep_pending_replies_retire_apres_commit` : ok
- `daemon::presence_tests::held159_sweep_retenu_non_expire_conserve` : ok
- `daemon::presence_tests::held159_sweep_retrait_resultat_envoi_rejected_avec_raison` : ok
- `daemon::presence_tests::held159_sweep_parite_avec_les_causes_du_pont` : ok

Aucune suite complète déduite. T010 GREEN pending jusqu’au reçu clos de ROOT.
