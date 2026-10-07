# Audit de reutilisation de l'existant — 141-messages-groupes

## Decision

Statut: PASS
Date: 2026-10-07
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes

Conclusion courte: le plan étend la projection et le corps Bridget existants. Il réutilise le bouton de copie avec le texte original pour les seuls lots directs et conserve les chemins de pièces jointes et d'ancrage. Aucun service, composant concurrent, modèle persistant ou endpoint n'est reconstruit. Le gate documentaire a été validé avant génération des tâches.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 10 |
| Items audites | 10 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 8 |
| Specs existantes applicables | 4 |

## Reutilisations correctement identifiees

Chaque ligne correspond à un item extrait de la section Réutilisation de l'existant du plan. Les deux suites de tests sont auditées séparément.

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Type de projection | BridgetEnvelopePresentation | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:86 | Extension facultative du type existant. |
| Projection pure | projectBridgetEnvelope | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 | Validation locale et corps source inchangé. |
| Branche des lots | Projection messages/notifications | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:163 | Modifier le cas messages seulement. |
| Carte existante | BridgetMessageBody | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4034 | Sections intégrées dans la carte, pas de seconde timeline. |
| Corps Markdown | UserMessageBody | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4117 | Corps complet, rendu existant sans HTML brut exécuté. |
| Clé fil/message | Clé de CollapsibleUserMessageBody | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2239 | Isolation d'état déjà présente. |
| Ancrage | ctx.onToggleWorkEntry | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4049 | Repli des sections utilise la même notification de hauteur. |
| Copie, pièces jointes et actions | UserTimelineRow et MessageCopyButton | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:1975 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2264 | Aucun nouveau bouton ; utiliser row.message.text pour lots directs seulement. La transformation actuelle en /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/packages/shared/src/composerContextReferences.ts:102 ne garantit pas la copie intégrale. |
| Tests de projection | Suite existante projectBridgetEnvelope | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:208 | Ajouter cas réels et ambigus avec titres SPEC141. |
| Tests de rendu | Suite existante MessagesTimeline | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:287 | Vérifier copie exacte, état et absence de sources pour lots seulement. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Aucun | Aucun item omis identifié | Recherche source et specs ci-dessous | Aucun arbitrage requis. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucune duplication proposée | Dix items du plan étendent les chemins existants | Aucune. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/AGENTS.md | Français, session validée, préservation des travaux actifs | Artefacts français et worktrees dédiés ; aucun redémarrage. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/.specify/memory/constitution.md | Pont vers la constitution globale | Constitution globale lue, pas de copie dans les artefacts. |
| /Users/moi/.speckit/constitution.md | Articles III, XVI, XVIII, XIX, XX et XXI | Cycle complet ; complexité linéaire ; réutilisation ; logs courts ; périmètre strict. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/.specify/memory/standards.md | Références à charger selon contexte | Références frontend, tests et qualité lues. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/.specify/memory/user-layer.json | Couche utilisateur et compatibilité | Sync déjà effectuée ; scripts/templates absents documentés. |
| /Users/moi/.speckit/ref/standards-frontend.md | TypeScript strict et état local ; prescriptions Next génériques | Réutiliser React T3 ; exception ciblée Next/frontend-v2/i18n dans le plan. |
| /Users/moi/.speckit/ref/standards-tests.md | Tests de comportement et champ Tests | Tests: 0/0 à la création ; Vitest T3 au lieu de Pytest ; scenarios d'acceptation explicites. |
| /Users/moi/.speckit/ref/code-quality-details.md | Contrats, validation aux frontières et dépendances | Projection pure, repli sûr, aucune dépendance ; aucun état persistant. |

Aucun fichier MEMORY.md Bridget n'a été trouvé dans /Users/moi/.Codex/projects lors de la recherche préparatoire. La mémoire relue comprend les artefacts livrés de SPEC140. Aucune mémoire inexistante n'est supposée.

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/114-joignabilite/spec.md | Tour contenant un lot direct sans réponse attendue | Le format existant est lu en /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:3033 ; transport conservé. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/134-noms-humains-messages/spec.md | Nom humain dans chaque en-tête de lot | Lire les noms remis ; aucune recherche de noms dans une base active. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/139-entetes-complets/spec.md | Reconnaissance d'enveloppe complète et conservation du brut | Reconnaissance bornée et repli sur les formats incomplets. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/140-bulles-compactes/plan.md | Carte compacte, projection pure, copie, état et ancrage | Réutiliser tout le chemin ; retirer les détails uniquement pour les lots directs selon demande explicite. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| rg --files puis lecture de la mémoire et des specs 114/134/139/140 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes | Patterns compatibles ; aucun autre plan concurrent trouvé pour cette présentation. |
| rg -n 'batch_envelope\|BatchFamily\|sender_label' puis lecture ciblée | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/crates | Producteur direct et notifications distincts ; aucun transport nouveau. |
| Lecture projection et recherche BridgetMessageBody/onToggleWorkEntry/copie | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat | Dix points d'extension déjà prévus par le plan. |
| Lecture package.json et suites de tests | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped et son package web | Vite Plus/Vitest déjà disponibles ; baseline principal 269 PASS. |
| Inventaire .specify et recherche MEMORY.md | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/.specify et /Users/moi/.Codex/projects | Aucun script/template local ni mémoire Bridget identifié. |
| Lecture maquette et instructions utilisateur | /Users/moi/.cache/bridget-grouped-mockup-20261007/index.html | Retenir sections et noms ; remplacer sujets par extraits ; exclure sources. |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Carte et projection | réutiliser | Responsabilité déjà couverte par SPEC140. | 2026-10-07 |
| Sources et sujets de maquette | exclure les sources et utiliser le corps littéral | Demande utilisateur explicite ; aucun fait déduit. | 2026-10-07 |
| Première ouverture | ouvrir le premier message seulement puis conserver les choix | Précision de la maquette validée transmise par le principal. | 2026-10-07 |
| Copie des références | transmettre row.message.text aux seuls lots directs | Revue indépendante : les références t3-context sans contexte structuré sont sinon transformées. | 2026-10-07 |
| UUID seul | conserver l'UUID visible | Aucun nom humain fourni ; ne pas inventer « Bridget ». | 2026-10-07 |
| Standards Next/Pytest | conserver React/Vitest T3 | Exception ciblée écrite dans le plan ; aucun framework nouveau. | 2026-10-07 |
| Primitives SpecKit absentes | appliquer le protocole manuellement | Le gate reste documenté ; aucune phase supprimée. | 2026-10-07 |
| Outils de validation | appeler les binaires installés directement | pnpm exec a tenté une réinstallation ; dépendances partagées préservées. | 2026-10-07 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Le statut PASS concerne l'audit de réutilisation. Il ne signifie pas que l'implémentation ou la recette sont terminées. Le principal a lu et validé ce gate avant la génération des tâches.
