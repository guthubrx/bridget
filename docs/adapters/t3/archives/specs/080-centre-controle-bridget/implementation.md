# Archives techniques de l’adaptateur T3

Document d’origine : [specs/080-centre-controle-bridget/implementation.md](../../../../../../specs/080-centre-controle-bridget/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La finition de l'overlay conserve les fonds sombres et les variables de couleurs Bridget. Le gabarit des réglages est en revanche porté du composant de réglages T3 Code installé :

<a id="extrait-2"></a>

## Extrait 2

Les contrôles restent des éléments HTML natifs et accessibles. Seule leur présentation a été adaptée, aucune couleur de fond Bridget n'a été remplacée par le thème T3.

<a id="extrait-3"></a>

## Extrait 3

- Le paquet corrigé doit être installé dans `/Applications/Bridget.app` seulement après fermeture explicite de l'instance en cours, puis validé visuellement avec Loin et Carte.
- La validation humaine de la vue Typographie après le portage T3 reste requise. Aucun résultat de clic ou de capture externe ambiguë n'est compté comme acceptation.
- Les coûts restent indisponibles sans tarifs versionnés. Les filtres avancés, la courbe quotidienne et les autres réglages serveur écrits restent des tâches non cochées dans `tasks.md`.
