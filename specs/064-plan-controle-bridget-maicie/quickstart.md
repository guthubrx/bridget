# Quickstart de validation : Plan de contrôle Bridget et Maicie

Ce document décrit les parcours à rendre exécutables pendant les futurs lots.
Il ne constitue pas une preuve d'implémentation dans la session 064.

## Préconditions futures

- worktree propre sur le lot concerné ;
- binaire fournisseur identifié par chemin, version et empreinte ;
- base de test éphémère ;
- aucun agent ou daemon de production utilisé par les tests isolés ;
- activation réelle seulement depuis une release admise.

## Parcours A - Steering Codex réellement corrélé

1. Démarrer un tour Codex de test qui reste actif.
2. Envoyer une correction avec un `client_message_id` connu.
3. Faire émettre un `item.id` différent du `clientId`.
4. Vérifier que seul le bon `item/started.userMessage.clientId` produit la
   visibilité et l'acquittement.
5. Vérifier qu'un accusé `turn/steer` seul ne produit aucun acquittement.

Résultat attendu : un acquittement exact sur consommation et aucun repli.

## Parcours B - Crash à chaque frontière

Injecter successivement un arrêt :

- après persistance de la soumission ;
- après création de la livraison ;
- après réception wrapper ;
- après acceptation fournisseur ;
- après visibilité ;
- avant acquittement.

Résultat attendu : après reprise, aucune perte, aucun doublon et une issue
observable pour chaque soumission.

## Parcours C - État opérateur

Faire passer un agent par :

- connecté et inactif ;
- travail en file ;
- tour actif ;
- attente d'autorisation ;
- attente humaine ;
- tour sans progrès ;
- interruption ;
- terminaison.

Résultat attendu : chaque état, son âge, sa preuve et l'action autorisée sont
visibles sans ouvrir le JSONL.

## Parcours D - Graphe parent-enfant

1. Créer un parent et deux enfants avec mandats différents.
2. Vérifier les liens, rôles et chemins.
3. Atteindre la limite de profondeur et constater le refus sans agent partiel.
4. Arrêter le parent et terminer un enfant.
5. Vérifier la politique de conservation et la remise du résultat.

Résultat attendu : aucun enfant ou résultat orphelin silencieux.

## Parcours E - Reprise native et reconstruction

1. Reprendre un thread avec un fournisseur compatible.
2. Vérifier la conservation de l'identité fournisseur.
3. Reprendre le même scénario avec un fournisseur sans capacité native.
4. Vérifier que la reconstruction est déclarée et possède une nouvelle identité.
5. Tester un fork et vérifier l'ascendance sans mutation de l'original.

Résultat attendu : le mode réel de reprise est toujours visible.

## Parcours F - Frontière Maicie

1. Créer une délégation et soumettre son travail.
2. Observer l'exécution, puis rendre la projection Bridget indisponible.
3. Terminer techniquement l'exécution.
4. Vérifier qu'aucune étape ne clôt ou ne réécrit l'objectif automatiquement.
5. Restaurer le flux et vérifier la reprise par curseur sans doublon.

Résultat attendu : faits runtime à jour, vérité métier inchangée sans décision.

## Parcours G - Version et capacités fournisseur

1. Présenter le Codex configuré puis une version plus ancienne.
2. Présenter Claude stream-json puis Cursor via `cursor-agent ... acp`.
3. Vérifier séparément le fournisseur Cursor et son chemin d'exécution ACP.
4. Vérifier les capacités observées et le contrat correspondant à chaque chemin.
5. Demander une opération absente à une version ou au contrat ACP observé.
6. Vérifier le refus ou repli prévu et son événement d'observabilité.

Résultat attendu : aucune capacité supposée à partir du seul nom du fournisseur,
et aucun adaptateur Cursor distinct du transport ACP commun.

## Parcours H - Budgets et continuation

1. Définir une limite de temps, d'usage et de descendants.
2. Consommer chaque limite séparément puis simultanément.
3. Vérifier pause ou arrêt et absence de nouveau tour après la borne.
4. Rendre l'agent inactif avec du travail restant et politique valide.
5. Vérifier une seule continuation, puis aucune clôture métier implicite.

Résultat attendu : autonomie bornée, coût attribué et décision métier préservée.

## Commandes de validation prévues

Les chemins de tests seront fixés dans `tasks.md`. La validation utilisera les
commandes Rust ciblées du workspace, puis la suite des crates touchées. Les
mesures réelles seront séparées des faux fournisseurs et ne seront lancées
qu'après activation contrôlée.

## Matrice exigences, scénarios et preuves

Chaque ligne est reliée à une tâche citée dans `tasks.md`; les commandes sont
les preuves minimales à compléter au fil des lots.

