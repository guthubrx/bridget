# Recherche - Session159 file retenue par le daemon et édition

Date : 2026-10-11, refondue après la contre-revue (rondes 1 et 2) (`adversarial-review-codex.md`).
Source historique du2026-10-11 : checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget`
(base `1f2c661d`) et T3 `origin/main`. Ces repères ne sont pas une lecture fraîche
de la base livrée158B. Session156 livrée ; ses acquis restent réemployés.
**Base 159A confirmée** : source 158B e52a96240b6168bf4466712e1bda8852b4ecb43f livré ; HEAD de préparation b235998638b3eadf8f8edc0a6e323b762e95cab0. GO159A acquis par ROOT, implémentation A en cours ; held_queue reste fermé. Le candidat 158B 2124 reste historique ; aucune livraison 159A déduite.
Les consignes de prélecture avant GO ci-dessous restent historiques ; les réponses T002 et les reçus ciblés actuels sont dans validation. Aucune preuve de livraison159A finale déduite.
Prélectures fraîches du candidat158B :
/Users/moi/.cache/bridget-prelecture159-20261012/readiness159A-after158B-candidate.md
/Users/moi/.cache/bridget-plan159-native-evidence-20261012/T002-readonly.md
Leurs repères doivent être revalidés sur la base livrée avant le GO159A.
Un ancien PASS de réemploi ne vaut pas un gate frais.

## R0 - Cadre de menace

Repris de 156 R0 (ADR 011, ADR 027) : la garde protège contre un agent qui agit par
ses outils Bridget (MCP, CLI, pont T3, wrapper). Un processus hostile du même compte
Unix qui forge des trames brutes est hors modèle ; on ne le promet pas. Les tests de
trames brutes sont de la défense en profondeur.

## R1 - Parité avec T3 (lu dans T3 `origin/main`)

Source : `apps/server/src/mcp/toolkits/thread/tools.ts:80-137`, `handlers.ts:234-293`.

| Outil T3 | Sémantique T3 | Reprise |
|---|---|---|
| `t3_queue_list` | liste dans l'ordre de remise | `bridget_queue list` (156, lu dans la base) |
| `t3_queue_read` | lit 16 000 caractères | écarté : l'aperçu suffit |
| `t3_queue_edit` | remplace le texte | reporté (R15) |
| `t3_queue_cancel` | annule un message en file | `cancel` (R7) |
| `t3_queue_reorder` | `beforeRunId` ou `null` | `reorder` (R8) |
| `t3_queue_promote_to_steer` | `targetRunId` | `promote` (R9) |

T3 y arrive parce que ses éditions sont des commandes sur une file durable côté
serveur (`queued-run.cancel`, `queued-run.edit`, `queued-run.reorder`,
`queued-message.promote-to-steer`). La leçon pour Bridget : tenir la file dans un
seul magasin durable.

## R2 - Pourquoi l'édition chez le porteur a échoué (première conception)

La première conception éditait la file mémoire de chaque porteur. Les preuves :
- Codex : `QueueState { messages, steer, active, steering_open }` (`codex_app_server.rs:876-900`) ; `cancel_delivery` annule d'abord le tour actif (`:1081-1093`).
- ACP : `cancel_delivery` annule aussi le tour actif (`acp.rs:518-531`).
- Pont T3 : `worker.queue`, borne 64, retrait par `report_discard` qui rend la remise `indeterminate` (`t3code.rs:44`, `:1913-1922`, `:1855-1880`) ; registre `cancelled` en mémoire (`:1926-1929`).
- Reçu durable du porteur : `Seen` écrit avant l'injection, `Indeterminate` au rejeu (`receipt_store.rs:123-165`).
- Reprise relit toute remise `dispatching` (`send_delivery.rs:379-449`).

Il fallait une tombe durable, un accusé, une réconciliation, un ordre répliqué et une
promotion en deux temps. La contre-revue y a trouvé 11 défauts, tous issus de la
même cause : deux processus se partagent l'état de la file.

## R3 - Conception retenue : le daemon retient, le porteur exécute

Un message destiné à un agent occupé reste en base du daemon, dans la table qui
porte déjà sa saga : `send_deliveries`, phase `held`. Le porteur ne reçoit un
message qu'à la libération. Une fois libéré, le message est engagé et plus
éditable. Il n'est donc plus nécessaire de coordonner deux processus pour éditer.

Machine à états de `send_deliveries.phase` après v7 :

| Phase | Sens | Éditable |
|---|---|---|
| `held` | acceptée, pas encore poussée | oui |
| `dispatching` | poussée au porteur, pas accusée (existant) | non |
| `acked` | injectée (existant) | non |
| `indeterminate` | sort inconnu (existant) | non |
| `orphaned` | destinataire purgé (existant) | non |
| `cancelled` | retirée avant libération (nouveau) | non, terminal |

Transitions : `held → dispatching` (libération), `held → cancelled` (retrait,
échéance, demande close), `held → orphaned` (présence purgée), `dispatching → …`
(existant, inchangé).

## R4 - Règle de rétention

Un message est retenu si et seulement si (FR-002) :
1. le type du destinataire a `held_queue` dans le registre (`AgentDefinition`, `registry.rs:49-79` ; champ à ajouter avec `#[serde(default)]`, comme `queue_capacity` et `notify_timeout_secs`) ;
2. envoi client idempotent sans intention ;
3. origine ni `Human` (un message humain interrompt, `codex_app_server.rs:866`) ni `System` ; ni alerte de fil, observation, incident délégué ; ni message de mission native (`delegation_store.for_mission`, `daemon.rs:4190-4195`) ;
4. destinataire non T3 (`presence.transport != "t3code"`) et instance ayant déclaré `turn_attest` (R5.1) ;
5. présence `busy` (signal `TurnState`) ou messages déjà retenus pour ce destinataire.

