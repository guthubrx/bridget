# Revue de code T3 — Permissions héritées (T004–T008) — r1

Date : 2026-10-10
Réviseur : sous-agent de revue indépendante (GLM 5.3 Flash)
Périmètre : code T3 T004–T008, contre `contracts/permissions.md` (contrat normatif permissions-149), plan r2 APPROVE (`validation/plan-permissions-r2.md`), data-model, ADR 149, ADR 017.
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
Baseline T3 : `33f6d04e116430bf7f0011902d6af3868163f2d1` (tout le delta T004–T008 est non committé, relu par `git diff`).

---

## Verdict

**APPROVE** — code T3 T004–T008 uniquement.

Les obligations contractuelles sont implantées dans le code, pas seulement décrites. Aucune violation bloquante trouvée. Deux constats mineurs de durcissement et des observations non bloquantes sont listés plus bas. Le Rust natif (T009+, Sol) est hors périmètre. Je ne confirme aucune exécution de test ni preuve runtime : ce n'est pas ma mission.

---

## Méthode

- Lecture seule. `git diff` ciblé contre la baseline, puis lectures de fichiers précis. Pas de build, pas de lint, pas de test, pas de mutation Git, pas de patch.
- Schéma du wire Codex vérifié dans `packages/effect-codex-app-server/src/_generated/schema.gen.ts` (source générée, pas supposée).
- Surface `Options` du SDK vérifiée dans `@anthropic-ai/claude-agent-sdk@0.3.276` (`sdk.d.ts`).
- Les deltas des fichiers hors périmètre (Orchestrator, ProjectionStore, ws, server, runtimeLayer, ThreadLaunchService, RpcAuthorization, client-runtimes) ont été balayés : zéro référence aux permissions, tout est Lineage.

---

## Vérifications par tâche

### T004 — Contrat `bridgetPermissions.ts`

| Exigence contrat | Preuve | Statut |
|---|---|---|
| Objets fermés | `declareConstructor` + `decodeUnknownEffect(codec, { onExcessProperty: "error" })` (bridgetPermissions.ts:86-88, 99-101). Motif maison déjà présent dans `bridget.ts`. | ✓ |
| Bornes | texte ≤256/512/2048/4096 octets, non vide, sans caractères de contrôle (lignes 5-13) ; révision 1..2^53−1 (ligne 79) ; règles ≤128×512 (lignes 12) ; ≤128 chemins (13) ; ≤32 sources (40) ; enveloppe ≤64 KiB (97) | ✓ |
| Corrélation v2 | `providerSessionId === provider_session_id` et `providerInstanceId === provider_instance_id` (lignes 94-97) | ✓ |
| Union v1/v2 | v1 identité seule (91), v2 avec `permissions` non nullable (91-93) | ✓ |
| Driver fermé | `codex_app_server` ↔ `CodexPolicy`, `claude_stream_json` ↔ policy Claude (82-85) | ✓ |
| Formes wire exactes | granular snake_case (`mcp_elicitations`, `request_permissions`, `rules`, `sandbox_approval`, `skill_approval`), sandbox camelCase — identiques à `schema.gen.ts` généré | ✓ |
| Refus `disableBypassPermissionsMode` | littéral unique `"disable"` (28) | ✓ |
| Champs facultatifs | `request_permissions`/`skill_approval` optionnels, jamais remplis par défaut inventé (61-62) | ✓ |

### T005 — Registre privé `mcpSession.ts`

| Exigence | Preuve | Statut |
|---|---|---|
| Fait lié au credential, au run, à l'attempt, au fil provider | `readMcpProviderPermissions` refuse si owner≠config, runId/attemptId/providerThreadId différents, ou permissions absentes (mcpSession.ts:65-69) | ✓ |
| Révision monotone | `(previous?.owner===config ? previous.revision : 0)+1`, décodage strict à la publication (83+) | ✓ |
| Fait invalide → empoisonné, jamais repli v1 | entrée stockée sans `permissions` → lecture rend `unavailable` (69) | ✓ |
| Tombe anti-réemploi | `invalidateMcpProviderPermissions` refuse d'écraser un autre owner/runId ; CAS `expectedRevision` (104+) | ✓ |
| Rotation credential | `setMcpProviderSession` supprime les faits quand l'objet config diffère (112+) | ✓ |
| TOCTOU | `verify()` sous try/catch, re-vérification des liens après `await` (74-78) | ✓ |

