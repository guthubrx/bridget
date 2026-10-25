# Audit de reutilisation de l'existant — 133-relais-sous-agents

## Decision

Statut: PASS
Date: 2026-10-04
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/133-relais-sous-agents/specs/133-relais-sous-agents

Conclusion courte: le plan étend les preuves, l'inventaire, le répartiteur et les
rendus existants. Aucun service, registre durable, commande ou dépendance n'est
recréé. La preuve enfant séparée est nouvelle parce que le marqueur principal
existant accorde une autorité plus large et ne peut pas porter le contrat borné.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 1 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Cycle de vie enfant | liens durables d'agents lancés par Bridget | `specs/064-plan-controle-bridget-maicie/spec.md:299` | Ne pas réutiliser comme identité : un sous-agent fournisseur n'est pas un agent lancé par Bridget. Réutiliser seulement le principe de propriété parent. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Recherche code et specs | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `AGENTS.md` | worktree et tests isolés | Session 133 dans `.worktrees/`, aucun daemon installé. |
| `.specify/memory/constitution.md` | cycle SpecKit complet | Plan, audit, tâches, analyse, implémentation et convergence obligatoires. |
| `/Users/moi/.speckit/constitution.md` | moindre privilège et minimalisme | Deux outils, aucune identité enfant durable. |
| `/Users/moi/.speckit/ref/agent-orchestration.md` | le parent reste responsable | Réponses et routage reviennent au parent. |
| `/Users/moi/.speckit/ref/standards-tests.md` | preuve causale et régressions | Tests RED/GREEN des refus et de la provenance. |
| `/Users/moi/.speckit/ref/code-quality-details.md` | pas d'abstraction spéculative | Structures limitées à la preuve et au champ de provenance. |

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/reuse-audit.md#extrait-2).

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/133-relais-sous-agents/reuse-audit.md#extrait-3).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Marqueur principal ou enfant | créer un schéma enfant dans la mécanique existante | Le marqueur principal prouve une autorité complète et rendrait le refus CLI impossible. | 2026-10-04 |
| Lien durable 064 | ne pas créer de `AgentLink` | L'enfant interne n'est pas lancé ni possédé par la flotte Bridget. | 2026-10-04 |
| Outils autorisés | réutiliser `who` et `send` seulement | Couvre la communication sans contrôle ni accès aux données du parent. | 2026-10-04 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
