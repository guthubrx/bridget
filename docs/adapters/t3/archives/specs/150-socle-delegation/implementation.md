# Archives techniques de l’adaptateur T3

Document d’origine : [specs/150-socle-delegation/implementation.md](../../../../../../specs/150-socle-delegation/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `run_maintenance` complet (il écrit dans `$HOME/Library/Logs`) : seul
  `maintain_stores` est testé.
- Aucun lecteur T3 réel ; aucun daemon réel ; aucune relance.
- Les 8 échecs préexistants de la suite complète (hors délégation) ne sont pas
  diagnostiqués : ils appartiennent au fil « Complément149 identité durable ».
