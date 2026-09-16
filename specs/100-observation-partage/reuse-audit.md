# Audit de reutilisation de l'existant — 100

## Decision

Statut: PASS
Date: 2026-09-16
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/100-observation-partage/specs/100-observation-partage

Les mécanismes de journal et de messagerie sont réutilisés. L'abonnement à des
faits choisis est distinct du flux de journal attach et du flux guichet : aucun
des deux ne gère filtres/expiration/notification à un agent. Aucun doublon évident.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

Preuves relatives à /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/100-observation-partage, lignes avant implémentation.

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lecture extrait | AttachWindow/Subscribe | crates/bridget-transport/src/protocol.rs:1855,2360 | REUTILISER Tail/Seq et snapshot |
| Assemblage | AttachClientState::accept_fragment | crates/bridget-daemon/src/attach.rs:516 | REUTILISER sans second parseur de fragments |
| Connexion bornée | DaemonConnection | crates/bridget-daemon/src/communication/client.rs:550 | REUTILISER budget10s et lecture bornée |
| Partage | cmd_send/execute_send | crates/bridget-daemon/src/cli.rs:184 ; crates/bridget-daemon/src/mcp.rs:446 | REUTILISER envoi et reply ; nouvelle entrée journal uniquement |
| Identité abonné | live_connection_identity | crates/bridget-daemon/src/daemon.rs:7213 | REUTILISER preuve099, pas de champ owner client |
| Source faits | JournalLiveFeed / WriterCommand::Entry | crates/bridget-transport/src/journal.rs ; crates/bridget-daemon/src/wrapper.rs AttachJournalRelay | ETENDRE le post-flush avec métadonnées, pas de polling |
| Notification | deliver_to_agent / push_control_message_until | crates/bridget-daemon/src/daemon.rs:1260 | REUTILISER message/borne, pas un workflow |
| Filtrage/TTL/collisions | aucun équivalent métier | recherche rg ObservationRequest,file_collision,subscriptions,collision | CREER état observation, contrainte isolation des responsabilités daemon |
| Métadonnées écriture | kind=file Codex, kind=tool Claude/ACP | crates/bridget-transport/src/codex_app_server.rs:210 ; crates/bridget-transport/src/acp.rs:1756 | ETENDRE producteurs avant troncature ; pas parse du texte |
| Façades abonnement | catalogue MCP et dispatch CLI | crates/bridget-daemon/src/mcp.rs:440 ; crates/bridget-daemon/src/cli.rs:169 | ETENDRE avec contrat fermé, pas dépendance externe |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Abonnement | CoordinationSubscribe | crates/bridget-transport/src/protocol.rs:2208 | Déjà distingué au plan : service guichet, pas notification générique |
| Extrait partagé | Artefacts | crates/bridget-daemon/src/mcp.rs:448 | Publication durable superflue pour extrait borné ; envoi suffit |

## Duplications evidentes

Aucune. Les nouveaux noms ne remplacent pas les contrats existants.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Namespace isolé | Aucun service historique touché |
| .specify/memory/constitution.md | Pont constitution | Constitution globale lue |
| .specify/memory/standards.md | Références | Tests/research consultés |
| /Users/moi/.speckit/constitution.md | XVIII–XX | Bornes, réutilisation, aucun framework |
| /Users/moi/.speckit/ref/standards-tests.md | Métier + technique | Gherkin et tests Rust existants |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | Communication | Pas de nouveau suivi reply |
| 092–093 | Attach/rejeu | Extraits sur même assemblage |
| 094 | Parité CLI/MCP | Contrat commun |
| 097 | Claude natif | Conserver métadonnées outil à la source |
| 098 | T3 adaptateur | Couverture bornée aux faits remontés |
| 099 | Identité/remises | Préserver preuves et budgets |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg ObservationRequest,JournalExcerpt,file_collision | crates | Aucun contrat existant |
| rg subscriptions,collision,extrait | crates/specs | Attach et guichet ; collisions registre sans rapport |
| rg Subscribe,Activity,Journal,Replay | protocol/daemon/wrapper | Replay et flux live réutilisables |
| rg file_path,kind=file,Tool | producteurs | Métadonnées existantes partiellement tronquées |
| Lecture Cargo.toml | workspace/crates | serde, uuid, collections std suffisants |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| observation.rs | creer nouveau | État borné testable indépendant ; ni attach ni guichet ne porte cette règle métier | 2026-09-16 |
| Table/worker externe | ne pas créer | État mémoire daemon et sortie bornée existante | 2026-09-16 |
| Lecteur journal | reutiliser | Assemblage attach existant, pas d'accès fichier arbitraire | 2026-09-16 |
| File observation JournalLiveFeed | étendre | L'examen de AttachJournalRelay::start_with_clock montre que le flux attach ne se draine qu'avec des vues ; un second consommateur volerait ces fragments. File256 de petites métadonnées, même writer et même relais ; pas de second lecteur disque | 2026-09-16 |
| Identifiant notification | étendre BridgetMessage.id | Le journal porte déjà l'id déclencheur dans les trois pilotes gérés ; préfixe interne pour exclure les cascades, pas de nouveau modèle de workflow | 2026-09-16 |
| confirmed_write_payload/tool_write_path | créer fonctions métier journal | Recherche file_path/path/write_confirmed : seuls payloads tronqués ou non confirmés ; utilisés par Claude, ACP et Codex, sans dépendance externe | 2026-09-16 |
| Tests CLI isolés | créer spec100_observation_test.rs | Harnais support/idempotent.rs proche mais nettoyage SIGKILL de groupe incompatible avec les consignes ; socket simulée et vrai CLI, aucune logique daemon dupliquée | 2026-09-16 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
