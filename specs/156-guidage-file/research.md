# Recherche - Session156 guidage, interruption et lecture de file

Date : 2026-10-11, révisée après contre-revue (`adversarial-review-codex.md`).
Source : lecture du checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget`
(base `1f2c661d`, non modifié) et de T3 `origin/main`. La session150 est supposée livrée.

## R0 - Cadre de menace (ADR 011, ADR 027)

- ADR 011 : la garantie tient « par impossibilité » face à un agent qui exécute des commandes mais ne peut pas appuyer sur les touches d'un terminal ouvert de l'utilisateur. ADR 027 : la reconnaissance du principal humain « arrête un agent qui agit par ses outils déclarés, pas un processus hostile du même compte ».
- **156 protège contre un agent qui utilise ses outils Bridget** (MCP, CLI, pont T3, wrapper).
- **Hors modèle** (limite acceptée, non promise) : un processus du même compte Unix qui ouvre une socket brute et forge des trames. Preuves : rôle déclaratif sans preuve (`daemon.rs:14213`), `human_from_bare_client` (`daemon.rs:8604-8606`), pseudo-terminal propre pour `bridget attach` (`attach.rs:267`, `:405-415`).
- **Dans le modèle, corrigé** : (1) CLI sans preuve d'identité qui signe `human` (`cli.rs:3479`, `:4322`) ; (2) message de `ControlExecution` à l'origine non normalisée ; (3) guidage implicite du pont T3 ; (4) `bridget_cancel` sur une demande active.
- Les tests de trames brutes (T014, T023) sont de la défense en profondeur : le daemon est le seul point d'application, il applique la garde quel que soit le client.

## R1 - Le protocole et les transports savent déjà faire

- `MessageIntent` : `QueueOnly`, `TriggerTurn`, `SteerCurrent`, `InterruptAndStart`, `ControlOnly` (`bridget-core/src/message.rs:54-60`).
- `ExecutionControlOperation` : seules `SteerCurrent` et `Interrupt` sont implémentées (`protocol.rs:219-227`, `daemon.rs:9025-9035`, `wrapper.rs:5603-5640`).
- Codex : `deliver` accepte `SteerCurrent` si un tour est actif et `steering_open` (`codex_app_server.rs:865-899`), interrompt pour `InterruptAndStart` (`:901-925`). Un `turn/steer` refusé remet le message en file (`:1700-1760`).
- Claude : `SteerCurrent` refusé (`claude_stream_json.rs:310-317`), `InterruptAndStart` accepté (`:329`). ACP : idem (`acp.rs:620-628`, `:630`).
- **Décision** : exposer, ne rien réinventer. L'intention demandée devient `message.intent`.

## R2 - Chemin d'envoi : `SendIdempotent`

- `ControlExecution` exige `generation` et `revision` (`daemon.rs:9161-9165`), pas pratique pour un agent.
- `SendIdempotent` porte clé, rejeu, garde de projet et admission d'exécution pour `TriggerTurn`/`InterruptAndStart` (`daemon.rs:8828-8850`). Le canon réserve l'intention AVANT le routage (`daemon.rs:8654`).
- **Décision** : l'intention passe par `bridget_send`. Le canon contient l'intention demandée. Le daemon ne la réécrit jamais.

## R3 - Failles existantes (contre-revue 1 et 2, vérifiées)

- `Send` historique et `SendIdempotent` ne retirent ni `intent` ni `origin` du client (`daemon.rs:17521-17560`, `:8584-8640`, seul `thread_notice` est neutralisé).
- Les transports interrompent tout message `origin=Human` si un tour est actif (`codex_app_server.rs:865-866`, `claude_stream_json.rs:329`, `acp.rs:630`).
- `Interrupt` de `ControlExecution` n'a pas de message, donc pas de contrôle d'identité (`daemon.rs:9002-9035`). Capacité `ExecutionControlV1` négociable par tout client (`daemon.rs:16174`). Génération et révision doivent être connues (`:9161-9165`), à la portée d'un client qui observe l'exécution.
- Aucun producteur de `origin=Human` hors tests dans `crates/` sauf le dépôt de service (`daemon.rs:15834`, ledger seul). Les producteurs réels (connexion attach, relais UI) sont hors des crates ; T002(b) les inventorie.
- **Décision** : une seule garde, trois chemins, plus la fabrication de l'origine par le daemon. Exception interne explicite pour l'interruption de pause (`handle_execution_control(None, …)`, `daemon.rs:9667`).

## R3b - Autorité humaine (ronde 2, point 1)

- `RoleHandshake` accepte tout rôle sans preuve (`daemon.rs:14213`). Le chemin attach pose `from="humain"` (`daemon.rs:17541`). Le client nu `from=human` est « humain » pour `SendIdempotent` (`:8604-8606`).
- La CLI signe `human` quand aucune preuve (fichier, variable, filiation) n'est trouvée (`cli.rs:3479`) et envoie alors comme client nu (`cli.rs:4322`). Un agent qui vide son environnement, ou rompt sa filiation, y arrive par l'outil. **Dans le modèle.**
- **Décision** : pour le contrôle (`steer`, `interrupt`, guidage implicite), l'humain attesté = connexion de rôle attach. Un client nu garde l'envoi en file. Une connexion qui a une identité d'agent (`conn_names`) ne devient pas humaine : test de reconnexion. Le pseudo-terminal propre reste hors modèle (R0). T002(b) confirme si un `RoleHandshake{Attach}` est accepté sur une connexion qui a déjà une identité d'agent.
- Conséquence : `bridget send --intent` depuis un terminal nu est refusé pour le contrôle (question 3b). Alternative rejetée : exiger un terminal côté CLI (contrôle côté client, donc contournable).

## R3c - Origine du message de `ControlExecution` (ronde 2, point 2)

- `handle_execution_control` encode la commande telle quelle (`daemon.rs:9060-9075`) puis `wrapper.rs:5629-5640` appelle `transport.deliver(message)`. Codex interrompt si `origin=Human` et tour actif (`codex_app_server.rs:866`, `:915`). Un parent autorisé à guider obtient donc guidage ET interruption, hors `mission_active` et limites.
- **Décision** : normaliser `origin` du message de commande avant le canon et avant la garde ; `Human` toujours retirée (le rôle Client n'est pas attach, `daemon.rs:14195-14213`).

## R3d - Guidage implicite du pont T3 (ronde 2, point 3)

- `deliverable_while_busy(provider, groupable_front)` (`t3code.rs:3126`) donne `steer_current = true` à `dispatch` (`t3code.rs:2206`) pour tout message groupable (sans réponse attendue) vers Claude ou Cursor occupé, sans regarder l'expéditeur. Un pair injecte ainsi du texte dans un tour actif.
- **Décision** : l'expéditeur doit pouvoir guider. Le daemon calcule `mid_turn_ok` (humain attaché, parent de la cible, enfant de la cible) aux points de construction de `DeliverIdempotent` (envoi et reprise) et le pose sur la trame ; le pont l'exige. `Deliver` historique : seul `origin=Human` fabriquée passe. Enfant vers parent reste permis : c'est le cas de la session 129 (coordinateur qui retenait 13 messages 8 h 45). Les pairs attendent la fin du tour. Changement visible : question 3a.
- Alternatives rejetées : laisser tel quel (guidage sans garde, sans pause ni rythme) ; décision dans le pont (il ne connaît pas la parenté).

## R3e - `bridget_cancel` et interruption active (ronde 2, point 4)

- `CancelRequest` vérifie l'émetteur seulement (`daemon.rs:18128-18150`), puis envoie `CancelDelivery` aux présences ACP ou `t3code` (`:18155-18175`). `AcpTransport::cancel_delivery` annule le tour actif si l'identifiant correspond (`acp.rs:518-531`). Le pont T3 ne fait que retirer de sa file (`t3code.rs:1684-1692`).
- La description MCP de l'outil dit déjà : « n'arrête pas l'agent » (`mcp.rs` description de `bridget_cancel`). Le comportement ACP la contredit.
- **Décision** : `CancelDelivery` gagne `interrupt_active: bool` (défaut faux). `ManagedSession::cancel_queued(id, reason) -> bool` (défaut faux) retire sans toucher au tour actif ; ACP l'implémente sous le même verrou. Le daemon met `interrupt_active = true` seulement si l'appelant passe la garde commune (droits, pause, rythme, mission), et compte l'interruption. Un ancien wrapper garde l'ancien comportement jusqu'à sa relance (noté dans la doc).
- Hors périmètre : Codex et Claude gérés reçoivent un texte, pas `CancelDelivery`.

## R4 - Intention demandée, pas d'intention effective

- Un état d'exécution n'existe que pour les envois `TriggerTurn`/`InterruptAndStart` (`execution_store.rs:527-531` projette les transitions) : un tour démarré par un envoi ordinaire est invisible du daemon.
- Le support de guidage et l'issue du `turn/steer` sont connus du transport seul, et de façon asynchrone.
- **Décision** : pas de table de décision à l'envoi, pas de `steered`/`started`/`interrupted_restarted`. Le résultat dit « admis, effet non confirmé ». L'effet est celui du transport (accusé de remise, journal). Alternative rejetée : prédire l'effet dans le daemon (fausse promesse). Les cas « tour changé » restent gérés par le transport (`expectedTurnId` dans `turn/steer`).

## R5 - Matrice de droits

Fonction pure `authorize(facts)`, testable par table (patron de `admit_autonomous_effect`, `referent_control.rs:122-133`).

| Action | Humain attesté | Parent de la cible | Destinataire (soi) | Pair même projet | Autre projet |
|---|:-:|:-:|:-:|:-:|:-:|
| envoi en file | oui | oui | - | oui | règle interprojet existante |
| `steer` / `interrupt` | oui | oui | non | non | non |
| `bridget_queue` | toute la file | toute la file | toute la file | ses messages | ses messages |

- Parent : `agent_link_for_child(child_instance_id)` puis `parent_instance_id` (`agent_links.rs:79-94`), ou tâche native dont l'appelant est propriétaire et la cible `child`.
- Humain attesté : connexion attach, ou client nu qui parle comme humain (`daemon.rs:8604-8606`). Borne de l'ADR 027.
- Pour le contrôle, « humain attesté » = rôle attach (R3b) ; un client nu n'en fait pas partie.
- Guidage implicite (message ordinaire remis en cours de tour) : humain attaché, parent, enfant de la cible. Pair : non.
- La garde de projet existante (`communication::project_scope`) reste appliquée après.
- La preuve enfant (ADR 044) est refusée pour `steer`, `interrupt` et `bridget_queue`.

## R6 - Anti-boucle et audit

- Table `guidance_events` (audit et compteur). Comptage `COUNT` indexé, O(log n). Cooldown 30 s par couple, plafond 3 par 10 min par cible. Humain exempt. Pause interne non comptée.
- **Ce n'est pas un reçu d'idempotence** (contre-revue 8) : le rejeu relit `idempotency_records` (7 jours, `daemon.rs:466`). Purge à 24 h seulement pour l'audit.
- Les refus de la garde sur `SendIdempotent` passent par `reject_idempotent_send` (`daemon.rs:7694-7718`) : mémorisés jusqu'à l'expiration, nouvelle clé pour réessayer. Même comportement que tous les refus d'envoi existants.

## R7 - Pause et mission native

- ADR 027 : `admit_autonomous_effect` classe chaque puits. `steer`/`interrupt` par un agent devient `AgentGuidance`. La reprise des remises déjà acceptées reste admise.
- La pause interrompt par `Interrupt` sans connexion (`daemon.rs:9667`) et mémorise la reprise (`control_pause_interruptions`, `execution_store.rs:1605`). Cette interruption n'est ni refusée par la garde ni comptée.
- Mission native : `interrupted` est lu comme un échec définitif (`native_delegation.rs:1193-1203`, spec 150). `interrupt` d'un enfant avec tâche native active : `mission_active` jusqu'à 152. `steer` reste permis.

## R8 - Lecture de la file, sans édition

- Source : `send_deliveries` en phase `dispatching` (`idempotency.rs:527-540`), jointe à `idempotency_records` pour `issued_at`. Ordre `rowid`, conservé par la réaffectation qui met à jour en place (`send_delivery.rs:230`). Une ligne `dispatching` peut être en cours d'injection : état `pending_or_injecting`.
- Un index `(recipient_instance_id, phase)` évite le parcours complet (`send_delivery.rs:372-377`).
- `CancelDelivery` (`protocol.rs:4147-4151`) est sans résultat. `cancel_delivery` Codex annule d'abord le tour actif si l'identifiant correspond (`codex_app_server.rs:1081-1093`).
- **Décision (Art. XIX, contre-revue 5 à 8 et 10)** : pas d'édition de file dans 156. Le `CHECK` de `phase` n'admet ni `cancelled` ni `rejected` (`idempotency.rs:534`) ; la reprise relit toute remise `dispatching` (`send_delivery.rs:379-449`) donc un retrait mémoire sans réservation durable est réinjecté ; `promote` exige une saga durable. Prérequis de la future session : réservation durable avant émission, accusé durable, réconciliation avant relivraison, phase d'annulation (reconstruction de table), retrait réservé aux remises encore en file sous le verrou du transport (Claude, ACP, file Codex `steer`), saga de promotion.
- Pour une demande suivie (`reply=true`), `bridget_cancel` existe déjà (`daemon.rs:18125-18200`) et envoie `CancelDelivery` aux fils T3 et ACP. Il est documenté, non modifié.

## R9 - Pont T3

- `dispatch_command(command, steer_current: bool)` ne pose que `deliveryIntent: "steer"` (`t3code_contract_v2.rs:255-272`). T3 accepte `auto | steer | restart` (T3 `orchestrationV2.ts:2858`).
- T3 `decideSteeringExecution` (`CommandPolicy.ts:238-262`) : sans `forceRestart` et sans guidage actif, T3 choisit `interrupt_restart` si le fournisseur sait interrompre. Donc un `steer` vers un fournisseur sans guidage natif interromprait en silence.
- **Décision** : `steer` envoyé seulement aux fournisseurs à guidage natif prouvé (Claude, Cursor, d'après la session 129, `t3code.rs:3126`) ; sinon rejeté `steer_unavailable`. `interrupt` pose `restart`.
- `maybe_deliver` n'examine que `queue.front()` (`t3code.rs:1962-1975`). Un message de guidage doit être choisi dans toute la file et rester hors lot (`batch_family`, `t3code.rs:3150`). La remise des groupables vers Claude/Cursor ne change pas.
- `QUEUE_BOUND` = 64 inchangé (`t3code.rs:44`) ; exposé par `bridget_queue`.

## R10 - Un outil MCP

- T3 a six outils de file. Bridget en prend un, en lecture seule : `bridget_queue` avec `target`. Un futur `action` s'ajoutera avec la session d'édition.
- Identité : `ResolvedIdentity` (`mcp.rs:673-684`), preuve privée, `RoleHandshake` Client puis `ClientHello` avec `QueueListV1`.

## R11 - Hors périmètre constaté

- `QueueOnly` du chemin historique `Send` accuse sans remettre (`daemon.rs:17634-17660`). `execution_queue` sans consommateur de production (`execution_store.rs:456`). Dettes notées.
- `cancel_delivery` Codex peut annuler le tour actif (voir R8) : à traiter dans la session d'édition.

## R12 - Complexité

- Garde : O(1). Comptages : O(log n). `bridget_queue` : O(log n + k), k ≤ 200. Pont : choix du message de guidage O(n), n ≤ 64.

## R13 - Contre-revue : ce qui a changé

Voir `adversarial-review-codex.md`. Supprimés : mode de repli, table de décision de l'effet, `intent_applied`, `QueueEdit`/`QueueControl`, `queue_rank`, phase `cancelled`, `cancel`/`reorder`/`promote`. Ajoutés : garde sur `Send` et `ControlExecution`, fabrication de l'origine, liste blanche T3, choix dans toute la file du pont, tests combinés.

## Vérifications ouvertes (levées par T002)

1. Ce que reçoit l'émetteur quand un transport rejette un `steer`.
2. Producteurs légitimes de `origin=Human` (attach, relais UI, client nu).
3. Fournisseurs T3 à guidage natif prouvé.
4. Noms des champs de la tâche native (propriétaire, `child`).
