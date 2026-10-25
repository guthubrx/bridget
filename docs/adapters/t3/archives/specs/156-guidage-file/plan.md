# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/plan.md](../../../../../../specs/156-guidage-file/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. **Intention d'envoi** : `bridget_send` et `bridget send --intent` posent
   l'intention DEMANDÉE avant le canon. Le daemon ne la réécrit jamais et ne
   promet pas l'effet (R2, R4).
2. **Garde de droits unique sur tous les chemins** : `SendIdempotent`, `Send`
   historique, `ControlExecution`, CLI, pont T3. Le daemon normalise la provenance
   validée sans changer le rôle de la connexion.
   Anti-boucle, pause, mission (R3, R5-R7).
3. **Lecture de la file** : `bridget_queue` en lecture seule (R8).

<a id="extrait-2"></a>

## Extrait 2

| Sujet | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Chemin d'envoi | `SendIdempotent` + `intent` | `ControlExecution` exposé | exige génération et révision (R2) |
| Intention | demandée, canonisée avant le routage | effective, calculée selon la cible | l'état de la cible est invisible ou asynchrone (R4) |
| Résultat | admission + `effect: unconfirmed` | `steered` / `started` | aucune preuve synchrone possible |
| Valeurs | `queue`, `steer`, `interrupt` | `auto`, repli en interruption | pas de dégradation silencieuse (R9) |
| Droits | une garde pure appelée aux trois chemins | gardes par chemin | deux failles fermées (R3) |
| Origine | provenance validée séparée du rôle immuable ; UI/systèmes internes réutilisés | origine déclarée ou alias from humain font autorité | préserver producteurs sans interruption fabriquée (F2) |
| Parent | `agent_links` ou tâche native | même projet suffit | un pair ne doit pas interrompre un pair |
| Anti-boucle | table `guidance_events` indexée ; admission durable avant effet | compteur mémoire | transaction SendIdempotent existante ; audit préalable ailleurs |
| Reçu de rejeu | `idempotency_records` existant | `guidance_events` | horizon 7 jours, canon complet (R6) |
| Pause | `AgentGuidance` + exception interne | exception ad hoc | ADR 027 |
| Lecture de file | `send_deliveries` + `rowid` ; Attach QueueListV1 exact et UI Client attestée voient tout | colonne de rang, `execution_queue`, promotion de rôle | rôle immuable ; aucune identité agent/conn_names ni autre capacité Attach |
| Édition de file | reportée | trame `QueueEdit` | saga durable nécessaire (R8) |
| Pont T3 | Claude seul avec activeSteering natif prouvé, choix dans toute la file ; Cursor steer refusé/busy attend | repli T3 | T3 pourrait interrompre (R9) |
| Humain pour le contrôle | rôle attach ; client nu refusé | `from=human` suffit | `env -i bridget send` signe `human` (R3b) |
| Origine de `ControlExecution` | normalisée avant canon et garde | confiance au message | Codex interrompt sur `origin=Human` (R3c) |
| Guidage implicite T3 | mid_turn_ok sur DeliverIdempotent ET Deliver historique ; UI attestée et systèmes internes | fabriquer origin=Human | cette origine ferait interrompre les transports (F2) |
| `bridget_cancel` | `interrupt_active` + `cancel_queued` atomique | `cancel_delivery` unique | mélange retrait et interruption (R3e) |
| Module | `guidance.rs` (pur) | tout dans `daemon.rs` | table de vérité isolée |

<a id="extrait-3"></a>

## Extrait 3

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/guidance.rs` (nouveau) | `authorize`, faits, refus ; tests table de vérité |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module |
| `crates/bridget-transport/src/protocol.rs` | `ClientCapability::QueueListV1`, `QueueList`/`QueueListResult`, `ExecutionControlRefusal::NotPermitted`, champs optionnels `DeliverIdempotent.mid_turn_ok` et `Deliver.mid_turn_ok`, champ `CancelDelivery.interrupt_active` |
| `crates/bridget-transport/src/managed_session.rs`, `acp.rs` | `cancel_queued` (défaut faux ; ACP atomique sous verrou) |
| `crates/bridget-daemon/src/wrapper.rs` | traitement de interrupt_active et du champ optionnel de remise historique |
| `crates/bridget-transport/src/codex_app_server.rs` | repli sur échéance réservé au steer humain validé ; aucun nouvel injecteur (F1) |
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

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

| Risque | Parade |
|---|---|
| Neutraliser l'origine casse un producteur légitime (attach, relais UI) | T002(b) inventorie avant ; tests `guidance156_origin_*` ; conservée pour attach/UI attestée ; système interne conservé par le producteur ; client nu sans privilège |
| Oubli d'un chemin de la garde | tests sur `SendIdempotent`, `Send`, `ControlExecution`, guidage implicite, `bridget_cancel` ; trames brutes en défense en profondeur (T014, T017, T021 à T023) |
| Un agent se fait passer pour l'humain avec la CLI nue | client nu refusé pour le contrôle (T016) ; pseudo-terminal propre et trame brute : hors modèle, documentés |
| Régression de la session 129 (coordinateur qui retient des messages) | enfant vers parent reste immédiat (T021) ; question 3a |
| Ancien wrapper garde l'ancien `cancel_delivery` | noté dans la doc, valable jusqu'à relance (T022) |
| `steer` transformé en interruption par T3 | liste blanche de fournisseurs, refus sinon (T012) |
| Message de guidage retenu derrière une tête de file bloquée | choix dans toute la file (T012) |
| Interruption d'une mission native | `mission_active` pour un agent tant que l'interruption utilisateur est terminale, même après152 ; humain explicite exempt |
| Boucle d'interruption | cooldown 30 s, plafond 3 par 10 min, persistants ; humain et pause exemptés |
| Faux sentiment de preuve d'effet | `effect: unconfirmed` dans le contrat et la doc |
| Conflit d'édition avec 150/152/155/157 | exécution séquentielle, diff par session |
| Tests qui figent les schémas MCP | mise à jour explicite en T008 et T026 |
