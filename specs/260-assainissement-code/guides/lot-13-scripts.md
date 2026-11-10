# Lot 13 - Scripts contrôlés (T128-T137)

But : les scripts Python et shell de `scripts/` passent un contrôle automatique, local et distant. Mesures : `releves.md` section 11 (`sed -n '119,126p'`).
Branche : `session-260-lot13-scripts`. Worktree : `.worktrees/260-lot13-scripts`. Aucune compilation Rust. Les scripts distribués ne changent **pas de comportement** (spec : FR23, R6.8).
**Interdit absolu : exécuter un script vivant** (`install*.sh`, `update.py`, `federate-ssh.sh`, `bridget-relance.sh`) pour « tester ». Seules vérifications : `ruff check`, `bash -n`, `python3 -m py_compile`.
Outils présents : `ruff` 0.14, `gitleaks`. Absents : `shellcheck`, `actionlint`. Ne pas les installer (R6.7).
Outils de contrôle de syntaxe : `python3 -m py_compile <fichier>` crée un dossier `__pycache__` ; le supprimer ensuite (`find scripts -name __pycache__ -prune -exec rm -rf {} +`).

## T128 - Constat

```sh
ruff check scripts --select E9,F,B,S --statistics > "$V/lot13-ruff-avant.txt" 2>&1; tail -20 "$V/lot13-ruff-avant.txt"
for f in scripts/*.sh; do bash -n "$f" || echo "SYNTAXE: $f"; done 2>&1 | tee "$V/lot13-bash-avant.txt"
grep -L 'set -e' scripts/*.sh | tee "$V/lot13-sans-set-e.txt"
```

Attendu : environ 137 signalements ruff ; 6 scripts sans `set -e`. Commit : `docs(specs): Relever l'etat des scripts avant le lot 13`.

## T129 - Configuration explicite de ruff

Créer `ruff.toml` à la racine :

```toml
# Contrôle des scripts (programme 260, lot 13). Les règles ignorées sont justifiées une à une.
target-version = "py39"          # le Python système de l'outil de mise à jour Mac est le 3.9.6
include = ["scripts/**/*.py"]

[lint]
select = ["E9", "F", "B", "S"]
ignore = [
  "S101",  # assert : utilisé par les scripts de test
  "S603",  # subprocess avec liste d'arguments, sans shell : voulu pour des outils internes
  "S607",  # chemin partiel d'exécutable (git, ssh) : résolu par le PATH de l'opérateur
]
```

Avant d'ignorer S603 : vérifier qu'aucun script n'appelle un shell avec une chaîne : `ruff check scripts --select S602,S604,S605` doit donner « All checks passed! » (c'est le cas au constat). Un résultat non vide : ne pas ignorer, noter.
Vérifier : `ruff check scripts` : le total baisse d'environ 94 (54 + 22 + 18). Commit : `chore(scripts): Configurer ruff avec les regles ignorees justifiees`.

## T130 - F401 et B904

- F401 (imports inutiles, 4) : `ruff check scripts --select F401 --fix`. Relire `git diff` : seules des lignes `import` disparaissent.
- B904 (11) : dans un `except ... as e:`, un `raise X(...)` doit devenir `raise X(...) from e` (ou `from None` si l'erreur d'origine est volontairement cachée : à choisir selon le contexte, lire 5 lignes autour). Ne rien changer d'autre.
Vérifier : `python3 -m py_compile <chaque fichier modifié>`, puis `ruff check scripts --select F401,B904`. Commit : `style(scripts): Corriger les imports inutiles et chainer les exceptions`.

## T131 - B905, B007, B023

- B905 (3) : `zip(a, b)` sans `strict=`. Le paramètre `strict` n'existe que depuis Python 3.10 ; le Python système cible est le 3.9.6 : **ne pas l'ajouter**. Avec `target-version = "py39"` (T129), ruff ne signale plus B905 : vérifier que le compte tombe à 0. S'il reste un signalement, ajouter `# noqa: B905` avec la raison `Python 3.9 : zip(strict=) indisponible`.
- B007 (2) : variable de boucle inutilisée : la renommer `_nom`.
- B023 (2) : fonction définie dans une boucle qui lit la variable de boucle. Si la fonction est appelée dans le tour où elle est définie : sans risque, `# noqa: B023` avec la raison. Si elle est stockée et appelée plus tard : **ne pas modifier**, noter le lieu.
Vérifier : `py_compile`, `ruff check scripts --select B905,B007,B023`. Commit : `style(scripts): Traiter les avertissements de boucle`.

## T132 - S110, S108, S105, S606, S324

Pour chaque occurrence : lire 8 lignes autour, puis choisir. Aucun changement de comportement.

