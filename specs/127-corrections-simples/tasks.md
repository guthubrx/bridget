# Tâches 127
Statut: Implemented - 5/5.
- [x] T001 `NO_REPLY_NOTICE` (bridget-core) repris par codex_app_server, acp, enveloppe ; pont T3 : message seul, lot, notification.
- [x] T002 `resolve_spawn_agent_type_for_posture` : type absent du registre → `UnknownType` (types connus, registre).
- [x] T003 `resolve_name_in_directory` : début d'UUID ≥ 6 unique ; `send` CLI (nom ou préfixe) et MCP (préfixe hexadécimal seul : un nom reste transmis tel quel).
- [x] T004 Tests `spec127_*`, `spec_088` complété, assertions de texte mises à jour (t3code, transport, core) ; skill et CHANGELOG.
- [x] T005 Première recette : 11 échecs dus à la 127 (MCP : connexion d'annuaire imprévue pour des noms factices ; textes T3) ; corrigés, MCP + t3code 136/136. Fichier vide retiré à la livraison.
