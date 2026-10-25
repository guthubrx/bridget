# Reprise d’une admission choisie — 25 octobre 2026

## Défaut observé

Après `remote_prepare:WorkerUnavailable`, la qualification KS-STOR conserve un reçu V2 `chosen`, sans tâche, dans l’époque 58 du coordinateur. Un rejeu strict renvoyait seulement `admission_pending`. L’entretien automatique parcourt les époques précédentes. La garde de maintenance voit donc une admission non soldée.

## Correction

Le rejeu explicite d’un reçu `chosen` reprend le parcours du choix enregistré. Il ne consulte aucun décideur et ne change ni candidat, ni modèle, ni effort, ni enveloppe. Un permis temporaire indexé par propriétaire et clé couvre la préparation ET la clôture CAS de son erreur. Un second appel concurrent attend. Le reçu est relu sous le verrou métier avant acquisition ; ses états terminaux et sa révision sont respectés.

Le parcours existant revalide la source Git, les droits, la définition et la connexion. Une tâche déjà admise ou un refus durable est seulement relu. Les états `pending` et `decision_started` conservent leur comportement. Le contrat du worker, la base persistée et la garde de maintenance ne changent pas.

L’index coûte O(1) en moyenne par entrée et contient uniquement les parcours choisis en cours. Aucune dépendance, table, boucle périodique ni commande d’administration n’est ajoutée.

## Preuves

- RED : le nouveau test de panne SQL laisse le reçu `chosen`, puis le rejeu échoue avec `admission_pending` malgré le retrait de la panne, sans changer d’époque.
- GREEN initial : trois tests couvrent la reprise, quatre appels concurrents, le propriétaire, l’enveloppe, le refus d’une base différente et une révocation.
- R1 indépendante : task `60d8e551-5add-4101-b321-26b4c9a85e6a`. Réserve P2 : le permis se libérait avant la clôture terminale. Corrigé en clôturant l’erreur dans le parcours exclusif.
- Test synchronisé : A observe une mauvaise base puis attend avant la clôture ; la fixture Git retrouve sa base ; B rejoue et reçoit `admission_pending` ; A clôt le refus. Aucun fournisseur ne démarre. La synchronisation de test est absente du binaire de production.
- Validation finale R2 : 35 tests `spec168_`, 4 tests `spec174_admission_hint` et 3 tests `spec183_rejeu_chosen` passent, soit 42 tests, aucun ignoré. Tous passent par `scripts/build.py cargo test -p bridget-daemon --lib FILTRE --jobs 1`.
- R2 indépendante : task `85de78a2-192e-40d5-a204-305c8a6cf1a8`, réserve P2 levée, aucun défaut bloquant trouvé. Revue statique ; les tests ont été exécutés par le responsable.
- `git diff --check` propre. Format vérifié sur les blocs nouveaux ; aucun reformatage global des modules historiques. Les warnings de compilation restants existaient hors du delta.

Le test de révocation observe le code existant `delegation_grant_required`, qui reste transitoire ; il n’est pas reclassé pour faire passer le test.

## Activation et limites

Cette note décrit une correction de sources, pas une activation. L’ancien core ne peut pas exécuter le nouveau rejeu avant remplacement. Le reçu de production n’est pas modifié manuellement. Aucun service n’est arrêté pour contourner `update_missions_active_or_unconfirmed`.

Une tâche historique `e5959986-e426-422d-956f-e035dbcf89be` reste également `cancelling`, avec nettoyage non confirmé. Sa réconciliation est distincte de la qualification OVH. Les travaux et preuves sont conservés.

Correction technique : aucun parcours visuel modifié, aucun diaporama requis.
