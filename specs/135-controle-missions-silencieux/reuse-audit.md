# Audit de réutilisation 135

Statut : READY. Analyse ciblée de la skill officielle et du lancement Politique.

## Éléments réutilisés

- `heartbeat-state.json` pour l'état durable du contrôleur.
- `events.jsonl` pour les preuves horodatées.
- `ack` pour la prise en charge initiale.
- `relaunch` pour la succession. `resolve-task` reste compatible pour les runs
  anciens, mais ne change plus un verdict terminal sous contrôle actif.
- Les sessions `existing_bridget` pour joindre worker, coordinateur et ROOT.
- Le LaunchAgent existant. Ses surcharges deviennent inutiles et l'appel est
  remplacé par le script officiel. Les anciens wrappers sont conservés.

## Éléments non créés

- Aucun daemon.
- Aucune base de données.
- Aucun second ordonnanceur.
- Aucun nouveau transport Bridget.
- Aucun fichier de suivi métier parallèle.

## Écart justifié

La commande `progress` est nouvelle. Aucun contrat existant ne permet de
distinguer une action vérifiable d'un simple message. Elle écrit dans les mêmes
tâches et le même journal.

## Arbitrages

Recherches effectuées par symbole et responsabilité dans le script et ses
tests. Aucun manifeste de dépendances : uniquement la bibliothèque Python.

| Élément | Existant recherché | Décision et raison |
|---|---|---|
| Preuve de progrès | `cmd_ack`, `input_snapshot`, `progress_snapshot` | Étendre les empreintes existantes ; créer `cmd_progress` car ACK seul n'est pas une preuve de suite. |
| Décision terminale | `cmd_resolve_task`, `cmd_relaunch` | Ajouter `cmd_disposition` car l'ancien resolve changeait le verdict ; réutiliser relaunch et reçus. |
| Migration | `cmd_init`, `run.json`, tâches v1 | Ajouter `cmd_migrate_run`, extension additive et datée, sans changer les verdicts. |
| Clôture | `ready_tasks`, statut de run | Ajouter `cmd_close_run` : disponibilité de la file ne prouve pas la fin. |
| État d'alerte | `notify_signature`, `heartbeat-state.json` | Étendre l'état existant ; `issue_key` et `advance_issue_state` portent la règle d'escalade. |
| Remise fiable | `send_bridget_message`, `write_continuation_receipt` | Étendre le transport existant avec ses clés idempotentes et des reçus dans le dossier existant. |
| Écriture concurrente | `update_task`, `write_json` | Étendre update_task avec relecture sous flock ; aucun second writer ou magasin. |
| Compatibilité | `cmd_heartbeat` historique | Conserver une branche legacy tant que les autres runs ne sont pas migrés. |

## Gate avant tasks

- [x] Les primitives existantes ont été recherchées et lues.
- [x] Aucune duplication de service, stockage ou transport.
- [x] Les nouvelles commandes répondent chacune à une exigence validée.
- [x] Aucune dépendance externe ajoutée.
- [x] Aucun arbitrage utilisateur manquant.
