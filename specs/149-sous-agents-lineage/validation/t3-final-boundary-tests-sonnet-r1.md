# Session 149 - Tests T3 de frontière G3 (T006) et G5 (T035) - Sonnet r1

Date : 2026-10-10. Rôle : sous-agent testeur. Décision : **APPROVE** (aucun nouveau bug, G3 et G5 fermés par des tests nommés).

## 1. Périmètre et propriété

Fichiers écrits (tests seulement) :

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` (G3)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` (G5)
- Ce rapport.

Aucun fichier de production, contrat, Git, installation, coche, configuration ou service n'a changé. Aucun nouveau fichier de test adjacent n'était nécessaire : la retraite Codex se prouve dans le premier fichier. Aucun export n'a changé.

Preuve de non-écriture en production : `find apps packages -newer` après ma première exécution ne liste que les deux fichiers de test. Les dates de `CodexAdapterV2.ts` (09:44), `Orchestrator.ts` (13:35), `ClaudeAdapterV2.ts` (12:49) et `mcpSession.ts` (09:42) précèdent ma session (début vers 14:54).

## 2. Empreintes (sha256)

| Fichier | Avant | Après |
|---|---|---|
| `Adapters/BridgetPermissions149.test.ts` | `45551ac1944c948e954a8df02a27b009c9b7ada5d330692791e5803422df69ce` | `dd872681be7335b9a4fb9b5c3e7e6d4cc34979bc6e1aa7409e32851a11e2e39e` |
| `Orchestrator.bridget149.test.ts` | `ba3210d373f8885a829b099cbb77015d87b4d8fd2f7cd190f6ac112f2e6abdc6` | `b5f564ec4f5104561487cfe52bcc7ed9145feb0e5c966734ac3e6c5c13327699` |

Production, inchangée (liste seule, aucune prétention sur un binaire ou une recette) :

| Fichier | sha256 |
|---|---|
| `Adapters/CodexAdapterV2.ts` | `df7d6080a005d39fb7bd04676e6337e7aefb6fd590b54011c58310b223a84936` |
| `Orchestrator.ts` | `d3a770b04090439df845449f2dc613a5d7ff50593df64f96ccdca02b50d284cd` |
| `Adapters/ClaudeAdapterV2.ts` | `2816899f654ef3c60e972e6213f1611e9f06e6ea764d14c0c3f1b05601ee7e71` |
| `ProviderRuntimeRecoveryService.ts` | `3863bc53f815ccdff3a0f9813a44f6dc2a4e7a19f9e6f5b1e300c7df0bbddf4a` |
| `packages/provider-core/src/server/mcpSession.ts` | `fcad9cfb2d485ea75ba9c47e5ba7cbf449173354c9d509a936993d925c7b2081` |

Seul ajout hors bloc de tests dans `Orchestrator.bridget149.test.ts` : l'option `adapter?` de `HarnessOptions` (une ligne de type, une ligne d'usage). Les 33 tests d'origine passent sans changement.

## 3. G3 - retrait du fait de permissions Codex (T006)

Chemin réel : `makeCodexAdapterV2` + app-server rejoué (`CodexReplay`) + vrai `mcpSession` (`publishMcpProviderPermissions`, `invalidateMcpProviderPermissions`, `readMcpProviderPermissions`). Aucun mock de la fonction testée. Le fait publié est lu par la même API que le MCP.

Sémantique lue : `absent` = aucun fait (repli v1 possible). `unavailable` = pierre tombale (interdit le repli v1). `valid` = fait courant.

| Test | Branche de production prouvée | Preuve |
|---|---|---|
| retire le fait quand l'app-server rejette `turn/start` | `Effect.onError` -> `retireBridgetPermissions` | avant : `absent` ; `startTurn` échoue ; après : `unavailable`. Le rejeu a validé le `turn/start` sortant (publication juste avant), donc le fait a existé puis a été retiré |
| fin native `completed` / `interrupted` / `failed` (3 cas paramétrés) | `finalizeCodexTurn` -> `retireBridgetPermissions` | `valid` juste après `startTurn`, puis `unavailable` stable (100 ms de plus) |
| fermeture du scope de session | `Effect.addFinalizer` | `valid` dans le scope, `unavailable` après sa fermeture, tour jamais terminé |
| un tour ancien qui finit ne supprime pas le fait d'un run plus récent | garde `runId` + `expectedRevision` | run A puis run B sur le même fil (fait B révision 2) ; fin native de A (événement `turn.terminal` vu : le retrait a eu lieu) ; lecture B = `valid` `run-attempt-g3-b` révision 2 ; lecture A = `unavailable` ; la fin de B met B en `unavailable` |
| un fil en échec ne touche pas un autre fil lié | cloison par fil | fil « ok » reste `valid` (`run-attempt-g3-ok`) pendant que le fil « other » devient `unavailable` |

