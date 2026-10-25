# Archives techniques de l’adaptateur T3

Document d’origine : [specs/128-fournisseurs-lancables/spec.md](../../../../../../specs/128-fournisseurs-lancables/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Problème observé
`bridget spawn glm` / `gclaude` refusé : aucun type GLM dans le registre du daemon, et la clé
`ZAI_API_KEY` du lanceur `gclaude` n'était connue que du coffre de T3. Les agents se rabattaient
sur des sous-agents plus coûteux. Fournisseurs T3 : `cursor` (type par défaut), `cx-pro` (Codex,
`CODEX_HOME` propre), `claude_glm` (GLM via `gclaude`, profil `~/.claude-glm`).

<a id="extrait-2"></a>

## Extrait 2

## Réalisation (configuration locale, hors dépôt)
- Clé GLM copiée du coffre de T3 vers le trousseau macOS (service `ZAI_API_KEY`), empreinte
  vérifiée identique ; `gclaude` la lit dans le trousseau quand la variable manque
  (sauvegarde `~/.local/bin/gclaude.avant-128`).
- Lanceur `~/.local/bin/codex-pro` (CODEX_HOME du compte pro).
- Registre `~/.cache/bridget-core/agents.json` (0600) : types `glm` (claude_stream_json, profil
  `~/.claude-glm`, glm-5.3) et `codex-pro` (codex_app_server, gpt-6.1-sol), validés par le code de
  lecture du registre ; variantes `project-discovery-glm` et `project-discovery-codex-pro`.
- Dépôt : documentation générique dans le guide d'installation, CHANGELOG.
