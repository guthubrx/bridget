# Quickstart — Spec 314

## 1. Exemple de graphe avec porte déterministe, escalade et branche de repli

```json
{
  "request_id": "req-314-demo",
  "title": "Implémentation sécurisée avec porte déterministe, escalade et repli",
  "steps": [
    {
      "step_id": "implement",
      "delegation": {
        "task": "Implémenter la fonctionnalité et ses tests.",
        "cwd": "/chemin/projet",
        "workspace": { "strategy": "worktree" }
      },
      "convergence": {
        "gate": {
          "argv": ["cargo", "test", "-p", "mon-crate"],
          "timeout_secs": 120
        },
        "verify": {
          "task": "Auditer le commit et vérifier les cas limites.",
          "cwd": "/chemin/projet",
          "posture": "discovery",
          "workspace": { "strategy": "commit_view" }
        },
        "different_provider": true,
        "max_iterations": 2,
        "stop_on_stagnation": true,
        "escalation": {
          "effort": "high",
          "extra_iterations": 2,
          "on_stagnation": true,
          "on_max_iterations": true
        }
      }
    },
    {
      "step_id": "deliver",
      "depends_on": ["implement"],
      "when": "all_succeeded",
      "delegation": {
        "task": "Finaliser la documentation et le bilan.",
        "cwd": "/chemin/projet"
      }
    },
    {
      "step_id": "diagnose_fallback",
      "depends_on": ["implement"],
      "when": "any_failed",
      "delegation": {
        "task": "Analyser les points bloquants de l'échec et proposer un plan B propre.",
        "cwd": "/chemin/projet",
        "posture": "discovery"
      }
    }
  ]
}
```

## 2. Consulter et paramétrer la rétention Time-Machine des métriques

```bash
# Lire les métriques agrégées (brut + journalier + mensuel)
bridget graphs metrics --window 90

# Lire la politique de rétention / rollup active
bridget graphs metrics-policy

# Ajuster les seuils et lancer un passage de rollup/purge immédiatement
bridget graphs metrics-policy --raw-days 30 --daily-days 180 --monthly-months 24 --graph-days 30 --run-now
```
