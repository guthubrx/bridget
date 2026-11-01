# Plan de correction
Réutiliser la version Cargo, Gradle, la configuration Firebase publique existante,
le certificat et la lignée de signature de production. Construire sur Cartae.
Remplacer uniquement les deux actifs Android de v0.7.1 après validation.
Conserver les quatre actifs serveurs et les données Android.
Ajouter une commande de publication avec vérification obligatoire avant upload.

## Réutilisation
Recherche scripts/tests/.github : aucun outil de publication APK ni contrôle
apksigner existant. MobileUpdates.verifyApk protège déjà l'installation : réutilisé
sans modification. Créer scripts/publish-android.py pour la barrière de publication,
avec outils SDK existants, sans dépendance Python nouvelle.
