# Vérification rapide - Session157

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas`.
Aucun test n'appelle un vrai fournisseur. Les faits de quota sont synthétiques.

## Environnement de test (obligatoire)

```sh
umask 077
spec157_root=$(mktemp -d /private/tmp/b.XXXXXX)
mkdir -m700 "$spec157_root/ns" "$spec157_root/home" "$spec157_root/tmp"
env -i HOME="$spec157_root/home" TMPDIR="$spec157_root/tmp" \
  BRIDGET_HOME="$spec157_root/ns" BRIDGET_SOCKET="$spec157_root/ns/bridget.sock" \
  CARGO_HOME=/Users/moi/.cargo RUSTUP_HOME=/Users/moi/.rustup \
  PATH=/opt/homebrew/anaconda3/bin:/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  CARGO_BUILD_JOBS=2 BRIDGET_BUILD_CACHE_GIB=40 \
  CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target \
  /usr/bin/nice -n10 python3 scripts/build.py cargo test --locked --offline \
  -p bridget-daemon --features test-support <filtre> -- --test-threads=2
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

Une seule commande Cargo à la fois. FULL unique à T025 ; reprises ciblées après
correction. Ne pas lancer une deuxième suite complète pour les mêmes sources.
