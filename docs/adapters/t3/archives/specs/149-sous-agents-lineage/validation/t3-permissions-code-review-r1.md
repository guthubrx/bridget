# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-permissions-code-review-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-permissions-code-review-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10
Réviseur : sous-agent de revue indépendante (GLM 5.3 Flash)
Périmètre : code T3 T004–T008, contre `contracts/permissions.md` (contrat normatif permissions-149), plan r2 APPROVE (`validation/plan-permissions-r2.md`), data-model, ADR 149, ADR 017.
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
Baseline T3 : `33f6d04e116430bf7f0011902d6af3868163f2d1` (tout le delta T004–T008 est non committé, relu par `git diff`).

<a id="extrait-2"></a>

## Extrait 2

**APPROVE** — code T3 T004–T008 uniquement.

<a id="extrait-3"></a>

## Extrait 3

- Emplacement : `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:850-851`.
- Constat : la condition ne vérifie pas `permissionPromptToolName`, `permissionPrompts`, `spawnClaudeCodeProcess`, `hooks` dans `input.options`. Aucun chemin T3 ne les pose aujourd'hui (vérifié dans le constructeur des options de requête). Si un chemin futur les introduisait, la médiation réelle changerait sans invalider le fait publié (`tool_approval: "prompt"` resterait annoncé).
- Oracle de déclenchement : `options.permissionPromptToolName = "x"` + `canUseTool` défini → fait publié alors que la médiation ne passe plus par le callback T3.
- Fix minimal pour Sol : exiger `undefined` pour ces quatre champs dans la condition de la ligne 850.

<a id="extrait-4"></a>

## Extrait 4

- Aucun `tsc`, build, lint ou test exécuté. La validité à la compilation n'est pas prouvée par cette revue.
- Aucune preuve runtime, aucun appel modèle, aucune preuve standalone. Je n'en annonce pas.
- Le Rust natif T009+ (admission inherit/development côté GP) est en cours chez Sol. Non relu. Les oracles G-P-01/G-P-02 sont post-native : non évaluables ici.
- Les tests permissions T3 ne sont pas encore écrits (lot de l'agent de tests). Je ne les demande pas en double. Un regret : la couverture visée devrait inclure l'oracle C-01 (clé `allowedMcpServers` dans settings inline) et C-02 (champ `permissionPromptToolName`).
- Le registre actif réel (`/Users/moi/.cache/bridget-core/agents.json`, entrée glm : `gclaude`, profil `claude-glm`) et la faute r1 corrigée du plan r2 sont conformes à ce que le code exige (hash launcher connu présent dans le code).

<a id="extrait-5"></a>

## Extrait 5

Le code T004–T008 tient le contrat permissions-149 côté T3 : héritage vrai attesté depuis l'état réel du provider, aucune autorité forgeable depuis MCP ou le prompt parent, faits privés par run/attempt/fil provider, liaison par objet credential, union v1/v2 avec tombes anti-downgrade, refus nommés conservateurs, hygiène des secrets respectée sur tout le chemin de publication.
