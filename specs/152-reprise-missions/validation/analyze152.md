# Analyse croisée fraîche — session152

Date : 2026-10-11. Sélection explicite : SPECIFY_FEATURE=152-reprise-missions.
Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions.
Base de lecture : cd2bb015755fde7af26cf87fdb8f4e658172e237.

**Verdict : PASS pour démarrer l’implémentation par les tests rouges.**
Aucun blocage documentaire actif. Les preuves fonctionnelles 152 restent à
obtenir dans les tâches existantes, avant leur code. Ce verdict ne déclare pas
la session implémentée et ne valide aucun runtime de production.

## Prérequis officiels et fallback

Commande tentée une fois dans le worktree explicite :

```sh
env SPECIFY_FEATURE=152-reprise-missions .specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
```

Résultat observé : RC127, `env: .specify/scripts/bash/check-prerequisites.sh: No such file or directory`.
Chemin absolu manquant : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/.specify/scripts/bash/check-prerequisites.sh.
La recherche `rg --files --hidden --no-ignore` de check-prerequisites/common
dans /Users/moi/.speckit/scripts, /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify
et /Users/moi/Documents/MAICompany/runtimes/speckit-portfolio-679aabe2-r14
ne trouve aucune copie stock portable. Aucun outil installé, helper créé ou hook
contourné. Analyse manuelle équivalente : chemins explicites et présence réelle
de spec/plan/tasks, contrôle des exigences, matrice de couverture et constitution.

Le fichier /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/.specify/extensions.yml est absent.
Aucun hook before/after déclaré à exécuter. La synchronisation du pont152 est
signalée RC0 par le principal ; constitution et standards locaux puis globaux
ont été chargés. Les rapports historiques restent signalés comme tels.

## Constats traités

| ID | Catégorie | Gravité initiale | Emplacement final | Constat et correction |
|---|---|---|---|
| A01 | Incohérence | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/spec.md:24 | Ancienne branche/WT150 et livraison sans commit remplacés par session152 puis livraison/nettoyage par session jusqu’à160. |
| A02 | Format | LOW | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/spec.md:8 | Statut et compteurs canoniques : Ready, 1/50, tests 152 non exécutés0/0. Une seule case cochée : T001. |
| A03 | Incohérence | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:44 | D9 et D16 alignées sur R25/R26 : première soumission et échéance calculée, aucun settle_suspension. |
| A04 | Existant | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:68 | ownership_from_desired privée : visibilité crate prévue. agent_link_by_id déjà présent : réemploi sans SQL nouveau. |
| A05 | Hypothèse périmée | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md:80 | Préserver align_request_deadline150 ; R4 ne prouve plus une table jamais modifiée. |
| A06 | Sécurité | HIGH | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:69 | Révocations/sources/cwd revérifiés avant les deux reprises ; extraction env seule insuffisante, cas T004/T024 existants. |
| A07 | Démarrage | HIGH | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:75 | Second contrôle d’identité du marqueur traité pour les identités natives lisibles ; fichiers corrompus/illisibles hors FR029, refus sûr conservé. |
| A08 | Couverture | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:24 | RequestWait calcule l’échéance métier native effective, sans mutation ; budget appelant inchangé. Cas horloges existants. |
| A09 | Contrat | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:58 | Admission imbriquée sur délai effectif ; date brute exposée documentée. |
| A10 | Pureté | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:63 | suspended dans current_task_view ; task_view reste pur et Lineage inchangé. |
| A11 | Erreur terminale | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:82 | Cleanup d’annulation ne remplace pas un terminal acquis ; cas d’annulation existants. |
| A12 | Performance | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:40 | Index temporel dans le schéma existant ; O(log N + P) honnête. Coût réel du classement demandé dans T026/T034. |
| A13 | Documentation | LOW | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:99 | ADR22 décisions, checklist29FR/9SC, quickstartWT152 et booléen suspended, erreur identity_mismatch dans le contrat. |
| A14 | Duplication | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:37 | Nouvel effet nommé NativeMissionDispatch conservé, avec bras de garde NativeDelegation partagé. |
| A15 | Lancement | MEDIUM | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:69 | Bail expiré : échec nommé compatible FR012 ; cas indispensables intégrés à T004/T024 existants. |

