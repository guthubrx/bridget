# Preuve T005/T006 — Panneau146

Date : 2026-10-08. Résultats fournis par le responsable UI et revérifiés par le principal. Le responsable documentaire a lu la transcription RED et recalculé les deux hashes ; il n'a pas relancé les tests.

Racine des commandes : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web`.

Commande de test : `./node_modules/.bin/vp test run src/components/BridgetPanel.test.tsx --project unit`.

RED à 06:25:45 :18 PASS/2 FAIL sur20. L'UI héritée145 demandait `list` plutôt que `list_recent` et n'affichait pas les éléments de date courte et détails attendus. Preuve : `/Users/moi/.cache/t3-spec146-preview.650MHJ/ui-red-captured-transcript.txt`, transcription verbatim de la sortie PTY32976/chunkfbb74b, explicitement pas une nouvelle exécution.

GREEN intermédiaire à 06:31:33 :26 PASS/0 FAIL, PTY96207/chunk236b4d. Une correction ultérieure de focus et son test ajoutent une assertion de non-régression ; ce résultat et ses hashes sont historiques.

GREEN final à 06:37:47 :27 PASS/0 FAIL, PTY16761/chunkc31842. Contrôle de type web `../../node_modules/.bin/tsc --noEmit` exit0. Fmt sur les deux fichiers exit0. Lint exit0 avec trois warnings hérités145 et aucun nouveau. Diff check exit0. Les validations intégrées et le build ont été refaits sur ces dernières sources, après détection de deux changements concurrents par hash. Ces tests seuls ne prouvent pas encore toute la recette navigateur T008.

Hashes source relus :

- `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx` : `b77cb990970b7f521084a444642bec4179013c38b1409a243d0bbf384cbb92d8`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx` : `8c6a3cde1eed7d8a5477d35fc9de8385036065e7e0b81ad69dc4e4846ab06f98`.

## Correction du focus prouvée dans le navigateur

Le principal a observé RED : le toggle dépliage focalisé disparaissait lors d'un élargissement du panneau et le focus retombait sur BODY. La correction conserve une référence au bouton Copie et y transfère le focus avant retrait du toggle, sans nouvel état ni polling. GREEN observé lors du passage240 →520 px : focus sur « Copier le message132 » ; copie exacte des1746 octets UTF-8 du corps ; Entrée déplie puis replie. Aucune capture image disponible ; ce constat rapporté ne remplace pas le reçu final d'aperçu T008.

## Réutilisation et minimalisme

Le helper local BridgetDate emploie Intl en français et les composants Tooltip/time natifs. Le tooltip global comporte des ordinaux anglais ; sa réutilisation directe ne donnerait pas les libellés demandés.

Le composant local BridgetMessage réunit le corps exact, le dépliage, la mesure du débordement par ResizeObserver, la copie native et les détails HTML. Aucun framework, module générique ou dépendance n'est créé. Button, Tooltip, ScrollArea, Input et la copie restent natifs T3. Le corps original unique sert à l'affichage, la recherche et la copie.
