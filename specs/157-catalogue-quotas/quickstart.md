# Vérification rapide - Session157

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
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

1. Cohérence catalogue et admission (incident du 2026-10-11) : filtre `native157_coherence`.
2. Préflight (binaire absent, garde de facturation) : filtre `native157_preflight`.
3. Mémoire et règle d'épuisement des quotas : filtre `native157_limits`.
4. Quota au catalogue et à l'admission : filtre `native157_quota`.
5. Remplacement : filtre `native157_fallback`.
6. Non-régression : filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`.
7. Mesure du catalogue : test ignoré par défaut `native157_catalogue_latence`
   (`-- --ignored --nocapture`), 10 fournisseurs de test, valeurs consignées dans
   `specs/157-catalogue-quotas/validation/`.
