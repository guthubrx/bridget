# Tâches 040 — Observateur d'espace disque fédéré

- [x] T001 Figer la mesure 0 octet candidat contre 47,3 Gio et les limites de
  sécurité dans `specs/040-observateur-espace-disque/spec.md`.
- [x] T002 Ajouter le fait d'espace libre optionnel au protocole dans
  `crates/bridget-transport/src/protocol.rs`.
- [x] T003 Propager l'attestation locale du wrapper à la présence puis à
  `AgentInfo` dans `crates/bridget-daemon/src/wrapper.rs` et
  `crates/bridget-daemon/src/daemon.rs`.
- [x] T004 Rendre le fait informatif dans `crates/bridget-daemon/src/cli.rs`.
- [x] T005 [US2] Ajouter le classificateur non destructif à racine explicite
  dans `crates/bridget-daemon/src/reaper.rs`.
- [x] T006 [US3] Ajouter le témoin agent actif sans descripteur, le candidat
  arrêté et le contrôle hors `TMPDIR` dans `crates/bridget-daemon/src/reaper.rs`.
- [ ] T007 Rejouer les mutants de garde active et de racine explicite, mesurer
  base/tête et documenter les preuves dans
  `specs/040-observateur-espace-disque/implementation.md`.
- [ ] T008 Vérifier le périmètre, publier et consigner le REX dans
  `specs/040-observateur-espace-disque/implementation.md`.