### T006 — Codex `CodexAdapterV2.ts`

| Exigence | Preuve | Statut |
|---|---|---|
| Credential par objet, pas par identifiant runtime | map `bridgetCredentialsByThread` par objet session, gardé si `stillCurrent` (1805, 1835-1836) | ✓ |
| Pas d'emprunt inter-fils | lookup par threadId ; objet session unique par fil | ✓ |
| Paramètres finaux réels | publication depuis `turnStartParams.approvalPolicy / approvalsReviewer / sandboxPolicy / cwd` (6263-6274) ; formes vérifiées égales au wire généré | ✓ |
| Publication avant envoi | `publishMcpProviderPermissions` puis `client.request("turn/start")` (6264 → 6276) | ✓ |
| Échec → pas de fait mensonger | paramètre absent → `cwd ?? ""` ou `!` → décodage échoue → fait sans permissions → refus. Direction conservatrice | ✓ |
| Retraite stale-safe | CAS sur `fact.revision` (1859-1861) ; à la finalisation racine `subagent === null` (5779), sur `Effect.onError` (6284), finaliseur de portée (1865) | ✓ |
| Invalidation pré-envoi même run | (6274) avec garde runId, jamais un fait d'un autre run | ✓ |
| Confinement OS réel | sandbox_policy publié est celui des params finaux ; la garde OS est native (T009+, hors périmètre) — le fait publié ne la prétend pas | ✓ |

### T007 — Claude `ClaudeAdapterV2.ts`

| Exigence | Preuve | Statut |
|---|---|---|
| Credential capturé au montage réel MCP | `bridgetCredential` dans `ClaudeAgentSdkQueryOpenInput` (341, 346), capturé à l'openQuery via `readMcpProviderSession` (7569, 7672) ; jamais fourni par un outil | ✓ |
| Garde du runner par objet | `mcpSession !== undefined && readMcpProviderSession(input.threadId) === mcpSession` (864-870) — plus d'égalité sur identifiant rotatif | ✓ |
| Overrides après `applyFlagSettings` réussie | `mounted` exigé + `appliedPermissionSettings = claudeSanitarySettings(readOnlySettings)` (888-889) ; échec → `permissionsRepresented = false` | ✓ |
| Conditions de représentation | `canUseTool` défini, `sandbox` absent, `managedSettings` absent, flags autorisés (850-851, 549+) | ✓ |
| Sources inchangées entre captures | comparaison `sourceSelection` initiale/finale, `settings_overrides` écartée de la comparaison (897) ; écart → pas de fait | ✓ |
| Jamais d'adoption au retry | `verify` compare un instantané frais au figé ; jamais adopte (899+) ; re-vérification avant publication | ✓ |
| Publication avant offer | `Ref.set(activeTurn)` → `publishBridgetPermissions` → `query.offer` (7883 → 7884 → 7900) | ✓ |
| Mode réel et mutable | `permission_mode` depuis `live.permissionMode`, mis à jour par init/status, re-publication à chaque changement (7740) ; réconciliation au réemploi (7600-7602) | ✓ |
| launch_context : launcher + CLI réels | realpath + SHA256 des deux binaires (484, 494) ; hash du launcher connu `dd8dee56…a8e` + ligne exec, pas une chaîne arbitraire | ✓ |
| Sources = hash de fichier complet | `claudePermissionSourceRevision` : lstat, refus symlink, répertoire trié récursif, fichier ≤4 MiB hash complet ; refus policyHelper/processWrapper dans les JSON | ✓ |
| Chaîne plugins gelée | manifeste sans hooks/settings/permissions/mcpServers/lspServers ; `hooks/hooks.json`, `settings.json`, `.mcp.json` absents sous le chemin d'installation | ✓ |
| MDM opaque → refus limité | plist MDM présent → `launch_context` entier absent, le fait reste publiée sans `launch_context` — refus limité aux mappings dépendants, pas de fausse positive same-family | ✓ |
| Hygiène des secrets | publications = chemins + digests + règles ; jamais contenu, env, PATH complet, token, en-tête d'autorisation | ✓ |
| Environnement de lancement final | copie env figée à l'openQuery, `BASH_ENV/ENV/BASH_FUNC_*` et `CLAUDE_*` non liste blanche → refus | ✓ |
| Retraite | inconditionnelle à la finalisation, `Effect.onError`, boucle closeSession, invalidation pré-envoi même runId stale | ✓ |

