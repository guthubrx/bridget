# Contrat - Guidage, interruption et lecture de file 156

Compatible avec les contrats 148, 149 et 150. Un appel sans nouveau champ produit
les mêmes octets canoniques et le même comportement qu'avant.

## `bridget_send` - champ ajouté

| Champ | Type | Obligatoire | Valeurs | Défaut |
|---|---|---|---|---|
| `intent` | texte | non | `queue`, `steer`, `interrupt` | absent = `queue` |

CLI : `bridget send --intent <valeur>`.

| `intent` | Intention posée sur le message avant le canon |
|---|---|
| absent, `queue` | aucune (octets historiques) |
| `steer` | `SteerCurrent` |
| `interrupt` | `InterruptAndStart` |

Le daemon ne réécrit jamais l'intention selon l'état de la cible.

Résultat d'un envoi avec intention : le résultat existant gagne `intent`
(valeur demandée) et `effect: "unconfirmed"`. Le contrat ne promet aucun effet.
L'effet réel (injection dans le tour, repli en file, rejet) est celui du
transport, lisible dans l'accusé de remise et le journal du transport.

Rejeu d'une clé admise : même résultat qu'au premier appel, quel que soit
l'état actuel de la cible. Autre intention sous la même clé :
`envelope_mismatch`.

## Refus

| Code | Quand | Mémorisé |
|---|---|---|
| `invalid_intent` | valeur inconnue | non (aucune clé réservée) |
| `intent_not_permitted` (motif : `not_parent`, `cross_project`, `self_target`, `identity_unattested`) | appelant sans droit | oui, par `reject_idempotent_send` |
| `referent_paused` | `steer`/`interrupt` d'un agent pendant la pause | oui |
| `mission_active` | `interrupt` d'un enfant avec tâche native active | oui |
| `interrupt_rate_limited` (`retry_after_secs`) | 1 par 30 s par couple, 3 par 10 min par cible | oui |
| `steer_unavailable` | fil T3 dont le fournisseur n'a pas de guidage natif prouvé | rapporté par le pont comme remise rejetée |

« Mémorisé » : pour réessayer, utiliser une nouvelle clé (comportement actuel
de tout refus de `SendIdempotent`). Sur `Send` historique, chaque refus est un
`Nack`. Sur `ControlExecution`, le refus de droits est
`ExecutionControlRefusal::NotPermitted`.

Ordre : `invalid_intent`, identité attestée, neutralisation de l'origine,
réservation (rejeu : `envelope_mismatch` ou résultat mémorisé),
`intent_not_permitted`, `referent_paused`, `mission_active`,
`interrupt_rate_limited`, garde de projet existante, remise.

## Chemins couverts par la même garde

`bridget_send` (MCP), `bridget send` (CLI), `SendIdempotent`, `Send`
historique (y compris `intent` bruts `SteerCurrent`/`InterruptAndStart` et
`origin=Human`, en défense en profondeur), `ControlExecution` (`SteerCurrent`,
`Interrupt`, avec normalisation de l'origine du message), guidage implicite du
pont T3, `bridget_cancel`. Exception interne : l'interruption de pause (sans
connexion cliente).

Humain attesté = connexion de rôle attach. Un client nu `from=human` garde
l'envoi en file et reçoit `identity_unattested` pour le contrôle.

Cadre de menace : la garde protège contre un agent qui utilise ses outils. Un
processus hostile du même compte qui forge des trames brutes est hors modèle
(ADR 011) ; le contrat ne le couvre pas.

## Guidage implicite du pont T3 (changement visible)

La trame `DeliverIdempotent` gagne `mid_turn_ok` (booléen optionnel, calculé par
le daemon à chaque construction de la trame). Le pont ne remet un message en
cours de tour (`deliveryIntent: "steer"`, session 129) que si `mid_turn_ok` est
vrai. Expéditeurs autorisés : humain attaché, parent de la cible, enfant de la
cible. Les pairs attendent la fin du tour. Absence de la valeur : attente.

## `bridget_cancel` : retrait et interruption

La demande est toujours annulée et son message encore en file est retiré. La
trame `CancelDelivery` gagne `interrupt_active` (défaut faux). Le daemon la met
à vrai seulement si l'appelant passe la garde commune (droits, pause, rythme,
mission). Sinon le tour actif continue. L'interruption compte dans les limites
de rythme.

## `bridget_queue` - nouvel outil, lecture seule

| Argument | Type | Obligatoire |
|---|---|---|
| `target` | texte (nom, UUID ou préfixe, comme `bridget_send`) | oui |

Réponse :

| Champ | Sens |
|---|---|
| `entries[]` | au plus 200 ; `message_id`, `from`, `intent`, `reply`, `age_secs`, `preview` (120 caractères), `position`, `state: "pending_or_injecting"` |
| `queue_bound` | 64 pour un fil T3, `null` sinon |

Visibilité : voir `research.md` R5. Preuve enfant refusée. Refus :
`intent_not_permitted` n'existe pas ici (la réponse est filtrée) ;
`target_unavailable` si la cible est inconnue.

## Annonce de capacités

`bridget_capabilities` ajoute :

| Clé | Valeur |
|---|---|
| `guidance.intents` | `["queue","steer","interrupt"]` |
| `guidance.interrupt_cooldown_secs` | 30 |
| `guidance.interrupt_cap_per_10min` | 3 |
| `guidance.queue_tool` | `true` |

## Protocole daemon

- Capacité client `QueueListV1`. Sans elle, `QueueList` est refusée `CapabilityNotNegotiated`.
- `QueueList { target }` (client vers daemon) et `QueueListResult { entries, queue_bound }`.
- Aucune trame daemon vers wrapper n'est ajoutée.
- Les trames sont classées dans la matrice exhaustive de `daemon.rs` (aucun `_`).
- `ExecutionControlRefusal::NotPermitted` ajouté.

## Pont T3

Aucun changement de forme de la projection Lineage. Le pont pose
`deliveryIntent: "steer"` (fournisseurs Claude et Cursor seulement) ou
`"restart"` (`interrupt`).
