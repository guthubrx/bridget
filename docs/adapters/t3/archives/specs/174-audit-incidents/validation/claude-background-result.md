# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/claude-background-result.md](../../../../../../../specs/174-audit-incidents/validation/claude-background-result.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Lecture de `ClaudeAdapter.ts` (événements `task_started`, `task_updated`,
`task_notification`), `ThreadBackgroundLiveness.ts` et `ThreadSettlementPolicy.ts`
dans T3. T3 sépare la fin du tour de la fin de ses tâches et suspend la clôture
quand une tâche reste active. Le SDK installé documente aussi les tâches Bash.
Le binaire Claude installé émet ces événements pour les tâches enregistrées.

<a id="extrait-2"></a>

## Extrait 2

La revue utilisait un SDK plus ancien. Les champs `ambient`,
`task_started.is_backgrounded` et `patch.status=paused` sont présents dans
le SDK 0.3.260 lu dans T3 et ne sont pas des champs inventés. Le suivi ne
suppose pas une garantie d’ordre des événements de progression.

<a id="extrait-3"></a>

## Extrait 3

Worker Cartae activé sur `174-be00ace5-opt`, reconnexion attestée dans le
journal maître. Le service conserve `KillMode=process` et `SendSIGKILL=no`.
Les quatre enfants encore actifs ont conservé leur PID et leur naissance ;
leurs douze processus observés ont survécu. Les trois autres processus de
l’instantané appartenaient à une tâche déjà terminale avant l’activation.
Aucune nouvelle mission applicative, reprise ou modification Opus2D.
Le maître, le pont et T3 n’ont pas été relancés.
