# Archives techniques de l’adaptateur T3

Document d’origine : [specs/165-cycle-vie-liens-t3/validation/runtime-before.md](../../../../../../../specs/165-cycle-vie-liens-t3/validation/runtime-before.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-14
Pont : PID 66786, démarré le 2026-10-14 à 19:23:19 CEST
Serveur T3 : PID 30234, `http://127.0.0.1:3773`

<a id="extrait-2"></a>

## Extrait 2

Les 834 sous-agents terminés restent dans l'historique T3. Ils ne doivent pas rester connectés à Bridget.

<a id="extrait-3"></a>

## Extrait 3

La boucle ouvre un lien complet pour chaque fil T3 non archivé. Elle ne distingue pas une conversation principale d'un sous-agent historique terminé. Le nombre de descripteurs croît donc avec l'historique des délégations.
