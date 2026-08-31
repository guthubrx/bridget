# Modèle de données - SPEC-079

## DeliveryExecutionContext

| Champ | Règle |
|---|---|
| execution_id | identifiant Bridget non vide |
| generation | génération d'exécution strictement positive |
| revision | révision courante |

Ce contexte est optionnel dans `DeliverIdempotent` pour la compatibilité des
remises historiques. Lorsqu'il existe, il provient exclusivement du lien
durable `send_delivery_execution_links`.

## ReconstructedExecution

Un descendant de reprise est une nouvelle ligne `executions` qui:

- conserve le `submission_id` du parent;
- conserve exactement sa `ProjectReference` optionnelle;
- pointe vers la nouvelle instance gérée;
- incrémente la génération d'exécution;
- démarre en `starting`;
- possède une ligne `execution_continuations` en mode `reconstructed` et raison `daemon_restart`.

Le parent devient `unreachable` avant l'activation du descendant. Les deux
écritures et la continuation partagent une transaction SQLite.

## ProjectRoundPolicy

| Champ | Règle |
|---|---|
| project_id | identité opaque 065 |
| binding_generation | génération exacte de la liaison active |
| enabled | booléen explicite |
| revision | entier monotone |
| updated_at | instant UTC |
| command_id | dernière commande idempotente appliquée |

Une absence de ligne équivaut à `disabled`. Une ligne dont la génération ne
correspond plus à `ProjectBinding.generation` est `stale` et ne produit aucune cible.

## ProjectRoundCommand

| Champ | Règle |
|---|---|
| command_id | clé stable et unique |
| canonical_bytes | projet, génération et état souhaité |
| outcome_json | résultat exact du premier passage |
| observed_at | instant de décision |

Un rejeu divergent est refusé. Un rejeu exact retourne le résultat conservé.

## RoundOccurrence

Identité logique:

```text
(project_id, binding_generation, scheduled_at)
```

La clé d'idempotence est dérivée de ce triplet. Aucune occurrence manquée n'est
rejouée lors d'une réactivation.
