# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/plan-permissions-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/plan-permissions-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10. Revueuse : GLM 5.3 Flash, indépendante.
Périmètre : `contracts/permissions.md`, `plan.md`, `data-model.md`, `test-strategy.md`,
`tasks.md`, `spec.md`, `research-native.md`, `research-t3.md`, ADR149.
Verdict demandé : APPROVE ou REQUEST_CHANGES sur le volet G-P.

<a id="extrait-2"></a>

## Extrait 2

| Fait vérifié | Source précise | Résultat |
|---|---|---|
| Parent externe borné à `Discovery` | `crates/bridget-daemon/src/daemon/native_delegation.rs:8-30` | Confirmé : `SameProject` → maximum `Discovery`. |
| Héritage development 148 par signature Codex exacte | `native_delegation.rs:70-86` | Confirmé : workspace-write sans réseau, sans racines, tmp exclus. |
| Refus `development_grant_required` pour un parent lecteur | `native_delegation.rs:205-210` | Confirmé. Code 148 à remplacer en 149, comme prévu. |
| Development réservé à `codex_app_server` | `crates/bridget-daemon/src/registry.rs:192-216` | Confirmé. |
| `permissions=allow` pose un bypass global | `crates/bridget-daemon/src/wrapper.rs:4787-4795` | Confirmé. La garde 149 « jamais bypass universel » vise le bon code. |
| `bridget_session` v1 atteste l'identité seulement, parseur fermé | `crates/bridget-daemon/src/t3code_mcp.rs:12-20`, `apps/server/src/mcp/OrchestratorMcpService.ts:1776-1789` | Confirmé : `version:1`, 4 champs camelCase, `deny_unknown_fields`. |
| `posture` obligatoire aujourd'hui | `crates/bridget-daemon/src/delegation_mcp.rs:9,55` | Confirmé. Le rend facultatif est bien un delta 149. |
| Le pilote Claude natif ignore `can_use_tool` | `crates/bridget-transport/src/claude_stream_json.rs` (grep : zéro occurrence ; seul l'accusé d'interruption est traité vers 1049-1075) | Confirmé. Le refus corrélé 149 est un comportement nouveau. |
| Le profil est épinglable par définition | `registry.rs:61-64` (`claude_config_dir` injecté, non secret) | Confirmé : la voie same-family peut garantir le même `config_dir`. |
| Source Codex = paramètres réels de `turn/start` | `CodexAdapterV2.ts:6220` (`buildCodexTurnStartParams`) | Confirmé. |
| Source Claude = queryOptions finales, mode mutable | `ClaudeAdapterV2.ts:7290-7395` (`makeClaudeQueryOptions`, `setPermissionMode`, réutilisation du contexte vivant) | Confirmé. `plan.md:64` décrit le vrai code. |
| Champs d'approbation granulaire Codex réels | `packages/effect-codex-app-server/src/_generated/schema.gen.ts:129-146` | Confirmé : `mcp_elicitations`, `request_permissions?`, `sandbox_approval`, `skill_approval?`. Union fidèle, rien d'inventé. |
| `approvals_reviewer` user/auto_review/guardian_subagent réels | `schema.gen.ts:153-160`, `CodexAdapterV2.ts:710` | Confirmé. T3 pose `auto_review` aujourd'hui. |
| Défaut `settingSources` du SDK | SDK `@anthropic-ai/claude-agent-sdk@0.3.276`, `sdk.mjs` : `ist=["user","project","local"]` | Confirmé : sans option, le CLI charge user+project+local. Le champ `settings_sources:"provider_default"` du contrat est honnête, ni plus ni moins. T3 ne passe pas `settingSources` sur le chemin principal (seule la sonde de capacités en passe, `ClaudeProvider.ts:231`). |

<a id="extrait-3"></a>

## Extrait 3

| Constat | Résultat |
|---|---|
| `/Users/moi/.claude-glm/settings.json` | Clés présentes : `enabledPlugins`, `extraKnownMarketplaces` seulement. Aucune clé `permissions`, `sandbox`, `hooks`, `policyHelper`, `apiKeyHelper`, `env`. |
| `/Library/Application Support/ClaudeCode/managed-settings.json` | Clés présentes : `modelPricing` seulement. |
| Dropins et remotes | `managed-settings.d` absent. `remote-settings.json` absent dans `~/.claude`, `~/.claude-glm` et le dossier application. |
| Cible T3 `claude_glm` | `/Users/moi/.t3/userdata/settings.json` : `binaryPath=/Users/moi/.local/bin/gclaude`, `homePath=/Users/moi/.claude-glm`. |
| Wrapper `gclaude` | Exporte l'auth Z.AI et des variables `ANTHROPIC_*`. Aucun override permission, settings ou tools. Ligne finale : `exec claude "$@"` (résolution PATH, voir G-P-04). Empreintes relevées : `gclaude` = `dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e` ; `claude` 2.1.296 = `c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937`. |
| Registre Bridget vivant | `/Users/moi/.config/bridget/agents.json` contient `cursor` et `claude` seulement. Aucune entrée `glm` (voir G-P-05). |

