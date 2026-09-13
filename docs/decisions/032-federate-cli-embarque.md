# ADR032 — Entrée federate et lanceur embarqué

2026-09-07 · Accepté · Session096

Le chemin du script095 ne doit pas être une connaissance demandée à l'humain. La sous-commande federate fournit l'ergonomie URL/options, mais conserve le script095 comme unique propriétaire de launchd/systemd, validations, reçus et rollback.

Le script est embarqué lors de la compilation et matérialisé dans une zone privée pour exécution. Ni recherche d'un script dans un ancien dépôt, ni réécriture Rust des services. L'installation native copie toujours son propre runner autonome. Le coût assumé est une fine couture d'arguments et de durée de vie du script ; elle est testée depuis une copie isolée du binaire.
