# Spécification 122 - Les abonnements d'observation survivent à un redémarrage du daemon

## Fiche synthèse
Spec: 122-abonnements-durables | Statut: Implemented | Priorité: P1 | Date: 2026-09-25
Branche: session-122-abonnements-durables | Point 2 du plan. Décision : [ADR 043](../../docs/decisions/043-avis-d-observation-amortis-et-abonnements-repris.md).

## Problème observé
Relance du daemon le 25/09 à 06:04 : `sol_city_ai` a reçu quatre avis « nouvel abonnement requis »,
a dépensé des tours à se réabonner, et restait aveugle aux fins de tour de ses workers d'ici là.

## Exigences
- **FR-001** : au démarrage, un abonnement non expiré de l'instantané est repris sans nouvel
  abonnement ; `source_unavailable` jusqu'au retour de sa source, puis `active`.
- **FR-002** : le propriétaire reçoit un seul avertissement à son retour, qui dit que les faits de
  la coupure sont perdus et non rejoués.
- **FR-003** : une source qui ne revient pas est annoncée indisponible après 30 s (règle de la 119).
- **FR-004** : la durée absolue (TTL) de l'abonnement est préservée.

## Critères de succès
- **SC-001** : test d'intégration avec vrai redémarrage : avertissement, état `source_unavailable`
  puis `active`, fin de tour remise sans nouvel `sub`.
- **SC-002** : documentation et catalogue (`coverage.lifetime`) décrivent la reprise.
