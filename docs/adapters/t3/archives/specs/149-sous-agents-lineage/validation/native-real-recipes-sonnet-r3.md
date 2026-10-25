# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `make_fixture_registry.py`, `run_parent_pty.py`, `verify_oracles.py` : compilation Python OK. `recipe_env.sh`, `run_fixture_daemon.sh` : `bash -n` OK.
- `run_parent_pty.py` : arrêt par SIGTERM sur un seul PID vérifié, attente 3 s, aucun `killpg`, `pkill` ni `-9`.
- `recipe_env.sh` r2 : refus global des règles retiré. Fixture positive `<cwd>/.claude/settings.json` (allow Read/Glob/Grep/`Edit(/allowed/**)`, deny `Bash` et `Edit(/forbidden/**)`). Note d'oracle : selon la doc permissions, les chemins ne sont évalués que pour `Edit(path)` (qui couvre Write) et `Read(path)` ; une règle `Write(path)` serait ignorée. Les règles d'outil ne sont pas un confinement OS.
- Garde-fous : baseline filesystem avant/après, snapshot des sources, empreintes lanceur gclaude (`dd8dee...`) et CLI résolu, absence des variables T3.
- Prompts présents : `parent_delegate_glm.md`, `parent_delegate_glm_from_codex.md`, `parent_delegate_codex_child.md`, `parent_delegate_cancel.md`.
