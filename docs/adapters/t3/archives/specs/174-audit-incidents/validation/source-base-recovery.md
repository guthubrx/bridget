# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/source-base-recovery.md](../../../../../../../specs/174-audit-incidents/validation/source-base-recovery.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Livraison : source 41ade47a7d6061d75d4d0df67b4827e8e33e7d5f poussée sur main.
Binaire macOS optimisé et signé ; SHA-256
28b4db9d1b0699d9d0e5c0da5006699b238723ccd4b2e773c6344d7ef1f92a95.
Daemon actif confirmé par le MCP officiel : build 41ade47a7d60.
Une reconnexion MCP transitoire pendant activation, puis accès rétabli.
Les processus T3 et worker distant sont conservés. Les missions distantes
encore actives gardent leur instance et leur état working sans erreur.
Une mission est devenue result_available sans erreur entre les observations.
Aucune mission métier créée ou rejouée pour cette validation.
Conseils transmis au parent demandeur par Bridget ; lecture non confirmée.
Entretien du cache effectué par le wrapper, sauvegarde précédente conservée.
