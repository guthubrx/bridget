# Archives techniques de l’adaptateur T3

Document d’origine : [specs/131-relais-visible/spec.md](../../../../../../specs/131-relais-visible/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Problème observé (données réelles)
01/10 05:13:26Z, fil du coordinateur `opus_city_ai` : une demande avec réponse attendue d'un autre
agent ; la réponse finale du coordinateur, relayée à cet agent, est restée affichée dans le fil de
l'utilisateur sans repère et en tutoyant l'autre agent. À 05:32:24Z l'utilisateur demande :
« C'est à moi que tu parles ou c'est à la jambe bridget ». Ce n'était pas un raisonnement interne :
le raisonnement est déjà rangé à part par T3.
