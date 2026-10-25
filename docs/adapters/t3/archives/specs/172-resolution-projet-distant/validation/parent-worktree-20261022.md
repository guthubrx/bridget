# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/parent-worktree-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Fichier lu : /Users/moi/11.Repositories/t3code/apps/server/src/vcs/GitVcsDriverCore.ts, fonction resolveRepositoryPathsUncached, lignes 1038–1110 au moment de la lecture. T3 distingue commonDir et worktreeRoot depuis le dossier demandé. Bridget reprend cette séparation. Le contrôle de portée du parent et la confirmation distante restent nécessaires pour son exécution distante.

<a id="extrait-2"></a>

## Extrait 2

- Worker Cartae : PID 2219043, zéro redémarrage, build 172-76b682d6.
- Mission Notre-Dame : 28da9844-0695-4f99-9159-b9beeab9bd6d, même instance et génération 714.
- Wrapper : PID 2350358, naissance 218843575, inchangés.
- Fournisseur Claude : PID 2351171, naissance 218844336, inchangés.
- Pont : PID 898 ; T3 : PID 28528 et 28591, préservés.

<a id="extrait-3"></a>

## Extrait 3

La lecture durable après activation donne result_available, ronde 1, error=null pour cette mission. Cela confirme la continuité de cette mission pendant le remplacement du maître. Aucune annulation, duplication ou nouvelle mission métier n’a été envoyée par ce correctif. Le worker et T3 n’ont pas été redémarrés.
