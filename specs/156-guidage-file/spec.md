# Session156 - Guidage, interruption et lecture de file exposés aux agents

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 156-guidage-file
Titre: Guidage et interruption exposés aux agents, droits complets, lecture de la file
Statut: Completed (32/32 ; livré, actif et nettoyé le2026-10-12)
Priorité: P1
Tâches: 32/32 (T001–T032 validées par le principal)
Tests: suite unique 2350 PASS Cargo + 3 PASS enfants séparés / 11 FAIL / 73 ignored ; 80 unitaires156 et 3 E2E inclus, trois fixtures corrigées par reprises 2+1 PASS ; huit échecs hérités, aucun GREEN global

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
**Status**: Completed
**Priority**: P1
**Dependencies**: SPEC-150 (acquis supposés livrés), SPEC-148, SPEC-149

Critères de la feuille de route : Guidage, interruption, file 2 vers **4**
(le 5 attend la session d'édition de file, voir « Hors périmètre ») ; Envoi de
messages et suivi 4 vers 5
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

## Besoin et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-1).

Constats prouvés dans le code (checkout principal, base `1f2c661d`) :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-2).

## Cadre de menace (ADR 011, ADR 027)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-3).

**Hors modèle, déclaré et non promis.** Un processus hostile du même compte Unix
qui ouvre une socket brute et forge des trames n'est pas arrêté. Chemins prouvés :
- le rôle d'une connexion est déclaratif : `RoleHandshake{Attach}` est accepté sans preuve (`daemon.rs:14213`) ;
- un client nu qui écrit `from=human` est traité comme humain par `SendIdempotent` (`daemon.rs:8604-8606`) ;
- l'agent qui alloue son propre pseudo-terminal et pilote `bridget attach` simule un clavier (`attach.rs:267`, `:405-415`) ; l'ADR 011 ne couvre que le clavier réel de l'utilisateur.

156 teste certains de ces cas comme défense en profondeur. Elle ne promet pas de
les fermer.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-4).

**Mission native.** Une interruption utilisateur reste terminale. La session152
reprend pause/panne, pas une interruption utilisateur. Un agent ne peut donc
pas interrompre un enfant en mission active. L'humain explicitement autorisé
est exempt comme FR012. Chemin réel : crates/bridget-daemon/src/daemon/native_delegation.rs.
T002 doit revalider ce comportement sur la base155 livrée.

**Hors périmètre (décision après contre-revue, Art. XIX)** :
- **Éditer la file : `cancel`, `reorder`, `promote`.** Une édition sûre exige une saga durable : réservation avant émission, accusé durable, réconciliation à la reprise, phase d'annulation (le `CHECK` de `send_deliveries.phase` n'en admet pas, `idempotency.rs:534`), saga de promotion. Le coût dépasse le gain. Les demandes suivies (`reply=true`) ont déjà `bridget_cancel`. Voir la question 2.
- Mode `auto` ; dégradation silencieuse de `steer` en interruption ; édition du corps ; changement de la borne de 64 ; correction du chemin historique `QueueOnly` sans consommateur (`execution_store.rs:456`, `daemon.rs:17634-17660`) ; guidage d'une session interactive TUI ; fédération entre machines ; reprise d'un tour interrompu (session152) ; budgets (session158).

## Scénarios utilisateur

### US1 - Choisir l'intention à l'envoi (P1)

Un parent voit son enfant partir dans la mauvaise direction. Il appelle
`bridget_send` avec `intent: "steer"` et la correction. Avec `intent:
"interrupt"`, le transport demande l'arrêt du tour actif et garde le message dans la FIFO. Un message plus ancien peut porter le prochain tour ; aucune priorité nouvelle n'est promise.
Sans `intent`, ou avec `intent: "queue"`, l'envoi ordinaire conserve ses octets
canoniques. Les restrictions de provenance et le guidage implicite FR-022
restent des changements visibles. La CLI offre `--intent`.

Le résultat de l'envoi dit que l'intention demandée est admise. Il ne promet
pas l'effet. L'effet réel est celui du transport : injecté dans le tour, remis
en file si le fournisseur refuse le guidage, ou rejeté (Claude, ACP). Il se lit
dans l'accusé de remise et le journal du transport.

### US2 - Droits et garde-fous sur tous les chemins (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-5).

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

Un envoi ordinaire légitime sans nouveau paramètre conserve ses octets
canoniques et son comportement, sous les restrictions explicites de FR-009
et FR-022. Une provenance forgée ne conserve aucun privilège. La remise immédiate des messages groupables vers un fil
Claude occupé avec activeSteering natif prouvé reste inchangée **pour l'humain attaché, l'UI attestée, les systèmes internes, le parent
et ses enfants** ; les autres expéditeurs attendent la fin du tour (FR-022,
changement visible, question 3). Cursor occupé attend aussi, même pour ces
acteurs et mid_turn_ok=true ; son steer explicite est refusé pour fermer le
repli interrupt_restart non sûr accepté par root. Les 64 remises du pont restent la borne.

