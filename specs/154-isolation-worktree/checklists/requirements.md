# Specification Quality Checklist: Isolation optionnelle de l'enfant dans un worktree Git

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

- Les noms d'outils (`bridget_delegate`, `bridget_task_status`), le paramètre
  `workspace` et les codes de refus (`workspace_not_git`, etc.) sont le contrat
  public du produit. Les chemins de code cités dans « Besoin et périmètre » sont
  des preuves de l'état actuel.
- Le nom des commandes Git de la liste blanche est une contrainte utilisateur
  (Bridget ne gère rien d'autre côté Git), pas un détail d'implémentation.
- Cas limites couverts : refus sans reste, rejeu, retour arrière après refus
  tardif, lien symbolique, injection d'option, worktree sale, annulation,
  panne entre résultat et nettoyage, purge 150, collision par chemins distincts,
  enfant sans droit de commit, dossier du parent en sous-répertoire.
- Révision après contre-revue : Bridget ne supprime jamais ; Git hors verrou ; dépôts à
  configuration exécutable refusés. Ces choix restent des règles de produit
  vérifiables, pas des détails d'implémentation.
- Trois questions pour l'utilisateur, avec défaut noté : paires Claude vers
  Claude, droits Git de l'enfant sous `workspace-write`, emplacement et
  ignorance de `.worktrees/`.
- Décisions laissées au plan : délai de Git réglé par la mesure T002, forme de la
  ligne de fin du résultat (alignement possible avec la session151).
