# Contrat de libellé d’expéditeur

## Entrées

- `from` : UUID routable obligatoire.
- `from_display_name` : nom de présentation optionnel.
- `delegated_origin` : fournisseur et référence enfant opaques optionnels.

## Sorties

| Cas | Libellé attendu |
|---|---|
| Nom `Regional`, UUID `cbd8…` | `Regional (cbd8…)` |
| Nom absent ou vide | `cbd8…` |
| Nom égal à l’UUID | `cbd8…` |
| Parent nommé avec provenance | `Regional (cbd8…) (via sous-agent codex abc…)` |

Le rendu utilise l’UUID complet. Les points de suspension ci-dessus servent
uniquement à raccourcir les exemples de documentation.

## Invariants

- Le nom n’est jamais une clé de routage, d’autorisation ou de réponse.
- Un nom invalide ne bloque pas la remise.
- Le corps du message ne participe pas au libellé.
- Un lot applique ce contrat séparément à chaque message.
