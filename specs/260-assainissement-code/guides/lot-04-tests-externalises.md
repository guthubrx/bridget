# Lot 4 - Tests hors des fichiers de production (T036-T053)

But : sortir les modules de tests de plus de 200 lignes des fichiers de production, un fichier voisin par module. Les 35 modules concernés pèsent plus de 110 000 lignes. Après le lot, `daemon.rs` ne contient plus 32 000 lignes de tests et le lot 5 devient praticable. Mesures : `releves.md` section 4.
Branche : `session-260-lot04-tests`. Worktree : `.worktrees/260-lot04-tests`. Aucun comportement ne change : les tests sont déplacés, jamais modifiés.
Preuve : `C-NOMS` identique à `validation/base-noms.txt` (même liste de tests) et `C-ECHECS` identique à `validation/base-echecs.txt`.

## Outil

`specs/260-assainissement-code/outils/extraire_module_tests.py`. Sa règle est dans son en-tête. Résumé :

- `--list FICHIER` montre chaque module `#[cfg(test)]` de colonne 0 avec sa taille et le verdict « a sortir » (plus de 200 lignes) ou « reste (petit) ».
- `--extract FICHIER LIGNE` déplace le module dont la ligne `mod X {` est LIGNE vers un fichier voisin, et laisse `#[cfg(test)] #[path = "..."] mod X;`.
- Il refuse (code 2) au lieu de deviner : fichier cible existant, `mod y;` dans le corps, attribut sur plusieurs lignes, reconstruction différente.
- Il n'appelle ni Cargo ni Git. Le corps n'est pas désindenté (le lot 10 le fera).

Variables à poser :

```sh
O=specs/260-assainissement-code/outils/extraire_module_tests.py
```

## Règle d'ordre dans un fichier

Un fichier peut avoir plusieurs modules « a sortir ». **Extraire du numéro de ligne le plus grand au plus petit.** Extraire un module décale les lignes des modules qui le suivent, pas celles qui le précèdent. Relancer `--list` après chaque extraction si un doute subsiste.

## T036 - Préparation

1. Vérifier que `validation/base-noms.txt` et `validation/base-echecs.txt` existent (lot 0). Sinon : arrêter, faire le lot 0.
2. Le worktree doit être propre avant le premier Cargo (R1). Chaque tâche committe avant que la suivante lance `C-CHECK`. `git status --short` ne montre aucune ligne avant T037.
3. Test à sec de l'outil sur une copie (sans Cargo) :

```sh
T=$(mktemp -d /tmp/ext.XXXX); cp crates/bridget-daemon/src/registry.rs $T/
python3 -I $O --list $T/registry.rs
python3 -I $O --extract $T/registry.rs <LIGNE affichée>
ls $T; rm -rf $T
```

Attendu : un fichier `registry_tests.rs` apparaît dans `$T`, et `registry.rs` contient `#[path = "registry_tests.rs"]` suivi de `mod tests;`. Un refus ou une erreur Python : arrêter le lot et le noter. Pas de commit pour cette tâche : l'écrire dans le journal.

## T037 à T052 - Un fichier par tâche

Chaque tâche suit la même recette. `<F>` est le fichier. `C-CHECK` compile les tests : c'est la vérification principale.

```sh
python3 -I $O --list <F>                       # relever les lignes marquées "a sortir"
python3 -I $O --extract <F> <LIGNE>            # du plus grand numéro au plus petit
# ... une commande par module "a sortir"
python3 -I $O --list <F>                       # attendu : plus aucun "a sortir"
C-CHECK                                        # attendu : code 0
git status --short                             # attendu : <F> modifié + N fichiers neufs
git add <F> <fichiers neufs> && git commit -m "refactor(tests): Sortir les tests de <nom du fichier>"
```

