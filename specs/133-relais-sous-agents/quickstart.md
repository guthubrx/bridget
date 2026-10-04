# Validation isolée 133

Aucune commande ne déploie le binaire. Utiliser le worktree et un état privé.

Racine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/133-relais-sous-agents`

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/133-relais-sous-agents
export BRIDGET_HOME="$(mktemp -d /tmp/bg133.XXXXXX)"
export BRIDGET_SOCKET="$BRIDGET_HOME/b.sock"
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-core spec133 -- --nocapture
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-daemon spec133 -- --nocapture
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-transport spec133 -- --nocapture
/Users/moi/.cargo/bin/cargo test --offline --locked --workspace
/Users/moi/.cargo/bin/cargo fmt --all -- --check
/Users/moi/.cargo/bin/cargo clippy --offline --locked --workspace --all-targets -- -D warnings
/Users/moi/.cargo/bin/cargo build --offline --locked --workspace --release
```

Les tests ne lisent pas l'état T3 réel. Ils utilisent des inventaires, arbres de
processus, fichiers privés et connexions daemon synthétiques. Supprimer le dossier
temporaire seulement après la fin des processus du test.
