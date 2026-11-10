# Règles de l'implémenteur - programme 260

Ces règles valent pour toutes les tâches. Une tâche ne les répète pas.
En cas de conflit entre une tâche et une règle, la règle gagne. Écrire le conflit dans `implementation.md`.

## R1 - Un lot, une branche, un worktree

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget
git fetch origin && git status --short          # attendu : aucune ligne
git worktree add .worktrees/260-lotNN-nom -b session-260-lotNN-nom origin/main
cd .worktrees/260-lotNN-nom
```

- Un seul lot ouvert à la fois.
- Faire un premier commit avant le premier Cargo. Sans commit propre, l'outil de build retire le worktree.
- Fin de lot : intégration à `main`, push, retrait du worktree et de la branche (procédure dans `quickstart.md`).

## R2 - Cargo seulement par l'outil du projet

Interdit : `cargo build`, `cargo test`, `cargo check`, `cargo clippy` en direct.
Autorisé : `make build`, `make test`, `python3 scripts/build.py cargo ...`.
Exception : `cargo fmt` ne compile pas. Il s'appelle en direct, et seulement à partir du lot 10.

## R3 - Preuve de comportement constant

- Lot ordinaire : `C-ECHECS` du lot est identique à `validation/base-echecs.txt`.
- Déplacement de code ou de tests : `C-NOMS` est identique à `validation/base-noms.txt`.
- Un test en échec absent de la ligne de base : le rejouer seul trois fois. Trois succès : test instable, le noter. Un échec : régression, annuler le commit fautif (`git revert`).

## R4 - Règles d'arrêt

- Deux échecs sur la même hypothèse : observer une donnée réelle (sortie complète du compilateur, `git diff`, contenu du fichier) avant un troisième essai.
- Trois échecs : arrêt de la tâche. Annuler les changements non commités de la tâche (`git checkout -- <fichiers>`). Écrire dans `implementation.md` : tâche, commande, erreur exacte, hypothèses testées. Passer à la tâche suivante qui n'en dépend pas.
- Disque : `df -g / | tail -1`. Moins de 12 Gio libres : arrêt complet.
- Conflit Git sur un fichier déplacé ou renommé : arrêt du lot. Ne pas résoudre à la main.
- Une commande d'un guide échoue par erreur de syntaxe : l'ajuster et noter l'ajustement. Ce n'est pas un échec de la tâche.

## R5 - Commits

- Une tâche = un commit, sauf si la tâche demande un commit par élément.
- Format strict : `git commit -m "type(scope): Description"`. Types : `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `style`.
- Aucune trace d'outil automatique : pas de `Co-Authored-By`, pas de « Generated with », pas de nom de modèle.
- Ne jamais modifier `git config user.name` ni `user.email`. Ne jamais utiliser `--no-verify`.
- Après chaque commit : cocher la tâche dans `tasks.md` et ajouter une entrée dans `implementation.md`.

## R6 - Interdits

1. Modifier `WrapperToDaemon`, `DaemonToWrapper` ou tout type sérialisé du protocole.
2. Modifier un format durable : schéma SQLite, fichier d'état, journal, spool.
3. Modifier une chaîne littérale durable : code d'erreur, nom de fichier, clé JSON, préfixe d'identifiant.
4. Modifier un droit, une garde d'identité ou une permission.
5. Arrêter, redémarrer ou remplacer T3. Utiliser `kill`. Terminer un processus d'un autre travail.
6. `git stash` nu, `git push --force`, réécriture d'historique.
7. Ajouter une dépendance Rust, Python ou Gradle.
8. Exécuter un script vivant de `scripts/` pour « tester » (`install*.sh`, `update.py`, `federate-ssh.sh`, `bridget-relance.sh`). Vérifier par `bash -n` ou `python3 -m py_compile`.
9. Recopier une adresse IP, un jeton ou un chemin de secret dans un fichier suivi.
10. Élargir une tâche. Une idée d'amélioration va dans `implementation.md`, section « Idées », sans code.

