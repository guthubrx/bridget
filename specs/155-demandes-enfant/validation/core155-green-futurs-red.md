# Core155 — PASS partiels et RED futurs, exécution interrompue

Ce reçu ne décrit ni une suite complète ni un GREEN global. Aucune tâche n’est
cochée par son rédacteur. Les noms bruts « green-futurs-red » restent inchangés.

## Exécutions distinctes

| Run | Résultat | Limite |
|---|---|---|
| R1 | RC101, 9,82 s ; 214 sources stables | Compilation refusée : import de la macro debug absent ; aucun test exécuté |
| R2 | RC101, 68,8 s ; 214 sources stables ; 37 tests terminés sur 38 | Root a envoyé SIGTERM au PID22801 ; dernier test bloqué, aucune synthèse finale Rust |

R2 a terminé 18 PASS et 19 FAIL, 0 ignored ; un test reste bloqué. Le champ
`test_results` du reçu brut est vide parce qu’aucune ligne finale test-result
n’existe. Les résultats ci-dessous sont extraits des 37 lignes terminales
`test NOM ... ok|FAILED`, chaque nom compté une fois. Le test bloqué n’est
compté ni PASS ni FAIL.

| Groupe | PASS | FAIL | Bloqué |
|---|---:|---:|---:|
| Core | 14 | 1 | 1 |
| Respond | 1 | 13 | 0 |
| Human | 2 | 3 | 0 |
| Chain | 0 | 1 | 0 |
| Hostile | 1 | 0 | 0 |
| Wait | 0 | 1 | 0 |

## Diagnostic et portée

Deux échecs relèvent du montage des fixtures : la reconnexion utilise un
agent_type incorrect ; l’identité étrangère n’a pas de writer de connexion.
Leurs assertions de montage échouent avant les oracles métier. Les 17 autres
échecs terminés sont des assertions métier des futures tâches T024/T026/T027,
T028,T030,T034. Ils restent RED ; le PASS isolé Respond après échéance ne
valide pas toute la réponse parent.

Le test native155_request_budgets_ouverte_et_totale ne termine pas. Le sample
montre RequestFixture155::receive → receive → execute_controls → flush → sendto.
La boucle de fixture publie 32 avis sans lecteur parent qui les draine. Root a
terminé le seul PID22801 par SIGTERM. Ce blocage de fixture ne prouve ni un
succès ni un échec de la règle de budget.

Les PASS lifecycle incluent expiration, invalidation avant renvoi, accusés et
course de fermeture prévus en T023. Le PASS Hostile atteste les faits séparés
du texte déjà couverts en T018. Cette portée reste celle de chaque test listé,
sans déclarer toutes les tâches terminées ni étendre la validation.

## Noms extraits du log R2

Le préfixe commun est
`daemon::native_delegation::native_delegation_permissions149_tests::native_child_requests155_tests`.
Le JSON dérivé conserve chaque nom intégral. Les noms suivants sont exacts,
sans le seul préfixe commun.

