# Validation r1 — Lot T3 Lineage serveur (tests GLM)

Session : 149-sous-agents-lineage. Lot : L (serveur).
Auteur : sous-agent test du lot serveur. Périmètre : 5 fichiers test + ce rapport.
Date : 2026-10-10. Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

## 1. Résultat en une ligne

106 tests : 105 PASS, 1 rouge documenté (bug production `Orchestrator.ts:10210`).
Régressions 148 : 115 PASS. Aucune édition production. Aucun cochage Git. Aucun commit.

## 2. Fichiers test (ownership exclusif)

| Fichier | Tests | État |
|---|---|---|
| `apps/server/src/bridget/BridgetReader149.test.ts` | 58 | 58 PASS |
| `apps/server/src/bridget/BridgetLineage149.test.ts` | 16 | 16 PASS |
| `apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` | 26 | 25 PASS, 1 rouge (T1) |
| `apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts` | 4 | 4 PASS |
| `apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts` | 2 | 2 PASS |

Total : 106. PASS : 105. Rouge : 1.

## 3. Commandes exactes et compteurs

Runner : `./node_modules/.bin/vp test run <fichiers>` depuis la racine du worktree. Node v26.9.0.

Lot des 5 fichiers :

```
./node_modules/.bin/vp test run \
  apps/server/src/bridget/BridgetReader149.test.ts \
  apps/server/src/bridget/BridgetLineage149.test.ts \
  apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts \
  apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts \
  apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts
```

Sortie finale : `Test Files  1 failed | 4 passed (5)` — `Tests  1 failed | 105 passed (106)`. VP_EXIT=1.
L'unique échec est T1 (voir §4). Preuve : `/tmp/vp149-run-lot3.log`.

Typecheck :

```
cd apps/server && ../../node_modules/.bin/tsc --noEmit
```

TSC_EXIT=1. Erreurs « error TS » hors suggestions : 90 dans le dépôt.
Erreurs dans MES 5 fichiers : 0. Preuve : `/tmp/tsc149-server-4.log`.
Les 90 erreurs sont hors périmètre : `scripts/lib/*` (45), production bridget (8, voir §6),
tests d'autres domaines (mcp, observability, orchestration-v2 non-149, etc.).

Régressions 148 :

```
./node_modules/.bin/vp test run \
  apps/server/src/bridget/BridgetReader.test.ts \
  apps/server/src/orchestration-v2/ThreadStop.test.ts \
  apps/server/src/orchestration-v2/ProjectionStore.test.ts \
  apps/server/src/orchestration-v2/ThreadLaunchService.test.ts
```

`Test Files  4 passed (4)` — `Tests  115 passed (115)`. VP_EXIT=0. Preuve : `/tmp/vp149-regressions148.log`.

## 4. Bug production confirmé — Orchestrator.ts:10210 (t3149)

Code production (`Orchestrator.ts`, fonction `dispatchBridgetLineageSync`) :

```ts
const providerInstanceId = ProviderInstanceId.make(`bridget:${task.agent_type}`)
```

`ProviderInstanceId` porte un filtre slug `^[a-zA-Z][a-zA-Z0-9_-]*$`. Le préfixe `bridget:`
contient `:`. Aucun `agent_type` ne peut rendre la chaîne conforme.

Oracle T1 (« sync writes the exact virtual threads, markers and native subagents ») :

- Expected : `storedEvents.map(e => e.type)` = `["thread.created", "subagent.updated",
  "thread.created", "subagent.updated", "thread.metadata-updated"]`.
- Actual : die `Error: Schema validation failed` (non wrappé). Le sync n'écrit rien.

Stack capturée verbatim (`/tmp/vp149-run-lot3.log`) :

```
Error: Schema validation failed
    at runSync (.../effect/dist/SchemaParser.js:921:9)
    at Schema.make (.../effect/dist/SchemaParser.js:948:12)
    at apps/server/src/orchestration-v2/Orchestrator.ts:10210:53
    at orchestrationV2.dispatch.once (Orchestrator.ts:10660:25)
    at orchestrationV2.dispatch.once (definition) (Orchestrator.ts:10271:31)
    at orchestrationV2.dispatch.withReceipt (Orchestrator.ts:10794:55)
    at orchestrationV2.dispatch.withReceipt (definition) (Orchestrator.ts:10585:44)
[cause]: Issue Filter « a string matching the RegExp ^[a-zA-Z][a-zA-Z0-9_-]*$ » (_tag: InvalidValue)
```

Le contrat fermé `specs/149-sous-agents-lineage/contracts/lineage.md` ne spécifie PAS le
format du `providerInstanceId` des fils natifs. Décision au principal :
soit le préfixe devient slug (`bridget-claude`), soit le filtre change.
Après fix, T1 vérifiera le format réel choisi. Aucun fix fait de mon côté.

