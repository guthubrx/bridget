# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/158-controle-global-portees.md](../../../../../decisions/158-controle-global-portees.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La vue Lineage exige un fil T3 et ne remplace pas une vue globale humaine.
La pause de l’ADR027 diffère les effets autonomes ;elle n’est pas un arrêt
persistant des nouvelles délégations. Le catalogue MCP n’impose pas de portée
par action. Un enfant discovery peut rester vivant entre rondes ou après un
résultat ; un plafond limité aux tâches non terminales serait donc insuffisant.
