# Contrat - Délégation native 150

Compatible avec le contrat 148 (`specs/148-identite-delegation/contracts/delegation.md`)
et le contrat Lineage 149. Un appel sans nouveau champ produit les mêmes octets
canoniques et le même comportement qu'en 149.

## `bridget_delegate` - champ ajouté

| Champ | Type | Obligatoire | Bornes | Défaut |
|---|---|---|---|---|
| `reply_timeout` | entier | non | 60 à 86 400 | 3600 |

Le champ fait partie de l'enveloppe idempotente. Omis et `3600` explicite sont
deux enveloppes différentes.

## Refus ajoutés (code renvoyé dans `native_delegation_refused`)

| Code | Quand | Écriture |
|---|---|---|
| `referent_paused` | pause du référent active, nouvelle clé | aucune ; la clé reste libre |
| `request_expired` | clé d'une tâche purgée depuis moins de 30 jours | aucune |
| `retention_saturated` | 4096 tâches conservées et aucun arbre purgeable | aucune (après une purge de borne : seuls les arbres nécessaires pour repasser sous 4096 sont retirés, sans limite de 128) |
| `reply_timeout_out_of_range` | délai hors 60 à 86 400 | aucune |
| `control_state_unavailable` | état de pause illisible à l'admission : refus fermé plutôt qu'admission pendant une pause possible | aucune |
| `reply_timeout_exceeds_parent` | délai explicite plus long que le temps restant du parent (jamais sans délai explicite) | aucune |

Refus retiré : `task_limit` pour cause d'historique cumulé. `task_limit` reste
pour 128 tâches actives ou 16 enfants actifs par instance.

Ordre : `identity_unavailable`, `invalid_request`, `reply_timeout_out_of_range`,
rejeu (`envelope_mismatch`, `request_expired`), pause (`referent_paused`, ou
`control_state_unavailable` si l'état de pause est illisible), gardes de droits 149, `task_limit`, `retention_saturated`, lignée parente,
`reply_timeout_exceeds_parent`.

Un rejeu d'une clé existante pendant la pause rend la tâche existante, comme
en 149.

## `bridget_capabilities` - clés

| Clé | Valeur |
|---|---|
| `mission_reply_limit_secs` | 3600 (défaut, inchangé) |
| `mission_reply_min_secs` | 60 (nouveau) |
| `mission_reply_max_secs` | 86 400 (nouveau) |

## Lineage T3

Aucun changement de forme. Une purge change la génération : le lecteur fait un
snapshot complet, comme déjà prévu par le contrat 149.
