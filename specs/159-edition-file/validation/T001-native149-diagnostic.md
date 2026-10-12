# Diagnostic référenceT001 — native149 O4 lot mixte

Observation UTC : 2026-10-12T13:51:09.394146+00:00. Lecture seule. Aucun signal, test, Cargo, DB, service ou Git exécuté. Aucun seuil/fixture/source modifié.

Le journal /Users/moi/.cache/bridget-tests159A-20261012/T001-daemon-reference.log:602–612 montre l'unwrap du test:4143 sur InvalidStatus("groupe périmé encore vivant pour ordi-mixte (pgid24647)"). Le lot lib termine528PASS/1FAIL/3ignored en51.74s ; cela ne décrit pas les autres exécutables du batch.

Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:4129–4154. Le test démarre un groupe natif coopératif (script_lent:3830–3832, fermeture0.4s) et un groupe ordinaire qui ignore SIGTERM (script_tetu:3835–3837). Seul le natif figure dans native_identities. COURT:3982 vaut100ms ; SONDE:3983 vaut10ms ; grâce native10s. Le résultat attendu reste deux groupes soldés, natif exit0 et ordinaire SIGKILL.

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file/crates/bridget-daemon/src/managed_process.rs:1278–1324 envoie SIGTERM aux natifs puis traite les ordinaires :50ms coopératifs, SIGKILL, puis50ms forcés. La grâce native10s ne rallonge pas ces50ms ordinaires. L'InvalidStatus cité n'arrive qu'après wait_group_gone encore false à la deuxième échéance. Donc l'escalade a été appelée, sans erreur de signal remontée. Le journal ne prouve pas quels membres restaient à cet instant.

Le test utilise /bin/sh avec enfant sleep0.05. GroupeReel:3866–3910 démarre aussi un thread enfant.wait pour récolter le leader, explicitement afin de supprimer son zombie. Le thread de collecte est asynchrone. group_exists:498–511 teste kill(-pgid,0), sans distinguer un processus actif d'un zombie ; wait_group_gone:530–541 échantillonne jusqu'à l'échéance. Une collecte du leader tardive, une fin tardive d'un descendant ou son zombie peuvent donc expliquer ce contrôle encore positif dans50ms. **Hypothèse plausible, non démontrée** : contention/ordonnancement de la recette réelle (nice10/test-threads2) ou délai de collecte. Ce constat ne suffit pas à classer le problème «fixture seulement», ni à exonérer le chemin production de réconciliation.

Observation de processus réalisée sans signal : `ps -p24647 -o pid,ppid,pgid,lstart,stat,command` n'a retourné que l'en-tête (code1 : aucunPID) ; `ps -axo pid,ppid,pgid,stat,command` filtré sur PGID24647 n'a retourné aucune ligne. Le groupe n'est plus observable à cette date. Aucun PID recyclé n'a été signalé. Une preuve rétrospective du zombie ou de la charge pendant l'échec n'est pas disponible.

Le test échoue avant natif.statut()/ordinaire.statut() : aucun statut de sortie runtime n'est enregistré dans le log. Drop:3949–3962 peut demander une fermeture coopérative lors du déroulement de pile si le leader est encore vivant ; cette possibilité ne prouve pas comment le groupe a disparu. Pas de correction proposée, aucune instrumentation ajoutée. Le diagnostic est indépendant du métier159, encore absent.

## Rejeu exact parROOT seulement

Répertoire absolu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file

```sh
CARGO_BUILD_JOBS=2 /usr/bin/nice -n 10 python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --lib --features test-support daemon::native_delegation::native_delegation_permissions149_tests::native149_o4_lot_mixte_garde_la_grace_native_et_l_escalade_ordinaire -- --exact --nocapture --test-threads=2
```

Le rejeu conserve les seuils, la charge prescrite et toutes les assertions. Son résultat doit être archivé ; un PASS isolé serait une preuve de non-reproduction, pas une explication définitive de l'échec initial.