## R7 - Portes de décision

Lire `decisions.md` depuis `origin/main` au début de chaque lot. Appliquer le mode d'emploi en tête de ce fichier.

## R8 - Activation

Selon D1. Par défaut : l'implémenteur intègre et pousse les sources. Il n'active rien sur les machines.
Chaque lot qui modifie du code exécutable porte « activation en attente » dans `implementation.md`.
Le mot « livré » est interdit sans activation vérifiée.

## Format

- Avant le lot 10 : ne jamais lancer `cargo fmt`. Garder la forme du code voisin.
- Après le lot 10 : `make fmt` avant chaque commit.

## Commandes nommées

Environnement de test, à poser une fois par terminal :

```sh
umask 077
R=$(mktemp -d /private/tmp/b.XXXX); mkdir -m 700 "$R/state" "$R/tmp"
V=specs/260-assainissement-code/validation
ENVT=(env -i HOME="$HOME" BRIDGET_HOME="$R/state" BRIDGET_SOCKET="$R/state/bridget.sock" TMPDIR="$R/tmp" CARGO_TARGET_DIR="$PWD/target" PATH="$HOME/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin")
F=(--offline --locked --workspace --features bridget-daemon/test-support)
```

Sur Linux, remplacer `/private/tmp` par `/tmp` et retirer `/opt/homebrew/bin`.

| Nom | Commande | Attendu |
|-----|----------|---------|
| `C-CHECK` | `python3 scripts/build.py cargo check --all-targets "${F[@]}"` | code de retour 0 |
| `C-CLIPPY` | `python3 scripts/build.py cargo clippy --all-targets "${F[@]}" 2>&1 \| tee "$V/<nom>-clippy.txt"` | selon la tâche |
| `C-TESTS` | `"${ENVT[@]}" python3 scripts/build.py cargo test "${F[@]}" --no-fail-fast -- --test-threads=4 2>&1 \| tee "$V/<nom>-tests.txt"` | la suite se termine |
| `C-ECHECS` | `grep -E '^test .* \.\.\. FAILED' "$V/<nom>-tests.txt" \| sort > "$V/<nom>-echecs.txt"; diff "$V/base-echecs.txt" "$V/<nom>-echecs.txt"` | aucune ligne |
| `C-NOMS` | `"${ENVT[@]}" python3 scripts/build.py cargo test "${F[@]}" -- --list 2>/dev/null \| grep ': test$' \| sort > "$V/<nom>-noms.txt"; diff "$V/base-noms.txt" "$V/<nom>-noms.txt"` | aucune ligne |
| `C-CRATE` | `"${ENVT[@]}" python3 scripts/build.py cargo test --offline --locked -p <crate> --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4` | mêmes échecs que la base pour ce crate |
| `C-DISQUE` | `df -g / \| tail -1` | 12 Gio libres ou plus |

`<nom>` vaut `lotNN` ou `lotNN-Txxx`. Pour `-p bridget-transport` ou `-p bridget-core`, retirer `--features bridget-daemon/test-support` si Cargo le refuse, et le noter.

Rythme : `C-CHECK` après chaque tâche qui touche du Rust ; `C-CRATE` toutes les 5 tâches ; `C-TESTS` puis `C-ECHECS` à la porte de chaque lot ; `C-NOMS` à la porte des lots 4, 5, 9, 10, 11.

## Journal `implementation.md`

Une entrée par tâche :

```markdown
### T0xx - titre court
- Statut : fait | arrêté | différé (porte Dn)
- Commit : `abc1234` type(scope): description
- Fichiers : chemins
- Vérifications : commande -> résultat (fichier de preuve)
- Écarts : ajustement de commande, action par défaut appliquée, test instable
- Non vérifié : ce qui reste à prouver
```

Les preuves sont des fichiers `.txt` dans `validation/`. L'extension `.log` est ignorée par Git.
