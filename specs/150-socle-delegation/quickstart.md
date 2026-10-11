# Vérification rapide - Session150

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=4
```

Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.

## Scénarios à observer

1. Délégation après plus de 4096 tâches : filtre `native150_admission`.
2. Purge, arbres et pierre tombale : filtre `native150_purge`.
3. Ouverture avec 10 000 lignes : filtre `native150_ouverture`.
4. Pause à l'admission : filtre `native150_pause`.
5. Délai par tâche : filtre `native150_delai`.
6. Non-régression 148-149 : filtres `native148`, `native149`, `delegation149`.
