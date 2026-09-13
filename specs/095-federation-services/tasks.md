# Tâches 095

## Préparation
- [x] T001 Cadrer réemploi et contrat dans specs/095-federation-services/spec.md, plan.md et docs/decisions/031-services-federation-autonomes.md.
## US1 — Services autonomes
- [x] T002 [US1] Poser et exécuter oracles rouges install/status/remove et collision dans scripts/tests/federation_095_test.sh.
- [x] T003 [US1] Implémenter templates launchd/systemd utilisateur et copie autonome dans scripts/federate-ssh.sh, sans anciennes dépendances.
## US3 — Reconnexion
- [x] T004 [US3] Tester et réaliser récupération sûre de socket périmée et refus de socket vivante dans scripts/federate-ssh.sh et tests 095.
## US2 — Mise en service
- [x] T005 [US2] Sauvegarder l'ancien ensemble Cartae, installer le client sur Cartae et le service tunnel sur le Mac maître (-R Cartae:Mac) ; consigner chemins et reçus dans specs/095-federation-services/implementation.md.
- [x] T006 [US2] Vérifier mêmes UUID et deux échanges réels Mac/Cartae dans specs/095-federation-services/implementation.md.
## Consolidation
- [x] T007 Rejouer shell 089/095, revue sécurité, launchd natif et coupure/reconnexion ; documenter limites et rollback dans docs/federation-services.md. Systemd couvert par gestionnaire doublé sur Linux ; aucune nouvelle autorisation SSH créée pour une recette maître Linux.

Dépendances : T001→T002→T003→T004→T005→T006→T007. Préparation de migration lecture seule en parallèle des scripts. Pas d'écriture simultanée par plusieurs agents.
