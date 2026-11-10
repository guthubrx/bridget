# Modèle de données 260

Ce programme ne crée aucune donnée du produit. Les entités ci-dessous sont les objets de travail de l'implémenteur.

## Lot

| Champ | Valeur |
|-------|--------|
| Numéro | 0 à 17 |
| Guide | `guides/lot-NN-*.md` (lot 0 et clôture : `quickstart.md`) |
| Branche | `session-260-lotNN-nom` |
| Worktree | `.worktrees/260-lotNN-nom` |
| État | à faire, ouvert, intégré, arrêté, différé |

Règles : un seul lot « ouvert ». Un lot passe à « intégré » quand sa porte est franchie et que `origin/main` contient ses commits.

## Tâche

| Champ | Valeur |
|-------|--------|
| Identifiant | `T001` à `T157`, fixe |
| Lot | un seul |
| Contenu | fichier, symbole à chercher, action, commande de vérification, résultat attendu |
| État | case vide, case cochée. « arrêté » et « différé » s'écrivent dans `implementation.md` ; la case reste vide |

Règle : une case se coche après la vérification, jamais avant.

## Porte de décision

| Champ | Valeur |
|-------|--------|
| Identifiant | `D1` à `D16` |
| Fichier | `decisions.md` |
| État | ouverte (action par défaut appliquée) ou tranchée (réponse de François écrite) |

Règle : l'implémenteur ne tranche jamais une porte. Il applique l'action par défaut et l'écrit dans le journal.

## Porte de lot

Dernière tâche d'un lot. Elle lance `C-TESTS`, `C-ECHECS`, et `C-NOMS` pour les lots 4, 5, 9, 10, 11. Les lots 6 et 7 ajoutent des tests : leur `C-NOMS` attend exactement les lignes ajoutées (guide du lot). Le lot 11 compare à des lignes de base transformées (les noms de tests contiennent le chemin du module). Elle écrit le bilan du lot dans `implementation.md`.

## Ligne de base

| Fichier dans `validation/` | Contenu | Créé par |
|----------------------------|---------|----------|
| `base-tests.txt` | sortie complète de la suite | T003 |
| `base-echecs.txt` | lignes `test ... FAILED`, triées | T003 |
| `base-noms.txt` | noms de tous les tests, triés | T004 |
| `base-clippy.txt` | sortie de l'analyse | T005 |
| `base-format.txt` | fichiers signalés par le contrôle de format | T006 |
| `base-linux-echecs.txt` | même liste, sur Linux | T021 |

Règles :

- Une ligne de base vaut pour une machine. Mac et Linux ont chacune la leur.
- Elle se refait après un lot qui ajoute ou retire des tests volontairement. Le guide du lot le dit. Sinon elle ne change pas.
- Elle se refait si `main` a reçu des tests d'une autre session. Voir `quickstart.md`, « Ligne de base périmée ».

## Preuve

Fichier `.txt` dans `validation/`, nommé `lotNN-...` ou `lotNN-Txxx-...`. Une preuve est la sortie d'une commande, pas un résumé.
Les sorties complètes des tests de lot ne sont pas commitées (plusieurs Mo). Les listes d'échecs et de noms le sont.

## Journal

`implementation.md`. Une entrée par tâche (format dans `contracts/regles-implementeur.md`), un bilan par lot, une section « Idées ».
