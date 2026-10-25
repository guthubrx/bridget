# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/spec.md](../../../../../../specs/159-edition-file/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Résumé:
- Contexte historique avant159A: T3 Code permet de lister, retirer, éditer, réordonner un message en file et de le promouvoir en guidage, parce que sa file est durable côté serveur. Avant159A, la file Bridget vivait dans la mémoire de chaque porteur (transport géré, pont T3). Toute édition devient donc une saga répartie entre deux processus, et la contre-revue de la première conception a relevé 11 défauts de cette racine commune.
- Objectif: Le daemon retient en base les messages destinés à un agent occupé, puis les libère quand l'agent est libre. Retirer, réordonner et promouvoir un message retenu devient une transaction unique. Guidage 4 vers 5.
- Dépendances: SPEC-156 (garde de droits, origine fabriquée, `bridget_queue` en lecture, cadre de menace), SPEC-150, ADR 007, ADR 011, ADR 027, ADR 044.

<a id="extrait-2"></a>

## Extrait 2

Accord utilisateur du 2026-10-11 : être au moins aussi bon que T3
(`t3_queue_list/read/edit/cancel/reorder/promote_to_steer`). Critère visé :
Guidage, interruption, file 4 vers **5**
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`).

<a id="extrait-3"></a>

## Extrait 3

Aujourd'hui, `bridget_send` pousse le message au porteur dès son admission
(`DeliverIdempotent`, `daemon.rs:4112-4172`). Le porteur le range dans sa file
mémoire (Codex : `messages` et `steer`, `codex_app_server.rs:876-900` ; Claude et
ACP : `messages` ; pont T3 : `worker.queue`, borne 64, `t3code.rs:44`). Éditer
cette file exige de coordonner deux processus : retrait en mémoire, tombe durable,
accusé, réconciliation à la reprise, ordre divergent, réaffectation d'instance.
La première conception de 159 l'a fait et la contre-revue l'a refusée (11 points,
voir `adversarial-review-codex.md`).

<a id="extrait-4"></a>

## Extrait 4

- **Fils T3.** Le pont n'envoie pas `TurnState` (aucune occurrence dans `t3code.rs`) et il possède le regroupement de la session 114 (jusqu'à 8 messages par tour, `t3code.rs:3150`) et sa propre file de 64. Ils restent sur le chemin actuel, non éditable. Option 159c, à étudier : confier leur file à la file durable de T3 (`queued-run.cancel/edit/reorder`, `queued-message.promote-to-steer`, T3 `handlers.ts:262-293`).
- **`edit` (changer le texte).** Reporté : le texte fait partie de l'enveloppe idempotente de l'expéditeur (ADR 007) ; l'édition par un tiers fait dire à l'auteur ce qu'il n'a pas dit ; `cancel` puis nouvel envoi donne le même effet (question 3).
- Mode `auto` ; édition des messages déjà libérés ; file durable des messages libérés après redémarrage d'un porteur (comportement actuel) ; fédération entre machines ; `execution_queue` (dette notée, voir `research.md` R14).

<a id="extrait-5"></a>

## Extrait 5

Sans le réglage, le chemin de remise, les octets canoniques et les suites de test
existantes ne changent pas. Les fils T3 ne changent pas. Une copie de la base158B livrée migre sans
perte.

<a id="extrait-6"></a>

## Extrait 6

- **FR-001** : Le registre des types d'agents gagne un réglage booléen `held_queue` (défaut faux). Seuls les types qui le mettent à vrai utilisent la file retenue.
- **FR-002** : Un message est retenu si et seulement si : le type du destinataire a `held_queue` ; l'instance du destinataire a déclaré `turn_attest` à l'enregistrement (FR-023) ; c'est un envoi client idempotent sans intention (`queue` ou absente) ; son origine n'est ni `Human` ni `System` et il n'est ni alerte de fil, ni observation (`bridget-observation:`), ni incident délégué (`delegated-runtime-`), ni message d'une mission native ; le destinataire n'est pas un fil T3 ; la présence du destinataire est `busy`, ou il a déjà au moins un message retenu (l'ordre est conservé). Sinon, chemin actuel.
- **FR-003** : Un message retenu a la phase `held` dans `send_deliveries`, sans exécution liée et sans push. Le résultat rendu à l'expéditeur est celui d'aujourd'hui (admission acceptée). L'échéance de tour est posée à la libération (`stamp_turn_deadline_for_delivery`), pas à l'admission.
- **FR-004** : Une libération ordinaire (`held` vers `dispatching`) se fait dans une seule transaction, et seulement si toutes les conditions suivantes tiennent : le repos du destinataire est **attesté** (FR-023), ou la période occupée a atteint la sortie forcée explicitement prévue par FR-025 ; aucune réservation de tour n'est ouverte pour lui (FR-024) ; le contrôle n'est pas en pause et aucune reprise de pause n'attend pour cet agent (FR-026) ; l'enregistrement d'envoi n'a pas expiré ; si le message suit une demande (`reply=true`), la demande est encore ouverte et son échéance n'est pas passée (relue dans la transaction). Cette sortie forcée est une expiration, pas une preuve que le fournisseur est libre. Elle ne fabrique pas de nouvelle attestation. Le plus petit rang part. Puis la trame `DeliverIdempotent` part par le chemin de reprise existant. Un arrêt entre les deux laisse une remise `dispatching` que la reprise relivre. La promotion explicite relève de FR-016 et FR-018 : elle exige un tour actif au lieu du repos, avec l’exception humaine de pause prévue par 156. Ces exceptions ne changent pas les libérations automatiques de FR-026.
- **FR-005** : Déclencheurs de libération : repos attesté (`TurnState{in_progress: false}` véridique), enregistrement ou reconnexion de l'instance, fin d'exécution du destinataire, levée de la pause, et balayage périodique sur le tick des relances. Un déclencheur ne prouve pas à lui seul le repos : FR-004 le revérifie.
- **FR-006** : Le balayage retire de la file tout message retenu dont l'enregistrement d'envoi a expiré ou dont la demande suivie est close ou expirée. Le message passe à `cancelled` (raison `expired_in_queue` ou `request_closed_in_queue`), l'envoi d'origine devient `Rejected(<raison>)`, l'auteur est prévenu. Après chaque retrait validé, le daemon retire le suivi en mémoire (`pending_replies`) de ce message. Tant qu'un message est retenu, le destinataire ne reçoit aucun rappel (paliers 1 et 2) pour sa demande : il ne l'a pas reçue.
- **FR-007** : Chaque remise garde un rattachement durable à son destinataire logique (`recipient_agent`, identifiant d'agent résolu à l'admission). À chaque enregistrement d'un agent, le daemon rattache à la nouvelle instance les messages retenus de cet agent, même si les présences sont vides après un redémarrage du daemon. Une instance purgée orpheline ses messages retenus comme ses remises `dispatching` ; un agent qui ne revient pas voit ses messages retirés par le balayage à l'échéance.
- **FR-008** : Rang global `queue_rank`, monotone, attribué dans la transaction `Immediate` de l'écriture de la remise. Il fixe l'ordre de libération et de reprise (remplace `ORDER BY delivery_id`, `send_delivery.rs:397`). L'ordre persistant est l'ordre de validation des transactions.
- **FR-009** : `bridget_queue list` (156) lit la base : `held` (éditable, ordre de libération) et `released` (phase `dispatching`).
- **FR-010** : Migration v7 de la base d'idempotence : phases `held` et `cancelled`, colonnes `queue_rank`, `recipient_agent`, `turn_gate` et `acked_at`, liens d'exécution conservés, `foreign_key_check` à zéro, rejouable. Le précontrôle de schéma passe à 7 dans la même modification. Le retour à un binaire antérieur est refusé par conception. **Avant la livraison du lot A**, la restauration d'une sauvegarde est répétée en isolation avec l'ancien binaire et ses données contrôlées (SC-006).
- **FR-011** : Aucune mesure de latence n'est supposée : le comportement du lot A est vérifié par une mesure avant / après de l'intervalle repos attesté vers début du message suivant (SC-004).
- **FR-022** : Le nombre de messages retenus par destinataire est plafonné par `queue_capacity` du type. Au-delà, l'envoi est refusé `queue_full` à l'admission (refus net, mémorisé comme les autres refus d'envoi).
- **FR-023** : Le signal de tour est véridique et réattesté. Un refus de remise n'envoie plus `TurnState false` quand un tour est en cours (aujourd'hui il l'envoie sans condition, `wrapper.rs:6666`) : le wrapper envoie l'état réel de la session. Il réattèste cet état à intervalle régulier (au plus 30 s). Un wrapper déclare ce comportement par le champ `turn_attest` de `Register` (défaut faux). La présence garde le début de la période occupée (`busy_period_start`, non remis à zéro par un nouveau `true`) et la date de la dernière attestation.
- **FR-024** : Réservation de tour durable. Une libération pose `turn_gate = 1` sur la remise. Tant qu'une remise de l'agent porte `turn_gate = 1`, aucune autre n'est libérée. La réservation se lève quand : le daemon observe `TurnState true` après la libération ; ou la remise sort de `dispatching` sans accusé (indéterminée, orpheline, refusée) ; ou la remise est accusée depuis plus de 30 s sans que l'agent soit occupé (le tour s'est terminé ou n'a pas eu lieu). Elle survit à un redémarrage du daemon.
- **FR-025** : Sortie de `busy` : un repos réattesté libère aussitôt. La sortie forcée exige conjointement un silence de plus de90s depuis `last_attested_at` et une période occupée ayant atteint `notify_timeout_secs + 30 s` depuis `busy_period_start`. Un `true` frais maintient `busy` même après ce délai. Avec le défaut2700s et silence depuis le début, la sortie reste à2730s. Un délai configuré plus court ne permet jamais de sortir avant le silence requis. Cette expiration ne prouve pas le repos du fournisseur.
- **FR-026** : Classification de la pause (ADR 027). Une libération démarre du travail pour un agent que la pause vient d'interrompre : c'est un effet autonome. Nouvel effet `AutonomousEffect::HeldRelease`, différé pendant la pause. Après la levée, la reprise de l'exécution interrompue passe avant ; la libération n'a lieu qu'ensuite. Différence assumée avec l'état actuel, où la file mémoire du transport poursuit pendant la pause.

<a id="extrait-7"></a>

## Extrait 7

- **SC-001** : Réglage fermé : un envoi produit les mêmes octets, la même phase et la même poussée qu'avant (test d'égalité) ; les suites `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::` et transport restent vertes. Aucune nouvelle régression n'est admise. Les échecs préexistants exigent un arbitrage ROOT documenté ; leur présence ne constitue pas une preuve de suites vertes.
- **SC-002** : **Garantie réduite et honnête.** Pour un message retenu, tableau de pannes (daemon arrêté après la rétention, après la transaction de libération, avant et après le push ; porteur redémarré avec et sans message en cours ; daemon puis porteur redémarrés avec une nouvelle instance) : le message est retenu, libéré ou `cancelled`, jamais perdu en silence, jamais doublé. **Après la libération**, le message suit les garanties de toute remise : le porteur écrit `Seen` avant l'injection, et un arrêt entre les deux rend la remise `indeterminate` sans injection (`receipt_store.rs:147-160`). Ce cas est testé (point d'arrêt ajouté) et déclaré, pas supprimé.
- **SC-003** : Les messages d'un agent occupé arrivent dans l'ordre de leur rang, y compris après une réaffectation d'instance.
- **SC-004** : Intervalle repos attesté vers début du message suivant, mesuré avant et après sur un porteur Codex simulé, conditions identiques, consigné dans `validation/`. Un écart supérieur à 250 ms au 95e centile est signalé et décide du réglage par défaut.
- **SC-005** : Repos perdu : si le wrapper est vivant, libération au plus à la prochaine réattestation (30s). S'il ne réatteste plus, la sortie forcée exige à la fois plus de90s de silence et `notify_timeout_secs + 30 s` de période occupée. Le défaut donne2730s lorsque le silence commence au début du busy ; une attestation tardive ou une configuration courte peut reporter la sortie jusqu'au silence requis. Les deux cas sont testés séparément ; un `true` frais n'est jamais un repos.
- **SC-006** : Une copie de la base158B livrée migre sans perte, `foreign_key_check` à zéro, liens d'exécution conservés, ordre préservé ; la migration se rejoue sans effet ; le précontrôle accepte la version 7 ; un binaire antérieur refuse la base ; **la restauration de la sauvegarde avec l'ancien binaire est répétée en isolation et les données comparées avant la livraison du lot A**.
- **SC-007** : `cancel` : 100 % des messages retirés ne sont jamais livrés, y compris après arrêt du daemon à chaque point ; l'auteur reçoit une notice ; le suivi de réponse est clos en base et en mémoire ; aucun rappel n'est émis pour une demande retenue ; 0 interruption de tour.
- **SC-008** : `reorder` : l'ordre de libération observé est l'ordre persistant. Test des deux validations possibles ([A,B], A placé à la fin, C arrive) : B, A, C quand le déplacement valide d'abord ; B, C, A quand l'arrivée valide d'abord.
- **SC-009** : **Réduit.** `promote` : le message n'est pas perdu avant la libération ; sur un transport qui refuse le guidage (Claude, ACP, Codex sans tour actif), il est enfilé en file ordinaire. Saturation de la file ordinaire : rejet explicite et testé. Erreur ambiguë de Codex après application : doublon possible, testé et documenté.
- **SC-010** : Un appelant sans droit (pair, destinataire, client nu, preuve enfant) obtient un refus et n'écrit rien ; mêmes tests par `bridget_queue`, CLI et trames brutes (défense en profondeur).
- **SC-011** : Rejouer une édition avec les mêmes paramètres rend le même résultat ; avec d'autres : `envelope_mismatch`.
- **SC-012** : Un message ne démarre jamais deux tours par une seule réservation : refus de guidage en plein tour (aucun faux repos), accusé puis faux signal terminal, déclencheurs concurrents : au plus une libération par cycle de tour.
- **SC-013** : Pause : pause, interruption, fin de tour, aucune libération ; levée : reprise de l'exécution interrompue d'abord, libération ensuite.
- **SC-014** : Demande close ou échue juste avant une libération ou une promotion : le message n'est pas libéré, il passe à `cancelled`.
- **SC-015** : Daemon puis porteur redémarrés avec une nouvelle instance : les messages retenus sont retrouvés et libérés pour la nouvelle instance.

<a id="extrait-8"></a>

## Extrait 8

- Le signal de tour des porteurs gérés n'est **pas** fiable aujourd'hui : un refus de remise envoie `TurnState false` sans condition (`wrapper.rs:6666`) et chaque `true` remet `busy_since` à zéro (`daemon.rs:3484`). Le lot A corrige ces deux points (FR-023, FR-025) et n'ouvre la file qu'aux instances qui déclarent `turn_attest`.
- Un envoi sans intention ne crée pas d'exécution liée (`admit_starting_message` ne concerne que `TriggerTurn` et `InterruptAndStart`, `daemon.rs:8828-8850`).
- `IdempotencyStore` peut clore le suivi de réponse dans sa transaction (il écrit déjà `tracked_requests` et le ledger dans la transaction de `begin_send_delivery_with_guidance` (repère candidat : `send_delivery.rs:89–175`)).
- Parent, humain attesté, preuve enfant, origine fabriquée : définitions de 156.
- Les tests suivent la recette `umask 077` de la session 150.
- La livraison suit le pipeline déjà autorisé152–160 : commit, intégration et push, build signé, installation et activation conditionnelle de Bridget après sauvegardeSC006/T011 et contrôle d'absence de mission native non terminale. T3 reste protégé, sans relance explicite. `held_queue` reste fermé dans la livraisonA.

<a id="extrait-9"></a>

## Extrait 9

1. **Le destinataire peut-il retirer ou réordonner les messages de sa propre file ?** Défaut prudent : non. Seuls l'auteur (retrait de ses messages), le parent de la cible et l'humain attaché éditent.
2. **La file retenue est-elle ouverte par défaut ?** Défaut prudent : non. Réglage `held_queue` par type d'agent, fermé au départ. On l'ouvre pour `codex` d'abord, on mesure (SC-004, SC-005), puis on étend. Les fils T3 restent hors lot.
3. **Faut-il `edit` (changer le texte d'un message retenu), réservé à l'auteur ?** Défaut prudent : non, reporté (voir « Hors périmètre »). L'auteur retire puis renvoie.
