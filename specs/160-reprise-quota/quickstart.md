# Vérification rapide - Session160

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota`.
Aucun test n'appelle un vrai fournisseur. Les faits de quota sont synthétiques.

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

1. Tests rouges des six exigences : filtre `native160_rouge`.
2. Qualification, décision en attente, silence : filtre `native160_attente`.
3. Reconstruction ciblée et relance unique (huit frontières a–h de crash) : filtre
   `native160_relance`.
4. Bornes et compteur durable : filtre `native160_bornes`.
5. Gel cumulé, annulation, pause : filtre `native160_gel`.
6. Opt-in, octets canoniques, aucune substitution : filtre `native160_optin`.
7. Non-régression : filtres `native148`, `native149`, `native150`, `native152`,
   `native157`, `delegation149`, `spec_087`.

Les fixtures rejouent les événements du wrapper (fait de quota, transition
d'exécution `failed`, `DeliveryRejected`) par le gestionnaire complet du daemon,
dans les deux ordres. Les sorties sont consignées dans
`specs/160-reprise-quota/validation/`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/160-reprise-quota/quickstart.md#extrait-1).
