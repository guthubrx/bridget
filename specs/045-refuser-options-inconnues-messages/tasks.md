# Tâches 045 — Refuser les options inconnues dans les messages

- [x] T001 Consigner l'inventaire, les deux contrats de texte libre et le
  périmètre dans `specs/045-refuser-options-inconnues-messages/spec.md`.
- [x] T002 [US1] Ajouter les témoins rouges de refus et d'absence de message
  sérialisé dans `crates/bridget-daemon/tests/cli_arguments_integration_test.rs`.
- [x] T003 [US2] Ajouter les contrôles positifs du séparateur dans
  `crates/bridget-daemon/tests/cli_arguments_integration_test.rs`.
- [x] T004 [US3] Ajouter les témoins des valeurs manquantes de `--to` et
  `--from` dans `crates/bridget-daemon/tests/cli_arguments_integration_test.rs`.
- [x] T005 [US1] [US2] [US3] Durcir `cmd_send` et `cmd_reply` dans
  `crates/bridget-daemon/src/cli.rs` sans modifier la cohérence des options.
- [x] T006 Rejouer les mutants de garde et de séparateur, restaurer avec
  condensats identiques et exécuter les portes finales.
- [x] T007 Finaliser `specs/045-refuser-options-inconnues-messages/implementation.md`,
  relire le diff, committer et publier la tête.
