# Plan 100 — Observation et partage

Statut : Livré — main et release c680c6ce36c5, services vérifiés le 2026-09-16. Spec : spec.md. Socle 099 final : e1b83e95.

## Contexte technique

Workspace Rust, trois crates, sockets Unix JSONL, journal append-only versionné.
Pas de nouvelle dépendance, pas de service, pas de migration ou de moteur SQL.
Portée : daemon de communication local ; aucune fédération nouvelle des événements.

## Contrôle constitutionnel

Isolation dans le worktree 100. La demande du 2026-09-16 autorise maintenant
commit, fusion et installation après tests et sauvegarde ; la consigne initiale
sans déploiement reste applicable aux recettes de test. Aucun fournisseur actif
ne doit être interrompu. Protocoles SpecKit manuels car scripts/modèles absents.
Tests Rust natifs au lieu de pytest pour tester réellement les crates ; scénarios
Gherkin métier associés. Articles XVIII–XX : travail incrémental, mémoire et
temps bornés, aucune interprétation LLM des événements, réutilisation par défaut.

## Réutilisation de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/100-observation-partage/plan.md#extrait-1).

## Structure et vérification

Modifiés : crates/bridget-daemon/src/{attach.rs,communication/client.rs,cli.rs,
mcp.rs,daemon.rs,lib.rs}, crates/bridget-transport/src/protocol.rs et producteurs
structurés nécessaires. Module métier observation.rs nouveau seulement après
audit : responsabilités de filtrage/expiration/collision, tests unitaires purs.
Tests aux frontières socket/CLI/MCP et producteurs ; régressions 099, attach,
suite workspace dans la limite d'espace disponible, fmt et clippy.

## Complexité et responsabilité future

Extrait O(octets reçus) borné par budget/200 entrées/64 Kio de résultat.
Dispatch O(S), S≤128 ; filtres motif/chemin bornés ; écriture HashMap indexée,
purge O(F), F≤4096. Pas de scan récursif du projet, pas de boucle de polling des
fichiers ou du transcript. Chaque borne et limite sera visible dans la recette.

## Hors périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/100-observation-partage/plan.md#extrait-2).
