# Checklist exigences — Maicie v3

**Purpose**: Vérifier la qualité de la spécification avant implémentation.  
**Created**: 2026-08-22  
**Feature**: `specs/011-maicie-orchestration/spec.md`

## Périmètre et autonomie

- [x] CHK001 Les objectifs sont créés explicitement, jamais par un message direct.
- [x] CHK002 Le mode collaboratif et le mode délégué sont distingués sans verrouiller la conversation.
- [x] CHK003 GUI, TUI, DSH, T3 Code, DAG et scheduler sont explicitement hors périmètre MVP.
- [x] CHK004 Tous les futurs fichiers source Maicie appartiennent à `plugins/maicie/`.

## Frontière Bridget / Maicie

- [x] CHK005 Bridget est propriétaire de la présence, livraison, timeout et demande suivie.
- [x] CHK006 Maicie est propriétaire de l'objectif, délégation, profil et décision.
- [x] CHK007 Aucune base SQLite n'est partagée ni lue directement entre les deux couches.
- [x] CHK008 Chaque délégation possède une outbox et un message_id persistés avant I/O Bridget.

## État et sécurité

- [x] CHK009 Les faits ACP viennent de Subscribe 008 avec séquence, fraîcheur et Gap/End ; aucun rapport sémantique MVP.
- [x] CHK010 `bloqué` et `besoin_de_décision` ne sont jamais inférés d'un terminal ou d'un silence.
- [x] CHK011 Une permission ACP est affichée comme auto-décidée par 007, jamais comme attente humaine.
- [x] CHK012 Le réveil exige une approbation mono-usage TOCTOU-safe et SpawnOrder 009 ; Maicie ne spawn rien.

## Vérifiabilité

- [x] CHK013 Chaque user story possède un test indépendant et des scénarios Given/When/Then.
- [x] CHK014 Les critères de succès sont mesurables et couverts par une tâche.
- [x] CHK015 Les tâches utilisent une checklist, un identifiant et un chemin précis.
- [x] CHK016 La reprise après redémarrage, le DND, les absences et l'échec de livraison sont couverts.