Dans tous les autres cas : chemin actuel, inchangé. Le règlement 5, deuxième terme,
conserve l'ordre : tant qu'il reste un message retenu, un nouveau passe derrière.

Plafond : `queue_capacity` du type (`registry.rs:67`, valeur par défaut
`DEFAULT_QUEUE_CAPACITY`). Au-delà, l'envoi est refusé `queue_full` par
`reject_idempotent_send` (refus net à l'admission au lieu du `DeliveryRejected`
asynchrone d'aujourd'hui).

## R5 - Libération

**Constats de la ronde 2, vérifiés dans le code**
- Le signal de tour n'est pas un signal de fin de tour. Un refus de remise envoie `TurnState{in_progress: false}` sans condition (`wrapper.rs:6666`), même si un tour est en cours (par exemple un guidage refusé). Un « faux repos » ouvrirait une libération en plein tour.
- Chaque `TurnState true` remet `busy_since` à zéro (`daemon.rs:5240`) ; l'obsolescence se mesure depuis ce `busy_since` (`daemon.rs:814-823`). Une suite de `true` empêche donc toute sortie forcée, et rien ne réattèste le repos.
- Après l'accusé d'une remise, la ligne n'est plus `dispatching`, et la présence n'est pas encore `busy` : une seconde libération passerait.
- Après un redémarrage du daemon, `presences` est vide (`daemon.rs:3150`) et la migration de remises d'une ancienne instance suppose de la retrouver (`daemon.rs:6155-6176`, `:6311-6325`).

**Conception corrigée**
1. **Signal véridique** (FR-023). Le wrapper envoie l'état réel de sa session après un refus (pas `false` en dur) et réattèste cet état toutes les 30 s au plus. Il déclare `turn_attest` dans `Register` (champ optionnel). Seules les instances qui le déclarent reçoivent la file retenue ; les autres gardent le chemin actuel. La présence gagne `busy_period_start` (début de la période occupée, non remis à zéro par un nouveau `true`) et `last_attested_at`.
La minuterie Instant du heartbeat existant est vérifiée à chaque passage de la boucle, même sous trafic continu. Elle déclenche une réattestation vraie périodique (30 s au plus), sans nouveau scheduler. Pour toute instance turn_attest, la sortie forcée utilise busy_period_start, même si held_queue est fermé. Les instances anciennes non attestées gardent le calcul legacy sur busy_since. La borne par défaut de 2730 s vaut notify_timeout_secs = 2700 s + grâce de 30 s ; notify_timeout_secs reste configurable.
2. **Réservation de tour durable** (FR-024). Colonne `turn_gate` sur `send_deliveries`. La libération la pose, dans la même transaction que le passage à `dispatching`. Une libération ne se fait que si aucune remise de l'agent ne porte `turn_gate = 1`. Levée : `TurnState true` observé après la libération ; sortie de `dispatching` sans accusé (indéterminée, orpheline, refusée) ; accusé depuis plus de 30 s sans que l'agent soit occupé. Durable : survit à un redémarrage du daemon. Garantie : un cycle de tour, au plus une libération.
3. **Conditions de libération** (FR-004), revérifiées sous le verrou State et dans la transaction SQL : repos attesté en mémoire, ou expiration forcée de la période occupée selon FR-025, stable jusqu’au commit ; pas de réservation ; pas de pause et pas de reprise de pause en attente pour l'agent ; saga non expirée ; demande suivie ouverte et dans son échéance (lecture de `tracked_requests`, même connexion que l'écriture de `begin_send_delivery_with_reply`). L'expiration ne prouve pas le repos du fournisseur. Elle conserve la dernière date d'attestation réelle et reste distincte d'un vrai signal `false`.
Garde de libération (décision ROOT) : conserver le mutex State de la lecture du repos jusqu’au commit de Held vers Dispatching. La même transaction Immediate d’IdempotencyStore lit control_state et control_pause_interruptions dans config.db_path, partagé par les magasins existants. Préserver l’ordre de verrous existant ; aucun writer socket ni autre connexion SQL dans cette transaction. La libération est linéarisée au commit. Le push différé se fait hors verrou ; une pause validée après ce commit ne reprend pas une remise déjà libérée. Pas de table miroir. Les oracles concurrents de T007 déjà prévus couvrent l’ordre pause/TurnState true ; aucun résultat runtime nouveau n’est annoncé.
4. **Pause** (FR-026, ADR 027). La libération démarre du travail pour un agent que la pause vient d'interrompre ; elle est classée effet autonome `HeldRelease` (nouvelle variante de `AutonomousEffect`, `referent_control.rs:105-133`). L'ADR 027 garde admises « la reprise des remises déjà acceptées et la réconciliation de flotte » (décision 2) : un message retenu n'a été remis à aucun porteur, ce n'est pas une reprise. À la levée, la reprise de l'exécution interrompue (`resume_executions_after_pause`, `daemon.rs:9675+`) passe avant ; une libération n'est faite que si aucune ligne `control_pause_interruptions` en attente ne vise l'agent. Différence assumée : aujourd'hui la file mémoire d'un transport poursuit pendant la pause.
5. **Rattachement durable** (FR-007). Colonne `recipient_agent` (identifiant d'agent résolu à l'admission, `routing_agent_id`). À chaque enregistrement d'un agent (`daemon.rs:6311` et suivants), `adopt_held_for_agent(agent_id, instance_id)` rattache à la nouvelle instance les lignes `held` de cet agent, sans passer par `presences`. Rempli à la migration pour les lignes existantes par `json_extract(message_bytes, '$.to')`.

