# Contrat — ouverture d'objectif

## Entrée

Une ouverture reçoit une requête canonique et un `ObjectiveOpeningPermit`
opaque.

## Invariants

1. L'origine du permit et celle du payload sont identiques.
2. Une origine humaine appartient aux octets canoniques versionnés de la
   commande idempotente. L'origine automatique conserve les octets v2/v3
   historiques pour permettre le rejeu des réservations existantes.
3. `legacy_unknown` ne peut jamais ouvrir une nouvelle ligne.
4. Une origine humaine sans attestation vérifiée n'obtient aucun permit.
5. Un refus antérieur au commit n'écrit aucune ligne d'objectif, délégation,
   outbox ou idempotence.
6. Une transition d'objectif existant ne peut pas tomber dans une branche
   d'insertion.

## Compatibilité

Les payloads historiques sans champ `origin` restent lisibles et rendent
`legacy_unknown` dans le modèle. Leur re-sérialisation omet le champ et conserve
ainsi les octets historiques.

Le `canonical_request_sha256` de l'attestation porte sur le sujet canonique
hors attestation ; il ne peut pas être calculé sur un document qui le contient.
