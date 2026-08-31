# Modèle de données - Pilotage des rondes par projet

## ProjectRoundProjection

Projection effective réutilisée et enrichie.

| Champ | Type | Règle |
|---|---|---|
| `project_id` | chaîne opaque | non vide, identité durable Bridget |
| `binding_generation` | entier optionnel | obligatoire pour une liaison connue |
| `active` | booléen | dérivé de la liaison courante |
| `configured` | booléen | vrai si une décision existe pour cette génération |
| `enabled` | booléen | vrai seulement si actif, configuré et activé |
| `revision` | entier | zéro sans décision, strictement positif sinon |
| `updated_at` | instant Unix | dernière décision de politique ou observation sûre |
| `last_occurrence_at` | instant Unix optionnel | dernière occurrence traitée pour cette génération |
| `last_dispatch_state` | état fermé optionnel | `deposited`, `refused` ou `indeterminate` |
| `last_dispatch_observed_at` | instant Unix optionnel | instant où Bridget a classé le résultat |

### Invariants

- Sans ligne de politique : `configured=false`, `enabled=false` et aucun dernier passage.
- Un projet inactif ne peut jamais être projeté `enabled=true`.
- Les trois champs du dernier passage sont tous présents ou tous absents.
- Une occurrence plus ancienne ne remplace jamais l'occurrence stockée.
- Une nouvelle génération ne lit jamais les faits de l'ancienne génération.

## ProjectRoundDispatchState

| Valeur | Signification |
|---|---|
| `deposited` | le socle a accepté la remise ou a fourni un identifiant de remise |
| `refused` | le socle a attesté un refus ou une enveloppe invalide |
| `indeterminate` | aucune conclusion terminale sûre ne peut être affichée |

Cet état ne décrit pas la réponse de l'agent ni la progression du travail. Il décrit seulement la tentative de remise de la ronde.

## UiProjectRoundV1

Projection du relais vers le navigateur.

| Champ | Règle |
|---|---|
| `configured`, `enabled`, `revision`, `updated_at` | copies confirmées de la politique |
| `interval_secs` | constante canonique `420` |
| `last_occurrence_at` | optionnel |
| `last_dispatch_state` | optionnel et fermé |
| `last_dispatch_observed_at` | optionnel |

Le prochain instant n'est pas stocké ni calculé. L'UI déduit seulement une attente maximale de `interval_secs`.

## UiProjectListEntryV1

Ajoute à l'entrée existante :

- `binding_generation`, obligatoire pour les projets rendus;
- `round`, projection confirmée décrite ci-dessus.

## UiProjectRoundMutationV1

| Champ | Règle |
|---|---|
| `version` | exactement 1 |
| `command_id` | non vide, borné et rejouable |
| `project_id` | identifiant opaque valide |
| `binding_generation` | strictement positif |
| `enabled` | état souhaité explicite |

## Transitions

```text
non configuré -- activer --> activé (révision 1)
non configuré -- désactiver --> désactivé explicite (révision 1)
activé -- désactiver --> désactivé (révision +1)
désactivé -- activer --> activé (révision +1)
même état -- même ou nouvelle commande --> état inchangé, révision inchangée
rebind --> nouvelle génération non configurée
```
