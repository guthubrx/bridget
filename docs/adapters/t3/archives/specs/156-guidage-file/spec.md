# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/spec.md](../../../../../../specs/156-guidage-file/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Un agent qui en supervise un autre ne peut aujourd'hui que poster un message
dans sa file. Il ne peut pas corriger le tour en cours, ni l'arrêter. Il ne voit
pas ce qui attend. T3 Code offre ces choses (`research-t3-bridget.md`, B2).

<a id="extrait-2"></a>

## Extrait 2

1. **Les capacités existent, personne ne les expose.** `MessageIntent` a cinq valeurs (`crates/bridget-core/src/message.rs:54-60`). Codex sait guider un tour actif (`codex_app_server.rs:865-899`). Claude et ACP refusent le guidage (`claude_stream_json.rs:310-317`, `acp.rs:620-628`) mais interrompent (`:329`, `:630`). `bridget_send` n'a aucun argument d'intention (`mcp.rs:685-694`, schéma `:2028-2046`).
2. **Aucun contrôle de droits entre agents, sur trois chemins.**
   - `SendIdempotent` et `Send` historique ne retirent ni `intent` ni `origin` fournis par le client ; seul `thread_notice` est neutralisé (`daemon.rs:8584-8640`, `:17521-17560`). Un agent attesté peut donc envoyer `SteerCurrent` ou `InterruptAndStart` par une trame brute.
   - Les transports interrompent tout message `origin=Human` quand un tour est actif, même sans intention (`codex_app_server.rs:865-866`, `claude_stream_json.rs:329`, `acp.rs:630`). Une origine `Human` forgée par un agent suffit.
   - `ControlExecution` n'a aucune comparaison d'identité pour `Interrupt` (`daemon.rs:9002-9035`). Le client doit connaître `execution_id`, génération et révision, et négocier `ExecutionControlV1` (`daemon.rs:16174`), ce qui est à sa portée.
   Exposer le guidage sans corriger ces trois chemins élargirait la faille.
3. **La file de remise est opaque.** Les remises vivent dans la saga durable `send_deliveries` (`idempotency.rs:527`) et dans la file mémoire du transport ou du pont T3 (64 remises, `t3code.rs:44`).
4. **Le pont T3 sait déjà guider, avec deux pièges.** Il pose `deliveryIntent: "steer"` pour Claude et Cursor (`t3code_contract_v2.rs:269-271`). T3 peut cependant transformer un `steer` en interruption puis relance si le fournisseur ne guide pas nativement (T3 `CommandPolicy.ts:238-262`). Le pont n'examine que la tête de sa file (`t3code.rs:1962-1975`).

<a id="extrait-3"></a>

## Extrait 3

La garde 156 protège contre **un agent qui agit par ses outils Bridget** : MCP,
CLI, pont T3, wrapper. C'est la borne de l'ADR 011 (« un agent local peut
exécuter des commandes mais ne peut pas appuyer sur les touches d'un terminal
ouvert ») reprise par l'ADR 027.

<a id="extrait-4"></a>

## Extrait 4

**Dans le modèle (atteignable par les outils), donc à corriger** :
- `env -i bridget send …` : la CLI, sans preuve d'identité, signe `human` (`cli.rs:3479`, `:4322`) et devient un client nu traité en humain. Une seule commande suffit. FR-008 retire l'autorité de guidage aux clients nus.
- Le guidage implicite du pont T3 : un message ordinaire vers un fil Claude ou Cursor occupé part en `deliveryIntent: "steer"` (`t3code.rs:2206`, `t3code_contract_v2.rs:269`). FR-022.
- `bridget_cancel` : l'émetteur d'une demande déjà active interrompt la cible ACP, car `cancel_delivery` annule le tour actif (`daemon.rs:18128-18160`, `acp.rs:518-531`). FR-023.
- `ControlExecution` : l'origine `Human` du message n'est pas normalisée ; Codex guide ET interrompt (`wrapper.rs:5639`, `codex_app_server.rs:866`). FR-009.

<a id="extrait-5"></a>

## Extrait 5

Seuls l'humain attesté et le parent d'une cible guident ou interrompent cette
cible. Un pair du même projet ne le peut pas. Un agent ne peut pas se guider
lui-même. Aucun guidage ne franchit la frontière d'un projet. La garde est la
même pour `bridget_send`, `SendIdempotent`, `Send` historique,
`ControlExecution`, la CLI et le pont T3. Une trame brute avec `SteerCurrent`,
`InterruptAndStart` ou `origin=Human` d'un agent est refusée.

<a id="extrait-6"></a>

## Extrait 6