## 5. Contournement de test (pas un contournement d'oracle)

Le T1 reste l'oracle exact rouge. Tous les autres tests du lot orchestration-v2 seedent la
projection par la voie publique `projections.apply`, avec exactement les events que le sync
produira (thread.created / subagent.updated / thread.metadata-updated). Seule différence :
l'instance provider utilise le stand-in slug-valide `bridget-claude`, à cause du bug §4.
Cela prouve les guards, stop, marqueurs et reads indépendamment du bug.

## 6. Changements production détectés PENDANT la session (t3149)

À 11:08, `BridgetReader.ts` a changé sous ma session :

- L420 : `journalEnvelopeBytes = 24 * 1024` (était 16 KiB). L440 : show aussi 24 KiB.
- Mon oracle budget (L553) : `expect(calls[0]!.maxOutputBytes).toBe(24 * 1024 + 1024)`,
  avec commentaire « Transport budget only — lineage.md does not fix it ». Le contrat ne
  fixe pas cette constante de transport. Suivi seulement.

Erreurs TS dans la production bridget (rapportées, NON corrigées) :

- `BridgetLineage.ts` (édité 09:29) : TS377030 L34, L44, L48 ; TS2345 L45.
- `BridgetReader.ts` (édité 11:08) : TS377026 L371, L428 (`JSON.parse` → préférer Schema) ;
  TS7022 L399, L400 (`raw`/`page` sans annotation).

Fichier du lot permissions (`OrchestratorMcpService.bridgetPermissions149.test.ts`,
propriétaire GLMpermissions) : édité 11:56 par son propriétaire. Au tsc final de 12:08 il
ne présente plus d'erreur. Rien à signaler.

## 7. Incident de session — régression causée par ma propre édition (transparence)

À 12:06, en corrigeant le typage du mock runner (fichier 1), j'ai déclaré la queue
`const queued = [...(options.results ?? [])]` DANS le corps du callback `run:`. La copie
était recréée à chaque appel. Le mock a servi la première page en boucle. Le reader a vu
un cursor dupliqué et a répondu `invalid_output` (`BridgetReader.ts:408`, branche
« cursors.has(cursor) »). 5 tests du fichier 1 sont passés au rouge.

Diagnostic par données runtime (stack exacte L408), pas par retentative. Fix : la queue est
déclarée une fois au niveau du harness. Fichier seul relancé : 58/58, VP_EXIT=0.
Lot complet relancé après : 105/106 (§3).

## 8. Corrections TypeScript appliquées (fichiers test seulement)

- Fichier 1 : `v: 1 as const` ; queue du mock typée et sortie du callback ; `deletedAt`
  via `DateTime.toDateUtc(DateTime.makeUnsafe(0))` (règle `globalDateInEffect`) ; `calls`
  accepte `maxOutputBytes: number | undefined` (champ optionnel de `ProcessRunRequest`).
- Fichier 2 : `BridgetLineageError["code"]` (le type `BridgetLineageErrorCode` n'existe
  pas, seule la const Schema) ; `(raw: unknown)` sur les stubs ; option `journalPage`
  omise au lieu de `undefined` (`exactOptionalPropertyTypes`) ; import de type nettoyé.
- Fichiers 3, 4, 5 : les helpers prennent les shapes de service
  (`OrchestratorV2Shape`, `ProjectionStoreV2Shape`), pas les Tags. `yield*` sur un Tag
  retourne la shape. Typé sur le Tag, chaque helper exigeait le service en canal R et
  produisait 130+ erreurs en cascade (TS2339/TS2739/TS2345/TS7006/TS2375).
- Reste non bloquant : suggestion TS377019 L139 fichier 2 (`yield* Effect.fail`),
  documentée, laissée telle quelle.

## 9. Limites

1. Branche « identity » du stop natif (`Orchestrator.ts:9062`, « Native request identity
   is unavailable ») inatteignable dans le harnais : le layer de base résout toujours un
   `Crypto`. Seule la branche « transport » (L9064) est testée. Le test a été re-ciblé
   avec ce commentaire en tête.
2. T1 assertera le format réel du `providerInstanceId` après le fix t3149 (§4).
3. Les erreurs TS des scripts (`scripts/lib/*`) et d'autres domaines sont hors périmètre.
   Elles préexistaient ou appartiennent à d'autres lots. Non touchées.

## 10. Garanties de mission

- Oracles issus du contrat fermé lineage.md. Jamais adaptés au code, sauf deux cas
  documentés : budget journal (§6, constante hors contrat) et branche stop re-ciblée (§9).
- Aucune édition production. Aucun fichier d'un autre agent touché.
- Aucun cochage Git. Aucun commit. Aucun restart. Aucune config/DB de production.
- Aucun `pnpm install`. Aucune copie node_modules. Cache `t3-final-148` intact.
- Aucun succès 149 affirmé au-delà des preuves ci-dessus.
