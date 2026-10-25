# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le client MCP courant ne transmet que `code` et `detail`. Les textes constants sont donc aussi placés dans `detail.message`, déjà transmis par ce client. Aucun rechargement de ce client ni redémarrage T3 ne sont nécessaires pour rendre le conseil lisible. Aucune logique de production du client MCP n’a changé.

<a id="extrait-2"></a>

## Extrait 2

Lecture de /Users/moi/11.Repositories/t3code-local/apps/server/src/mcp/OrchestratorMcpService.ts :

<a id="extrait-3"></a>

## Extrait 3

Mécanismes repris : distinguer état et droits, conserver une explication et ne pas relancer depuis une erreur. T3 ne couvre pas les phases `WorkerObservation` de Bridget. La preuve de phase distante reste donc propre à son worker. Le transport MCP existant de Bridget via `detail` permet la compatibilité avec les clients déjà chargés.

<a id="extrait-4"></a>

## Extrait 4

Le maître Mac a été activé sur le build 172-222941e6, source 222941e6d655bc7d3d0735327601c5f992da0755. Binaire signé SHA-256 0b0ad4d076554f81082773360193b70f926d66a276b5c10011bd5857d99bfd46. Nouveau PID 39365. T3, son serveur, le pont et le worker Cartae gardent leurs PID et dates de naissance ; l’ASAR T3 reste identique. Le contrôle avant activation ne trouvait aucun wrapper distant ni commande worker active. Aucun processus métier n’a été interrompu.