- **FR-001** : `bridget_send` accepte `intent` parmi `queue`, `steer`, `interrupt`. Absent ou `queue` : aucune intention posée, octets canoniques inchangés.
- **FR-002** : `bridget send` accepte `--intent` avec les mêmes valeurs.
- **FR-003** : Une autre valeur est refusée `invalid_intent`. Rien n'est envoyé.
- **FR-004** : L'intention DEMANDÉE est posée sur le message avant le canon : `steer` donne `SteerCurrent`, `interrupt` donne `InterruptAndStart`. Le daemon ne la réécrit jamais selon l'état de la cible. Rejouer une clé avec une autre intention rend `envelope_mismatch`. Le rejeu d'une clé admise rend le même résultat que le premier appel, que la cible soit active ou non. `steer` avec `reply=true` est refusé `invalid_intent` avant connexion/réservation : aucun tour dédié ne porte sa réponse.
- **FR-005** : Le résultat d'un envoi avec intention contient l'intention demandée et `effect: "unconfirmed"`. Le daemon ne dit jamais `steered`, `interrupted_restarted` ni `started`.
- **FR-006** : Vers un fil T3, `steer` n'est envoyé que si le fournisseur a un guidage natif prouvé (Claude seul confirmé ; Cursor refusé). Sinon la remise est rejetée `steer_unavailable`. T3 ne reçoit jamais un `steer` qu'il pourrait transformer en interruption. `interrupt` pose `deliveryIntent: "restart"`.
- **FR-007** : Dans le pont T3, un message de guidage passe devant les messages bloqués de la file et n'entre jamais dans un lot.

<a id="extrait-7"></a>

## Extrait 7

- **FR-017** : `bridget_queue` (lecture seule) rend, pour une cible, au plus 200 remises en phase `dispatching` : `message_id`, `from`, `intent`, `reply`, `age_secs`, `preview` (120 caractères), `position`. Chaque entrée est marquée « en attente ou en cours d'injection ». La réponse ajoute `queue_bound` (64 pour un fil T3, `null` sinon). Périmètre exact : send_deliveries idempotent en phase dispatching ; les Send historiques et les files internes des transports sont exclus.
- **FR-018** : La file entière est visible au destinataire, à l’humain Attach, à l’UI Client attestée et au parent. Tout autre appelant ne voit que ses propres messages. La preuve enfant (ADR 044) est refusée.
- **FR-019** : `bridget_capabilities` annonce `guidance` : intentions, délai et plafond d'interruption, présence de `bridget_queue`.

<a id="extrait-8"></a>

## Extrait 8

- **FR-020** : Les anciens clients et trames se lisent sans changement. La nouvelle requête exige la capacité `QueueListV1`. Attach conserve son rôle et peut envoyer ClientHello uniquement avec la liste exacte [QueueListV1], sans identité agent/conn_names ni autre capacité. L’UI Client attestée garde son rôle et peut lire toute la projection.
- **FR-022** : Le daemon calcule mid_turn_ok sur DeliverIdempotent ET Deliver historique, envoi/reprise. La remise immédiate utilise la connexion source réelle. À la récupération, from ne permet jamais d’emprunter une route UI ou humaine ; sans preuve UI persistée, la remise attend normalement. Pour les agents UUID, recalculer les liens depuis les faits actuels. Une remise ordinaire déjà groupable vers T3 Claude occupé, avec capacité activeSteering native prouvée, reste immédiate pour attach, UI attestée par sa preuve existante, producteur système interne, parent ou enfant de la cible. Le rôle Client de l'UI reste inchangé ; from=humain seul ne donne aucun droit. Les pairs/clients nus attendent ; valeur absente : attente. Cursor attend aussi en cours de tour, même avec mid_turn_ok=true, car son steer peut devenir interrupt_restart. Aucun origin=Human fabriqué. Les critères groupable existants restent : reply=true, sollicitations et notifications ne deviennent pas groupables.
- **FR-023** : `bridget_cancel` distingue le retrait en attente de l'interruption active. Pour tout appelant, la demande est annulée et son message encore en file est retiré. Le tour actif n'est interrompu que si l'appelant passe la garde commune (droits, pause, rythme, mission) ; l'interruption compte dans les limites de rythme. Sinon le tour continue, comme le dit déjà la description de l'outil (« n'arrête pas l'agent »). La trame `CancelDelivery` porte `interrupt_active` (défaut faux) et l'acteur ACP exécute les deux cas de façon atomique sous son verrou.
- **FR-024** : Le cadre de menace de cette spec figure dans `docs/reference-communication.md` et dans l'ADR.
- **FR-021** : `docs/reference-communication.md` (lignes 175-180 : « aucune option CLI de contrôle nouvelle »), la skill Bridget et une ADR décrivent la nouveauté. La doc dit que `bridget_cancel` annule une demande suivie.