**Opération.** Transaction `Immediate` de `IdempotencyStore` : vérifier les conditions 2 et 3 par requêtes, sélectionner la ligne `held` de plus petit `queue_rank`, la passer à `dispatching` avec `turn_gate = 1`, rendre `SendDelivery`. Le daemon construit la trame par `defer_idempotent_delivery` (`daemon.rs:4112-4172`). Un arrêt entre la transaction et l'envoi laisse une remise `dispatching` relivrée par la reprise (`daemon.rs:4175-4230`).

**Déclencheurs.** Repos attesté, enregistrement ou reconnexion, fin d'exécution du destinataire, levée de la pause, tick des relances. Aucun ne prouve seul le repos : les conditions ci-dessus sont revérifiées.

**Signal perdu, deux cas distincts (SC-005 corrigé)**
- Wrapper vivant : la prochaine réattestation (≤ 30 s) rétablit le repos et libère.
- Wrapper muet : la sortie forcée exige plus de90s depuis la dernière attestation ET une période occupée d'au moins `notify_timeout_secs + 30 s`. Les deux horloges restent distinctes. Avec notify_timeout_secs=1, un `true` à30s interdit une sortie à31s et à120s ; elle devient possible à121s. Le défaut2700s reste à2730s pour le silence depuis le début. Une attestation tardive reporte la sortie ; aucune borne absolue depuis le seul début ne doit être promise. L'expiration conserve la date d'attestation, ne prouve pas le repos et ne change pas le chemin legacy.

**Mise en file d'un message déjà libéré.** Hors de ce lot : il suit les garanties de toute remise (R9, SC-002).

## R6 - Rang et migration v7

- Colonnes ajoutées par la v7 : `queue_rank`, `recipient_agent` (R5.5), `turn_gate` et `acked_at` (R5.2). L'âge d'un ACK ne peut pas être déduit de issued_at, expires_at ou d'une attestation. La saga actuelle ne conserve aucune date d'ACK. `acked_at` est donc écrit au premier ACK dans sa transaction, puis conservé sur rejeu. Les anciens ACK migrés gardent NULL et turn_gate=0 ; aucune date n'est inventée. Cette donnée rend la grâce de 30 s durable sans nouveau magasin ni compteur.
- Rang global durable : `queue_rank INTEGER PRIMARY KEY AUTOINCREMENT`, `delivery_id UNIQUE NOT NULL` comme cible des FK existantes. L’INSERT omet queue_rank ; SQLite conserve sa séquence durable après purge. Les anciens rowid sont copiés à la migration. Aucun nouveau compteur ni table métier. L’admission réemploie begin_send_delivery_with_guidance sous transaction Immediate. Une seule copie de l’ordre.
- Réaffectation (`reassign_dispatching_deliveries`, `send_delivery.rs:214-245`) : met à jour en place l'instance ; la clause `phase = 'dispatching'` devient `phase IN ('dispatching','held')`. Idem pour `orphan_dispatching_for_instance` (`send_delivery.rs:585-665`). Le rang ne change pas.
- Reconstruction de table, comme la v4 (`idempotency.rs:709-806`). Versions prises : 2, 3, 4, 5, 6. Prochaine : **7**.
- **Piège** : `send_delivery_execution_links` référence `send_deliveries(delivery_id)` avec `ON DELETE CASCADE` (`idempotency.rs:843`). Un `DROP TABLE` sous `foreign_keys=ON` supprime implicitement ses lignes. La migration copie les liens avant le `DROP` et les réinsère après le renommage, dans la même transaction ; `PRAGMA foreign_key_check` à zéro ; `foreign_keys` reste activé (règle de la v4).
- **Précontrôle** : `MAX_IDEMPOTENCY_SCHEMA_VERSION = 6` (`store_schema.rs:4`) refuse toute base de version supérieure, « aucune migration » (`:30-45`). La même modification relève la constante à 7. Test : le précontrôle accepte 7 et refuse 8.
- **Retour arrière** : un binaire antérieur refuse une base v7 (échec fermé, voulu). Pas de rétrogradation automatique. Procédure : `.backup` de la base avant la première ouverture par le binaire neuf ; retour = restauration de cette copie. À écrire dans l'ADR et la doc (T011, T012).
- Lignes existantes : `queue_rank` = ancien `rowid`. Remises sans parent `idempotency_records` : écartées avec `warn!` comme en v4.
- Index : `idx_send_deliveries_operation` (unique), `idx_send_deliveries_kind_key`, index non unique existant `idx_send_deliveries_recipient_phase`, étendu à `(recipient_instance_id, phase, queue_rank)` sans second index recipient.

