# Quickstart 135 — Contrôle silencieux des missions

## Nouveau run

```bash
python3 /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py init \
  --run-id exemple --objective "Vider le chantier" \
  --acceptance "Toutes les missions ont une décision"
```

## Run existant

```bash
python3 /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py migrate-run \
  --run-id exemple
```

## Prouver un progrès

```bash
python3 /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py progress \
  --run-id exemple --task-id T001 --worker AGENT_UUID \
  --kind source_changed --evidence-ref file:/chemin/absolu/fichier
```

## Décider la suite

```bash
python3 /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py disposition \
  --run-id exemple --task-id T001 --decision accepted \
  --reason-code accepted_in_scope
```

## Fermer

```bash
python3 /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py close-run \
  --run-id exemple --reason "Toutes les obligations sont levées"
```
