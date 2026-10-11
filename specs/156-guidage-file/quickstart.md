# Vérification rapide - Session156

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

Tests du crate transport : même environnement, `cargo test -p bridget-transport <filtre>`.
Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.
Une modification de texte injecté ou de trame exige aussi tous les tests
`t3code`, `mcp::` et transport, pas seulement les nouveaux.

## Scénarios à observer

1. Table des droits : filtre `guidance156_matrix`.
2. Intention à l'envoi (MCP, CLI, daemon, canon) : filtres `guidance156_mcp`, `guidance156_cli`, `guidance156_send`.
3. Garde sur les chemins, origine, autorité humaine, contrôle : filtres `guidance156_guard`, `guidance156_origin`, `guidance156_human`, `guidance156_control`.
4. Pont T3 (`steer`, `restart`, hors lot, tête bloquée) : filtre `guidance156_t3`.
5. Anti-boucle : filtre `guidance156_rate`.
6. Pause : filtre `guidance156_pause`.
7. Enfant en mission : filtre `guidance156_mission`.
8. Guidage implicite T3, annulation, tests combinés : filtres `guidance156_midturn`, `guidance156_cancel`, `guidance156_combo`.
9. Lecture de file : filtre `guidance156_queue`.
10. Bout en bout : `cargo test -p bridget-daemon --features test-support --test guidance156_e2e`.
11. Non-régression : filtres `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::`.

## Recette manuelle (après relance décidée par l'utilisateur)

Ne pas relancer les services dans cette session. Après relance :

```sh
bridget send --to <enfant> --intent steer "corrige X"
bridget queue --target <enfant>
```
