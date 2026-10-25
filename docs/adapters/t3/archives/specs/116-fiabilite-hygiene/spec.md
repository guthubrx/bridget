# Archives techniques de l’adaptateur T3

Document d’origine : [specs/116-fiabilite-hygiene/spec.md](../../../../../../specs/116-fiabilite-hygiene/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**A. Des réponses se perdent.** 16 demandes suivies abandonnées depuis le 14/09 sur six fils,
« au repos sans appariement certain ». Le pont apparie message et tour par rang, et ne voit les tours
qu'à travers leurs messages écrits. Sur le fil `horizon-terra-captation` : 38 messages, 38 tours
selon T3, 34 tours écrits. Sur `opus_city_ai` (Claude) : 58 messages, 139 tours écrits (réveils
d'arrière-plan). Les points de contrôle T3 ne sont pas une alternative : absents sur 5 fils hors
Git, et non un pour un sur 9 fils sur 20.
