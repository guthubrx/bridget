# Modèle de données - SPEC-081

## AgentIdentity

| Champ | Type | Règle |
|---|---|---|
| agent_id | UUID v4 | clé primaire opaque, stable, non affichée |
| created_at | epoch secondes | immuable |
| updated_at | epoch secondes | audit |
| lifecycle | active, stopped, retired | suppression = retired |

## AgentProfile

| Champ | Type | Règle |
|---|---|---|
| agent_id | FK AgentIdentity | relation 1:1 |
| display_name | texte 1..80 | unique parmi agents non retired |
| labels | collection ordonnée | 0..12, affichage |
| avatar_shape, avatar_color | enum | affichage |
| instructions | texte borné | injection fournisseur contrôlée |

## Principaux de message

| Classe | Valeur | Règle |
|---|---|---|
| Agent | UUID v4 | doit référencer une identité active ou stopped |
| Humain | human | principal réservé |
| Système | system:<service> | bridget, maicie, resume documentés |

## Maicie

| Objet | Champ cible | Si agent absent |
|---|---|---|
| Profil | agent_id | profil inactif |
| Délégation | agent_id | requires_retarget |
| Outbox | agent_id | pas de livraison |
| Routine | agent_id | suspendue, retarget requis |
| Objectif | agent_ids | conservation avec marquage |

## Invariants

1. Deux agents actifs ou stopped ne partagent pas agent_id.
2. Renommer display_name ne change aucune référence technique.
3. Une identité retired ne peut pas recevoir une nouvelle livraison.
4. requires_retarget n'est jamais éligible à l'outbox.
5. Une valeur historique n'est plus utilisée après migration.
6. agent_id n'est jamais une preuve d'autorisation.
