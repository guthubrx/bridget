# Vérification rapide - Session154

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree`.
Aucune commande ne lance de service ni de modèle.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
  GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t \
  perl -e 'alarm 1800; exec @ARGV' -- \
  cargo test -p bridget-daemon --features test-support <filtre> -- --test-threads=4
```

Les tests créent leurs dépôts Git sous `$R` (jamais dans le dépôt Bridget). Les
variables `GIT_*` rendent les commits de test déterministes et ignorent la
configuration de l'utilisateur.

Suite complète avant clôture : même environnement,
`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast`.

## Mesure préalable (T002, règle le délai de Git)

```sh
umask 077; R=$(mktemp -d /tmp/b.XXXX)
git clone --local --no-hardlinks /Users/moi/Nextcloud/10.Scripts/64.bridget $R/depot
for repetition in 1 2 3 4 5; do
  time git -C "$R/depot" worktree add -b "mesure-$repetition" "$R/depot/.worktrees/m$repetition" HEAD
done
# Conserver ce clone privé jusqu’au nettoyage contrôlé du principal.
```

Consigner les cinq temps, leur médiane et maximum dans `validation/mesure-worktree.md`. T002 est exécutée par le principal. T025 exécute une seule suite workspace finale.

## Scénarios à observer

1. Exécuteur Git (liste blanche, délai, environnement) : filtre `native154_git`.
2. Contrat et enveloppe : filtre `native154_contrat`.
3. Création sûre et refus : filtre `native154_creation`.
4. Isolation effective et droits : filtre `native154_isolation`.
5. Rapport (0, 2, 60 commits) : filtre `native154_rapport`.
6. Conservation sans suppression, annulation, purge 150 : filtre `native154_conservation`.
7. Avertissement de partage, collision et wrapper : filtres
   `native154_partage`, `native154_wrapper`, `spec100`.
8. Non-régression T001 : filtres `native148`, `native149`, `native150`,
   `delegation149`, `spec100`, `spec_087`. Les contrats153 restent préservés.

## Vérification manuelle de fin (sans modèle)

Après les tests, vérifier qu'aucun reste n'a été laissé dans le dépôt Bridget :
`git -C /Users/moi/Nextcloud/10.Scripts/64.bridget worktree list` ne doit
montrer aucun chemin `.worktrees/<8 hex>` créé par les tests.
