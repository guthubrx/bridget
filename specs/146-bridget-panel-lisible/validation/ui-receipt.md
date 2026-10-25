# Preuve T005/T006 — Panneau146

Date : 2026-10-08. Résultats fournis par le responsable UI et revérifiés par le principal. Le responsable documentaire a lu la transcription RED et recalculé les deux hashes ; il n'a pas relancé les tests.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/validation/ui-receipt.md#extrait-1).

Commande de test : `./node_modules/.bin/vp test run src/components/BridgetPanel.test.tsx --project unit`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/validation/ui-receipt.md#extrait-2).

GREEN intermédiaire à 06:31:33 :26 PASS/0 FAIL, PTY96207/chunk236b4d. Une correction ultérieure de focus et son test ajoutent une assertion de non-régression ; ce résultat et ses hashes sont historiques.

GREEN final à 06:37:47 :27 PASS/0 FAIL, PTY16761/chunkc31842. Contrôle de type web `../../node_modules/.bin/tsc --noEmit` exit0. Fmt sur les deux fichiers exit0. Lint exit0 avec trois warnings hérités145 et aucun nouveau. Diff check exit0. Les validations intégrées et le build ont été refaits sur ces dernières sources, après détection de deux changements concurrents par hash. Ces tests seuls ne prouvent pas encore toute la recette navigateur T008.

Hashes source relus :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/validation/ui-receipt.md#extrait-3).

## Correction du focus prouvée dans le navigateur

Le principal a observé RED : le toggle dépliage focalisé disparaissait lors d'un élargissement du panneau et le focus retombait sur BODY. La correction conserve une référence au bouton Copie et y transfère le focus avant retrait du toggle, sans nouvel état ni polling. GREEN observé lors du passage240 →520 px : focus sur « Copier le message132 » ; copie exacte des1746 octets UTF-8 du corps ; Entrée déplie puis replie. Aucune capture image disponible ; ce constat rapporté ne remplace pas le reçu final d'aperçu T008.

## Réutilisation et minimalisme

Le helper local BridgetDate emploie Intl en français et les composants Tooltip/time natifs. Le tooltip global comporte des ordinaux anglais ; sa réutilisation directe ne donnerait pas les libellés demandés.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/validation/ui-receipt.md#extrait-4).
