# Contrat - Guidage, interruption et lecture de file 156

Compatible avec les contrats 148, 149 et 150. Un envoi ordinaire légitime sans
nouveau champ conserve ses octets canoniques. FR-009 retire les privilèges des
provenances forgées ; FR-022 restreint le guidage implicite des pairs et clients
nus. Les producteurs humains attestés et systèmes internes sont conservés.

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
| `invalid_intent` | valeur inconnue ou steer+reply=true | non (aucune clé réservée) |
| `intent_not_permitted` (motif : `not_parent`, `cross_project`, `self_target`, `identity_unattested`) | appelant sans droit | oui, par `reject_idempotent_send` |
| `referent_paused` | `steer`/`interrupt` d'un agent pendant la pause | oui |
| `mission_active` | interrupt agent vers enfant en mission ; humain explicite exempt | oui |
| `interrupt_rate_limited` (`retry_after_secs`) | 1 par 30 s par couple, 3 par 10 min par cible | oui |
| `steer_unavailable` | fil T3 dont le fournisseur n'a pas de guidage natif prouvé | rapporté par le pont comme remise rejetée |

« Mémorisé » : pour réessayer, utiliser une nouvelle clé (comportement actuel
de tout refus de `SendIdempotent`). Sur `Send` historique, chaque refus est un
`Nack`. Sur `ControlExecution`, le refus de droits est
`ExecutionControlRefusal::NotPermitted`.

Ordre : `invalid_intent`, identité attestée, neutralisation de l'origine,
réservation (rejeu : `envelope_mismatch` ou résultat mémorisé),
garde de projet existante, `intent_not_permitted`, `referent_paused`,
`mission_active`, `interrupt_rate_limited`, acceptation de la remise.
Sur SendIdempotent, inscrire admitted dans begin_send_delivery_with_guidance,
la transaction existante qui admet
remise, suivi, lien et journal. Sur historique/control/cancel, l’audit accepté
est durable avant émission physique. Un INSERT échoué interdit le contrôle.
Un crash après admission et avant émission peut consommer le rythme sans effet
physique ; ce choix est conservateur. Un refus tardif n’est pas compté ;
l’effet fournisseur reste unconfirmed.

Pour ControlExecution, reserve_control_command conserve canon et rejeu AVANT la garde. La garde d’une commande neuve précède mark_control_dispatched, le comptage et l’émission ; le rejeu conserve sa réponse existante.

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

DeliverIdempotent ET Deliver historique portent mid_turn_ok optionnel calculé
au daemon à chaque envoi/reprise. Sans le champ, forme historique compatible ;
valeur absente au pont nouveau : attente. Avec vrai, garder les critères groupable
existants. Sources admises : attach, UI prouvée par credential/instance primaire
réservée type ui, systèmes internes, parent/enfant. Pairs/clients nus attendent.
Le rôle Client UI reste inchangé ; aucun from=humain ou origin client ne fait
preuve. Ne jamais fabriquer origin=Human pour obtenir ce flag.
La remise immédiate utilise la connexion source réelle. À la récupération, from ne permet jamais d’emprunter une route UI ou humaine ; sans preuve UI persistée, la remise attend normalement. Pour les agents UUID, recalculer les liens depuis les faits actuels.

Aucun steer agent non consommé ne déclenche automatiquement turn/interrupt,
même avec une deadline fournie. Le timeout humain validé et l'échéance propre
du tour principal restent. Interrupt conserve la FIFO : un message plus ancien
peut porter le prochain tour.

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

Périmètre : seulement send_deliveries idempotent en phase dispatching ; aucun
Send historique ni file interne de transport.

Visibilité : destinataire, parent, Attach et UI Client attestée voient toute
la projection ; autres appelants : leurs propres messages. Preuve enfant refusée. Refus :
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

- Capacité `QueueListV1`. Sans elle, `QueueList` est refusée `CapabilityNotNegotiated`.
- Attach négocie ClientHello uniquement avec la liste exacte [QueueListV1]. Son
  rôle reste Attach, sans identité agent/conn_names ni autres capacités. L’UI
  Client attestée peut lire toute la projection sans changer de rôle ou obtenir
  une autorité de contrôle explicite.
- `QueueList { target }` (client vers daemon) et `QueueListResult { entries, queue_bound }`.
- Aucune trame daemon vers wrapper n'est ajoutée.
- Les trames sont classées dans la matrice exhaustive de `daemon.rs` (aucun `_`).
- `ExecutionControlRefusal::NotPermitted` ajouté.

## Pont T3

Aucun changement de forme de la projection Lineage. Le pont pose
`deliveryIntent: "steer"` (Claude seul, activeSteering natif prouvé) ou
`"restart"` (`interrupt`).

Cursor n’a pas activeSteering natif dans le bundle installé149 ; son support
SteeringByInterruptRestart peut convertir steer en interrupt_restart. Son
steer est donc refusé steer_unavailable et sa remise implicite pendant un tour
attend, même avec mid_turn_ok=true. Aucun repli interrupt ni bypass du rythme.
La remise Claude requiert aussi la garde de provenance et le flag ; cela ne
change pas le transport Codex natif. Ces changements visibles ferment un repli
non sûr dans T012/T021/T029 existants, sans nouveau scénario ou permission.

## Limite de compatibilité REST/V1 du pont T3

La voie REST/V1 Client.dispatch ignore l’ancien booléen et ne sait pas exprimer
deliveryIntent. Elle refuse donc localement toute intention explicite avant
émission : steer_unavailable pour steer, motif de capacité explicite pour
interrupt, avec les erreurs/report_discard existants. L’envoi ordinaire None
reste inchangé. Le paquet installé149 est V2 : le trajet de production visé
n’est pas changé par cette fermeture V1. Aucun protocole, API, type ou nom
d’oracle supplémentaire ; T012 étend seulement les cas déjà prévus.
