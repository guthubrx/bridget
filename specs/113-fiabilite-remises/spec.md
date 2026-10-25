# Spécification 113 — Une remise écartée par le pont n'est plus « en vol »

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/113-fiabilite-remises/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/113-fiabilite-remises/spec.md#extrait-2).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/113-fiabilite-remises/spec.md#extrait-3).

## Critères de succès
- **SC-001** : saturation → saga close + expéditeur prévenu avec motif « saturée ».
- **SC-002** : échéance passée → saga close + refus porteur de « échéance ».
- **SC-003** : demande répondue ou annulée → saga close, aucun `DeliveryRejected`.
- **SC-004** : recette complète verte ; après livraison, une remise jetée par le pont n'apparaît plus
  `dispatching` côté daemon.
