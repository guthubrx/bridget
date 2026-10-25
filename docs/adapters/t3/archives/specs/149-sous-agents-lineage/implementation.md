# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/implementation.md](../../../../../../specs/149-sous-agents-lineage/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Spec** : 149-sous-agents-lineage. **Branche** : session-149-sous-agents-lineage.
- **Worktree Bridget** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`
- **Worktree T3** : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
- **Base Bridget** : `6807c22b7ada683f757486a6382aeda170ec68ab`
- **Date de référence** : 2026-10-10. Date de début non relevée dans les sources.
- **Statut global au 2026-10-10 : Livré sur disque, non activé. T001 à T045 validés ; reçu final validation/final.md.** Ce bloc prévaut sur les listes « Ce qui est vrai » et « Ce qui n'est pas vrai » ci-dessous, qui datent de la ronde r3 et restent historiques. Synthèse : `validation/prelivraison149.md`.

<a id="extrait-2"></a>

## Extrait 2

| Document | Rôle | Chemin absolu |
|---|---|---|
| Recettes réelles GLM et Codex (r5) | Preuves réelles, par le parcours natif Bridget | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md` |
| Écriture et refus réels (r5) | Preuves d'écriture autorisée et de refus | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/provider-write149.md` |
| Parents externes par le parcours natif Bridget (r5) | Preuves des parents dans un terminal | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/standalone149.md` |
| Interop réseau (r2) | T036, périmètre exécuté 65/65 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| Scénarios dégradés (r2) | T039, PARTIEL 35/37 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` |
| Synthèse des preuves réseau (r2) | Récapitulatif de l'interop et de la reprise | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md` |
| Correctif de fixture SQLite (r7) | Cause de collision corrigée, tests de fixture seulement | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-fixture-sqlite-sonnet-r7.md` |
| Reçu complémentaire (r7) | Empreintes, compteurs et binaires | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-validation-receipt-r7.json` |
| Latence du hachage (r6) | Coût du hachage du CLI, avant et après `sha2` | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/hash-latency149.md` |
| Tests de frontières T3 (r2) | G3 et G5, 262 PASS | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-final-boundary-tests-sonnet-r2.md` |
| Durcissement runtime T3 (r5) | Permissions, UI, types | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md` |
| Lint et format T3 (r2) | 0 erreur lint, format vert | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r2.md` |
| Tests natifs r9 | 1840 PASS agrégés, 0 FAIL final, SC005 non conclu | `validation/native-tests-sonnet-r9.md`, `validation/native149-release-receipt-r9.json` |
| Revue sources finale r2 | SOURCE_ONLY_APPROVE, lecture seule | `validation/native-restart-final-source-review-sonnet-r2.md` |
| Preuves réseau r3 | Interop, reprise, O4.6, E2, F3 sur release r8 | `validation/native-network-proofs-r3.md` |
| Recette UI native r1 | UI web sur daemon réel r8, enfants fermés, APPROVE sur périmètre | `validation/ui-native-recipe149-sonnet-r1.md` |
| Smoke réel r6 | GLM PASS ; arrêt du daemon avec enfant Codex ; relance contournée | `validation/native-real-smoke-sonnet-r6.md` |
| Journal UI local r1 | Libellé `journal_unavailable`, 25 tests UI (rectification du principal en tête) | `validation/ui-journal-local-error-haiku-r1.md` |
| Historique daté | `docs-proof-sync-haiku-r5.md` et `guides-native149-haiku-r1.md` : rapports documentaires antérieurs aux rondes r6 à r9, non mis à jour | `validation/` |
| Revue finale | Verdict des sources, gaps G1 à G9 (historique, antérieur à r2 à r9) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` |
| Matrice de preuves | Niveau de preuve par tâche. Antérieure aux rondes r2 à r9, non relue. À relire après T039. | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/proof-map149.md` |
| Routage agents | Qui fait quoi | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/agent-routing149.md` |
| Dégradations T3 | Six scénarios, couche T3 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/degradation149.md` |
| Recette UI | Preuves visuelles (T040) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipe149.md` |
| Reçu de build natif | Empreintes et binaire (r3) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json` |

<a id="extrait-3"></a>

## Extrait 3

