# Validation finale - SPEC-065

**Date**: 2026-08-30
**Branche**: `session-065-registre-identite-projets`
**Base**: `main` `04fd9c6`
**Portée**: aucune publication, aucun push, aucun déploiement et aucun
redémarrage de service.

## Commandes réussies

| Commande | Résultat exact |
|---|---|
| `/home/moi/.cargo/bin/cargo fmt --check` | succès, code de sortie 0 |
| `/home/moi/.cargo/bin/cargo check -p bridget-daemon -p maicie` | succès, code de sortie 0 |
| `/home/moi/.cargo/bin/cargo test --workspace --no-run` | succès, code de sortie 0 |
| `cargo test -p bridget-daemon --test project_binding_integration_test` | 1 passé, 0 échec |
| `cargo test -p bridget-daemon lifecycle::tests::cwd_projet_accepte_racine_descendante_et_worktree_lie_mais_refuse_un_voisin --lib` | 1 passé, 0 échec |
| `cargo test -p bridget-daemon --test execution_store_test reference_projet_du_snapshot_survit_au_redemarrage_sans_reduction` | 1 passé, 0 échec |
| `cargo test -p bridget-daemon migration_v6_ajoute_les_references_projet_apres_une_base_deja_en_v5 --lib` | 1 passé, 0 échec |
| `cargo test -p bridget-daemon spec_068_faits_runtime_delegues_restent_ordonnes_et_accuses --lib` | 1 passé, 0 échec |
| `cargo test -p bridget-daemon fleet::tests::reprise_expose_les_generations_en_vol_dans_l_ordre_des_noms --lib` | 1 passé, 0 échec |
| `cargo test -p maicie --test contract execution_projection -- --test-threads=1` | 8 passés, 0 échec |
| `cargo test -p maicie project_correlation_tests --lib -- --test-threads=1` | 2 passés, 0 échec |
| `cargo test -p bridget-daemon --test build_id_integration_test daemon_et_cli_reels_transmettent_et_comparent_le_build_id -- --nocapture` | 1 passé, 0 échec, 3 filtrés |

## Résultats non verts, hors SPEC-065

| Commande | Résultat | Analyse |
|---|---|---|
| `/home/moi/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` | échec, code 101 | Deux diagnostics préexistants: `collapsible_if` dans `crates/bridget-transport/src/codex_app_server.rs:2215` et `should_implement_trait` dans `crates/bridget-transport/src/protocol.rs:1103`. |
| `/home/moi/.cargo/bin/cargo test --workspace` | échec pendant un test d'attente de wrapper, rejoué seul avec succès | Instabilité de contention observée. Le test isolé `reprise_apres_renommage_reel_conserve_le_nouveau_nom` passe: 1 passé, 0 échec. |
| `/home/moi/.cargo/bin/cargo test --workspace -- --test-threads=1` | échec, code 101, 804 passés, 3 échecs, 9 ignorés parmi les tests exécutés | Les trois échecs sont dans `crates/bridget-daemon/tests/managed_parity_test.rs`: une matrice FR-008 et deux scénarios de reprise MCP. Aucun fichier de ce test n'est modifié par SPEC-065. |

Le test `build_id_integration_test` a été rendu robuste à l'avertissement
opérationnel de disque, qui dépend de l'espace libre réel de l'hôte. Le verdict
de build-id reste comparé à l'identique. Cette correction est un commit séparé
de l'implémentation SPEC-065.

## Verdict

Les preuves ciblées de SPEC-065 sont vertes. La branche n'est pas déclarée
validée au niveau workspace tant que les deux diagnostics Clippy et les trois
échecs MCP de baseline ne sont pas traités ou explicitement acceptés lors de
l'intégration.
