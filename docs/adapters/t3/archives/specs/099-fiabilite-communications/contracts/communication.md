# Archives techniques de l’adaptateur T3

Document d’origine : [specs/099-fiabilite-communications/contracts/communication.md](../../../../../../../specs/099-fiabilite-communications/contracts/communication.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

CancelDelivery s'applique aussi au transport t3code. Une demande non encore démarrée
est retirée sans ouvrir un nouveau tour fournisseur. Aucune promesse d'interruption
atomique d'un tour accepté par le fournisseur.
Les rappels doux/fermes ne deviennent pas des tours t3code supplémentaires ;
l'expiration conserve son état terminal et utilise un contrôle structuré.
Les réponses attendent une confirmation ; les états terminaux du daemon permettent
de réconcilier un accusé perdu. Le redémarrage ne réexécute pas un tour pour refaire
une réponse déjà préparée.
Le journal ne coupe pas un texte sans signal ; un refus d'écriture n'avance pas
silencieusement le repère d'observation.
