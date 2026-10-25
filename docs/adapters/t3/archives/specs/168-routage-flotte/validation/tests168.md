# Archives techniques de l’adaptateur T3

Document d’origine : [specs/168-routage-flotte/validation/tests168.md](../../../../../../../specs/168-routage-flotte/validation/tests168.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Diffcheck final : RC0,0,043s.
- rustfmt initial : RC1,6,862s. Les lib.rs entraînent un parcours récursif horsdiff, notamment t3code_mcp.rs hérité. Aucun formatage massif.
- Contrôle correctement limité aux24fichiers Rust modifiés : --config skip_children=true, RC0,2,174s, sortie vide. Aucun GREEN de format global.
- ROOT rapporte aussi le parse des JSON modifiés PASS. Le JSON statique lu contient seulement rustfmt/diffcheck : cette attestation de parse est distinguée d’une trace dédiée et ne change aucun compteur.

<a id="extrait-2"></a>

## Extrait 2

| Tâche | État documentaire | Preuves | Portée et limite |
| --- | --- | --- | --- |
| T002 | cochée | TP/TR, MCP, W128/Wepoch/W163 | Types fermés, négociation168, wire163 et Lineagev4 ; publication B et réemploi confirmés. |
| T003 | cochée | ST | DDL neuf, migration BLOB dans cinq états, index/FK/trigger, faute de copie et rollback. |
| T004 | cochée | ST, Nservice/Nrollback | Reçu versionné et CAS niveau/époque/révision ; décision et Chosen atomiques, lien Task conservé. |
| T005 | cochée | REG, CLI, E2E | Matrice12/128 entière ; limite129 y compris désactivée ; axes et modèles distincts. |
| T006 | cochée | REG et reçu A | Provenance/défaut effort résoluble ; catalogue pur sans découverte/auth. Droit fournisseur demeure inconnu. |
| T007 | cochée | Ncatalogue, E2E, CLI | MêmeUID, rôle Client et capacité dédiée ; lecture réelle sans agent/T3, aucun effet métier. |
| T008 | cochée | CLI, E2E | Lecture humaine --catalogue --json, filtre machine et refus d’ancien maître. |
| T009 | cochée | REG, Ncatalogue, CLI, E2E | Catalogue12/128, registre129, données publiques et zéroTask/reçu/fournisseur dans namespace isolé. |
| T010 | cochée | MCP, CLI, Nunique | FaçadeV2 stable, axes/workload fermés ; wireV1 explicite et refus d’ancien maître. |
| T011 | cochée | Naxes, REG | Axes indépendants, effort imposé fermé, défaut natif null et contraintes contradictoires. |
| T012 | cochée | Naxes/Nunique/Ninaccessible, MCP, CLI | Modèle seul/machine seule, contraintes, rejeu avant politique fraîche ; refus version et enveloppe sans bascule. |
| T013 | cochée | HR, W128, reçu B/C | Collecteur natif borné, premier CPU inconnu, un relevé du lot128 ; réemploi du tick publié sans nouveau daemon. |
| T014 | cochée | Q, Nscore et reçu C | Faits quota liés connexion/instance/définition/époque/TTL ; crédits disponibles avec fenêtre100%, aucun pool inventé. |
| T015 | cochée | Nscore, Q, REG | Score borné, ordre stable, inconnus neutres, saturation exclue avant construction de charge ; compilation déclarée. |
| T016 | cochée | W128/Wepoch/W163, Nscore, E2E | Probe128 sans effets, première mesure, révocation fraîche et ancienne époque/capacité séparées. |
| T017 | cochée | Ncharge, DSpost, N12 | Charge entière/contraintes/objectifs et agrégation conservées ; capacité insuffisante explicite, pas de top8. |
| T018 | cochée | TP, qualification CLI distincte | Transport Claude isolé testé par faux exécutable ; flags/env/cwd, StructuredOutput seul, stderr nul, bornes avant parsing. |
| T019 | cochée | DS, Nservice/N12 | Adaptateur Laya168 entier et fermé, legacy163 séparé, défaut effort, capacité/deadline et permis jusqu’au drain. |
| T020 | cochée | Nservice/N12/Nunique/Ninaccessible/Nservicepost | Chaîne native vers HTTP/transport contrôlés, niveauOFF sans essai, CAS et rejet des réponses tardives. |
| T021 | cochée | TP, DS, Nservicepost | Capacités/outillage/sorties contradictoires, stdout/JSON, timeout dès spawn, groupe enfant/descendant et permis réel. |
| T022 | cochée | Nservice/N12/Nunique, Nscore | Parcours contrôlé complet : service valide, liste12 au secours, OFF→score et ID exclu non réintroduit. Pas recette fournisseur réel. |
| T023 | cochée | N12, W128, Q et reçu C | Connexions résolues via lanceurs propres, environnements isolés et recontrôles enfant ; aucun credential transféré. |
| T024 | cochée | W128/WhorsGit, TP | PrepareV2 recontrôle connexion exacte/moteur/digests/droits avant effets et Start après révocation ; v1 conservé. |
| T025 | ouverte | REG, N12 — PARTIEL | Deux références Codex représentées ; deux environnements réellement observés dans N12 sont Claude, pas Codex. Preuve exacte des deux environnements Codex reste ouverte. |
| T026 | cochée | ST, Nservicepost/Nrollback/Nunion | CAS et ancienne réponse, rollback Task/reçu, états techniques conservés et rétention partagée au puits. |
| T027 | cochée | Npending/Nrollback/Nservicepost et reçu C | Pending/Started/Chosen récupérés depuis snapshot ; même choix/repli et aucune nouvelle consultation ; gardes existantes réemployées. |
| T028 | cochée | ST, Nunion/Nrollback et procédure documentée | SQLite persistante, BLOB non textuels/cinq états, CHECK16/128Kio, rollback/reouverture/atomicité et UNION ; retour arrière reste procédure opérateur. |
| T029 | cochée | TP, Lunit, LCLI | Lineagev4 depuis décision figée, v3null168, placement et163 conservés ; CLI sans downgrade silencieux. |
| T030 | cochée | Nservicepost/Npending/Nrollback, ST | Concurrence/collision, réponse tardive, réouverture et crashPending/Started/Chosen/Admitted ; uneTask et choix durable. |
| T031 | cochée | NlocalR4, CLI/MCP et reçu C | SHA source exact pour Shared/Worktree, horsGit général distinct de métadonnées cassées ; aucune base distante implicite. |
| T032 | cochée | W128/WhorsGit, W163 | Base au Prepare, base_ref antérieure àHEAD, SHA manquant/inexact, horsGit et révocation avant fournisseur. |
| T033 | ouverte | NlocalR4, W128/WhorsGit — PARTIEL | SHA/base_ref/horsGit et Git cassé exercés. Aucun scénario I/O/timeout Git168 distinct identifié parmi les57 ; timeout HTTP n’est pas assimilé à cette preuve. |
| T034 | ouverte | NlocalR4, Lunit/LCLI — PARTIEL | JSON garde SHA complets/dirty, résumé abrégé7, branche/path/Kept et source inchangée. Pas de preuve de deux worktrees sur deux machines ni de récupération distante. |
| T035 | ouverte | Tous les groupes et statiques — PARTIEL | 57noms à dernière issuePASS, compilation et format ciblé valides ; dépendances/preuvesT025/T033/T034 encore ouvertes. |

<a id="extrait-3"></a>

## Extrait 3

Le test `t3code_identity::spec101_identity::spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` passe au premier replay instrumenté. Le diagnostic conditionnel ajouté n’est pas atteint. La cause initiale de `process_arguments=None` demeure inconnue ; ce PASS ne prouve pas sa résolution causale.

<a id="extrait-4"></a>

## Extrait 4

- /home/moi/.local/state/bridget-delivery168/tests/t036-full168-1ru8z5rf/result.json — SHA-256 `149bc0a5470df503d706b30cc50ee11bc3aa1fe302785fdc42c091a64dd1eaa0`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/result.json — SHA-256 `17378e035ca07e49080373eaecde8f1e68fddb43eb9f1c9ca66d65ca0c480586`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/result.json — SHA-256 `78a6de7c1f9e0291f901112b1f5b6faedef8fa65e96827ba4cb67627926e1d3a`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/result.json — SHA-256 `2b359ce75ecc546c76dba1e4a3122c467566854df1739c160924d42fea3efe65`.
- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-post-full168.md — SHA-256 `c8240ff0386b00d27214c776028c8237b300f79dc1173b9983336bd3369737bd`.
- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-delta-core-t3.md — SHA-256 `42ea238ebad7c0a9e89c431bdba77f12ba3ce70129a34e3bf334d373a34068e2`.

<a id="extrait-5"></a>

## Extrait 5

- `t3code_identity::spec101_identity::spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-00.log ; SHA-256 `fe4a91071ea652b277c0c32b22f727ec676f648c400be3be1b98a0007c28d646`.
- `projet_refuse_et_session_hote_spawn_send_attach_stop_sans_docker` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-11.log ; SHA-256 `22c73226c3c908eca9f98bf02a8cf52643c95da4559660d10fb7ac4af87178a2`.
- `spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-12.log ; SHA-256 `0fd23a5f2e6400a9b726f28e2a119b1006961313e65e1cfad9718ccd693d70bf`.
- `daemon_started_at_256_raises_soft_to_4096_and_holds_over_256_descriptors` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-13.log ; SHA-256 `2f6ce457522c1a3366f8c33e3d581d55d5bcd893be716a80e67b612d801c1102`.
- `daemon_with_hard_512_is_clamped_to_hard_and_warns` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-14.log ; SHA-256 `b1300616815ca8bcb4295eddaf95adee7560650a1ca9a19f0f8b26f6f142b283`.
- `cli::hook_tests::spec167_delegate_conserve_les_choix_explicites_sans_defaults_caches` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-15.log ; SHA-256 `c323a849dad6da930b74be5f96a881db4f67c83eb30c5c4fe6e6b803735d56b4`.
- `cli::hook_tests::spec167_delegate_reduit_le_parcours_nominal_a_la_mission` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-16.log ; SHA-256 `a37d95429c047b2f9f9c8e9990063b7cc52f3060d9222b6a731354ed32e10974`.
- `daemon::native_lineage::native_lineage_tests::native149_validate_ferme_les_formes_ouvertes` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-17.log ; SHA-256 `ab52accc5743d6e0c3a4390230976d39e9322733d9c714e9cf2cf89e4fddf06f`.
- `daemon::presence_tests::guidance156_rate_survit_au_redemarrage` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-18.log ; SHA-256 `0a21e98956f53664a828b4f9407543dfd4815be064b844b0f5e2b9d20f6ba847`.
- `mcp::tests::matrice_fr009_couvre_les_quinze_cas` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-20.log ; SHA-256 `b136b2d78c89b50c6cb239de7d5fe3643c83a4803d89a755bd2da8c82e65561e`.
- `mcp::tests::mcp158_plafond_parent_garde_tous_les_outils` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-21.log ; SHA-256 `db0a34de4ea74a5b0ddf85946205468e01931c64c45cdfeb74207d01cef082f7`.
- `native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-22.log ; SHA-256 `db88f59c940693af710d5adb2ad934c1335c67f8cf3c6f6432ec73c9b60b2796`.
