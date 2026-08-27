# Tâches — Session 038

## Phase 1 — Contrat et décision

- [x] T001 Formaliser la spec, le contrat, le plan, l'ADR et les oracles dans `specs/038-regeneration-politique/` et `docs/decisions/013-regeneration-explicite-politique.md`

## Phase 2 — Inventaire vivant

- [x] T002 [US2] Ajouter la source mesurée et l'inventaire fermé dans `crates/bridget-transport/src/greffe_policy_refresh.rs`, avec compatibilité dans `crates/bridget-transport/src/greffe_authorization.rs`
- [x] T003 [US2] Scanner les marqueurs par PID et naissance dans `crates/bridget-daemon/src/mcp_identity.rs` et `crates/bridget-daemon/src/greffe_policy_refresh.rs`
- [x] T004 [US2] Exposer `scan` par le binaire dédié dans `crates/bridget-daemon/src/bin/bridget-greffe-policy-refresh.rs` sans toucher à `cli.rs`

## Phase 3 — Régénération fermée

- [x] T005 [US1] Calculer le remplacement des seules instances approuvées et préserver les principaux morts dans `crates/bridget-transport/src/greffe_policy_refresh.rs`
- [x] T006 [US1] Sérialiser sous verrou et remplacement privé atomique, avec prévisualisation par défaut, dans `crates/bridget-transport/src/greffe_policy_refresh.rs`
- [x] T007 [US3] Ajouter le chemin `refresh` et son rapport non secret dans `crates/bridget-daemon/src/greffe_policy_refresh.rs`

## Phase 4 — Oracles d'effet

- [x] T008 [US1] Ajouter les oracles sources complètes, zéro marqueur, remplacement, conservation et génération dans `crates/bridget-transport/src/greffe_policy_refresh.rs` et `crates/bridget-daemon/src/mcp_identity.rs`
- [x] T009 [US3] Éprouver le vrai binaire puis une mutation durable acceptée dans `crates/bridget-daemon/tests/greffe_policy_refresh_test.rs`, et rejouer les mutants de génération et d'écriture directe après correction

## Phase 5 — Validation et livraison

- [ ] T010 Exécuter compilation avant comptage, univers ciblés, dépendants, suite finale, formatage, clippy, revue hostile et REX dans `specs/038-regeneration-politique/implementation.md`

## Dépendances

```text
T001 -> T002 -> T003 -> T004 -> T005 -> T006 -> T007 -> T008 -> T009 -> T010
```

Les surfaces partagent le contrat de politique et la chaîne binaire ; aucune
tâche d'implémentation n'est parallèle afin d'éviter deux formats en circulation.

## Critères indépendants par story

- **US1** : une ancienne instance est remplacée, la génération croît et
  l'ancienne ne peut plus autoriser d'effet.
- **US2** : deux sources sont requises ; en omettre une, en vider une ou lire un
  marqueur ambigu laisse la politique octet-identique.
- **US3** : la sortie du vrai binaire est relue par la garde réelle et une
  mutation durable aboutit avec la nouvelle instance.

## MVP

T001 à T009 forment un seul incrément de sécurité : livrer sans contrôle positif
ou sans source distante complète reproduirait le défaut initial.
