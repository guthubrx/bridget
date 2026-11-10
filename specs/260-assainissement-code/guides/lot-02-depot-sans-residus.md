# Lot 2 - Dépôt sans résidus (T025-T030)

But : retirer de la racine les 15 fichiers suivis qui sont des résidus de travail. Documenter les fichiers suivis malgré l'exclusion. Mesures : `releves.md` section 14.
Branche : `session-260-lot02-residus`. Worktree : `.worktrees/260-lot02-residus`. Aucun code exécutable ne change. Aucun redémarrage.
Ce lot ne supprime aucune donnée : il déplace. Les fichiers déplacés restent dans l'historique et dans le dossier d'archive.

## T025 - Constat de départ

Commandes, depuis le worktree :

```sh
git ls-files -ci --exclude-standard | wc -l                      # attendu : 345
git ls-files .wip-mesures .wip-reprise-preflight.txt watch_20260823-175033 | wc -l   # attendu : 15
git ls-files -ci --exclude-standard | sed -E 's#^(audits|specs/[^/]+)/.*#\1#' | sort | uniq -c | sort -nr | head
```

Écrire les trois sorties dans `validation/lot02-constat.txt`. Les chiffres peuvent différer de quelques unités si `main` a bougé : noter l'écart, continuer. Un écart de plus de 20 fichiers : arrêter le lot et le noter (le dépôt a changé de forme).
Lire D3 dans `decisions.md` (`origin/main`). Seule l'action par défaut est prévue dans ce guide. Si D3 est `tranchée` avec une autre réponse : appliquer la réponse de François, écrire laquelle dans le journal.

## T026 - Archiver les résidus de la racine

Cible : `specs/archives/residus-racine-20261110/` (dossier à créer). Aucun de ces fichiers n'est lu par le code ni par un script : vérifier d'abord.

```sh
rg -n 'wip-mesures|wip-reprise-preflight|watch_20260823' --glob '!specs/archives/**' --glob '!.git/**' . | head
```

Attendu : aucune ligne hors `releves.md`, `spec.md`, `plan.md`, `guides/`, `tasks.md` de ce programme et hors `AGENTS.md` (cité dans les reçus). Une ligne dans un script ou dans le code : arrêter la tâche et la noter.
Déplacer avec `git mv` (conserve l'historique) :

```sh
mkdir -p specs/archives/residus-racine-20261110
git mv .wip-mesures specs/archives/residus-racine-20261110/wip-mesures
git mv .wip-reprise-preflight.txt specs/archives/residus-racine-20261110/
git mv watch_20260823-175033 specs/archives/residus-racine-20261110/
```

Vérifier : `git ls-files .wip-mesures .wip-reprise-preflight.txt watch_20260823-175033 | wc -l` donne 0 ; `git ls-files specs/archives/residus-racine-20261110 | wc -l` donne 15. Un fichier non suivi resté à la racine (`ls -a`) : ne pas le toucher, le noter.
Commit : `chore(depot): Archiver les residus de travail de la racine`.

## T027 - Prévenir le retour des résidus

Fichier : `.gitignore`. Ajouter, à la fin, avec un commentaire :

```gitignore
# Résidus de travail de la racine (programme 260, lot 2)
/.wip-*
/watch_*
```

Vérifier : `git check-ignore -v .wip-test watch_test` affiche deux lignes. `git status --short` ne change pas pour les fichiers déjà suivis (ils sont dans `specs/archives/`).
Commit : `chore(depot): Ignorer les residus de travail de la racine`.

## T028 - Documenter les fichiers suivis malgré l'exclusion (action par défaut de D3)

Fichier : `.gitignore`. Ajouter un commentaire (aucune règle nouvelle) qui liste les 4 exceptions historiques :

```gitignore
# Fichiers suivis par Git bien qu'ils correspondent à une règle ci-dessus (décision D3 du programme 260 : conservés) :
#   audits/                          317 fichiers, rapports d'audit d'octobre 2026
#   specs/159-edition-file/validation/*.log   26 journaux de la session 159
#   AGENTS.md                        consignes du projet
#   .specify/feature.json            pointeur de la fonctionnalité courante
```

Vérifier : `git ls-files -ci --exclude-standard | wc -l` donne 345 moins 0 (inchangé : on documente, on ne retire pas). `git diff --stat` ne montre que `.gitignore`.
Commit : `docs(depot): Lister les fichiers suivis malgre l'exclusion`.

## T029 - Inventaire des gros fichiers suivis

Aucune suppression. Écrire la liste des fichiers suivis de plus d'un Mio :

```sh
git ls-files -z | xargs -0 stat -f '%z %N' 2>/dev/null | awk '$1 > 1048576' | sort -nr > specs/260-assainissement-code/validation/lot02-gros-fichiers.txt
```

(Linux : `stat -c '%s %n'` au lieu de `stat -f '%z %N'`.) Lire le fichier. Pour chaque ligne, écrire dans le journal une proposition : garder, archiver ailleurs, ou sortir de Git. **Proposition seulement** : ne rien déplacer sans décision de François.
Commit : `docs(specs): Inventorier les gros fichiers suivis`.

## T030 - Porte du lot

1. `bash scripts/check-secrets.sh` : attendu code 0.
2. `python3 scripts/check-docs.py` : attendu code 0.
3. `git ls-files .wip-mesures .wip-reprise-preflight.txt watch_20260823-175033 | wc -l` : 0 (critère SC03).
4. `ls -a | rg 'wip|watch'` : aucune ligne pour les fichiers du lot.
5. Aucune compilation n'est requise : le lot ne touche pas au code. Pas de `C-TESTS`.
6. Bilan dans `implementation.md`. Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- Un script ou du code lit l'un des résidus : arrêter T026, ne rien déplacer.
- `scripts/check-secrets.sh` signale une valeur dans un fichier déplacé : ne pas committer, noter le fichier (sans recopier la valeur) et arrêter le lot.