Les corrections ci-dessus sont documentaires. Elles ne constituent ni un
RED observé ni un GREEN152. Aucun ID de tâche, test autonome, dépendance ou
module de confort n’a été ajouté. Les trois valeurs par défaut utilisateur sont conservés :
échéance hors pause continue, borne 2, Lineage garde working.

## Couverture des 29 exigences fonctionnelles

Toutes les lignes ci-dessous décrivent des preuves **prévues**. La colonne
source indique le code existant à réutiliser ou à corriger, pas une réalisation152.

| FR | Tâches et premier repère | Oracle prévu | Code actuel concerné |
|---|---|---|---|
| FR-001 | T012, T050 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:47 | native152_pause_tick_ne_perime_pas_l_echeance | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1243 |
| FR-002 | T003, T013, T044 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:22 | native152_pause_chemin_a_suivi_conserve_sans_avis | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1099 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:18452 |
| FR-003 | T003, T014 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:22 | native152_pause_chemin_b_interrupted_ne_fait_pas_echouer | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1482 |
| FR-004 | T015, T042 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:50 | native152_pause_reprise_attend_interrupted | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4540 |
| FR-005 | T010, T014, T015 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:38 | native152_pause_reprise_meme_identifiant_de_mission | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1298 |
| FR-006 | T015, T043 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:50 | native152_pause_chaine_suivie_apres_reprise | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4540 |
| FR-007 | T016, T027 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:52 | native152_pause_annulee_pendant_la_pause_n_est_pas_ressuscitee | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1488 |
| FR-008 | T009, T019 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:37 | native152_pause_aucun_lancement_ni_remise | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:107 |
| FR-009 | T005, T017, T050 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:24 | native152_horloges_echeance_gelee_et_decalee | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1287 |
| FR-010 | T011, T018, T039, T045, T049 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:39 | cas horloges prévus, avec RequestWait natif et budget inchangé | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:289 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:7184 |
| FR-011 | T006, T020 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:25 | native152_lancement_retenu_600s_reussit | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:823 |
| FR-012 | T006, T024, T033 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:25 | native152_lancement_saga_existante_inchangee ; bail expiré dans cas T004/T024 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4204 |
| FR-013 | T004, T023, T025 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:23 | native152_panne_mission_working_n_echoue_pas | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 |
| FR-014 | T004, T022, T024 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:23 | native152_panne_enfant_relance_avec_environnement_natif | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4174 |
| FR-015 | T023, T025, T047 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:27 | native152_panne_trois_crashes_avant_l_envoi | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 |
| FR-016 | T023, T024 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:27 | native152_panne_echeance_passee_echoue ; identité KO ; borne | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 |
| FR-017 | T023 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:27 | cas conservation waiting_for_children/result_available/terminaux/queued | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 |
| FR-018 | T007, T026, T027 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:26 | native152_annulation_membre_non_arretable_n_arrete_rien | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9561 |
| FR-019 | T027, T040, T046 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:82 | native152_annulation_aucun_descendant_marque_si_refus | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1672 |
| FR-020 | T028 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:84 | native152_annulation_reessai_ne_rejoue_que_les_echecs | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1672 |
| FR-021 | T029, T030 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:90 | native152_refus_rejeu_identique ; autre enveloppe | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:660 |
| FR-022 | T029, T030 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:90 | native152_refus_texte_dynamique_jamais_stocke ; transitoires | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:660 |
| FR-023 | T031 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:92 | native152_refus_purge_30_jours | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1909 |
| FR-024 | T021 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:63 | native152_vue_suspended_additive ; octets Lineage | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:126 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:73 |
| FR-025 | T008 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:36 | payload151 sans champs152, défauts et visible_changed | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation.rs:12 |
| FR-026 | T033 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:97 | revue docs/skill/commandes des erreurs nouvelles | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/docs/delegation-native.md:1 |
| FR-027 | T004, T024, T043, T048 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:23 | native152_filiation_controles_vrais_apres_reprise ; deux redémarrages et imbriquée | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/idempotency/agent_links.rs:96 |
| FR-028 | T038, T041, T042, T049 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:27 | panne sous pause active/levée et avant premier tick | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/execution_store.rs:1488 |
| FR-029 | T043, T048 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:75 | native152_filiation_incoherence_n_empeche_pas_le_demarrage, identité native lisible | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233 |