## Exigences fonctionnelles

**Intention d'envoi**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-6).

**Droits et garde-fous**

- **FR-008** : Steer/interrupt explicites sont permis à l'humain attaché attesté et au parent de la cible (lien ouvert ou propriétaire natif). Un client nu signé from=human garde la file mais reçoit identity_unattested pour le contrôle. Autres refus : intent_not_permitted (not_parent, cross_project, self_target, identity_unattested). Le rôle est immuable sur la même connexion ; une reconnexion avec preuve d'agent ne devient pas humaine. Une nouvelle connexion Attach brute du même compte reste hors modèle. La provenance UI attestée des messages ordinaires ne change pas son rôle Client ni cette autorité explicite.
- **FR-009** : L'identité vient de la connexion attestée, jamais de from. Le daemon normalise la provenance avant le canon sur Send et SendIdempotent. Une origine humaine déjà fournie est conservée seulement pour attach ou pour le relais UI prouvé (identité UI réservée, instance primaire vivante de type ui, credential existant validé). La provenance système interne vient du producteur, pas de origin=System déclaré par un client. Aucun droit ne vient du seul from=humain ni de l'origine cliente. Ne jamais fabriquer origin=Human pour autoriser mid_turn_ok.
- **FR-010** : Le message porté par une commande `ControlExecution` reçoit la même normalisation d'origine que les envois, avant le canon et avant la garde ; `origin=Human` y est toujours retirée (une commande de contrôle n'est pas une voie attach). Pour ControlExecution, reserve_control_command conserve canon et rejeu AVANT la garde. La garde d’une commande neuve précède mark_control_dispatched, le comptage et l’émission ; le rejeu conserve sa réponse existante. La même garde s'applique à `SendIdempotent`, `Send` historique (y compris `SteerCurrent`/`InterruptAndStart` bruts) et `ControlExecution` (opérations `SteerCurrent` et `Interrupt`). Refus de `ControlExecution` : nouvelle valeur `not_permitted` de `ExecutionControlRefusal`.
- **FR-011** : L'interruption de pause (sans connexion cliente, `daemon.rs:9667`) est une exception interne explicite. Elle n'est ni refusée ni comptée par les limites de rythme.
- **FR-012** : `interrupt` par un agent est limité à 1 par 30 s par couple appelant/cible et à 3 par 10 min par cible. Refus `interrupt_rate_limited` avec `retry_after_secs`. L'humain attesté n'est pas limité.
- **FR-013** : Interrupt par un agent vers un enfant en mission native active est refusé mission_active : l'interruption utilisateur reste terminale malgré la reprise152 de pause/panne. L'humain explicitement autorisé est exempt comme FR012. Steer agent reste permis ; aucun repli automatique en turn/interrupt, même à l'échéance d'un steer non consommé ou avec une deadline fournie. Garder le timeout du steer humain validé et l'échéance propre du tour principal.
- **FR-014** : Pendant la pause du référent, `steer` et `interrupt` par un agent rendent `referent_paused`. L'envoi en file reste admis. L'humain attesté n'est pas bloqué.
- **FR-015** : Un refus de `SendIdempotent` est mémorisé par le mécanisme existant (`reject_idempotent_send`, jusqu'à l'expiration de l'enregistrement) : pour réessayer, il faut une nouvelle clé. Un refus de `Send` historique est un `Nack`. Aucun message n'est remis dans les deux cas.
- **FR-016** : Chaque action de guidage admise ou refusée est tracée (appelant, cible, action, chemin, résultat, date) dans une table d'audit séparée du reçu d'idempotence. Elle sert aussi aux limites de rythme. Elle est purgée après 24 h. SendIdempotent inscrit l’admission guidance_events dans la même transaction begin_send_delivery_with_guidance que remise/suivi/lien/journal. Sur les chemins historique/control/cancel, l’audit accepté est durable avant émission physique. Un INSERT échoué interdit le contrôle. Un crash après admission mais avant émission peut consommer le rythme sans effet physique ; ce choix est conservateur.

**Lecture de la file**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-7).

**Compatibilité et documentation**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-8).

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-9).

## Entités et hypothèses

**Entités**

- **Intention d'envoi** : `queue`, `steer`, `interrupt`. Correspond à aucune intention posée, `SteerCurrent`, `InterruptAndStart`.
- **Remise en file** : ligne `send_deliveries` en phase `dispatching`.
- **Événement de guidage** : trace d'audit, support des limites de rythme. Ce n'est pas un reçu d'idempotence.

**Hypothèses**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-10).

## Questions pour l'utilisateur

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/spec.md#extrait-11).
