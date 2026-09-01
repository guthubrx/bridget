# Checklist qualité - SPEC-084

## Complétude fonctionnelle

- [x] Le choix et le changement de serveur sont spécifiés.
- [x] Création et import sont distingués.
- [x] Les types `workspace`, `exact_project` et la réserve système sont définis.
- [x] La migration v1 est fail-closed pour la création.
- [x] La migration inventorie les projets historiques potentiellement concernés.
- [x] L'identité inter-serveurs est définie sans base centrale.
- [x] Les effets du retrait d'un emplacement sont définis.

## Testabilité

- [x] Chaque user story possède des scénarios indépendants.
- [x] Les exigences ont des résultats observables.
- [x] Les cas de concurrence, canonicalisation et déconnexion sont couverts.
- [x] Les critères de succès sont mesurables.

## Bornage

- [x] Docker est reporté à SPEC-085.
- [x] Le panneau serveur ne reçoit aucune autorité sur les autres serveurs.
- [x] Le dogfooding est reporté à SPEC-086.
- [x] Aucune base globale ou découverte générale du filesystem n'est introduite.
- [x] Aucun chemin de workspace n'est imposé par le produit.

## Verdict

PASS - La spécification peut entrer en planification sans clarification bloquante.
