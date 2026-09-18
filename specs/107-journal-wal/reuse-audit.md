# Audit de l'existant 107

## Décision

Statut: PASS

## Synthèse

Le plan n'introduit aucun service, table, endpoint, helper ni dépendance. Une seule règle d'ouverture est
ajoutée à un point d'entrée existant.

| Élément proposé | Existant et preuve | Décision |
|---|---|---|
| Réglage journal WAL | aucune occurrence de `journal_mode` (`rg` dans `crates/*/src`) | CRÉER (un pragma) dans `Store::open` (`store.rs:65`) |
| Point d'ouverture | `Store::open` premier ouvreur (`daemon.rs:2919`) ; autres ouvreurs `idempotency.rs:471`, `execution_store.rs:213`, `agent_profile.rs:174`, `artifact_store.rs:179` | RÉUTILISER `Store::open` ; ne pas dupliquer dans les autres (mode persistant) |
| Lecteur lecture seule | `Store::open_read_only` (`store.rs:84`), `store_schema::validate_existing` (`store_schema.rs:33`) | RÉUTILISER pour les tests de non-blocage |
| Tests de contention | `store.rs:513`, `daemon.rs:9416` (écrivain-écrivain), `tests/search_104_test.rs:997` (lecteur) | CONSERVER les deux premiers ; ADAPTER le troisième |
| Exclusion de déploiement | `scripts/deploy-remote.sh:71` `--exclude='*.db'` | ÉTENDRE avec `*.db-*` |

## Duplications évidentes

Aucune.

## Arbitrages

Aucun arbitrage utilisateur requis.

## Gate avant tasks

- [x] Éléments proposés recensés.
- [x] Recherche par nom et responsabilité exécutée.
- [x] Aucune duplication évidente.
- [x] Plan fondé sur l'existant.
