# Clôture fixtures permissions T3 — lot sous-agent, r3

Date : 2026-10-10
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` (branche `session-149-sous-agents-lineage`, HEAD `33f6d04e11` + changements non commités)
Base de comparaison : checkout principal `/Users/moi/11.Repositories/t3code-local` à `33f6d04e11` (voir r2 pour les preuves baseline).
Périmètre r3 : clôture des 2 fixtures rouges (BridgetSession, ProviderTurnControlService), vérification du fix Sol `ClaudeAdapterV2.ts:901`, montage des deps manquantes du worktree. Sources production **read-only** — aucun fichier production touché.

## 1. Compteurs (stdout vitest, font foi)

Run consolidé unique, 6 fichiers :

| Fichier | Tests | Résultat |
|---|---|---|
| `packages/contracts/src/bridgetPermissions149.test.ts` | 30/30 | PASS |
| `packages/provider-core/src/server/mcpSession149.test.ts` | 14/14 | PASS |
| `apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts` | 9/9 | PASS |
| `apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` | 14/14 | PASS |
| `apps/server/src/mcp/BridgetSession.test.ts` | **6/6** | **PASS** (5 originaux + 1 cas unmounted) |
| `apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts` | **1/1** | **PASS** (oracle ProviderTurn inchangé) |

**Total : 67/67 permissions (inchangés) + 6 + 1 = 74/74 PASS, Test Files 6 passed (6).**
Runs isolés préalables : BridgetSession 6/6 (6.35 s), ProviderTurnControlService 1/1 (2.00 s).

## 2. Fixtures écrites par cette ronde (ownership test)

### 2.1 `apps/server/src/mcp/BridgetSession.test.ts`

But original conservé : deux sessions distinctes (`thread:a`/`session:a`, `thread:b`/`session:b`) sur la même instance `codex` rendent deux identités distinctes. Changelog :

- **Montage du credential** : helper `mountCredential(threadId, sessionId)` qui appelle `McpProviderSession.setMcpProviderSession` avec un vrai objet `McpProviderSessionConfig` (endpoint/authorizationHeader factices privés, jamais publiés dans les réponses) avant chaque `sessionIdentity`. Le run mock vivant existait déjà ; aucun fait de permissions n'est jamais publié → la branche v1 du contrat (G-P-07a) est exercée pour de vrai.
- **Assertion de distinction conservée** : les deux `assert.deepEqual` v1 restent, avec valeurs distinctes par thread/session.
- **Nouveau cas de refus** : `refuses a session without a mounted provider credential` — sans montage, `sessionIdentity` échoue avec `permission_attestation_unavailable`. Le test positif v1 reste un test positif ; aucun ancien test positif n'a été converti en refus.
- **Hygiène** : `clearMountedCredentials` via `Effect.ensuring` purge `thread:a` et `thread:b` de la Map statique, même en cas d'échec d'assertion.
- **Types exacts, sans `any`** : itération en `as const` (littéraux), `ThreadId.make` pour le brand, plus aucune assertion non-nulle `!` dans la boucle. Les 2 TS2322 `(48,84)`/`(48,102)` du r2 sont guéries (cf. §5).

Référence utilisée (non modifiée) : `OrchestratorMcpService.bridgetPermissions149.test.ts` et ses helpers `mountedCredential`/`withCleanup`/`expectAttestationRefusal`.

### 2.2 `apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts`

Une seule ligne ajoutée au mock `ProjectionStoreV2.of({...})` :

```ts
getBridgetTaskThreads: () => Effect.succeed([]),
```

Signature réelle (`ProjectionStore.ts:340`) : `(rootThreadId, candidateIds?) => Effect<ReadonlyArray<OrchestrationV2AppThread>, ProjectionStoreV2Error>` — `Effect.succeed([])` est assignable. La lecture vide décrit la fixture 148 : aucune tâche native. Placement après `searchThreadStream`, à l'image de la shape. Aucun oracle ProviderTurn changé.

## 3. Fix Sol vérifié : `ClaudeAdapterV2.ts:901` (lecture seule, non re-patché)

Le brief demandait de lire la source avant exécution. État constaté : Sol a déjà posé le remplacement de `JSON.stringify(finalLaunch)` par `Schema.encodeSync(Schema.fromJsonString(Schema.Unknown))(finalLaunch)` avec `catch { permissionsRepresented = false; }`.

Vérification de l'API sur la déclaration réelle effect `4.0.1_patch_hash=a33cba07…` :

- `Schema.fromJsonString` : **existe**, `dist/dts/Schema.d.ts` ligne 6876 (`export declare function fromJsonString<S extends Constraint>`).
- `Schema.encodeSync` : **existe**, même fichier ligne 1820.
- `Schema.UnknownFromJsonString` : **0 occurrence** dans la déclaration — l'API citée par le précédent essai Sol n'existe bien nulle part.

Comportement de la chaîne : wrapper absent → `launchSnapshot` reste `undefined`, `bridgetPermissionInputs` reste construit sans `launch_context` ; échec d'encodage → `permissionsRepresented = false` → `bridgetPermissionInputs` undefined. Aucun contenu d'attestation ne fuit. Le typecheck apps/server (§5) ne contient plus aucune erreur sur `ClaudeAdapterV2.ts` : le TS377026 `(901,32)` du r2 est guéri. **Aucun patch production nécessaire ni appliqué.**

## 4. Artefact environnement : deps `scripts/lib` résolues par symlinks

Diagnostic (r2 §4 : 59 erreurs TS2307 + cascade sur `../../scripts/lib/*`) :

- `WT/node_modules` est déjà un symlink → `main/node_modules` (posé le 10-10 09:35 ; je ne l'ai pas touché).
- `effect` n'existe pas à la racine `node_modules` : layout pnpm pur, tout passe par `.pnpm/…` et les `node_modules` locaux par package.
- La remontée de résolution d'un fichier `scripts/lib/*.ts` passe par **`scripts/node_modules`**. Ce répertoire **existe dans main** (10 entrées, liens pnpm relatifs) et **n'existait pas dans le WT**. C'est la seule différence : fichiers et tsconfig identiques à HEAD.

Montage exécuté : création de `WT/scripts/node_modules` + réplication 1:1 des **13 liens** de `main/scripts/node_modules`, mêmes cibles relatives, chacune vérifiée existante avant création :

```
effect, pngjs, sharp, typescript, typescript-legacy, vite-plus
        -> ../../node_modules/.pnpm/<version>/node_modules/<pkg>
@effect/platform-node, @effect/vitest, @electron/asar, @electron/osx-sign,
@types/pngjs           -> ../../../node_modules/.pnpm/<version>/node_modules/…
@t3tools/shared        -> ../../../packages/shared   (résout dans le WT, HEAD identique)
@t3tools/tailscale     -> ../../../packages/tailscale
```

- 13/13 liens résolvent depuis le WT (vérification `-e` après création).
- Aucune modification du checkout principal (`git status scripts/` vide). Aucun lien d'un autre agent supprimé. Aucun code/tsconfig/packageJson/npm install/copie.
- `.bin` non répliqué : les exécutables utilisés (`../../node_modules/.bin/vp`, `tsc`) viennent du `node_modules` racine partagé ; le typecheck ne résout pas de binaire via `scripts/node_modules/.bin`.
- `scripts/node_modules` est gitignore : zéro bruit dans `git status`.
- Résultat : les 59 erreurs `scripts/lib` du r2 ont disparu du log r3.

## 5. Typechecks (log complet : `t3-permissions-types-r3.log`)

| Package | Exit | Résultat |
|---|---|---|
| `packages/contracts` | **0** | Propre. |
| `packages/provider-core` | **0** | Propre. |
| `apps/server` | **1** | **16 erreurs** — cf. ci-dessous. Pas de claim « clean ». |

Delta vs r2 (97 erreurs) : **97 → 16**.

- 19 Lineage (`BridgetReader.ts` ×16, `RpcInstrumentation.ts` ×2, `ProviderTurnControlService.test.ts` ×1) : **plus aucune** — corrigées par l'agent Lineage (et ma ligne de mock pour la 19e).
- 1 production permissions (`ClaudeAdapterV2.ts(901,32)` TS377026) : **guérie** par le fix Sol (§3).
- 2 `BridgetSession.test.ts` TS2322 : **guéries** par ma fixture (§2.1).
- 59 artefact environnement `scripts/lib` : **guéries** par le montage §4.
- **16 restantes = baseline r2 exacte**, fichier par fichier : `BridgetRustInterop.testkit.ts` ×12, `BridgetRustInterop.test.ts` ×2, `BridgetRustInteropObserver.test.ts` ×1, `CodexMcp.ts` ×1. Pré-existantes à 33f6d04, hors permissions, hors scope 149.

Les autres mentions (`BridgetSession`, `ClaudeAdapterV2.test`, `BridgetReader`…) dans le log sont des *suggestions* TS377xxx, pas des erreurs.

## 6. Limites baseline et non-repetition

- **`ClaudeMcpPreparation.test.ts` ×2 rouges** : prouvés pré-existants à `33f6d04` par exécution au r2 (§3.1, `--exclude "**/.worktrees/**"`), signature identique `ClaudeMcpPreparation.test.ts:74:12`. Hors 149 — à traiter par le lot preparation/Sol. Non re-exécutés en r3 (aucun changement ne les affecte).
- Batterie large baseline (53 permissions restantes déjà vertes au r2 en consolidation) : non re-tournée en r3 hors le run consolidé §1, qui inclut déjà les 4 fichiers permissions complets.
- UI r2 (client/web/mobile), tests Rust : non relancés — aucun défaut nouveau ne le justifie.
- Les 2 rouges baseline restent **hors mon ownership** : je ne les corrige pas, même si l'exécution reste triviale.

## 7. Commandes exactes

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run apps/server/src/mcp/BridgetSession.test.ts
→ Test Files 1 passed (1) | Tests 6 passed (6)
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts
→ Test Files 1 passed (1) | Tests 1 passed (1)
./node_modules/.bin/vp test run \
  packages/contracts/src/bridgetPermissions149.test.ts \
  packages/provider-core/src/server/mcpSession149.test.ts \
  apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts \
  apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts \
  apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts
→ Test Files 6 passed (6) | Tests 74 passed (74)
cd packages/contracts    && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd packages/provider-core && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd apps/server           && ../../node_modules/.bin/tsc --noEmit → EXIT=1 (16, artefact t3-permissions-types-r3.log)
```

`vp test` retourne 0 même en échec : seuls les compteurs stdout font foi. Tous les runs sont lancés **depuis le worktree** ; la baseline n'est donc jamais exécutée en double (piège r2 §5).

## 8. Verdict

- Les 2 fixtures rouges du r2 sont fermées : BridgetSession 6/6 (but original + cas unmounted), ProviderTurnControlService 1/1. **74/74** sur le run consolidé.
- Fix Sol `ClaudeAdapterV2.ts:901` validé : API réelle vérifiée dans la déclaration effect (`fromJsonString` l.6876, `encodeSync` l.1820, `UnknownFromJsonString` absente), typecheck guéri, absence/échec → `permissionsRepresented=false` sans fuite. Production intacte.
- 59 erreurs environnement résolues **uniquement** par 13 symlinks vérifiés vers des packages existants ; main inchangé ; aucun lien d'autrui touché ; chemin gitignoré.
- Typechecks : contracts 0, provider-core 0, server EXIT=1 avec exactement les 16 baseline r2. **Serveur pas « clean » : EXIT=1**, mais 0 erreur attributable au lot 149 permissions, à Lineage ou aux fixtures.
- Bug concret restant pour Sol : les 2 rouges `ClaudeMcpPreparation` (baseline `33f6d04`, `expected 1 to equal 2` à `:74:12`, 2 appels `mcpServerStatus` attendus contre 1 reçus) et, hors scope, les 16 TS baseline (`BridgetRustInterop*`, `CodexMcp.ts:22`).
- Modifications de cette ronde : `BridgetSession.test.ts`, `ProviderTurnControlService.test.ts` (+1 ligne), `WT/scripts/node_modules` (montage gitignore), ce rapport, le log typage. Tous les edits des autres agents sont préservés.
