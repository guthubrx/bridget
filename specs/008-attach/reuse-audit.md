# Reuse Audit : `bridget attach` (008)

**Date** : 2026-08-22 · **Base de code auditée** : branche
`session-07-transport-acp` au commit **`48c3bf7`** (MVP 007 validé : T701-T706,
corrigé après revue — l'audit initial référençait `60b0577` ; certaines lignes
de `journal.rs` citées ont pu glisser avec les correctifs, les preuves restent
valables par nom de symbole).
**Statut** : OK — aucun doublon bloquant, réutilisations massives confirmées
avec preuves, quatre créations justifiées.

## Items du plan confrontés à l'existant

| Item proposé | Équivalent existant | Preuve | Verdict |
|---|---|---|---|
| Lecture du journal (rejeu + suivi côté wrapper) | module `SessionJournal` livré en T706 : `append`/`append_at`, `valid_events()` (lecture tolérante aux lignes partielles/corrompues), récupération de `seq` | `journal.rs:10-50`, fixtures `tests/fixtures/journal/*` | **RÉUTILISER** — le lecteur du relais s'appuie sur `valid_events` et les fixtures existantes ; ne pas réécrire un parseur |
| Variantes d'annulation/refus typées | `CancelDelivery { id, reason }` (daemon→wrapper), `DeliveryRejected { id, reason }` (wrapper→daemon) livrées en T705 | `protocol.rs:126`, `protocol.rs:38` | **RÉUTILISER** le style et le canal ; les nouvelles variantes d'abonnement (Subscribe/Subscribed/fragment/SnapshotCaughtUp/Gap/End) suivent le même motif dans le même fichier |
| Modèle lecteur unique + writer sérialisé du client attach | motif D-204 implémenté dans le client ACP (lecteur propriétaire, waiters, writer Mutex) | `acp.rs:56-93` | **RÉUTILISER le motif** (pas le code : rôles différents, machines d'état non couplées — même règle que D-405 de la 010) |
| Worker de relais wrapper à canal borné | worker de tours ACP (canal borné, thread dédié) | `acp.rs` (worker T704) | **RÉUTILISER le motif** ; le relais journal est un second worker du wrapper, distinct du worker de tours |
| Raw mode termios | `libc` déjà dépendance des deux crates ; aucun code termios existant | `Cargo.toml` (libc) ; `rg termios` : 0 résultat | **CRÉER** (D-307) — première utilisation, garde RAII locale à `attach.rs`, pas d'abstraction |
| Sanitisation Unicode liste blanche | aucun équivalent | `rg "sanitiz|escape"` : 0 résultat pertinent | **CRÉER** (D-304) — fonction pure + fixtures hostiles, dans `attach.rs` |
| Connexion attach persistante du client | connexions CLI existantes (`cli-send-*` éphémères, `send_control_to_daemon`) | `cli.rs:454` | **ÉTENDRE** : le client attach garde la connexion ouverte (le daemon gère déjà des connexions longues : les wrappers) ; l'envoi humain D-308 passe par cette connexion avec `Send` existant (`protocol.rs` WrapperToDaemon::Send) — pas de nouveau type de message d'envoi |
| Fan-out daemon vers vues | aucun mécanisme de diffusion existant (le daemon route point à point) | lecture `daemon.rs` | **CRÉER** la table d'abonnements + écrivain par vue (D-305) — c'est le cœur nouveau de la session |
| Sous-commande `attach` | dispatch CLI par registre (T703) | `cli.rs`, `registry.rs` | **RÉUTILISER** le dispatch ; `attach` est une sous-commande client, pas un type d'agent |

## Arbitrages

| Arbitrage | Décision | Justification |
|---|---|---|
| réécrire un lecteur JSONL vs `valid_events()` | réutiliser `valid_events` en l'étendant (il retourne les événements valides ; le relais a besoin en plus des positions/offsets et du signalement des lignes invalides avec numéro/offset) — extension du module T706, pas un second parseur | un seul endroit qui comprend le format v1 ; les fixtures restent le contrat unique |
| motif lecteur/writer : partager du code avec `acp.rs` ? | non — motif répliqué, code séparé | machines d'état différentes (client ACP vs client daemon) ; même décision que D-405 (extraction seulement sur preuve de duplication identique, à réévaluer après implémentation) |
| nouvelles variantes de protocole | créer 6 variantes d'abonnement dans `protocol.rs` | style D-209 existant ; aucun message actuel ne porte la sémantique d'abonnement |

## Risques rapportés avant tasks

1. `valid_events()` lit tout le fichier en mémoire (`Vec<Value>`) — acceptable
   pour la récupération de `seq`, pas pour le rejeu d'un gros journal : son
   extension pour le relais doit lire **par tranches** (exigence D-302) sans
   casser son usage T706.
2. Le daemon n'a aujourd'hui aucune connexion « client longue durée » autre que
   les wrappers : la connexion attach persistante emprunte le chemin `Register`
   — vérifier qu'un client attach n'apparaît pas comme un agent dans l'annuaire
   (type dédié ou enregistrement séparé, à trancher en tasks).
3. L'envoi via la connexion attach réutilise `Send`, mais le refus
   `reply_requires_agent` (`daemon.rs:1214`) vise les clients éphémères — la
   connexion attach persistante doit être reconnue comme apte au `reply` différé
   ou l'envoi limité à `reply=false` (à trancher en tasks, avec cxbridget).

## Gate avant tasks

- [x] chaque item confronté avec preuve `fichier:ligne`
- [x] aucune duplication évidente non arbitrée
- [x] arbitrages écrits
- [x] aucun statut NEEDS_ARBITRATION/BLOCKED
