# Vérification rapide - Session158a

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

1. Vue globale sans T3, client agent refusé : filtre `control158_overview`.
2. Arrêt global (4 combinaisons avec la pause, reçus A-B-rejeu, annulation en cascade) : filtre `control158_halt`.
3. Portées MCP (table, plafond, fils en lecture et ACK, surface) : filtre `mcp158_`.
4. Non-régression : filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`.

## Recette manuelle (jamais sur le daemon de production)

À faire seulement dans un `BRIDGET_HOME` privé avec un daemon de test.

```sh
bridget control delegations --state all --json
bridget control halt-delegations --reason "essai"      # terminal interactif
bridget control status                                   # montre l'arrêt
bridget control resume-delegations
```

Aucune relance de service n'est faite par cette session.
