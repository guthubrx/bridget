# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/reuse-audit.md](../../../../../../specs/099-fiabilite-communications/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 089 | Autorité, suivi, socket privé | Préserver |
| 094 | Parité CLI/MCP | Tester les deux accès |
| 097 | Sessions natives | Ne pas casser les wrappers |
| 098 | Identité t3, file, journal | Corriger à l'intérieur du pont |

<a id="extrait-3"></a>

## Extrait 3

| Requete | Portee | Resultat |
|---|---|---|
| rg credential, token, attestation, peer_pid | daemon, wrapper, mcp_identity, protocole | Pas de preuve d'agent commune réutilisable |
| rg ClientHello, SendIdempotent, from_declared | CLI, MCP, daemon | Trois voies d'attribution à couvrir |
| rg push_control_message_until, track_reply_cycle | daemon | Bornes et suivi déjà disponibles |
| rg ThreadState, Pending, wait_idle, settle_pending | t3code | Étendre l'état et la boucle actuels |
| rg enqueue, failure, sync_channel | journal | File bornée et signal de panne existants |
| Cargo.toml, specs089/094/097/098 | dépôt | Dépendances et contrats réutilisables |

<a id="extrait-4"></a>

## Extrait 4

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
