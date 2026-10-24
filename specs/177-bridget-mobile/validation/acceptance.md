# Recette 177 — état partiel

Statut : In Progress. Cette page ne clôt pas T036.

## Faits réels du 24 octobre 2026

- Appareil identifié par Android : Pixel 10a, Android 17. USB autorisé.
- APK de développement installé puis mis à jour en place ; aucune désinstallation.
- Sept tests instrumentés PASS, zéro échec et zéro saut, sur bases et alias Keystore de fixture isolés.
- Deux tests JVM de politique HTTPS PASS ; build Android PASS.
- Accueil français et lisibilité des barres système vérifiés sur capture réelle.
- Diaporama de la seule correction visible vérifié dans le moteur HTML : preuves-corrections/2026-10-24-177-android-system-bars/preuves.html.
- Les originaux et empreintes restent privés ; aucune capture d’une application extérieure dans le livrable.

## Non acquis

Association à la fédération réelle, parcours complet du scanner, Wi-Fi/4G, retour après fermeture du processus, notifications, fichiers, tâches, APK de livraison signé et conservation après mise à jour finale. La bascule du maître et le test Mac éteint restent ouverts. L’application installée n’est pas encore une version utilisable au quotidien.

La preuve TCP native coupe réellement le client après commit et avant réponse ; lecture de reçu puis rejeu strict donnent une remise. Ce test isolé ne remplace pas la recette réseau du téléphone.

## Complément réel — version signée et association

L’APK release signé 0.5.0 est installé par mise à jour, sans effacement. Quatorze tests de stockage instrumentés sont passés sur le Pixel. L’instrumentation Android indépendante des classes R8 relit les données AES-GCM avec la même clé Keystore après rotation du certificat : preserved_after_certificate_rotation. Le scanner réel s’ouvre et utilise la permission caméra accordée. L’accueil release est vérifié sur capture.

Après seconde bascule r6, le QR privé a été produit par l’administration native locale du maître dans un terminal SSH interactif et affiché uniquement dans Chrome local. Le Pixel a consommé cette association et le serveur enregistre un appareil. L’annuaire connecté puis une conversation d’agent Mac ont été effectivement affichés sur le Pixel via le serveur permanent HTTPS. Les captures et textes contenant de vraies conversations restent privés et ne seront pas inclus dans les preuves publiques. Aucun envoi métier de recette n’a été effectué.

L’application du téléphone a ensuite changé au premier plan. Les actions de test sont suspendues pour ne pas agir dans une autre application. Les notifications, la recette réseau, les pièces jointes et la preuve Mac hors ligne restent ouvertes.
