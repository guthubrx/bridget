# Archives techniques de l’adaptateur T3

Document d’origine : [specs/180-file-mobile/validation/verification.md](../../../../../../../specs/180-file-mobile/validation/verification.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Recette réelle Android vers un destinataire natif contrôlé : envoi normal, volet replié/déplié, édition persistante, annulation, guidage explicite, remise au repos avec application arrêtée, réouverture. Trois commandes natives : un cancelled, deux acked ; témoin destinataire : exactement deux IDs uniques, aucun message annulé. Le moteur de recette est contrôlé et ne lance aucun modèle. Le guidage Codex/T3 est qualifié par les tests ciblés et la lecture de son adaptateur ; aucun nouvel envoi de test dans un vrai fil utilisateur.

<a id="extrait-2"></a>

## Extrait 2

Preuves : diaporama vérifié,13étapes/7clics,338660octets. Première pulsation/curseur visibles ; lecture/pause, précédent/suivant et dernière étape vérifiés dans T3. Aucun message réel des autres fils dans les images versionnées. Captures originales, manifestes et témoins natifs hors Git. Avant du lien non capturé.
