# Modèle de données — SPEC-167

## Principe

La session n’ajoute aucune table et aucun enregistrement durable. Elle réutilise les entités de délégation native.

## Entités réutilisées

### Identité du coordinateur

- `name` : identifiant stable de l’agent parent.
- `instance_id` : incarnation vivante du parent.
- Source : `resolve_current_identity()`.
- Règle : aucune identité ne vient d’un argument CLI.

### Demande de délégation automatique

- `request_id` : clé de rejeu stable.
- `task` : mission.
- `cwd` : dossier du projet.
- Sélecteurs facultatifs : type, modèle, effort, posture, machine et projet.
- Contrôles facultatifs : délai, garde, stratégie de demandes enfant, urgence et livrable.

### Tâche native

- `task_id` : identifiant durable rendu par le daemon.
- `owner` et `owner_instance` : coordinateur.
- `child` et `child_instance` : agent choisi.
- `state` : état durable existant.
- `result` : résultat stable existant.
- `pending_requests` : demandes actionnables existantes.

## Transitions

La façade CLI n’ajoute aucune transition. Elle invoque les transitions existantes :

```text
admission → queued → starting → mission_pending → working
working → waiting_for_children → result_available
working → failed | cancelling → cancelled
result_available → continuation d’une nouvelle ronde, si la garde le permet
```

## Invariants

1. Le parent est déduit de la connexion authentifiée.
2. La clé de demande n’accorde aucun droit.
3. Un rejeu identique rend le reçu existant.
4. Un rejeu divergent est refusé.
5. La sortie CLI ne devient jamais une seconde source de vérité.
