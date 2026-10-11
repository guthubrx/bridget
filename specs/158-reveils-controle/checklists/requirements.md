# Specification Quality Checklist: Contrôle humain, sécurité de l'appelant et réveils (158)

**Purpose**: Valider la complétude de la spec avant la planification
**Created**: 2026-10-11
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] Pas de détail d'implémentation superflu (les noms d'outils, de refus et de commandes sont le contrat public)
- [x] Centré sur la valeur pour l'humain et pour l'agent appelant
- [x] Écrit pour un lecteur non spécialiste (phrases courtes)
- [x] Toutes les sections obligatoires présentes

## Requirement Completeness

- [x] Aucun marqueur [NEEDS CLARIFICATION] (3 questions posées avec défaut)
- [x] Exigences testables (FR-001 à FR-028, budget de tours retiré)
- [x] Critères de succès mesurables (SC-001 à SC-008)
- [x] Critères de succès sans détail d'implémentation (sauf noms de refus publics)
- [x] Scénarios d'acceptation définis (US1 à US4 en 158a ; US5 à US8 en 158b)
- [x] Cas limites identifiés (client agent négocié, A-B-rejeu de l'arrêt, arrêt + pause, rejeu pendant l'arrêt, outil ajouté par 151 à 157)
- [x] Périmètre borné (liste « Hors périmètre », webhook justifié)
- [x] Dépendances et hypothèses identifiées (150, 151, ADR 027, ADR 011)

## Feature Readiness

- [x] Chaque exigence 158a a au moins une tâche (voir `analyze-report.md`)
- [x] Les scénarios couvrent les flux principaux
- [x] Les critères de succès relient aux critères visés (Contrôle humain 3→4, Sécurité 4→5)
- [x] 158b reste en spec seule, sans fuite dans le plan ni les tâches

## Notes

- Le découpage 158a/158b est décidé dans la spec. 158b ne bloque pas 158a.
- Le webhook HTTP est déclaré hors périmètre avec justification (règle socket Unix seulement).
- La demande de secret est spécifiée et reportée : aucun consommateur Bridget n'existe (question 3).
- Budget de tours retiré après la contre-revue (compteur non attesté pour Codex) : question 3.
- Les lignes de code citées viennent du checkout principal. Elles sont à relire après 150 et 151.
