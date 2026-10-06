# SPEC138 — fils, preuves de développement

## Avant production

Recherche de réutilisation: `lookup_operation` et `thread_operations.result_json`
existent dans le store. Le service possède `thread_show`, qui vérifie la qualité
de membre avant de rendre tous les lecteurs. Ces chemins sont étendus. Aucune
table, colonne, index ou journal parallèle n'est créé.

Tests ajoutés avant les changements de production dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs.

Tentative Cargo RED: compilation interrompue par la publication simultanée des
types de protocole du noyau; cette sortie ne constitue pas une preuve RED.

RED réel ensuite: ancien binaire compilé avant les changements, démarré sous
BRIDGET_HOME privé `/tmp/b138-red-*`, socket dans ce home. Deux propriétaires
synthétiques sont inscrits sans fait projet; une création legacy aboutit.
L'assertion `project_warnings[0].code == project_unknown` échoue: le résultat
ne contient aucun avertissement. Code de sortie 1. Arrêt SIGTERM et attente de
sortie réelle. Aucun namespace de production utilisé.

## Changement

Le service calcule le canon puis consulte le reçu exact avant la garde de
portée. La transaction du store refait le lookup pour couvrir la concurrence.
Un suffixe absent conserve le canon historique. Un motif présent change le
canon. Les avertissements prévalidés sont enregistrés dans le résultat JSON
existant, au même commit que l'opération. Pour post, l'audience est la totalité
des lecteurs, pas notify. Un refus de portée arrive avant ACK et dépôt.

## Vérification réelle

Tests réels: création legacy inconnue et reçu durable; fil A/B/U dont U inconnu
ne neutralise pas A/B; dépôt history silencieux mais mixte; motif différent ou
absent sous une clé déjà autorisée; création et post déjà acceptés puis faits
changés après redémarrage; ancienne alerte encore en remise; motif vide, trop
long UTF-8, contrôle et NUL; historique exact et zéro alerte supplémentaire.

Commandes depuis
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet:

```bash
umask 077
export BRIDGET_HOME=$(mktemp -d /tmp/b138-th.XXXXXX)
export BRIDGET_SOCKET=$BRIDGET_HOME/bridget.sock
export TMPDIR=$BRIDGET_HOME
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/Volumes/8TB2/01-workflow/bridget-138-cargo.HjVvbG
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test spec102_threads_test spec138_ -- --nocapture
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --test spec102_threads_test -- --nocapture
/Users/moi/.cargo/bin/cargo test -p bridget-daemon --lib threads::tests::spec138_thread_reason_absence_is_legacy_but_null_is_invalid -- --exact --nocapture
```

Résultats capturés le 2026-10-06, codes de sortie 0:

```text
running 4 tests
test spec138_invalid_reason_never_creates_a_thread ... ok
test spec138_unknown_legacy_thread_returns_durable_warning ... ok
test spec138_mixed_thread_checks_all_readers_even_with_unknown ... ok
test spec138_accepted_thread_replay_precedes_changed_project_guard_after_restart ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 41 filtered out; finished in 1.90s

running 45 tests
test result: ok. 44 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 9.40s

running 1 test
test threads::tests::spec138_thread_reason_absence_is_legacy_but_null_is_invalid ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1035 filtered out; finished in 0.00s
```

Le test ignoré est le helper de daemon de performance, destiné au sous-processus
du harnais, pas un cas fournisseur sauté. La suite comprend aussi les anciens
tests 102 et 136, dont 10 000 entrées, les accès, les ACK, la pagination, les
échecs/reprises de notification et le silence de history.

Le test A/B/U confirme en plus qu'un refus de portée avec `ack_receipt` ne
confirme pas la page et ne dépose rien. Le test legacy vérifie l'empreinte
historique de create dans SQLite contre un SHA-256 fixe. Le test de reprise
confirme le résultat exact après changement de faits et restart, sans nouveau
dépôt; une ancienne alerte acceptée avant restart reste remise une seule fois.

`rustfmt` exécuté seulement sur les trois fichiers réservés. `git diff --check`
sur ces fichiers: succès. Aucune table, colonne, dépendance, installation,
mutation de mission, commande fournisseur, notification réelle ou livraison.

Statut du volet fils: GREEN. La validation complète et la livraison restent
au principal. Aucun commit effectué.