## Couverture des 9 critères de succès

| SC | Tâches et repère | Observation prévue |
|---|---|---|
| SC-001 | T002, T003, T013, T014, T015, T044 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:21 | trois séquences, même mission et résultat unique |
| SC-002 | T019 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:61 | 0 lancement/remise sur300 ticks simulés |
| SC-003 | T017, T018, T039, T049, T050 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:58 | pause300s, délai120s, écart≤2s, reconnexion/crash |
| SC-004 | T006, T020 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:25 | pause600s, enveloppe de lancement inchangée |
| SC-005 | T004, T024, T025, T043, T047, T048 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:23 | deux reprises, troisième refusée, filiation imbriquée, compte conservateur |
| SC-006 | T007, T027, T028, T040, T046 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:26 | 0 arrêt au refus ; reprise seulement des arrêts absents |
| SC-007 | T029, T030 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:90 | cinq codes définitifs rejoués ; aucun transitoire persisté |
| SC-008 | T001, T032, T036 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:20 | baseline puis non-régression six filtres ; suite complète distincte |
| SC-009 | T038, T041, T042, T049 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:27 | panne pendant pause, active/levée, interrupting réconcilié |

## Couverture des 6 scénarios utilisateur

| US | Tâches et repère | Parcours couvert |
|---|---|---|
| US1 | T002, T003, T012, T013, T014, T015, T016, T044 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:21 | pause et reprise par les deux chemins réels |
| US2 | T005, T006, T017, T018, T019, T020, T021, T039, T045, T049, T050 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:24 | horloges et absence de dispatch |
| US3 | T004, T022, T023, T024, T025, T038, T041, T042, T043, T047, T048 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:23 | reprise après panne, autorisations et identité |
| US4 | T007, T026, T027, T028, T040, T046 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:26 | annulation vérifiée d’abord et cleanup conservateur |
| US5 | T029, T030, T031 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:90 | refus rejoués et entretien |
| US6 | T008, T021, T032, T033, T036 — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md:36 | payloads/vues/CLI-MCP/Lineage compatibles |

Tâches transversales justifiées : T034 pour complexité/minimalisme, T035 pour
ADR (Article VII), T037 pour revue finale et preuve du diff. T001 prépare la
référence ; T002 prépare les événements réels. Aucune tâche orpheline.

## Ordre, checklists et gates

- 50 identifiants de tâches uniques ; aucune tâche supprimée ni ajoutée.
- T001 seule cochée après sa preuve publiée ; T002 est la première restante.
- T002 précède T003. Les cas rougesT003–T007/T038–T040/T047–T049 précèdent leur code.
- T050 précède T012/T017/T018/T041. T009 et T050 ont le même propriétaire.
- T014 précède T015 ; T013 précède T044 ; T043 précède T023/T024.
- Les frontières de marqueur lisible sont prouvées dans T048 avant T043.
- Les phases qui modifient le module natif restent séquentielles.
- Les cinq cases du gate de réutilisation sont closes ; checklist complète relue.

Les questions de la spec conservent leurs trois valeurs par défaut déjà prévues.
H1/H2 sont réattestées par la présence des socles 150/151. H4 (instance et
filiation sur les deux reprises) reste une preuve d’implémentation T004,
pas une question documentaire bloquante. L’illisibilité/corruption générale des
marqueurs n’est pas traitée par 152 ; aucune suppression non sûre n’est prévue.

## Constitution, minimalisme et responsabilité

III : spec/plan/tasks présents avant code. VII : ADR prévue T035.
XV : preuve avant cochage ; commits/livraison/nettoyage par session selon le
contrat autorisé. XVI : worktree152 dédié. XVIII : index temporel, coûts réels
et annotation T034 ; aucun coût de runtime mesuré par cette passe.

