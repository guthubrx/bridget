# Preuves de l'implémenteur - programme 260

Ce dossier reçoit les preuves : sorties de commandes, listes de comparaison, constats de départ.
Il est vide au départ. Les fichiers sont créés par les tâches (voir `tasks.md` et `guides/`).

## Règles

- Extension `.txt` ou `.md`. L'extension `.log` est ignorée par Git.
- Nommage : `base-*.txt` (lignes de base, lot 0), `lotNN-*.txt` (lot NN), `lotNN-Txxx-*.txt` (tâche précise).
- Une preuve est la sortie brute d'une commande. Un résumé ne suffit pas.
- Les sorties complètes de tests (`*-tests.txt`) pèsent plusieurs Mo : elles ne se committent pas. Le fichier `.gitignore` de ce dossier les exclut. Les listes d'échecs (`*-echecs.txt`) et de noms (`*-noms.txt`) se committent.
- Aucune adresse IP, aucun jeton, aucun chemin de secret dans ces fichiers (dépôt public).

## Lignes de base (lot 0)

| Fichier | Contenu | Tâche |
|---------|---------|-------|
| `base-tests.txt` | sortie complète de la suite (non commitée) | T003 |
| `base-echecs.txt` | lignes `test ... FAILED`, triées | T003 |
| `base-noms.txt` | noms de tous les tests, triés | T004 |
| `base-clippy.txt` | sortie de l'analyse statique | T005 |
| `base-format.txt` | fichiers signalés par le contrôle de format | T006 |
| `base-linux-echecs.txt` | liste d'échecs sur Linux | T021 |

Voir `data-model.md` pour les règles de péremption.
