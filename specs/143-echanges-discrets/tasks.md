# Tâches — SPEC143

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/tasks.md#extrait-1).

## Setup

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/tasks.md#extrait-2).

## Histoires utilisateur

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/tasks.md#extrait-3).

## Validation et clôture

- [x] T006 — Contrôles proportionnés : format/lint des quatre fichiers, types et build frontend avec outils déjà installés. Consigner commandes, résultats et warnings de baseline ; aucun contrôle global ni réinstallation. Résultat : contrôles ciblés sans erreur nouvelle. FR09/10, SC05.
- [x] T007 — Recette en aperçu isolé autorisé : entrées et sorties réelles en fixture, dépliage/clavier/focus, noms/IDs, erreurs/refus/unknown et completed sans livraison, largeur étroite, thèmes, actions/ancrage. Résultat : preuves UI factuelles sans restart ni déploiement. FR01–10, SC01–05.
- [x] T008 — Converge lecture seule puis audit ciblé et journal documentaire. Confronter dix FR/cinq SC au code et tests ; tâches byte-identiques si CONVERGED. Cocher après preuves, jamais par intention. Aucun commit ni déploiement. Résultat : verdict et limites traçables, statut exact. FR01–10, SC01–05.

## Dépendances et couverture

T001 → T002/T003 → T004 → T005 → T006 → T007 → T008. Responsabilités source distribuées par le principal ; préserver les edits d'autrui.
FR01→T003/T007 ; FR02→T003/T005 ; FR03→T002/T004 ; FR04→T004/T005 ; FR05→T002/T007 ; FR06→T002/T007 ; FR07→T002/T004/T005 ; FR08→T003/T004 ; FR09→T002–T006 ; FR10→T001/T005–T007. T008 vérifie la totalité.

## Complément autorisé US4

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/143-echanges-discrets/tasks.md#extrait-4).

Dépendances du complément : T009 → T010 → T011 → T012 → T013. Couverture : FR11→T009–T011/T013 ; FR12→T010/T011/T013 ; FR13→T009–T011/T013 ; FR14→T009–T011/T013 ; FR15→T010–T013 ; FR16→T009–T013. SC06→T010/T011/T013 ; SC07→T010–T013 ; SC08→T010–T013. Les tâches historiques T001–T008 ne sont ni rouvertes ni réécrites.
