# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/implementation.md](../../../../../../specs/177-bridget-mobile/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

48 tests Rust mobile177 passent après les tests de migration du magasin et de refus sans configuration Google. Build Android succès ; sur le Pixel, 13 cas exécutables passent, le cas conditionnel d’arrêt externe reste distinct. Aucun envoi FCM réel effectué. Le navigateur MCP voit un runtime serveur non connecté, alors que l’utilisateur confirme sa connexion dans le navigateur local T3. Le contrôle natif local est indisponible (Accessibilité refusée) ; ne pas demander encore une reconnexion à l’utilisateur. La recette T033 reste ouverte.