| Phase | Contenu | État |
|---|---|---|
| Phase 0 | Contrats et gate documentaire | Prouvée. Gate G-P approuvé (`permissions-contract-deltas-r4.md`). |
| Phase 1 | Attestation effective et héritage natif | Prouvée au niveau unitaire. Partielle pour le processus réel. Les parents GLM et Codex réels sont observés (r5, R1, R2a, R2c). Reste T012 (recheck à la reprise du wrapper, G7). |
| Phase 2 | État natif, journal et suivi | Prouvée au niveau unitaire pour l'essentiel. Partielle sur T020, T032, T034. Reprise réelle : F2 corrigé (r3), O4.6 et E2 à prouver (r4). Suivi `--follow` sans test de bout en bout (G8, O5). |
| Phase 3 | Projection et surfaces T3 | Prouvée au niveau serveur T3 et UI, avec daemon simulé (r5). |
| Phase 4 | Recettes et revue finale | **Partielle.** T036 : interop 65/65 sur le périmètre exécuté, pair Codex simulé. T037 : preuves réelles r5 (GLM et Codex) et smoke r6 (GLM), cases non cochées. T038 : preuves réelles partielles r5 (annulation Codex, branches négatives). Annulation GLM et mode plan non joués. Smoke r6 : arrêt du daemon pendant un enfant Codex PASS (release r8) ; relance avec parent vivant contournée. T039 : recovery149.ts 43/43 PASS en r3 ; F2 corrigé ; O4.6 et E2 en FAIL r3, à prouver en r4. T040 : UI native r1 (daemon réel r8) APPROVE sur périmètre ; recette r5 avec daemon simulé. T041 : revue sources r2 SOURCE_ONLY_APPROVE ; tests natifs r9 1840 PASS agrégés ; relectures finales non faites ici. |
| Phase 5 | Sources et livraison sans activation | **Non commencée.** T043 à T045. |

<a id="extrait-4"></a>

## Extrait 4

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/delegation_mcp.rs` : schémas MCP.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs` : admission, héritage, annulation.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code_contract.rs` et `t3code_contract_v2.rs` : projection T3 v1 et v2.
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-transport/src/claude_stream_json.rs` et `codex_app_server.rs` : transports fournisseurs.

<a id="extrait-5"></a>

## Extrait 5

Côté T3 :

<a id="extrait-6"></a>

