# Journal d'implémentation — Session 038

## Métadonnées

- **Spec** : 038-regeneration-politique
- **Branche** : session-038-regeneration-politique
- **Base gelée** : b676afa86174df1def7a57caa73b969567364c42
- **Démarré** : 2026-08-27
- **Terminé** : En cours

## Progression

### T001 — Contrat et décision

- **Statut** : Terminé
- **Commit** : `docs(038): Formalise la regeneration de politique`
- **Fichiers modifiés** : documentation de session et ADR
- **Tests exécutés** : `git diff --check`
- **Notes** : la politique réelle en service n'a été ni lue ni modifiée.

### T002 à T007 — Inventaire et régénération

- **Statut** : Terminés
- **Commit** : `feat(038): Regenere les instances approuvees`
- **Fichiers modifiés** : contrat partagé de politique, module de régénération,
  scanner MCP, binaire dédié et exemple 038.
- **Tests exécutés** : `cargo check -p bridget-transport`,
  `cargo check -p bridget-daemon --bin bridget-greffe-policy-refresh`.
- **Notes** : `marker_source` reste optionnel pour la garde historique et
  devient obligatoire uniquement pour la régénération. Le chemin de politique
  est explicite, absolu, canonique et non lié.

### T008 — Oracles unitaires

- **Statut** : Terminé
- **Commit** : `feat(038): Regenere les instances approuvees`
- **Univers** : 11 tests transport ; 9 tests `mcp_identity`, dont 3 nouveaux ;
  2 tests du parseur de commande.
- **Résultats** : 11/0/0, 9/0/0 et 2/0/0.
- **Propriétés** : sources exactes, zéro vivant, PID recyclé, ambiguïté,
  fraîcheur, conservation d'un principal arrêté, refus d'enrôlement,
  génération croissante et deux frontières du remplacement atomique.

### T009 — Chemin réel et mutants

- **Statut** : Terminé
- **Commit** : `test(038): Prouve le renouvellement par le vrai binaire`
- **Univers** : 1 test d'intégration réel.
- **Résultat nominal** : 1 passed / 0 failed / 0 ignored.
- **Mutant génération** : 0 passed / 1 failed ; assertion métier dans
  `greffe_policy_refresh.rs`, valeur observée 7 contre valeur attendue 8.
- **Mutant écriture directe** : 0 passed / 1 failed ; assertion métier sur les
  phases observées, liste vide contre `[BeforeRename, AfterRename]`.
- **Restauration** : test exact revenu à 1/0/0 après chaque mutant.
- **Contrôle positif** : le vrai binaire scanne, prévisualise puis applique ; la
  garde réelle refuse l'ancienne instance avant effet et la nouvelle écrit le
  fichier durable.

### T010 — Gates et livraison

- **Statut** : En cours
- **Commit** : À venir
- **Notes** : le clippy strict expose des diagnostics préexistants hors delta ;
  une seconde commande ciblée, avec ces diagnostics explicitement neutralisés,
  ne trouve aucun avertissement supplémentaire dans 038.

## REX — Retour d'expérience

À compléter après les gates et la revue externe.