### T008 — Introspection `OrchestratorMcpService.ts` (section identité) + `tools.ts`

| Exigence | Preuve | Statut |
|---|---|---|
| Appelant vivant, ré-vérifié après I/O | `assertLiveCaller` avant et après, relecture projection, égalité run/attempt/providerThreadId/credential | ✓ |
| Credential lié au scope authentifié | refus si `credential.environmentId !== scope.environmentId` ou `providerSessionId !== scope.thread.providerSessionId` ou instance différente → `permission_attestation_unavailable` | ✓ |
| Cohérence de la corrélation | le `providerSessionId` du credential EST l'UUID porté par le scope du token MCP (`McpSessionRegistry.issue` frappe un UUID v4 frais par credential). L'égalité exige donc le credential réellement monté. Une rotation produit un refus nommé, jamais un faux v1 | ✓ |
| v1 seulement si aucun fait n'a jamais existé | `absent` → v1 ; `unavailable` ou `valid` → v2, jamais v1 (tombe anti-downgrade) | ✓ |
| Décodage strict en sortie | `decodeUnknownEffect(BridgetSessionIdentity)` → refus nommé en échec | ✓ |
| Pas d'argument forgeable | `BridgetSessionTool` : `parameters: Tool.EmptyParams`, description mise à jour | ✓ |
| Inventaire des refus | codes nommés uniquement, aucun code inventé | ✓ |

---

## Constats

### C-01 (Mineur — durcissement) — Denylist au lieu d'allowlist pour les clés settings de premier niveau

- Emplacement : `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:535` (`claudeSanitarySettings`).
- Constat : la fonction refuse `sandbox`, `env`, `hooks`, `policyHelper`, `policyHelpers`. Elle ignore silencieusement les autres clés de premier niveau. Le SDK 0.3.276 expose des clés élargissantes dans `Settings` : `allowedMcpServers` (sdk.d.ts:6548), `enableAllProjectMcpServers` (6514). Le filtre « restrictive-only » du SDK ne s'applique qu'à `managedSettings` (sdk.d.ts:2123), pas à `settings` inline. Si un jour une de ces clés arrivait dans `options.settings`, l'enfant la recevrait sans que le fait publié l'atteste. Le contrat demande : les clés de sécurité non représentées rendent le chemin indisponible.
- Oracle de déclenchement : `options.settings = { allowedMcpServers: { ... } }` atteint `claudeSanitarySettings` → aucune exception → `permissionsRepresented` reste vrai → fait publié sans attestation de l'élargissement MCP.
- Portée actuelle : aucune. Les seules settings atteignables viennent de `compileClaudeModelSelection` (`alwaysThinkingEnabled`, `fastMode`, `ultracode`) et de deux clés bénignes. Aucun chemin actif.
- Fix minimal pour Sol : transformer la denylist en allowlist fermée — n'accepter que `permissions` plus une liste explicite de clés bénignes nommées ; toute autre clé de premier niveau lève l'exception. Ajouter aussi `processWrapper` à la refus pour cohérence avec `claudePermissionSourceRevision`.

### C-02 (Mineur — durcissement) — Champs SDK de médiation non couverts par `permissionsRepresented`

- Emplacement : `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:850-851`.
- Constat : la condition ne vérifie pas `permissionPromptToolName`, `permissionPrompts`, `spawnClaudeCodeProcess`, `hooks` dans `input.options`. Aucun chemin T3 ne les pose aujourd'hui (vérifié dans le constructeur des options de requête). Si un chemin futur les introduisait, la médiation réelle changerait sans invalider le fait publié (`tool_approval: "prompt"` resterait annoncé).
- Oracle de déclenchement : `options.permissionPromptToolName = "x"` + `canUseTool` défini → fait publié alors que la médiation ne passe plus par le callback T3.
- Fix minimal pour Sol : exiger `undefined` pour ces quatre champs dans la condition de la ligne 850.