Total ajouté : 7 tests, 19 asserts nommés (`assert.`), 287 lignes. Fixtures : transcripts `perm-149-g3-*`, crédentiels privés `credential-session-149-g3-*`, nettoyés par `clearMcpProviderSession`.

### Preuve que les tests ne sont pas décoratifs (mutation côté seam, sans écrire en production)

J'ai ajouté temporairement un `vi.mock` de `@t3tools/provider-core/server/mcpSession` dans le fichier de test, piloté par une variable d'environnement. Je l'ai retiré ensuite (`grep -c G3_MUTATE` = 0, copie sauvegardée restaurée).

| Mutation | Résultat |
|---|---|
| aucune | 21 / 21 PASS |
| `invalidate` sans effet | **9 FAIL** : les 7 nouveaux tests plus 2 tests Claude d'origine. Chaque branche de retrait (erreur, fin, finalizer, cloison) dépend bien de l'invalidation |
| `invalidate` qui ignore le `runId` (retrait par le dernier run publié) | **1 FAIL** : seulement le test « run plus récent ». Les autres restent verts, donc ce test isole bien la garde `runId` |

Limite : je n'ai pas pu muter `CodexAdapterV2.ts` lui-même (écriture interdite). Les trois sites de retrait ont chacun un test dont l'observation tombe avant la fermeture du scope : erreur (lue avant fermeture), fin native (lue dans le scope), finalizer (lu après fermeture, tour non terminé). Retirer un seul site ferait donc échouer un test précis.

Durée : 21 tests en environ 5 s. Les tests de fin native utilisent l'horloge réelle (`it.live`, `afterMs: 400`, lecture immédiate de `valid`). Marge mesurée : l'observation `valid` prend quelques ms.

## 4. G5 - matrice trois origines par quatre branches (T035)

Producteurs réels : `delegated_task.request` (app_owned), `bridget.lineage.sync` (bridget_native), événements de projection de forme identique à ceux de `ProviderEventIngestor` avec le vrai `makeSubagentChildThread` (provider_native ; l'ingestor n'est pas exposé par le harnais). Consommateurs réels : `message.dispatch`, `ProviderRuntimeRecoveryService.recover` + `resumeQueuedRuns`, `delegated_task.completion-delivery.acknowledge`, `run.interrupt` + `ThreadManagementService.stopDelegatedTasks`.

