# Tests157 — reçu ciblé avant clôture T025

Complément après revue157-r1 : le test T004 existant `native157_limits_fenetre_perimee`
est étendu à une nouvelle observation passée de la même fenêtre. RED : 0 PASS/1 FAIL,
11,97 s ; après retrait de l'ancien fait : 13 PASS/0 FAIL, 13,24 s (6 mémoire + 7 quota).
Reçus conservés ici : review157-quota-red.json et review157-quota-green.json.
Le FULL unique reste historique ; aucune seconde suite complète n'a été exécutée.

État de clôture préparé : 26/26 sous réserve de la validation finale T026 et de la capture du diff par le principal. T025 et le replay exact sont prouvés. Aucun cochage, GREEN global, Converge ou livraison n’est annoncé par ce document.

Les lignes ci-dessous décrivent des exécutions distinctes. PASS/FAIL/ignored sont les comptes des cibles finales de chaque reçu, pas un total de tests uniques. Les ensembles se recouvrent. Chaque JSON indique sources_stable:true. Les SHA256 des logs ont été vérifiés par lecture des fichiers. Les logs se trouvent dans le même dossier que leur JSON, avec le suffixe .log.

## Exécutions conservées

| Reçu JSON absolu | PASS/FAIL/ignored | RC | Secondes | SHA256 du log |
|---|---:|---:|---:|---|
| /Users/moi/.cache/bridget-plan157-20261012-r1/T001-baseline157.json | 177/0/1 | 0 | 152.36 | b615c986c43eee86fc2a6fa6452023807766a7ed2c47186ba5189b32858b914b |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T001-latence157-before-r2.json | 1/0/0 | 0 | 13.03 | 0078b7d116ce0f7968fbad7694e7c62bdd2c2762c090b4246fdfc5f670707dae |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T001-latence157-before.json | 0/1/0 | 101 | 4.00 | 1c3fe52c12b6c6d6a75be98bb80835baf1eb58df4fbffba1d68bf282ee05cb63 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T003-T006-green157-daemon.json | 24/0/0 | 0 | 12.14 | 7ddf0adf97a53cffb27e9ed32f3a0139cd6627a791801aff363e072e70dbdf06 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T003-green157-transport.json | 1/0/0 | 0 | 18.43 | 80874eb6414470f135e9a929f862239f705cdcb9b2580ae0513a84842d5611e2 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T004-T005-red157.json | 1/10/0 | 101 | 35.31 | 7d3a048d0ab925c522884b7e9567dfc0bde7e993a934b1ed3a6559afa4a11770 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T007-T010-red157.json | 0/8/0 | 101 | 13.66 | f69b2820fd760330f1d79af868c2df55a76d3e53fc505c14b68ff38a569e92b4 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T007-T011-green157-r2-cibles.json | 15/0/0 | 0 | 30.32 | 13f8be16067a32770416d361bf61b70ce32668b7aeea613172b9b6de8cd728fd |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T007-T011-green157.json | 167/2/0 | 101 | 98.99 | 9758e49474d1c353eb50396da502242213fc25d84b735dc28022345af1f56356 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T012-T015-green157.json | 19/0/0 | 0 | 21.15 | b3d5c30729d3b1cbb19c8e0afefda1f0abede19fa881d8a16f628d4333d24a9a |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T012-latence157-after-r2.json | 1/0/0 | 0 | 7.34 | ed359c0fd79cbed34ae6ff4edb3ae2433bf01e1cb7a2cba4382c4df49bf89656 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T012-latence157-after.json | 1/0/0 | 0 | 22.44 | 7f1cc369a535a23f96f46e8146306c7ff486f7376bc1d767785027f536057ab2 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T013-red157.json | 0/6/0 | 101 | 21.14 | 15e43eefa40ab3d4cf44373cd02e41b9d51ff6c2d83a8d101cdecc456d673d26 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T016-T020-red157-T014-historique.json | 2/3/0 | 101 | 15.10 | 6e4efc6a4331a3690a7cced97d6a17a4674fac2b00b22973da496c20bfb3c22e |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T017-T021-green157-replay154-concurrence.json | 1/0/0 | 0 | 8.65 | cfbcf9fecf98f1c87bd50cc0f29dcbdcdb04e0efeb13f7f22a13ce050dfed36d |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T017-T021-green157.json | 28/1/0 | 101 | 38.63 | c9fbfa7c24dd87fffbbab59a1387696cb7c6f9ed1a4915e7dcc0c7f068b18c6e |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T017-red157-T016-T020-green.json | 12/10/0 | 101 | 20.37 | 52dae29e0a0c758d30d2b891921b32d7863f0147f167e12e21c5b642211d9272 |
| /Users/moi/.cache/bridget-plan157-20261012-r1/T022-baseline157-final.json | 184/0/1 | 0 | 130.16 | 1eb0bbb4ba2704664fe1ae1fb4da534b9a2e0f18d5f6e7b39eba0c6c6c4b958b |

