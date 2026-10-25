# Archives techniques de l’adaptateur T3

Document d’origine : [specs/120-garde-doublon-t3/implementation.md](../../../../../../specs/120-garde-doublon-t3/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Incident déclencheur (05:18Z)
Doublon `T3 Code (Alpha).app` (pid 8275, serveur 8352, port 3774) ouvert par un agent via `getApp`.
Tours de GLM3 (`dfccceef`) et GLM4 (`106521cc`) marqués en erreur à 05:18:59Z, nouveaux tours
`pending` sans `started_at` ; processus Claude 1286 et 8742 (enfants du serveur Local 22214) actifs
d'après leurs transcrits. Pont resté sur 3773 (garde 114), mais appariement impossible. Doublon
quitté par l'agent ; fichier d'état réécrit à la main.
