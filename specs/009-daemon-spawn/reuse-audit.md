# Reuse Audit : équipiers gérés par le daemon (009)

**Date** : 2026-08-22 · **Base auditée** : `session-08-attach@e028311`
(007 complète, 008 T801-T805a/T804b closes). **Correction factuelle
(review)** : les restes 008 touchent `attach.rs`/`cli.rs` **et** — pour les
bancs T808 et la finition T810 — `acp.rs`/`journal.rs` (instrumentation
test-only) et les docs ; chevauchements réels énumérés, tous disjoints des
modules nouveaux 009 (`fleet.rs`, `managed_process.rs`, `desired_state.rs`).
**Statut** : OK.
**Amendement D-503 appliqué au plan avant cet audit** (socle 012).

## Items du plan confrontés à l'existant

| Item | Équivalent existant | Preuve | Verdict |
|---|---|---|---|
| Chemin de lancement injectable | `launch_acp_with(registry, socket, home)` (007-T708) | `wrapper.rs:1548` | **RÉUTILISER** tel quel — c'est le point d'entrée que `managed-bootstrap` exec |
| Idempotence des ordres (`command_id`) | **socle 012** `idempotency.rs` (T1202 close : réservation atomique, expires_at figé, rejeu) | `idempotency.rs`, `lib.rs:3` | **CONSOMMER** (amendement D-503) : `operation_kind="spawn"`, `spawn_commands` = saga seule, transaction unique FK |
| Store durable + purge transactionnelle | `store.rs` (motif `request_events`/`d68ed09`) | `store.rs:275` | **RÉUTILISER** le style pour `spawn_commands` |
| État désiré durable (fichier) | `fsutil` public (extraction T1203 : écriture atomique+fsync, permissions) | `crates/bridget-transport/src/fsutil.rs` | **RÉUTILISER** pour `desired_state.rs` (fleet.json temp+fsync+rename+fsync dir) |
| Marqueurs anti-pid-recyclé | motif `agent-pids` typé conçu en 010/FR-004 (naissance+instance) — non implémenté ; marqueurs `managed/` de la 009 sont les premiers | plan 009 D-502 | **CRÉER** dans `managed_process.rs` (le motif servira ensuite à la 010) |
| Canal de statut structuré | canal de contrôle/actions hors verrou (motifs `bbd391a`, T804b writer→daemon signal) | `daemon.rs` | **RÉUTILISER le motif** (événements typés consommés hors verrou) pour `BootstrapReady`/`StartupFailed` |
| Arrêt de groupe | handshake d'arrêt wrapper (chemin d'arrêt complet 007 : annulation, drain, Unregister) | `wrapper.rs` (shutdown T704 fixes) | **RÉUTILISER** : l'ordre d'arrêt ciblé déclenche le chemin existant ; seuls `killpg`+polling sont nouveaux |
| Sous-mode exécutable | dispatch de sous-commandes `cli.rs` | `cli.rs` | **ÉTENDRE** : `managed-bootstrap` (caché) |
| État `Recovering` visible | états d'annuaire 007-T709 (`busy`/`stopped`…) | `daemon.rs:318-331` | **ÉTENDRE** le mécanisme de présence/statut |
| Abonnements attach au stop/reprise | `End` typé + générations (008 T804a/b closes) | `daemon.rs` | **RÉUTILISER** : FR-011ter appelle le chemin de fermeture existant |

## Arbitrages

| Arbitrage | Décision |
|---|---|
| attendre la 008 finale vs démarrer sur `e028311` | démarrer : les tâches 008 restantes (T805b/T806/T807/T808 = `attach.rs`, tests, bancs) sont disjointes des ancrages 009 ; gate de resynchronisation avant clôture (comme 008-T810) |
| `spawn_commands` propre vs socle 012 | socle (amendement D-503) — la phase 1 de la 009 dépend donc de la clôture des tâches socle 012 (T1202 close ✅) |

## Risques rapportés avant tasks

1. `daemon.rs` recevra encore des STOP éventuels des reviews 008 en cours —
   les tâches 009 touchant `daemon.rs` (ordres, Recovering) sont gatées par le
   référent, comme pour la 012.
2. Le worktree 012 modifie `store.rs`/`lib.rs` (socle) : la branche 009
   partira de la **012** (session-12) pour consommer le socle sans merge
   intermédiaire — ordre de fondation voulu par l'utilisateur.
3. `killpg`/process groups : premiers usages dans le workspace — tests de
   crash réels exigés (pas de simulation), déjà au plan.

## Gate avant tasks

- [x] items confrontés avec preuves — [x] arbitrages écrits — [x] pas de
  doublon non arbitré — [x] pas de NEEDS_ARBITRATION/BLOCKED
