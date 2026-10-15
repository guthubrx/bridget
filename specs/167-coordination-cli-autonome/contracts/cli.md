# Contrat CLI — SPEC-167

## Démarrer une mission

```text
bridget delegate <MISSION> [OPTIONS]
```

Options prévues :

- `--request-id <ID>` ;
- `--cwd <CHEMIN>` ;
- `--agent-type <TYPE>` ;
- `--model <MODÈLE>` ;
- `--effort <EFFORT>` ;
- `--posture discovery|development` ;
- `--reply-timeout <SECONDES>` ;
- `--keep-alive-secs <SECONDES>` ;
- `--child-requests deny|forward` ;
- `--urgent` ;
- `--deliverable <TEXTE>` ;
- `--json`.

La mission est le seul argument obligatoire. Le dossier par défaut est le dossier courant. La clé est générée si elle est absente.

## Piloter une tâche

```text
bridget task list [--state active|all] [--limit N] [--json]
bridget task status <TASK_ID> [--json]
bridget task wait <TASK_ID>... [--timeout SECONDES] [--json]
bridget task continue <TASK_ID> <MISSION> [--request-id ID] [--json]
bridget task respond <TASK_ID> <REQUEST_ID> accept|decline|cancel [--answer JSON] [--json]
bridget task cancel <TASK_ID> [--json]
bridget task ack <TASK_ID>... [--json]
```

`task list` est une projection humaine globale et bornée. Elle n’accorde aucun droit. Les autres sous-commandes vérifient l’identité du coordinateur propriétaire.

## Sortie

- Sans `--json` : lignes courtes avec identifiants, état et action utile.
- Avec `--json` : un seul objet JSON sur stdout.
- Les avertissements et erreurs vont sur stderr.
- Une erreur d’identité explique comment lancer ou rattacher un agent Bridget.

## Compatibilité

`bridget auto-delegate` reste disponible et inchangé. Les nouvelles commandes sont une façade ergonomique. Elles ne remplacent pas le contrat technique historique.