## R7 - `cancel` : une transaction

Dans la connexion de `IdempotencyStore`, une transaction `Immediate` :
1. reçu : lecture de `queue_edits` pour la clé (rejeu ou `envelope_mismatch`) ;
2. `UPDATE send_deliveries SET phase='cancelled' WHERE delivery_id=? AND phase='held'` ; zéro ligne touchée : refus `not_held` (mémorisé) ;
3. `idempotency_records` de l'envoi passe à `terminal`, `Rejected(cancelled_in_queue)` (précédent `Orphaned`, `send_delivery.rs:585-665`) ;
4. ligne `orphan_emitter_notices` (`idempotency.rs:835-841`) ;
5. fermeture du suivi de réponse si `reply=true` (la connexion écrit déjà `tracked_requests` et le ledger dans la transaction de `begin_send_delivery_with_guidance`, repère candidat `send_delivery.rs:89–175`) ;
6. insertion du reçu (état `applied`).

**Après le commit** : le suivi de réponse vit aussi en mémoire (`st.pending_replies`, lu par les relances, `daemon.rs:6896+`, et retiré à l'annulation, `daemon.rs:18146`). Fermer le suivi en SQL ne le retire pas. Le daemon retire donc l'entrée en mémoire après chaque commit de retrait (`cancel`, balayage). Un ensemble en mémoire des identifiants retenus (reconstruit depuis la base au démarrage) fait ignorer les paliers 1 et 2 pour une demande retenue : le destinataire ne la connaît pas encore. Le palier 3 (échec notifié à l'émetteur) et l'expiration restent actifs ; ils ferment la demande, et le balayage retire le message retenu.
Tout ou rien. Pas d'exécution liée à écrire : un envoi sans intention n'en crée pas (`admit_starting_message` ne vaut que pour `TriggerTurn` et `InterruptAndStart`, `daemon.rs:8828-8850`). Les magasins ouvrent le même fichier config.db_path par des connexions distinctes (daemon.rs:4806–4818). La transaction de la saga peut lire les tables de pause et de reprise dans sa propre connexion. Elle ne doit pas appeler un writer d’une autre connexion pendant cette transaction. L’édition sans intention ne crée aucun lien d’exécution.

## R8 - `reorder`

API : `message_id` + `before_message_id` (ou `null` = fin), comme T3. Transaction : lire les lignes `held` du destinataire dans l'ordre de rang, calculer la nouvelle séquence, ré-affecter les valeurs de rang existantes du groupe à cette séquence. Les rangs des autres messages ne bougent pas. Une arrivée concurrente omet queue_rank ; SQLite attribue un nouveau rang AUTOINCREMENT dans sa transaction, sérialisée avant ou après. La permutation passe par des rangs provisoires négatifs dans la même transaction unique pour éviter les collisions de clé primaire ; ils ne sont jamais publiés, sans CHECK positif qui les interdit. [A,B], A placé à la fin, C arrive : si le déplacement valide d'abord, l'ordre est B, A, C ; si l'arrivée valide d'abord, il est B, C, A. Les deux sont valides. L'ordre persistant est l'ordre de validation, celui que la base montre et que la libération suit ; le test vérifie les deux cas (T019). Coût de réordonnancement : O(m log n) pour les écritures indexées, m ≤ queue_capacity ; aucune latence mesurée.

## R9 - `promote`

- Pré-conditions : identité attestée et droits/gardes de `Steer` de 156 ; phase `held` ; destinataire `busy` avec tour actif recontrôlé (sinon `no_active_turn`) ; porteur ayant déclaré `queue_fallback` (sinon `steer_unavailable`) ; aucune réservation de tour ; saga non expirée ; demande ouverte et non échue si `reply=true`. Ces faits sont recontrôlés dans la transaction. Le tour actif remplace le repos exigé pour une libération ordinaire. Un agent est refusé pendant la pause. L’humain attaché attesté garde l’exception de guidage de 156, sans nouvelle autorité issue du message.
- Effet : une transaction passe la ligne `held` à `dispatching` et pose la réservation de tour ; la trame de remise porte `intent = SteerCurrent` et la référence `steer-or-queue`. L'enveloppe stockée et le canon ne changent pas.
- **Ce que la promotion garantit** : le message n'est pas perdu *avant* la libération (il est en base). Un transport qui reçoit `SteerCurrent` avec `steer-or-queue` et ne peut pas guider (pas de tour actif, `steering_open` faux, fournisseur sans guidage) enfile le message en file ordinaire au lieu de le rejeter.
- **Ce qu'elle ne garantit pas** (ronde 2, point 5, vérifié) :
  - la file ordinaire du porteur peut être pleine : Claude rejette `file Claude pleine` (`claude_stream_json.rs:318-324`), Codex et ACP aussi. Le rejet rend la remise `indeterminate`, sort connu mais message non livré ;
  - Codex remet le message en file sur *toute* erreur du guidage, y compris quand le guidage a été appliqué mais que la réponse est une erreur (`codex_app_server.rs:1724-1740`) : doublon possible ;
  - le porteur n'accepte pas durablement : il écrit `Seen` avant d'injecter (R17).
- Panne entre la transaction et le push : la reprise relivre depuis `message_bytes` (intention d'origine : aucune), en file ordinaire. Dégradation sans perte avant la remise au porteur.
- Les deux cas ci-dessus sont testés et documentés (T020, T021), pas supprimés. Effet : non confirmé (156).

## R10 - Lecture

`list` = une requête sur `send_deliveries` : `held` (ordre de rang, éditable) et `dispatching` (`released`). La lecture list et la reprise dispatching utilisent toutes deux queue_rank ; la reprise garde son filtre de phase. Plus de snapshot du porteur, plus de source « inconnue ». Un message `held` n'a pas de dépendance au porteur : il survit à son redémarrage.

## R11 - Messages jamais retenus ni éditables

Origine humaine ou système, alertes de fil, observations, incidents délégués, missions natives, fils T3, intentions `steer` et `interrupt` (156), types d'agents sans `held_queue`. Un envoi qui n'est pas retenu ne peut pas être édité : `not_held`.

## R12 - Droits (extension de la matrice de 156)

| Action | Humain attaché | Parent de la cible | Auteur du message | Destinataire | Pair |
|---|:-:|:-:|:-:|:-:|:-:|
| `cancel` | oui | oui | oui (le sien) | non (question 1) | non |
| `reorder` | oui | oui | non | non (question 1) | non |
| `promote` | oui | oui | non | non | non |

`promote` suit les droits et gardes de `Steer` de 156 : refus pendant la pause pour un agent, exception pour l’humain attaché attesté. L’identité vient de la connexion validée, pas de `from=human` ni d’une référence. Cette commande humaine explicite n’est pas une libération autonome : la pause et la priorité de reprise continuent à bloquer les libérations automatiques `HeldRelease`, pas cette exception. Si le tour est réellement fermé, `no_active_turn` reste applicable. `cancel` et `reorder` restent admis pendant la pause : ils ne créent pas de travail. Preuve enfant (ADR 044), client nu `from=human`, identité non attestée : refus.

## R13 - Complexité

Rétention : O(log n) (index existant non unique `idx_send_deliveries_recipient_phase`, étendu à `(recipient_instance_id, phase, queue_rank)` ; pas un deuxième index recipient). Libération : O(log n), plus les lignes bornées du destinataire et les contrôles de suivi. Balayage : O(n + B + h(log n + log r) + L + P), n = remises persistantes parcourues, h = messages retenus, r = demandes, B = octets des enveloppes, L = dépôts liés et P = suivis mémoire. Les notices effectivement committées sont regroupées dans un HashSet existant puis pending_replies est filtré une seule fois, même si une erreur tardive suit un COMMIT réussi ; aucun index global de phase ajouté. `cancel` : O(log n), plus les écritures et contrôles de suivi. `reorder` : O(m log n) avec écritures indexées. `list` : O(log n + k), k ≤ 200. Garde : O(1).

## R14 - Évaluation : file tenue par le daemon (Art. XIX), preuves et coûts

**Peut-on réutiliser `execution_queue` ?** Non.
- La table existe (`execution_store.rs:1555`, colonnes `submission_id`, `target_agent`, `priority`, `enqueued_at`) et `take_next_submission` retire le prochain de façon atomique (`:456-500`).
- Mais aucun code de production ne l'appelle : seuls des tests (`tests/work_submission_test.rs`, `tests/execution_store_test.rs`). Le chemin historique `Send` y écrit les `QueueOnly` sans jamais les remettre (`daemon.rs:17634-17660`).
- Elle vit dans ExecutionStore. Ce magasin ouvre le même fichier SQLite par une connexion distincte (daemon.rs:4806–4818). La libération peut lire les tables de pause dans la transaction de la saga. La file d’exécution garde une autre responsabilité métier : elle représente les exécutions, tandis que held représente une remise sans intention. Aucun deuxième writer SQL n’est appelé sous cette transaction.
- Elle n'a ni échéance, ni lien à la saga, ni instance destinataire.
- Elle sert de plus à un autre contrat (`work_submissions`, exécutions). La mêler serait une dette.
- **Ce qu'il faut** : une phase dans `send_deliveries` (la saga existe déjà, avec `message_bytes`, instance, échéance, reprise, orphelinage).

**Ce que coûte le changement**, surface par surface :

| Surface | Preuve | Effet | Décision |
|---|---|---|---|
| Fils T3 : pas de signal de tour | aucune occurrence de `TurnState` dans `t3code.rs` ; le pont interroge T3 toutes les 3 s (`DEFAULT_POLL`, `t3code.rs:36`) | le daemon ne sait pas si un fil est occupé | hors lot ; option 159c |
| Regroupement de la session 114 | `BATCH_BOUND = 8`, `BATCH_BODY_BOUND`, `batch_family` (`t3code.rs:50-53`, `:3150`) | libérer un message par tour détruirait le regroupement (23 tours au lieu d'un) | hors lot (T3 seulement) |
| Remise en cours de tour (session 129) | `deliverable_while_busy` (`t3code.rs:3126`) | chemin du pont T3, séparé | inchangé |
| Échéance de tour | `stamp_turn_deadline_for_delivery` (`daemon.rs:7320-7345`) | posée à la libération, comme la session 112 | cohérent |
| Suivi de réponse et délais | `track_reply_cycle` à l'admission ; causes d'écart du pont (`t3code.rs:1855-1912`) | une demande peut expirer pendant la rétention | balayage de parité (T007) |
| Exécution liée | `admit_starting_message` (`daemon.rs:8828-8850`) | seulement `TriggerTurn` / `InterruptAndStart`, jamais retenus | aucun changement |
| Reprise | `schedule_idempotent_delivery_recovery` (`daemon.rs:4175-4230`) | les lignes `held` ne sont pas rendues par `dispatching_deliveries_for_instance` (filtre de phase) ; libération par déclencheur | T004, T006 |
| Latence | libération dans le gestionnaire de `TurnState` (`daemon.rs:17812`) : une transaction SQLite + une écriture de socket, contre un transfert en mémoire dans le porteur | un saut de plus ; l'écart est à mesurer, pas à supposer | SC-004 |
| Signal de tour | `wrapper.rs:6666` envoie `false` sur un refus ; `daemon.rs:5240` remet `busy_since` à zéro | faux repos ; sortie forcée impossible avec des `true` répétés | signal véridique et réattesté, `busy_period_start`, `turn_attest` (R5, T006) |
| Signal perdu | `release_stale_busy_turns` (`daemon.rs:3496-3520`) | retard borné par la réattestation (30 s) ; si muet, délai configurable `notify_timeout_secs + 30 s`, soit 2730 s par défaut, pour les instances turn_attest | SC-005 corrigé |
| Porteur ancien | la libération utilise la trame `DeliverIdempotent` existante, mais la rétention exige un signal de tour véridique | non retenu sans `turn_attest` ; chemin actuel | `turn_attest` ; `queue_fallback` pour `promote` seulement |
| Base de données | `store_schema.rs:4`, migration à sens unique | pas de retour arrière | sauvegarde obligatoire (R6) |
| Quotas | `queue_capacity` du type | `queue_full` à l'admission | T005 |
| Dette `execution_queue` / `QueueOnly` | voir plus haut | non traité | noté |

**Verdict** : la refonte du chemin de remise par défaut en une session est trop
risquée (reprise, échéances, suivi de réponse, ordre, mesure de latence, tous à la
fois). Découpage retenu : 159a (la file retenue, réglage fermé par défaut, mesurable),
puis 159b (les éditions, de simples transactions). Les fils T3 sortent du périmètre
(159c).

## R15 - `edit` reporté

- Le texte appartient à l'enveloppe idempotente de l'expéditeur (`canonical_bytes`, `daemon.rs:8654`, ADR 007) et au ledger d'émission (`send_delivery.rs:109-111`).
- La remise garde `from` : l'édition par un tiers met des mots dans la bouche de l'auteur.
- `cancel` puis nouvel envoi par l'auteur donne le même effet ; il perd seulement sa place.
- Si l'utilisateur le veut (question 3) : `edit` réservé à l'auteur, nouvel événement de ledger, reçu complet. Hors de cette liste de tâches. Avec la file retenue, il devient une transaction de plus.

## R16 - Option 159c pour les fils T3

Deux pistes, non engagées :
1. Faire émettre `TurnState` au pont et déplacer le regroupement (famille de messages, bornes) du pont vers la libération. Refonte lourde.
2. Confier à T3 sa file : le pont envoie les messages avec `dispatchMode: queue_after_active` (déjà posé, `t3code_contract_v2.rs:255-272`) et édite par les commandes `queued-run.*`. Reste à vérifier : correspondance `messageId` Bridget et `queuedRunId` T3, accès de la commande par le RPC v2 du pont, compatibilité avec la saga d'accusé.

## R17 - Garantie après libération : honnêteté

- Le porteur écrit `Seen` avant d'injecter (`receipt_store.rs:147-160`). Un arrêt entre l'écriture et l'injection donne `Indeterminate` au rejeu, sans injection (`wrapper.rs:757-776`). Le message est alors non livré et son état est explicite (`indeterminate`).
- La file retenue réduit l'exposition : seul un message à la fois est chez le porteur. Elle ne la supprime pas. Une reprise durable du porteur (rejouer un `Seen` non accusé) changerait la sémantique d'injection de toutes les remises : hors périmètre.
- Les promesses du lot B sont donc : avant la libération, ni perte ni doublon ; après, garanties de toute remise. `applied` dans le reçu d’édition atteste le commit de libération ; `effect: "unconfirmed"` ne prouve pas une consommation par le modèle. Le daemon ne refait pas l’effet au rejeu et ne fabrique ni ACK ni reçu d’injection. Le repli peut être rejeté si la file ordinaire est pleine ; une erreur ambiguë de Codex après application peut créer un doublon, conformément R9. Les critères SC-002 et SC-009 sont écrits ainsi, avec le point d'arrêt « entre `Seen` et l'injection » dans le tableau de pannes (T010).

## Vérifications ouvertes (levées par T002)

1. Quels modes de porteurs gérés envoient `TurnState` et à quels événements (`wrapper.rs:4757`, `:5403`, `:6103-6224`), y compris les modes interactifs ; comment le wrapper connaît l'état réel de la session (`is_busy`) ; où attacher la réattestation (cadence du battement) ; quels types d'agents peuvent déclarer `turn_attest`.
2. Période du tick des relances (`collect_reminder_actions`) et possibilité d'y attacher le balayage.
3. Causes d'écart du pont à reproduire pour les messages retenus, et ce que le daemon fait aujourd'hui à la réception de `DeliveryIndeterminate` pour un message `reply=true` (`t3code.rs:1855-1912`, `daemon.rs:16395-16420`).
4. Un envoi sans intention ne crée aucune exécution liée (confirmer `admit_starting_message`).
5. `IdempotencyStore` peut fermer le suivi de réponse dans sa transaction (table `tracked_requests`, ledger).
6. Les porteurs gérés regroupent-ils des messages (`BatchedDelivery`, `deliver_batch_to_interactive`) ? Si oui, la libération « un par tour » les affecte.
7. Forme finale de `guidance.rs` et de `bridget_queue list` livrés par 156.
8. Colonnes de `tracked_requests` à relire dans la transaction de libération (`state`, `deadline_at`) et lien message vers demande.
9. Où `resume_executions_after_pause` et la levée de la pause peuvent déclencher la libération ; contenu de `control_pause_interruptions` pour décider « reprise en attente ».
10. Résolution de l'identifiant d'agent à l'admission (`routing_agent_id`, `router.get_agent`) et correspondance avec `final_agent_id` à l'enregistrement.

Décision ROOT du 12 octobre : AUTOINCREMENT garantit les admissions monotones même après purge, au prix d’un surcoût CPU/E/S SQLite non mesuré ici, nécessaire à FR008. delivery_id UNIQUE NOT NULL conserve la cible des FK. Le mécanisme sqlite_sequence est interne à SQLite, pas un compteur métier nouveau. [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html), [SQLite Foreign Keys](https://www.sqlite.org/foreignkeys.html).

Portée de SC001 : égalité des octets canoniques du message et de la remise, de la phase et de la poussée du chemin d’envoi avec held_queue fermé. FR023 exige séparément le flux auxiliaire nouveau de réattestation vraie ; cette exigence ne promet pas l’identité des octets de tout le flux auxiliaire. busy_since legacy reste réservé aux instances sans turn_attest. L’exigence de suites vertes et l’interdiction de nouvelle régression sont conservées, sans dérogation implicite.

Les lectures control_state et control_pause_interruptions sont exécutées après BEGIN IMMEDIATE, dans cette même transaction. En cas de SQLITE_BUSY, la remise reste held : aucun refus, aucune libération ni push ; le prochain déclencheur réessaie. L’attente SQL peut durer busy_timeout (2 s) sous State ; ce risque de latence sera mesuré, sans nouvelle garantie. Les chemins TurnState et pause prennent ce même mutex ; les cas concurrents T007 prévus vérifient l’ordre.

## Vérifications levées — T002

Les dix réponses avec repères source sont conservées intégralement dans validation/T002-readonly.md. Les treize sources ont été attestées identiques dans validation/T002-source-correspondence159A.json avant le raccord du worker test-support. La seule modification métier absente de cette préparation est le raccord test-only T908 sous feature ; aucun fait mémoire/pause/admission de T002 changé.

# T002159A — repères frais, lecture seule

Date UTC : 2026-10-12T13:10:03.988849+00:00. Base158B déclarée par ROOT : candidat daemon2124b1d0 ; aucune livraison/push vérifiés ici. Aucun test, Cargo, service, base de données ou commande Git exécuté. Aucun code WT modifié. Les dix axes ci-dessous répondent aux questions ouvertes de T002 ; ils ne valent ni cochage T002 ni GO159.

Tous les chemins relatifs cités ci-dessous sont développés dans le manifeste. Racine source absolue : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle.

1. **Mode géré, repos réel et battement.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/wrapper.rs:4019–4024 borne les protocoles gérés à ACP, Claude stream JSON et Codex app-server. `reconnect_managed_session`:5579/5622 lit `transport.is_busy()` puis réémet cet état. Les implémentations réelles sont ACP:775–778, Claude:514–516, Codex:1326–1336 ; Codex inclut aussi `external_turn`. `forward_managed_events_with_redaction`:6521 transmet ActivityObserved ; TurnStarted/Finished:6532/6582 dépendent du mode automatic_reply. DeliveryRejected:6658–6666 publie false en dur : ce refus ne prouve donc pas toujours le repos. HEARTBEAT_INTERVAL:45 vaut3s, mais l’émission:4711–4713 est dans la branche timeout/WouldBlock. Elle n’est pas une réattestation de repos. Daemon Heartbeat:21181 ne fait que touch_link. `set_turn_state`:5229 remet busy_since à chaque true ; le TTL:5252 libère un état ancien sans preuve réelle. Aucun `turn_attest` actuel trouvé. Réemploi : session ManagedSession réelle et battement existant ; aucune nouvelle minuterie proposée.

2. **Tick des relances.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:6555–6584 porte la boucle, avec attente nominale1s:6559. `native_delegation::tick` précède l’échantillonnage frais de scheduled_wakes_tick. `collect_reminder_actions`:8718 réemploie cette boucle. Le coût des traitements s’ajoute à l’attente : aucune garantie de latence1s n’est mesurée ici.

3. **Écarts pont et issue indéterminée.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/t3code.rs:1850–1926 traite Saturated, Expired (saga ou demande), RequestClosed et GuidanceUnavailable. `report_discard`:1868 émet DeliveryIndeterminate pour l’idempotent ; il émet aussi DeliveryRejected lorsque reply=true ou guidance indisponible, sauf RequestClosed. Le daemon:19581–19621 appelle `mark_delivery_indeterminate` avec instance/génération attestées ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs:597–618 ne change que dispatching→indeterminate. Cette seule trame ne ferme pas une demande suivie. Wrapper:749–813 renvoie aussi Indeterminate si mauvaise instance, reçu Seen/indéterminé, quota du reçu ou erreur de stockage. Ces causes sont distinctes ; ne pas traduire toute issue indéterminée en non-livraison certaine.

4. **Aucune intention, aucun lien d’exécution.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:11333–11361 ne construit DeliveryExecutionLink que pour TriggerTurn ou InterruptAndStart. Le puits réutilisé est `begin_send_delivery_with_guidance`. Une remise retenue sans intention doit rester hors execution_queue ; le nom d’une file ne crée aucun lien d’exécution.

5. **Lecture/fermeture des demandes dans une transaction.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs:89–175 ouvre Immediate, passe le reçu Prepared→Dispatching, insère livraison/ledger/lien puis `tracked_requests` lorsque reply est présent, avant commit. `acknowledge_send_delivery`:768–832 appelle `mark_answered_in_transaction` dans cette transaction. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store/ledger_requests.rs:699–714 ferme seulement une demande open avec identités sender/target exactes. `cancel_request`:330–358 est une autre transaction Store. La connexion IdempotencyStore connaît donc déjà la table ; une fermeture atomique held devra réutiliser ce patron sur la même transaction, sans appeler une seconde transaction Store comme si elle était atomique.

6. **BatchedDelivery des porteurs gérés.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/wrapper.rs:891–986 contient `deliver_batch_to_interactive` et le type Historic/Idempotent. Son seul appel production trouvé:906 passe un vecteur d’un seul élément. Les appels multiples trouvés sont des tests. Le regroupement effectif de plusieurs livraisons gérées n’est donc pas établi. T3 possède séparément BATCH_BOUND=8 et batch_family:3261 ; ce chemin reste hors159A. Réutiliser le traitement existant lorsque pertinent, sans annoncer un lot géré actuel sur la foi du nom.

7. **Garde et liste156.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/guidance.rs:1–142 fournit la garde pure (source attestée, child proof, projet, pause, mission et taux). QueueList laisse le filtrage au caller après le refus child proof. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:10749–10841 filtre par droits/auteur/reçu avant le texte preview120 ; la position reste globale, borne200. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs:403–425 ne lit actuellement que dispatching non expiré, ordre rowid. Held159 doit étendre ce même puits et sa projection, pas une file parallèle. La surface CLI/MCP entière n’a pas été réauditée dans cette lecture courte.

8. **Schéma exact et lien message.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store.rs:170–187 définit `tracked_requests` : id, sender, target, state fermé (open/answered/cancelled/timed_out), created_at, deadline_at et champs de résultat. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:11324–11332 lie ReplyTracking.request_id au message.id ; la réponse porte in_reply_to, consommé au commit ACK. SQL et pending_replies mémoire sont distincts. Tous les retraits mémoire n’ont pas été relus. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store_schema.rs:4 reste version6 ; le CHECK send_deliveries actuel ne comprend pas held.

9. **Interruption de pause et reprise.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/execution_store.rs:1521–1552 stocke et relit `control_pause_interruptions`, avec resumed_at IS NULL ; schéma:1655 et index pending sur target_agent. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:12718–12750 reprend via `schedule_execution_recovery` lorsque route/presence existe et n’est pas busy. Lever la pause:12632–12659 et TurnStatefalse:21201 appellent la reprise. La garde held future doit lire l’absence de pause pending dans sa transaction ; aucune implémentation held atomique n’existe encore. La politique interne de toutes les reprises d’exécution n’a pas été relue exhaustivement.

10. **Destinataire admis et identité enregistrée.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:9710–9717 traduit uniquement l’alias humain vers UI_HUMAN_AGENT_ID. `prepare_dispatch`:10844–10886 utilise routing_agent_id sur from/to avant les lookups. Register:7952–7971 reprend/remplace les routes puis assigne final_agent_id=agent_id ; presences:8225 et restauration:8259 utilisent cette identité finale. Pour held159, recipient_agent doit désigner la clé de route résolue, distincte de recipient_instance et du nom to brut. L’alias UI n’est pas une preuve de provenance et Human reste exclu du classement held.

## Limites et suite

R0–R17 de research159 ont été relus, ainsi que T002. Les références T3 externes et les anciennes captures de recherche restent documentaires ; elles ne sont pas des lectures fraîches de T3. Aucun runtime, cadence réelle, reprise après panne ou mesure de latence n’a été exécuté. Les porteurs PTY/tmux/T3 n’ont aucune preuve nouvelle de session ManagedSession attestée dans cette investigation. Held, turn_gate durable, migrationv7 et fraîcheur turn_attest restent absents. Aucun helper ou test nouveau proposé. La préparation de159 attend le GO ROOT après livraison158B.