| Nom | Résultat | Groupe |
|---|---|---|
| `chain155::native155_chain_trois_niveaux_snapshot_saut_unique_et_silence` | FAIL | Chain |
| `chain155::native155_hostile_texte_cite_faits_separes_et_aucune_autorite` | PASS | Hostile |
| `human155::native155_human_250_decisions_obsoletes_ne_bloquent_pas_la_nouvelle` | FAIL | Human |
| `human155::native155_human_decisions_obsoletes_expiration_annulation_purge` | PASS | Human |
| `human155::native155_human_faits_options_decision_attestee_acquittee` | FAIL | Human |
| `human155::native155_human_item_ouvert_ferme_a_expiration` | PASS | Human |
| `human155::native155_human_ne_peut_pas_repondre_a_une_question` | FAIL | Human |
| `native155_human_depot_idempotent_apres_panne` | PASS | Core |
| `native155_human_item_deja_resolu_reutilise` | PASS | Core |
| `native155_lifecycle_accord_apres_echeance_jamais_envoye` | PASS | Core |
| `native155_lifecycle_accord_pending_annulation_puis_reconnexion_void` | FAIL | Core |
| `native155_lifecycle_accuse_negatif_annule` | PASS | Core |
| `native155_lifecycle_annulation_course_closed_avant_ou_apres_application` | PASS | Core |
| `native155_lifecycle_enfant_disparu_accord_sent_reste_unknown` | PASS | Core |
| `native155_lifecycle_expiration_refus_et_cloture_none_pending_void` | PASS | Core |
| `native155_lifecycle_invalidations_avant_renvoi_et_finalisation` | PASS | Core |
| `native155_lifecycle_plantage_entre_sent_et_envoi` | PASS | Core |
| `native155_lifecycle_renvoi_jusqu_a_accuse` | PASS | Core |
| `native155_request_avis_idempotent` | PASS | Core |
| `native155_request_echeances_fenetres_et_decision_tardive` | PASS | Core |
| `native155_request_tuple_idempotence_conflit_tour_et_identite` | PASS | Core |
| `native155_request_verdicts_et_forbidden_sans_inbox` | PASS | Core |
| `respond155::native155_respond_accept_question_nested_answer` | FAIL | Respond |
| `respond155::native155_respond_answer_invalide` | FAIL | Respond |
| `respond155::native155_respond_approbation_route_humaine_refusee` | FAIL | Respond |
| `respond155::native155_respond_apres_echeance` | PASS | Respond |
| `respond155::native155_respond_apres_levee_reussit` | FAIL | Respond |
| `respond155::native155_respond_decision_durable_et_non_proposee` | FAIL | Respond |
| `respond155::native155_respond_decline_question` | FAIL | Respond |
| `respond155::native155_respond_identite_proprietaire_et_ownself_refusees` | FAIL | Respond |
| `respond155::native155_respond_mcp_champs_inconnus` | FAIL | Respond |
| `respond155::native155_respond_parent_revoque` | FAIL | Respond |
| `respond155::native155_respond_pause_refuse` | FAIL | Respond |
| `respond155::native155_respond_rejeu_identique_et_autre_decision_close` | FAIL | Respond |
| `respond155::native155_respond_statut_cles_existantes_inchangees` | FAIL | Respond |
| `respond155::native155_respond_statut_liste_ouvertes` | FAIL | Respond |
| `wait155::native155_attente_reveillee_par_une_demande` | FAIL | Wait |
| `native155_request_budgets_ouverte_et_totale` | BLOCKED | Core |

## Preuves et empreintes

- Reçu R1 : /Users/moi/.cache/bridget-delivery155.um4xccv4/core155-green-futurs-red-r1.json.
- Log R1 : /Users/moi/.cache/bridget-delivery155.um4xccv4/core155-green-futurs-red-r1.log. SHA256 `2eae6e8a4be427f20476ad852167c109538e07dd100de6483a955eff50fb8c22`.
- Reçu R2 : /Users/moi/.cache/bridget-delivery155.um4xccv4/core155-green-futurs-red-r2.json.
- Log R2 : /Users/moi/.cache/bridget-delivery155.um4xccv4/core155-green-futurs-red-r2.log. SHA256 `4e695a294847c142d04eae22b474f391359b8e41cfbed7d5c6158950555cf094`.
- Sample : /Users/moi/.cache/bridget-delivery155.um4xccv4/core155-budget-block.sample.txt. SHA256 `0dbf5c5d48e79fc0e60cdba4bf7571fa72a8e9f48da070ccd4221822ceb6fd75`.
- Gate dérivé : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/core155-green-futurs-red-gate.json.

Les 214 empreintes avant/après figurent dans chaque reçu source et sont égales.
Le gate dérivé recopie les empreintes ciblées du runR2, pas celles des sources
qui pourraient avoir changé après ce run. Aucun hash actuel n’est substitué.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs : `64125a08bc4cab0f0d146731cfbe7e082051324960304073eb5949406b985be5`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests155_tests.rs : `c77d467d6cde83d07dc3d3f2ea51906dbdd273191d4887e8f17c3c98aee78894`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation_respond155_tests.rs : `e6114a6e95ea2082288e4267cbe80ef98bf544aafadfc95a4b40592faffe7927`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests_human155_tests.rs : `5fcd0ec2eb67be6fb44f20a1bca88f53599b79a0eb9d67f59b61201945478a02`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation_chain155_tests.rs : `6cf8fec6677f895b7d3f4d9068c4e73cdfac15efcfbcda79e7d3a46de0c8a615`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation_wait155_tests.rs : `90a725b91fcf79c03bbc9b5e5f29564152c09210b6ba9982bf602d7c004e6983`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs : `ef258f2304ec850b8a97f77d5fa4370ae314f637f733e0902bf88b80ffac3555`.