## Extrait 6

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage.ts` et `BridgetReader.ts` : lecture de la lignée, décodage strict.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.ts` : filtre et arrêt.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal.tsx` : journal de tâche.

<a id="extrait-7"></a>

## Extrait 7

| Correction | Constat initial | Résultat après relecture ou preuve |
|---|---|---|
| Ordre de révocation | Le refus d'héritage passerait avant le rejeu. | Jugé faux. Le rejeu reste évalué avant `parent_fact`. Test : `native148_replay_precedes_grant_revocation_and_definition_change`. Révocation directe : `delegation_grant_required`. Révocation de la racine : `permission_not_inherited`. |
| Source opaque | Une source nécessaire opaque pouvait être ignorée. | Refus nommé `permission_source_unavailable`, sans repli. Preuve réelle : R4a (r5). |
| Bouton Arrêter à 1280 px | Masqué en r2. | Atteignable en r3 à r5 (clic réel, `ui-recipe149.md`, `t3-runtime-hardening-sonnet-r5.md`). |
| Contexte forgé (F1 de revue) | CHANGES_REQUIRED en r3. | Corrigé en r4 : 5 tests et un test de mutation. |
| Barre basse (F2 de revue) | Dernière ligne masquée en r3. | Corrigé en r4. Mesures à 375, 480 et 1280 px (r5). |
| Tests natifs 148 | 3 tests rouges en r3. | Corrigés en r4 dans `native_delegation.rs`. |
| Rejeu, contenu différent | Refus nommé attendu. | Test `native_delegation.rs` : `envelope_mismatch`. |
| Latence du hook (r4) | Hook au-delà de 3 s en debug. | Corrigée par `sha2` `asm` (r6). Release : 118 ms. Recette réelle r5 : hook sans latence bloquante. |
| Harnais de recette (r5) | Prompt en argv refusé ; touches de dialogue trop tôt ; aide au grant présente. | Corrigé dans `recipes/` : `--type-prompt`, touches différées, aide au grant retirée. Source : `native-real-recipes-sonnet-r5.md`. |
| Collision de racine de fixture (r7) | 11 échecs sur 400 exécutions. `DatabaseBusy` observé. | Correctif de test seulement : compteur atomique dans le nom de racine. Aucune assertion affaiblie. Production inchangée. |
| Pair Codex réel sur `api.openai.com` (r2) | Un faux `codex` au PATH a lancé le vrai binaire. Réponse 401. | Corrigé : chemin absolu dans `settings.json` de la fixture. Aucun secret envoyé. Voir `interop149.md`. |
| F1 : reprise avec mission en vol (r1) | Mission relancée à la reprise : deux tours, deux PID, 12 lancements à 13. | Corrigé par `prepare_restart` (r5) : une tâche en vol devient `failed: unreachable` au démarrage. 1 tour, 1 PID, 0 relancement (R9.2.a à e). |
| F2 : racine retenue après redémarrage (r2) | Racine bloquée en `waiting_for_children` par une exécution laissée `running`. | **Corrigé au runtime (r3, release r8).** R9.4.a à i PASS : parent hors ligne, racine retenue sort de l'attente, une seule remise au retour du parent. Revue sources r2 APPROVE. |
| O4.6 : relance rapide du daemon (r3) | SIGKILL au groupe du wrapper après 0,5 s ; fournisseur natif orphelin 18,3 s. | Délai natif commun de 8 s, sans SIGKILL natif, marqueurs conservés ; 9 tests O4 r9 PASS ; revue sources r2 APPROVE. **Runtime à prouver en r4.** |
| E2 : propriétaire natif perdu (r3, simulé) | `queued` imbriqué bloquant, racine en attente sans fin. | Garde de filiation ; 3 tests E2 r9 PASS ; revue sources r2 APPROVE. **Runtime à prouver en r4.** |
| F3 : hygiène d'annulation (r3, E3.c) | Exécution `starting` restée active après `cancelled`. | Fermeture CAS avant `cancelled` ; 5 tests F3 r9 PASS ; revue sources r2 APPROVE. Câblage final à prouver. |
| Alias Codex (r9) | Alias de socket dans `BRIDGET_HOME`, cause d'échec de recette. | Alias sous `/private/tmp/bridget-codex-*` (0700). 2 tests unitaires et 4 recettes réelles Codex 0.161.0 avec faux fournisseur HTTP, pas un modèle. Recette TUI 0.153.4 : timeout, hors périmètre. |

<a id="extrait-8"></a>

## Extrait 8

| # | Scénario | Preuve sur fixture ou unitaire | Preuve réelle | Reste à prouver |
|---|---|---|---|---|
| 1 | Rejeu | `delegation149_cancel_rejeu_mismatch_etats_et_plafond`. T3 : D10 PASS (fixture). | R1.1 (r2) : 20 rejeux, 1 lancement, 1 tour, 1 remise. Modèle réel non rejoué. | Rejeu avec modèle réel (T039). |
| 2 | Annulation | `native148_cancel_stops_native_descendants_before_parent`. T3 : D13 PASS (fixture). | R5 (r5) : enfant Codex réel annulé après 20 s de travail, fichier figé à 7 lignes pendant les six secondes de contrôle, 0 processus restant. R2.2 (r2) : deux PID terminés. | Annulation d'un enfant GLM (non joué). Délai de traitement, nombre exact d'appels `task_cancel` et de PID (non mesurés). |
| 3 | Reprise après coupure | T3 : D7 et D9 PASS (UI, daemon simulé). | R9.1 et R9.2.a à e (r2) : F1 corrigé, 1 PID, 0 relancement. R9.4.a à i (r3) : F2 corrigé, une seule remise. Arrêt SIGTERM du daemon pendant un enfant Codex (r6) : processus morts en 1,32 s au plus, puis `failed/unreachable` sans doublon. | O4.6 (relance rapide, SIGKILL après 0,5 s) et E2 : à prouver en r4. Relance réelle avec parent Codex vivant : bloquée par un lien d'état, contournée (r6). Sous launchd non vérifié. |
| 4 | Parent extérieur | `native149_cancel_recu_rejeu_et_mismatch_via_flux`, `native149_gardes_client_et_authorite`. S149-13. | R3.1, S6.1, S6.2 (r2) : `task_unavailable`, aucune fuite. | Aucun. |
| 5 | Identité inconnue | S149-32 : `permission_attestation_unavailable`. 9 tests T3. | R4.1 (r2) : token forgé, identité absente, fil sans binding : refus nommé. R4b (r5) : daemon absent, refus du hook. | Identité réelle d'un parent GLM hors fixture (T036). |
| 6 | Refus fournisseur | `native149_can_use_tool_deny_est_correle_par_request_id_exact_et_refuse_la_remise` (faux CLI `/bin/sh`). | R1 (r5) : GLM réel, écriture hors politique refusée, fichier absent, tâche `failed` + `provider_permission_denied`. R2b (r5) : confinement refusé. | Refus d'un outil par Codex réel (non joué). Passage à `failed` côté daemon (G4) : le cas GLM réel le montre, le test unitaire reste à rattacher. |

<a id="extrait-9"></a>

## Extrait 9

Couche T3 (`degradation149.md`) : 13 PASS, 1 SKIP (groupe natif), serveur T3 et base réels, daemon simulé.

<a id="extrait-10"></a>

## Extrait 10

| Critère | Ce qui est prouvé | Ce qui reste |
|---|---|---|
| SC001 : un enfant réel avec tâche, résultat, zéro lancement T3 | Enfant GLM réel avec résultat (R2a, r5). Enfant Codex réel avec résultat (R2c, r5). Ces recettes n'utilisent pas T3. Lineage affiche l'enfant dans l'UI web, avec daemon simulé (T040). | Enfant réel affiché dans l'interface T3 : non prouvé. |
| SC002 : ouvrir le journal ne relance pas la mission | D11 : 100 `show` et 100 `list` sans événement ajouté. R9.1 et R9.1b (r2) : reprise sans relance, ACK conservé. | Aucun. |
| SC003 : écritures permises réussies côté GLM et Codex | Réel (r5) : écriture GLM depuis parent GLM (R1), depuis parent Codex (R2a), Codex vers Codex dans le workspace (R2c). | Écriture GLM depuis Codex `workspaceWrite` : refusée nommément, pas réussie (R2b). |
| SC004 : par le parcours natif Bridget, délégation, lecture et annulation | Recettes r5 sans variable T3 ni processus T3 (`standalone149.md`). R6 (r2) : T3 coupé, annulation native servie. | Processus réel avec T3 réel et modèle réel : non joué. |
| SC005 : opt-outs existants | Tests 147 et 148 dans la batterie. S149-21 non nommé. | Nommer le test (action de suivi). |
| SC006 : six scénarios US6 | Section 6. Les six scénarios ont une preuve réelle, avec fournisseur réel (R1, R2b, R5) ou simulé (r2). | F2 corrigé (r3). O4.6 et E2 à prouver (r4) avant T039. |
| SC007 : livraison sans redémarrage | Non commencée. | T043 à T045. |

<a id="extrait-11"></a>

## Extrait 11

| Suite | Résultat | Source |
|---|---|---|
| Natif daemon | 1507 PASS, 0 FAIL final, 58 ignorés (r9). Trois échecs transitoires sous charge, rejoués seuls en PASS. Bench SC005 p95 non conclu. Précédent r7 : 1478 PASS, 61 ignorés. | `native-tests-sonnet-r9.md`, `native149-release-receipt-r9.json` |
| Natif transport | 333 PASS, 0 FAIL, 2 ignorés (r9). Précédent r6 : 328 PASS. | `native-tests-sonnet-r9.md` |
| Natif agrégé | 1840 PASS, 0 FAIL final, 60 ignorés (r9) : 1507 + 333. Deux exécutions distinctes, pas une passe unique. Recompte r8 corrigé : 1491 + 331 = 1822 ; r9 = 1822 + 19 nouveaux - 1 test non conclu. Aucun claim global GREEN. | `native-tests-sonnet-r9.md` |
| Interop réseau (r2) | 65 PASS sur 65 (43, 15 et 7). Périmètre exécuté. Pair Codex simulé. | `interop149.md` |
| Scénarios dégradés (r2) | 35 PASS sur 37. R9.4.b et R9.4.d FAIL (F2). | `recovery149.md` |
| Recettes réelles (r5) | Scénarios R1 à R5 PASS sur les voies exécutées, limites nommées. Une exécution par scénario. | `native-real-recipes-sonnet-r5.md` |
| Serveur T3 (G3 et G5) | 262 PASS sur 16 fichiers. G3 : 21 tests. G5 : 48 tests. Références r2, pas r1. | `t3-final-boundary-tests-sonnet-r2.md` |
| UI web et permissions T3 (r5) | 128 PASS (permissions et consommateurs) et 24 PASS (UI et client). Navigateur de prévisualisation avec MOCK natif, non un modèle réel. | `t3-runtime-hardening-sonnet-r5.md` |
| Types (`tsc --noEmit`) | Serveur : 16 diagnostics, dont 15 dans `BridgetRustInterop*` et 1 dans `CodexMcp.ts` (TS377030, hors 149). Web : 10 diagnostics. Tous en baseline, 0 nouveau. | `t3-runtime-hardening-sonnet-r5.md` |
| Lint et format T3 (r2) | Lint global : 0 erreur, 908 avertissements (7 nouveaux, non bloquants dans `BridgetReader.ts` et `orchestration.ts`). Format : 48 fichiers corrects. | `t3-lint-format-haiku-r2.md` |

<a id="extrait-12"></a>

## Extrait 12

**Limites :** les fixtures `claude` sont des copies de `/bin/echo` ou `/bin/sh` pour les tests unitaires. Les recettes réelles (r5) appellent les modèles réels GLM et Codex, une exécution par scénario. Les preuves T3 et réseau (r2, r5) utilisent un serveur T3 réel, un pair Codex simulé et un daemon simulé. Les dépendances existantes sont liées, sans installation.

<a id="extrait-13"></a>

## Extrait 13

| Id | Tâche | Constat | État |
|---|---|---|---|
| G1 | T009 | Chemin T3 privé (`delegation_mcp::execute`, `reattest`) sans test Rust. | Ouvert. Couvert en partie par T036 (r2). |
| G2 | T011 | Injection de `turn/start` Codex non assertée par un test d'octets. Le test T036 (U0) observe la méthode, sans test d'octets. | Ouvert. |
| G3 | T006 | Clos : 7 tests vérifient le retrait du fait Codex et l'isolation entre runs et fils. | Clos. Preuve : `t3-final-boundary-tests-sonnet-r2.md`. |
| G4 | T015 | Passage de la tâche à `failed` après refus. Recette GLM réelle (R1, r5) : tâche `failed` + `provider_permission_denied`. | Partiel. Le test unitaire daemon reste à rattacher. |
| G5 | T035 | Clos : matrice des trois origines et quatre branches, avec contrôles positifs. | Clos. Preuve : `t3-final-boundary-tests-sonnet-r2.md` (48 tests). |
| G6 | T022 | Branche `parse_snapshot` v1 sans test direct. | Ouvert. |
| G7 | T012 | Recheck à la reprise du wrapper sans test. | Ouvert. |
| G8 | T020, T032 | Suivi `--follow` (`journal_follow`) sans test Rust. | Ouvert. |
| G9 | T017 | Débordement de `seq` sans test. | Ouvert. |

<a id="extrait-14"></a>

## Extrait 14

| Id | Constat | État |
|---|---|---|
| O1 | `thread.stop` lit la lignée avant d'interrompre (`Orchestrator.ts`, lignes 9073 à 9094). Coût non mesuré. Pire cas : 6 s par page. Sens exact de G-P-07(b) : clarification proposée (r2). | **Ouvert.** |
| O2 | Réécriture de tout le snapshot à chaque mutation. | **Clos au niveau T3 avec moteur natif simulé** (r4, section Z2). |
| O3 | Un fil pouvait afficher « Bridget indisponible » après une erreur locale de journal. | **Clos au niveau T3 avec moteur natif simulé** (r4, ZG-4). Les erreurs globales dégradent toujours la racine. |
| O4 | Le fournisseur en vol survit à l'arrêt du daemon : `ppid 1`, 6,9 s, groupe de processus propre (r2, R9.2). Sous launchd non vérifié. | **Partiellement fermé.** Arrêt au SIGTERM du daemon prouvé (r6, release r8) : fournisseur et commande morts en 1,32 s au plus. Relance rapide O4.6 à prouver (r4). Sous launchd non vérifié. |
| O5 | Suivi `--follow` sans test de bout en bout. | **Ouvert.** |
| O6 | `queued` non engagé non exposable en réel après SIGTERM (6 tentatives en `starting`). | Information. Couvert par un test unitaire. |
| O7 | Un fournisseur est lancé pendant l'arrêt du daemon, sans mission. | Information. |
| E1 | Lecture échouée volontairement dans un test. | Clos. |
| E4 | Fournisseur ignorant SIGTERM, arrêté par un SIGKILL externe du wrapper. | Limite connue, pas une promesse globale. |
