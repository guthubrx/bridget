# Spécification 128 - GLM et second compte Codex lançables par Bridget

## Fiche synthèse
Spec: 128-fournisseurs-lancables | Statut: In Progress | Priorité: P1 | Date: 2026-10-02
Branche: session-128-fournisseurs-lancables | Accord de l'utilisateur pour GLM « et tout autre provider ».

## Problème observé
`bridget spawn glm` / `gclaude` refusé : aucun type GLM dans le registre du daemon, et la clé
`ZAI_API_KEY` du lanceur `gclaude` n'était connue que du coffre de T3. Les agents se rabattaient
sur des sous-agents plus coûteux. Fournisseurs T3 : `cursor` (type par défaut), `cx-pro` (Codex,
`CODEX_HOME` propre), `claude_glm` (GLM via `gclaude`, profil `~/.claude-glm`).

## Réalisation (configuration locale, hors dépôt)
- Clé GLM copiée du coffre de T3 vers le trousseau macOS (service `ZAI_API_KEY`), empreinte
  vérifiée identique ; `gclaude` la lit dans le trousseau quand la variable manque
  (sauvegarde `~/.local/bin/gclaude.avant-128`).
- Lanceur `~/.local/bin/codex-pro` (CODEX_HOME du compte pro).
- Registre `~/.cache/bridget-core/agents.json` (0600) : types `glm` (claude_stream_json, profil
  `~/.claude-glm`, glm-5.3) et `codex-pro` (codex_app_server, gpt-6.1-sol), validés par le code de
  lecture du registre ; variantes `project-discovery-glm` et `project-discovery-codex-pro`.
- Dépôt : documentation générique dans le guide d'installation, CHANGELOG.

## Critères de succès
- **SC-001** : après relance du daemon, `bridget spawn glm` et `bridget spawn codex-pro` sans
  `--posture`, depuis une session sans terminal, sont acceptés en lecture seule, répondent à une
  demande, puis s'arrêtent.