Si `C-CHECK` échoue : lire l'erreur. Une cause fréquente est un chemin `#[path]` relatif mal résolu pour un fichier qui n'est pas à la racine du dossier. Dans ce cas : annuler (`git checkout -- <F>`, supprimer les fichiers neufs listés par l'outil avec `rm <chemin exact>`), noter l'erreur, passer au fichier suivant (R4, une seule tentative d'ajustement, puis arrêt de la tâche).
Après chaque 5 tâches : `C-CRATE` pour les deux crates (`bridget-daemon`, `bridget-transport`).

| Tâche | Fichier (depuis la racine du dépôt) | Modules à sortir attendus |
|-------|-------------------------------------|----------------------------|
| T037 | `crates/bridget-daemon/src/daemon.rs` | 7 |
| T038 | `crates/bridget-daemon/src/daemon/native_delegation.rs` | 3 |
| T039 | `crates/bridget-daemon/src/remote_worker.rs` | 1 |
| T040 | `crates/bridget-daemon/src/mcp.rs` | 1 |
| T041 | `crates/bridget-daemon/src/wrapper.rs` | 5 |
| T042 | `crates/bridget-transport/src/codex_app_server.rs` | 1 |
| T043 | `crates/bridget-daemon/src/attach.rs` | 1 |
| T044 | `crates/bridget-daemon/src/cli.rs` | 4 |
| T045 | `crates/bridget-transport/src/protocol.rs` | 3 |
| T046 | `crates/bridget-daemon/src/t3code.rs` | 1 |
| T047 | `crates/bridget-daemon/src/delegation.rs` | 1 |
| T048 | `crates/bridget-transport/src/claude_stream_json.rs` | 1 |
| T049 | `crates/bridget-transport/src/acp.rs` | 1 |
| T050 | `crates/bridget-daemon/src/delegation_mcp.rs` | 3 |
| T051 | `crates/bridget-daemon/src/daemon/mobile_operations.rs` | 1 |
| T052 | `crates/bridget-daemon/src/registry.rs` | 1 |

Le nombre attendu peut différer de 1 ou 2 si `main` a changé : noter l'écart. Un module « a sortir » absent de la liste est traité aussi.
Cas particuliers :

- Le fichier est déjà en grande partie externalisé (forme `#[path]` existante) : ne pas y toucher. L'outil ne voit que les modules en ligne.
- Un module de moins de 201 lignes reste en place (forme petite). Ne pas l'extraire à la main.
- Un module `#[cfg(test)]` isolé qui n'est pas un `mod` (une fonction, un `use`) reste en place.
- Nom de fichier déjà pris : l'outil refuse. Arrêter la tâche, noter le nom. Ne pas renommer à la main.

## T053 - Porte du lot

1. `C-TESTS` avec `<nom>` = `lot04`, puis `C-ECHECS` : aucune ligne de différence.
2. `C-NOMS` avec `<nom>` = `lot04` : aucune ligne de différence. Une ligne en moins ou en plus signifie qu'un test a été perdu ou dupliqué : chercher le nom (`rg -n '<nom du test>' crates`), corriger ou annuler le commit du fichier concerné.
3. Mesure du critère SC05 (les tests en ligne restants font moins de 15 000 lignes) :

```sh
for f in $(rg -l '^#\[cfg\(test\)\]' crates --glob '*.rs'); do python3 -I $O --list $f; done | awk -F' - ' '{split($2,a," "); s+=a[1]} END{print s, "lignes de tests en ligne"}' | tee "$V/lot04-restant.txt"
```

4. `wc -l crates/bridget-daemon/src/daemon.rs` : écrire la valeur dans le journal (le lot 5 part de ce chiffre).
5. Bilan dans `implementation.md`. Ce lot ne change pas le comportement : pas de mention « activation en attente » (aucun comportement), mais le binaire reste à reconstruire. Fin de lot (`quickstart.md`).

## Arrêts propres à ce lot

- `C-NOMS` montre une différence que l'extraction n'explique pas : arrêt du lot (R3).
- Conflit Git sur un fichier déplacé pendant l'intégration : arrêt du lot (R4).
- Disque sous 12 Gio : arrêt complet. Les nouveaux fichiers dupliquent peu (déplacement), mais la compilation remplit le cache.
- Ne pas lancer `cargo fmt` pour « réparer l'indentation » : le corps déplacé reste décalé de 4 espaces jusqu'au lot 10.
