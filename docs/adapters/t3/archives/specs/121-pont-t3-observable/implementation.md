# Archives techniques de l’adaptateur T3

Document d’origine : [specs/121-pont-t3-observable/implementation.md](../../../../../../specs/121-pont-t3-observable/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Décisions
- Tour spontané : preuve par couverture (un message de la page est antérieur à la demande du tour)
  et absence de message utilisateur à son horodatage. Origine enregistrée `t3-turn:<id>` : jamais un
  identifiant de message, donc jamais appariée à une demande. Nos notifications sont toujours des
  messages, un tour spontané ne peut pas en provenir : pas de boucle d'observation.
- Réponse incertaine : bornée au message utilisateur suivant (une notification intercalée coupe
  aussi : préférer moins que le texte d'autrui) ; aucune si rien n'a été écrit.
- Journal : les deux derniers avertissements répétés du chemin de projection passent par le
  signalement unique et différé de la 117.
