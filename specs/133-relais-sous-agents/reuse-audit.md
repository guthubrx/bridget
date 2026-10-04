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

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Marqueur enfant privé | écriture atomique de marqueur typé | `crates/bridget-daemon/src/mcp_identity.rs:52`, `crates/bridget-daemon/src/mcp_identity.rs:414` | Même protection, schéma séparé pour l'autorité réduite. |
| Résolution MCP/CLI | résolution de filiation bornée | `crates/bridget-daemon/src/mcp_identity.rs:233`, `crates/bridget-daemon/src/mcp_identity.rs:337` | Étendre le parcours existant, ne pas créer un résolveur concurrent. |
| Inventaire fournisseur | `ProcessEvidence` et inventaire T3 | `crates/bridget-daemon/src/t3code_identity.rs:20`, `crates/bridget-daemon/src/t3code_identity.rs:531` | Les PID, naissances, sessions et lignées sont déjà collectés. |
| Parent unique | `select_bindings` | `crates/bridget-daemon/src/t3code_identity.rs:437` | Réutiliser la correspondance unique du principal. |
| Détection des fournisseurs imbriqués | `primary_provider_processes` | `crates/bridget-daemon/src/t3code_identity.rs:608` | Étendre la classification au lieu de rescanner l'OS. |
| Admission des outils | répartiteur MCP unique | `crates/bridget-daemon/src/mcp.rs:286`, `crates/bridget-daemon/src/mcp.rs:409` | Refus avant l'appel du handler. |
| Actions enfant permises | handlers `send` et `who` | `crates/bridget-daemon/src/mcp.rs:446`, `crates/bridget-daemon/src/mcp.rs:540` | Aucun nouvel outil public. |
| Provenance | champ optionnel de l'enveloppe existante | `crates/bridget-core/src/message.rs:64`, `crates/bridget-core/src/message.rs:115` | Même pattern que `thread_notice`, avec compatibilité serde. |
| Idempotence | sérialisation canonique existante | `crates/bridget-daemon/src/communication.rs:34`, `crates/bridget-daemon/src/communication.rs:74` | Ajouter la provenance au contrat canonique seulement. |
| Rendu destinataire | trois points de rendu existants | `crates/bridget-transport/src/acp.rs:2065`, `crates/bridget-transport/src/codex_app_server.rs:1298`, `crates/bridget-daemon/src/t3code.rs:3060` | Aucun moteur de rendu supplémentaire. |

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

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 064 | propriété parent-enfant | Le parent reste propriétaire, sans réutiliser le registre durable. |
| 090 | sous-agents internes sans identité | La session 133 remplace uniquement l'interdiction totale par un relais borné. |
| 098 | pont T3 en lecture seule | La preuve vient de l'inventaire existant ; T3 n'est pas modifié. |
| 099 | preuve auxiliaire privée | La connexion au daemon réutilise la preuve du parent après admission locale. |
| 101 | rattachement T3 par session native | Même source de preuve et mêmes bornes. |
| 114 | fournisseurs imbriqués exclus du principal | Les imbriqués restent exclus du principal et reçoivent une preuve distincte. |
| 115 | CLI résout une identité attestée | La CLI doit maintenant refuser explicitement la preuve enfant. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg AgentPidMarker, ResolvedIdentity` | `crates/bridget-daemon` | marqueur et résolution uniques trouvés. |
| `rg ProcessEvidence, primary_provider_processes` | `crates/bridget-daemon` | inventaire T3 et exclusion des imbriqués trouvés. |
| `rg bridget_send, bridget_who, dispatch_with_executor` | `crates/bridget-daemon` | point d'admission unique et handlers existants. |
| `rg BridgetMessage, canonical_message_control` | `crates/` | enveloppe et idempotence existantes. |
| `rg prompt_for, communication_prompt, envelope` | `crates/` | trois rendus à étendre. |
| `rg sous-agents, parent, preuve de rattachement` | `specs/` | sessions 064, 090, 099, 101, 114, 115 pertinentes. |

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
