# Recherche — SPEC-166

## Observations

Le pont Bridget répond pour le fil courant. Il refuse le fil repris avec `t3_session_unavailable`. Le lien du fil repris était pourtant exposé avant et après le redémarrage du pont. Le défaut ne vient donc ni d’un lien absent, ni de la limite de fichiers corrigée par la session 165.

Le fil repris et le fil courant utilisent la même session fournisseur Codex partagée. La session privée Bridget du fil repris a été remplacée après le chargement de son fil natif.

## Lecture du code

- `threadRuntimeParams()` prépare la configuration MCP et mémorise la preuve montée.
- `resumeThread()` recharge cette configuration dans le fil Codex natif.
- `startNativeTurn()` compare la preuve mémorisée avec la preuve courante.
- En cas de différence, `startNativeTurn()` invalide les droits, puis lance quand même le tour.

## Décision

Réutiliser `thread/resume` juste avant le tour lorsque la preuve diffère. Cette opération garde le même fil et son historique. Elle charge la preuve courante dans le processus qui va exécuter le tour.

## Alternatives écartées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/research.md#extrait-1).

## Conclusion

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/research.md#extrait-2).
