# Contrôles distants attendus - programme 260

STATUT : NON EXÉCUTÉ. Aucun de ces fichiers n'a tourné sur GitHub. Les tâches T010 et T020 les valident par un premier passage réel.
Le dépôt est PUBLIC. Même règle que `.github/workflows/repository-hygiene.yml` : actions épinglées par empreinte, droits en lecture seule, pas de jeton gardé.

## Règles communes

- `permissions: contents: read` au niveau du workflow.
- `actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4` avec `persist-credentials: false`. Cette empreinte est déjà utilisée par `repository-hygiene.yml`.
- Aucune autre action tierce (pas de cache, pas d'installeur de chaîne d'outils). Rust se prépare avec `rustup` déjà présent sur `ubuntu-latest`.
- Chaîne d'outils : 1.92.0, lue dans `rust-toolchain.toml` (`rustup show` l'installe).
- Cargo dans le workflow : appel direct autorisé. La règle R2 (outil du projet) vaut pour les machines de travail, car l'outil nettoie les caches locaux. Sur un exécuteur jetable, l'appel direct est correct.
- Options : `--offline` est impossible en CI (premier téléchargement). Utiliser `--locked`.

## Fichier 1 : `.github/workflows/rust.yml` (tâche T010)

Déclencheurs : `push` et `pull_request`.

| Job | Commande | Mode |
|-----|----------|------|
| `clippy` | `cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings` | bloquant dès T019 ; avant T019, `continue-on-error: true` |
| `format` | `cargo fmt --all -- --check` | `continue-on-error: true` jusqu'au lot 10 (D9), puis bloquant |
| `tests` | `cargo test --workspace --locked --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4` | `continue-on-error: true` jusqu'à T024 (D16) |

Squelette (les étapes `uses` gardent l'empreinte ci-dessus) :

```yaml
name: Rust
on:
  push:
  pull_request:
permissions:
  contents: read
jobs:
  clippy:
    runs-on: ubuntu-latest
    continue-on-error: true   # retirer à T019
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
        with:
          persist-credentials: false
      - run: rustup show
      - run: cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings
```

Les jobs `format` et `tests` suivent le même modèle. Le job `tests` fixe `umask 077` et `TMPDIR` court :

```yaml
      - run: |
          umask 077
          export TMPDIR="$(mktemp -d /tmp/b.XXXX)"
          cargo test --workspace --locked --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4
```

Raison : les sockets de test dépassent la limite de longueur d'un chemin si `TMPDIR` est long (mémoire du projet : sans cela, des dizaines de faux échecs).

Validation de T010 : sur la branche du lot, pousser et lire l'état du contrôle (`gh run list --workflow rust.yml --limit 1`, puis `gh run view <id> --log-failed`). Si le fichier est invalide, `gh` l'indique. Corriger la syntaxe. Un échec de test ou d'analyse n'invalide pas le fichier (jobs en observation).

## Fichier 2 : `.github/workflows/rust-audit.yml` (tâche T020)

Déclencheurs : `schedule` (une fois par semaine, ex. `cron: '17 5 * * 1'`), `workflow_dispatch`, et `pull_request` limité aux chemins `Cargo.lock` et `**/Cargo.toml`.

Un job `audit` : installe `cargo-audit` à version fixe, puis lance `cargo audit`.

```yaml
      - name: Install pinned cargo-audit
        run: cargo install cargo-audit --version <VERSION> --locked
      - run: cargo audit
```

Règles propres à ce fichier :

- `<VERSION>` : choisir la dernière version stable de `cargo-audit` au moment de T020 (`cargo search cargo-audit --limit 1`). Écrire la version choisie dans le journal. Pas de `latest`.
- `cargo install` compile l'outil (plusieurs minutes). Acceptable pour un contrôle hebdomadaire.
- Premier passage : `continue-on-error: true`. Les alertes trouvées vont dans le journal. Les corriger est hors périmètre (R6 : pas de nouvelle dépendance, pas de changement de version sans décision). Écrire la liste des alertes dans `validation/lot01-audit.txt`.
- Pas de `cargo-deny` dans ce programme : un seul outil suffit (article XIX).

## Ce que l'implémenteur ne fait pas

- Ne pas ajouter d'action de cache, d'installation de Rust ou de publication.
- Ne pas modifier `repository-hygiene.yml`.
- Ne pas mettre de secret, de jeton ni d'adresse dans ces fichiers.
- Ne pas déclarer « le contrôle distant passe » sans avoir lu un passage réel (`gh run view`). Sinon écrire « non exécuté ».
