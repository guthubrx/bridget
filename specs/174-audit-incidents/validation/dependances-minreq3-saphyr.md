# Dépendances minreq 3 et serde-saphyr — 24 octobre 2026

## Correction

Le client HTTP `minreq` passe de 2.14.1 à 3.0.0. La crate `serde_yaml` est remplacée par `serde-saphyr` 2.0.0. Les détails et les raisons sont dans l'ADR-170 (`docs/decisions/170-minreq-3-et-serde-saphyr.md`).

Fichiers touchés : `crates/bridget-daemon/Cargo.toml`, `Cargo.lock`, `crates/bridget-daemon/src/decision_service.rs`, `crates/bridget-daemon/src/t3code_contract.rs`, `crates/bridget-daemon/src/t3code_mcp.rs`, `crates/bridget-daemon/src/reprise.rs`.

Versions résolues : `minreq` 3.0.0, `rustls` 0.23.46, `rustls-webpki` 0.103.16, `webpki-roots` 1.0.11, `serde-saphyr` 2.0.0. `rustls` 0.21.12, `rustls-webpki` 0.101.7, `webpki-roots` 0.25.4, `serde_yaml` et `unsafe-libyaml` ont disparu du graphe.

## Vérifications faites sans compilation

- `cargo metadata` : résolution du lockfile et téléchargement des sources.
- `cargo tree` : `minreq`, `serde-saphyr` et `aws-lc-sys` sont présents, `serde_yaml` et `unsafe-libyaml` sont absents.
- `rustfmt --check` : `reprise.rs`, `decision_service.rs` et `t3code_contract.rs` sont conformes. `t3code_mcp.rs` avait 17 écarts de format avant cette correction. Mes lignes ne les ajoutent pas.
- Lecture des sources de `minreq` 3.0.0 et de `serde-saphyr` 2.0.0 : signatures, types de retour, `Options` non exhaustif, macro `options!`, traitement de `!include` sans feature `include`, valeurs par défaut.
- Équivalent T3 : la copie `/home/moi/.cache/bridget-174-remaining/t3` ne contient ni TLS ni YAML. Ses délais (`supervisor.ts`, `session.ts`) concernent le transport T3, pas le client Rust. Ils ne sont pas modifiés.

## Tests à exécuter

Aucun build ni test Cargo n'a été lancé dans cette session. ROOT doit lancer, sur le cache unique :

1. `cargo build -p bridget-daemon` (un compilateur C est requis pour `aws-lc-sys`).
2. `cargo test -p bridget-daemon reprise::tests::pin_yaml` (deux nouveaux tests : pin réel légitime, pin avec `!include` refusé).
3. `cargo test -p bridget-daemon decision_service` (contrat HTTP du décideur, garde `MINREQ_TIMEOUT`).
4. `cargo test -p bridget-daemon t3code` et `cargo test -p bridget-daemon --test t3code_opaque_identifiers149_test`.
5. `cargo tree -p bridget-daemon -d` pour vérifier l'absence de doublons inattendus.

## Limites

- Le comportement de `serde-saphyr` n'a pas été comparé à `serde_yaml` 0.9 sur des cas limites. Les deux tests de pin ne couvrent pas toute la grammaire.
- Les valeurs par défaut (doublons refusés, budget par défaut) viennent de la documentation et du code source de `serde-saphyr` 2.0.0. Elles n'ont pas été mesurées à l'exécution.
- Le client T3 (`t3code_contract.rs`, `t3code_mcp.rs`) n'a pas été testé contre un serveur T3 réel.
