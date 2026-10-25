# Vérification rapide - Session153

Toutes les commandes partent du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte`.

## Environnement de test (obligatoire)

```sh
umask 077
R=$(mktemp -d /tmp/b.XXXX); mkdir -m 700 $R/ns
env -i HOME=$HOME TMPDIR=$R BRIDGET_HOME=$R/ns \
  PATH=/opt/homebrew/anaconda3/bin:$HOME/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- \
  /opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline \
    -p bridget-daemon --features test-support <filtre> -- --test-threads=4 --nocapture
```

Suite complète avant clôture : même environnement,
`/opt/homebrew/anaconda3/bin/python3 scripts/build.py cargo test --locked --offline --workspace --features bridget-daemon/test-support --no-fail-fast`.

## Scénarios à observer

1. Brief : filtre `native153_brief`.
2. Garde, expiration, fin explicite : filtre `native153_garde`.
3. Rondes, rejeu, identifiants de résultat : filtre `native153_ronde`.
4. Limites, pause, purge : filtre `native153_limites`.
5. Réponses tardives, accusés par ronde, droits actuels, échéance : filtres
   `native153_ronde_reponse_tardive`, `native153_limites`, `native153_ronde`.
6. Trois rondes avec un faux enfant : test d'intégration `native_delegation_e2e`
   (filtre `native153_e2e`).
7. Non-régression : filtres `native148`, `native149`, `native150`,
   `delegation149`, `spec103`, `spec_087`.

## Recette réelle (mémoire de l'enfant)

Environnement isolé uniquement, sans relance des services de production :

1. Déléguer avec `keep_alive_secs=600` une tâche : « retiens le mot BLEU-7 et
   réponds OK ».
2. Observer au moins60s que l’enfant reste connecté sans arrêt, puis appeler `bridget_task_continue` avec une nouvelle
   `request_id` et la consigne : « quel mot t'ai-je demandé de retenir ? ».
3. Attendu : l'enfant répond `BLEU-7`, `child_instance` identique, ronde 2.
4. Appeler `bridget_task_cancel` : l'enfant est arrêté, `status` reste
   `result_available`.

Si la recette réelle n'est pas lancée, la consigner « non vérifié » dans
`validation/tests-153.md`.


[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/153-rondes-contexte/quickstart.md#extrait-1).
