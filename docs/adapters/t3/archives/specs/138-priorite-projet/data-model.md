# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/data-model.md](../../../../../../specs/138-priorite-projet/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Fait attaché à une connexion et son instance déjà attestées. Champs minimaux:
hôte attesté, racine canonique commune, origine de preuve T3 ou wrapper.
La clé de comparaison dérive de l'hôte et de la racine. Aucun identifiant de
registre runtime retiré n'est accepté comme autorité.

<a id="extrait-2"></a>

## Extrait 2

Le client de fond porte la racine explicite sur sa propre connexion négociée.
Le daemon valide ce contexte sans créer d'agent et sans emprunter d'identité T3.
Un auxiliaire conserve le fait parent. Un run ancien sans racine reste inconnu.
L'outbox fige racine, motif et destinataire avant la première tentative ; un
rejeu ne les reprend pas depuis la configuration courante du run.
Le résultat publié, son result_path et son archive restent cohérents sous le
verrou commun existant. Une publication concurrente ne devient pas une archive
obsolète après une réservation CAS. T020 couvre cet interleave.
