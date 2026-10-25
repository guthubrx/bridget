# Tâches 133 — Relais Bridget pour les sous-agents internes

Statut : Completed — 12/12.
Branche : `session-133-relais-sous-agents`.
Plan : `specs/133-relais-sous-agents/plan.md`.

## Phase 1 — Préparation

- [x] T001 Consigner la baseline ciblée déjà verte et les limites de validation dans `specs/133-relais-sous-agents/implementation.md`; résultat observable : commandes, nombres et absence d'accès à l'état installé sont explicites [SC-13305].

## Phase 2 — Fondation commune

- [x] T002 [P] Ajouter les tests RED du schéma `DelegatedOrigin`, de la compatibilité des anciens messages et de l'exclusion de la provenance de l'empreinte de rejeu dans `crates/bridget-core/src/message.rs` et `crates/bridget-daemon/src/communication.rs`; résultat observable : changement de provenance seul conserve les bytes de rejeu, ancien JSON reste identique [FR-13305] [FR-13306] [FR-13310].
- [x] T003 Implémenter le champ optionnel fermé `delegated_origin` dans `crates/bridget-core/src/message.rs` et confirmer le canon existant dans `crates/bridget-daemon/src/communication.rs`; résultat observable : T002 passe sans modifier la référence historique [FR-13305] [FR-13306] [FR-13310].

## Phase 3 — US1 : contacter un autre agent

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/tasks.md#extrait-1).

Test indépendant : arbre principal → enfant → MCP, envoi idempotent vers un faux destinataire, provenance visible et demande suivie possédée par le parent.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/tasks.md#extrait-2).

## Phase 4 — US2 : autorité minimale et refus CLI

Objectif : aucun outil de contrôle et aucune commande CLI sensible ne récupère l'autorité du parent.

Test indépendant : depuis un contexte enfant, `who` et `send` passent ; renommage, annulation, journal, artefact, contrôle, Maicie et CLI sont refusés avant effet.

- [x] T008 [P] [US2] Ajouter les tests RED de matrice MCP et le garde-fou d'inventaire CLI dans `crates/bridget-daemon/src/mcp.rs` et `crates/bridget-daemon/src/cli.rs`; résultat observable : tous les outils hors `who`/`send` attendent `delegated_tool_forbidden` et aucune commande sensible n'évite la résolution centrale [FR-13303] [FR-13304].
- [x] T009 [US2] Appliquer le refus avant handler dans `crates/bridget-daemon/src/mcp.rs` et `delegated_mcp_only` pour la résolution CLI dans `crates/bridget-daemon/src/mcp_identity.rs`; résultat observable : zéro appel du handler ou du daemon lors des refus [FR-13303] [FR-13304].

## Phase 5 — US3 : ambiguïtés, péremption et propriété

Objectif : aucune preuve invalide ne tombe sur l'autorité principale et aucun pont ne retire la preuve d'un autre.

Test indépendant : marqueur enfant périmé avec parent principal vivant, PID recyclé, symlink, fichier trop grand, parent ambigu, pont concurrent et arrêt du propriétaire.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/tasks.md#extrait-3).

## Phase 6 — US4 et validation transversale

Objectif : les agents principaux et messages historiques restent inchangés.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/tasks.md#extrait-4).

- [x] T012 [US4] Exécuter la compatibilité, la convergence exigence→preuve, la contre-revue finale et l'audit; mettre `specs/133-relais-sous-agents/implementation.md`, `specs/133-relais-sous-agents/tasks.md` et le statut de `specs/133-relais-sous-agents/spec.md` à jour seulement après `cargo test --workspace`, `cargo fmt --check`, `cargo clippy -D warnings` et `cargo build --release` vérifiés, avec toute limite du harnais reproduite sur `main` et consignée [FR-13310] [FR-13311] [SC-13301] [SC-13302] [SC-13303] [SC-13304] [SC-13305] [SC-13306].

## Dépendances

- T001 peut démarrer immédiatement.
- T002 précède T003.
- T003 précède T004 à T007.
- T004 précède T005, puis T006.
- T007 peut suivre T003 en parallèle de T004 à T006.
- T008 précède T009.
- T010 précède T011.
- T005 et T009 précèdent T010, car le test causal exerce les deux résolutions.
- T012 attend T001 à T011.

## Exemples de parallélisation sûre

- Après T003 : T004 et T007 touchent des fichiers distincts.
- Après T006 : T008 et la préparation de T010 peuvent avancer sur des modules distincts, sans éditer le même fichier au même moment.
- Les commandes de validation indépendantes de T012 peuvent être lancées en parallèle seulement après gel du diff.

## Stratégie d'implémentation

La première tranche testable est T002 à T007 : preuve, envoi et provenance.
Le pipeline ne s'arrête pas à cette tranche. T008 à T012 sont obligatoires pour
fermer l'escalade de droits, les cas périmés et la compatibilité globale.

Chaque tâche de code exige une relecture Article XIX/XX : nécessité, solution la
plus simple, hypothèses, vérifications, non vérifié, code évité et complexité
ajoutée. Aucune dépendance, table, service ou identité durable n'est autorisé.
