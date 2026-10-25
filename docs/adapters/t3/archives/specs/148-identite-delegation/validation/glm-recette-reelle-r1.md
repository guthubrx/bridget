# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-recette-reelle-r1.md](../../../../../../../specs/148-identite-delegation/validation/glm-recette-reelle-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le parent est une **fixture Codex synthétique**. Aucun modèle Codex réel n'est exécuté.
- L'enfant GLM est réel (daemon, MCP, flotte, wrapper réels). Aucun serveur T3 ne fait partie du chemin natif de l'enfant. Le fait que le testeur soit lui-même lancé par T3 est hors du chemin testé.
- Aucun effort (effort level) n'a été fourni : le catalogue source n'en déclare pas.

<a id="extrait-2"></a>

## Extrait 2

| Étape | Commande | Résultat | RC |
|---|---|---|---|
| Tests lib ciblés | `--lib native148` | 23 passed, 0 failed | 0 |
| Tests lib ciblés | `--lib matrice_fr009` | 1 passed | 0 |
| Tests lib ciblés | `--lib mcp_identity` | 15 passed | 0 |
| Tests lib ciblés | `--lib t3code_mcp` | 6 passed | 0 |
| E2E | `--test native_delegation_e2e` | 1 passed, 1 ignored | 0 |
| E2E | `--test t3_session_identity_env` | 1 passed, 1 ignored | 0 |
| Format | `cargo fmt --all -- --check` | 0 diff | 0 |
| Clippy | `--workspace --lib -- -D warnings -A clippy::too_many_arguments` | 0 warning, 0 error | 0 |
| Compilation | `--test native_delegation_real_glm --no-run` | binaire `native_delegation_real_glm-888c55e9145aadf8` | 0 |
