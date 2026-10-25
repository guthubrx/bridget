# Vérification rapide - Session151

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache`.

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/quickstart.md#extrait-1).

## Compatibilité avec un ancien daemon (T018)

Compiler le daemon de la base 150 dans un dossier temporaire hors du dépôt
(`git worktree add /tmp/b151-ancien <commit base 150>`, jamais de `checkout` dans le
worktree), lancer le client 151 contre lui avec la même recette, constater
`wait_unsupported` en moins de 5 s, puis supprimer le dossier temporaire.

## Recette réelle (après installation, sans modèle de production)

Hors de cette session : l'installation des fichiers ne recharge pas les processus
MCP déjà ouverts. Avec un daemon 151 chargé, un parent synthétique appelle
`bridget_delegate`, puis `bridget_task_wait` avec `wait_timeout: 5`. Résultat
attendu : `timed_out: true` si l'enfant synthétique n'a pas répondu, puis un seul
message `native-result-<task>` à l'arrivée de la réponse, et `mailbox.state` qui
passe à `delivered` après l'accusé.
