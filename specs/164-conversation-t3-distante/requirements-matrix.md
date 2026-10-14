# Matrice164 — Exigences et preuves

Cette matrice décrit les preuves attendues. Aucun PASS de code164 n'est annoncé. Les identifiants Tasks ci-dessous sont figés après clôture documentaire de la contre-revue R1 et génération Tasks.

| Exigence | Surface/propriétaire | Scénarios de preuve et tâches |
|---|---|---|
| FR-001 | ROOT + DEV : diff connecteur seul | S164-12 + self-review du diff et absence de modification Rust  ; T001/T013/T014/T018 |
| FR-002 | DEV : fil virtuel existant | S164-09/12 + lecture prouvant l’absence d’effets fournisseur  ; T009/T010/T013 |
| FR-003 | DEV : projection commune | S164-01/02/04  ; T003/T004/T009 |
| FR-004 | DEV : texte, séquences, sessions et messages | S164-01/02  ; T003 |
| FR-005 | DEV : frontières de continuité | S164-02/03  ; T005/T011 |
| FR-006 | DEV : Markdown web/mobile | S164-07/08/11  ; T008/T009/T010 |
| FR-007 | DEV : activités sans corrélation inventée | S164-05  ; T006 |
| FR-008 | DEV : détails originaux | S164-06  ; T007 |
| FR-009 | DEV : état et contact natifs | S164-09  ; T009/T010 |
| FR-010 | DEV : actions, permissions et lecture seule | S164-09/11  ; T009/T010/T011 |
| FR-011 | DEV : conteneurs et projection | S164-04/10  ; T004/T011 |
| FR-012 | DEV : fenêtre, pages et RPC existants | S164-03/08/10 + lecture de complexité  ; T004/T005/T011 |
| FR-013 | DEV : historique/résultat/copie | S164-08/10  ; T009/T010/T011 |
| FR-014 | DEV : logique commune, deux rendus | S164-11/12  ; T003/T009/T010 |
| FR-015 | DEV : présentation/ROOT : recette | S164-07/12  ; T009/T010/T017 |
| FR-016 | DEV : pureté/ROOT : audit | S164-01/10 + diff/I/O/complexité  ; T003/T005/T007/T013/T014 |
| FR-017 | ROOT : Git/paquet/install/cleanup | Reçus livraison G8/G9 et références distantes  ; T015/T016/T018/T019/T020 |
| SC-001 | DEV : texte exact | S164-01/04  ; T003/T004 |
| SC-002 | DEV : lisibilité et fidélité | S164-03/05/06/08/11  ; T005/T006/T007/T009/T010/T011 |
| SC-003 | DEV : métier, connexion et actions | S164-09/10  ; T009/T010/T011 |
| SC-004 | DEV : rendu sans action | S164-07/12  ; T008/T010/T017 |
| SC-005 | DEV : limites et ordre | S164-02/03/04/10 + lecture de complexité  ; T004/T005/T011/T013 |
| SC-006 | ROOT : produit livré | Campagne/revue/audit/paquet/G8/G9  ; T012/T013/T014/T015/T016/T017/T018/T019/T020 |

## Couverture des histoires

| Histoire | Exigences |
|---|---|
| US1 | FR-003/004/005/006/007/015 |
| US2 | FR-005/009/010/011 |
| US3 | FR-008/011/012/013 |
| US4 | FR-001/002/014/016/017 |

## États de preuve

Spécification, plan et audit sont produits. Contre-revue de plan et Tasks sont produites ; analyse, GO, implémentation, tests, recette et livraison restent des gates distinctes. Les résultats historiques149/162/163 ne prouvent pas le code164. Le responsable ajoute la commande exacte, le hash de la source testée, le résultat et les limites avant tout cochage d'implémentation.