| Exigence | Scénario | Commande de preuve |
|---|---|---|
| FR-001 | A, B | `cargo test -p bridget-daemon work_submission` |
| FR-002 | A | `cargo test -p bridget-core message` |
| FR-003 | A, B | `cargo test -p bridget-daemon execution_lifecycle` |
| FR-004 | C | `cargo test -p bridget-daemon execution_lifecycle` |
| FR-005 | A | `cargo test -p bridget-transport codex_app_server` |
| FR-006 | B | `cargo test -p bridget-daemon execution_store` |
| FR-007 | A, B | `cargo test -p bridget-core execution` |
| FR-008 | B | `cargo test -p bridget-daemon work_submission` |
| FR-009 | A, B | `cargo test -p bridget-daemon managed_wrapper` |
| FR-010 | A | `cargo test -p bridget-transport protocol` |
| FR-011 | D | `cargo test -p bridget-daemon agent_graph` |
| FR-012 | D | `cargo test -p bridget-daemon agent_graph` |
| FR-013 | A, D | `cargo test -p bridget-daemon managed_supervisor` |
| FR-014 | D | `cargo test -p bridget-daemon agent_graph` |
| FR-015 | D | `cargo test -p bridget-daemon agent_graph` |
| FR-016 | E | `cargo test -p bridget-daemon execution_resume` |
| FR-017 | E | `cargo test -p bridget-transport managed_session` |
| FR-018 | E, G | `cargo test -p bridget-transport provider_contract` |
| FR-019 | E | `cargo test -p bridget-daemon execution_resume` |
| FR-020 | E | `cargo test -p bridget-daemon execution_resume` |
| FR-021 | F, H | `cargo test -p maicie` |
| FR-022 | F | `cargo test -p bridget-daemon mission_boundary` |
| FR-023 | F, H | `cargo test -p maicie` |
| FR-024 | F, H | `cargo test -p maicie` |
| FR-025 | F | `cargo test -p maicie` |
| FR-026 | F | `cargo tree -p bridget-daemon` |
| FR-027 | G | `cargo test -p bridget-transport provider_contract` |
| FR-028 | G | `cargo test -p bridget-transport provider_contract` |
| FR-029 | A, G | `cargo test -p bridget-daemon capabilities_integration` |
| FR-030 | A, G | `cargo test -p bridget-daemon capabilities_integration` |
| FR-031 | A, G | `cargo test -p bridget-daemon capabilities_integration` |
| FR-032 | G | `cargo test -p bridget-transport provider_contract` |
| FR-033 | C | `cargo test -p bridget-daemon ui_relay` |
| FR-034 | C | `cargo test -p bridget-daemon execution_observability` |
| FR-035 | C, H | `cargo test -p bridget-daemon execution_observability` |
| FR-036 | C | `cargo test -p bridget-daemon execution_observability` |
| FR-037 | H | `cargo test -p bridget-daemon execution_budget` |
| FR-038 | H | `cargo test -p bridget-daemon execution_budget` |
| FR-039 | H | `cargo test -p bridget-daemon execution_budget` |
| FR-040 | G | `cargo test -p bridget-transport provider_contract` |
| FR-041 | G | `cargo test -p bridget-transport provider_contract` |
| FR-042 | B | `cargo test -p bridget-daemon execution_store` |
| FR-043 | B, C | `cargo test -p bridget-daemon registry` |
| FR-044 | B | `cargo test -p bridget-core message` |

| Succès | Scénario | Preuve de clôture |
|---|---|---|
| SC-001 | A | parcours US1 T035 |
| SC-002 | A | SPEC-063 et T035 |
| SC-003 | B | migrations T017 et T044 |
| SC-004 | C | UI T026, T033 et T035 |
| SC-005 | D | parcours US2 T044 |
| SC-006 | E | parcours US3 T052 |
| SC-007 | F, H | parcours US4 T061 et US6 T079 |
| SC-008 | G | baseline T002 et parcours US5 T070 |
| SC-009 | C | alertes T081 à T083 |
| SC-010 | A à H | T087 puis T089 |
| SC-011 | B, C | rollout T085 puis T089 |

## Validation realisee dans le worktree isole - 2026-08-29

Les scenarios Rust ont ete couverts par la suite workspace complete, puis par
les suites de regression les plus sensibles. Aucune de ces commandes ne touche
un daemon, une base ou un agent de production.

```text
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-13 /home/moi/.cargo/bin/cargo test --workspace --quiet
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-13 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test managed_wrapper_test --quiet
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-13 /home/moi/.cargo/bin/cargo test -p maicie --test schema_migration_guard --quiet
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-13 /home/moi/.cargo/bin/cargo test -p bridget-transport --test provider_contract_test --quiet
CARGO_TARGET_DIR=/tmp/bridget-spec064-target-13 /home/moi/.cargo/bin/cargo test -p bridget-daemon --test capabilities_integration_test --quiet
```

Verdict : succes. Les scenarios A a H sont exerces par les tests de crate et
les preuves US1 a US6. Cursor a ete exerce avec son binaire reel via le
transport ACP commun : session, prompt et `session/cancel` sont attestes dans
`evidence/us5-providers.md`.
