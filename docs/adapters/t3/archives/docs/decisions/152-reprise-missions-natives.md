# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/152-reprise-missions-natives.md](../../../../../decisions/152-reprise-missions-natives.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les anciennes captures restent conservées. Git, installation sur disque
et processus chargés doivent être prouvés séparément à la livraison.
L'activation est coordonnée par le principal sans interrompre les autres
fils T3 ; aucun redémarrage T3 n'est autorisé par cet ADR.
