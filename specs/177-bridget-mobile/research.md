# Recherche 177

Date : 2026-10-24

## Constats locaux

- Base Git Mac et Cartae : be00ace5c2dca3126e420214ec248c54083440d1, main propre.
- docs/federation-services.md décrit un maître unique, un tunnel SSH et aucune réplication.
- daemon.rs:18115 : la garde de lecture humaine exige une liaison à une conversation de bureau et une présence locale.
- referent_control.rs:24–38 : un principal administratif humain local possède une autorité différente d'un agent. Le modèle de menace admet les processus du même compte local.
- daemon.rs:13341 : annulation humaine existante avec contrôle d'acteur, identifiant de demande et human_cancel.
- communication/client.rs:15 : client Unix borné ; ne pas en créer un second.
- environment.rs:18 : validation du namespace et des fichiers privés.
- t3code_mcp.rs:39 : la réattestation consulte un runtime local et un endpoint validé. Un déplacement de maître ne peut pas être déclaré transparent sans adaptation ou preuve.
- Cartae utilise Caddy sur 80/443. Le domaine racine cartae.app a une entrée A vers ce serveur ; bridget n'a pas encore d'entrée dédiée.
- Aucun JDK, Gradle ou adb trouvé dans le PATH du serveur. L'outillage Android devra être installé dans un emplacement identifié, puis réutilisé.
- setup-plan.sh n'existe pas à l'emplacement fourni par la primitive. Le protocole de planification est exécuté directement ; ce constat ne dispense d'aucun artefact.

## Décisions

### Accès public séparé
Décision : bridget.cartae.app, publié par Caddy vers une écoute locale dédiée.
Raison : le site et les autres applications conservent leurs routes ; les règles HTTPS existantes sont réutilisées.
Alternative : chemin sous cartae.app, possible mais mélange les politiques d'accès et les journaux.
Charge future : une route lisible et indépendante.

### Autorité du téléphone
Décision : association humaine explicite et révocable ; opérations et portée de projets/machines contrôlées sur chaque accès.
Raison : les identités d'agent et les conversations de bureau ne doivent pas servir de raccourci d'autorité.
Alternative rejetée : inscrire un agent humain fictif ou copier une session fournisseur.
Charge future : une frontière de droits vérifiable, sans dette cachée dans un nom.

### Moteur et idempotence
Décision : réutiliser les reçus, annulations et séquences natifs.
Raison : une nouvelle file de lancement serait un second moteur et risquerait les doublons.
Alternative : nouvel orchestrateur mobile, rejetée.
Charge future : une seule source des faits métier.

### Maître permanent
Décision : préparer et vérifier une migration explicite, conditionnée par la portabilité des attestations et des chemins.
Raison : un relais sur un serveur ne rend pas une autorité Mac indépendante du Mac.
Alternative rejetée : annoncer l'indépendance avec un simple proxy.
Point à prouver dans l'implémentation : aucune attestation loopback, tâche ou liaison active n'est rendue invalide par la bascule. Si la primitive existante ne couvre pas le parcours, écrire le contrat d'adaptation précis et ses tests avant modification. La migration reste ouverte tant que ce test échoue.

### Android et notifications
Décision : Kotlin/Compose pour Android ; stockage protégé par la plateforme. FCM proposé pour les notifications sans corps de conversation.
Raison : une plateforme demandée, cycle de vie natif et une seule application à installer.
Alternative : application web seule, ne couvre pas le packaging demandé ni les notifications natives de manière équivalente.
Préconditions externes : téléphone disponible et configuration du service de notification. Aucun secret ni projet externe n'est inventé.

## Sources primaires

- https://developer.android.com/compose — interface native Android.
- https://developer.android.com/build/releases/gradle-plugin — compatibilité de build à verrouiller.
- https://developer.android.com/studio/publish/app-signing — signature et mise à jour.
- https://firebase.google.com/docs/cloud-messaging/android/get-started — prérequis des notifications.
- https://caddyserver.com/docs/caddyfile/directives/reverse_proxy — publication par proxy.
- https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html — HTTPS, contrôle d'accès et codes d'erreur.
- https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html — secrets de session, expiration et révocation.

Les baselines sécurité, confidentialité, frontend et infrastructure de la mémoire globale ont été consultées. Les mécanismes comparés hors dépôt sont seulement des références de raisonnement ; aucune copie de code ou d'assets n'est prévue. Le relevé privé des lectures n'est pas publié.

## Inconnues de livraison et non garanties

La configuration de notification, le téléphone de recette, la signature finale et les préconditions d'une bascule réelle doivent être constatés. L'existence de l'accès DNS en lecture ne prouve pas une mutation DNS réussie. Aucun runtime n'a été remplacé à ce stade.

## Choix d’outillage verrouillés avant ajout

- Serveur HTTP : Axum 0.8.9 et Tokio 1.54.1, avec les seules fonctionnalités réseau, runtime et temps nécessaires ; Tower pour les limites si nécessaire. Aucun équivalent serveur HTTP dans les dépendances existantes. minreq et tungstenite ne couvrent pas le routage HTTP borné. Cette dépendance évite un parseur HTTP interne et porte la sécurité/compatibilité de transport. Le moteur Rust existant reste synchrone ; l’adaptation HTTP est isolée de sa boucle.
- Android : AGP 8.9.2, Gradle 8.11.1, JDK 17 et SDK 35. Matrice de compatibilité officielle consultée ; ces versions sont choisies pour un socle connu plutôt que pour reproduire la totalité d’un autre produit. Kotlin/Compose seront verrouillés dans le build et vérifiés par compilation.
- Le JDK 17 a été installé depuis les paquets signés Ubuntu existants. Aucune mise à niveau globale demandée, aucun conteneur ou service Bridget relancé par cette commande. Le gestionnaire de paquets a effectué son entretien système ; Caddy et le worker restent observés actifs.
- SDK et outils Android à installer sous /home/moi/.local/share/bridget/android-sdk, distribution Gradle réutilisée sous /home/moi/.cache/bridget-mobile-tools. Les intermédiaires du module restent identifiables et hors Git.

## Administration et QR

L'administration d'appareil doit atteindre le maître par la socket existante, y compris depuis un client fédéré. Écrire un magasin local sur le Mac client serait un doublon d'autorité. Le contrat ajoutera une capacité administrative mobile dédiée, après négociation et contrôle du pair Unix ; aucune façade MCP ne l'exposera. Le client humain conserve son contrôle de terminal interactif.

Une bibliothèque de QR est nécessaire pour le scan demandé. Recherche QrCode/qrcode dans crates, scripts et Cargo.lock : aucun équivalent. Choix qrcode 0.14.1, rendu Unicode/SVG sans dépendance image. Cette dépendance évite un encodeur QR interne et un outil à installer en plus par l'utilisateur.

## Décision utilisateur — 24 octobre 2026

Première livraison : piloter les agents existants et leurs tâches. La création directe de nouveaux agents est reportée comme extension. L'envoi à un agent existant reste possible ; cet agent applique ses droits de délégation. Le téléphone ne reçoit aucun droit de lancement direct par anticipation.
