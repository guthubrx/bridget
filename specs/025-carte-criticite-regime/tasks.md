# Tâches 025 — Carte de criticité auto-élue et régime de revue

## Phase 1 — Contrat et preuves métier

- [x] T2501 Geler la propriété, les décisions et les dépendances dans `specs/025-carte-criticite-regime/spec.md`.
- [x] T2502 [P] Publier le contrat fermé dans `specs/025-carte-criticite-regime/contracts/revue-lot-v1.md`.
- [x] T2503 [P] Écrire les scénarios Gherkin dans `tests/features/025-carte-criticite-regime.feature`.
- [x] T2504 [P] Inscrire la décision structurante dans `docs/decisions/012-carte-criticite-auto-elue.md`.

## Phase 2 — Noyau pur F38

- [x] T2505 [US2] Écrire les tests du résolveur : exact, unique, homonymes, suffixe de ligne et contrat strict dans `plugins/maicie/tests/contract/review_criticality.rs`.
- [x] T2506 [US3] Écrire les tests des quatre germes et trois voies, avec contrôles positifs non vides, dans `plugins/maicie/tests/contract/review_criticality.rs`.
- [x] T2507 [US2] Implémenter l’index, les preuves et le calcul pur dans `plugins/maicie/src/review.rs`.
- [x] T2508 [US3] Poser et vérifier les mutants du résolveur, des quatre germes et du seuil de deux Majors.

## Phase 3 — Mesure Git exacte

- [ ] T2509 [US1] Écrire les tests avec vrais dépôts : paire exacte, worktree divergent, référence déplacée, base non ancêtre, renommage et limites dans `plugins/maicie/tests/review_git_integration.rs`.
- [ ] T2510 [US1] Implémenter l’adaptateur borné dans `plugins/maicie/src/review_git.rs` sans checkout ni aide de diff externe.
- [ ] T2511 [US1] Vérifier que les mêmes SHA rendent les mêmes preuves malgré worktree et environnement divergents.

## Phase 4 — Gate d’intégration des migrations

- [ ] T2512 Attendre puis vérifier l’absorption de la session 021/v17, de v18 et de la session 026/v19 ; rebaser avant toute écriture store.
- [ ] T2513 Écrire l’oracle v19 réel → v20 et le faux v19 sans DDL inchangé dans `plugins/maicie/tests/schema_migration_guard.rs`.
- [ ] T2514 Implémenter la migration v20 et les tables de revue dans `plugins/maicie/src/store.rs`.

## Phase 5 — Soumission, décision et comptage

- [ ] T2515 [US1] Étendre le vocabulaire fédéré v19 avec `review_lot_submit` et `review_regime_select` dans le contrat producteur↔consommateur.
- [ ] T2516 [US1] Implémenter la soumission transactionnelle carte + proposition + reçu dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/store.rs`.
- [ ] T2517 [US4] Implémenter la sélection du référent, le régime propre fixe et les écarts sans champ libre dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/store.rs`.
- [ ] T2518 [US4] Prouver que chaque refus métier laisse une ligne durable et qu’aucune décision par défaut n’avance la soumission.
- [ ] T2519 [US5] Ajouter les projections `review list|metrics` dans `plugins/maicie/src/main.rs` et leur corpus exact.

## Phase 6 — Fermeture factuelle et confidentialité

- [ ] T2520 [US4] Consommer le verdict SHA de la session 021 sans vocabulaire parallèle ; laisser la complétude de ronde multi-relecteurs explicitement indisponible.
- [ ] T2521 [US5] Écrire le test sentinelle qui cherche un secret synthétique dans la base et toutes les sorties.
- [ ] T2522 [US5] Vérifier les compteurs exacts de soumissions, décisions, écarts, refus et ambiguïtés.

## Phase 7 — Gates et livraison

- [ ] T2523 Exécuter `cargo fmt --all --check`, `cargo test --workspace --no-run`, les tests ciblés puis `cargo test --workspace --no-fail-fast` avec target privé.
- [ ] T2524 Exécuter Clippy sur les cibles touchées, `git diff --check` et les contrats de migration/guichet.
- [ ] T2525 Mener la revue hostile : TOCTOU Git, path traversal, fuite de contenu, idempotence, crashs, faux zéro et DDL manquant.
- [ ] T2526 Compléter `specs/025-carte-criticite-regime/implementation.md`, figer la tête, pousser et transmettre le lot.

## Dépendances

```text
T2501–T2504
    ├── T2505–T2508 ── T2509–T2511
    └── attente externe T2512
                         ↓
                    T2513–T2514
                         ↓
                    T2515–T2522
                         ↓
                    T2523–T2526
```

Les phases 2 et 3 laissent un incrément compilable sans migration. La phase 4
est un gate dur : aucun numéro ni DDL v20 n’est écrit avant son succès.
