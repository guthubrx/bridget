# Validation T004–T008 — tests permissions adaptateur de bureau (lot sous-agent, r1)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md#extrait-1).

## 1. Livrables et compteurs

| # | Fichier (ownership exclusif) | Tests | Résultat |
|---|---|---|---|
| 1 | `packages/contracts/src/bridgetPermissions149.test.ts` | 30 | **30/30 PASS** |
| 2 | `packages/provider-core/src/server/mcpSession149.test.ts` | 14 | **14/14 PASS** |
| 3 | `apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts` | 9 | **2/9 PASS** — 7 FAIL intentionnels, cause unique = bug production #3 (section 4) |
| 4 | `apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` | 12 | **12/12 PASS** (A : 9, B : 2, C : 1) |

Consolidation (un seul run, les 4 fichiers) : **65 tests = 58 PASS / 7 FAIL** (les 7 du bug #3 uniquement).

## 2. Commandes exactes et exit codes

`vp test` (vite-plus) retourne toujours 0 dans le shell. Les compteurs viennent du résumé vitest, comme en r1.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md#extrait-2).

## 3. Différenciation 149 / régression

Les 3 échecs de régression ne viennent pas de mon lot :

- `BridgetSession.test.ts > keeps two sessions distinct even when their provider instance is shared` : **même bug #3** (stack identique : `new OrchestratorMcpFailure` contracts `orchestratorMcp.ts:648` ← `failure()` `OrchestratorMcpService.ts:200` ← `OrchestratorMcpService.ts:1788`). Un test pré-existant touche le même throw.
- `ClaudeMcpPreparation.test.ts` × 2 (`prepares Bridget before publishing…`, `closes a failed MCP candidate…`) : `AssertionError: expected 1 to equal 2` — cause distincte, sans rapport avec le schéma. Le worktree porte les sources production 149 **non commitées de Sol** (`git status` : OrchestratorMcpService.ts, ClaudeAdapterV2.ts, CodexAdapterV2.ts, … modifiés). Mes 4 fichiers sont nouveaux et importés par aucun test existant (grep : 0 référence). Ces 2 échecs pré-existent donc à mon lot.

## 4. Bugs production à signaler à Sol (aucun fix production de ma part)

### Bug #3 — code `permission_attestation_unavailable` absent du union `OrchestratorMcpFailure`

- Usage : `OrchestratorMcpService.ts:1788`, `:1800`, `:1811` (chemins de refus de `sessionIdentity`).
- Union : `packages/contracts/src/orchestratorMcp.ts:648+` (`OrchestratorMcpFailure`). 16 littéraux présents : `capability_denied`, `parent_not_active`, `provider_unavailable`, `model_unavailable`, `runtime_mode_escalation_denied`, `interaction_mode_escalation_denied`, `task_not_found`, `task_not_cancellable`, `thread_not_found`, `run_not_found`, `thread_not_sendable`, `thread_not_interruptible`, `invalid_request`, `orchestration_error`, `thread_credential_required`, `target_required`.
- Effet runtime : `new OrchestratorMcpFailure({ code: "permission_attestation_unavailable", … })` → throw `Error: Schema validation failed` (issue path `["code"]`, tag `AnyOf`). Au lieu d'un refus propre, l'appelant reçoit une exception de schéma.
- Fix attendu côté Sol : ajouter le littéral au union (1 ligne). Mes 7 tests refus (fichier 3) passeront dès ce fix — ils assertent `code === "permission_attestation_unavailable"` et le message exact. Aucune autre modification nécessaire.

### TS2322 `bridgetPermissions.ts:87` et `:100` (déjà capturées à la version 9:32)

Toujours présentes, inchangées. Signature de decoder : `(input: unknown, options?: ParseOptions)` incompatible avec `(u: unknown, self: Declaration, options: ParseOptions)` attendu par `Declaration`. 2 lignes, même cause.

## 5. Oracles par groupe (ce que les tests prouvent)

**Fichier 1 (contracts, 30)** : parse fermé v1/v2 ; corrélation stricte driver/kind (`codex_app_server`↔policy codex, `claude_stream_json`↔policy claude, `claude_code` refusé aux deux) ; enums exacts (runtime_mode 4 valeurs, interaction_mode 2, approval_policy 3 chaînes + granular exact) ; `revision` entier dans [1, 2^53−1], refus NaN/2^53/1.5 ; `cwd` absolu borné 4096, refus `~`, relatif, vide, 4097. Union refusée dès qu'une clé inventée ou manquante apparaît.

**Fichier 2 (mcpSession, 14)** : registre privé clé = **objet** credential (deux configs égaux champ à champ = credentials distincts ; même UUID pas suffisant) ; publish/read/clear liés au credential monté ; fact liée à `run_id` + `attemptId` + `providerThreadId` (refus cross-run, reuse avec autre attempt/thread, attemptId null) ; cleanup stale-safe (mauvais run ou mauvaise révision = no-op ; exact = tombe) ; credential étranger ne nettoie pas ; rotation de session redémarre les révisions ; verifier `false` → tombe définitive (jamais re-valide) ; verifier qui throw → `unavailable` sans fuite du message ; re-check post-vérification (credential démonté pendant verify → refus) ; payload hors contrat wire → aucun fait stocké ; aucune fuite endpoint/authorization dans la lecture (clés exactes listées).

**Fichier 3 (service, 9)** : `sessionIdentity` v1 identité seule sans fait valide ; v2 avec les permissions exactes une fois la fait publiée pour le run vivant (projection + re-vérification double) ; sérialisation sans endpoint ni authorization. Les 7 refus (non-monté, session/instance mismatch, fait d'un autre run, tombe, verifier invalide, run changé entre les 2 projections, credential remonté pendant la vérification) attendent le fix #3.

**Fichier 4 (adapters, 12)** :
- A (9) — capture Claude réelle (runner réel, SDK `query` mocké, fichiers réels sous temp home) : digests frais depuis les **bytes réels** du launcher épinglé `~/.local/bin/gclaude` (digest `sha256:dd8dee56…`), priorité PATH (launcher avant CLI), sources absentes listées `absent` (user, managed×2, project, local) ; settings finales d'instance après `applyFlagSettings` (deny préservé) + `permission_callback {kind: t3_runtime, tool_approval, plan_exit: deny}` ; mutation/suppression/restauration de `settings.json` → verify false/false/true ; launcher muté/supprimé, CLI script → verify false ; plugin propre (manifest hashé, hooks/settings/mcp absents) vs manifest avec hooks → `launch_context` absent ; settings avec `hooks` ou `permissions.policyHelper` → inputs undefined ; `canUseTool` undefined, sandbox, managedSettings, hooks, extraArgs sandbox → undefined ; env hostile (BASH_ENV, var non listée, PATH relative) → inputs définis mais `launch_context` absent ; plist utilisateur MDM → `launch_context` absent.
- B (2) — wire réel adapter Claude (`it.live`, horloge réelle) : publish rev1 au retour de `startTurn` (session credential ≠ session runtime, driver `claude_stream_json`, cwd attesté, `provider_policy` deep-equal complet : `permission_mode: bypassPermissions`, tools [], `settings_sources`, callback allow) ; message init CLI (`acceptEdits`) → rev2, status `plan` → rev3 ; fin de stream → tombe `unavailable`, jamais downgrade. B2 : credential d'une autre instance → `unavailable` (tombe rev 0 par `invalidate` sans fait — fail-closed documenté, `mcpSession.ts:104-110`).
- C (1) — wire réel adapter Codex via replay app-server : la fact porte les **paramètres FINAUX** de `turn/start` (`approval_policy: never`, `approvals_reviewer: user`, `sandbox_policy: dangerFullAccess`), pas les défauts de session `thread/start` (`on-request`/`user`/`workspaceWrite`) ; `provider_session_id` = credential, ≠ runtime ; driver `codex_app_server` ; deep-equal complet de la fact.

## 6. Découvertes wire partie C (utiles aux prochains lots)

La partie C est le premier test qui combine **credential monté + `startTurn`** sur Codex. Le wire réel est :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md#extrait-4).

## 7. Corrections limitées à MES harnais (aucune source production)

- effect v4 : `Effect.fn(...)()` non itérable (plain `Effect.gen`), `makeDirectory` (pas `mkdir`), pas de `zipRight` data-last (`Effect.ensuring`), `it.effect` = TestClock (→ `it.live` pour temps réel), `Effect.Success<T>` export module, `Crypto.digest("SHA-256", bytes)` (règle `nodeBuiltinImport`), `Effect.die` pour les timeouts de harnais (règle `globalErrorInEffectFailure`).
- Widening TS dans les factories `fact()` (→ `as const` sur `approval_policy`/`approvals_reviewer`/`sandbox_policy`) ; `permissions()` driver élargi à `string` (le cas négatif `"claude_code"` est hors union par conception) ; `withCleanup <A,E,R>` / `expectAttestationRefusal <A,E>` / `withFixture <E,R>` génériques (sinon `unknown` résiduel dans les channels) ; `liveThreadShell(ThreadId.make(...))` (brand) ; deep-equal complet du policy aux révisions 2/3 (type-sûr face à l'union codex|claude, plus fort qu'un accès de propriété).

## 8. Limites et écarts

- **MDM** : le plist système `/Library/Managed Preferences/com.anthropic.claudecode.plist` est ABSENT sur cette machine. Seul le cas plist **utilisateur** est testé. `managed-settings.json` système présent — non asserté « absent » (les fixtures isolent `CLAUDE_CONFIG_DIR`).
- **Launcher épinglé** : les bytes réels de `~/.local/bin/gclaude` sont relus à chaque run (digest `sha256:dd8dee56…`). Un changement du CLI fait échouer A1 bruyamment — voulu (pas de fixture périmée), mais le test est machine-dépendant par construction.
- **Tombe rev 0** : `invalidateMcpProviderPermissions` sans fait existante crée une tombe (`mcpSession.ts:109`). Comportement fail-closed volontaire, non documenté dans le contrat — signalé ici pour le contrat permissions.
- **S149-29/30/31** (tests natifs) : autre lot, hors périmètre. Ici : introspection + inputs uniquement.
- Le mock SDK expose `bridget_thread`, `bridget_capabilities`, `bridget_task_status` (les 2 derniers = `CLAUDE_BRIDGET_READ_ONLY_TOOLS`, `ClaudeMcp.ts:24-27`) — sinon `catalogue_unavailable` (`ClaudeMcp.ts:221`).

## 9. Verdict

- **58/65** sur mon lot. Les **7 rouges sont intentionnels** : ils verrouillent le contrat de refus et passeront dès que Sol ajoute `permission_attestation_unavailable` au union `OrchestratorMcpFailure` (1 ligne, contracts).
- 2 TS2322 production (`bridgetPermissions.ts:87/100`) inchangées — déjà signalées, toujours présentes.
- Régression : 55/58. Les 3 échecs pré-existent à mon lot (1 = même bug #3 sur un test pré-existant ; 2 = `ClaudeMcpPreparation`, travail en cours de Sol, cause `expected 1 to equal 2` — à traiter par Sol).
- Aucune source production modifiée. Aucun fix d'oracle accommodant le code : chaque correction de harnais s'appuie sur une donnée empirique (sonde FileSystem, lectures de schémas/normalizer, frames reçues du replay).
