# Journal 113 — Une remise écartée par le pont n'est plus « en vol »

- **Base** : main `4f53dc4d` — **Date** : 2026-09-19 — **Statut** : Implemented, livré en production le 2026-09-19 21:06

## Diagnostic
Le pont T3 jetait une remise dans quatre cas sans rien dire au daemon : file saturée (`QUEUE_BOUND`),
échéance passée avant démarrage (deux contrôles dans `drive_queue`), demande suivie close (handler
`RequestList`), annulation (`CancelDelivery`). Pour une remise idempotente, la saga restait
`dispatching` jusqu'à expiration (7 jours) ; pour une demande suivie, l'expéditeur n'apprenait rien.
Base de production le 2026-09-19 à 20:50 : 40 sagas `dispatching` (33 d'août), 14 remises
« périmées avant démarrage » jetées en silence dans la journée. Le wrapper ACP (`wrapper.rs`)
signale déjà une injection ratée par `DeliveryRejected` puis `DeliveryIndeterminate` : même geste.

## Correction (`t3code.rs`)
- `Discard` : `Saturated`, `Expired(quoi)`, `RequestClosed(état)`.
- `discard_cause(frame, received, now)` : regroupe les règles 111 et 112 (échéance de tour créditée
  de l'attente, échéance de réponse, expiration de saga, demande close ou échue) et nomme le motif.
  Remplace les deux contrôles dupliqués de `drive_queue`.
- `report_discard(frame, why)` : trace `warn!` avec motif ; `DeliveryIndeterminate` pour une remise
  idempotente (saga terminale) ; `DeliveryRejected` avec motif pour une demande suivie encore ouverte.
  Un motif d'échéance porte « échéance » : le daemon clôt alors la demande (`claim_timeout`).
  Une demande déjà close (`RequestClosed`) ne produit pas de second échec.
- `discard_by_id(id, why)` : retrait + rapport, partagé par `CancelDelivery` et `RequestList`.
- Aucun changement de protocole ni côté daemon.

## Documentation (`docs/reference-communication.md`)
- Section t3code : « borne de deux minutes » remplacée ; paragraphe « Attente et écartement
  (sessions 111 à 113) » : ce qui périme, ce qui ne périme pas, ce que voit l'expéditeur.
- Section Claude sans tmux : les routes hors T3 vers une identité attestée.

## Vérifications
- Tests `spec113_*` (3) : saturation → saga close + expéditeur prévenu ; échéance passée → saga close
  + refus porteur de « échéance » ; demande répondue ou annulée → saga close sans `DeliveryRejected`.
- `t3code::tests` : 49/49. fmt OK ; clippy `-D warnings` OK.
- Recette complète (`cargo test --workspace --no-fail-fast`, umask 077, `BRIDGET_HOME` privé) :
  **1524 réussis, 0 échec, 52 ignorés** sur 80 binaires.

## T005 — Livraison
Commit `8ae71edb` fusionné dans main en fast-forward ; binaire reconstruit à 21:05 ; seul le service
du pont (`com.bridget.t3`) a été relancé, le daemon n'a pas changé et les autres agents n'ont pas
été coupés. À la reconnexion, le daemon a rejoué les remises `dispatching` des fils vivants et le
pont a écarté avec motif celles dont la demande était close : sept sagas sont passées de
`dispatching` à `indeterminate` en une seconde (40 → 33 ; 30 → 37). Les 33 restantes datent
d'août et visent des instances disparues : elles ne sont plus rejouées et relèvent du balayage
des sagas expirées, hors périmètre.
