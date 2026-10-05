# Tâches 134 — Noms humains dans les messages Bridget

## Phase 1 — Préparation

- [x] T001 Vérifier le worktree, la base `main`, le contrat et les tests ciblés existants ; consigner le résultat dans `specs/134-noms-humains-messages/implementation.md`

## Phase 2 — Libellé commun

Objectif : rendre le nom humain et l’UUID sans changer l’identité routable.

Test indépendant : les cas nommé, absent, vide, blanc, égal à l’UUID et délégué
produisent exactement le libellé du contrat.

- [x] T002 [US1] [US3] Ajouter les tests rouges `spec134` puis implémenter le libellé `nom (UUID)` avec repli UUID et provenance conservée dans `crates/bridget-core/src/message.rs`

## Phase 3 — Lots T3

Objectif : appliquer le libellé commun à chaque message groupé.

Test indépendant : un lot de deux expéditeurs nommés associe chaque nom au bon
UUID complet.

- [x] T003 [US2] Ajouter le test rouge `spec134` puis remplacer l’accès direct à l’expéditeur par `sender_label()` dans `crates/bridget-daemon/src/t3code.rs`

## Phase 4 — Réparation du profil

Objectif : garantir l’invariant identité, profil et état d’application après
chaque enregistrement.

Test indépendant : conserver une identité, retirer son profil, rappeler deux fois
`ensure_agent_ids()` puis vérifier une seule identité, un seul profil et un seul
état d’application.

- [ ] T004 [P] [US4] Ajouter le test rouge `spec134` puis rendre `ensure_agent_ids()` idempotent sur les trois lignes dans `crates/bridget-daemon/src/agent_profile.rs`

## Phase 5 — Validation et livraison

- [ ] T005 Exécuter les tests ciblés, `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo build --locked --release -p bridget-daemon`, puis consigner les preuves dans `specs/134-noms-humains-messages/implementation.md`
- [ ] T006 Faire la self-review Article XIX/XX, la contre-revue adverse post-implémentation, la convergence des artefacts et l’audit final ; résultat observable dans `specs/134-noms-humains-messages/implementation.md` et `specs/134-noms-humains-messages/validation/results.json`
- [ ] T007 Fusionner dans `main`, construire depuis la racine principale, sauvegarder le binaire actif, déployer, relancer uniquement `com.bridget.daemon` et `com.bridget.t3`, vérifier le build-id et un message réel, pousser vers `https://github.com/guthubrx/bridget.git`, puis nettoyer le worktree et la branche

## Dépendances

1. T001 précède toute modification.
2. T002 précède T003, car le lot réutilise le libellé central.
3. T004 peut avancer indépendamment de T002 et T003.
4. T002, T003 et T004 doivent être terminées avant T005.
5. T005 précède T006. T006 précède T007.

## Exécution parallèle possible

- T004 touche uniquement le registre de profils et peut être traité en parallèle
  de T002.
- T003 attend T002 pour éviter deux définitions du format visible.

## Stratégie d’implémentation

Le premier incrément utile est T002 : les transports qui utilisent déjà
`sender_label()` deviennent lisibles. T003 ferme l’écart des lots. T004 répare la
cause qui empêche certains titres T3 d’atteindre le message. Aucun incrément
n’ajoute une dépendance, une table ou un service.

## Validation Article XIX/XX

- T002 centralise une règle déjà existante et ajoute zéro abstraction.
- T003 supprime un accès direct divergent.
- T004 renforce le contrat d’un `ensure_*` existant dans sa transaction actuelle.
- T005 et T006 rendent les hypothèses et les limites observables pour le prochain
  mainteneur.