<a id="extrait-9"></a>

## Extrait 9

- **SC-001** : Avec un transport Codex simulé en tour actif, un `steer` d'un parent produit l'événement `turn_steer` du transport sur le tour ciblé.
- **SC-002** : Avec Claude simulé, steer est rejeté sans interruption. Interrupt arrête le tour actif et le message reste dans la FIFO ; sans message plus ancien dans la fixture, il porte le nouveau tour.
- **SC-003** : 0 guidage ou interruption admis pour un pair du même projet, un appelant non attesté, une cible d'un autre projet ou soi-même, sur les quatre chemins (`SendIdempotent`, `Send`, `ControlExecution`, pont/CLI) et pour une `origin=Human` forgée.
- **SC-004** : 4 interruptions d'agent en 10 min sur une même cible : la 4e est refusée. Deux en moins de 30 s du même couple : la 2e est refusée.
- **SC-005** : 100 % des `steer`/`interrupt` d'un agent pendant la pause sont refusés `referent_paused` ; 0 tour créé ; l'envoi en file passe ; la pause interrompt toujours, même après 3 interruptions d'utilisateur.
- **SC-006** : Dans le pont T3, un message de guidage derrière une tête de file bloquée est remis ; un `steer` vers un fournisseur hors liste n'est jamais envoyé à T3.
- **SC-007** : Un appel sans `intent` produit les mêmes octets canoniques qu'avant ; les suites `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::` restent vertes.
- **SC-008** : Un `ControlExecution` `Interrupt` d'un client non autorisé est refusé `not_permitted` (accepté avant).
- **SC-010** : Parent autorisé, enfant en mission active, `steer` avec `origin=Human` forgée dans un `ControlExecution` : le tour n'est pas interrompu, la mission continue. Même à l'échéance du steer agent non consommé, aucune interruption automatique ; timeout humain validé conservé.
- **SC-011** : Un message ordinaire groupable d'un pair vers T3 Claude occupé attend. Parent/enfant, attach, UI attestée et système interne conservent la remise immédiate sur les deux trames. Aucun droit ne vient du seul from=humain.
- **SC-012** : `bridget_cancel` d'un pair sur sa demande déjà active vers une cible ACP : la demande est annulée, le tour n'est pas interrompu. Même appel d'un parent hors pause, sous le plafond, sans mission : le tour est interrompu.
- **SC-013** : `env -i bridget send --intent steer` (client nu) est refusé `identity_unattested`.
- **SC-009** : `bridget_queue` ne modifie aucune ligne (test d'égalité de la base avant/après) et filtre selon les droits.

<a id="extrait-10"></a>

## Extrait 10

- Le parent s'établit par `agent_links` ouvert (`agent_links.rs:79-134`) ou par la tâche de délégation native (propriétaire de la tâche dont `child` est la cible).
- « Humain attesté » pour le contrôle = connexion de rôle attach. Un client nu n'en fait pas partie. L'humain qui veut un contrôle explicite depuis un terminal passe par l'attach. La provenance UI attestée des messages ordinaires est distincte (FR-022) et ne change pas le rôle Client.
- Pendant la pause, l'agent ne guide ni n'interrompt (ADR 027, garde unique) ; l'envoi en file reste admis.
- La preuve fraîche du bundle T3 installé149 établit Claude seul pour le guidage actif natif. Cursor n’a pas supportsActiveSteering ; son repli interrupt_restart est fermé. Les autres fournisseurs, Cursor inclus, sont refusés pour `steer` avec steer_unavailable. Le transport Codex natif garde ses propres gardes.
- Les tests suivent la recette `umask 077` de la session150.
- La série152–160 autorise, après validation, build signé, installation et activation Bridget lorsqu’aucune mission native non terminale ne reste, puis nettoyage. T3 reste sans relance explicite. Aucune nouvelle permission n’est demandée ; les checkpoints sans relance147–150 sont historiques.

<a id="extrait-11"></a>

## Extrait 11

1. **Un pair du même projet peut-il guider ?** Défaut retenu : non. Seuls l'humain et le parent guident ou interrompent.
2. **Reporter `cancel`, `reorder`, `promote` à une session dédiée avec saga durable ?** Défaut retenu : oui. Conséquence : Guidage vise 4 sur 5 dans 156 ; le 5 vient avec cette session.
3. **Deux changements visibles retenus** : les pairs attendent la fin du tour T3 occupé ; la CLI nue ne peut demander steer/interrupt. Les messages ordinaires attach/UI attestés et systèmes internes restent compatibles. Arbitrages F1/F2/F3/F5 acceptés par root le2026-10-12, sans nouvelle question utilisateur.