XIX : fonctions et magasins existants réutilisés. Aucun nouvel état, module,
outil ou dépendance. Refus : table séparée car un tombstone n’atteste pas un refus.
Les extractions env/arrêt ont deux usages, avec contraintes architecturales
explicites d’identité et de classement avant effet. Le potentiel minimalisme
du plan (~40 lignes) reste une estimation à vérifier sur le diff produit.

Six réponses de l’Article XX :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/152-reprise-missions/validation/analyze152.md#extrait-1).

## Mesures et limites

| Mesure | Valeur réelle |
|---|---:|
| FR avec tâche prévue | 29/29 (100%) |
| SC avec oracle prévu | 9/9 |
| US avec parcours prévu | 6/6 |
| Tâches | 50, dont 1 prouvée |
| Constats bloquants actifs après correction documentaire | 0 |
| Constats traités documentaires | 15 |
| Couverture de code exécutée / mutation | non mesurées |

Baseline du principal :239 PASS/0 FAIL/1 ignoré sur six filtres existants, tousRC0.
Le détail est dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline.md.
Ce total est propre à la référence 152. Il ne remplace ni le W004 historique
(2046 PASS/8 FAIL hérités/72 ignorés, RC101), ni le test T027 de 151 distinct.
La suite complète 152 reste prévue en T036 ; aucun succès global n’est déduit.

Portée de lecture : spec/plan/tasks/research/data-model/contrat/checklist et
audit de réutilisation et rapport d’analyse historique complets ; lecture ciblée du code par un passage
Explore puis vérification des points sensibles. Ce n’est pas une revue de tous
les fichiers ou de toutes les fonctions du dépôt. Aucun test, Cargo, service,
modèle ou Git mutateur exécuté par cette passe.

## Répartition proposée et prochaine action

- Cœur natif : un propriétaire pour native_delegation.rs, delegation.rs et delegation_lineage.rs, y compris leurs tests.
- Contrôle/exécution : un propriétaire pour referent_control.rs (T009/T050) et execution_store.rs (T010), après accord sur les signatures.
- Reprise/arrêts : un propriétaire pour daemon.rs et la visibilité fleet.rs ; coordonner l’ordre T043→T023/T024, et préserver les contrôles de sources.
- Documents : un propriétaire pour les artefacts152, la skill/docs et l’ADR.

Ces rôles ne s’exécutent en parallèle que pour des fichiers distincts et des
tâches réellement indépendantes. Aucun nouvel agent n’est lancé par ce rapport.
Prochaine action : T002 puis les cas RED requis sur la production inchangée.
Les gates de conception sont clos ; la production reste soumise à l’observation
des RED et aux tests prévus avant cochage.

## Empreintes des entrées finales

Les hashes suivants figent les documents pris en compte par cette analyse.
Le rapport est lui-même exclu pour éviter une empreinte récursive.

| Entrée | SHA-256 |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/spec.md | 985ac776862c6163359414069ee9d69a252ec31ed264a621c5f648301c68b588 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/plan.md | b07dd8c3caf5e7ae366fa1a7b2dbd00cc088805fba3d5073cf4664341a9eed61 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/tasks.md | a066b0db02b4166b694680c4c11c4332d249db312c8826c9275308de502fd479 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/research.md | 3d8b0c9efa8c09a177af67b9c5d30cdfd51ecdfc83392d25df7861185cbf11c1 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/data-model.md | a18807315aa26a408eacd71dcf5b3163fd095b0de054d719ab5a9067bb7d8725 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/contracts/reprise152.md | 932e549c67a621c0d7a28dec21d44bac03efefafca0cd5f33f75b29f4da746c6 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/checklists/requirements.md | 274d3bd203db4d410dad779ef613996cb484f832aea6ccf2d316f51014209746 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/reuse-audit.md | 8d2cc936542f1083ed000373b09c5b1603812ebd2de3701584af4bdd3f1691ed |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/adversarial-review-claude.md | 10f9080845103d800ad52422405eab6e95451e140c8fe71abddef0951d01e1bb |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline.md | f8cc13e9305e3b1067acb4a24bd4219d4a916319356a051ab4560f5ce338ea0a |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline-results.json | 12863fd07355c718bd6988dbaa50f6ac9a1ffb68add58950149342f26b6a5dd2 |
