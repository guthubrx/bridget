# Vérification rapide 134

Depuis la racine du worktree :

```sh
cargo test -p bridget-core sender_label
cargo test -p bridget-daemon spec134
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --locked --release -p bridget-daemon
```

Recette privée : démarrer le daemon avec un `BRIDGET_HOME` et une socket
temporaires. Enregistrer une identité déjà présente mais sans profil. Publier le
nom `Regional`. Le message livré doit commencer par un en-tête équivalent à :

```text
💬 Message Bridget de Regional (UUID-complet) (id …)
```

Le statut doit encore résoudre et répondre par l’UUID.
