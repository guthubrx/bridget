# Session156 - Guidage, interruption et lecture de file exposés aux agents

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 156-guidage-file
Titre: Guidage et interruption exposés aux agents, droits complets, lecture de la file
Statut: Spécifiée (révisée après contre-revues Codex rondes 1 et 2, 2026-10-11)
Priorité: P1
Tâches: 31 à cocher (voir tasks.md)
Tests: rouges d'abord, filtres `guidance156_*`

Résumé:
- Contexte: Le protocole sait déjà guider un tour (`SteerCurrent`) et l'interrompre (`InterruptAndStart`, `Interrupt`). Seule la pause du référent s'en sert. `bridget_send` et la CLI n'ont aucun paramètre d'intention. Pire : un agent attesté peut déjà guider ou interrompre un autre agent par une trame brute, car le daemon ne vérifie aucun droit entre agents.
- Objectif: Un agent choisit file, guidage ou interruption à l'envoi, sous une garde de droits unique qui couvre tous les chemins, sans boucle d'interruption. Un outil lit la file d'un agent.
- Dépendances: SPEC-150 (garde de pause, refus nommés), SPEC-148/149 (lignée, droits hérités), ADR 027 (pause), ADR 007 (idempotence), ADR 044 (preuve enfant). Lien avec SPEC-152 (interruption d'une mission).

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-156-guidage-file`
**Created**: 2026-10-11
**Status**: Spécifiée
**Priority**: P1
**Dependencies**: SPEC-150 (acquis supposés livrés), SPEC-148, SPEC-149

Critères de la feuille de route : Guidage, interruption, file 2 vers **4**
(le 5 attend la session d'édition de file, voir « Hors périmètre ») ; Envoi de
messages et suivi 4 vers 5
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

## Besoin et périmètre

Un agent qui en supervise un autre ne peut aujourd'hui que poster un message
dans sa file. Il ne peut pas corriger le tour en cours, ni l'arrêter. Il ne voit
pas ce qui attend. T3 Code offre ces choses (`research-t3-bridget.md`, B2).

Constats prouvés dans le code (checkout principal, base `1f2c661d`) :

1. **Les capacités existent, personne ne les expose.** `MessageIntent` a cinq valeurs (`crates/bridget-core/src/message.rs:54-60`). Codex sait guider un tour actif (`codex_app_server.rs:865-899`). Claude et ACP refusent le guidage (`claude_stream_json.rs:310-317`, `acp.rs:620-628`) mais interrompent (`:329`, `:630`). `bridget_send` n'a aucun argument d'intention (`mcp.rs:685-694`, schéma `:2028-2046`).
2. **Aucun contrôle de droits entre agents, sur trois chemins.**
   - `SendIdempotent` et `Send` historique ne retirent ni `intent` ni `origin` fournis par le client ; seul `thread_notice` est neutralisé (`daemon.rs:8584-8640`, `:17521-17560`). Un agent attesté peut donc envoyer `SteerCurrent` ou `InterruptAndStart` par une trame brute.
   - Les transports interrompent tout message `origin=Human` quand un tour est actif, même sans intention (`codex_app_server.rs:865-866`, `claude_stream_json.rs:329`, `acp.rs:630`). Une origine `Human` forgée par un agent suffit.
   - `ControlExecution` n'a aucune comparaison d'identité pour `Interrupt` (`daemon.rs:9002-9035`). Le client doit connaître `execution_id`, génération et révision, et négocier `ExecutionControlV1` (`daemon.rs:16174`), ce qui est à sa portée.
   Exposer le guidage sans corriger ces trois chemins élargirait la faille.
3. **La file de remise est opaque.** Les remises vivent dans la saga durable `send_deliveries` (`idempotency.rs:527`) et dans la file mémoire du transport ou du pont T3 (64 remises, `t3code.rs:44`).
4. **Le pont T3 sait déjà guider, avec deux pièges.** Il pose `deliveryIntent: "steer"` pour Claude et Cursor (`t3code_contract_v2.rs:269-271`). T3 peut cependant transformer un `steer` en interruption puis relance si le fournisseur ne guide pas nativement (T3 `CommandPolicy.ts:238-262`). Le pont n'examine que la tête de sa file (`t3code.rs:1962-1975`).

## Cadre de menace (ADR 011, ADR 027)

La garde 156 protège contre **un agent qui agit par ses outils Bridget** : MCP,
CLI, pont T3, wrapper. C'est la borne de l'ADR 011 (« un agent local peut
exécuter des commandes mais ne peut pas appuyer sur les touches d'un terminal
ouvert ») reprise par l'ADR 027.

**Hors modèle, déclaré et non promis.** Un processus hostile du même compte Unix
qui ouvre une socket brute et forge des trames n'est pas arrêté. Chemins prouvés :
- le rôle d'une connexion est déclaratif : `RoleHandshake{Attach}` est accepté sans preuve (`daemon.rs:14213`) ;
- un client nu qui écrit `from=human` est traité comme humain par `SendIdempotent` (`daemon.rs:8604-8606`) ;
- l'agent qui alloue son propre pseudo-terminal et pilote `bridget attach` simule un clavier (`attach.rs:267`, `:405-415`) ; l'ADR 011 ne couvre que le clavier réel de l'utilisateur.

156 teste certains de ces cas comme défense en profondeur. Elle ne promet pas de
les fermer.

**Dans le modèle (atteignable par les outils), donc à corriger** :
- `env -i bridget send …` : la CLI, sans preuve d'identité, signe `human` (`cli.rs:3479`, `:4322`) et devient un client nu traité en humain. Une seule commande suffit. FR-008 retire l'autorité de guidage aux clients nus.
- Le guidage implicite du pont T3 : un message ordinaire vers un fil Claude ou Cursor occupé part en `deliveryIntent: "steer"` (`t3code.rs:2206`, `t3code_contract_v2.rs:269`). FR-022.
- `bridget_cancel` : l'émetteur d'une demande déjà active interrompt la cible ACP, car `cancel_delivery` annule le tour actif (`daemon.rs:18128-18160`, `acp.rs:518-531`). FR-023.
- `ControlExecution` : l'origine `Human` du message n'est pas normalisée ; Codex guide ET interrompt (`wrapper.rs:5639`, `codex_app_server.rs:866`). FR-009.

**Constat transmis à la session152.** L'interruption d'un enfant en mission
native fait échouer la mission pour de bon (`native_delegation.rs:1193-1203`,
spec 150). Tant que 152 ne reprend pas une mission, cette session refuse
d'interrompre un enfant qui a une mission active.

**Hors périmètre (décision après contre-revue, Art. XIX)** :
- **Éditer la file : `cancel`, `reorder`, `promote`.** Une édition sûre exige une saga durable : réservation avant émission, accusé durable, réconciliation à la reprise, phase d'annulation (le `CHECK` de `send_deliveries.phase` n'en admet pas, `idempotency.rs:534`), saga de promotion. Le coût dépasse le gain. Les demandes suivies (`reply=true`) ont déjà `bridget_cancel`. Voir la question 2.
- Mode `auto` ; dégradation silencieuse de `steer` en interruption ; édition du corps ; changement de la borne de 64 ; correction du chemin historique `QueueOnly` sans consommateur (`execution_store.rs:456`, `daemon.rs:17634-17660`) ; guidage d'une session interactive TUI ; fédération entre machines ; reprise d'un tour interrompu (session152) ; budgets (session158).

## Scénarios utilisateur

### US1 - Choisir l'intention à l'envoi (P1)

Un parent voit son enfant partir dans la mauvaise direction. Il appelle
`bridget_send` avec `intent: "steer"` et la correction. Avec `intent:
"interrupt"`, le tour en cours s'arrête et le message démarre un nouveau tour.
Sans `intent`, ou avec `intent: "queue"`, le comportement est exactement celui
d'aujourd'hui. La CLI offre `--intent`.

Le résultat de l'envoi dit que l'intention demandée est admise. Il ne promet
pas l'effet. L'effet réel est celui du transport : injecté dans le tour, remis
en file si le fournisseur refuse le guidage, ou rejeté (Claude, ACP). Il se lit
dans l'accusé de remise et le journal du transport.

### US2 - Droits et garde-fous sur tous les chemins (P1)

Seuls l'humain attesté et le parent d'une cible guident ou interrompent cette
cible. Un pair du même projet ne le peut pas. Un agent ne peut pas se guider
lui-même. Aucun guidage ne franchit la frontière d'un projet. La garde est la
même pour `bridget_send`, `SendIdempotent`, `Send` historique,
`ControlExecution`, la CLI et le pont T3. Une trame brute avec `SteerCurrent`,
`InterruptAndStart` ou `origin=Human` d'un agent est refusée.

Une interruption ne boucle pas : au plus une par 30 s pour un couple
appelant/cible, trois par 10 min pour une cible. Pendant la pause du référent,
un agent ne guide ni n'interrompt. L'interruption d'un enfant en mission est
refusée. La pause elle-même interrompt toujours, même après trois
interruptions d'utilisateur.

### US3 - Voir la file d'un agent (P2)

Un agent appelle `bridget_queue` sur une cible. Il voit les messages en attente
ou en cours d'injection : identifiant, expéditeur, âge, intention, aperçu,
position. Il ne voit que ce que ses droits permettent. L'outil ne modifie rien.

### US4 - Aucun changement pour l'existant (P1)

Un appel sans nouveau paramètre produit les mêmes octets canoniques et le même
comportement qu'avant. La remise immédiate des messages groupables vers un fil
Claude ou Cursor occupé (session 129) reste inchangée **pour l'humain, le parent
et ses enfants** ; les autres expéditeurs attendent la fin du tour (FR-022,
changement visible, question 3). Les 64 remises du pont restent la borne.

## Exigences fonctionnelles

**Intention d'envoi**

- **FR-001** : `bridget_send` accepte `intent` parmi `queue`, `steer`, `interrupt`. Absent ou `queue` : aucune intention posée, octets canoniques inchangés.
- **FR-002** : `bridget send` accepte `--intent` avec les mêmes valeurs.
- **FR-003** : Une autre valeur est refusée `invalid_intent`. Rien n'est envoyé.
- **FR-004** : L'intention DEMANDÉE est posée sur le message avant le canon : `steer` donne `SteerCurrent`, `interrupt` donne `InterruptAndStart`. Le daemon ne la réécrit jamais selon l'état de la cible. Rejouer une clé avec une autre intention rend `envelope_mismatch`. Le rejeu d'une clé admise rend le même résultat que le premier appel, que la cible soit active ou non.
- **FR-005** : Le résultat d'un envoi avec intention contient l'intention demandée et `effect: "unconfirmed"`. Le daemon ne dit jamais `steered`, `interrupted_restarted` ni `started`.
- **FR-006** : Vers un fil T3, `steer` n'est envoyé que si le fournisseur a un guidage natif prouvé (Claude, Cursor). Sinon la remise est rejetée `steer_unavailable`. T3 ne reçoit jamais un `steer` qu'il pourrait transformer en interruption. `interrupt` pose `deliveryIntent: "restart"`.
- **FR-007** : Dans le pont T3, un message de guidage passe devant les messages bloqués de la file et n'entre jamais dans un lot.

**Droits et garde-fous**

- **FR-008** : `steer` et `interrupt` sont permis à l'humain attesté et au parent de la cible (lien d'agent ouvert, ou tâche de délégation native dont l'appelant est propriétaire). « Humain attesté » veut dire une connexion de rôle attach. Un client nu qui signe `from=human` (par exemple `env -i bridget send`) n'est PAS attesté pour le contrôle : refus `identity_unattested`. Il garde l'envoi en file. Tout autre appelant reçoit `intent_not_permitted` avec un motif (`not_parent`, `cross_project`, `self_target`, `identity_unattested`). Un agent attesté qui se reconnecte ou ouvre un autre rôle ne gagne aucune autorité humaine.
- **FR-009** : L'identité de l'appelant vient de la connexion attestée, jamais de `from`. Le daemon fabrique l'origine : `origin=Human` n'est conservée que pour une connexion attach ou un client nu humain ; pour tout autre client elle est retirée avant le canon, sur `Send` et sur `SendIdempotent`.
- **FR-010** : Le message porté par une commande `ControlExecution` reçoit la même normalisation d'origine que les envois, avant le canon et avant la garde ; `origin=Human` y est toujours retirée (une commande de contrôle n'est pas une voie attach). La même garde s'applique à `SendIdempotent`, `Send` historique (y compris `SteerCurrent`/`InterruptAndStart` bruts) et `ControlExecution` (opérations `SteerCurrent` et `Interrupt`). Refus de `ControlExecution` : nouvelle valeur `not_permitted` de `ExecutionControlRefusal`.
- **FR-011** : L'interruption de pause (sans connexion cliente, `daemon.rs:9667`) est une exception interne explicite. Elle n'est ni refusée ni comptée par les limites de rythme.
- **FR-012** : `interrupt` par un agent est limité à 1 par 30 s par couple appelant/cible et à 3 par 10 min par cible. Refus `interrupt_rate_limited` avec `retry_after_secs`. L'humain attesté n'est pas limité.
- **FR-013** : `interrupt` d'un enfant qui a une tâche de délégation native active est refusé `mission_active`. `steer` reste permis.
- **FR-014** : Pendant la pause du référent, `steer` et `interrupt` par un agent rendent `referent_paused`. L'envoi en file reste admis. L'humain attesté n'est pas bloqué.
- **FR-015** : Un refus de `SendIdempotent` est mémorisé par le mécanisme existant (`reject_idempotent_send`, jusqu'à l'expiration de l'enregistrement) : pour réessayer, il faut une nouvelle clé. Un refus de `Send` historique est un `Nack`. Aucun message n'est remis dans les deux cas.
- **FR-016** : Chaque action de guidage admise ou refusée est tracée (appelant, cible, action, chemin, résultat, date) dans une table d'audit séparée du reçu d'idempotence. Elle sert aussi aux limites de rythme. Elle est purgée après 24 h.

**Lecture de la file**

- **FR-017** : `bridget_queue` (lecture seule) rend, pour une cible, au plus 200 remises en phase `dispatching` : `message_id`, `from`, `intent`, `reply`, `age_secs`, `preview` (120 caractères), `position`. Chaque entrée est marquée « en attente ou en cours d'injection ». La réponse ajoute `queue_bound` (64 pour un fil T3, `null` sinon).
- **FR-018** : La file entière est visible au destinataire, à l'humain et au parent. Tout autre appelant ne voit que ses propres messages. La preuve enfant (ADR 044) est refusée.
- **FR-019** : `bridget_capabilities` annonce `guidance` : intentions, délai et plafond d'interruption, présence de `bridget_queue`.

**Compatibilité et documentation**

- **FR-020** : Les anciens clients et trames se lisent sans changement. La nouvelle requête exige la capacité `QueueListV1`.
- **FR-022** : Le guidage implicite du pont T3 est soumis à la garde. Un message ordinaire sans intention n'est remis en cours de tour (`deliveryIntent: "steer"`, session 129) que si son expéditeur est l'humain attesté, le parent de la cible ou un enfant de la cible. Le daemon calcule cette autorisation et la pose sur la trame de remise (`mid_turn_ok`). Les autres expéditeurs, dont les pairs, attendent la fin du tour. Sans la valeur (ancienne trame), le pont attend la fin du tour.
- **FR-023** : `bridget_cancel` distingue le retrait en attente de l'interruption active. Pour tout appelant, la demande est annulée et son message encore en file est retiré. Le tour actif n'est interrompu que si l'appelant passe la garde commune (droits, pause, rythme, mission) ; l'interruption compte dans les limites de rythme. Sinon le tour continue, comme le dit déjà la description de l'outil (« n'arrête pas l'agent »). La trame `CancelDelivery` porte `interrupt_active` (défaut faux) et l'acteur ACP exécute les deux cas de façon atomique sous son verrou.
- **FR-024** : Le cadre de menace de cette spec figure dans `docs/reference-communication.md` et dans l'ADR.
- **FR-021** : `docs/reference-communication.md` (lignes 175-180 : « aucune option CLI de contrôle nouvelle »), la skill Bridget et une ADR décrivent la nouveauté. La doc dit que `bridget_cancel` annule une demande suivie.

## Critères de succès

- **SC-001** : Avec un transport Codex simulé en tour actif, un `steer` d'un parent produit l'événement `turn_steer` du transport sur le tour ciblé.
- **SC-002** : Avec un transport Claude simulé, `steer` est rejeté et aucun tour n'est interrompu ; `interrupt` démarre un nouveau tour.
- **SC-003** : 0 guidage ou interruption admis pour un pair du même projet, un appelant non attesté, une cible d'un autre projet ou soi-même, sur les quatre chemins (`SendIdempotent`, `Send`, `ControlExecution`, pont/CLI) et pour une `origin=Human` forgée.
- **SC-004** : 4 interruptions d'agent en 10 min sur une même cible : la 4e est refusée. Deux en moins de 30 s du même couple : la 2e est refusée.
- **SC-005** : 100 % des `steer`/`interrupt` d'un agent pendant la pause sont refusés `referent_paused` ; 0 tour créé ; l'envoi en file passe ; la pause interrompt toujours, même après 3 interruptions d'utilisateur.
- **SC-006** : Dans le pont T3, un message de guidage derrière une tête de file bloquée est remis ; un `steer` vers un fournisseur hors liste n'est jamais envoyé à T3.
- **SC-007** : Un appel sans `intent` produit les mêmes octets canoniques qu'avant ; les suites `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::` restent vertes.
- **SC-008** : Un `ControlExecution` `Interrupt` d'un client non autorisé est refusé `not_permitted` (accepté avant).
- **SC-010** : Parent autorisé, enfant en mission active, `steer` avec `origin=Human` forgée dans un `ControlExecution` : le tour n'est pas interrompu, la mission continue.
- **SC-011** : Un pair du même projet qui envoie un message ordinaire à un fil Claude occupé : la remise attend la fin du tour. Un parent, un enfant ou l'humain attaché : remise immédiate comme avant.
- **SC-012** : `bridget_cancel` d'un pair sur sa demande déjà active vers une cible ACP : la demande est annulée, le tour n'est pas interrompu. Même appel d'un parent hors pause, sous le plafond, sans mission : le tour est interrompu.
- **SC-013** : `env -i bridget send --intent steer` (client nu) est refusé `identity_unattested`.
- **SC-009** : `bridget_queue` ne modifie aucune ligne (test d'égalité de la base avant/après) et filtre selon les droits.

## Entités et hypothèses

**Entités**

- **Intention d'envoi** : `queue`, `steer`, `interrupt`. Correspond à aucune intention posée, `SteerCurrent`, `InterruptAndStart`.
- **Remise en file** : ligne `send_deliveries` en phase `dispatching`.
- **Événement de guidage** : trace d'audit, support des limites de rythme. Ce n'est pas un reçu d'idempotence.

**Hypothèses**

- Le parent s'établit par `agent_links` ouvert (`agent_links.rs:79-134`) ou par la tâche de délégation native (propriétaire de la tâche dont `child` est la cible).
- « Humain attesté » pour le contrôle = connexion de rôle attach (UI, TUI). Un client nu n'en fait pas partie. L'humain qui veut guider depuis un terminal passe par l'attach ou l'interface T3 (question 3).
- Pendant la pause, l'agent ne guide ni n'interrompt (ADR 027, garde unique) ; l'envoi en file reste admis.
- Un fournisseur T3 à guidage natif prouvé est Claude ou Cursor (session 129, `t3code.rs:3126`). Les autres sont refusés pour `steer`.
- Les tests suivent la recette `umask 077` de la session150.
- La livraison suit 147 à 150 : fichiers sur disque, sans relance de service.

## Questions pour l'utilisateur

1. **Un pair du même projet peut-il guider ?** Défaut retenu : non. Seuls l'humain et le parent guident ou interrompent.
2. **Reporter `cancel`, `reorder`, `promote` à une session dédiée avec saga durable ?** Défaut retenu : oui. Conséquence : Guidage vise 4 sur 5 dans 156 ; le 5 vient avec cette session.
3. **Acceptez-vous deux changements visibles ?** (a) Un message ordinaire vers un fil T3 Claude ou Cursor occupé n'est plus remis en cours de tour que s'il vient de l'humain, du parent ou d'un enfant de la cible ; un pair attend la fin du tour. (b) `bridget send --intent steer|interrupt` depuis un terminal nu est refusé ; l'humain guide par l'attach ou l'interface T3. Défaut retenu : oui aux deux, car sans (a) et (b) un pair ou un agent à l'environnement vidé contourne la garde.
