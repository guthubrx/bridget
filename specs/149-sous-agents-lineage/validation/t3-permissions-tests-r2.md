# Validation T004–T008 — tests permissions T3 (lot sous-agent, r2)

Date : 2026-10-10
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` (branche `session-149-sous-agents-lineage`, HEAD `33f6d04e11` + changements non commités de Sol)
Baseline propre : checkout principal `/Users/moi/11.Repositories/t3code-local` à `33f6d04e11`, statut git propre (aucun changement non commité).
Périmètre r2 : reprise après fixes Sol (union `permission_attestation_unavailable`, wrappers `decodeConstructor`, durcissement C01/C02). Sources production **read-only**. Preuves baseline par exécution réelle sur le checkout propre — pas sur l'argument « tests non importés ».

## 1. Compteurs

| # | Fichier (ownership exclusif) | r1 | r2 |
|---|---|---|---|
| 1 | `packages/contracts/src/bridgetPermissions149.test.ts` | 30/30 | **30/30 PASS** |
| 2 | `packages/provider-core/src/server/mcpSession149.test.ts` | 14/14 | **14/14 PASS** |
| 3 | `apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts` | 2/9 | **9/9 PASS** (les 7 refus sont verts) |
| 4 | `apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` | 12/12 | **14/14 PASS** (+2 cas C01/C02) |

Consolidation (un run, les 4 fichiers) : **67 tests = 67 PASS / 0 FAIL**.

## 2. Fixes Sol vérifiés au runtime (aucun oracle modifié)

- **Union #3** : littéral `permission_attestation_unavailable` présent (`packages/contracts/src/orchestratorMcp.ts:654`). Les 7 tests refus du fichier 3 passent : le service rend maintenant le refus typé attendu au lieu du throw « Schema validation failed ». Fichier 3 **intouché** entre r1 et r2.
- **Wrappers decodeConstructor** : `bridgetPermissions.ts` — `(input) => decode(input)` sur `BridgetPermissions` et `BridgetSessionIdentity`, `onExcessProperty: "error"` conservé. Les 2 TS2322 (`:87`/`:100`) sont **guéries** : typecheck contracts EXIT=0.
- **C01/C02 (ClaudeAdapterV2)** : allowlist en ligne 540 = exactement les 6 clés `permissions, alwaysThinkingEnabled, fastMode, ultracode, autoCompactWindow, showThinkingSummaries` ; sous-clés permissions fermées (ligne ~545) ; guards ligne 852-853 (`canUseTool` défini, `sandbox`, `managedSettings`, `permissionPromptToolName`, `permissionPrompts`, `spawnClaudeCodeProcess`, `hooks` undefined) ; `processWrapper`/`policyHelper` refusés dans les guards fichiers (l.392/419/451).

Nouveaux cas ajoutés au **fichier 4 uniquement** (harnais réel, fixture filesystem temporaire) :

1. `does not represent an unrepresented permission surface` — **étendu** : `permissionPromptToolName: "ask"`, `permissionPrompts: "none"`, `spawnClaudeCodeProcess: fn` → `bridgetPermissionInputs` undefined (guards C02 restants).
2. `refuses settings keys outside the represented allowlist` — **nouveau** : settings avec `allowedMcpServers`, `enableAllProjectMcpServers`, `processWrapper` → undefined (C01, clés nommées dans la revue APPROVE).
3. `still represents settings limited to the represented keys` — **nouveau** : contrôle positif `alwaysThinkingEnabled: true` → inputs représentés, `launch_context` défini, `settings_overrides.permissions` = allow enrichie des 2 outils bridget + deny préservé. Prouve que l'allowlist est exacte : clé dedans = OK, clé dehors = refus.

## 3. Régression r1 (4 fichiers existants, worktree avec fixes Sol)

`vp test run packages/contracts/src/bridget.test.ts packages/provider-core/src/server/mcpSession.test.ts apps/server/src/mcp/BridgetSession.test.ts apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts`
→ **Test Files 2 failed | 2 passed (4) — Tests 3 failed | 55 passed (58)**.

### 3.1 `ClaudeMcpPreparation.test.ts` ×2 — pré-existants, PROUVÉS par baseline réelle

Exécution isolée sur le checkout propre 33f6d04 (`--exclude "**/.worktrees/**"` pour éviter la copie worktree imbriquée, cf. §5) :

- `prepares Bridget before publishing a V2 query and preserves T3 MCP` → **ROUGE à la baseline**, `AssertionError: expected 1 to equal 2` à `ClaudeMcpPreparation.test.ts:74:12` — signature et ligne **identiques** au worktree.
- `closes a failed MCP candidate without publishing it` → **ROUGE à la baseline**, même dump d'objet session qu'au worktree.

Conclusion : les 2 échecs pré-existent au commit de base `33f6d04`. Ils ne sont causés **ni** par le lot de tests 149, **ni** par les changements production non commités de Sol. Ils restent hors mon ownership — à traiter par le lot preparation/Sol. (L'assertion en échec attend 2 appels `mcpServerStatus`, n'en reçoit qu'1.)

### 3.2 `BridgetSession.test.ts` — vert à la baseline, rouge au worktree : régression **intentionnelle** du design 149

- Baseline 33f6d04 : `keeps two sessions distinct even when their provider instance is shared` → **VERT** (5/5 tests du fichier passent).
- Worktree : ROUGE avec `OrchestratorMcpFailure: The current provider credential does not own this attestation.` (`OrchestratorMcpService.ts:1788`).

Cause : le test pré-existant (fichier **inchangé** vs baseline, `git diff` vide) appelle `sessionIdentity` **sans credential monté** et attend l'enveloppe v1 inconditionnelle — la sémantique 148. Le code 149 (non commité) exige un credential monté + run vivant ; sans credential → refus contractuel `permission_attestation_unavailable`. Le contrat (G-P-07) donne la v1 seulement pour un credential **monté** sans fait jamais existée ; la branche « non monté » est le refus — comportement verrouillé par mes tests 149 revus. **Ce n'est pas un bug production** : c'est un test pré-existant périmé face au nouveau contrat.

Pour Sol (je ne touche pas au fichier, ownership limité) :
- Cause : mock sans `setMcpProviderSession` + attente `{version: 1, …}` inconditionnelle (ligne 44-50).
- Reproducteur : `vp test run apps/server/src/mcp/BridgetSession.test.ts` sur le worktree → 1 failed / 4 passed.
- Fix minimal suggéré : monter un credential par thread via `McpProviderSession.setMcpProviderSession({environmentId, threadId, providerSessionId: "session:a"|"session:b", providerInstanceId: codex, …})` avant les appels (le run mocké est déjà actif ; fait absente → v1 rendue), ou réécrire l'attente en refus `permission_attestation_unavailable`. Le même test porte aussi 2 erreurs de typage (§4) réglées par le même changement.

## 4. Typechecks (worktree, après signature fix)

Artefact complet : `validation/t3-permissions-types-r2.log` (97 `error TS` + 157 `suggestion TS`, apps/server).

| Package | Exit | Résultat |
|---|---|---|
| `packages/contracts` | **0** | Les 2 TS2322 `bridgetPermissions.ts:87/100` ont **disparu** (fix wrappers validé). Restent 2 *suggestions* TS377112 pré-existantes (`auth.test.ts`, `baseSchemas.test.ts`) — pas des erreurs. |
| `packages/provider-core` | **0** | Propre. 1 *suggestion* pré-existante (`snapshotProbe.test.ts`). |
| `apps/server` | **1** | 97 erreurs, classées ci-dessous. **0 dans mes 4 fichiers de tests.** Pas de claim « clean » global. |

Baseline apps/server (checkout propre, exécuté 2× pour écarter une course avec les autres agents) : **16 erreurs stables** : `BridgetRustInterop.testkit.ts` ×12, `BridgetRustInterop.test.ts` ×2, `CodexMcp.ts:22` TS377030 ×1, `BridgetRustInteropObserver.test.ts` ×1. Pré-existantes, hors permissions.

Classification des 97 (delta baseline = 81) :

- **Production 149 permissions (non commité Sol) — 1 erreur** : `src/orchestration-v2/Adapters/ClaudeAdapterV2.ts(901,32)` TS377026 `preferSchemaOverJson` — `JSON.stringify(finalLaunch)` pour `launchSnapshot`. Absente de la baseline. Fix minimal suggéré : passer par un encodage Schema (`Schema.toCodecJson`/decoder JSON équivalent) ou une sérialisation typée, même sémantique de snapshot.
- **Test pré-existant cassé par le type 149 — 2 erreurs** : `src/mcp/BridgetSession.test.ts(48,84)` et `(48,102)` TS2322 (`string | undefined` → `string`). Absentes de la baseline ; même racine que le rouge runtime §3.2 (l'objet attendu hérite du nouveau type union `BridgetSessionIdentity`). Un seul et même fix pour Sol.
- **Lot Lineage (autre agent) — 19 erreurs, non patchées** : `src/bridget/BridgetReader.ts` ×16 (dont `:367,56` TS2551 `Schema.UnknownFromJsonString` inexistant dans effect 4.0.1 — vraie erreur d'API), `src/observability/RpcInstrumentation.ts:205/247` ×2 (clés `bridget.lineage.read/journal/watch/cancel` absentes de la map de namespaces), `src/orchestration-v2/ProviderTurnControlService.test.ts(212,46)` ×1 (mock `ProjectionStoreV2Shape` sans `getBridgetTaskThreads`).
- **Artefact environnement worktree — 59 erreurs** : `../../scripts/lib/*` (dev-share ×18+12, cli-external-packages ×8, resolve-catalog ×7+1, build-target-arch ×4+5, cursor-sdk-packaging ×2, icon-export ×1, cli-executable-imports ×1). TS2307 « Cannot find module » (`@t3tools/tailscale`, `effect/Effect`, `typescript-legacy`…) sur des fichiers **identiques à HEAD** et un tsconfig **identique** : la même arborescence typecheck **sans aucune erreur scripts/lib depuis le checkout principal** (re-vérifié 2×). Cause : résolution de modules depuis le worktree imbriqué dont `node_modules` racine est un symlink partagé. Pas une régression 149 — capturée pour transparence, à ne pas traiter comme du code à corriger.
- **Pré-existantes baseline — 16 erreurs** : cf. plus haut (BridgetRustInterop*, CodexMcp).

UI r2 (client/web/mobile) : non dupliqué. Batterie longue : non relancée (aucun défaut nouveau ne la justifie).

## 5. Piège d'exécution découvert (utile aux prochains runs)

Lancer `vp test run apps/server/src/...` depuis le **checkout principal** exécute AUSSI les copies du worktree imbriqué `.worktrees/149-sous-agents-lineage/` (le filtre vitest est un match de sous-chaîne et `.worktrees` n'est pas dans les excludes par défaut) : un même fichier tourne alors 2× sous deux arbres de sources différents. Preuve baseline valide : ajouter `--exclude "**/.worktrees/**"` ou filtrer sur le préfixe volume résolu. Réciproquement, les runs worktree ne voient pas la baseline.

## 6. Commandes exactes

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 1 passed (1) | Tests 14 passed (14)
./node_modules/.bin/vp test run \
  packages/contracts/src/bridgetPermissions149.test.ts \
  packages/provider-core/src/server/mcpSession149.test.ts \
  apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts \
  apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 4 passed (4) | Tests 67 passed (67)
./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts \
  packages/provider-core/src/server/mcpSession.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts \
  apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts
→ Test Files 2 failed | 2 passed (4) | Tests 3 failed | 55 passed (58)
cd packages/contracts && ../../node_modules/.bin/tsc --noEmit   → EXIT=0
cd packages/provider-core && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd apps/server && ../../node_modules/.bin/tsc --noEmit          → EXIT=1 (97, cf. artefact)

cd /Users/moi/11.Repositories/t3code-local   # baseline propre 33f6d04
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts --exclude "**/.worktrees/**"
→ Test Files 1 failed | 1 passed (2) | Tests 2 failed | 7 passed (9)
  (ClaudeMcpPreparation ×2 rouges, BridgetSession 5/5 verts)
cd apps/server && ../../node_modules/.bin/tsc --noEmit → EXIT=1 (16, 0 dans scripts/lib)
```

`vp test` retourne 0 même en échec : les compteurs viennent du résumé vitest ; les exits ci-dessus sont les vrais exits `tsc`.

## 7. Verdict

- **67/67** sur mon lot : les 7 refus sont verts avec le fix union de Sol, sans modification d'oracle ; C01/C02 vérifiés au runtime par 3 cas réels (2 nouveaux + 1 étendu).
- Fix decodeConstructor validé : contracts EXIT=0, TS2322:87/100 guéries.
- Baseline des 2 rouges `ClaudeMcpPreparation` **prouvée par exécution** : pré-existants à `33f6d04`, signature identique (ligne 74:12). Hors mon ownership.
- `BridgetSession` : régression intentionnelle du design 149 (contrat G-P-07) sur un test pré-existant périmé — cause/reproducteur/fix minimal fournis à Sol (§3.2). Aucun bug production permissions détecté en r2.
- Typechecks : contracts et provider-core EXIT=0 ; server EXIT=1 avec 97 erreurs classées (1 production permissions 149, 19 lot Lineage, 59 artefact environnement, 16+2 pré-existantes/stale). Aucune erreur dans mes fichiers.
- Aucune source production modifiée. Modifications de cette ronde : le seul fichier 4 (2 cas + extension d'un cas existant), ce rapport, l'artefact types.