### Observations non bloquantes

1. Refus conservateurs : un CLI résolu qui est un script shebang ferme `launch_context` (ClaudeAdapterV2.ts:494) — direction conservatrice, conforme à l'esprit « un nom de wrapper n'est pas une preuve ». Un `managed-settings.d` hashé même non chargé, et un cwd profond sans `.git` dépassant 32 sources, ferment aussi le chemin. Tout est dans le sens du refus, jamais de l'excès.
2. Coût de `verify()` : chaque lecture v2 fait un parcours FS complet (jusqu'à ~30 sources plus les plugins). Acceptable pour la fraîcheur exigée par le contrat. `initialLaunch` est calculé même quand `permissionsRepresented` est déjà faux — petite dépense d'I/O inutile à chaque openQuery sans callback.
3. La copie d'environnement retenue par la fermeture `verify` reste en mémoire. Elle n'est jamais sérialisée ni publiée — pas de brèche sur le chemin de publication.
4. Codex : une rotation de credential entre `ensureThread` et `startNativeTurn` laisse le fil sans fait → refus conservateur, pas de faux v1.
5. Les `!` sur `approvalPolicy`/`approvalsReviewer`/`sandboxPolicy` et `cwd ?? ""` (CodexAdapterV2.ts:6263+) sont sûrs par construction (les params finaux les fournissent toujours), mais un typage explicite rendrait l'invariant visible.

---

## Axes constitution

- **Complexité** : la structure est justifiée. Les faits privés, les tombes, le CAS et les re-vérifications répondent chacun à une exigence nommée du contrat. Pas de couche superflue.
- **Minimalisme** : les refus conservateurs (C-observations 1) coûtent des fermetures de chemin, pas du code mort. Le fix C-01 supprime même de la logique (denylist → allowlist courte).
- **Responsabilité future** : C-01 et C-02 sont précisément des dettes de périmètre futur (nouvelles clés SDK, nouveaux champs de médiation). Les deux fixes sont d'une ligne à quelques lignes. À traiter par Sol dans le lot courant ou le suivant ; rien ne bloque la suite.

---

## Limites réelles de cette revue

- Aucun `tsc`, build, lint ou test exécuté. La validité à la compilation n'est pas prouvée par cette revue.
- Aucune preuve runtime, aucun appel modèle, aucune preuve standalone. Je n'en annonce pas.
- Le Rust natif T009+ (admission inherit/development côté GP) est en cours chez Sol. Non relu. Les oracles G-P-01/G-P-02 sont post-native : non évaluables ici.
- Les tests permissions T3 ne sont pas encore écrits (lot de l'agent de tests). Je ne les demande pas en double. Un regret : la couverture visée devrait inclure l'oracle C-01 (clé `allowedMcpServers` dans settings inline) et C-02 (champ `permissionPromptToolName`).
- Le registre actif réel (`/Users/moi/.cache/bridget-core/agents.json`, entrée glm : `gclaude`, profil `claude-glm`) et la faute r1 corrigée du plan r2 sont conformes à ce que le code exige (hash launcher connu présent dans le code).

---

## Conclusion

Le code T004–T008 tient le contrat permissions-149 côté T3 : héritage vrai attesté depuis l'état réel du provider, aucune autorité forgeable depuis MCP ou le prompt parent, faits privés par run/attempt/fil provider, liaison par objet credential, union v1/v2 avec tombes anti-downgrade, refus nommés conservateurs, hygiène des secrets respectée sur tout le chemin de publication.

**Verdict : APPROVE.** C-01 et C-02 sont des durcissements à planifier, pas des bloquants. Aucun changement de contrat n'est requis : le contrat est déjà plus strict que le code sur ces deux points, et c'est le code qui doit rejoindre le contrat, pas l'inverse.
