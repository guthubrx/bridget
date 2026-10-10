# Validation r2 — Lot T3 Lineage serveur (tests GLM)

Session : 149-sous-agents-lineage. Lot : L (serveur), reprise r2.
Auteur : sous-agent test du lot serveur. Périmètre : 5 fichiers test + ce rapport + le log types.
Date : 2026-10-10. Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

## 1. Résultat en une ligne

106/106 PASS (T1 clos sur le format synthétique de t3149), 115/115 régressions 148 PASS,
0 erreur TypeScript restante en production 149. Aucune édition production de ma part.
Aucun cochage Git. Aucun commit.

## 2. Fichiers test (ownership exclusif)

| Fichier | Tests | État |
|---|---|---|
| `apps/server/src/bridget/BridgetReader149.test.ts` | 58 | 58 PASS |
| `apps/server/src/bridget/BridgetLineage149.test.ts` | 16 | 16 PASS |
| `apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` | 26 | 26 PASS (T1 clos) |
| `apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts` | 4 | 4 PASS |
| `apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts` | 2 | 2 PASS |

Total : 106. PASS : 106. Aucun rouge.

## 3. Commandes exactes et compteurs

Runner : `./node_modules/.bin/vp test run <fichiers>` depuis la racine du worktree.
Node v24.13.1 (toolchain `/Users/moi/.cache/t3-toolchains/148`, PATH en tête).
Sortie vérifiée par compteurs stdout (`Test Files` / `Tests`), pas par code de sortie.

Lot des 5 fichiers — état r1 + correctifs Sol, avant mon ajustement T1 :

```
Test Files  5 passed (5)   Tests  106 passed (106)   VP_EXIT=0
```

Preuve : `/tmp/vp149-r2-lot.log`.

Régressions 148 (BridgetReader.test, ThreadStop.test, ProjectionStore.test, ThreadLaunchService.test) :

```
Test Files  4 passed (4)   Tests  115 passed (115)   VP_EXIT=0
```

Preuve : `/tmp/vp149-r2-regressions.log`.

Lot relancé après mon ajustement T1 (voir §5) : `Test Files 5 passed (5)` — `Tests 106 passed (106)`, VP_EXIT=0. Preuve : `/tmp/vp149-r2-lot-final.log`.
Régressions relancées sur l'état final de la production (après le correctif BridgetReader de 12:36, §6) : `Test Files 4 passed (4)` — `Tests 115 passed (115)`, VP_EXIT=0. Preuve : `/tmp/vp149-r2-regressions-final.log`.

## 4. Closure t3149 — T1 (sync écrit les fils virtuels exacts)

Le correctif Sol est en place : `Orchestrator.ts:10212` construit
`ProviderInstanceId.make(`bridget-task-${task.task_id}`)` (49 caractères, injectif par UUID de tâche).
T1 reste l'oracle du sync réel complet : aucune passe par `projections.apply`. Assertions ajoutées :

- Format réel : `storedA.providerInstanceId` vaut `bridget-task-<uuid(1)>`, `storedB` vaut `bridget-task-<uuid(2)>`.
- Règle slug réelle : la valeur matche `^[a-zA-Z][a-zA-Z0-9_-]*$` et fait ≤ 64 caractères.
- Unicité injective par UUID : `expectedB ≠ expectedA`.
- `agent_type` arbitraire : la tâche B porte un nom de registre de 210 caractères, long et unicode
  (`agenté-🎉-xxx…`). Le sync reste vert. L'identifiant d'instance n'en dépend plus.
- Modèle natif affiché conservé : `storedA.modelSelection = { instanceId, model: "glm-5.3-flash" }`,
  `storedB.modelSelection = { instanceId, model: "qwen3-coder" }`. Chaque tâche porte son modèle natif.
- Pas de nouveau ProviderRun : après le sync réel, `runs`, `providerThreads` et `turnItems` restent vides pour l'enfant.

Les autres seeds (`seedVirtual`) sont alignés au nouveau format réel (`bridget-task-<UUID>`, par tâche).
Le stand-in `bridget-claude` est supprimé. Aucune attente ownership, guard ou atomicité n'a baissé.
Le contrat lineage.md ne fixe pas ce format. L'assertion porte le format réel choisi, pas l'inverse.

## 5. Closure des 8 erreurs TS production (r1) et de RPC/afterSeq

État vérifié dans le typecheck final : production 149 = 0 erreur.

- `BridgetLineage.ts` : 0 erreur (r1 : 4). Branches read séparées (list → synchronize, autres →
  `lineageRead`), retries `snapshot_changed` toujours 3 (`snapshotAttempt(input, 3)`), erreurs explicites.
- `BridgetReader.ts` : 0 erreur (r1 : 4, pic intermédiaire : 16, voir §6).
- `RpcInstrumentation.ts` : 0 erreur. Les quatre `bridgetLineageRead/Watch/Journal/Cancel` sont dans
  `RPC_AGGREGATES` (L205-208), catégorie existante `bridget`, chaque méthode différenciée.
