# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/research.md](../../../../../../specs/159-edition-file/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

Repris de 156 R0 (ADR 011, ADR 027) : la garde protège contre un agent qui agit par
ses outils Bridget (MCP, CLI, pont T3, wrapper). Un processus hostile du même compte
Unix qui forge des trames brutes est hors modèle ; on ne le promet pas. Les tests de
trames brutes sont de la défense en profondeur.

<a id="extrait-3"></a>

## Extrait 3

| Outil T3 | Sémantique T3 | Reprise |
|---|---|---|
| `t3_queue_list` | liste dans l'ordre de remise | `bridget_queue list` (156, lu dans la base) |
| `t3_queue_read` | lit 16 000 caractères | écarté : l'aperçu suffit |
| `t3_queue_edit` | remplace le texte | reporté (R15) |
| `t3_queue_cancel` | annule un message en file | `cancel` (R7) |
| `t3_queue_reorder` | `beforeRunId` ou `null` | `reorder` (R8) |
| `t3_queue_promote_to_steer` | `targetRunId` | `promote` (R9) |

<a id="extrait-4"></a>

## Extrait 4

T3 y arrive parce que ses éditions sont des commandes sur une file durable côté
serveur (`queued-run.cancel`, `queued-run.edit`, `queued-run.reorder`,
`queued-message.promote-to-steer`). La leçon pour Bridget : tenir la file dans un
seul magasin durable.

<a id="extrait-5"></a>

## Extrait 5

La première conception éditait la file mémoire de chaque porteur. Les preuves :
- Codex : `QueueState { messages, steer, active, steering_open }` (`codex_app_server.rs:876-900`) ; `cancel_delivery` annule d'abord le tour actif (`:1081-1093`).
- ACP : `cancel_delivery` annule aussi le tour actif (`acp.rs:518-531`).
- Pont T3 : `worker.queue`, borne 64, retrait par `report_discard` qui rend la remise `indeterminate` (`t3code.rs:44`, `:1913-1922`, `:1855-1880`) ; registre `cancelled` en mémoire (`:1926-1929`).
- Reçu durable du porteur : `Seen` écrit avant l'injection, `Indeterminate` au rejeu (`receipt_store.rs:123-165`).
- Reprise relit toute remise `dispatching` (`send_delivery.rs:379-449`).

<a id="extrait-6"></a>

## Extrait 6

Un message est retenu si et seulement si (FR-002) :
1. le type du destinataire a `held_queue` dans le registre (`AgentDefinition`, `registry.rs:49-79` ; champ à ajouter avec `#[serde(default)]`, comme `queue_capacity` et `notify_timeout_secs`) ;
2. envoi client idempotent sans intention ;
3. origine ni `Human` (un message humain interrompt, `codex_app_server.rs:866`) ni `System` ; ni alerte de fil, observation, incident délégué ; ni message de mission native (`delegation_store.for_mission`, `daemon.rs:4190-4195`) ;
4. destinataire non T3 (`presence.transport != "t3code"`) et instance ayant déclaré `turn_attest` (R5.1) ;
5. présence `busy` (signal `TurnState`) ou messages déjà retenus pour ce destinataire.

<a id="extrait-7"></a>

## Extrait 7

Origine humaine ou système, alertes de fil, observations, incidents délégués, missions natives, fils T3, intentions `steer` et `interrupt` (156), types d'agents sans `held_queue`. Un envoi qui n'est pas retenu ne peut pas être édité : `not_held`.

<a id="extrait-8"></a>

## Extrait 8

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

<a id="extrait-9"></a>

## Extrait 9

**Verdict** : la refonte du chemin de remise par défaut en une session est trop
risquée (reprise, échéances, suivi de réponse, ordre, mesure de latence, tous à la
fois). Découpage retenu : 159a (la file retenue, réglage fermé par défaut, mesurable),
puis 159b (les éditions, de simples transactions). Les fils T3 sortent du périmètre
(159c).

<a id="extrait-10"></a>

## Extrait 10

Deux pistes, non engagées :
1. Faire émettre `TurnState` au pont et déplacer le regroupement (famille de messages, bornes) du pont vers la libération. Refonte lourde.
2. Confier à T3 sa file : le pont envoie les messages avec `dispatchMode: queue_after_active` (déjà posé, `t3code_contract_v2.rs:255-272`) et édite par les commandes `queued-run.*`. Reste à vérifier : correspondance `messageId` Bridget et `queuedRunId` T3, accès de la commande par le RPC v2 du pont, compatibilité avec la saga d'accusé.

<a id="extrait-11"></a>

## Extrait 11

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

<a id="extrait-12"></a>

## Extrait 12

3. **Écarts pont et issue indéterminée.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/t3code.rs:1850–1926 traite Saturated, Expired (saga ou demande), RequestClosed et GuidanceUnavailable. `report_discard`:1868 émet DeliveryIndeterminate pour l’idempotent ; il émet aussi DeliveryRejected lorsque reply=true ou guidance indisponible, sauf RequestClosed. Le daemon:19581–19621 appelle `mark_delivery_indeterminate` avec instance/génération attestées ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/idempotency/send_delivery.rs:597–618 ne change que dispatching→indeterminate. Cette seule trame ne ferme pas une demande suivie. Wrapper:749–813 renvoie aussi Indeterminate si mauvaise instance, reçu Seen/indéterminé, quota du reçu ou erreur de stockage. Ces causes sont distinctes ; ne pas traduire toute issue indéterminée en non-livraison certaine.

<a id="extrait-13"></a>

## Extrait 13

6. **BatchedDelivery des porteurs gérés.** /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/wrapper.rs:891–986 contient `deliver_batch_to_interactive` et le type Historic/Idempotent. Son seul appel production trouvé:906 passe un vecteur d’un seul élément. Les appels multiples trouvés sont des tests. Le regroupement effectif de plusieurs livraisons gérées n’est donc pas établi. T3 possède séparément BATCH_BOUND=8 et batch_family:3261 ; ce chemin reste hors159A. Réutiliser le traitement existant lorsque pertinent, sans annoncer un lot géré actuel sur la foi du nom.
