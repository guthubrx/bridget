# Tasks 020 — Vérification transactionnelle avant envoi

## Règles d'exécution

- Une seule tâche active à la fois.
- Tests avant implémentation.
- Aucun hook actif n'est modifié avant jury et merge.
- Chaque chemin ci-dessous est relatif à la racine du dépôt.

## Phase 1 — Contrat et témoins

- [x] T001 [US1] Finaliser la propriété et les cas limites dans `specs/020-verification-pre-push/spec.md`
- [x] T002 [US1] Décrire l'observation distante et le calcul d'ensemble dans `specs/020-verification-pre-push/plan.md`
- [x] T003 [US1] Écrire les sept scénarios dans `scripts/test-git-pre-push-authorship.sh`
- [x] T004 [US1] Exécuter les scénarios contre un stub permissif et conserver le compte rouge dans `specs/020-verification-pre-push/implementation.md`

## Phase 2 — Barrière transactionnelle

- [x] T005 [US1] Implémenter le calcul transactionnel fail-closed dans `scripts/git-pre-push-authorship.sh`
- [x] T006 [US2] Valider les contrôles positifs et l'héritage distant dans `scripts/test-git-pre-push-authorship.sh`
- [x] T007 [US4] Muter volontairement le motif réel, observer le rouge puis restaurer `scripts/git-pre-push-authorship.sh`

## Phase 3 — Livraison sans activation

- [x] T008 [US3] Exécuter la suite finale, relire le diff et compléter `specs/020-verification-pre-push/implementation.md`

## Phase 4 — Amendements après jury

- [x] T009 [US3] Faire refuser une erreur du filtre et tuer isolément le retour optimiste
- [x] T010 [US3] Arrêter le banc si `mktemp` ou une fixture échoue, avant dérivation ou faux vert
- [x] T011 [US1] Remplacer les U rescans distants par une indexation O(U+R)
- [x] T012 [US3] Séparer les quatre causes fail-closed et tuer chacune par un mutant isolé

## Dépendances et stratégie

T001 et T002 précèdent le banc. T003 précède T004. T005 ne commence qu'après
le rouge de T004. T006 et T007 valident deux propriétés indépendantes du même
artefact et restent séquentielles car elles touchent le même script. T008 ferme
le lot sans activer le hook.
