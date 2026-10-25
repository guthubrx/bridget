# Archives techniques de l’adaptateur T3

Document d’origine : [audits/2026-10-07/session-2026-10-07-spec141-01/scoring.md](../../../../../../../audits/2026-10-07/session-2026-10-07-spec141-01/scoring.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- T009 : une expression de terminaison optionnelle CR/LF pouvait consommer le CR appartenant à un corps comme A\r. Le parseur retire maintenant exactement le séparateur déduit de l'en-tête canonique. Des séparateurs structurels mélangés imposent le repli sûr. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:129 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.test.ts:334.
- T010 : un en-tête canonique au-delà de 1024 caractères sortait de la projection compacte, mais sa copie suivait alors le chemin qui remplace les références par leurs labels. La copie reconnaît désormais ce seul en-tête canonique séparément, sans activer un rendu compact. Preuves : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.logic.ts:101 et /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.tsx:2268 ; test /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web/src/components/chat/MessagesTimeline.test.tsx:396.

<a id="extrait-2"></a>

## Extrait 2

La capacité native preview_resize a expiré deux fois. La recette a donc utilisé un iframe de 320 pixels dans le même aperçu T3, sans navigateur alternatif. Cet audit a inspecté les deux captures finales ; il n'a pas exécuté les interactions du principal. Captures :

<a id="extrait-3"></a>

## Extrait 3

- /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp14lx-25fd3477.png
- /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp1vid-2e2033d4.png

<a id="extrait-4"></a>

## Extrait 4

- Corps HTML traité comme texte par le renderer existant, parseRawHtml=false.
- Copie originale complète conservée, même avec lien t3-context et en-tête canonique long.
- Repli entier sur ambiguïté, sans expéditeur ou message inventé.
- Pas de nouveau transport, stockage, fournisseur ou modification d'Agent Loop.
- Réutilisation du renderer, de la copie, de la clé fil/message et de l'ancrage existants.
