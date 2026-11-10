# Démarrage rapide 260 - pour l'implémenteur

Ce fichier couvre le lot 0 (T001-T007), la clôture (T155-T157), la fin de lot et la ligne de base périmée.
Lire d'abord `contracts/regles-implementeur.md` (règles R1 à R8, commandes nommées). Ensuite `decisions.md`. Ensuite `tasks.md`.

## Avant la première tâche

1. Vous travaillez depuis `/Users/moi/Nextcloud/10.Scripts/64.bridget`. Le dépôt est PUBLIC. Ne jamais écrire d'adresse IP, de jeton ni de chemin de secret dans un fichier suivi.
2. Un lot = un worktree = une branche (règle R1). Un seul lot ouvert.
3. Cargo passe toujours par `python3 scripts/build.py cargo ...` ou `make` (règle R2).
4. Une tâche = un commit. Après chaque commit : cocher la tâche, écrire l'entrée du journal.
5. En cas de doute sur une règle, la règle gagne. Écrire le doute dans `implementation.md`.

## Lot 0 - Fondations (T001-T007)

Aucun code exécutable ne change. Ce lot crée le journal et les lignes de base.

| Tâche | Action | Résultat attendu |
|-------|--------|------------------|
| T001 | Lire `contracts/regles-implementeur.md` et `decisions.md`. Créer `specs/260-assainissement-code/implementation.md` avec les sections : `# Journal 260`, `## Lots`, `## Idées`. Créer le dossier `validation/` s'il manque. | Le fichier existe. `git status --short` ne montre que lui. |
| T002 | Vérifier les outils : `rg --version`, `python3 --version`, `gh --version`, `ruff --version`, `git --version`, `python3 scripts/build.py --help`. Noter les absents (`shellcheck`, `actionlint`, `cargo-audit`, `cargo-deny` sont absents de la machine de rédaction). Lancer `C-DISQUE`. | Versions notées dans le journal. 12 Gio libres ou plus, sinon arrêt complet (R4). |
| T003 | Poser l'environnement de test (règles, « Commandes nommées »). Faire un premier commit propre sur la branche. Lancer `C-TESTS` avec `<nom>` = `base`, puis écrire `validation/base-echecs.txt`. | `validation/base-tests.txt` existe. `base-echecs.txt` liste les tests en échec (environ 40 sur Mac). La suite se termine. |
| T004 | Écrire `validation/base-noms.txt` avec la commande `C-NOMS` (la comparaison `diff` est sans objet : le fichier n'existe pas encore, retirer le `diff`). | Fichier trié d'environ 3813 lignes, une par test. |
| T005 | Lancer `C-CLIPPY` avec `<nom>` = `base`. Copier la sortie dans `validation/base-clippy.txt`. | Le fichier contient 3 erreurs et 385 avertissements (`releves.md` section 2). Un écart de quelques unités est possible : noter les chiffres lus. |
| T006 | Lancer `cargo fmt --all -- --check 2>&1 \| rg '^Diff in' \| sed -E 's/^Diff in (.*):[0-9]+:?$/\1/' \| sort -u > validation/base-format.txt`. C'est l'exception de la règle R2 : `cargo fmt` ne compile pas. Ne PAS lancer `cargo fmt --all` sans `--check`. | Environ 120 fichiers listés. Aucun fichier source modifié (`git status --short`). |
| T007 | Créer `validation/README.md` avec le texte de la section « Sorties de preuve » ci-dessous. Committer le journal, `validation/` et les fichiers `base-*.txt`. Intégrer à `main` (procédure « Fin de lot »). | `git ls-remote origin main` donne le SHA du dernier commit. |

Si T003 échoue avec plus de 100 échecs, la recette d'environnement est fausse. Relire « Commandes nommées », refaire `umask 077` et le dossier `$R`. Si l'échec persiste après deux essais, observer la sortie réelle (R4) puis arrêter le lot.

### Sorties de preuve (contenu de `validation/README.md`)

- Les fichiers `.txt` de ce dossier sont des preuves. Ne pas utiliser `.log` : Git l'ignore.
- Les sorties complètes de tests (`*-tests.txt`) pèsent plusieurs Mo. Elles ne se committent pas. Ajouter la ligne `*-tests.txt` dans `validation/.gitignore`, sauf `base-tests.txt` si sa taille reste sous 2 Mo.
- Les listes d'échecs (`*-echecs.txt`), de noms (`*-noms.txt`), de clippy et de format se committent.

## Fin de lot (procédure citée par la règle R1)

À faire quand la porte du lot est franchie (dernière tâche du lot) :

```sh
# 1. Dans le worktree du lot : tout est commité
git status --short                       # attendu : aucune ligne
git fetch origin
git log --oneline HEAD..origin/main      # commits arrivés pendant le lot

# 2. Si origin/main a avancé : réaligner
git rebase origin/main
# Conflit sur un fichier déplacé ou renommé : ARRÊT du lot (règle R4). Ne pas résoudre.
# Après rebase : relancer C-CHECK. Relancer C-TESTS + C-ECHECS si des fichiers Rust ont changé.

# 3. Intégrer depuis le dépôt principal (jamais de force)
cd /Users/moi/Nextcloud/10.Scripts/64.bridget
git status --short                       # attendu : aucune ligne
git merge --ff-only session-260-lotNN-nom
git push origin main
git ls-remote origin main                # doit afficher le SHA de HEAD
git rev-parse HEAD

# 4. Retirer le worktree et la branche, une fois l'intégration vérifiée
git worktree remove .worktrees/260-lotNN-nom
git branch -d session-260-lotNN-nom
git worktree prune
```

Vérifications :

- `git merge --ff-only` échoue : `main` a avancé. Retourner dans le worktree, refaire l'étape 2.
- `git branch -d` refuse : la branche n'est pas intégrée. Ne pas utiliser `-D`. Écrire l'arrêt dans le journal.
- `git worktree remove` refuse (fichiers non suivis) : lire `git status`, ne rien supprimer à l'aveugle. Conserver et noter.
- Écrire le bilan du lot dans `implementation.md` : tâches faites, arrêtées, différées ; preuves ; « activation en attente » si du code exécutable a changé (R8).
- Ne jamais écrire « livré » sans activation vérifiée (R8, D1).

## Ligne de base périmée

La ligne de base (`validation/base-*.txt`) est périmée si :

1. `git log --oneline <SHA du dernier lot intégré>..origin/main -- 'crates/**/*.rs'` montre des commits d'une autre session qui ajoutent ou retirent des tests ; ou
2. un lot a ajouté ou retiré des tests volontairement (le guide du lot le dit) ; ou
3. `C-ECHECS` montre un écart sur un test que personne n'a touché, et son rejeu seul trois fois réussit (test instable).

Procédure :

1. Se placer sur `origin/main` propre, dans un worktree neuf (règle R1).
2. Refaire T003 et T004 avec `<nom>` = `base2`. Garder les anciens fichiers `base-*.txt` jusqu'à la fin.
3. Remplacer `base-echecs.txt` et `base-noms.txt` par les nouveaux. Commit : `chore(specs): Actualiser la ligne de base de tests du programme 260`.
4. Écrire dans `implementation.md` la raison, la date, l'ancien et le nouveau nombre d'échecs.
5. Un nouvel échec qui apparaît seulement parce qu'un autre travail est arrivé sur `main` n'est pas une régression du lot. Le noter.

## Clôture (T155-T157)

| Tâche | Action | Résultat attendu |
|-------|--------|------------------|
| T155 | Pour chaque critère SC01 à SC14 de `spec.md`, relancer la commande de mesure indiquée dans le guide du lot concerné. Écrire un tableau `critère / mesure / lot / preuve` dans `implementation.md`, section « Bilan final ». Un critère non atteint s'écrit « non atteint » avec la raison (porte en attente, tâche arrêtée). | 14 lignes. Aucune ligne sans preuve ni raison. |
| T156 | Contrôle des portes : relire `decisions.md` sur `origin/main`. Pour chaque porte D1 à D16, écrire dans le bilan : tranchée (réponse appliquée) ou en attente (action par défaut appliquée, effet). | 16 lignes. |
| T157 | Nettoyage : `git worktree list` ne montre aucun worktree `260-*` ; `git branch --list 'session-260-*'` est vide ; `git branch -r --list 'origin/session-260-*'` est vide (sinon `git push origin --delete <branche>` pour les seules branches intégrées). Lancer `C-DISQUE`. Écrire le statut : « In Progress » si une tâche est arrêtée ou différée, sinon « Implemented, activation en attente » (D1). | Bilan commité et poussé sur `main`. |

## Quand s'arrêter

Voir R4. En résumé : deux échecs sur la même hypothèse, observer une donnée réelle. Trois échecs, arrêt de la tâche. Disque sous 12 Gio, arrêt complet. Conflit sur fichier déplacé, arrêt du lot.
