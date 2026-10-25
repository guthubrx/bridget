# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/source-arguments.md](../../../../../../../specs/174-audit-incidents/validation/source-arguments.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Source fe18a02a18ff87654e4b3108faeada4e471d5ede was pushed to main.
Optimized clients built and installed on macOS and Cartae. A fresh MCP process
from each installed executable returned the corrected schema through
initialize / notifications/initialized / tools/list. The macOS signature was
verified. No daemon, worker, provider or T3 restart was performed. Already-open
MCP processes keep their loaded catalog until their normal restart.
