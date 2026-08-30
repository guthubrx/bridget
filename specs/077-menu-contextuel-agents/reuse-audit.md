# Audit de réutilisation de l'existant - SPEC-077 Menu contextuel des agents

## Decision

Statut: PASS
Date: 2026-08-30
Feature dir: /home/moi/bridget-referent/.worktrees/session-077-menu-contextuel-agents/specs/077-menu-contextuel-agents

Conclusion courte: Le plan transforme la fiche globale existante au lieu d'ajouter un second menu. Les règles de cycle de vie, l'identité runtime, le positionnement, la lecture et les patterns de stockage sont réutilisés. Aucun endpoint, service, dépendance ou doublon évident n'est proposé.

## Synthèse

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 8 |
| Items audités | 8 |
| Réutilisations déjà prévues | 7 |
| Existants potentiellement pertinents | 0 |
| Duplications évidentes | 0 |
| Règles et mémoires applicables | 6 |
| Specs existantes applicables | 4 |

## Réutilisations correctement identifiées

| Item du plan | Existant réutilisé | Preuve | Commentaire |
|---|---|---|---|
| Menu global | identityCard attachée au body | crates/bridget-daemon/assets/ui/app.js:5362 | Transformer ce nœud, ne pas en créer un second |
| Identité compacte | identityCardData et catalogue runtime | crates/bridget-daemon/assets/ui/app.js:3035 | Conserver la source factuelle SPEC-071 |
| Positionnement | identityCardPosition | crates/bridget-daemon/assets/ui/app.js:3055 | Accepte également un rectangle ponctuel |
| Trois points | renderAgentButton et agent-row__actions | crates/bridget-daemon/assets/ui/app.js:5876, crates/bridget-daemon/assets/ui/theme.css:395 | Étendre le déclencheur existant |
| Cycle de vie | agentLifecycleEligibility et routes SPEC-075 | crates/bridget-daemon/assets/ui/app.js:3088, specs/075-cycle-vie-agents/contracts/agent-lifecycle-v1.md:50 | Aucun contrat nouveau |
| Marquer lu | markSelectedReadIfEligible et readThrough | crates/bridget-daemon/assets/ui/app.js:6243 | Étendre à une commande et persister le curseur |
| Préférences locales | largeur et apparence dans localStorage | crates/bridget-daemon/assets/ui/app.js:5277, crates/bridget-daemon/assets/ui/app.js:5800 | Réutiliser le pattern de fallback |
| Agents masqués | section Agents arrêtés | crates/bridget-daemon/assets/ui/index.html:46, crates/bridget-daemon/assets/ui/theme.css:1069 | Réutiliser details, summary et rendu de ligne |

## Existant potentiellement pertinent non mentionné

| Item du plan | Existant proche | Preuve | Décision attendue |
|---|---|---|---|
| Aucun | Aucun | Recherche contextmenu, épinglage et masquage sans résultat | Aucune |

## Duplications évidentes

| Item proposé | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | Tous les mécanismes proches sont explicitement étendus | Aucune |

## Mémoires et règles applicables

| Source | Règle | Impact sur le plan |
|---|---|---|
| /home/moi/.speckit/constitution.md Article III | cycle SpecKit complet | pipeline jusqu'à audit |
| /home/moi/.speckit/constitution.md Article XVI | worktree dédié | branche et worktree 077 |
| /home/moi/.speckit/constitution.md Article XVIII | complexité bornée | projection au plus O(n log n) |
| /home/moi/.speckit/constitution.md Article XIX | réutiliser avant créer | transformation ciblée de la fiche |
| /home/moi/.speckit/constitution.md Article XX | charge future | aucune dépendance et modèle unique |
| /home/moi/.speckit/ref/standards-tests.md | tests traçables | tests nommés SPEC-077 dans app.js |

## Specs livrées applicables

| Spec | Pattern déjà établi | Impact |
|---|---|---|
| SPEC-069 | lecture et stabilité du défilement | ne pas recharger ni faire sauter la colonne |
| SPEC-071 | identité runtime et logos fournisseurs | réutiliser identityCardData |
| SPEC-073 | trois points, fiche globale et confirmation | transformer, ne pas doubler |
| SPEC-075 | stop, relaunch et decommission | réutiliser matrice, routes et verdicts |

## Journal de recherche

| Requête | Portée | Résultat |
|---|---|---|
| rg identityCard, renderAgentButton | assets UI | fiche globale et bouton existants |
| rg agentLifecycleEligibility, /v1/agents | code et specs | contrat lifecycle complet existant |
| rg localStorage | app.js | deux patterns robustes existants |
| rg markSelectedReadIfEligible, readThrough | app.js | curseur de lecture de session existant |
| rg stopped-agents | HTML et CSS | section repliée réutilisable |
| rg contextmenu, agent-sidebar-preferences | crates et specs proches | aucun mécanisme concurrent |
| rg dépendances | Cargo manifests | aucune dépendance frontend nécessaire |

## Arbitrages

| Sujet | Décision | Justification | Date |
|---|---|---|---|
| Composant de menu | réutiliser | la fiche globale résout déjà clipping et cycle de vie | 2026-08-30 |
| Modèle d'actions pur | créer dans app.js | aucune matrice locale existante, elle porte les règles UX des trois voies | 2026-08-30 |
| Préférences locales v1 | créer dans app.js | aucune préférence de barre existante, pas de besoin serveur | 2026-08-30 |
| Section masquée | réutiliser le pattern arrêté | garantit une restauration sans nouvelle page | 2026-08-30 |
| Dépendance externe | ne pas créer | DOM et CSS natifs suffisent | 2026-08-30 |

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les règles projet applicables ont été lues
- [x] Les specs existantes proches ont été vérifiées
- [x] Le plan.md réutilise les existants ou justifie les créations
