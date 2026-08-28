# Modèle de données — Session 056

## ObjectiveOrigin

Somme fermée persistée dans le payload JSON de chaque objectif :

- `auto_generated` : ouverture autonome du système ;
- `human_request` : ouverture causée par un message humain précis et une
  attestation daemon liée à la requête canonique ;
- `legacy_unknown` : lecture conservatrice d'un payload historique sans
  origine. Cette valeur ne possède aucun constructeur d'ouverture.

## ObjectiveOpeningPermit

Capacité opaque et non sérialisable. Elle porte l'origine autorisée par une
requête d'ouverture. L'insertion vérifie l'égalité avec l'origine du payload.
Un permit automatique ne peut donc pas persister une origine humaine.

Le permit est exigé par :

- `delegate` ;
- la réservation immédiate ;
- la réservation différée ;
- l'unique branche SQL qui insère un objectif absent.

Les transitions d'un objectif déjà existant utilisent une opération distincte
qui ne possède aucune branche d'insertion.

La capacité n'implémente ni sérialisation ni clonage. Ce n'est toutefois pas
un type linéaire : l'usage unique d'une future attestation humaine sera tenu
par le ledger et le gate des tranches suivantes, pas revendiqué ici. Dans cette
tranche, son unique constructeur public produit `auto_generated` ; aucun
appelant ne peut fabriquer un permit `human_request` à partir d'un identifiant
déclaré.

Le store contient par ailleurs une sonde de compatibilité v19 qui insère une
ligne temporaire sous savepoint puis l'annule et compare les cardinaux. Cette
écriture n'est pas une ouverture durable et reste séparée de la frontière
`open_objective`.

## Compatibilité historique

L'absence de `origin` se désérialise en `legacy_unknown`. Aucun `UPDATE` de
masse n'est exécuté : l'absence historique reste distinguable d'une provenance
mesurée après déploiement. Une re-sérialisation de cette variante omet encore
le champ : le corpus v1 reste identique octet par octet.

Pour préserver les réservations idempotentes existantes, le canon de la voie
automatique reste byte-identique aux versions 2 et 3 et omet `origin`. La future
voie humaine utilisera les versions 4 et 5 avec son origine attestée.

## Dette humaine — modèle réservé

Le ledger restera l'autorité de `HumanResponseObligationOpen(message_id)`. Le
store Maicie ne conservera qu'une projection conservatrice nécessaire à la
sérialisation. Cette projection ne devient jamais une seconde interprétation
du journal.
