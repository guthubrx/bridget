# Journal d'implémentation — 098 Adaptateur adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-2).

## T002 — Dépendances et baseline
- `minreq = { version = "2", default-features = false }` (crate daemon), `uuid` avec `v5` (workspace) ; `cargo fetch` puis build vert ; fmt et clippy verts. La suite complète est celle de `main` au départ du worktree (1328/0/56) ; elle sera rejouée en T014.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-3).

## T005 — Primitives du wrapper rendues visibles
- `deliver_idempotent_to_interactive`, `IdempotentDeliveryTracker::{open, open_at}`, `connect_and_register_with_domain_at`, `send_wrapper_message`, `record_interactive_turn`, `AttachRelayWorker::{start, subscribe, unsubscribe, shutdown, reset_generation}` en `pub(crate)` ; comportement inchangé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/098-t3code-adapter/implementation.md#extrait-7).

## T014 — Gates
- Faux échec d'environnement identifié : `daemon_guard_nettoie_apres_une_panique_injectee` appelle `lsof`, qui vit dans `/usr/sbin` et manquait au `PATH` de la recette de test ; avec `/usr/sbin`, le test est vert. La recette de gate ajoute donc `/usr/sbin`.

## Contre-revue #2 et corrections (bridget-revue, BLOCKED → corrigé)
- Six objections sur l'implémentation, cinq retenues et corrigées, une close par la revue (détail et preuves dans adversarial-review-bridget-revue.md, tour 4).
- Corrections : corrélation prouvée au lieu d'ordinale (nouveau verdict `Ambiguous`), attente écrite avant le dispatch avec identifiants déterministes, identifiant de réponse stable et sauvegarde après chaque décision, verrou `auth_failed` après refus d'une session neuve, aucune session laissée orpheline (révocation immédiate, `orphan-sessions.json`, `uninstall` qui refuse d'effacer sans révocation acquise, rollback du manifeste).
- Défaut trouvé en écrivant le test réclamé par l'objection 6 : la politique du fil était figée à l'ouverture du lien. Elle est désormais lue au moment du dispatch, ce qui supprime aussi un champ d'état.
- Tests ajoutés : 4 unitaires (tour sans réponse, tour de sous-agent, fil au repos sans tour, identifiant de réponse stable) et 2 renforcements d'oracle (verrou d'émission dans la durée, politique changée entre deux sondages). 17 tests unitaires et 3 oracles verts.
