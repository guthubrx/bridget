# Vérification rapide - Session152

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions`.

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

1. Reproduction rouge de l'échec sous pause : filtre `native152_rouge` (doit
   échouer sur le code 150, passer après US1).
2. Pause, chemins A, B et ordre inverse : filtre `native152_pause`.
3. Gel des horloges : filtre `native152_horloges`.
4. Lancement retenu puis lancé : filtre `native152_lancement`.
5. Reprise après redémarrage et borne : filtre `native152_panne`.
6. Annulation vérifiée d'abord : filtre `native152_annulation`.
7. Reçus de refus : filtre `native152_refus`.
8. Panne pendant pause, filiation et deux redémarrages : filtres `native152_panne`, `native152_filiation`.
9. Non-régression : filtres `native148`, `native149`, `native150`,
   `delegation149`, `spec_087`.

## Contrôle réel après activation coordonnée

En phase 8, le principal observe les processus, le namespace et la quiescence
des travaux. Il coordonne l’activation de production prévue par H6 sans
interrompre les autres fils T3 et sans redémarrer T3. Aucune relance n’est
effectuée à l’étape actuelle de développement.

Le contrôle réel s’exécute après cette activation : déléguer une mission longue,
mettre le référent en pause (`bridget` contrôle d’état), attendre plus que le
`reply_timeout`, puis lire `bridget_task_status` (`working` avec
`suspended: true`). Lever la pause, lire `working` avec `suspended: false`,
puis vérifier le résultat.

Le reçu de livraison distingue les sources Git, les fichiers installés et le
code réellement actif dans les processus. L’état actif doit être prouvé par
les faits observés lors de la livraison.
