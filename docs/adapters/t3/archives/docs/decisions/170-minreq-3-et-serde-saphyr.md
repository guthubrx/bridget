# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/170-minreq-3-et-serde-saphyr.md](../../../../../decisions/170-minreq-3-et-serde-saphyr.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

The session 174 validation records the pin, decision-service, T3, permission and transport tests, the complete remote delegation path and the dependency scan. Builds and tests use the repository's scripts/build.py wrapper. The dependency graph is checked with cargo tree; no new HTTP client or unsafe TLS option is introduced.
