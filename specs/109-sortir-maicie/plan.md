# Plan 109 — Sortir le nom Maicie du noyau

## Décision

Renommage pur, sans changement de comportement. Aucune donnée persistée ne porte les valeurs
concernées, vérifié sur la base de production : tables de boîte humaine et de guichet vides,
aucune occurrence dans les 361 événements de coordination. Pas de migration, pas de double lecture.

## Table de renommage

| Avant | Après | Nature |
|---|---|---|
| service réservé `"maicie"` | `"guichet"` | valeur du contrat de service |
| `ServiceCapability::MaicieGuichet` | `ServiceCapability::GuichetV1` | sérialisé `guichet_v1` |
| `HumanInboxProducer::Maicie` | `HumanInboxProducer::Guichet` | sérialisé `guichet` |
| `maicie_delegate` | `guichet_delegate` | outil MCP |
| `maicie_objective_close` | `guichet_objective_close` | outil MCP |
| `maicie_registre_add` | `guichet_registre_add` | outil MCP |
| `maicie_request_status` | `guichet_request_status` | outil MCP |
| `MaicieStore` | `GuichetStore` | type interne |
| noms de tests `maicie_*` | `guichet_*` | lisibilité |

Les textes de documentation et de commentaires citant le produit sont réécrits en « service
compagnon » ou « service de guichet ».

## Périmètre

`crates/bridget-transport`, `crates/bridget-daemon`, `docs`, `skills`, `tests`. Les scripts
d'exploitation locale qui appellent le binaire compagnon restent inchangés dans le dépôt de
travail et sortent du périmètre publié.

## Vérification

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo test --workspace` avec environnement privé, puis recherche insensible à la casse.
