# Audit de réutilisation de l'existant — Maicie v3

## Decision

Statut: PASS_WITH_REQUIRED_GATES  
Date: 2026-08-22  
Feature dir: `specs/011-maicie-orchestration`

Maicie réutilise le workspace, l'annuaire et le cycle de demande Bridget, mais
ne reconstruit ni transport, journal interne ACP ni gestion de processus. Les
contrats publics sessions 008 et 009 sont des gates avant leurs lots respectifs.

## Synthèse

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 9 |
| Items audites | 9 |
| Reutilisations prevues | 6 |
| Existants pertinents | 3 |
| Duplications evidentes | 0 |
| Gates externes | 2 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Workspace Rust | workspace Cargo | `Cargo.toml:1-20` | nouveau membre sous plugins/maicie |
| Demande suivie/timeout | Store Bridget | `crates/bridget-daemon/src/store.rs:44-86` | Bridget conserve livraison et horloge |
| Identité client de message | modèle Bridget existant | `specs/010-mcp/contracts/outils-mcp.md:21-44` | exposer l'id client idempotent au port Maicie |
| Annuaire | ListAgents/AgentInfo | `crates/bridget-transport/src/protocol.rs:47-48,129-170` | sélection déterministe par tags |
| Abonnement runtime | Subscribe/Seq/Gap/End session 008 | `specs/008-attach/contracts/abonnement-attach.md` | gate obligatoire, aucune lecture interne |
| Lifecycle agent | SpawnOrder session 009 | `specs/009-daemon-spawn/` | gate obligatoire, aucun spawn Maicie |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Outbox idempotente | déduplication de message Bridget | `crates/bridget-daemon/src/daemon.rs` | vérifier que le contrat public expose id/recherche par id ; sinon étendre Bridget avant Maicie |
| Permission ACP | réponse auto du registre 007 | `specs/007-transport-acp/contracts/livraison-acp.md:49-71` | afficher l'historique, ne pas intercepter |
| Observabilité | SQLite/events Bridget | `docs/decisions/002-persistence-inspiree-systeme-tiers.md:46-97` | journal privé Maicie, aucune copie de transport |

## Duplications evidentes

Aucune. Le superviseur historique, le scheduler, le journal ACP interne et
le spawn local sont explicitement exclus.

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item a une preuve ou un gate explicite
- [x] Session 008 exigée avant T017–T020
- [x] Session 009 exigée avant T021–T023
- [x] Extension Bridget requise avant T006 si l'idempotence client n'est pas publique
