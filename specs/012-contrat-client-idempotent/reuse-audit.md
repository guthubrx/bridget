# Reuse Audit : contrat client idempotent (012)

**Date** : 2026-08-22 · **Base auditée** : `session-08-attach@61f7072` (007
complète + 008 T801-T804a livrées). **Statut** : OK.

## Items du plan confrontés à l'existant

| Item | Équivalent existant | Preuve | Verdict |
|---|---|---|---|
| Rôle de connexion `client` | mécanique de rôles 008 : `ConnectionRole`, `RoleHandshake`/`RoleAccepted`, figement au premier message, matrice `allowed_for_attach` | `protocol.rs:17,54,200`, fix `5ff5514`/`bbd391a` | **ÉTENDRE** : variante `Client` dans l'enum existant + matrice dédiée — aucun second mécanisme |
| Actions hors verrou (écritures socket) | motif « cloner sous verrou, écrire après » établi par `bbd391a`, écrivains dédiés T804b | `daemon.rs:300-321,350-377` | **RÉUTILISER** le motif pour toutes les réponses client et remises |
| Tables durables + purge transactionnelle | `store.rs` : `request_events` + purge même-transaction (`d68ed09`), rusqlite | `store.rs:57-63,275` | **RÉUTILISER** le style ; nouvelles tables `idempotency_records` + `send_deliveries` avec FK et transaction unique (D-601) |
| Store d'état par instance côté wrapper | permissions/écriture atomique du journal (0700/0600, fsync) ; PAS de précédent `~/.local/state` | `journal.rs:41-60` | **CRÉER** `receipt_store.rs` (D-608) en réutilisant les helpers de permissions/atomicité ; répertoire d'état nouveau (précédent assumé) |
| Enveloppe/`content_key` | `BridgetMessage.content_key()` (dedup mémoire) | `message.rs:100` | **NE PAS RÉUTILISER** comme canon (non stable, champs partiels) — le canon D-605 est distinct, octets exacts ; `content_key` garde son rôle dedup existant |
| ID généré daemon | `BridgetMessage::new` (uuid v4) | `cli.rs`, `message.rs:73` | **CONSERVER** pour la voie historique ; `SendIdempotent` porte l'ID client (D-607, variante distincte) |
| Livraison vers wrapper | `Deliver(BridgetMessage)` + file ACP T705 | `protocol.rs:124` | **CRÉER** `DeliverIdempotent` distinct (champs D-602 round 5) ; la file/turn ACP est réutilisée telle quelle derrière le reçu `Seen` |
| Observable d'injection | `TurnStarted` existe mais AVANT la frame — disqualifié en review | `acp.rs:504-525` | **CRÉER** `PromptDispatched` (émission après write+flush de la frame — point D-602) |
| Négociation de version | aucune (protocole non versionné) | — | **CRÉER** `ClientHello`/`ClientWelcome` (D-604) |

## Arbitrages

| Arbitrage | Décision |
|---|---|
| brancher la 012 sur la branche 008 en cours | oui — worktree `012` depuis `session-08-attach` ; lots 1-2 sur fichiers NOUVEAUX (idempotency.rs, receipt_store.rs, tables) pour minimiser les conflits avec T804b/T805 en vol ; intégration daemon (lots 3+) après clôture T804b — je (bridget) séquence |
| `sha`/digest pour l'index du canon | préfiltre = longueur + premiers octets ; preuve = octets exacts (D-605, pas de crate) |

## Risques rapportés avant tasks

1. `daemon.rs` est en mouvement (T804b) : les lots 012 touchant `daemon.rs`
   attendent la clôture T804b — séquencement imposé par le référent.
2. La table `spawn_commands` (009) n'existe pas encore : l'amendement D-503
   se fera à la génération des tasks 009 (consommatrice du socle) — aucun
   travail 012 à faire pour ça.
3. Gate T006/011 (issuer_scope chez Maicie) : côté prospective, hors 012.

## Gate avant tasks

- [x] items confrontés avec preuves — [x] arbitrages écrits — [x] aucun
  doublon non arbitré — [x] pas de NEEDS_ARBITRATION/BLOCKED
