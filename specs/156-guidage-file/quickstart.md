# Vérification rapide - Session156

Après livraison155 et création par root, toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file`.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  CARGO_BUILD_JOBS=2 PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  nice -n 10 cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=2
```

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/quickstart.md#extrait-1).

## Scénarios à observer

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/quickstart.md#extrait-2).

## Recette manuelle après activation Bridget autorisée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/quickstart.md#extrait-3).

```sh
bridget send --to <enfant> --intent steer "corrige X"
bridget queue --target <enfant>
```

Étendre seulement les scénarios existants : échéance steer agent sans interrupt,
timeout humain validé, UI attestée/systèmes internes sur les deux trames, refus
tardifs non comptés, FIFO. QueueList exclut Send historique et files internes.
Aucun test ni commande de recette exécuté dans la préparationR1.
