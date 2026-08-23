# Quickstart — Vérifier le modèle et l'effort dans l'annuaire

**Feature** : 004-runtime-modele-agents

## Installation

```bash
cd ~/bridget
cargo build --release
./target/release/bridget install-hooks     # une seule fois, pour les agents Claude
```

`install-hooks` affiche le chemin de la sauvegarde de `~/.claude/settings.json`.
Pour revenir en arrière : `./target/release/bridget install-hooks --remove`.

Le hook ne prend effet que pour les sessions Claude **démarrées après**
l'installation.

## Vérification 1 — affichage (User Story 1)

```bash
bridget who
```

Attendu : deux colonnes `MODÈLE` et `EFFORT` avant `ÉTAT`. Un agent qui n'a pas
encore produit de tour affiche `—` dans les deux, sans casser l'alignement.

```bash
bridget agents --json | python3 -m json.tool
```

Attendu : les champs `model` et `effort` pour chaque agent.

## Vérification 2 — changement de modèle (User Story 2)

Sur un agent Claude lancé par `bridget claude` :

1. `bridget who` → noter le modèle affiché.
2. Dans l'agent : `/model haiku`, puis lui faire produire une réponse
   quelconque.
3. `bridget who` → le modèle affiché doit avoir changé.

Sur un agent Codex lancé par `bridget codex` :

1. Dans l'agent, changer l'effort de raisonnement, puis lui faire produire un
   tour.
2. Attendre 20 secondes, puis `bridget who`.

## Vérification 3 — déclaration explicite (User Story 3)

Depuis un agent connecté :

```bash
bridget runtime --model gemini-3-pro --effort medium
bridget who
```

Depuis un shell ordinaire, hors agent :

```bash
bridget runtime --model test
# attendu : « runtime indisponible hors d'un agent Bridget », code 1
```

## Diagnostic

Le hook est silencieux par construction. Pour l'observer, l'exécuter à la main
avec un payload réel :

```bash
echo '{"transcript_path":"'"$(ls -t ~/.claude/projects/*/*.jsonl | head -1)"'"}' \
  | BRIDGET_AGENT_NAME=agent-2 bridget hook claude-runtime
bridget who
```

Pour la sonde Codex, vérifier que le rollout est bien visible :

```bash
lsof -p "$(pgrep -x codex | head -1)" | grep jsonl
```

Journaux du daemon : `RUST_LOG=debug bridget daemon` fait apparaître la source
de chaque mise à jour (`codex-rollout`, `claude-hook`, `declared`).

## Tests automatisés

```bash
cargo test
```

Attendu : les 36 tests existants plus ceux de la feature, tous verts.
