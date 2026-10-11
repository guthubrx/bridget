# Plan - Session156 guidage, interruption et lecture de file

**Spec** : `specs/156-guidage-file/spec.md`
**Branche** : `session-156-guidage-file`
**Worktree** : `.worktrees/150-socle-delegation` (exécution séquentielle 150-158)
**Date** : 2026-10-11 (révisé après contre-revue Codex : ronde 1 BLOCKED, 11 points ; ronde 2, 4 points)

## Résumé

Trois apports, sans nouveau mécanisme d'injection :

1. **Intention d'envoi** : `bridget_send` et `bridget send --intent` posent
   l'intention DEMANDÉE avant le canon. Le daemon ne la réécrit jamais et ne
   promet pas l'effet (R2, R4).
2. **Garde de droits unique sur tous les chemins** : `SendIdempotent`, `Send`
   historique, `ControlExecution`, CLI, pont T3. Le daemon fabrique l'origine.
   Anti-boucle, pause, mission (R3, R5-R7).
3. **Lecture de la file** : `bridget_queue` en lecture seule (R8).

**Hors périmètre, décidé après contre-revue** : `cancel`, `reorder`, `promote`
(saga durable, R8). Effet sur la note : Guidage vise 4, pas 5.

**Cadre de menace** (ADR 011, spec « Cadre de menace ») : la garde protège contre
un agent qui utilise ses outils Bridget. Un processus hostile du même compte
Unix qui forge des trames brutes reste hors modèle ; on ne le promet pas.

## Contexte technique

- Rust 2024, workspace à 3 crates. Aucune nouvelle dépendance.
- SQLite (`bridget.db`), même base que l'idempotence.
- Tests : recette `umask 077` de la session150 (`quickstart.md`).

## Décisions architecturales

| Sujet | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Chemin d'envoi | `SendIdempotent` + `intent` | `ControlExecution` exposé | exige génération et révision (R2) |
| Intention | demandée, canonisée avant le routage | effective, calculée selon la cible | l'état de la cible est invisible ou asynchrone (R4) |
| Résultat | admission + `effect: unconfirmed` | `steered` / `started` | aucune preuve synchrone possible |
| Valeurs | `queue`, `steer`, `interrupt` | `auto`, repli en interruption | pas de dégradation silencieuse (R9) |
| Droits | une garde pure appelée aux trois chemins | gardes par chemin | deux failles fermées (R3) |
| Origine | fabriquée par le daemon | confiance au client | les transports interrompent sur `origin=Human` (R3) |
| Parent | `agent_links` ou tâche native | même projet suffit | un pair ne doit pas interrompre un pair |
| Anti-boucle | table `guidance_events` indexée | compteur mémoire | survit au redémarrage ; audit |
| Reçu de rejeu | `idempotency_records` existant | `guidance_events` | horizon 7 jours, canon complet (R6) |
| Pause | `AgentGuidance` + exception interne | exception ad hoc | ADR 027 |
| Lecture de file | `send_deliveries` + `rowid` | colonne de rang, `execution_queue` | rien à migrer (R8) |
| Édition de file | reportée | trame `QueueEdit` | saga durable nécessaire (R8) |
| Pont T3 | liste blanche de fournisseurs, choix dans toute la file | repli T3 | T3 pourrait interrompre (R9) |
| Humain pour le contrôle | rôle attach ; client nu refusé | `from=human` suffit | `env -i bridget send` signe `human` (R3b) |
| Origine de `ControlExecution` | normalisée avant canon et garde | confiance au message | Codex interrompt sur `origin=Human` (R3c) |
| Guidage implicite T3 | `mid_turn_ok` calculé par le daemon | décision dans le pont | le pont ignore la parenté (R3d) |
| `bridget_cancel` | `interrupt_active` + `cancel_queued` atomique | `cancel_delivery` unique | mélange retrait et interruption (R3e) |
| Module | `guidance.rs` (pur) | tout dans `daemon.rs` | table de vérité isolée |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/guidance.rs` (nouveau) | `authorize`, faits, refus ; tests table de vérité |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module |
| `crates/bridget-transport/src/protocol.rs` | `ClientCapability::QueueListV1`, `QueueList`/`QueueListResult`, `ExecutionControlRefusal::NotPermitted`, champ optionnel `DeliverIdempotent.mid_turn_ok`, champ `CancelDelivery.interrupt_active` |
| `crates/bridget-transport/src/managed_session.rs`, `acp.rs` | `cancel_queued` (défaut faux ; ACP atomique sous verrou) |
| `crates/bridget-daemon/src/wrapper.rs` | traitement de `interrupt_active` |
| `crates/bridget-daemon/src/mcp.rs` | argument `intent`, résultat, outil `bridget_queue`, capacités |
| `crates/bridget-daemon/src/cli.rs` | `--intent`, `bridget queue` |
| `crates/bridget-daemon/src/daemon.rs` | garde et neutralisation d'origine sur `Send` et `SendIdempotent`, garde dans `handle_execution_control`, `QueueList`, matrice exhaustive, purge des traces |
| `crates/bridget-daemon/src/referent_control.rs` | `AutonomousEffect::AgentGuidance` |
| `crates/bridget-daemon/src/idempotency.rs`, `idempotency/send_delivery.rs` | table `guidance_events`, index de lecture, `queued_deliveries_for_recipient` |
| `crates/bridget-daemon/src/t3code.rs`, `t3code_contract.rs`, `t3code_contract_v2.rs` | `deliveryIntent` généralisé, liste blanche, choix dans toute la file, hors lot |
| `crates/bridget-daemon/tests/guidance156_e2e.rs` (nouveau) | bout en bout |
| tests des mêmes modules | tests `guidance156_*` |
| `docs/reference-communication.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | intention, refus, outil |
| `docs/decisions/156-guidage-et-lecture-de-file.md` | ADR |

