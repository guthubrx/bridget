# Validation141 — Guide ciblé

## Préconditions

Worktree T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes
Utiliser des fixtures synthétiques. Aucun accès en écriture aux conversations actives. Les commandes ci-dessous sont prévues ; leur présence ne signifie pas qu'elles ont réussi.

## Tests de projection et de rendu

```bash
cd /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web
./node_modules/.bin/vp test run --project unit src/components/chat/MessagesTimeline.logic.test.ts src/components/chat/MessagesTimeline.test.tsx
../../node_modules/.bin/tsc --noEmit
./node_modules/.bin/vp build
```

Format et lint ciblés sur les fichiers modifiés :

```bash
cd /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped
./node_modules/.bin/vp fmt --check apps/web/src/components/chat/MessagesTimeline.logic.ts apps/web/src/components/chat/MessagesTimeline.logic.test.ts apps/web/src/components/chat/MessagesTimeline.tsx apps/web/src/components/chat/MessagesTimeline.test.tsx
./node_modules/.bin/vp lint apps/web/src/components/chat/MessagesTimeline.logic.ts apps/web/src/components/chat/MessagesTimeline.logic.test.ts apps/web/src/components/chat/MessagesTimeline.tsx apps/web/src/components/chat/MessagesTimeline.test.tsx
```

## Recette du composant réel

1. Monter la timeline avec un lot direct de trois messages synthétiques, dont deux ont le même nom.
2. Vérifier une ligne au repos. Ouvrir le groupe : seule la première section est ouverte. Ouvrir une seconde section. Vérifier les noms, l'ordre, l'extrait littéral borné à 120 et les corps entiers. Refermer puis ouvrir le groupe conserve les choix.
3. Refaire le parcours avec Tab, Entrée et Espace. Vérifier focus et états annoncés.
4. Copier le message complet et comparer au texte original row.message.text. Inclure [x](t3-context://v1/skill/ctx_1) sans contexte structuré : le lien doit rester dans la copie du lot valide ou en repli sûr. Vérifier la copie actuelle des autres familles, puis pièces jointes et actions existantes. Vérifier aussi un UUID seul visible sans nom inventé.
5. Essayer compteur faux, indices répétés, séparateur dans le corps, suffixe inconnu et en-tête trop long. Vérifier le texte entier et l'absence de panneau source.
6. Vérifier thèmes clair/sombre, largeur 320, corps long, nom long, ancrage et changement de fil/message.
7. Vérifier les cinq familles SPEC140. Seul le lot direct change. Les autres détails techniques restent accessibles comme auparavant.

Consigner les résultats et leurs limites. Un rendu de maquette seul ne valide pas le composant. Le dossier Converge doit relier les exigences aux preuves, puis l'audit doit relire le diff ciblé.

Baseline exécutée par le principal : 269 tests PASS dans les deux suites, 4,53 secondes. La commande pnpm exec a tenté une réinstallation et échoué avec ERR_PNPM_ABORTED_REMOVE_MODULES_DIR_NO_TTY. Utiliser directement le binaire déjà installé. Les dépendances sont liées au checkout principal ; ne pas les réinstaller.

Dans le dossier /Users/moi/11.Repositories/t3code-local/.worktrees/141-bridget-grouped/apps/web, le binaire tsc n'est pas disponible sous node_modules/.bin. Utiliser la commande ../../node_modules/.bin/tsc --noEmit ci-dessus. Elle cible le projet web depuis son dossier courant et n'exécute pas le contrôle global du monorepo.

Passage historique avant le gel : 288 tests PASS, 211 de logique et 77 de rendu. La recette et le code étaient encore en cours à ce stade. Le passage final sur les sources gelées donne 294 tests PASS, 215 de logique et 79 de rendu, en 2,84 secondes. Les 294 tests ont ensuite été fraîchement rejoués avant livraison. Format/lint/types/build web, recette isolée, Converge et audit sont validés. L'application adjacente est installée et vérifiée ; la présentation n'est pas activée dans T3 en cours.

## Livraison autorisée, sans relance de T3

Le 2026-10-07, l'utilisateur a autorisé commit, fusion, push et installation de la session141. Il a interdit de relancer T3. Le principal a installé et vérifié /Applications/T3 Code (Local SPEC141).app à côté de l'application active /Applications/T3 Code (Local).app. Aucun quit, open ou restart effectué. Les preuves figurent dans le journal ci-dessous.

Activation future : quand l'utilisateur quittera volontairement l'ancienne application, ouvrir /Applications/T3 Code (Local SPEC141).app. Une simple relance de /Applications/T3 Code (Local).app ne charge pas141. Aucun lancement n'est effectué par cette documentation.

Journal de livraison : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/141-messages-groupes/specs/141-messages-groupes/livraison.md
