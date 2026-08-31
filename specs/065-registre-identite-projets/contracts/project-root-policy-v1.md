# Contrat de configuration v1: racines projet autorisées

## Autorité et chargement

Bridget est l'unique autorité de cette politique hôte. Le daemon reçoit au
démarrage un chemin absolu explicite vers le document v1. Il n'existe aucun
chemin implicite ni valeur par défaut. Le document est chargé une fois et tout
changement exige un nouveau démarrage contrôlé du daemon.

L'absence du document, une erreur de syntaxe, une version inconnue, une liste
vide, un propriétaire différent de l'UID du daemon ou un fichier modifiable par
le groupe ou les autres comptes ferme toutes les mutations du registre. Les
lancements host historiques sans ProjectReference restent disponibles.

## Forme fermée

```json
{
  "contract_version": 1,
  "allowed_project_roots": [
    "/home/moi/bridget-referent",
    "/home/moi/20.cartae"
  ]
}
```

Les champs inconnus et les doublons après canonicalisation sont refusés. Chaque
racine doit exister, être un répertoire absolu canonique et rester sous le
contrôle du compte opérateur. `/`, `/home`, `/Users` et le home complet du
daemon sont interdits comme frontières trop larges. Une racine de projet ou un
worktree candidat est canonicalisé avant comparaison.

## Exemples figés pour les tests

Politique fixture valide:

```json
{
  "contract_version": 1,
  "allowed_project_roots": [
    "/srv/bridget-fixtures/projects",
    "/srv/bridget-fixtures/reviews"
  ]
}
```

Entrées refusées avant toute mutation:

| Cas | Forme ou métadonnée | Refus Bridget attendu |
|---|---|---|
| Document absent | chemin de politique inexistant | `project_root_policy_unavailable` |
| Version inconnue | `contract_version` différent de 1 | `project_root_policy_invalid` |
| Champs supplémentaires | clé hors schéma fermé | `project_root_policy_invalid` |
| Liste vide | `allowed_project_roots: []` | `project_root_policy_invalid` |
| Frontière trop large | `/`, `/home`, `/Users` ou home complet | `project_root_policy_invalid` |
| Doublon canonique | deux alias d'une même racine | `project_root_policy_invalid` |
| Propriétaire ou mode invalide | UID différent, groupe ou autres inscriptibles | `project_root_policy_permissions_invalid` |

Ces chemins sont des fixtures de contrat. Ils ne sont ni lus ni créés par la
documentation et ne constituent aucune valeur par défaut de production.

## Évolution

SPEC-065 ne lit que ce document. Les politiques runtime de SPEC-066 et le
catalogue de ressources de SPEC-067 utilisent leurs propres contrats fermés,
afin qu'une tranche puisse être livrée ou abandonnée sans parser les champs
des tranches suivantes.
