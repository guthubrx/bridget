# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/165-cycle-vie-liens-t3.md](../../../../../decisions/165-cycle-vie-liens-t3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le pont T3 ouvrait un lien Bridget pour chaque fil non archivé. L'historique courant contient 1 121 sous-agents terminés. Ces liens sans travail détenaient chacun des sockets, des lecteurs, un worker et un journal. Le pont atteignait donc sa limite de fichiers ouverts. Une hausse de cette limite retardait seulement la panne.

<a id="extrait-2"></a>

## Extrait 2

T3 V2 publie la relation réelle dans `threads[].lineage.relationshipToParent`. Les valeurs prises en charge sont `subagent`, `fork` et l'absence de relation pour une racine. T3 V1 ne publie pas ce champ.

<a id="extrait-3"></a>

## Extrait 3

- La filiation vient du contrat T3. Aucun titre ni identifiant ne sert d'heuristique.
- Une relation V2 inconnue ou mal formée invalide le snapshot. Si T3 ajoute une
  nouvelle relation, le pont s'arrête explicitement jusqu'à la mise à jour de
  ce contrat fermé.
- La présence historique d'un fil ne suffit pas à ouvrir un lien.
- Une activité non terminale ouvre ou conserve le lien au rafraîchissement suivant.
- Une remise reçue pendant le drain interdit une fermeture fondée sur une ancienne clé.
- Une obligation durable n'ouvre jamais plus de 32 liens de reprise simultanés.
- Chaque retrait de la table des liens finalise le worker, le lecteur et le flux possédés.
- Aucun nouveau service, stockage, nettoyeur ou réglage de limite n'est ajouté.

<a id="extrait-4"></a>

## Extrait 4

Le coût de chaque snapshot reste linéaire dans le nombre de fils. Le nombre de liens dépend désormais des conversations principales et du travail réellement actif, plus un plafond de 32 reprises. Les conversations terminées restent dans T3 comme historique. Elles ne détiennent plus de ressources Bridget.