- Signature `afterSeq` : canonique et optionnelle dans `BridgetLineageJournalInput`. Côté reader :
  `--after-seq` seulement si défini (L443), défaut 0 pour le suivi (L516). Les 58 tests Reader passent.

## 6. Incident de session — correction production BridgetReader pendant mon run (t3149-suite)

Mon premier typecheck (12:36) montrait 16 erreurs `BridgetReader.ts`, racine L367 :
`Schema.UnknownFromJsonString` n'existe pas dans le d.ts d'effect 4.0.1 (TS2551) mais existe à
runtime (fonction, décodage vérifié par sonde Node) — d'où les 106 tests verts et 15 erreurs en
cascade (`any` canal requirements : 7× TS377030 ; service L524 : 3× TS2322, 3× TS377004, 2× TS2719).
À 12:36, pendant ma session, un autre acteur (Sol) a appliqué le fix minimal attendu :
`Schema.decodeEffect(Schema.fromJsonString(Schema.Unknown))` — le pattern déjà utilisé aux L52/L368
du même fichier. Sémantique identique, prouvée par les 58 tests Reader et le lot 106 relancé dessus.
Mon typecheck final (79 erreurs, §7) tourne sur cette production corrigée : BridgetReader = 0.
Je n'ai touché aucune ligne production.

## 7. Typecheck serveur final — preuve baseline et liste restante

```
cd apps/server && ../../node_modules/.bin/tsc --noEmit
TSC_EXIT=1 — 79 erreurs « error TS » (hors suggestions)
Mes 5 fichiers test : 0 erreur.
```

Preuve baseline : typecheck frais du checkout principal propre sur
`local/main-20261009` @ `33f6d04e11` : **16 erreurs** (`/tmp/tsc-main-33f6d04.log`).
Ces 16 existent à l'identique dans le worktree : 15× `mcp/BridgetRustInterop*` (TS377057/63/66/72)
et 1× `provider/Drivers/CodexMcp.ts(22)` TS377030. Je ne les compte pas au débit de 149.

Répartition des 79 restantes :

| Famille | Nombre | Périmètre |
|---|---|---|
| `scripts/lib/*` | 59 | Hors `apps/server`. Absentes de main. Autre lot. |
| baseline main (ci-dessus) | 16 | Préexistantes à 33f6d04. |
| `mcp/BridgetSession.test.ts(48,84)` et `(48,102)` TS2322 (`string \| undefined` non assignable à `string`) | 2 | Lot mcp — signalé au propriétaire, source précise ici, non modifié par moi. |
| `orchestration-v2/Adapters/ClaudeAdapterV2.ts(905)` TS377037 | 1 | Production orchestration, hors 149. Le fichier a été édité pendant la session (r1 : ligne 901, TS377026). |
| `orchestration-v2/ProviderTurnControlService.test.ts(212)` TS2741 | 1 | Test hors 149. |

**Production 149 (bridget lineage + Orchestrator + instrumentation) : 0 erreur.**

Note de traçabilité : le log `t3-lineage-server-types-r2.log` a été réécrit à 12:43 par une
session concurrente (permissions r2 fait aussi un typecheck serveur). Contenu repris : même
compte de 79, mêmes familles. La liste compacte ci-dessus fait foi.

## 8. Aucun bug runtime détecté

Aucun déclencheur/attendu/réel à remonter : les 106 oracles passent sur la production corrigée.
La réserve théorique r1 sur la re-sérialisation à la frontière 16 KiB reste une note de
disponibilité sûre, sans nouveau cycle demandé (F1/F2 24 KiB enveloppe / 16 KiB contenu et les
anciens enfants absents `unavailable` restent approuvés, bornes conservées).

## 9. Limites

1. Branche « identity » du stop natif toujours inatteignable dans le harnais (le layer de base
   résout toujours un `Crypto`) — inchangée depuis r1, documentée dans le fichier test.
2. Le worktree est vivant : trois autres agents r2 actifs. Mes compteurs portent sur l'état
   instantané de mes runs (12:32 à 12:44). Un changement production ultérieur n'est pas couvert.
3. `vp` sort 0 même sur échec selon le launcher : tous les verdicts reposent sur les compteurs
   stdout, jamais sur le code de sortie seul.

## 10. Garanties de mission

- Seuls mes 5 fichiers test ont été édités (commentaire d'en-tête, assertions T1, alignement seed).
- Aucune édition production. Aucun fichier d'un autre agent touché.
- Aucun cochage Git. Aucun commit. Aucun restart. Aucune config/DB de production. Aucun navigateur.
- Aucun `pnpm install`. Aucune copie `node_modules` (symlink du worktree intact).
- `.vite-temp` partagé non nettoyé pendant les agents actifs.
- Aucun succès 149 affirmé au-delà des preuves ci-dessus.
