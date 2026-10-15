# Checklist de qualité — SPEC168 Routage de la flotte

Créée : 2026-10-15.
Spécification : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/spec.md
Portée : revue de la spécification, pas preuve d’implémentation ou de tests.

## Qualité du contenu
- [x] Le besoin, la valeur et les acteurs sont décrits.
- [x] Les sections obligatoires sont remplies.
- [x] La spécification décrit le comportement attendu sans imposer de nouveau framework.
- [x] Le texte sépare ce qui est déclaré, attesté et inconnu.

## Complétude des exigences
- [x] Aucune question bloquante non résolue n’est laissée dans le besoin.
- [x] Les 35 exigences sont vérifiables et les 11 critères de succès mesurables.
- [x] Les sept histoires couvrent contraintes, comptes, charge, chaîne de choix, reprise et Git.
- [x] Les cas limites, dépendances et exclusions sont explicites.
- [x] La compatibilité et l’indépendance de T3 sont requises.
- [x] Le petit décideur de secours appartient bien au périmètre complet.

## Préparation de la suite
- [x] Les scénarios d’acceptation permettent un test indépendant de chaque histoire.
- [x] Aucun statut Implemented ou succès de test n’est anticipé.
- [x] Les détails techniques et bornes seront arbitrés dans le plan et les contrats.
- [x] Les états inconnus, les crédits et les droits ne sont jamais assimilés à une autorisation.

## Notes
Revue documentaire initiale et corrections ciblées r1 terminées. Les cases attestent la qualité du besoin seulement.
Corrections intégrées : charge/workload explicite hors HandoffV1, SHA attendu aussi en Shared Git,
accès catalogue local sans T3, sûreté OAuth safe-mode, borne globale128, efforts natifs null,
recettes Laya compatible versus12+LLM/score et garantie de non-injection limitée aux données automatiques.
Le plan doit encore passer l’audit de réutilisation et la revue contradictoire avant génération des tâches.
