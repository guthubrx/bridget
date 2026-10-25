# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/git-diagnostics.md](../../../../../../../specs/174-audit-incidents/validation/git-diagnostics.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 renvoie une erreur Git explicite et n'écrit aucun reçu distant.
Bridget écrit un reçu `ready=false` avant `git worktree add`. Le reçu doit donc être nettoyé quand l'absence est prouvée, et conservé sinon.
