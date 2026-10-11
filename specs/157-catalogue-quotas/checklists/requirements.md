# Specification Quality Checklist: Catalogue fiable, quotas et remplacement

**Purpose**: Valider la complétude de la spec avant la planification
**Created**: 2026-10-11
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain (3 questions posées avec défaut)
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

- Les noms d'outils (`bridget_capabilities`, `bridget_delegate`) et les codes de
  refus sont le contrat public. Ils restent dans la spec.
- Les chemins de code cités dans « Besoin et périmètre » sont des preuves des
  défauts, pas des choix de conception.
- Cas limites couverts : fait de quota périmé, quota inconnu, fenêtre sans heure
  de retour, modèle absent (pas de remplacement), doublon dans `fallback`,
  rejeu avec substitution, heure de retour trop lointaine, annulation pendant
  l'attente, redémarrage du daemon, pause pendant l'attente.
- La reprise sur quota est sortie de la 157 (session future 160). Ses six
  exigences d'entrée sont dans la spec.
