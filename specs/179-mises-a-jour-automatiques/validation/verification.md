# Validation179 — progression

Base commune d6a2984f : sécurité178 et mobile177 conservés.

Contrôles effectués avant la publication :
- tests Python du gestionnaire : 13 PASS ; fichiers, archives et liens réels, services/RPC simulés pour les pannes ;
- Rust updates179 : 7 PASS, dont socket Unix réelle et garde humaine/révocation ;
- Rust build_info : 9 PASS ;
- Rust mobile177 : 69 PASS, 1 ignored préexistant ;
- Android : testDebugUnitTest, lintRelease, assembleRelease, assembleDebug et assembleDebugAndroidTest réussis avec le JDK corrigé178 ;
- Pixel Android17 : 3 tests instrumentés PASS dans un paquet isolé, sans toucher l'association principale ;
- premier APK de préparation signé avec la clé installée, installation sans effacement réussie ;
- parcours Réglages : défaut ON, OFF conservé après fermeture/réouverture, remis ON. Diaporama partiel dans preuves-corrections.

La recette a révélé un not_found du backend178 encore actif sur la route nouvelle. Guidage corrigé en update_hosts_unavailable : choix Android conservé, choix des hôtes en attente. Dernier lot Kotlin et version0.6.0 : construction Android, lint et tests JVM réussis ; sept tests Rust updates179 réussis à nouveau.

Délégation interdite dans ce tour : pas de contre-revue indépendante revendiquée. Relecture directe : origines de téléchargement, signature, bornes, permissions, atomicité, course d'activation, panne après libération, reprise et nettoyage.

Limites à lever ou rapporter à la livraison :
- activation initiale de la nouvelle API ;
- tâche historique e5959986 en cancelling avec arrêt non confirmé, conservée ;
- publication commune et relecture des actifs ;
- recette de l'installation Android automatique et réglage hôtes après activation ;
- clients MCP déjà chargés : fichier stable différent ne signifie pas code rechargé.
