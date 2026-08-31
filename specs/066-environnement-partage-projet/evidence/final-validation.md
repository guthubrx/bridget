# Validation intermédiaire - SPEC-066

Date: 2026-08-30
Statut: In Progress. Cette preuve ne valide pas les tâches encore ouvertes.

## Contrôles réussis

- cargo fmt --all -- --check: succès.
- git diff --check: succès.
- cargo test -p bridget-daemon --lib -q: 709 réussites, 7 ignorés.
- cargo test -p bridget-daemon --test project_runtime_integration_test -q:
  1 réussite. Le test construit image locale par iidfile, prépare conteneur,
  vérifie uid, gid, HOME, XDG, socket, lecture fixture 0600 et state root.
- cargo test -p bridget-transport --test project_runtime_contract_test -q:
  2 réussites.
- Tests ciblés ingress et wrapper: refus sans réservation, falsification
  d identité ou d epoch, admission unique avant Register, identité runtime
  incomplète et repli Docker vers hôte interdit: réussites.
- cargo test -p bridget-daemon --test integration_test
  tests::reprise_apres_renommage_reel_conserve_le_nouveau_nom -- --exact:
  1 réussite après un dépassement de délai observé seulement pendant le premier
  passage workspace sous charge.
- cargo test --workspace spec_063_, spec_064_, spec_065_, spec_068_ et
  spec_075_: succès pour chaque filtre.

## Contrôles non verts hors périmètre SPEC-066

- cargo clippy --workspace --all-targets -- -D warnings: échec sur deux
  avertissements existants dans bridget-transport: collapsible_if dans
  codex_app_server.rs et should_implement_trait sur une méthode from_str de
  protocol.rs. Aucun fichier concerné par ces deux diagnostics n a été modifié
  pour SPEC-066.
- cargo test --workspace -q: échec 101, après rejeu. Les trois échecs restent
  matrice_fr008_compare_le_meme_corpus_et_les_frames_attach,
  reprise_codex_sans_amorcage_ne_decouvre_pas_mcp et
  reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees, tous dans
  crates/bridget-daemon/tests/managed_parity_test.rs. Le dépassement de délai
  initial dans integration_test ne se reproduit pas isolément.

## Conclusion

Les preuves ciblées Docker et les régressions filtrées sont vertes. La validation
workspace et clippy restent non verts pour les causes listées ci-dessus. Aucun
déploiement ni redémarrage de service n a été effectué.

## Validation finale - 2026-08-30

Cette section remplace le statut intermédiaire ci-dessus.

### Outils workspace

- `cargo fmt --check` : succès.
- `cargo clippy --workspace --all-targets -- -D warnings` : succès.
- `cargo test --workspace` : 717 réussites, 7 ignorés, 0 échec pour `bridget-daemon`; toutes les crates et tous les tests d'intégration ont terminé avec succès.

### Regressions et runtime Docker

- SPEC-065 : 11 tests ciblés réussis.
- SPEC-068 : 7 tests ciblés réussis.
- SPEC-075 : 2 tests ciblés réussis.
- `project_runtime_integration_test` : 2 réussites Docker réelles.
- `project_runtime_agents_test`, `project_runtime_ingress_test` et `project_runtime_mounts_test` : 1 réussite chacun.

Conclusion : toutes les tâches SPEC-066 sont prouvées sur fixtures. Aucun service de production n'a été touché pendant cette validation.
