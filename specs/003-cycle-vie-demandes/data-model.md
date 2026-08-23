# Modèle de données : demandes suivies

## Demande suivie

| Champ | Rôle |
|---|---|
| `id` | identifiant unique de la demande, identique à celui du message initial |
| `sender` | agent qui peut l'annuler et la consulter |
| `target` | agent sollicité |
| `state` | `open`, `answered`, `cancelled` ou `timed_out` |
| `created_at` | date de création persistée |
| `deadline_at` | date limite persistée |
| `escalation_level` | dernier rappel émis, pour empêcher les doublons après redémarrage |
| `cancel_reason` | motif optionnel de l'annulation |
| `completed_at` | date de transition vers un état terminal |

## Transitions autorisées

```text
open ── réponse identifiée ──> answered
open ── annulation émetteur ─> cancelled
open ── délai atteint ───────> timed_out
```

`answered`, `cancelled` et `timed_out` sont terminaux. Une annulation répétée de `cancelled` est acceptée sans effet; toute autre transition depuis un état terminal est refusée ou ignorée selon l'opération.

## Contraintes d'accès

- seul `sender` peut annuler ou lister une demande;
- le destinataire peut répondre mais ne peut pas modifier l'état directement;
- un lien `in_reply_to` doit viser une demande ouverte dont `sender` et `target` correspondent au sens de la réponse.