<a id="extrait-4"></a>

## Extrait 4

Le couple launcher/profil est bien identique des deux côtés T3 (`claude_glm`)
et contrat 149 (`permissions.md:217-223`). Le chemin positif same-family est
donc ancré sur des inputs réels et vérifiables par digest.

<a id="extrait-5"></a>

## Extrait 5

| Critère | État | Preuve ou trou |
|---|---|---|
| Schéma `bridget_session` v2 = identité 148 + `permissions` v1, arguments vides fermés | Couvert | `permissions.md:38-76` ; parseur fermé 148 vérifié ; corrélations driver/kind, bornes 256 o, 64 KiB. |
| Callback de confiance, lien credential/run, cwd fournisseur, politique finale | Couvert | `permission_callback` `t3_runtime` (`permissions.md:130-134`) ; fait lié au credential et au `run_id` (`permissions.md:234-243`) ; source = paramètres validés de `turn/start` et queryOptions finales. Vérifié contre le code T3 réel. |
| Mise à jour Claude init/status/mode ; settings réellement appliqués après montage MCP | Couvert | `plan.md:64` ; T007 (`tasks.md:23`) ; réutilisation et `openedPermissionMode` confirmés dans `ClaudeAdapterV2.ts` ; les options finales incluent la fusion des outils MCP montés. Défaut SDK `["user","project","local"]` vérifié : `provider_default` est exact. |
| Source Codex sandbox/approval exacte | Couvert | `permissions.md:79-114` ; champs granulaires et reviewer fidèles au schéma généré ; flags non représentés → attestation indisponible. |
| Source Claude same-family launcher/profil/cwd/sources/overrides ; digests fichier entier ou `absent` ; jamais auth/env/MCP bruts | Couvert | `permissions.md:136-199` ; `permission_sources` kind+path+sha256 ; interdiction explicite de copier le brut. Trous de preuve : G-P-04. |
| Snapshots natifs figés avant effet ; retry sans élargissement ; preuve hors verrou puis recontrôle binding | Couvert | `permissions.md:26-30`, `:245-250` ; invariant 5 du plan (`plan.md:49`) ; data-model (`data-model.md:59-70`). Oracle de divergence : G-P-02. |
| Aucun credential T3 persisté, transmis à l'enfant ou révélé par Debug | Couvert | `permissions.md:252-255` ; T009/T013/T015 ; S149-11. |
| Standalone natif : Codex `config/read` transport ; Claude PTY hook propriétaire (mode/session + inputs choisis, jamais prompt/agent) ; propriété de la connexion ; révocation | Couvert | Table `permissions.md:264-269` ; hook PreToolUse observateur sans décision ; fait lié à la connexion primaire (`permissions.md:259-263`). |
| Voie positive même profil sans fusion SDK inventée ; callbacks, règles deny, précédence, sources claires | Couvert | `permissions.md:201-230` ; la précédence reste déléguée au fournisseur, Bridget ne refusionne pas ; `settings_overrides` limité à `permissions` ; clés de sécurité non représentées → chemin indisponible. |
| Refus managed opaque limité à son contexte | Couvert avec défaut de nommage | Voir G-P-03. |
| Changement de source avant lancement/reprise → `settings_revision_changed`, sans re-résolution élargie | Contrat couvert, oracle manquant | Voir G-P-02. |
| Table de mappage : same-family ; full Codex→GLM positif ; readonly explicite ; workspace-write Codex→GLM refus de confinement OS | Couvert | `permissions.md:291-301` ; `provider_confinement_unavailable` sans équivalent attesté ; « jamais flag --yolo global » ligne 293 ; S149-24. |
| Remplacement du refus development 148 sans yolo global | Couvert | Refus 148 confirmé au code ; S149-24(3) « aucun réglage global modifié » ; wrapper limité à la définition enfant (`plan.md:74`). |
| Parent ordinaire Claude SDK full-access → GLM autonome ; premier parent Claude PTY dans un terminal ; pas de substitution descendant managed → parent externe | Couvert | Table des sources distincte (`permissions.md:265-266`) ; risque gate bloquant assumé (`plan.md:146`) ; règle correction 3 de la stratégie. |
| Admission hors projet = logique, pas confinement OS pour tout full-access | Couvert | `permissions.md:21-24` ; correction 2 de la stratégie ; S149-10/S149-15(1). |
| `--permission-prompts none` / `can_use_tool` corrélé ; faux succès impossible | Contrat couvert, oracle manquant | Voir G-P-01. |
| Aucun nouveau bypass par une politique 149 globale | Couvert | Wrapper = définition enfant seule ; S149-24(3) ; opt-out : réutilisation des chemins existants, aucun toggle nouveau (`permissions.md:32-34`, `plan.md:153`). |
| Pas de restart, pas de config de production | Couvert | Phases 5, T043-T045, non-buts de la stratégie ; managed/profile locaux inchangés requis par S149-14. |
