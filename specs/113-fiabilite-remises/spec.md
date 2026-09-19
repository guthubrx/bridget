# Spécification 113 — Une remise écartée par le pont n'est plus « en vol »

## Fiche synthèse
Spec: 113-fiabilite-remises | Statut: In Progress | Priorité: P1 | Date: 2026-09-19
Branche: session-113-tete-de-file | Dépend de 111 et 112 (attente en file du pont T3).
Origine : contre-revue adverse d'evols-t3 (Codex) du 2026-09-19, voir `adversarial-review-evols-t3.md`.

## Problème observé
Le pont T3 écarte une remise dans quatre cas : file saturée (64 remises), échéance passée avant
démarrage, demande suivie close côté daemon, annulation. Dans les quatre, il ne dit rien au daemon.
La saga d'envoi idempotent reste en phase `dispatching` jusqu'à son expiration (7 jours) et
l'expéditeur voit « remise en vol » pour un message que le pont a jeté. Le 2026-09-19, la base de
production comptait 40 sagas `dispatching`, dont 33 d'août ; 14 remises « périmées avant démarrage »
ont été jetées en silence dans la journée. Le wrapper ACP, lui, signale déjà une injection ratée
(`DeliveryRejected` + `DeliveryIndeterminate`).

## Exigences
- **FR-001** : toute remise écartée par le pont, quel que soit le motif, est rapportée au daemon.
- **FR-002** : pour une remise idempotente, le rapport rend la saga terminale (`DeliveryIndeterminate`,
  comme le wrapper ACP pour une injection ratée) : plus de « en vol » fantôme.
- **FR-003** : pour une demande suivie encore ouverte, le rapport prévient l'expéditeur
  (`DeliveryRejected` avec motif) ; un motif d'échéance porte le mot « échéance », sur lequel le
  daemon clôt la demande en `timed_out`.
- **FR-004** : une demande déjà close côté daemon (répondue, annulée, expirée) ne produit pas de
  second échec : seule la saga est close.
- **FR-005** : chaque écartement laisse une trace de journal avec le motif ; la purge silencieuse
  disparaît.
- **FR-006** : aucun changement de protocole ni côté daemon ; les règles 111 et 112 sont inchangées.

## Hors périmètre
- Regrouper plusieurs remises dans un même tour (proposition écartée par la contre-revue pour les
  `reply=true` : une réponse par tour).
- `SteerCurrent` sur le pont : dépend du support T3.
- Balayage des 33 sagas d'août : elles expirent d'elles-mêmes à 7 jours.

## Critères de succès
- **SC-001** : saturation → saga close + expéditeur prévenu avec motif « saturée ».
- **SC-002** : échéance passée → saga close + refus porteur de « échéance ».
- **SC-003** : demande répondue ou annulée → saga close, aucun `DeliveryRejected`.
- **SC-004** : recette complète verte ; après livraison, une remise jetée par le pont n'apparaît plus
  `dispatching` côté daemon.
