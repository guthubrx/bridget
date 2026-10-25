# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/contracts/rattachement-reprise.md](../../../../../../../specs/166-rattachement-bridget-reprise/contracts/rattachement-reprise.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Un tour Codex va démarrer pour un fil T3 qui possède une preuve privée Bridget courante.

<a id="extrait-2"></a>

## Extrait 2

- Le `ThreadId` T3 ne change pas.
- L’identifiant natif Codex ne change pas.
- L’historique n’est ni remplacé ni dupliqué.
- Une preuve n’est jamais partagée entre deux fils.
- Une preuve ancienne ne reçoit pas les droits du nouveau run.
- Aucun repli global n’est autorisé.