Puits observés : lignes SQL `runs`, `provider_threads`, `provider_turns`, `provider_sessions`, `effect_outbox` (par fil et par type) et un faux fournisseur sec qui compte `openSession`/`getCapabilities`/`planSelectionTransition` (aucune vraie auth ni modèle). `openSession` vaut 0 dans les 14 cas qui montent le harnais (le test de contrat n'en monte pas). Un `provider-turn.start` dans l'outbox est le seul chemin qui démarre un tour (l'effect worker est éteint).

| Branche | app_owned (contrôle positif normal) | provider_native (contrôle positif normal) | bridget_native |
|---|---|---|---|
| start : `message.dispatch` (`queue_after_active`) sur le fil enfant | accepté ; enfant `starting` + 1 ligne `provider-turn.start` ; le message ajoute un run `queued` | refus `OrchestratorSubagentThreadReadOnlyError` (`creationSource` provider) ; 0 run, 0 effet ; sinks identiques avant/après | même refus, mais `isProviderNativeSubagentThread` = faux : seul `bridgetTaskRef` l'explique ; 0 run, 0 thread/tour/session fournisseur, 0 effet ; sinks identiques |
| resume : recovery au démarrage + `resumeQueuedRuns` | run parent `starting` -> `cancelled`, run enfant `cancelled`, tâche reste `running` (l'enfant la règle) | run parent `cancelled` ; sous-agent -> `cancelled` | run parent `cancelled` ; enregistrement natif et fil virtuel strictement inchangés (`runId` null) ; 0 run/effet/thread/tour/session pour le fil virtuel ; `resumeQueuedRuns` = 0 |
| remise : acquittement de livraison | accepté ; `completionDelivery` = `acknowledged` | refus « is not an app-owned task » ; sinks et tâche inchangés | même refus ; sinks et tâche inchangés ; aucun `completionDelivery` |
| outbox : `run.interrupt` (hold) sur un run parent vivant, puis exécution de `delegated-tasks.stop` | 1 effet `delegated-tasks.stop` sur le parent ; cohorte `disposed` ; enfant `interrupted` à l'exécution | idem effet ; la cascade de run passe le sous-agent à `interrupted` ; aucun run/effet pour l'enfant | idem effet seulement sur le parent ; tâche `running` sans livraison ; aucun effet vers le fil virtuel ; `lineageCancel` jamais appelé ; 0 run/thread/tour/session pour l'enfant |

Total ajouté : 15 tests (12 de la matrice + 3 de marqueurs et contrat), 73 `expect`, 449 lignes. Suite complète du fichier : 48 PASS (33 d'origine + 15).

Marqueurs canoniques (pas seulement l'étiquette d'origine) :

1. Un fil qui copie l'identifiant `thread:bridget-task:<uuid>`, `creationSource` server, lignée subagent et un enregistrement `bridget_native`, mais sans `bridgetTaskRef`, prend la branche ordinaire (run `starting` + `provider-turn.start`). Ni l'étiquette ni le préfixe ne protègent.
2. Un fil avec `bridgetTaskRef` dont l'enregistrement est ré-étiqueté `app_owned` reste refusé (`ReadOnly`), 0 run, 0 effet.
3. Contrat : l'enum d'origine a exactement trois valeurs ; `BridgetTaskRef` a exactement sept champs, refuse un champ en trop (`onExcessProperty: "error"`) et un `taskId` non UUID.

Sensibilité : deux attentes initiales fausses ont échoué avant correction du test, avec cause diagnostiquée.
- Sous-agent provider_native resté `running` après Stop : le run était `starting`. Stop prend alors la branche « avant démarrage du tour » qui ne cascade pas. Avec un run `running` et un tour fournisseur (session perdue, comme `ThreadStop.test.ts`), la cascade passe le sous-agent à `interrupted`. Le test utilise ce second cas, qui exerce la ligne `Orchestrator.ts:8324` (`=== "provider_native"`).
- Comptage global de tours = 1 : c'était ma propre ligne de tour semée sur le fil racine. Les compteurs sont maintenant par fil.

## 5. Régressions ciblées (après ajout des tests)

| Commande (depuis `apps/server`, Node 24.13.1) | Résultat |
|---|---|
| `vp test run` sur `BridgetPermissions149`, `Orchestrator.bridget149`, `ProjectionStore.bridget149`, `ProviderTurnControlService`, `mcp/BridgetSession`, `ThreadStop`, `ProviderRuntimeRecoveryService` | 7 fichiers, **104 / 104 PASS** |
| `tsc --noEmit` (serveur) | 16 erreurs, toutes hors de mes fichiers (`BridgetRustInterop*`) : identique à la base 16. **0** diagnostic sur les deux fichiers de test |

Avant correction, les seuls échecs observés étaient les deux attentes fausses ci-dessus (G5) et les mutations volontaires (G3).

## 6. Non fait / non vérifié

- `vp lint` n'a pas pu tourner : le plugin oxlint `@oxlint/plugins` manque dans ce worktree. Je n'ai rien installé.
- `vp fmt --check` signale les deux fichiers. Je n'ai pas formaté, pour ne pas réécrire le code d'autres propriétaires. Le formatage précédent des fichiers n'était pas vérifié non plus.
- L'ingestor n'est pas dans le harnais : provider_native est semé par événements de même forme, pas par un vrai appel d'adaptateur.
- La cellule start de provider_native vérifie le refus normal, pas un démarrage (un fil provider-native ne prend aucun message).
- Pas de mutation sur le code de production (écriture interdite). Les mutations G3 passent par le seam `mcpSession`.
- G1, G2, G4, G6, G7, G8 : hors périmètre, non traités. Pas de test global T3, pas de 1796 natif, pas de reconstruction.
- Observation, pas un bug : en recovery, la protection d'un enregistrement bridget_native tient au `runId` nul que `bridget.lineage.sync` écrit toujours. Un enregistrement non canonique avec un `runId` serait traité comme du travail fournisseur. Le producteur réel ne le produit pas.

## 7. Verdict

**APPROVE.** G3 : retrait Codex prouvé sur erreur, fin de tour, fermeture de session, run plus récent et fil voisin, avec mutations qui échouent. G5 : 12 cellules (3 origines x 4 branches) sur branches réelles, trois tests de marqueurs, zéro `ProviderSession`/`Run`/`Turn`/effet pour un fil Bridget virtuel. Aucun bug nouveau. Preuves en attente : lint (outillage absent) et formatage.
