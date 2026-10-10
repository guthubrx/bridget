# Matrice de preuves 149 (version relue par la revue T041)

## Synchronisation finale du 2026-10-10 (prévaut sur les sections ci-dessous)

- **Tâches :** T001 à T042 validés au principal. T043 à T045 en attente (livraison). Le principal coche les cases de `tasks.md`. Ce document ne coche rien.
- **Revue des sources :** `validation/native-restart-final-source-review-sonnet-r2.md` = SOURCE_ONLY_APPROVE sur la production b2b87458 (111 fichiers). Un petit changement de libellé du journal T3 (sha4abac...) est approuvé.
- **Recettes réelles :** GLM `glm-5.3-flash` et Codex `gpt-6.1-sol` (effort high), sans nouveau grant : `validation/native-real-recipes-sonnet-r5.md`. Smoke r6 : `validation/native-real-smoke-sonnet-r6.md`. Redémarrage avec parent Codex vivant : `validation/native-real-restart-sonnet-r7.md` (29 oracles OK, 0 échec).
- **Réseau et interop :** `validation/native-network-proofs-r3.md` (T036 fermé, F2, journal et interface réelle). `validation/native-network-proofs-r4.md` ferme O4.6, E2 et F3 (16 contrôles OK, 0 FAIL, niveaux distingués).
- **Interface :** `validation/ui-native-recipe149-sonnet-r1.md` (daemon réel, fournisseurs fictifs explicites).
- **Frontière T3 :** `validation/t3-final-boundary-tests-sonnet-r2.md` (G3 et G5 fermés).
- **Tests natifs :** `validation/native-tests-sonnet-r9.md` : 1840 PASS (1507 daemon, 333 transport), 0 FAIL final. Le recompte r8 de 1822 remplace le chiffre 1825. Ce n'est pas un GREEN global exhaustif.
- **SC005 :** le bench de performance r9 n'est pas concluant sous charge. Ce bench ancien ne mesure pas le SC005 de l'opt-out SPEC149.
- **Scénarios dégradés :** six scénarios mappés aux preuves réseau r3 et r4, aux recettes réelles r5 et r7, et aux tests `readOnly`. Les cas `readOnly` et annulation GLM réels ne sont pas joués : limites acceptées.
- **Opt-out (S149-21) :** les tests 147 et 148 déjà présents en régression couvrent l'opt-out et le MCP désactivé. Aucun nom de test n'est inventé. Si un nom ne se trouve pas dans les sources citées, il reste un manque d'audit.
- **Limites acceptées :** G6 (parse v1) et G9 (rollover de seq) sont couverts par revue de source et tests de forme, sans test direct. Un fournisseur qui ignore SIGTERM plus de 8 s refuse le démarrage et garde son marqueur (conforme au guide d'exploitation). E4 (SIGKILL externe du fournisseur) n'a pas de garantie universelle. Les liens `agent_links` après reprise sont observés (r4), sans impact testé : note non bloquante.
- **Synthèse de livraison :** `validation/prelivraison149.md`.


## Actualisation du principal — après r9 et réseau r4

Les sections ci-dessous conservent la matrice historique antérieure aux recettes réelles. Leurs compteurs et mentions « absent », « aucun modèle » ou « pending T036–T039 » ne décrivent plus l'état courant. Les preuves actuelles sont `native-real-recipes-sonnet-r5.md` (GLM et Codex réels, droits hérités sans grant neuf), `native-network-proofs-r3.md` (interop privée, résultat retenu, suivi CLI réel), `ui-native-recipe149-sonnet-r1.md` (UI avec daemon réel), `t3-final-boundary-tests-sonnet-r2.md` (G3/G5), `native-tests-sonnet-r9.md` et `native-restart-final-source-review-sonnet-r2.md`.

`native-network-proofs-r4.md` ferme O4.6, E2 et F3 sur le candidat release `abc850858975` : 16 contrôles réussis, dont l'état initial E2 simulé et une observation O4. Aucun crash naturel de cette fenêtre E2 n'est prétendu. Le test de performance r9 reste non conclu. La dernière recette avec parent Codex vivant r7 et les tâches de livraison T043–T045 restent en attente. Les 45 cases relèvent encore du principal.

Date : 2026-10-10. Cette version remplace la matrice intermédiaire du rédacteur documentaire. Elle a été corrigée après lecture des tests exécutés, de leurs assertions et des logs. Elle n'exécute rien. Aucune case de `tasks.md` n'est cochée. Le cochage revient au principal.

Rapport de revue lié : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` (verdict SOURCES : SOURCE_ONLY_APPROVE ; session : PARTIEL).

## 1. Règles de classement

Un état se lit avec un **niveau cible**. Une tâche est « Prouvée » quand son propre critère dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md` est rempli à son niveau. Elle n'a pas besoin d'une recette réelle quand son critère ne la demande pas.

| Niveau | Sens |
|---|---|
| DOC | Document relu et gates APPROVE |
| U | Test unitaire ou de contrat, nommé, exécuté, PASS |
| P | Test avec un vrai processus et un faux CLI (`/bin/sh`, copie de `/bin/echo`). Aucun modèle. |
| R | Recette sur serveur T3 réel avec base privée et **daemon Bridget simulé** |
| M | Modèle ou CLI réel de bout en bout. **Aucun à ce jour.** |

États : **Prouvée** (niveau cible atteint) | **Partielle** (une part du critère manque ; le manque est nommé) | **Non prouvée** (aucune exécution) | **Pending** (dépend d'un autre propriétaire).

Un plan, une revue ou un diff ne prouvent pas une implémentation. Un test nommé et exécuté la prouve à son niveau. Les copies de `/bin/echo` ne prouvent aucun comportement de modèle.

## 2. Contradictions entre rapports (le plus récent précis prévaut)

| Point | Rapport ancien | Rapport retenu |
|---|---|---|
| Gate G-P | En-tête de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md` : « REQUEST_CHANGES ciblé r3 » | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md` : APPROVE (G-P-07, G-P-08 fermés). L'en-tête est périmé, à corriger par le principal. |
| Batterie native | r3 : 1791 PASS, 6 FAIL | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-tests-sonnet-r4.md` : 1796 PASS, 0 FAIL, 63 ignorés. Recomptée sur les logs : daemon 1471/0/61, transport 325/0/2. |
| Transport natif r3 | « 326 PASS » | r4 : 325 (erreur de somme corrigée) |
| Serveur T3 149 | r1 106, r3 115 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` : 120 PASS |
| UI et client | r4 « 22 PASS » (3 fichiers) | Rejoué par la revue : **24 PASS sur 4 fichiers** (le mobile ajoute 2 tests). Le « 22 » de r4 ne compte pas le mobile. |
| Bouton « Arrêter » à 1280 px | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149.md` (r2) : masqué | r3 et r4 : atteignable, clic réel |
| Finding F1 (contexte forgé) | r3 : CHANGES_REQUIRED | r4 : corrigé, 5 tests, test de mutation, captures 06 à 09 |
| Finding F2 (barre basse) | r3 : dernière ligne masquée | r4 : corrigé, mesures à 375, 480, 1280 px |
| `native148` | r3 : 3 tests rouges | r4 : corrigé (`native_delegation.rs` lignes 40 à 42), 19 PASS |
| Binaire natif | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r3.md` : binaire absent | r4 : `bridget-823e8a5fab8a`. L'ancien `bridget-ed5e28bc8ddc` est périmé. |
| Recette UI | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149.md` (r2) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` est canonique pour T040 |
| Matrice intermédiaire | T022 et T029 « Non prouvées », T001 à T002 et T040 « Partielles », plusieurs « non nommés » | Corrigés ci-dessous après lecture des tests. |

Sources inchangées depuis r4 : empreinte Rust `e6f45086b23974604667c93d3b34755d7d73dc5dadbe4dde94c1e09f34285408` (196 fichiers), `git diff HEAD` T3 `fc9fa99f4ada2dca`. Vérifié par la revue.

## 3. Compteurs retenus (dernière ronde par suite)

| Suite | Compte | Source |
|---|---|---|
| Natif daemon | 1471 PASS, 0 FAIL, 61 ignorés | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-r4-final-daemon.log` |
| Natif transport | 325 PASS, 0 FAIL, 2 ignorés | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-r4-final-transport.log` |
| Observer en série | 10 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-r4-observer-serial.log` |
| Serveur T3 149 (5 fichiers) | 120 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` |
| UI et client (4 fichiers, mobile inclus) | **24 PASS**, rejoué par la revue | sortie `vp test run` de la revue |
| Permissions T3 | 74 PASS (67 + 6 + 1) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-permissions-fixtures-final-r3.md` |
| Contrats Lineage T3 | 22 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-lineage-contract-tests-r1.md` |
| Matrice réseau RPC | 38 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/network149-matrix-run-r4.txt` |
| Navigateur (fixture) | 9 PASS, 9 captures | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` |
| `tsc --noEmit` | contracts, client-runtime, provider-core, mobile : 0. Serveur : 16, web : 10. Tous dans des fichiers non touchés par le diff (règles de style Effect). | revue T041 |

Les rondes ne s'additionnent pas.

## 4. Matrice par tâche (T001 à T045)

Les chemins de tests sont absolus. Les noms de tests sont ceux du code.

### Phase 0 : contrats et gate documentaire

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T001 | Prouvée | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/plan-lineage-r3.md` (G-L APPROVE), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md` (G-P APPROVE) | Rien. |
| T002 | Prouvée | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/test-strategy.md` (33 scénarios), deux décisions distinctes G-L et G-P toutes deux APPROVE | Le critère est la stratégie et les deux gates. L'exécution des 33 scénarios n'en fait pas partie. |
| T003 | Pending | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-prepare-r1.md` : empreintes de préparation du lanceur et du CLI | Registre de recette réel et provenance standalone : attendent T038 (autre propriétaire). |

### Phase 1 : attestation effective et héritage natif

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T004 | Prouvée | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/bridgetPermissions149.test.ts` : 30 tests | Rien. |
| T005 | Prouvée | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/provider-core/src/server/mcpSession149.test.ts` : 14 tests (publication, rotation, vérificateur qui invalide, fait plus récent gardé, credential étranger, charge fermée, aucun secret exposé) | MCP privé authentifié réel : T036. |
| T006 | Partielle | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` : « publishes the final turn/start parameters for the mounted credential » | Gap G3 : retrait du fait Codex sur échec, fin et finalisation (code présent, aucun test Codex nommé). |
| T007 | Prouvée | U | Même fichier : « sends the final permission settings… », « publishes on turn start, follows init and status modes and retires at turn end », « captures fresh CLI digests through the pinned launcher with PATH priority », « invalidates… when the launcher or CLI target changes » | CLI réel : T037, T038. |
| T008 | Prouvée | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts` : 9 tests (v1 sans fait, v2, absent, étranger, autre run, tombstone, invalidé, run changé, credential remonté) | Rien de nommé. |
| T009 | Partielle | U | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` : `native149_un_fait_d_autre_demande_ne_donne_aucun_droit`, `native149_sans_fait_le_v1_est_refuse_et_les_lectures_survivent_au_tombstone` | Gap G1 : `handle_attested`, `reattest`, `private_proof` et le message `NativeDelegationT3` n'ont aucun test Rust. Preuve attendue de T036. |
| T010 | Prouvée | U | Même fichier : `native149_posture_omise_resout_le_developpement_effectif_du_fait`, `native149_spawn_refuse_les_entrees_gelees_changees_sans_lancement` ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions149_tests.rs` : `native149_snapshot_fige_et_deduplique_les_contextes` ; oracle 148 de rejeu avant fait (`native148_replay_precedes_grant_revocation_and_definition_change`) | Reprise avec processus réel : T039. |
| T011 | Partielle | U | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permissions149_tests.rs` : `native149_codex_complet_devise_bypass_sans_yolo`, `native149_codex_lecteur_refuse_le_developpement_et_reduit_en_decouverte`, `native149_claude_vers_codex_exige_lire_ou_complet_sans_regles` ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/tests/native_permissions149_test.rs` : `native149_formes_fermees_codex_accepte_quatre_sandbox_et_refuse_le_reste`, `native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement` | Gap G2 : l'injection de `approvalPolicy`, `approvalsReviewer`, `sandboxPolicy` et `cwd` dans `turn/start` n'est assertée par aucun test. Aucun Codex réel : T037. |
| T012 | Partielle | P | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/tests/native_permissions149_e2e.rs` (5 tests, dont `native149_fait_claude_incremente_la_revision_quand_le_cwd_change`) ; `native149_claude_meme_famille_exige_contexte_vivant_et_meme_racine` | Gap G4 : la tâche `failed` avec le code nommé n'est pas assertée côté daemon. Aucune trame `can_use_tool` réelle. |
| T013 | Partielle | U | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/native_permission_observer149_tests.rs` : `native149_cycle_de_vie_fichiers_prives_et_nettoyage_raii` ; reçu natif r4 : fait initial publié avant le démarrage du TUI natif (`wrapper.rs` lignes 4148 à 4184) | « Profil global non modifié » et « secrets absents de l'enfant » : pas de test nommé. `env_remove` des deux variables est dans le code (lu par la revue). |
| T014 | Partielle | U | Observer 10/10 (liste dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` §6) | Chemin standalone réel (premier parent externe PTY) : T038. |
| T015 | Partielle | P | Voir §6 : S149-29 à S149-33 nommés et exécutés | Gap G4 (S149-29, lien daemon). S149-33 branche d (vrai CLI en PTY) : T038. |

### Phase 2 : état natif, journal et suivi

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T016 | Prouvée | U | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_lineage_tests.rs` : `delegation149_migration_148_derive_racine_parent_et_horodatages`, `delegation149_migration_fermee_cycle_profondeur_et_volume`, `delegation149_parent_lineage_profondeur_et_descendants` (`task_depth_limit`) | Racine et parent conservés après retry : lus dans `save_projected`, sans test dédié. |
| T017 | Prouvée | U | Même fichier : `delegation149_insert_seq_signal_durable_et_horodatages`, `delegation149_conflits_insertion_rollback`, `delegation149_lectures_silencieuses_et_save_inchange` (100 lectures : séquence inchangée, aucun signal) | Débordement de séquence : gap G9. |
| T018 | Prouvée | U + P | `delegation149_page_bornes_ordre_pagination_budget_et_curseur`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_lineage_tests.rs` : `native149_list_show_journal_et_disponibilite_journal` ; `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/lineage_cli_test.rs` : `lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel` | Rien de nommé. |
| T019 | Prouvée | U + P | `native149_watch_ready_changed_silence_resync_et_invalidation`, `native149_watch_refuse_sans_socket_ou_sans_capacite`, `lineage_cli_149_watch_ready_puis_fermeture_propre` | Rien. |
| T020 | Partielle | U | `native149_page_journal_bornes_permissions_et_fermeture` (pages 16 KiB, 100 événements, droits, fermeture) | Gap G8 : le suivi réel `--follow` (AttachRelay) n'a aucun test Rust. |
| T021 | Prouvée | U + P | `native149_cancel_recu_rejeu_et_mismatch_via_flux`, `delegation149_cancel_rejeu_mismatch_etats_et_plafond`, `native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant`, oracle 148 `native148_cancel_stops_native_descendants_before_parent` | Rien. |
| T022 | Prouvée | U | `native149_marqueur_projection_ferme_et_exclut` (11 marqueurs invalides refusés, préfixe seul insuffisant), `native149_snapshot_v2_exclut_le_filtre_avant_normalisation` (exclusion avant normalisation, marqueur invalide refuse tout, fil ordinaire intact) | Gap G6 (branche v1 de `parse_snapshot` sans test direct). `t3code.rs` n'a pas changé : le filtre est dans l'analyseur. |
| T023 | Prouvée | U + P | Suites de T016 à T021 avec magasin SQLite réel et CLI en processus | Rien. |

### Phase 3 : projection et surfaces T3

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T024 | Prouvée | U + revue | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/bridgetLineage149.test.ts` (22). Audit des comparaisons d'origine par la revue : tous les `origin !== "app_owned"` excluent `bridget_native`. | Voir G5. |
| T025 | Prouvée | U + R | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage149.test.ts` : 23 ; `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader149.test.ts` : 58 ; 5 tests F1 avec test de mutation | Rien. |
| T026 | Prouvée | U + R | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` (33) ; compteurs SQL à 0 pour runs, sessions, tours, liaisons, requêtes, workflows | Rien. |
| T027 | Prouvée | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts` : 4 | Rien de nommé. |
| T028 | Prouvée | U + R | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts`, refus des commandes sur fil virtuel (dont `queue.resume`), matrice réseau 38 | Rien. |
| T029 | Prouvée | U + R | `Orchestrator.bridget149.test.ts`, groupe « native stop » : 7 tests : arrêt d'un enfant virtuel (un seul appel natif, rien créé), refus sans transport (enfant), refus sans transport (parent avec enfants actifs), arrêt du parent seulement pour la tâche sans ancêtre actif, ancêtre échoué avec descendant actif, enfants déjà soldés sans appel, fil ordinaire sans lecteur. Clic réel sur « Arrêter » (capture 04, r4). | Observation O1 (latence d'un arrêt ordinaire, non mesurée). |
| T030 | Prouvée | U | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/client-runtime/src/state/orchestration.bridget149.test.ts` : 9 | Rien. |
| T031 | Prouvée | U + R | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx` : 4 ; recette UI `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` | Rien. |
| T032 | Partielle | U + R | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal149.test.tsx` : 9 ; reconnexion et lacune en navigateur | Préservation du focus, de la copie et de la sélection : non mesurée. Gap G8 pour le suivi réel. |
| T033 | Prouvée | U (jsdom) | `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx` : 2 PASS rejoués par la revue ; `tsc` mobile : 0 erreur | Appareil réel et iOS : hors plan. |
| T034 | Partielle | R | Matrice réseau 38 PASS, rejeux r3 (23) et r4 (22) contre le vrai serveur | Fermeture des abonnements scopés : non mesurée. |
| T035 | Partielle | U | 33 + 23 + 58 + 4 + 22 tests T3 ; audit statique des origines par la revue | Gap G5 : matrice « trois origines par quatre branches » absente. |

### Phase 4 : recettes complètes et revue finale

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T036 | Pending | R | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/network149.md` : couche T3 PASS. Dossier de l'autre propriétaire : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` absent. Couche native (MCP privé, deux sessions, rotation) sans reçu. |
| T037 | Pending | M | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r3.md` : checkpoint, aucun modèle lancé | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/provider-write149.md` absent. Aucun GLM ni Codex réel. |
| T038 | Pending | M | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-prepare-r1.md` : préparation seule | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/standalone149.md` absent. Aucun parent externe PTY exécuté. S149-33 branche d. |
| T039 | Pending | R + M | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/degradation149.md` : 13 PASS (couche T3), 1 groupe SKIP (natif) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` absent. Cinq scénarios natifs sans preuve de processus. |
| T040 | Prouvée | R | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` (document canonique), rondes r2, r3, r4 avec captures. Daemon **simulé**, données **synthétiques**, navigateur preview. | Pas de coque desktop (le plan l'exclut). SC001 et T037 sont séparés. |
| T041 | Prouvée (sources) | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` : SOURCE_ONLY_APPROVE, zéro finding actif | Rapport global PARTIEL tant que T036 à T039 n'ont pas de reçu. |
| T042 | Non prouvée | | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/quickstart.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/implementation.md` n'existent pas | À écrire. |
| T043 | Non prouvée | | 41 entrées modifiées dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/`, aucun commit 149 | Git réservé au principal. |
| T044 | Non prouvée | | Aucun paquet final | Pending. |
| T045 | Non prouvée | | `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/final.md` absent | Pending. |

## 5. Scénarios S149-01 à S149-33

Un scénario réel (M) reste pending tant qu'aucun reçu n'existe. Les cases « U » sont des preuves de niveau test, pas de comportement de modèle.

| Scénario | État | Niveau | Preuve nommée |
|---|---|---|---|
| S149-01 héritage discovery et development | Partiel | U | `native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain`, `native149_codex_complet_devise_bypass_sans_yolo` ; le réel est S149-14 |
| S149-02 parent lecteur refusé | Prouvé | U | `native149_developpement_d_un_parent_lecteur_refuse_sans_tache_ni_invitation`, `native149_catalogue_parent_lecteur_et_confinement_refuses_par_nom` |
| S149-03 révocation persistante | Prouvé | U | oracle 148 `native148_agent_revocation_survives_a_new_instance_until_explicit_grant`, `native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant` |
| S149-04 profondeur 8 admise, 9 refusée | Prouvé | U | `delegation149_parent_lineage_profondeur_et_descendants` |
| S149-05 lecture pure | Prouvé | U | `delegation149_lectures_silencieuses_et_save_inchange` (100 lectures, aucun signal) ; côté T3 `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/degradation149.md` D11 (200 lectures) |
| S149-06 parent lecteur refuse development | Prouvé | U | comme S149-02 ; le réel (cas 3 de S149-15) est pending |
| S149-07 parent development sans grant | Partiel | U | `native149_posture_omise_resout_le_developpement_effectif_du_fait` |
| S149-08 nested complet | Partiel | U | `native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant` (enfant et petit-enfant), `native148_cancel_stops_native_descendants_before_parent` ; processus : T039 |
| S149-09 retry, restart, cancel | Partiel | U | rejeu avant fait (oracle 148), `delegation149_cancel_rejeu_mismatch_etats_et_plafond` ; `SIGTERM` en vol : T039 |
| S149-10 écriture enfant | Pending | M | T037 |
| S149-11 identité jamais le prompt | Partiel | U | `native149_un_fait_d_autre_demande_ne_donne_aucun_droit` et oracles 148 hérités |
| S149-12 T3 mort ou partiel | Partiel | U + R | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/degradation149.md` (13 PASS côté T3) ; chemin natif sans T3 en processus : T038 |
| S149-13 autres parents invisibles | Prouvé | U | `native149_cancel_recu_rejeu_et_mismatch_via_flux`, `native149_gardes_client_et_authorite` |
| S149-14 GLM et Codex réels | Pending | M | T037, T038 |
| S149-15 refus réels | Pending | M | cas 1 et 2 couverts en U par S149-02 |
| S149-16 projection | Prouvé | U + R | `BridgetLineage149.test.ts`, `Orchestrator.bridget149.test.ts`, recette UI |
| S149-17 aucune double exécution | Prouvé | U + R | compteurs SQL à 0 et rejeux (100 `list`, 100 `show`, 0 événement) dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md` |
| S149-18 panne T3 et dégradation | Prouvé | U + R | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/degradation149.md` (13 PASS), matrice réseau |
| S149-19 invisibilité inter-conversations | Partiel | R | autre racine : 0 événement (rejeux r3). Limite : la CLI de fixture ignore `--t3-thread` (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` §6). |
| S149-20 révocation persistante | Prouvé | U | comme S149-03 |
| S149-21 opt-out MCP persistant | Non nommé | | Aucun test 149 nommé dans les rapports. Les tests 147 et 148 d'opt-out sont dans la batterie complète, sans être cités. À nommer. |
| S149-22 filtre du pont | Prouvé | U | `native149_marqueur_projection_ferme_et_exclut`, `native149_snapshot_v2_exclut_le_filtre_avant_normalisation` |
| S149-23 anciens fils intacts | Prouvé | U + R | dernière assertion de `native149_snapshot_v2_exclut_le_filtre_avant_normalisation` ; historique de 130 fils `available:false` préservé (r4) |
| S149-24 mappage inter-fournisseurs | Prouvé | U | `native149_workspace_codex_vers_claude_refuse_le_confinement`, `native149_claude_vers_codex_*`, `native149_decouverte_reduit_et_conserve_les_refus_existant` |
| S149-25 deux conversations, droits différents | Prouvé | U | `mcpSession149.test.ts` : « keeps two equal-field credentials distinct », « refuses a read from another credential on the same thread » ; `native149_un_fait_d_autre_demande_ne_donne_aucun_droit` |
| S149-26 péremption du fait | Partiel | U | `mcpSession149.test.ts` : « keeps a newer run fact when a cleanup targets the older run », « drops the fact on session rotation » ; Claude : « retires at turn end ». Codex : gap G3. |
| S149-27 séquence sûre | Partiel | U | `lineage149_version_et_borne_seq_figees`, `delegation149_insert_seq_signal_durable_et_horodatages`. Débordement : gap G9. |
| S149-28 recette UI web | Prouvé | R | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` |
| S149-29 refus `can_use_tool` corrélé | Partiel | P | `native149_can_use_tool_deny_est_correle_par_request_id_exact_et_refuse_la_remise` (+ 3 tests voisins) ; gap G4 |
| S149-30 divergence de digest | Prouvé | U + P | `native149_spawn_refuse_les_entrees_gelees_changees_sans_lancement`, `native149_recheck_des_sources_detecte_mutation_ajout_et_suppression`, `native149_revision_de_source_fichier_dossier_symlink_et_absent`, `native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement` ; reprise du wrapper : gap G7 |
| S149-31 source opaque | Prouvé | U | `native149_capture_refuse_les_sources_opaques_et_garde_les_inline` |
| S149-32 union v1/v2 | Prouvé | U | `native149_sans_fait_le_v1_est_refuse_et_les_lectures_survivent_au_tombstone` ; 9 tests `OrchestratorMcpService.bridgetPermissions149.test.ts` ; l'entrée du Rust autonome relève de T036 et T038 |
| S149-33 observer PTY | Partiel | U | 10 tests de `native_permission_observer149_tests.rs` (§6 de `review-final149.md`) ; **branche d en vrai CLI : T038** |

## 6. Gaps de preuve réels et minimaux

Détail dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` section 5. Aucun ne bloque SOURCE_ONLY_APPROVE.

| Id | Sujet | Test minimal |
|---|---|---|
| G1 | Chemin T3 privé (`handle_attested`, `reattest`) sans test Rust | Test avec faux `reattest`, ou reçu T036 |
| G2 | Injection Codex `turn/start` non assertée | Faux serveur Codex qui capture les quatre champs |
| G3 | Retrait du fait Codex sur échec | Un tour Codex qui échoue, puis fait absent |
| G4 | Tâche `failed` avec `provider_permission_denied` côté daemon | Test de `native_delegation::rejected` |
| G5 | Matrice trois origines par quatre branches | Accepter l'audit statique ou tester |
| G6 | Branche v1 de `parse_snapshot` | Un snapshot v1 avec marqueur valide |
| G7 | Refus à la reprise du wrapper | Couvert par T039 |
| G8 | Suivi réel `--follow` du journal | Test CLI `--follow`, ou reçu T036 |
| G9 | Débordement de séquence (génération renouvelée) | Fixer `seq` près de la borne puis insérer |

## 7. Ce qui reste avant le cochage final

1. T036 à T039 : reçus réels des autres propriétaires. Sans eux, SC001, SC003 et SC004 restent non prouvés.
2. S149-21 : nommer le test d'opt-out.
3. Décider du sort de G1 à G9.
4. T042 : `quickstart.md` et `implementation.md`.
5. T043 à T045 : Git, paquets, reçu final.

## 8. Coquilles relevées, non corrigées

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md`, en-tête : « G-P REQUEST_CHANGES ciblé r3 » est périmé.
- Même fichier : « àS149-33 », « Phase0 » et suivants, « profondeur8 », « liste100/128KiB », « pages16KiB/100 ».
