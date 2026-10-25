# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/170-minreq-3-et-serde-saphyr.fr.md](../../../../../decisions/170-minreq-3-et-serde-saphyr.fr.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `cargo build -p bridget-daemon`, avec un compilateur C disponible.
- Tests ciblés : `reprise::tests::pin_yaml`, `decision_service`, `t3code` et `t3code_opaque_identifiers149_test`.
- `cargo tree -p bridget-daemon -d` pour détecter les doublons de versions.