## Portée et diagnostics

La référence T001 est 177 PASS / 0 FAIL / 1 ignored. T022 est 184 PASS / 0 FAIL / 1 ignored : les 177 de référence et sept gardes155. Ces sept tests portent sur forward Codex, fournisseur non supporté, guardian_subagent, auto_review, politique enfant, octets canoniques et envelope_mismatch. Les anciens octets sans fallback sont également prouvés par le test transport T003 et la frontière MCP.

Les fondations atteignent 24 PASS daemon et 1 PASS transport dans deux exécutions. Le RED T004/T005 comporte 1 PASS / 10 FAIL : neuf échecs métier de mémoire/préflight et un montage de fixture de définition épinglée (InvalidOrder). Ce dernier n’est pas présenté comme un RED métier. Son montage a été corrigé avec prepare_recovery existant, sans nouveau scénario.

US1 : le RED T007/T010 est 0 PASS / 8 FAIL. La passe suivante est 167 PASS / 2 FAIL ; les causes et les 60 cellules de matrice157 passent. Deux anciennes fixtures148 échouaient. Le modèle demandé a été épinglé par le helper existant avant préflight. L’oracle SameProject sans racine est corrigé vers catalogue faux/cwd:null, sans retirer l’admission dans un projet concret. La reprise ciblée est 15 PASS / 0 FAIL. La source Claude illisible vers Codex appelle claude_inputs_without_rules/read_json_source ; settings_revision_changed de recheck_context reste distinct et non remplaçable.

Quotas : T013 commence à 0 PASS / 6 FAIL puis passe dans l’exécution T012/T015 à 19 PASS / 0 FAIL. Les six scénarios couvrent catalogue épuisé, refus avec retour et clé libre, péremption, null inconnu, survie à la disparition de la présence et vrai handler sous le nom du registre pour les trois sources. Les horodatages restent synthétiques, aucun fournisseur réel n’est appelé.

MCP : l’exécution mixte T017-red/T016-T020-green conserve son RC101 et ses 12 PASS / 10 FAIL. Ses huit tests MCP passent ; les 14 tests fallback sont alors 4 PASS / 10 FAIL. Après implémentation, les 14 tests fallback passent dans T017/T021 à 28 PASS / 1 FAIL. Les invalidations Reserve/Commit sont incluses dans le test prévu native157_fallback_premier_puis_second_candidat_valide. Son RED initial échoue avant d’atteindre ces sous-cas ; seules les assertions GREEN prouvent cette stabilité.

Le seul échec de cette passe est l’ancien native154_creation_quarante_appels_trois_proprietaires, sur lecture Git common-dir pendant créations concurrentes. Le même test rejoué seul passe 1/0 en 8,65 s, avec les sources inchangées. Aucun retry de production n’a été ajouté pour masquer cet échec.

## Mesure du catalogue et R8

Le banc ignoré par défaut mesure 50 appels sur 10 fournisseurs synthétiques. Le premier montage attendait 20 entrées et échouait ; ce résultat n’est pas un RED métier. Après correction de cette seule attente, la référence est médiane 40,201 ms et p95 42,698 ms. La première mesure après verdict est 183,236/206,447 ms. Son test finit RC0 mais la performance dépasse le seuil SC007 de 2 ; elle n’est pas déclarée conforme.

R8 réemploie les corps du verdict et de la politique. Il prépare les lectures liées aux mêmes entrées une fois, puis dérive les trois postures. La mesure finale est 23,623/25,244 ms, ratios 0,588/0,591 par rapport à la référence. SC007 est acquis sur ce banc privé. Cette mesure ne prouve aucune latence d’authentification ni de fournisseur réel. Reçu détaillé : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/specs/157-catalogue-quotas/validation/mesure-catalogue.md.

## Suite complète unique T025 — terminée

La suite complète unique T025 est terminée : 2396 PASS Cargo, 9 FAIL, 74 ignored, 95 cibles, RC101 en 747,24 s. Trois PASS de processus enfants sont exclus du total Cargo. Les huit échecs anciens concordent avec FULL156 ; le neuvième était un oracle154 désormais aligné sur le contrat157. Sa reprise exacte passe 1/0 en 14,28 s, RC0. Les 44 tests native157 exécutés passent dans FULL ; le banc de latence reste ignored et est prouvé séparément. Il n’y a ni deuxième FULL ni GREEN global.

Reçu : /Users/moi/.cache/bridget-plan157-20261012-r1/T025-full157-once.json, SHA256 : 442d5b063c06fd01697bf32dfc5631356ebb053088c02b06ed2f0bedc1fa398c. Log : /Users/moi/.cache/bridget-plan157-20261012-r1/T025-full157-once.log, SHA256 : 0a85192a923d137ecc0471306658d170980bb4717c826e23c3cdda92f676191e. L’exécution commence le 2026-10-12 à 07:31:21,425 UTC selon le reçu, plutôt que l’heure approximative 07:32 annoncée pendant son lancement. Ses 217 sources sont strictement stables avant/après.

