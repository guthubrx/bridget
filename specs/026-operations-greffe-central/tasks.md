# Tâches — Session 026

## Première tranche urgente

- [x] T2601 Décider une seule porte de vérité : le guichet.
- [x] T2602 Rebaser sur 021@`2623772` et conserver son motif composé.
- [x] T2603 Composer la vraie v18 de rc1 avant toute v19.
- [x] T2604 Ajouter l'enveloppe canonique `delegate` et sa validation daemon.
- [x] T2605 Parser la charge côté Maicie avec `deny_unknown_fields` et bornes.
- [x] T2606 Appeler directement `citation::unclassified_known_citations`.
- [x] T2607 Refuser exactement `suite=aucune` + UUID connu non classé.
- [x] T2608 Persister puis rejouer le reçu dans `guichet_refusal_receptions`.
- [x] T2609 Retourner `operation_not_available` pour une demande sans
  contradiction, sans appliquer de délégation.
- [x] T2610 Déclarer et tester la limite branche/SHA sans heuristique.
- [x] T2611 Engendrer `ALL`, `as_sql()` et le parseur depuis les enums Maicie.
- [x] T2612 Implémenter v19 sur la seule table des refus fédérés.
- [x] T2613 Préflighter v18 sous savepoint avec rollback dans les deux branches.
- [x] T2614 Tester vraie v18, fausse v18, fausse v19, v14 privé et bootstrap.
- [x] T2615 Refuser fail-closed une valeur SQL inconnue à la lecture.
- [ ] T2616 Exécuter le comptage final, clippy, geler et pousser la tête.

## Seconde tranche urgente — pilotage fédéré

- [ ] T2620 Appliquer réellement `delegate` avec configuration et candidats
  centraux par le même service que le CLI.
- [ ] T2621 Ajouter `registre_add` sans chemin fourni par l'appelant, via le
  journal déclaré par le greffe.
- [ ] T2622 Ajouter `objective_close` sur preuve centrale attestée, via la même
  transaction et les mêmes gardes que le CLI.
- [ ] T2623 Rendre durables les tentatives `profile_approve` et
  `routine_approve`, sans les ajouter à l'enum autorisé.
- [ ] T2624 Exposer les quatre outils MCP, enregistrer l'identité MCP résolue et
  rendre l'issue terminale complète depuis le daemon maître.

## Critères de livraison de la première tranche

- `cargo test --workspace --no-run` précède le comptage ;
- le contrat contradiction et la migration v19 sont verts ;
- les rouges workspace sont listés et imputés ;
- les deux approbations reportées sont annoncées comme ignorées, jamais comme
  tenues ;
- le SHA réellement mesuré et la chaîne v17→v18→v19 sont publiés ;
- le commit suit strictement `type(scope): Description`, sans trailer.
