# Specification Quality Checklist: Attente et boîte de fin de tâche

**Purpose**: Valider la complétude et la qualité de la spec avant la planification
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
- [ ] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [ ] No implementation details leak into specification

## Notes

- Case non cochée « technology-agnostic » : SC-008 et SC-011 citent des outils
  (`bridget_task_cancel`, `wait_unsupported`) et une borne mesurable. Ce sont des noms
  du contrat public. Aucun nom de fonction Rust n'y figure.
- Case non cochée « implementation details » : FR-028 (`tracked_requests.reply_summary`),
  FR-031 (`DaemonIdentityReport`, `features`) et FR-033 (`tool_timeout_sec`) nomment des
  champs et un réglage. Ils sont gardés dans la spec parce que le comportement
  testable en dépend (résumé figé, compatibilité d'un ancien daemon, délai des hôtes).
  Le détail d'implémentation vit dans `plan.md` et `data-model.md`.
- Cas limites couverts : résultat publié entre deux lectures (FR-005), client coupé par
  l'hôte (FR-009, FR-029), attente expirée puis fin de l'enfant (FR-006, FR-020),
  tâche d'un autre parent (FR-002), pause (FR-010), 65e attente et 5e attente d'une
  façade (FR-008, FR-012), annulation MCP (FR-013), ancien daemon (FR-031), réponse
  déjà injectée (FR-030), réponse sans résumé (FR-028), deux scopes avec le même
  identifiant de message (SC-009).
- Trois questions ouvertes au plus, avec une valeur par défaut chacune
  (section « Questions pour l'utilisateur »).
- Révision après contre-revue Codex (verdict BLOCKED, 8 points) : l'attente est une
  lecture pure. Les exigences d'écriture de livraison ont disparu.
- Non vérifié à ce stade : délai d'outil réel de Codex et de Claude Code, absence de
  délai d'inactivité sur les connexions du daemon, tests qui figent les clés exactes
  de `task_view`, nom de la lecture de phase de 150. Ces points sont des vérifications
  de la tâche T001.

## Levée explicite des deux réserves avant implémentation

Les deux réserves de forme sont levées pour ce run. Les noms des outils et leurs
limites définissent le contrat public mesurable. Les champs privés cités dans les
FR sont conservés comme contraintes de fiabilité déjà choisies et expliquées dans
le plan. Aucun besoin fonctionnel ne reste ambigu. Cette levée ne dispense pas
des tests de résumé figé, de compatibilité et de délai des hôtes.
