# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/remaining-final.md](../../../../../../../specs/174-audit-incidents/validation/remaining-final.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- macOS SHA-256: de497df7e374fea50f6baf9762a851b0cb1b745c3911d58972980455645a4941.
- Linux SHA-256: 34f4260661b6325aa41d57d1ff034706acef8c94602a5e0d7a9f339024c81ffc.
- Master PID after activation: 17132. Worker PID: 4063096.
- The active remote business wrapper retained PID 4045899 and its process birth value. T3 and its bridge were not restarted. Existing client processes keep their loaded client code until their normal restart; daemon-side corrections are already active.
