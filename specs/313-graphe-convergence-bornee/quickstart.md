# Guide rapide — Graphe avec étape de convergence bornée (`313`)

## Exemple minimal `bridget_delegate_graph`

```json
{
  "request_id": "feat-auth-20261127-01",
  "title": "Implémentation + contre-revue bornée",
  "on_failure": "stop_dependents",
  "notify_policy": "on_graph_done_or_failure",
  "steps": [
    {
      "step_id": "impl-et-revue",
      "delegation": {
        "engine": "codex",
        "cwd": "/chemin/absolu/du/projet",
        "posture": "development",
        "workspace": { "strategy": "worktree" },
        "expected_base_commit": "0123456789abcdef0123456789abcdef01234567",
        "task": "Implémenter X, lancer les tests et committer sur la branche du worktree."
      },
      "convergence": {
        "max_iterations": 3,
        "stop_on_stagnation": true,
        "verify": {
          "engine": "claude",
          "cwd": "/chemin/absolu/du/projet",
          "posture": "discovery",
          "workspace": { "strategy": "commit_view" },
          "expected_base_commit": "0123456789abcdef0123456789abcdef01234567",
          "task": "Vérifier le diff, les tests et les cas limites de l'étape précédente."
        }
      }
    },
    {
      "step_id": "synthese",
      "depends_on": ["impl-et-revue"],
      "delegation": {
        "cwd": "/chemin/absolu/du/projet",
        "posture": "discovery",
        "task": "Rédiger la note de livraison à partir du commit validé."
      }
    }
  ]
}
```

## Contrat de verdict injecté automatiquement au vérificateur
Le vérificateur termine par une ligne JSON autonome :
- `{"verdict":"approve","reason":"..."}` → étape `completed`, déblocage des étapes aval ;
- `{"verdict":"revise","reason":"...","blocking_points":["..."]}` → relance automatique du producteur (sauf stagnation ou `max_iterations`) ;
- `{"verdict":"escalate","reason":"..."}` → arrêt immédiat vers le parent.