Aucun changement du schéma de `send_deliveries` ni du `CHECK` de phase. Les transports Codex et Claude ne changent pas.

## Étapes d'implémentation

1. Référence de non-régression (T001) et six vérifications (T002 a à f).
2. Fondations en parallèle : protocole (T003), pause (T004), garde pure (T005), stockage (T006).
3. US1 : intention (T007 à T013).
4. US2 : garde sur tous les chemins atteignables par les outils (envoi, contrôle, guidage implicite, annulation), autorité humaine, origine, anti-boucle, pause, mission, tests combinés (T014 à T023).
5. US3 : lecture de file (T024 à T026).
6. Documentation, non-régression, suite complète, relecture (T027 à T031).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 156 prévue (T028) |
| VIII Forensic | OK | failles prouvées par lecture ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; tâche cochée après test vert |
| XVI Worktree | OK | worktree partagé 150-158 |
| XVIII Complexité | OK | R12 |
| XIX Minimalisme | OK | périmètre réduit après contre-revue |
| XX Vertus LLM | OK | chaque décision a sa preuve `fichier:ligne` ; contre-revue archivée |

### Minimalisme (Article XIX)

- Les intentions, transports et le pont existent. On expose et on protège.
- Un outil en lecture seule. Ni `cancel`, ni `reorder`, ni `promote`, ni `auto`.
- Un module pur, une table d'audit, une capacité, une paire de trames client.
- Rien dans les wrappers, les transports ni le schéma de `send_deliveries`.

Potentiel minimalisme : environ 10 lignes (index de lecture, justifié par l'absence d'index existant).

### Responsabilité future (Article XX)

- Non vérifiable sans exécution : le comportement réel de l'émetteur quand un transport rejette un `steer` (T002(a)).
- Prérequis de la session d'édition de file notés en R8 et dans l'ADR.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | `intent` rejeté comme argument inconnu ; trame brute `InterruptAndStart` d'un agent admise ; `origin=Human` forgée admise ; `Interrupt` de `ControlExecution` sans droits admis |
| Table de vérité | `guidance156_matrix_*`, `guidance156_pause_*`, `guidance156_rate_*` |
| Chemins | `guidance156_guard_*`, `guidance156_origin_*`, `guidance156_human_*`, `guidance156_control_*`, `guidance156_midturn_*`, `guidance156_cancel_*`, `guidance156_combo_*` |
| Pont T3 | `guidance156_t3_*` (tête bloquée, liste blanche, hors lot, session 129 inchangée) |
| Lecture | `guidance156_queue_*` (ordre, droits, base inchangée) |
| Bout en bout | `guidance156_e2e.rs` (Codex et Claude simulés) |
| Non-régression | `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::`, transport |
| Suite complète | recette `--workspace --no-fail-fast` |

## Risques

| Risque | Parade |
|---|---|
| Neutraliser l'origine casse un producteur légitime (attach, relais UI) | T002(b) inventorie avant ; tests `guidance156_origin_*` ; conservée pour attach et client nu |
| Oubli d'un chemin de la garde | tests sur `SendIdempotent`, `Send`, `ControlExecution`, guidage implicite, `bridget_cancel` ; trames brutes en défense en profondeur (T014, T017, T021 à T023) |
| Un agent se fait passer pour l'humain avec la CLI nue | client nu refusé pour le contrôle (T016) ; pseudo-terminal propre et trame brute : hors modèle, documentés |
| Régression de la session 129 (coordinateur qui retient des messages) | enfant vers parent reste immédiat (T021) ; question 3a |
| Ancien wrapper garde l'ancien `cancel_delivery` | noté dans la doc, valable jusqu'à relance (T022) |
| `steer` transformé en interruption par T3 | liste blanche de fournisseurs, refus sinon (T012) |
| Message de guidage retenu derrière une tête de file bloquée | choix dans toute la file (T012) |
| Interruption d'une mission native | `mission_active` jusqu'à 152 |
| Boucle d'interruption | cooldown 30 s, plafond 3 par 10 min, persistants ; humain et pause exemptés |
| Faux sentiment de preuve d'effet | `effect: unconfirmed` dans le contrat et la doc |
| Conflit d'édition avec 150/152/155/157 | exécution séquentielle, diff par session |
| Tests qui figent les schémas MCP | mise à jour explicite en T008 et T026 |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diff enregistré dans `validation/diff-156.patch`.
