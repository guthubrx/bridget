# Tâches096

## Préparation
- [x] T001 Préparer source cumulative isolée et spec/plan/contrat dans specs/096-federate-cli/.
## US1 — Destination connue
- [x] T002 [US1] Poser oracles rouges URL/port/ambiguïté et liaison connue dans crates/bridget-daemon/tests/federate_096_test.rs et scripts/tests/federation_096_test.sh.
- [x] T003 [US1] Raccorder cli.rs/lib.rs/federate.rs au script095 embarqué ; retrouver la liaison connue sans mutation dans scripts/federate-ssh.sh.
## US2 — Nouvelle destination
- [x] T004 [US2] Tester et implémenter paramètres manquants, double-TTY et binaire autonome dans les mêmes coutures ; pas de logique native dupliquée.
## US3 — Statut/retrait
- [x] T005 [US3] Couvrir et raccorder statut global, retrait explicite et erreurs du gestionnaire dans les tests096.
## Consolidation
- [x] T006 Aligner README.md, README.en.md, docs/federation-services.md, skills/bridget et inventaireCLI094 ; préserver tous les acquis095.
- [x] T007 Revue indépendante et tests ciblés/095/089, fmt/clippy, build release ; publier Mac/Linux et vérifier commande Cartae sans changer PID/reçus. Consigner preuves dans implementation.md.

Dépendances : T001→T002→T003→T004→T005→T006→T007. Deux auteurs sur des fichiers disjoints (Rust et Bash/docs), root artefacts/recette, reviewer lecture seule après gel coordonné.
