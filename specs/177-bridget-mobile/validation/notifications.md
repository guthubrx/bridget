# Notifications 177 — configuration et preuves

Statut : In Progress, aucune notification physique reçue encore prouvée.

Le projet Firebase Bridget Mobile (bridget-mobile) a été créé par l’utilisateur. Le forfait Spark est affiché ; aucune facturation n’a été activée. L’API Firebase Cloud Messaging V1 est confirmée activée dans les paramètres du projet. L’ancienne API reste désactivée. L’application Android app.cartae.bridget est enregistrée ; le certificat public de signature est ajouté.

Un compte de service dédié ne possède que le rôle roles/firebasecloudmessaging.admin. Sa clé JSON a été téléchargée directement dans un dossier privé, protégée en 0600 et transférée par SSH dans un dossier privé du serveur. Aucun secret n’est dans Git, le téléphone ou les journaux. Les options publiques Android sont séparées du compte serveur.

La compilation Android configurée réussit (25 s). Installation de mise à jour sans désinstallation ; tests physiques ProtectedStoreTest : 13 PASS. Le lot natif mobile177 après corrections : 54 PASS, 1 recette d’origine ignorée par défaut. Ces tests ne prouvent pas la réception en arrière-plan.

À terminer : configuration du maître permanent par administration humaine locale, consentement Android, enregistrement du jeton, notification minimale réelle en arrière-plan, ouverture authentifiée et refus de permission sans blocage interactif. Aucun contenu métier n’est envoyé à Firebase.

Sources primaires : https://firebase.google.com/docs/projects/iam/roles-predefined-product et https://docs.cloud.google.com/iam/docs/roles-permissions/firebasecloudmessaging .

## Contrôle r10b — 24 octobre

Le compte serveur permanent est configuré et protégé en 0600. Un appareil actif
possède un jeton enregistré. L’authentification OAuth et la validation seule FCM
ont renvoyé HTTP 200. Cette demande n’envoie pas de notification et ne prouve
pas la réception sur le Pixel. Le téléphone n’étant plus visible en USB, le
contrôle en arrière-plan et l’ouverture restent à terminer.
