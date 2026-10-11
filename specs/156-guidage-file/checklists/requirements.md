# Specification Quality Checklist: Guidage, interruption et file d'attente

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-10-11
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
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

- Les noms d'outils (`bridget_send`, `bridget_queue`), de valeurs (`steer`,
  `interrupt`) et de refus (`invalid_intent`, `intent_not_permitted`,
  `interrupt_rate_limited`, `mission_active`, `referent_paused`,
  `steer_unavailable`) sont le contrat public du produit.
- Les chemins de code de « Besoin et périmètre » sont des preuves de constats.
- Cas limites couverts : trame brute avec intention, origine `Human` forgée,
  preuve enfant, cible d'un autre projet, soi-même, boucle d'interruptions,
  pause (dont l'interruption de pause interne), enfant en mission, tête de file
  du pont bloquée, fournisseur T3 sans guidage natif, rejeu d'une clé quand la
  cible change d'état, base 150 migrée.
- Périmètre réduit après contre-revue : `cancel`, `reorder`, `promote` sont
  hors périmètre (saga durable), justifié dans la spec et l'ADR.
- Trois questions ouvertes avec défaut noté. Aucune ne bloque la planification.
- Cadre de menace explicite (ADR 011) : agent par ses outils dans le modèle ; processus hostile à trames brutes hors modèle, limites prouvées dans la spec.
- Points de sécurité signalés : `Send` et `SendIdempotent` laissent passer
  `intent` et `origin` bruts ; `ControlExecution` n'a pas de contrôle de droits.
