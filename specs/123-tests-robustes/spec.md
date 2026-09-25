# Spécification 123 - Tests d'intégration robustes à la charge

## Fiche synthèse
Spec: 123-tests-robustes | Statut: Implemented | Priorité: P2 | Date: 2026-09-25
Branche: session-123-tests-robustes | Point 4 du plan.

## Problème observé
`search_104_test` échouait une fois sur deux sur machine chargée, sur l'ancien code comme sur le
nouveau (5/10 chacun en essais alternés, session 119) : chaque recette demandait des vérifications
manuelles. Deux causes : le client de test abandonnait après 3 s sans réponse (lecture de socket),
et s24 exigeait un « busy » en moins de 500 ms.

## Exigences
- **FR-001** : le délai de lecture du client de test partagé est porté à 30 s ; un vrai blocage
  échoue toujours, le garde-fou global de 360 s reste.
- **FR-002** : s24 vérifie la propriété voulue (le « busy » revient avant la fin des recherches qui
  tiennent les permis), non un temps absolu.
- **FR-003** : les seuils de performance, déjà réservés à la compilation optimisée, sont inchangés.

## Critères de succès
- **SC-001** : `search_104_test` vert à répétition sur machine chargée.
