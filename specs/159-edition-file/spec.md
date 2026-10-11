# Session159 - File de remise tenue par le daemon et édition (159a, 159b)

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 159-edition-file
Titre: File de remise tenue par le daemon (159a) et édition de cette file (159b)
Statut: Spécifiée (conception refondue après contre-revues Codex, rondes 1 et 2, 2026-10-11)
Priorité: P1
Tâches: 27 à cocher (159a : 13, 159b : 14, voir tasks.md)
Tests: rouges d'abord, filtres `held159_*` (lot A) et `edit159_*` (lot B)

Résumé:
- Contexte: T3 Code permet de lister, retirer, éditer, réordonner un message en file et de le promouvoir en guidage, parce que sa file est durable côté serveur. Chez Bridget la file vit dans la mémoire de chaque porteur (transport géré, pont T3). Toute édition devient donc une saga répartie entre deux processus, et la contre-revue de la première conception a relevé 11 défauts de cette racine commune.
- Objectif: Le daemon retient en base les messages destinés à un agent occupé, puis les libère quand l'agent est libre. Retirer, réordonner et promouvoir un message retenu devient une transaction unique. Guidage 4 vers 5.
- Dépendances: SPEC-156 (garde de droits, origine fabriquée, `bridget_queue` en lecture, cadre de menace), SPEC-150, ADR 007, ADR 011, ADR 027, ADR 044.

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-159-edition-file`
**Created**: 2026-10-11
**Status**: Spécifiée
**Priority**: P1
**Dependencies**: SPEC-156 (acquis supposés livrés), SPEC-150

Accord utilisateur du 2026-10-11 : être au moins aussi bon que T3
(`t3_queue_list/read/edit/cancel/reorder/promote_to_steer`). Critère visé :
Guidage, interruption, file 4 vers **5**
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

## Besoin et périmètre

Un parent voit un message en file qui n'a plus de sens, ou qui doit passer avant
les autres, ou qui doit corriger le tour en cours. Il ne peut rien faire.

### Pourquoi la file doit vivre dans le daemon

Aujourd'hui, `bridget_send` pousse le message au porteur dès son admission
(`DeliverIdempotent`, `daemon.rs:4112-4172`). Le porteur le range dans sa file
mémoire (Codex : `messages` et `steer`, `codex_app_server.rs:876-900` ; Claude et
ACP : `messages` ; pont T3 : `worker.queue`, borne 64, `t3code.rs:44`). Éditer
cette file exige de coordonner deux processus : retrait en mémoire, tombe durable,
accusé, réconciliation à la reprise, ordre divergent, réaffectation d'instance.
La première conception de 159 l'a fait et la contre-revue l'a refusée (11 points,
voir `adversarial-review-codex.md`).

Le daemon possède déjà tout ce qu'il faut pour tenir la file :
- une base durable unique, avec la saga de remise `send_deliveries` (`idempotency.rs:527`) ;
- un signal de fin de tour des porteurs gérés : `TurnState` met la présence en `busy` ou `connected` (`daemon.rs:3473-3491`, `:17812`) ;
- une reprise des remises non accusées (`daemon.rs:4175-4230`).

Ce qui manque : une phase « retenue » et son déclencheur de libération.

### Exigences d'entrée (issues de 156, objections 5 à 8 et 10) et leur sort

| # | Exigence | Satisfaite par |
|---|---|---|
| E1 | Retirer seulement un message en file, jamais un message actif | Seuls les messages `held` (retenus par le daemon) sont éditables. Un message libéré est engagé : `not_held` |
| E2 | Accusé durable, état final `cancelled` | Une transaction écrit l'état `cancelled` ; rien à accuser chez le porteur |
| E3 | Réservation avant émission, réconciliation avant relivraison | Une édition n'émet rien. La seule émission est la libération, déjà couverte par la reprise des remises `dispatching` |
| E4 | Migration valide du `CHECK` de phase | Migration v7 (phases `held` et `cancelled`, `queue_rank`), précontrôle relevé à 7 |
| E5 | Attente contre engagé, rangs atomiques | `held` contre `dispatching`; une seule copie de l'ordre ; rang global attribué dans la transaction d'écriture |
| E6 | Reçu complet, ni perte ni doublon | Reçu `queue_edits` écrit dans la même transaction que l'effet ; résultat toujours final |

### Découpage

- **159a - File retenue par le daemon.** Retenir, libérer, balayer, lire. Réglage par type d'agent, **fermé par défaut** : sans le réglage, aucun changement de comportement. Aucune édition.
- **159b - Édition.** `cancel`, `reorder`, `promote` sur les messages retenus, plus `bridget_queue` et la CLI. Dépend de 159a.
- Chaque lot se livre et se mesure seul. 159b n'a d'effet que sur les types d'agents où 159a est ouvert.

### Cadre de menace

Identique à 156 (ADR 011, ADR 027) : la garde protège contre un agent qui agit par
ses outils Bridget. Un processus hostile du même compte Unix qui forge des trames
brutes reste hors modèle ; on ne le promet pas. Le daemon reste le seul point
d'application. Les droits passent par la garde commune de 156 (`guidance.rs`).

### Hors périmètre

- **Fils T3.** Le pont n'envoie pas `TurnState` (aucune occurrence dans `t3code.rs`) et il possède le regroupement de la session 114 (jusqu'à 8 messages par tour, `t3code.rs:3150`) et sa propre file de 64. Ils restent sur le chemin actuel, non éditable. Option 159c, à étudier : confier leur file à la file durable de T3 (`queued-run.cancel/edit/reorder`, `queued-message.promote-to-steer`, T3 `handlers.ts:262-293`).
- **`edit` (changer le texte).** Reporté : le texte fait partie de l'enveloppe idempotente de l'expéditeur (ADR 007) ; l'édition par un tiers fait dire à l'auteur ce qu'il n'a pas dit ; `cancel` puis nouvel envoi donne le même effet (question 3).
- Mode `auto` ; édition des messages déjà libérés ; file durable des messages libérés après redémarrage d'un porteur (comportement actuel) ; fédération entre machines ; `execution_queue` (dette notée, voir `research.md` R14).

## Scénarios utilisateur

### US1 - Retenir les messages d'un agent occupé (P1, lot A)

Un type d'agent est ouvert à la file du daemon. Un pair envoie un message ordinaire
à un agent de ce type qui est en plein tour. Le daemon enregistre l'envoi (accepté
comme aujourd'hui) mais ne le pousse pas. Quand le tour finit, le daemon libère le
plus ancien message retenu. Les messages arrivent dans l'ordre, un par tour. Sans
le réglage, tout se passe comme avant.

### US2 - Rien n'est perdu, rien n'est doublé (P1, lot A)

Le daemon ou le porteur redémarre pendant la rétention ou la libération. Aucun
message encore retenu n'est perdu ni livré deux fois. Un message déjà libéré suit
les garanties de toute remise : s'il était entre l'écriture du reçu `Seen` et
l'injection, il devient `indeterminate`, état explicite, jamais silencieux. Un message retenu qui
dépasse son échéance, ou dont la demande est close, est retiré avec un état
final et l'auteur est prévenu.

### US3 - Voir la file (P1, lot A)

`bridget_queue list` rend les messages `held` (éditables, dans l'ordre de
libération) et `released` (déjà libérés, non éditables), lus dans la base.

### US4 - Retirer un message retenu (P1, lot B)

Un parent retire un message retenu. L'état passe à `cancelled` dans la même
transaction que la prévenance de l'auteur et la clôture du suivi de réponse. Le
message n'est jamais livré. Un message déjà libéré donne `not_held`.

### US5 - Réordonner (P2, lot B)

Un parent place un message retenu avant un autre (ou à la fin). L'ordre de
libération change dans une transaction. Il tient face à une arrivée concurrente.

### US6 - Promouvoir en guidage (P1, lot B)

Un parent promeut un message retenu : le daemon le libère tout de suite en
guidage dans le tour en cours. Si le transport ne peut pas guider, il l'enfile
en file ordinaire : le message n'est jamais rejeté. Un porteur qui n'a pas
déclaré cette garantie reçoit un refus `steer_unavailable` et le message reste
retenu.

### US7 - Aucun changement pour l'existant (P1)

Sans le réglage, le chemin de remise, les octets canoniques et les suites de test
existantes ne changent pas. Les fils T3 ne changent pas. Une base 156 migre sans
perte.

## Exigences fonctionnelles

**Lot A - File retenue**

- **FR-001** : Le registre des types d'agents gagne un réglage booléen `held_queue` (défaut faux). Seuls les types qui le mettent à vrai utilisent la file retenue.
- **FR-002** : Un message est retenu si et seulement si : le type du destinataire a `held_queue` ; l'instance du destinataire a déclaré `turn_attest` à l'enregistrement (FR-023) ; c'est un envoi client idempotent sans intention (`queue` ou absente) ; son origine n'est ni `Human` ni `System` et il n'est ni alerte de fil, ni observation (`bridget-observation:`), ni incident délégué (`delegated-runtime-`), ni message d'une mission native ; le destinataire n'est pas un fil T3 ; la présence du destinataire est `busy`, ou il a déjà au moins un message retenu (l'ordre est conservé). Sinon, chemin actuel.
- **FR-003** : Un message retenu a la phase `held` dans `send_deliveries`, sans exécution liée et sans push. Le résultat rendu à l'expéditeur est celui d'aujourd'hui (admission acceptée). L'échéance de tour est posée à la libération (`stamp_turn_deadline_for_delivery`), pas à l'admission.
- **FR-004** : Une libération (`held` vers `dispatching`) se fait dans une seule transaction, et seulement si toutes les conditions suivantes tiennent : le repos du destinataire est **attesté** (FR-023) ; aucune réservation de tour n'est ouverte pour lui (FR-024) ; le contrôle n'est pas en pause et aucune reprise de pause n'attend pour cet agent (FR-026) ; l'enregistrement d'envoi n'a pas expiré ; si le message suit une demande (`reply=true`), la demande est encore ouverte et son échéance n'est pas passée (relue dans la transaction). Le plus petit rang part. Puis la trame `DeliverIdempotent` part par le chemin de reprise existant. Un arrêt entre les deux laisse une remise `dispatching` que la reprise relivre.
- **FR-005** : Déclencheurs de libération : repos attesté (`TurnState{in_progress: false}` véridique), enregistrement ou reconnexion de l'instance, fin d'exécution du destinataire, levée de la pause, et balayage périodique sur le tick des relances. Un déclencheur ne prouve pas à lui seul le repos : FR-004 le revérifie.
- **FR-006** : Le balayage retire de la file tout message retenu dont l'enregistrement d'envoi a expiré ou dont la demande suivie est close ou expirée. Le message passe à `cancelled` (raison `expired_in_queue` ou `request_closed_in_queue`), l'envoi d'origine devient `Rejected(<raison>)`, l'auteur est prévenu. Après chaque retrait validé, le daemon retire le suivi en mémoire (`pending_replies`) de ce message. Tant qu'un message est retenu, le destinataire ne reçoit aucun rappel (paliers 1 et 2) pour sa demande : il ne l'a pas reçue.
- **FR-007** : Chaque remise garde un rattachement durable à son destinataire logique (`recipient_agent`, identifiant d'agent résolu à l'admission). À chaque enregistrement d'un agent, le daemon rattache à la nouvelle instance les messages retenus de cet agent, même si les présences sont vides après un redémarrage du daemon. Une instance purgée orpheline ses messages retenus comme ses remises `dispatching` ; un agent qui ne revient pas voit ses messages retirés par le balayage à l'échéance.
- **FR-008** : Rang global `queue_rank`, monotone, attribué dans la transaction `Immediate` de l'écriture de la remise. Il fixe l'ordre de libération et de reprise (remplace `ORDER BY delivery_id`, `send_delivery.rs:397`). L'ordre persistant est l'ordre de validation des transactions.
- **FR-009** : `bridget_queue list` (156) lit la base : `held` (éditable, ordre de libération) et `released` (phase `dispatching`).
- **FR-010** : Migration v7 de la base d'idempotence : phases `held` et `cancelled`, colonnes `queue_rank`, `recipient_agent` et `turn_gate`, liens d'exécution conservés, `foreign_key_check` à zéro, rejouable. Le précontrôle de schéma passe à 7 dans la même modification. Le retour à un binaire antérieur est refusé par conception. **Avant la livraison du lot A**, la restauration d'une sauvegarde est répétée en isolation avec l'ancien binaire et ses données contrôlées (SC-006).
- **FR-011** : Aucune mesure de latence n'est supposée : le comportement du lot A est vérifié par une mesure avant / après de l'intervalle repos attesté vers début du message suivant (SC-004).
- **FR-022** : Le nombre de messages retenus par destinataire est plafonné par `queue_capacity` du type. Au-delà, l'envoi est refusé `queue_full` à l'admission (refus net, mémorisé comme les autres refus d'envoi).
- **FR-023** : Le signal de tour est véridique et réattesté. Un refus de remise n'envoie plus `TurnState false` quand un tour est en cours (aujourd'hui il l'envoie sans condition, `wrapper.rs:6224`) : le wrapper envoie l'état réel de la session. Il réattèste cet état à intervalle régulier (au plus 30 s). Un wrapper déclare ce comportement par le champ `turn_attest` de `Register` (défaut faux). La présence garde le début de la période occupée (`busy_period_start`, non remis à zéro par un nouveau `true`) et la date de la dernière attestation.
- **FR-024** : Réservation de tour durable. Une libération pose `turn_gate = 1` sur la remise. Tant qu'une remise de l'agent porte `turn_gate = 1`, aucune autre n'est libérée. La réservation se lève quand : le daemon observe `TurnState true` après la libération ; ou la remise sort de `dispatching` sans accusé (indéterminée, orpheline, refusée) ; ou la remise est accusée depuis plus de 30 s sans que l'agent soit occupé (le tour s'est terminé ou n'a pas eu lieu). Elle survit à un redémarrage du daemon.
- **FR-025** : Sortie de `busy` : un repos réattesté libère aussitôt. Si le wrapper ne réattèste plus (silence supérieur à trois périodes), les messages restent retenus jusqu'à la sortie forcée de `busy` par `release_stale_busy_turns`, comptée depuis `busy_period_start` : au plus 2730 s.
- **FR-026** : Classification de la pause (ADR 027). Une libération démarre du travail pour un agent que la pause vient d'interrompre : c'est un effet autonome. Nouvel effet `AutonomousEffect::HeldRelease`, différé pendant la pause. Après la levée, la reprise de l'exécution interrompue passe avant ; la libération n'a lieu qu'ensuite. Différence assumée avec l'état actuel, où la file mémoire du transport poursuit pendant la pause.

**Lot B - Édition**

- **FR-012** : `bridget_queue` accepte `action: "cancel" | "reorder" | "promote"` avec `target`, `message_id`, `request_id` (obligatoire), et pour `reorder` `before_message_id` (texte ou `null` pour la fin).
- **FR-013** : Seuls les messages `held` sont éditables. Un message libéré, accusé, orphelin, indéterminé ou déjà retiré donne `not_held`.
- **FR-014** : `cancel` : une transaction unique écrit la phase `cancelled`, le résultat terminal de l'envoi d'origine `Rejected(cancelled_in_queue)`, la notice d'auteur, la clôture du suivi de réponse et le reçu ; après le commit, le suivi mémoire est retiré (FR-006). Le message n'est plus jamais livré. Le résultat est final (`applied`) : jamais `outcome_unknown`.
- **FR-015** : `reorder` : une transaction re-numérote les seuls messages retenus du destinataire. Les valeurs de rang du groupe sont conservées, leur attribution change. Les deux messages doivent être retenus, du même destinataire. Une arrivée concurrente donne l'un de deux ordres valides selon l'ordre de validation ; le résultat persistant est celui que `list` montre.
- **FR-016** : `promote` : refusé hors tour actif (`no_active_turn`), refusé si le porteur n'a pas déclaré `queue_fallback` (`steer_unavailable`). Sinon une transaction libère le message (mêmes vérifications que FR-004 : demande ouverte, échéance, pause) et la trame de remise porte l'intention de guidage et le marqueur `steer-or-queue`. Un transport qui ne peut pas guider enfile le message en file ordinaire au lieu de le rejeter. **Garanties réduites, dites telles quelles** : après la libération, le message suit les garanties de toute remise. Si la file ordinaire du porteur est pleine, la remise est rejetée explicitement (indéterminée, sort connu de l'expéditeur par les voies existantes). Si Codex reçoit une erreur ambiguë alors que le guidage a été appliqué, il remet le message en file (`codex_app_server.rs:1724`) et un doublon est possible. Ces deux cas sont testés et documentés ; ils ne sont pas supprimés.
- **FR-017** : Reçu `queue_edits` complet, écrit dans la transaction de l'effet, conservé 7 jours. Même clé et mêmes paramètres : même résultat. Autres paramètres : `envelope_mismatch`. Les refus sont mémorisés ; pour réessayer il faut une nouvelle clé.
- **FR-018** : Droits : `cancel` : auteur du message, parent de la cible, humain attaché. `reorder` : parent, humain attaché. `promote` : droits de `steer` de 156 (garde obligatoire). Le destinataire et les pairs n'éditent pas (question 1). Pendant la pause, `promote` est refusé pour un agent ; `cancel` et `reorder` sont admis. Preuve enfant et client nu : refusés, comme en 156.
- **FR-019** : Un porteur déclare `queue_fallback` à l'enregistrement (champ optionnel, défaut faux). Un porteur ancien reste valable pour `cancel` et `reorder` mais pas pour `promote`, et n'ouvre pas la file retenue sans `turn_attest`.
- **FR-020** : Capacité client `QueueEditV1` pour les écritures de `bridget_queue`. Sans elle : `CapabilityNotNegotiated`.
- **FR-021** : `docs/reference-communication.md`, la skill Bridget et une ADR décrivent la nouveauté, le cadre de menace, le réglage `held_queue`, les garanties réduites, et la procédure de sauvegarde et de restauration avant la migration.

## Critères de succès

- **SC-001** : Réglage fermé : un envoi produit les mêmes octets, la même phase et la même poussée qu'avant (test d'égalité) ; les suites `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::` et transport restent vertes.
- **SC-002** : **Garantie réduite et honnête.** Pour un message retenu, tableau de pannes (daemon arrêté après la rétention, après la transaction de libération, avant et après le push ; porteur redémarré avec et sans message en cours ; daemon puis porteur redémarrés avec une nouvelle instance) : le message est retenu, libéré ou `cancelled`, jamais perdu en silence, jamais doublé. **Après la libération**, le message suit les garanties de toute remise : le porteur écrit `Seen` avant l'injection, et un arrêt entre les deux rend la remise `indeterminate` sans injection (`receipt_store.rs:147-160`). Ce cas est testé (point d'arrêt ajouté) et déclaré, pas supprimé.
- **SC-003** : Les messages d'un agent occupé arrivent dans l'ordre de leur rang, y compris après une réaffectation d'instance.
- **SC-004** : Intervalle repos attesté vers début du message suivant, mesuré avant et après sur un porteur Codex simulé, conditions identiques, consigné dans `validation/`. Un écart supérieur à 250 ms au 95e centile est signalé et décide du réglage par défaut.
- **SC-005** : **Corrigé.** Repos perdu : si le wrapper est vivant, libération au plus à la prochaine réattestation (30 s). Si le wrapper ne réattèste plus, libération à la sortie forcée de `busy` : au plus 2730 s après le début de la période occupée (`busy_period_start`). Les deux cas sont testés séparément.
- **SC-006** : Une base 156 migre sans perte, `foreign_key_check` à zéro, liens d'exécution conservés, ordre préservé ; la migration se rejoue sans effet ; le précontrôle accepte la version 7 ; un binaire antérieur refuse la base ; **la restauration de la sauvegarde avec l'ancien binaire est répétée en isolation et les données comparées avant la livraison du lot A**.
- **SC-007** : `cancel` : 100 % des messages retirés ne sont jamais livrés, y compris après arrêt du daemon à chaque point ; l'auteur reçoit une notice ; le suivi de réponse est clos en base et en mémoire ; aucun rappel n'est émis pour une demande retenue ; 0 interruption de tour.
- **SC-008** : `reorder` : l'ordre de libération observé est l'ordre persistant. Test des deux validations possibles ([A,B], A placé à la fin, C arrive) : B, A, C quand le déplacement valide d'abord ; B, C, A quand l'arrivée valide d'abord.
- **SC-009** : **Réduit.** `promote` : le message n'est pas perdu avant la libération ; sur un transport qui refuse le guidage (Claude, ACP, Codex sans tour actif), il est enfilé en file ordinaire. Saturation de la file ordinaire : rejet explicite et testé. Erreur ambiguë de Codex après application : doublon possible, testé et documenté.
- **SC-010** : Un appelant sans droit (pair, destinataire, client nu, preuve enfant) obtient un refus et n'écrit rien ; mêmes tests par `bridget_queue`, CLI et trames brutes (défense en profondeur).
- **SC-011** : Rejouer une édition avec les mêmes paramètres rend le même résultat ; avec d'autres : `envelope_mismatch`.
- **SC-012** : Un message ne démarre jamais deux tours par une seule réservation : refus de guidage en plein tour (aucun faux repos), accusé puis faux signal terminal, déclencheurs concurrents : au plus une libération par cycle de tour.
- **SC-013** : Pause : pause, interruption, fin de tour, aucune libération ; levée : reprise de l'exécution interrompue d'abord, libération ensuite.
- **SC-014** : Demande close ou échue juste avant une libération ou une promotion : le message n'est pas libéré, il passe à `cancelled`.
- **SC-015** : Daemon puis porteur redémarrés avec une nouvelle instance : les messages retenus sont retrouvés et libérés pour la nouvelle instance.

## Entités et hypothèses

**Entités**

- **Message retenu (`held`)** : remise acceptée mais pas encore poussée au porteur, en base du daemon, avec un rang.
- **Message libéré** : remise `dispatching` ou après : engagée, non éditable.
- **Réglage `held_queue`** : booléen par type d'agent dans le registre.
- **Édition** : `cancel`, `reorder`, `promote`, avec reçu `queue_edits`.

**Hypothèses (vérifiées en T002)**

- Le signal de tour des porteurs gérés n'est **pas** fiable aujourd'hui : un refus de remise envoie `TurnState false` sans condition (`wrapper.rs:6224`) et chaque `true` remet `busy_since` à zéro (`daemon.rs:3484`). Le lot A corrige ces deux points (FR-023, FR-025) et n'ouvre la file qu'aux instances qui déclarent `turn_attest`.
- Un envoi sans intention ne crée pas d'exécution liée (`admit_starting_message` ne concerne que `TriggerTurn` et `InterruptAndStart`, `daemon.rs:8828-8850`).
- `IdempotencyStore` peut clore le suivi de réponse dans sa transaction (il écrit déjà `requests` et le ledger à `begin_send_delivery_with_reply`).
- Parent, humain attesté, preuve enfant, origine fabriquée : définitions de 156.
- Les tests suivent la recette `umask 077` de la session 150.
- La livraison suit 147 à 156 : fichiers sur disque, sans relance de service.

## Questions pour l'utilisateur

1. **Le destinataire peut-il retirer ou réordonner les messages de sa propre file ?** Défaut prudent : non. Seuls l'auteur (retrait de ses messages), le parent de la cible et l'humain attaché éditent.
2. **La file retenue est-elle ouverte par défaut ?** Défaut prudent : non. Réglage `held_queue` par type d'agent, fermé au départ. On l'ouvre pour `codex` d'abord, on mesure (SC-004, SC-005), puis on étend. Les fils T3 restent hors lot.
3. **Faut-il `edit` (changer le texte d'un message retenu), réservé à l'auteur ?** Défaut prudent : non, reporté (voir « Hors périmètre »). L'auteur retire puis renvoie.
