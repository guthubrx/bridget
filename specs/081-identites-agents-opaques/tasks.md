# Tâches - SPEC-081

## P1 - Contrat et identité Bridget

- [x] T001 [US2] Ajouter des tests de validation UUID/principaux réservés et d'absence d'auto-nom dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-core/src/router.rs. Résultat : le routeur refuse un nom legacy et indexe une connexion par agent_id.
- [x] T002 [US2] Remplacer la clé name par agent_id dans Router, RegisteredAgent, erreurs et tests de /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-core/src/router.rs. Résultat : aucune résolution d'alias.
- [x] T003 [US2] Faire évoluer BridgetMessage, Register et AgentInfo dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-core/src/message.rs et /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-transport/src/protocol.rs. Résultat : annuaire v2 agent_id/display_name et handshake sans name.
- [x] T004 [US1] Étendre AgentProfileStore dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-daemon/src/agent_profile.rs. Résultat : agent_id reste la seule clé vivante et la projection par ID fournit display_name.
- [x] T005 [US1] Ajouter les tests de migration du store de profils : conversion des alias, rejet d'ambiguïté et suppression des alias après succès. Résultat : migration locale idempotente et sans fuite legacy.

## P1 - Runtime, flotte, messages et UI

- [x] T010 [US2] Migrer les usages daemon, présence, flotte, exécution et commandes de cycle de vie de name vers agent_id dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-daemon/src/. Résultat : relance et reprise ciblent l'agent_id.
- [x] T011 [US2] Migrer le wrapper et les transports dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-daemon/src/wrapper.rs et /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-transport/src/. Résultat : handshake agent_id et prompt seulement display_name.
- [x] T012 [US1] Faire évoluer la projection et les routes du relais dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-daemon/src/ui.rs et assets UI. Résultat : API v2 cible agent_id, UI affiche display_name sans fuite.
- [x] T013 [US1] Ajouter les tests d'intégration UI/messages : renommage stable, conversation affichée et recherche sans nom legacy. Résultat : vérification observable des critères US1.

## P1 - Maicie et migration

- [x] T020 [US3] Remplacer agent_name par agent_id dans config, profils, sélection et sorties Maicie sous /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/plugins/maicie/src/. Résultat : la sélection lie un agent_id attesté.
- [x] T021 [US3] Étendre domaine/store/outbox/routines Maicie pour agent_id et requires_retarget. Résultat : une cible supprimée n'est jamais livrée.
- [x] T022 [US3] Ajouter les tests de migration Maicie : délégation active convertie, référence retirée retargetée, outbox bloquée. Résultat : payload et index restent cohérents.
- [x] T023 [US4] Implémenter la sous-commande bridget identity migrate --dry-run et --apply dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/crates/bridget-daemon/src/cli.rs. Résultat : préflight, sauvegardes, journal de progression, transactions par ressource et reprise.
- [x] T024 [US4] Migrer fleet.json, stores SQLite Bridget et configuration Maicie dans la commande. Résultat : les agents actifs sont relançables par agent_id et stopped reste stopped.

## P1 - Vérification et documentation

- [x] T030 Ajouter les scénarios Gherkin de migration et de retarget dans /home/moi/bridget-referent/.worktrees/session-081-identites-agents-opaques/specs/081-identites-agents-opaques/acceptance.feature. Résultat : critères exécutables ou explicitement tracés.
- [x] T031 Exécuter formatage et tests ciblés Core, Daemon, Transport et Maicie. Résultat : commandes et sorties consignées dans implementation.md.
- [x] T032 Relecture de convergence spec -> plan -> code -> tests. Résultat : aucune exigence sans preuve ou tâches ajoutées explicitement.

## Ordre

T001 -> T002 -> T003 -> T004 -> T005 -> T010 -> T011 -> T012 -> T013 -> T020 -> T021 -> T022 -> T023 -> T024 -> T030 -> T031 -> T032
