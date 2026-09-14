# Démarrage rapide 097

## Session interactive sans tmux

```sh
bridget claude                 # nouvelle session dans le terminal courant
bridget claude --resume        # reprise, même comportement
bridget who                    # attendu : claude | claude_pty | cli
```

Depuis un autre agent : `bridget send --to <UUID> --reply -- 'Confirme réception.'`
Le message apparaît dans la conversation Claude ; la réponse liée clôt la demande.

## Recette Claude géré (compte local, un tour)

```sh
umask 077; R=$(mktemp -d /private/tmp/b097.XXXXXX); mkdir -m 700 $R/state $R/tmp
BRIDGET_CLAUDE_NATIVE_GATE=1 BRIDGET_TEST_CLAUDE_BIN=/Users/moi/.local/bin/claude \
BRIDGET_HOME=$R/state BRIDGET_SOCKET=$R/state/bridget.sock TMPDIR=$R/tmp \
cargo test --offline --locked -p bridget-daemon --features test-support \
  --test core_089_native_test claude -- --include-ignored --test-threads=1 --nocapture
```

HOME et USER restent ceux de la session : c'est ce qui porte la connexion au compte.

## Harnais sans compte

```sh
cargo test --offline --locked -p bridget-daemon --features test-support \
  --test claude_interactive_097_test -- --test-threads=1
```

## Gates

```sh
cargo fmt --all --check
cargo clippy --offline --locked --workspace --all-targets --features bridget-daemon/test-support -- -D warnings
# suite complète : voir specs/089-communication-core/implementation.md (environnement privé, umask 077)
```
