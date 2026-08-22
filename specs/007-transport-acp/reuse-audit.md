# Reuse Audit : Transport ACP (007)

**Date** : 2026-08-22 · **Statut** : OK — aucun doublon bloquant, un arbitrage
tranché et appliqué au plan (TOML → JSON).

## Items proposés par le plan, confrontés à l'existant

| Item proposé | Équivalent existant | Preuve | Verdict |
|---|---|---|---|
| Remontée de la réponse d'un tour au daemon | `WrapperToDaemon::Send(BridgetMessage)` + champ `in_reply_to` + clôture déjà implémentée | `protocol.rs:36`, `message.rs:64`, `daemon.rs:1221` | **RÉUTILISER** — la réponse ACP est un `Send` avec `in_reply_to = id d'origine` ; la clôture des demandes suivies est du code existant, zéro sémantique nouvelle |
| Remontée de l'état de tour (occupé/inactif) | `WrapperToDaemon::Runtime` (modèle/effort) — objet différent | `protocol.rs:90` (RuntimeSource) | **CRÉER** une variante légère dédiée : détourner `Runtime` mélangerait deux notions ; une variante à deux états est plus petite que le contournement |
| Registre d'agents en TOML | aucun parseur TOML dans le workspace ; `serde_json` présent partout ; précédent de config simple : `federation.env` lu ligne à ligne | `Cargo.toml` (workspace deps), `wrapper.rs:70-83` | **ARBITRAGE TRANCHÉ** : registre en **JSON** (`agents.json`) via `serde_json` — TOML aurait exigé une crate nouvelle, en contradiction avec la ligne Article XIX du plan (« zéro nouvelle crate »). Plan et data-model corrigés en conséquence |
| Journal de session | `store.rs` (rusqlite) persiste les demandes | `crates/bridget-daemon/src/store.rs` | **CRÉER** en JSONL append-only : le store est dédié au cycle de vie des demandes (ADR 002) ; un journal d'événements de session est un flux, pas un état — JSONL est lisible par `tail`, sans migration de schéma, et la session 08 le consommera tel quel. Duplication légère assumée plutôt que d'élargir le rôle du store |
| Répertoire d'état pour les journaux | motif existant `~/.cache/bridget/{agent-names,agent-domains,agent-pids}` | `wrapper.rs:116-122,258-260` | **RÉUTILISER** le motif : `~/.cache/bridget/sessions/<agent>/` |
| Garde clé API au lancement | aucune garde d'environnement existante | recherche `rg -n "API_KEY" crates/` : 0 résultat | **CRÉER** (FR-011) |
| `AcpTransport` | `TmuxTransport` implémente le trait `Transport` | `tmux.rs:11`, `transport.rs:30` | **CRÉER** — c'est l'objet même de la feature ; le trait est réutilisé tel quel, extension minimale d'une méthode d'état (D-204) |
| Client JSON-RPC minimal | encode/decode `serde_json` ligne à ligne du protocole wrapper↔daemon | `protocol.rs:138-147` | **CRÉER** dans `acp.rs` en reprenant les mêmes primitives (style et helpers identiques) ; la crate officielle async est écartée (research R-004) |
| Sélection de type d'agent au lancement | liste blanche en dur + dispatch `cli.rs` | `wrapper.rs:522`, `cli.rs:62-77` | **REMPLACER** par le registre (FR-013) — la liste en dur est inscrite au registre des dépréciations puis supprimée |
| `docs/DEPRECATIONS.md`, ADR 003 | `docs/decisions/` contient 001 et 002 | `ls docs/decisions/` | **CRÉER** (aucun équivalent) |

## Arbitrages

| Arbitrage | Décision | Justification |
|---|---|---|
| TOML vs JSON pour le registre | JSON | zéro dépendance nouvelle (`serde_json` déjà en workspace) ; la perte des commentaires TOML est compensée par un registre par défaut embarqué documenté |
| JSONL vs rusqlite pour le journal | JSONL | flux append-only consultable sans outil, pas d'élargissement du rôle de `store.rs` ; duplication légère assumée (Article XIX §2) |
| Variante protocole dédiée pour l'état de tour | créer | plus petite que le détournement de `Runtime` ; deux notions distinctes |
| Variantes `CancelDelivery`/`DeliveryRejected` (D-209, round 2 contre-revue) | créer | l'annulation actuelle est un texte livré, impurgeable d'une file par id ; le daemon accuse réception au push, un refus synchrone de file pleine est impossible — l'échec doit remonter en asynchrone typé |

## Risques rapportés avant tasks

1. Le `Send` réutilisé passe par le routage normal du daemon (dedup, disjoncteur,
   budget de sauts) : une réponse d'équipier ne doit pas être dédupliquée à tort
   si deux demandes identiques se suivent — à couvrir par un test (le
   `content_key` inclut-il `in_reply_to` ? `message.rs:100` à vérifier en tâche).
2. stdio bloquant multi-threads : le client JSON-RPC lit les notifications et
   les réponses sur le même flux — un seul thread lecteur avec dispatch, motif
   déjà employé par le thread d'écoute du wrapper (`wrapper.rs:582`).
3. `npx` au lancement = dépendance réseau au premier run ; mitigation : le
   registre par défaut documente l'installation préalable (`npm i -g`) comme
   alternative hors-ligne.

## Gate avant tasks

- [x] Chaque service/composant/table/endpoint du plan a été confronté à
      l'existant avec preuve `fichier:ligne`
- [x] Aucune duplication évidente non arbitrée
- [x] Les arbitrages sont écrits et appliqués aux artefacts amont
- [x] Aucun statut NEEDS_ARBITRATION ou BLOCKED
