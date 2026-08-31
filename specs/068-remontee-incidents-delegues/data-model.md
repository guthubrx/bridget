# Modèle de données - SPEC-068

## Fait d'incident délégué

| Champ | Règle |
|---|---|
| `cursor` | entier monotone, ordre de reprise du parent |
| `event_id` | identifiant stable de l'occurrence |
| `link_id` | lien durable déjà existant |
| `parent_instance_id` | cible de la remise, dérivée du lien |
| `child_instance_id` | enfant à l'origine du fait |
| `child_execution_id` | absent seulement si le fournisseur ne l'a pas corrélé |
| `kind` | `warning` ou `failed` |
| `code` | code stable non vide |
| `reference` | référence redacted non vide |
| `observed_at` | instant Unix constaté par Bridget |
| `acknowledged_at` | absent avant remise technique au parent |

## Invariants

1. Le lien doit exister au moment de l'enregistrement.
2. Un `warning` ne modifie aucune transition d'exécution.
3. Un `failed` est créé uniquement à partir d'un terminal d'exécution attesté.
4. Ni le détail fournisseur ni le corps d'un message ne sont stockés.
5. Un accusé ne peut être posé que par la connexion dont l'instance est le
   parent du lien.
6. Les faits non accusés sont relus par `cursor` ascendant.
7. Le même fait est dédupliqué par son identité stable avant toute remise.

## Relations

```text
agent_links (1) ──── (n) delegated_runtime_events
parent wrapper (1) ─── accuse ─── delegated_runtime_events
child execution (0..1) ─── corrèle ─── delegated_runtime_events
```
