# Vérification rapide - Session159

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
Un changement de trame, de reçu ou de file exige aussi tous les tests `t3code`,
`mcp::`, `guidance156` et transport, pas seulement les nouveaux.

**Migration** : ne jamais l'essayer sur la base réelle. Copier la base dans le
répertoire privé de test (`$R/ns`) avec `.backup`, puis ouvrir la copie.

## Scénarios à observer

Lot A (159a) :
1. Migration v7 (sans perte, ordre, liens d'exécution, rejeu, rollback, précontrôle) : filtre `held159_migration`.
2. Rang atomique, réaffectation, orphelinage : filtre `held159_rank`.
3. Décision de rétention : filtre `held159_hold`.
4. Signal de tour véridique et réattesté : filtre `held159_turnstate`. Libération, réservation de tour, pause : filtre `held159_release`. Rattachement par agent : filtre `held159_adopt`.
5. Balayage (échéances, demandes closes) : filtre `held159_sweep`.
6. Lecture `held` / `released` : filtre `held159_list`.
7. Pannes, ordre, latence : `cargo test -p bridget-daemon --features test-support --test held159_e2e`.
8. Répétition de la restauration : suivre `validation/restauration-v7.md` (T011), en isolation.

Lot B (159b) :
9. Reçu et droits : filtres `edit159_receipt_store`, `edit159_rights`.
10. Retrait : filtre `edit159_cancel`. Déplacement : filtre `edit159_reorder`.
11. Promotion et transports : filtres `edit159_promote`, `edit159_promote_transport`.
12. Outil et CLI : filtres `edit159_mcp`, `edit159_cli`.
13. Bout en bout : `cargo test -p bridget-daemon --features test-support --test edit159_e2e`.

Non-régression : filtres `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`.

## Recette manuelle (après relance décidée par l'utilisateur)

Ne pas relancer les services dans cette session. Après relance :

```sh
bridget queue --target <enfant>
bridget queue cancel --target <enfant> --message <id> --request <clé>
```

Pour l'ouverture du réglage (après le lot A) : mettre `held_queue = true` pour le
type `codex` dans le registre des agents, relancer le daemon, puis comparer la
latence fin de tour vers début du message suivant à la référence de T001.

```sh
# sauvegarde obligatoire avant la première ouverture par le binaire neuf
sqlite3 ~/.cache/bridget/bridget.db ".backup ~/.cache/bridget/bridget.db.avant-v7"
```
