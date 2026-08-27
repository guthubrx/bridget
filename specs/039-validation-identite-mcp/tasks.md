# Tâches 039 — Valider l'identité MCP

- [x] T001 Mesurer les deux grammaires caractère par caractère et établir leur
  relation d'inclusion.
- [x] T002 Inventorier les noms réellement visibles et prouver qu'ils passent
  tous la garde canonique.
- [x] T003 Figer la propriété, le périmètre minimal et les deux contrôles
  opposés dans `specs/039-validation-identite-mcp/spec.md`.
- [x] T004 [US1] Ajouter l'oracle Unicode qui prouve zéro exécution dans
  `crates/bridget-daemon/src/mcp.rs`.
- [x] T005 [US2] Ajouter le contrôle ASCII qui prouve une exécution dans
  `crates/bridget-daemon/src/mcp.rs`.
- [x] T006 [US1] Remplacer la grammaire privée par la garde canonique dans
  `crates/bridget-daemon/src/mcp_identity.rs`.
- [x] T007 Rejouer le mutant Unicode, le contrôle ASCII et la restauration.
- [x] T008 Comparer base et tête sur l'univers du paquet daemon, puis jouer fmt,
  clippy strict et diff-check.
- [x] T009 Effectuer la revue hostile sécurité et documenter les preuves dans
  `specs/039-validation-identite-mcp/implementation.md`.
- [x] T010 Committer, pousser la tête gelée et rendre le matériel après
  vérification locale/distante.
