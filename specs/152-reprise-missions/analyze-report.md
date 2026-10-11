# Rapport d'analyse - Session152

Rapport historique des deux rondes de conception. Le gate frais post-150/151
du 2026-10-11 fait autorité dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/analyze152.md`.
Les corrections ci-dessous décrivent les étapes successives ; R22 à R28
remplacent les hypothèses retirées, pas des travaux de code déjà validés.

Date : 2026-10-11. Analyse croisée de `spec.md`, `plan.md`, `research.md`,
`data-model.md`, `contracts/reprise152.md`, `tasks.md`, `reuse-audit.md`.
Lecture seule du code : aucune exécution de test.

## Couverture des exigences

| Exigence | Tâches | Test principal |
|---|---|---|
| FR-001 | T012 | `native152_pause_tick_ne_perime_pas_l_echeance` |
| FR-002 | T013 | `native152_pause_chemin_a_*` |
| FR-003 | T014 | `native152_pause_chemin_b_*` |
| FR-004 | T015 | `native152_pause_reprise_attend_interrupted` |
| FR-005, FR-006 | T014, T015 | `native152_pause_reprise_meme_identifiant_de_mission` |
| FR-007 | T016 | `native152_pause_annulee_pendant_la_pause_*` |
| FR-008 | T019 | `native152_pause_aucun_lancement_ni_remise` |
| FR-009 | T017 | `native152_horloges_echeance_gelee_et_decalee` |
| FR-010 | T018 | `native152_horloges_suivi_survit_a_la_reconnexion` |
| FR-011 | T020 | `native152_lancement_retenu_600s_reussit` |
| FR-012 | T006, T033 | `native152_lancement_saga_existante_inchangee` |
| FR-013, FR-016, FR-017 | T023 | `native152_panne_*` |
| FR-014 | T022, T024 | `native152_panne_enfant_relance_avec_environnement_natif` |
| FR-015 | T025 | `native152_panne_borne_de_deux_reprises` |
| FR-018, FR-019 | T026, T027 | `native152_annulation_membre_non_arretable_n_arrete_rien` |
| FR-020 | T028 | `native152_annulation_reessai_ne_rejoue_que_les_echecs` |
| FR-021, FR-022 | T029, T030 | `native152_refus_*` |
| FR-023 | T031 | `native152_refus_purge_30_jours` |
| FR-024 | T021 | `native152_vue_suspended_additive` |
| FR-025 | T008 | relecture d'un payload 151 |
| FR-026 | T033 | revue de documentation |

Les 29 FR sont couverts par au moins une tâche ; FR-027 et FR-028 par T043, T041, T042, FR-029 par T043 et T048. Les 9 SC ont chacun un test prévu : SC-001 (T015), SC-002
(T019), SC-003 (T017), SC-004 (T020), SC-005 (T025), SC-006 (T027), SC-007 (T030),
SC-008 (T032), SC-009 (T041, T042).

Les objections de la contre-revue 150 sont couvertes : n° 1 (T013), n° 2 (T015),
n° 3 (T014), n° 4 (T012, T017, T018), n° 7 (T020).

## Constats et corrections appliquées

| # | Constat | Gravité | Correction appliquée |
|---|---|---|---|
| 1 | `research.md` R8 citait T015 pour l'inventaire des codes de refus ; la tâche réelle est T029 | moyenne | R8 corrigée vers T029 |
| 2 | Une première idée décalait les horloges à la levée seulement. La contre-revue (objection 4) montre que l'échéance expire avant | haute | Conception changée : gel en pause (T012), décalage à la levée (T017) |
| 3 | Un décalage relatif de `PendingReply` ne survit pas à un rejeu | haute | `frozen_secs` est affecté depuis `suspended_total_secs` (idempotent) |
| 4 | `reconstruct_active_for_agent` accepte `interrupting` : une levée rapide pourrait fermer l'exécution en `unreachable` | haute | Reprise native exigée en `interrupted`, pilotée par `tick` (T015) |
| 5 | Une annulation pendant la pause laisserait une ligne de pause à reprendre | moyenne | `abandon_pause_interruption` et `may_resume` refuse `cancelling` (T016) |
| 6 | Un reçu de refus pour les refus purement syntaxiques n'apporte rien : ils se rejouent déjà à l'identique | basse | Reçus limités aux refus dépendants de l'état du monde (R8) |

## Restants (non bloquants pour la planification)

| # | Restant | Traitement |
|---|---|---|
| R1 | H4 : l'instance du bail de reprise garde-t-elle le même `instance_id` que le bootstrap natif ? La flotte accepte-t-elle un lien dont le parent est une instance disparue (mission imbriquée) ? Non prouvé par lecture | test rouge T004 (a à d) ; `native_child_env` et `ownership_from_desired` s'adaptent (T024) |
| R2 | Les codes de refus de registre ne sont pas tous inventoriés | T029 avant l'écriture des reçus |
| R3 | Une saga de lancement en cours au début d'une pause de plus de 300 s peut expirer | limite documentée (FR-012) ; test T006 |
| R4 | Historique clos : la spec et le code 151 sont désormais livrés et relus | H1 réattestée ; RequestWait pris en compte dans T018, budget de l'appelant inchangé |
| R5 | Mission imbriquée : le propriétaire (un enfant) peut être relancé avec une autre instance. `recover_owner` existe (`N:172-190`) mais le cas n'est pas testé avec reprise | ajouter un cas imbriqué à T025 si T004 le révèle |
| R6 | La vue Lineage T3 montre `working` pendant la pause | question utilisateur 3, défaut non |
| R7 | Plusieurs sessions modifient `native_delegation.rs` (151, 153) | exécution séquentielle |

## Verdict

Planification révisée après la contre-revue 152 (voir `adversarial-review-codex.md`). 50 tâches. La couverture annoncée ne vaut que pour les scénarios rouges ajoutés (T038 à T040) et les tests de conservation de T023. Prêt pour l'implémentation après réponse (ou défaut) aux
trois questions de la spec. L'implémentation commence par les tests rouges T003 à
T007, et T004 tranche R1 avant T024.

## Corrections de la contre-revue 152

| # | Constat | Gravité | Correction appliquée |
|---|---|---|---|
| 7 | Reprise d'un enfant connecté sans filiation (`daemon.rs:3995-4020`) | critique | R13, T024 en deux chemins, T043, T004 étendu |
| 8 | Compteur de reprises non atomique | haute | R14 : compte dérivé, `resume_attempts` retiré de la tâche (T010, T025) |
| 9 | Panne pendant pause : échéance brute, `interrupting` sans terminal | haute | R15, T041, T042, T038 |
| 10 | Chemin A : le gestionnaire général notifie et retire le suivi | haute | R16, T044, T002 et T003 étendus |
| 11 | `latest_in_chain` trié par date et identifiant aléatoire ; borne de chaîne fausse | haute | R17 : génération, complexité corrigée (T010) |
| 12 | Gel du suivi cumule des pauses antérieures à la demande ; échéance du suivi calée sur `created_at` | haute | R19, `request_frozen_secs`, `mission_issued_at`, T017, T018, T039, T045 |
| 13 | Descendants marqués `cancelling` avant le contrôle puis annulés comme racines | haute | R20, T027, T046, T040 |
| 14 | Liste des reçus de refus indéterminée | moyenne | R18 : cinq codes, texte dynamique ramené à `agent_type_unknown` (T029) |
| 15 | Couverture trop forte, conservation non testée | moyenne | T023 étendu (trois tests de conservation), scénarios rouges T038 à T040 |

## Ronde 2 de la contre-revue 152

| # | Constat | Gravité | Correction appliquée |
|---|---|---|---|
| 16 | Borne de reprises inopérante en `starting` et `mission_pending` (aucune exécution) | haute | R22 : `restart_resumes` incrémenté à la décision de reprise (T023, T025, rouge T047) |
| 17 | Identité au redémarrage : `Err` fatal, trois écritures séparées | critique | R24 : commande dérivée du lien attesté, réconciliation, échec nommé non fatal (T043, rouge T048) |
| 18 | Pause enregistrée sans `suspended_at` si crash avant le tick | haute | R25 : temps de pause lu dans `control_events` (T050, T017, T041, rouge T049) |
| 19 | `working` ne prouve pas le début du suivi | haute | R23 : début lu dans la ligne durable de la demande (T018, rouge T049) |

Hypothèses retirées : `suspended_at`, `request_frozen_secs`, `fleet_command_id`,
`crash_resume_count`. Audit frais R27 : l'API `agent_link_by_id` existe déjà,
les événements `pause_on`/`pause_off` sont présents. T050 prévoit l'index temporel
et documente O(log N + P), au lieu de supposer le journal toujours court.
