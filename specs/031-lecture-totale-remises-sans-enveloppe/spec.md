# Session 031 — Lecture totale des remises sans enveloppe

## Contexte

Le lot parent `7ea339b789efe076d2a5d194067e431060f5ec6c` rend le lecteur
`send_delivery` total lorsque `message_bytes` vaut `NULL`. Deux autres lecteurs
restent stricts sur le schéma hérité : la reprise d'une instance et l'accusé
d'une remise. La base de production mesurée ne contient actuellement aucune
enveloppe absente ; cette session défend donc un état dormant mais reproductible
après restauration ou maintenance d'une base antérieure.

## Propriétés

### P0 — Classement observable

Toute lecture qui rend ou écarte une remise produit un classement observable.
Une ligne sans enveloppe ne peut pas disparaître silencieusement du résultat
tout en conservant un état qui affirme qu'elle reste en vol.

### P1 — Aucun état absorbant pendant la reprise

Une remise `dispatching` sans enveloppe est reclassée `indeterminate` pendant
la reprise. L'enveloppe absente rend l'injection inconnaissable ; elle ne prouve
pas l'échec connu porté par la phase voisine `orphaned`.

### P2 — Isolation des remises relivrables

L'illisibilité d'une remise n'affecte pas les autres remises de la même
instance. Une reprise contenant une ligne sans enveloppe et deux enveloppes
valides retourne exactement les deux remises valides, sans erreur SQLite.

### P3 — Un accusé reçu enregistre le fait connu

Un accusé valide ne dépend pas de la présence de l'enveloppe locale. Pour une
remise `dispatching` sans enveloppe, la transaction passe la remise à `acked` et
le record idempotent à `terminal/accepted`. La corrélation `in_reply_to`, devenue
impossible, est nommée par une trace et le résultat ne prétend pas l'avoir faite.
Une remise déjà `acked` reste idempotente ; toute autre phase conserve son
verdict métier sans exposer `InvalidColumnType`.

## Critères d'acceptation

- Les deux oracles du jury ne voient plus aucune fuite
  `Sqlite(InvalidColumnType)` sur la reprise et l'accusé.
- Après une reprise, la ligne sans enveloppe n'est plus `dispatching` et vaut
  explicitement `indeterminate`.
- Une ligne illisible accompagnée de deux lignes valides n'empêche aucune des
  deux remises valides d'être retournée.
- Un accusé `dispatching + NULL` laisse la remise `acked`, le record
  `terminal/accepted`, aucune corrélation inventée et une trace observable.
- Trois mutants causaux sont tués séparément : filtrage sans reclassement,
  échec global de collecte, refus de l'accusé lorsque l'enveloppe manque.
- Les attentes métier sont écrites sans appeler le prédicat ou le helper dont
  elles attestent le comportement.

## Composition et base

- Branche empilée sur `7ea339b789efe076d2a5d194067e431060f5ec6c`, car le lot parent
  n'est pas encore ancêtre de `origin/main` au démarrage de la session.
- La phase `orphaned` peut atterrir avant le rebase final sans modifier le choix
  de P1 : absence d'enveloppe signifie `indeterminate`, pas échec connu.
- Après absorption du parent, le rebase vérifie d'abord la conservation du
  périmètre par diff, puis rejoue toutes les preuves.

## Hors périmètre

- Le typage Rust global des phases, capturé séparément comme idée SpecKit 26.
- La modification des migrations historiques ou l'invention d'une cinquième
  phase.
- Toute affirmation d'incident actif en production : zéro `message_bytes NULL`
  a été observé dans la copie de production mesurée avant cette session.
