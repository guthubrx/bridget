# Journal d’implémentation 025

## Métadonnées

- **Branche** : `session-025-carte-criticite-regime`
- **Base** : `b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`
- **Démarré** : 2026-08-25
- **Terminé** : en cours
- **Migration** : v20 réservée

## Progression

### T2501–T2504 — Contrat et preuves métier

- **Statut** : complété
- **Fichiers** : spec, plan, modèle, contrat, tâches, Gherkin et ADR 012
- **Tests** : `git diff --check` et contrôle des termes interdits
- **Note** : le lot est coupé avant l’élection des relecteurs sur la dépendance
  réelle au capteur de la session 023. La migration v20 attend v17–v19.

### T2505–T2508 — Noyau pur F38

- **Statut** : complété
- **Fichiers** : `plugins/maicie/src/review.rs`, surface publique et sept
  tests contractuels
- **Mesure** : 7 passés, 0 échoué, 0 ignoré ; compilation préalable avec
  `cargo test -p maicie --test contract --no-run`
- **Mutants tués dans les assertions** : retrait séparé des quatre germes,
  élection arbitraire du premier homonyme et seuil Major abaissé de deux à un
- **Garde** : les ambiguïtés ne portent que l’empreinte du jeton et les chemins
  candidats ; aucun contenu de diff ou de constat ne franchit la sortie pure

## REX

À compléter après les gates et la revue hostile.
