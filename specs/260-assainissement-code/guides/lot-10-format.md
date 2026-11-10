# Lot 10 - Format uniforme (T114-T118)

But : un seul style d'écriture partout, appliqué par l'outil officiel `rustfmt`, en **un seul commit isolé**. 120 fichiers sur 309 sont aujourd'hui signalés (`releves.md` section 2). Choix : `research.md` C11.
Branche : `session-260-lot10-format`. Worktree : `.worktrees/260-lot10-format`. Code exécutable : forme seulement (même programme compilé). « Activation en attente » n'est pas nécessaire : le binaire est identique à une signature de fichier près ; l'écrire quand même dans le journal par prudence (R8).
**Porte D9 (bloquante pour ce lot).** Le formatage de masse touche 120 fichiers. Toute autre branche ouverte entrerait en conflit. Lire D9 dans `decisions.md` (`origin/main`) avant la première commande.

## T114 - Porte D9 et dépôt calme

1. D9 `en attente` : **le lot ne démarre pas.** Écrire dans le journal « différé (porte D9) » pour T114 à T118. Passer au lot suivant de la liste. Ne rien commiter dans ce lot.
2. D9 `tranchée` : appliquer la réponse (moment indiqué par François). Ensuite, vérifier que le dépôt est calme :

```sh
git fetch origin
git worktree list                                  # attendu : seulement le dépôt principal et ce worktree
git branch -r | rg -v 'origin/(main|HEAD)'         # branches distantes d'autres sessions : à noter
git log origin/main --since='2 hours ago' --oneline | wc -l   # attendu : 0
```

Une autre branche de travail ouverte, ou un commit de moins de 2 heures sur `main` : attendre (noter l'heure) et relancer. Écrire le résultat dans `validation/lot10-calme.txt`.
3. Prérequis : les lots 4, 5 et 9 sont intégrés (ils déplacent beaucoup de code ; formater après évite de reformater deux fois). Sinon : écrire « différé (lots 4, 5, 9 non intégrés) ».

## T115 - Formatage de masse (un seul commit)

```sh
cargo fmt --all -- --check 2>&1 | rg '^Diff in' | wc -l     # avant : nombre de blocs de différences (noter)
cargo fmt --all
git status --short | wc -l                                  # environ 120 fichiers modifiés
git diff --stat | tail -1
```

`cargo fmt` est l'exception de la règle R2 : il ne compile pas. Il lit `rust-toolchain.toml` (rustfmt 1.92.0).
Vérifier :

1. `cargo fmt --all -- --check` : code 0, aucune sortie.
2. `C-CHECK` : code 0 (le formatage ne change pas le sens).
3. `git diff -U0 -w --stat | tail -1` : ignore les espaces. Les lignes changées hors espaces correspondent aux retours à la ligne ; pas d'inquiétude.
4. Aucun fichier hors `crates/` n'est modifié : `git status --short | rg -v '^ M crates/'` ne montre aucune ligne.

Commit unique : `style: Appliquer rustfmt sur tout le depot`. **Ne rien ajouter d'autre à ce commit** (sinon il n'est plus isolé).
Noter le SHA : `git rev-parse HEAD`.

## T116 - Ignorer ce commit dans `git blame`

Créer `.git-blame-ignore-revs` à la racine :

```text
# Commits de forme seulement (programme 260)
<SHA complet du commit de T115>
```

Documenter, dans la section « Contrôles avant livraison » de la documentation de développement (lot 1, T022), une ligne : `git config blame.ignoreRevsFile .git-blame-ignore-revs` (réglage local de chaque développeur, non commité). Ne pas exécuter `git config` pour l'identité (R5). Le réglage `blame.ignoreRevsFile` est local au dépôt : l'écrire dans le journal comme conseil, ne pas l'imposer.
Commit : `chore(depot): Ignorer le commit de formatage dans blame`.

## T117 - Format bloquant

1. `.github/workflows/rust.yml` : retirer `continue-on-error: true` du job `format`.
2. `Makefile` : la cible `lint` dépend déjà de `fmt-check` (lot 1, T008). Vérifier : `make -n lint` affiche `cargo fmt --all -- --check`.
3. Dans la documentation de développement : `make fmt` avant chaque commit (règle Format de `contracts/regles-implementeur.md`).
Commit : `ci(rust): Rendre le controle de format bloquant`.

## T118 - Porte du lot

1. `cargo fmt --all -- --check` : code 0 (SC10, 0 fichier signalé).
2. `C-TESTS` (`<nom>` = `lot10`), `C-ECHECS` : aucune différence.
3. `C-NOMS` : aucune différence.
4. Bilan : SHA du commit de formatage, nombre de fichiers touchés. Fin de lot (`quickstart.md`).
5. **Informer les autres sessions** : le commit de formatage change presque toutes les lignes de 120 fichiers. Toute branche ouverte doit se réaligner. Écrire cette phrase dans le bilan. Ne pas envoyer de bulletin général (consigne « Communications attentives » d'`AGENTS.md`).

## Arrêts propres à ce lot

- `cargo fmt` plante ou modifie un fichier hors `crates/` : annuler (`git checkout -- .`), noter, arrêter le lot.
- `C-CHECK` échoue après le formatage : cas rare (macro sensible à l'espace). Annuler tout (`git checkout -- .`), noter le fichier fautif, arrêter le lot. Ne pas corriger à la main : le commit doit rester un formatage pur.
- Un autre commit arrive sur `main` pendant le lot : réaligner avant le commit de T115 en refaisant `cargo fmt --all`. Après le commit, un conflit de rebase est résolu en **refaisant** `cargo fmt --all` sur la version de `main`, pas à la main.
