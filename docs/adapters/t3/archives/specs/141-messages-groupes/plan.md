# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/plan.md](../../../../../../specs/141-messages-groupes/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Statut : Implemented, installé, activation différée. Gate de réutilisation, Converge passe 2 et audit final validés.
Début de préparation : 07:30 CEST. Estimation totale : 20–35 minutes, hors livraison de production.
Branche Bridget : session-141-messages-groupes.
Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes
Code T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
Maquette : /Users/moi/.cache/bridget-grouped-mockup-20261007/index.html

<a id="extrait-2"></a>

## Extrait 2

Le frontend T3 utilise React, TypeScript et Vite Plus/Vitest. SPEC140 fournit une projection pure puis BridgetMessageBody dans la timeline existante. Le format Rust est déjà produit par batch_envelope. Aucun changement Rust n'est prévu.

<a id="extrait-3"></a>

## Extrait 3

| Élément | Source vérifiée | Changement prévu |
|---|---|---|
| Type de projection | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:86 | Ajouter une collection facultative de messages directs projetés. |
| Projection pure | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:105 | Étendre la branche des lots directs avec un découpage conservateur. |
| Branche des lots | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:163 | Garder le comportement des notifications. |
| Carte existante | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4034 | Ajouter les sections dans BridgetMessageBody, sans seconde timeline. |
| Corps Markdown | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4119 | Réutiliser UserMessageBody pour chaque corps complet et le repli sûr. |
| Clé fil/message | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2239 | Conserver l'isolation de l'état local. |
| Ancrage | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:4049 | Appeler ctx.onToggleWorkEntry lors des replis imbriqués. |
| Copie et actions | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2264 | Réutiliser MessageCopyButton avec row.message.text original pour les seuls lots directs ; préserver les autres copies et les pièces jointes. |
| Tests existants | /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:208 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:287 | Étendre ces suites avec les cas SPEC141. |

<a id="extrait-5"></a>

## Extrait 5

Les prescriptions génériques Next.js, frontend-v2, next-intl et Pytest ne correspondent pas au frontend T3 React/Vite/Vitest vérifié. Les tests sont ajoutés aux suites existantes avec un titre SPEC141. Aucun framework, dossier de traduction ou outil de test n'est ajouté pour satisfaire une règle d'un autre projet. Les contrôles restent sur les fichiers et le package web concernés. Aucun lint, test ou build global du monorepo n'est imposé. Cette exception suit le périmètre explicite de la session et évite une correction de travaux étrangers.

<a id="extrait-6"></a>

## Extrait 6

Aucune décision architecturale nouvelle ni reconstruction de composant. L'implémentation et la recette ne redémarrent ni n'installent la production. La livraison autorisée ensuite forme une phase distincte, sans relance de T3. Aucun nettoyage d'autres branches ou worktrees. Les artefacts SpecKit restent dans le dépôt Bridget ; le dépôt T3 reçoit uniquement les changements d'implémentation nécessaires.

<a id="extrait-7"></a>

## Extrait 7

Le statut Implemented ne signifie pas activé dans l'application en cours. La session141 a suivi commit, fusion, push et installation autorisés, sans relancer T3. Le principal a installé et vérifié /Applications/T3 Code (Local SPEC141).app à côté de /Applications/T3 Code (Local).app, restée active et intacte. Les preuves sont consignées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/livraison.md. L'utilisateur devra fermer volontairement l'ancienne application puis ouvrir la nouvelle pour activer141. Relancer l'ancienne ne charge pas ces changements.
