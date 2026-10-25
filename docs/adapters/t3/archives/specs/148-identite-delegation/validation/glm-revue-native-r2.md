# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-revue-native-r2.md](../../../../../../../specs/148-identite-delegation/validation/glm-revue-native-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Wrapper 20 outils** : `BRIDGET_SAFE_MCP_TOOLS` = 20 entrées exactes (`wrapper.rs:4750-4771`), sans wildcard, `guichet_*` exclu. Deux usages cohérents : Claude `--allowedTools` préfixé `mcp__bridget__`, Codex `tools={...approve}` + `default_tools_approval_mode="prompt"`. Tests : spec091 octet-pour-octet, spec094, `native148_claude_allows...` (20, 4 nouveaux présents, pas de `*`). Restriction sous-agent interne who/send conservée (`mcp.rs:365-377` via `delegated_origin`) ; l'enfant natif n'est pas marqué `delegated_origin`, donc il garde `bridget_delegate` pour la profondeur 8.
- **Autres durcissements** : résolution Git hors verrou avant `state.lock`, comparaison au puits dans `authorize_cwd` — `SameProject` exige maintenant l'égalité exacte de projet (plus strict qu'un `starts_with`) ; cleanup/annulation sans preuve retour parent (test dédié) ; notices failed indexées sur `$.owner` (index SQL dédié) ; `parent_execution_id` depuis les exécutions actives seulement, `parent_execution_ambiguous` si >1 (`execution_store.rs:1456`, LIMIT 2).
- **T3 `applyFlagSettings`** : appliqué seulement après montage, statut `connected` relu, et présence vérifiée de tous les outils y compris les 2 lectures. Vieux catalogue sans les 2 outils → `catalogue_unavailable` → candidat fermé (`ClaudeAdapterV2.open`, onError → close) : pas de règles sur outils absents, pas de montage dégradé silencieux. Fusion `claudeBridgetReadOnlySettings` : préserve deny/ask/defaultMode/allow, ajoute seulement `capabilities`+`task_status`, aucun mutant ni wildcard ; settings en chemin de fichier → `undefined` → aucune application. Opt-out disabled/custom/revoked/explicit → zéro application, zéro appel. Aucun symbole nouveau côté Codex (grep).
- **Docs** : corrigées. `delegate-grant` référencé (`commandes.md:51`, `delegation-native.md`) ; exemple `glm` avec note « `claude_glm` n'est pas un type du registre » ; motif development distingué (`development_protocol_unavailable` au catalogue vs message registre à l'appel) ; échéance et révocation stable documentées (`contracts/delegation.md`) ; plus aucun « 16 outils » dans les guides.

<a id="extrait-2"></a>

## Extrait 2

1. **Recette GLM réelle non exécutée** : `native_delegation_real_glm.rs` existe, `#[ignore]` + `BRIDGET_148_REAL_GLM=1` requis, parent fixture Codex synthétique annoncé dans l'en-tête. Aucun résultat d'exécution réel n'est disponible dans ce que j'ai lu. Aucune preuve d'un modèle réel de bout en bout n'existe donc — je n'en fabrique pas.
2. **Tests non exécutés par moi** (contrainte lecture seule) : mes conclusions comportementales viennent du code et des tests lus, pas d'un passage réel.
3. **« Codex approval/sandbox inchangés »** : prouvé par absence des nouveaux symboles dans les chemins Codex et lecture du montage existant. Pas de diff Git (interdit). Preuve partielle.
4. **Test CLI T3 réel** (`BRIDGET_147_CLAUDE_METADATA_COMMAND`) : skippé sans la variable, non exécuté.
5. **Interop réseau identité HTTP** : annoncé en cours, hors de ce périmètre.

<a id="extrait-3"></a>

## Extrait 3

Les quatre objections R1 sont closes, chacune avec garde en code et test ciblé lu. L'intégration wrapper/T3 est conforme au brief. Les manques ci-dessus relèvent de l'exécution, pas du code lu.