Méthode de comptage : chaque marqueur Cargo Running ou Doc-tests ouvre une cible ; seule sa dernière ligne test result compte. La somme brute du runner est 2399/9/74. Une ligne enfant 1 PASS apparaît dans daemon lib, ligne 460 ; deux lignes enfant 1 PASS apparaissent dans transport lib, lignes 3113 et 3117. Ces trois résultats restent des exécutions embarquées, exclues du total 2396/9/74 des 95 cibles Cargo. Les cibles échouées sont daemon lib (1669/1/15), core_089_contract_test (1/2/1), core_089_crash_test (11/1/2), core_089_identity_test (1/3/1), core_089_reply_test (0/1/1) et t3code_098_test (2/1/1).

### Huit échecs historiques comparés à FULL156

La liste157 hors oracle154 est exactement celle des huit modes hérités de156. Les sites d’assertion sont identiques. Sept payloads d’erreur sont identiques ; T3 conserve la même première assertion «1 agent attendu, aucun vu» et le même refus HTTP404, avec ses dates/ports/session propres. Les causes sous-jacentes ne sont pas nouvellement diagnostiquées ni corrigées en157.

| Test | Site absolu dans le worktree157 | Symptôme inchangé |
|---|---|---|
| cli_uuid_herite_adresse_active_sans_pouvoir_usurper_ni_suivre_une_reponse_ephemere | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_contract_test.rs:400 | Identité --from non attestée |
| client_reference_cli_mcp_rejouent_le_meme_record_et_refusent_les_divergences_sans_mutation | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_contract_test.rs:180 | Identité auxiliaire indisponible |
| binaire_et_outil_mcp_partagent_les_quatre_issues_d_une_reponse_liee | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_crash_test.rs:634 | Identité auxiliaire indisponible |
| noms_humains_et_crash_ne_changent_ni_scope_ni_canon_ni_instance_du_wrapper | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| deux_processus_mcp_meme_identite_et_meme_cle_restent_isoles_par_instance | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| spec094_mcp_modifie_uniquement_sa_propre_presence | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_identity_test.rs:32 | auxiliary_credential_required |
| cli_demande_mcp_repond_annulation_et_timeout_ne_rouvrent_ni_ne_dupliquent | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/core_089_reply_test.rs:48 | Identité auxiliaire indisponible |
| spec098_pont_expose_remet_repond_renouvelle_et_retire | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/tests/t3code_098_test.rs:159 | Aucun agent T3 observé, snapshot HTTP404 |

Référence FULL156 : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-full156-once.json, SHA256 : 888957018e258ed5a5d83113e791e23f485f844f47fcf33ee37f9fc24c0b08f5. Log /Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-full156-once.log, SHA256 : fdb7deaff78eb9fa82a7af2879f59c3a0dd5858300450e8e198d80bd891e0274. Classification antérieure : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/156-guidage-file/validation/T029-full156-once-comparison.json, SHA256 : 218af3b449aab1af086d29ebf1b5168a12adb1b6a3a1832e96823026db03f27c.

### Oracle154 mis au contrat157 et reprise exacte

Le principal a seulement ajouté substitution:null au JSON attendu du test native154_partage_discovery_termine_et_statut_compatibles150 et adapté commentaire/assertion de version. La projection complète et ses comparaisons d’octets restent conservées ; aucun scénario nouveau. La plage 411–431 de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon/native_workspace154_tests.rs a été relue. SHA final 015db2e65365e27c9bb7aa6bc6027252fffbe7e0211b0576f8baeac9c73fb587.

Reprise exacte : /Users/moi/.cache/bridget-plan157-20261012-r1/T025-replay154-compat157.json, SHA256 : 74fd117b4e4c09da91b34fdb2935b93a3e45b7efa931b2e0103356639552ee52 ; 1 PASS / 0 FAIL / 0 ignored, RC0 en 14,28 s. Log /Users/moi/.cache/bridget-plan157-20261012-r1/T025-replay154-compat157.log, SHA256 : 8775c1d2ad3c3a4f18cfb9e3f7d7c99d7092576c2bfacc8b9b1136fd136fe121. La commande utilise --exact ; le runner borne les threads à 1 dans son environnement, sans argument --test-threads dans cette commande. Les 217 sources sont stables pendant cette reprise. La comparaison FULL/reprise montre uniquement le fichier de tests154 changé, aucune production modifiée.

Le résultat historique de FULL reste 2396 PASS / 9 FAIL / 74 ignored, RC101. La reprise solde son seul nouvel échec de fixture ; elle n’efface pas les huit anciens échecs et ne crée pas une seconde suite complète. Aucun modèle réel, login, build, installation ou recette157 n’est crédité. Les checkpoints 24/26 et FULL en cours du premier gel sont désormais supersédés par ce reçu, sous réserve de l’approbation finale T026 par le principal.
