# Audit de reutilisation de l'existant — 099

## Decision

Statut: PASS
Date: 2026-09-16
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/099-fiabilite-communications

Audit de recherche réalisé par l'explorateur puis consolidé contre les sources
déjà lues par le principal. Aucun service, stockage ou primitive redondant proposé.
Les ajouts du contrat d'identité sont nécessaires car les mécanismes actuels
ne prouvent que l'identité déclarée, pas son autorisation.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 9 |
| Items audites | 9 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Écriture bornée | push_control_message_until | crates/bridget-daemon/src/daemon.rs:1553 | Étendre l'usage, aucun second writer |
| Suivi avant remise | track_reply_cycle / tracked_requests | crates/bridget-daemon/src/daemon.rs:7359 ; src/store/ledger_requests.rs:288 | Déplacer la frontière, conserver les transactions existantes |
| Identité locale | mcp_identity | crates/bridget-daemon/src/mcp_identity.rs:115 | Retrouver la preuve côté wrapper, y compris SSH |
| Client auxiliaire | registered_connection | crates/bridget-daemon/src/communication/client.rs:749 | Remplacer l'inscription déclarative |
| Contrôle t3 | LinkWorker, LinkEvent, CancelDelivery | crates/bridget-daemon/src/t3code.rs:1199 ; daemon.rs:12111 | Étendre la file et la notification existantes |
| Réponse durable | ThreadState/Pending, Ack/Nack, RequestList | crates/bridget-daemon/src/t3code.rs:182 ; daemon.rs:12157 | Enrichir, pas de table/outbox parallèle |
| Persistance/journal | write_private_file_atomic, JournalWriter | crates/bridget-transport/src/fsutil.rs:49 ; crates/bridget-transport/src/journal.rs:447 | Ne pas confondre admission en file et écriture disque |
| Preuve d'autorisation | Aucun équivalent actuel | daemon.rs:5622 ; daemon.rs:7407 | CREER un jeton lié à l'incarnation dans les maps existantes |
| Attestation protocolaire | Aucun équivalent dans ClientHello/Register | crates/bridget-transport/src/protocol.rs:2100 | CREER une seule trame et un type à Debug masqué |

## Existant potentiellement pertinent non mentionne

Aucun. Le signal d'échec du JournalWriter doit être utilisé pour rendre visible
une panne après admission en file : enqueue seul n'atteste pas la durabilité.

## Duplications evidentes

Aucune.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | Français, chemins absolus, isolation | Aucune action sur daemon réel |
| .specify/memory/constitution.md | Constitution globale | Synchronisation effectuée |
| .specify/memory/standards.md | Références à la demande | Tests et sécurité consultés |
| ~/.speckit/constitution.md XIX/XX | Minimalisme et responsabilité | Pas de nouveau broker/framework |
| ~/.speckit/ref/standards-tests.md | Acceptations lisibles, tests traçables | Feature 099 et tests Rust existants |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | Autorité, suivi, socket privé | Préserver |
| 094 | Parité CLI/MCP | Tester les deux accès |
| 097 | Sessions natives | Ne pas casser les wrappers |
| 098 | Identité t3, file, journal | Corriger à l'intérieur du pont |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg credential, token, attestation, peer_pid | daemon, wrapper, mcp_identity, protocole | Pas de preuve d'agent commune réutilisable |
| rg ClientHello, SendIdempotent, from_declared | CLI, MCP, daemon | Trois voies d'attribution à couvrir |
| rg push_control_message_until, track_reply_cycle | daemon | Bornes et suivi déjà disponibles |
| rg ThreadState, Pending, wait_idle, settle_pending | t3code | Étendre l'état et la boucle actuels |
| rg enqueue, failure, sync_channel | journal | File bornée et signal de panne existants |
| Cargo.toml, specs089/094/097/098 | dépôt | Dépendances et contrats réutilisables |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Jeton et trame | créer | Contrainte de sécurité, trois voies réelles | 2026-09-16 |
| Réponses en attente | réutiliser | ThreadState existe déjà et survit aux reprises | 2026-09-16 |
| Journal | réutiliser | Bornes/rotation/signal de panne existants | 2026-09-16 |
| Tests | réutiliser | Fixtures socket/HTTP déjà présentes ; pas de framework | 2026-09-16 |
| Lecture et confirmation journal | réutiliser | IncrementalJournalReader déjà présent dans crates/bridget-transport/src/journal.rs ; lecteurs conservés par fichier, pas de second format ni lecteur ad hoc | 2026-09-16 |
| Frontière d'autorisation commune | créer | sender_is_authorized et authenticate_auxiliary portent la même règle de sécurité pour Send, SendIdempotent, MCP et CLI ; pas une couche sans responsabilité | 2026-09-16 |
| Fixtures d'identité | étendre | Preuves réellement émises par Registered pour tout vrai daemon ; preuve synthétique limitée aux mocks de protocole privés, jamais de bypass de production | 2026-09-16 |
| Nettoyage des tables t3 | réutiliser | HashSet déjà importé ; remplacer deux recherches linéaires imbriquées par ensembles d'identifiants locaux, sans helper générique | 2026-09-16 |
| Rappels t3 | étendre | collect_reminder_actions et CancelDelivery existants ; ne pas créer de tour textuel pour un pont qui corrèle automatiquement les réponses | 2026-09-16 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
