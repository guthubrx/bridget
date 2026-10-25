# Vérifications 180

Code : file native, édition/annulation/promotion par appareil, volet au-dessus du composeur ; lien préalable retiré. Version0.6.1.

Rust : dernier lot mobile77PASS/1ignored ; guidage6PASS. Un passage intermédiaire avait71PASS/6FAIL/1ignored, car le harnais de recette avait créé le dossier sessions avec des permissions trop larges. Répertoire immédiatement remis en0700 et fichiers du seul harnais en0600. Rejeu complet du lot affecté77PASS/1ignored. Pas de suite globale.

Android : JVM20PASS/0échec, lintRelease et assembleRelease passent. Un test instrumenté QueueStoreTest passe sur Pixel10a Android17 dans le paquet de recette distinct, ensuite désinstallé. Les XML privés et leur archive vérifiée sont conservés.

Recette réelle Android vers un destinataire natif contrôlé : envoi normal, volet replié/déplié, édition persistante, annulation, guidage explicite, remise au repos avec application arrêtée, réouverture. Trois commandes natives : un cancelled, deux acked ; témoin destinataire : exactement deux IDs uniques, aucun message annulé. Le moteur de recette est contrôlé et ne lance aucun modèle. Le guidage Codex/T3 est qualifié par les tests ciblés et la lecture de son adaptateur ; aucun nouvel envoi de test dans un vrai fil utilisateur.

Preuves : diaporama vérifié,13étapes/7clics,338660octets. Première pulsation/curseur visibles ; lecture/pause, précédent/suivant et dernière étape vérifiés dans T3. Aucun message réel des autres fils dans les images versionnées. Captures originales, manifestes et témoins natifs hors Git. Avant du lien non capturé.

Les réglages de filtre et favoris ont été remis. Pixel laissé sur Commander Bridget depuis Android. Aucun test envoyé aux autres conversations. Agent de recette fermé proprement. La revue indépendante et ses limites figurent dans revue.md.
