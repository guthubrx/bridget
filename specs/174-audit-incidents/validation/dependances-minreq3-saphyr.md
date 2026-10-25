# Dépendances minreq 3 et serde-saphyr — 24 octobre 2026

## Correction

Le client HTTP `minreq` passe de 2.14.1 à 3.0.0. La crate `serde_yaml` est remplacée par `serde-saphyr` 2.0.0. Les détails et les raisons sont dans l'ADR-170 (`docs/decisions/170-minreq-3-et-serde-saphyr.md`).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/dependances-minreq3-saphyr.md#extrait-1).

Versions résolues : `minreq` 3.0.0, `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-roots` 1.0.11, `serde-saphyr` 2.0.0. `rustls` 0.21.12, `rustls-webpki` 0.101.7, `webpki-roots` 0.25.4, `serde_yaml` et `unsafe-libyaml` ont disparu du graphe.

## Vérifications faites sans compilation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/dependances-minreq3-saphyr.md#extrait-2).

## Tests à exécuter

Aucun build ni test Cargo n'a été lancé dans cette session. ROOT doit lancer, sur le cache unique :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/dependances-minreq3-saphyr.md#extrait-3).

## Limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/dependances-minreq3-saphyr.md#extrait-4).
