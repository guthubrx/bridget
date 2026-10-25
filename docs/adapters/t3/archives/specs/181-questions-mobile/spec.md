# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/spec.md](../../../../../../specs/181-questions-mobile/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

### US5 P1 — Adaptateurs et indépendance
Les questions natives Bridget fonctionnent avec dans un environnement natif isolé. Les modes terminal capables d’un échange structuré utilisent ce parcours. Les questions natives T3 sont présentées par un adaptateur qui conserve leur demande d’origine. Les terminaux purement interactifs sans callback ni montage Bridget ne sont pas pilotés par simulation de touches : leur limite est explicite et un lancement compatible est indiqué.
Acceptation : une recette native complète dans un environnement natif isolé, puis une recette de l’adaptateur T3. Aucun changement du contenu des questions, aucun redémarrage de fournisseur pour répondre.
