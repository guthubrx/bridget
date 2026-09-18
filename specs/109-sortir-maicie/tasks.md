# Tâches 109

Statut: Implemented — 7/7 (2026-09-18). Recette complète 1513/0, fmt et clippy verts.

- [x] T001 Renommer la valeur de service réservé et la capacité dans crates/bridget-transport/src/protocol.rs ; attendu : `guichet` et `guichet_v1` sérialisés, tests de contrat verts.
- [x] T002 Renommer le producteur de boîte humaine dans crates/bridget-transport/src/protocol.rs ; attendu : `guichet` sérialisé.
- [x] T003 Propager les renommages dans crates/bridget-daemon (daemon, mcp, human_inbox, store, cli, wrapper) ; attendu : compilation sans erreur.
- [x] T004 Renommer les quatre outils MCP et leurs schémas dans crates/bridget-daemon/src/mcp.rs ; attendu : catalogue de vingt outils, quatre préfixés guichet_.
- [x] T005 Mettre à jour les tests et fixtures ; attendu : recette complète verte.
- [x] T006 Neutraliser la documentation et la skill ; attendu : zéro occurrence du nom du produit hors scripts d'exploitation.
- [x] T007 fmt, clippy, recette complète, puis consigner dans specs/109-sortir-maicie/implementation.md.