| Règle | Cas | Traitement |
|-------|-----|------------|
| S110 (4) | `except: pass` | ajouter une ligne de commentaire qui dit pourquoi l'erreur est ignorée ; si la raison est inconnue : `# noqa: S110` avec `raison non établie, à revoir` |
| S108 (10) | chemin `/tmp` écrit en dur | `# noqa: S108` + `chemin de test fixe, sans donnée sensible` seulement si c'est vrai (script de test) ; sinon noter |
| S105 (2) | « mot de passe » codé en dur | faux positifs attendus (nom de variable) : `# noqa: S105` + `libellé, pas un secret`. **Si la valeur ressemble à un secret : arrêter, ne pas la recopier, la signaler dans le journal** |
| S606 (2) | `os.exec*` sans shell | `# noqa: S606` + raison |
| S324 (1) | hachage faible (`real_hook_profile171.py`) | lire l'usage : empreinte non cryptographique → `# noqa: S324` + raison ; sinon noter |

Commit : `style(scripts): Justifier les exceptions de securite des scripts`.

## T133 - `exec` de `sentinel.py` (D11)

Fichier : `scripts/sentinel.py`. Chercher : `exec(`. Le code exécuté vient du démon (`crates/bridget-daemon/src/sentinel244.rs` ou `sentinel.rs` si le lot 11 est intégré, par `include_str!`) : même origine de confiance. Action par défaut de D11 : ajouter, au-dessus de la ligne : `# Invariant : la source exécutée est celle du démon (include_str!), de même confiance que ce script (programme 260, D11).` et en fin de ligne `# noqa: S102`. Aucun changement de mécanisme. Lire D11 : `tranchée` avec une autre réponse : l'appliquer et l'écrire. Commit : `docs(scripts): Documenter l'invariant de l'exec du script sentinelle`.

## T134 - S310 et restes

- `scripts/update.py` (chercher `urlopen`, S310) : lire l'URL ; si c'est un schéma `https` fixe : `# noqa: S310` avec la raison `URL https fixe`. Ne pas modifier la logique.
- `ruff check scripts` : lister ce qui reste. Chaque reste est traité par la règle du tableau ci-dessus ou noté dans `validation/lot13-restes.txt`. Cible : 0 signalement, ou une liste justifiée.
Vérifier : `python3 -m py_compile scripts/update.py` (ne pas l'exécuter). Commit : `style(scripts): Justifier l'ouverture d'URL de la mise a jour`.

## T135 - Scripts de test shell sans `set -e`

`scripts/test-053-review-witness-selection.sh` et `scripts/test-git-pre-push-authorship.sh` (aucune référence : `rg -n '<nom>' . --glob '!specs/**'`). Lire chaque script, puis ajouter `set -eu` après la ligne `#!`. Contrôle : `bash -n`. **Ne pas exécuter.** Si le script contient une commande dont l'échec est attendu (une ligne qui teste un échec avec `if !` ou `||`), elle reste valide. Si le doute subsiste : ne pas modifier, noter. Commit : `chore(scripts): Arreter les scripts de test au premier echec`.

## T136 - Scripts vivants sans `set -e`

`scripts/bridget-relance.sh`, `scripts/federate-ssh.sh`, `scripts/install-k1.sh`, `scripts/registre-nature-reclasser.sh`. Lire chacun. **Ne pas ajouter `set -e`** (changement de comportement). Ajouter en tête, après `#!`, un commentaire : `# Pas de set -e : <raison lue dans le script> (programme 260, lot 13).` Raison attendue : des commandes dont l'échec est toléré, listées par nom (ex. `la commande 'kill' échoue si le processus est déjà parti`). Une raison non établie : `# Pas de set -e : raison non établie ; à revoir par le mainteneur.` Contrôle : `bash -n`. Commit : `docs(scripts): Documenter l'absence de set -e des scripts vivants`.

## T137 - Contrôle local, contrôle distant, porte

1. `Makefile` : cible `lint-scripts` (et `.PHONY`) :
   ```make
   lint-scripts:
   	ruff check scripts
   	for f in scripts/*.sh; do bash -n "$$f" || exit 1; done
   ```
   Vérifier : `make lint-scripts` code 0.
2. `.github/workflows/rust.yml` : ajouter un job `scripts` (même squelette que `clippy`, `continue-on-error: true` au début) : `pipx run ruff==0.14.13 check scripts` puis la boucle `bash -n`. Version de ruff : celle de `ruff --version` au moment de la tâche, écrite dans le journal.
3. Porte : `ruff check scripts` : 0 ou liste justifiée ; `for f in scripts/*.sh; do bash -n "$f"; done` : code 0 ; `python3 scripts/check-docs.py` et `bash scripts/check-secrets.sh` : code 0 ; `python3 -m unittest discover -s scripts/tests -p 'test_install.py' -v` : mêmes résultats qu'avant le lot (comparer avec un passage sur `origin/main`). Aucun `C-TESTS` (pas de Rust).
4. Nettoyer : `find scripts -name __pycache__ -prune -exec rm -rf {} +`. Bilan dans `implementation.md`. Fin de lot (`quickstart.md`).
Commit : `ci(scripts): Controler les scripts en local et en integration continue`.

## Arrêts propres à ce lot

- Un script vivant est exécuté par erreur : le dire dans le journal sans attendre, vérifier l'effet (`git status`, processus) et arrêter le lot.
- Une valeur ressemblant à un secret est vue : ne pas la recopier ; écrire seulement le nom du fichier et la ligne.
- Un correctif demande de comprendre la logique d'un script de plus de 300 lignes : ne pas corriger, noter.
