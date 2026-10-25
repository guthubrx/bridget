# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/contracts/thread-link-lifecycle.md](../../../../../../../specs/165-cycle-vie-liens-t3/contracts/thread-link-lifecycle.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Un résumé de fil validé à la frontière HTTP T3.

<a id="extrait-2"></a>

## Extrait 2

- Le résumé respecte le schéma V1 ou V2 déjà accepté.
- La filiation, si elle existe, vient du contrat T3. Elle n’est jamais déduite du titre.
- Une relation V2 présente accepte seulement `subagent`, `fork` ou `null`. Toute autre forme est invalide.
