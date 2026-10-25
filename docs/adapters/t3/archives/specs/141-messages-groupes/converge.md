# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/converge.md](../../../../../../specs/141-messages-groupes/converge.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-2"></a>

## Extrait 2

Projection : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts
Tests de projection : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts
Rendu : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx
Tests de rendu : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx
Recette et contrôles : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/implementation.md
Résultats structurés : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/validation/results.json

<a id="extrait-3"></a>

## Extrait 3

Le principal a inspecté visuellement les deux captures sombre/claire du nom long dans 320 pixels. Le bouton Copy message du vrai T3 a été cliqué via le navigateur natif. L'appel de copie a été intercepté localement puis restauré ; le presse-papiers système a été préservé. La chaîne capturée de 3421 caractères est strictement égale au texte JSON SQLite de la fixture spec141-valid-batch. Aucun contenu synthétique n'a été imprimé.
