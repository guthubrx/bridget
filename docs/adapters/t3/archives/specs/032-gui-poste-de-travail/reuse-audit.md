# Archives techniques de l’adaptateur T3

Document d’origine : [specs/032-gui-poste-de-travail/reuse-audit.md](../../../../../../specs/032-gui-poste-de-travail/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Item du plan | Recherche | Existant | Issue |
|---|---|---|---|---|
| 1 | Route d'envoi `POST /v1/send` | `"POST"`, `/v1/send` dans `ui.rs` | **0 occurrence** — `serve_connection` refuse tout sauf `GET` | **ÉTENDRE** `ui.rs` |
| 2 | Authentification de la route | `UiRelayConfig.token` | **jeton UUID existe** | **RÉUTILISER** — rien à concevoir |
| 3 | Trace inter-agents | `peer_exchange`, échanges agent↔agent | **aucune projection** — mais `LedgerEntry` porte déjà `id`, `ts`, `sender`, `target`, `body`, `delivery_phase` | **RÉUTILISER la donnée**, ajouter l'**agrégation** |
| 4 | Lecture du ledger depuis le relais | `ledger` dans `ui.rs` | **6 occurrences** — déjà lu | **RÉUTILISER** |
| 5 | Liste d'agents | `UiSnapshotV1.agents` | **déjà servi** (`AgentInfo`) | **ÉTENDRE** — ajouter `state`, `unread`, `last_excerpt` |
| 6 | Flux temps réel | `/v1/watch` | **livré**, abonnement avant snapshot | **RÉUTILISER** |
| 7 | Journal d'agent | `/v1/journal` | **livré** | **RÉUTILISER** |
| 8 | Raisonnement Codex | `reasoning` dans `codex_app_server.rs` | **0** — l'aiguillage ne reconnaît que `agentMessage/delta` | **ÉTENDRE** l'aiguillage |
| 9 | Raisonnement Cursor | `agent_thought_chunk` dans `acp.rs` | **0** — `acp.rs:1500` rejette tout sauf `agent_message_chunk` | **ÉTENDRE** le filtre |
| 10 | Assets front | `crates/bridget-daemon/assets` | **absent** ; front = **153 lignes en dur dans le Rust** | **CRÉER** — voir Arbitrages |
| 11 | Charte / thème | aucun CSS dans le dépôt | **aucun** | **CRÉER**, valeurs relevées sur T3 Code (MIT) |

<a id="extrait-2"></a>

## Extrait 2

**Relevés autorisés** sur `~/11.Repositories/t3code` — **licence MIT vérifiée** :
formes de protocole, valeurs de style, vocabulaire de nommage.
Reprise de code autorisée avec mention de copyright ; marginale en pratique
(TypeScript/React contre Rust/JS nu).
