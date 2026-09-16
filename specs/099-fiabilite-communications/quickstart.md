# Validation isolée 099

Aucune commande de ce guide ne déploie le binaire. Les tests doivent créer leur
BRIDGET_HOME et BRIDGET_SOCKET privés, ainsi qu'un HOME enfant sans compte fournisseur.

Depuis /Users/moi/Nextcloud/10.Scripts/64.bridget :

```sh
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-core
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-transport --lib protocol::
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-daemon --lib spec099
/Users/moi/.cargo/bin/cargo test --offline --locked -p bridget-daemon --lib spec098
/Users/moi/.cargo/bin/cargo test --offline --locked --workspace --no-run
/Users/moi/.cargo/bin/cargo check --offline --locked --workspace --tests --features test-support
/Users/moi/.cargo/bin/cargo fmt --all -- --check
/Users/moi/.cargo/bin/cargo clippy --offline --locked --workspace --all-targets -- -D warnings
/Users/moi/.cargo/bin/cargo clippy --offline --locked --workspace --tests --features test-support -- -D warnings
```

Pour les unités daemon, fixer BRIDGET_HOME et BRIDGET_SOCKET sur un dossier privé
créé par mktemp -d /tmp/bg099-validation.XXXXXX, et TMPDIR=/tmp (borne des sockets
Unix). Ne jamais reprendre le home historique. Les commandes réellement utilisées
avec leur environnement figurent dans implementation.md.

La commande globale initialement prévue, cargo test --workspace, n'est PAS
exécutable sous la protection processus demandée : plusieurs harnais historiques
emploient SIGKILL et arrêtent des groupes (tests/support/idempotent.rs,
daemon presence_tests, transport ACP/Claude, etc.). Compiler tous les tests avec
--no-run vérifie les contrats Rust sans exécuter ces arrêts. Exécuter séparément
les modules sûrs et les recettes ci-dessous ; ne pas annoncer la suite complète
verte sur la seule base de --no-run.

Recettes de bout en bout, après cargo build --offline --locked -p bridget-daemon :

```sh
/usr/bin/python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_daemon.py --expect-fixed
/usr/bin/python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-09-16/session-2026-09-16-bridget-global-01/reproduce_t3.py --expect-fixed
```

Ces scripts créent leurs états privés, utilisent le fournisseur HTTP synthétique,
identifient leurs enfants avant SIGTERM individuel et vérifient leur arrêt.
Résultats détaillés et limites : implementation.md (produit au cours de l'exécution).
Ne jamais substituer le daemon historique aux fixtures si un test échoue.
