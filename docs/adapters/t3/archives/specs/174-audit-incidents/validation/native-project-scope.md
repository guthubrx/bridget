# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/native-project-scope.md](../../../../../../../specs/174-audit-incidents/validation/native-project-scope.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

`apps/server/src/orchestration/decider.ts` rattache un fil à `projectId`,
indépendamment de son `worktreePath`. La comparaison de projet lors d'une
reprise de plan porte sur cette identité logique. Bridget réutilise ce principe
pour sa relation native déjà admise. T3 ne fournit pas la preuve de placement
multi-hôte de Bridget : celle-ci et les droits hérités restent obligatoires.
