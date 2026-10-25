# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/quickstart.md](../../../../../../specs/141-messages-groupes/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Worktree T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes
Utiliser des fixtures synthétiques. Aucun accès en écriture aux conversations actives. Les commandes ci-dessous sont prévues ; leur présence ne signifie pas qu'elles ont réussi.

<a id="extrait-2"></a>

## Extrait 2

```bash
cd /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web
./node_modules/.bin/vp test run --project unit src/components/chat/MessagesTimeline.logic.test.ts src/components/chat/MessagesTimeline.test.tsx
../../node_modules/.bin/tsc --noEmit
./node_modules/.bin/vp build
```

<a id="extrait-3"></a>

## Extrait 3

```bash
cd /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
./node_modules/.bin/vp fmt --check apps/web/src/components/chat/MessagesTimeline.logic.ts apps/web/src/components/chat/MessagesTimeline.logic.test.ts apps/web/src/components/chat/MessagesTimeline.tsx apps/web/src/components/chat/MessagesTimeline.test.tsx
./node_modules/.bin/vp lint apps/web/src/components/chat/MessagesTimeline.logic.ts apps/web/src/components/chat/MessagesTimeline.logic.test.ts apps/web/src/components/chat/MessagesTimeline.tsx apps/web/src/components/chat/MessagesTimeline.test.tsx
```

<a id="extrait-5"></a>

## Extrait 5

Dans le dossier /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web, le binaire tsc n'est pas disponible sous node_modules/.bin. Utiliser la commande ../../node_modules/.bin/tsc --noEmit ci-dessus. Elle cible le projet web depuis son dossier courant et n'exécute pas le contrôle global du monorepo.

<a id="extrait-6"></a>

## Extrait 6

Passage historique avant le gel : 288 tests PASS, 211 de logique et 77 de rendu. La recette et le code étaient encore en cours à ce stade. Le passage final sur les sources gelées donne 294 tests PASS, 215 de logique et 79 de rendu, en 2,84 secondes. Les 294 tests ont ensuite été fraîchement rejoués avant livraison. Format/lint/types/build web, recette isolée, Converge et audit sont validés. L'application adjacente est installée et vérifiée ; la présentation n'est pas activée dans T3 en cours.

<a id="extrait-7"></a>

## Extrait 7

Le 2026-10-07, l'utilisateur a autorisé commit, fusion, push et installation de la session141. Il a interdit de relancer T3. Le principal a installé et vérifié /Applications/T3 Code (Local SPEC141).app à côté de l'application active /Applications/T3 Code (Local).app. Aucun quit, open ou restart effectué. Les preuves figurent dans le journal ci-dessous.

<a id="extrait-8"></a>

## Extrait 8

Activation future : quand l'utilisateur quittera volontairement l'ancienne application, ouvrir /Applications/T3 Code (Local SPEC141).app. Une simple relance de /Applications/T3 Code (Local).app ne charge pas141. Aucun lancement n'est effectué par cette documentation.
