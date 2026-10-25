# Plan de revue hostile — SPEC145

Date : 2026-10-07. Statut : procédure préparée ; aucun verdict d'implémentation.

## Disponibilité réelle

Le constat `who same_project` transmis par le principal ne montre que l'agent Codex lui-même. Aucun agent d'un autre fournisseur n'est joignable dans ce périmètre au moment de la planification. Ne pas inventer une validation indépendante, un échange interprovider ou un APPROVE. Une revue locale séparée du travail d'implémentation peut examiner le diff ; elle doit déclarer sa portée et ses limites.

## Périmètre de la revue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/adversarial-review-plan.md#extrait-1).

## Attaques et preuves à examiner

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/adversarial-review-plan.md#extrait-2).

## Scoring factuel

La revue prévue inclut audit v14 en mode fix, avec `auto_commit=false`. Les artefacts restent sous `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/audits/2026-10-07/session-2026-10-07-spec-145-01/`. Une première convergence en lecture seule précède l'audit. Après corrections et rejeux, effectuer un dernier cycle d'audit en lecture seule, puis la convergence finale. Converge peut seulement ajouter les tâches manquantes attestées ; sinon les tâches restent byte-identiques pendant la comparaison. Écrire les reçus append-only hors phase Converge, après comparaison. Les mises à jour de statuts et cases se font ensuite, dans une phase documentaire séparée. Ce plan ne remplace pas le protocole audit v14 par une mini revue locale.

Attribuer une note sur20 : autorisation5 ; absence mutation5 ; exactitude/pagination4 ; intégration/accessibilité4 ; contrôle de charge/erreurs2. Pour chaque point, citer test ou observation réelle et fichier exact. Une exigence sans preuve reste non vérifiée, même si le code paraît correct.

Un défaut d'accès hors appartenance, de mutation humaine, de secret exposé, de génération modèle ou de réponse hors contexte bloque la livraison. Un défaut majeur de pagination, texte exact, clavier ou compatibilité exige correction et rejeu. Les défauts mineurs doivent être expliqués et ne peuvent pas masquer une exigence manquante.

Le verdict final peut être APPROVE, APPROVE_WITH_CHANGES ou REQUEST_CHANGES selon les preuves. Il appartient à la revue réalisée, pas à ce plan. La convergence principale compare ensuite exigences, tâches et résultats. Une absence de relecteur interprovider est une limite déclarée, pas un verdict favorable.
