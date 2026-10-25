# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/implementation-b-foundations-annexe.md](../../../../../../specs/159-edition-file/implementation-b-foundations-annexe.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T024/T025 : hunks documentaires préparés, aucune application WT ni tâche cochée.
Syntaxe fondée sur les candidats CLI/MCP T022, pas sur un runtime validé.
Le reçu de promotion atteste le commit ; effect unconfirmed, repli FIFO,
rejet si pleine, doublon Codex possible après erreur ambiguë. Aucun avis
universel neuf pour reply=false. held_queue reste fermé ; T3, edit et159c différés.
