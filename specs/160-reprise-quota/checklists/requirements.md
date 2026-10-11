# Specification Quality Checklist: Reprise d'une mission après une limite d'usage

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

- Les six exigences d'entrée issues de la contre-revue de la 157 sont chacune
  couvertes par une exigence et un test rouge :
  compteur durable (FR-008, T007, T022), reconstruction ciblée (FR-013, T004, T018),
  identité de relance (FR-012, T008, T018, T020), cause distincte (FR-010, T006, T021),
  gel cumulé (FR-016, T005, T023), gestionnaire de `DeliveryRejected`
  (FR-006, T003, T015).
- Les noms de paramètre (`resume_on_limit`) et de codes (`usage_limit`) sont le
  contrat public. Les chemins de code cités sont des preuves, pas des choix de
  conception.
- Cas limites couverts : fait de quota avant ou après l'échec, fait trop ancien,
  limite Codex globale, retour passé ou trop lointain, 3e échec, crash à chaque
  frontière de la relance, pause qui chevauche l'attente, annulation, redémarrage
  pendant la décision et pendant l'attente, enfant absent.
- Points non vérifiés en réel, tracés : le chemin d'échec d'un vrai refus de
  quota (T002), les primitives de 152 sur le code livré (T001).
