# Plan 177 — Bridget Mobile et pilotage fédéré

Date : 2026-10-24
Statut : In Progress
Base : be00ace5c2dca3126e420214ec248c54083440d1
Branche : session-177-bridget-mobile

## Résumé

Une application Android native utilise un accès HTTPS Bridget sur un serveur permanent. Le moteur existant reste l'unique autorité des agents, messages, reçus, permissions et tâches. Le téléphone possède une identité humaine révocable et une portée explicite ; il n'emprunte aucune identité d'agent.

## Contexte technique observé

- Dépôt Rust 2024, version 0.4.1 ; pas d'application mobile ni de serveur HTTP dans le périmètre actuel.
- Bibliothèques existantes : serde, rusqlite, uuid, sha2, minreq et tungstenite.
- Le daemon utilise une socket Unix, un verrou exclusif et SQLite ; une fédération possède un maître unique.
- Les vues humaines de conversation et de lignée sont rattachées à une conversation de bureau. Le contrôle administratif humain local est déjà séparé des agents.
- L'envoi idempotent et le journal doivent être réutilisés ; aucune nouvelle file métier concurrente.
- Cartae dispose de Caddy et d'une route HTTPS existante pour cartae.app. Aucun outil Android ni JDK n'a été trouvé dans le PATH ou les emplacements usuels du serveur.
- Les attestations de session fournisseur peuvent dépendre d'un endpoint loopback sur le poste d'origine. Leur portabilité doit être prouvée avant une migration du maître.

## Architecture retenue

1. Application Android Kotlin/Compose, construite et signée depuis le serveur.
2. Accès mobile intégré au binaire Bridget : une façade HTTP locale, publiée uniquement par le reverse proxy HTTPS existant.
3. Un contrat mobile versionné et fermé ; aucune route de shell ni transport libre de trames internes.
4. Identité humaine d'appareil, créée par une association locale de confiance. Les autorisations portent sur les opérations, projets et machines. Le téléphone ne choisit jamais un acteur, un propriétaire ou un digest de permission.
5. Réemploi des puits natifs d'envoi, journal, catalogue, tâches et contrôle humain. Les actions de tâche proposées dépendent des capacités et autorisations réellement disponibles. Une opération absente reste bloquée jusqu'à son adaptation explicite.
6. Clés de commandes persistées avant envoi, réconciliation sur les reçus natifs ; une issue inconnue ne devient pas un nouvel envoi.
7. Le maître est déplacé vers un serveur permanent seulement après validation de la portabilité des chemins, liaisons, attestations et tâches. Les nouveaux accès mobiles peuvent être développés et vérifiés dans un namespace isolé avant cette bascule.
8. Notifications minimales via FCM, sans corps de message. L'application reste utilisable en mode interactif sans configuration de notification, mais la livraison complète garde la recette de notifications ouverte.

## Décisions structurantes

- Sous-domaine bridget.cartae.app : séparation claire de la vitrine et des autres produits. Aucun changement du routage racine.
- Android natif : une seule plateforme demandée, accès direct au stockage protégé, aux notifications et au cycle de vie.
- Façade mobile intégrée au produit : aucun assemblage manuel de programmes sur le téléphone.
- Un maître, aucune réplication de base ajoutée.
- La migration et la portabilité des attestations sont des prérequis vérifiés, pas une hypothèse cachée.
- Aucune copie de code ou d'éléments de présentation d'un autre produit.

## Réutilisation de l'existant

| Responsabilité | Source existante | Usage prévu |
|---|---|---|
| Namespace privé et chemins | crates/bridget-daemon/src/environment.rs | Réutiliser permissions, validation de fichiers et refus des liens |
| Connexion Unix bornée | crates/bridget-daemon/src/communication/client.rs | Réutiliser budgets et négociation |
| Annuaire et identités | crates/bridget-daemon/src/daemon.rs, bridget-core | Projeter l'annuaire puis appliquer la portée de l'appareil |
| Projets | crates/bridget-daemon/src/communication.rs | Réutiliser l'identité canonique de projet et les mappings |
| Envoi et reçu | crates/bridget-daemon/src/idempotency.rs, daemon.rs | Réutiliser le puits d'envoi ; ne pas construire une seconde remise |
| Journal humain | crates/bridget-daemon/src/attach.rs | Réutiliser pagination et séquences |
| Consultation de tâche | crates/bridget-daemon/src/daemon/native_delegation.rs | Étendre la projection, préserver les invariants natifs |
| Annulation humaine | crates/bridget-daemon/src/daemon.rs et delegation.rs | Réutiliser human_cancel et son reçu |
| Questions humaines | crates/bridget-daemon/src/human_inbox.rs | Réutiliser options, décision et acquittement |
| Migration et raccordement | crates/bridget-daemon/src/federate.rs, scripts/federate-ssh.sh | Étendre le préflight et documenter la bascule |
| HTTPS | Caddy existant de Cartae | Ajouter une route sans remplacer les autres routes |
| Entretien Rust | scripts/build.py et Makefile | Obligatoire avant et après chaque Cargo |

## Nouveautés nécessaires et gate anti-doublon

- Contrat mobile : aucun équivalent de principal appareil fédéré n'existe ; les contrats de lecture liés à une conversation ne confèrent pas cette autorité.
- Persistance des associations, sessions et révocations : nouvelle responsabilité, séparée des identités d'agent. Réutiliser SQLite et le namespace privé.
- Façade HTTP : aucun serveur existant dans Bridget ; choix de bibliothèque à verrouiller après vérification du contrôle des ressources. Ne pas réimplémenter un parseur HTTP.
- Application Android : nouveau dossier android/, dépendances limitées à la plateforme, Compose, scan QR et notification.
- Packaging mobile et migration : réemploi des scripts d'installation et de fédération ; pas de second superviseur générique.
- ADR : enregistrer ces choix dans docs/decisions/ avant l'implémentation correspondante.

## Plan de fichiers

- crates/bridget-daemon/src/mobile.rs : entrée CLI de l'accès mobile.
- crates/bridget-daemon/src/mobile/store.rs : associations, appareils, renouvellement et révocation privés.
- crates/bridget-daemon/src/mobile/http.rs : frontière HTTP authentifiée et bornée.
- crates/bridget-daemon/src/mobile/client.rs : adaptation vers les opérations natives.
- crates/bridget-daemon/src/mobile/notifications.rs : signal minimal vers le canal configuré.
- android/ : application, ressources FR/EN, tests et build.
- Les modules daemon, transport et fédération existants sont étendus pour les seuls contrats qui manquent.

## Données et contrats

Voir data-model.md et contracts/mobile-v1.md.
Les secrets ne sont jamais placés dans une URL publique persistante, un journal ou un dépôt. Le QR peut porter un secret d'association temporaire dans un fragment local ; l'échange passe par HTTPS et n'est jamais journalisé.
Les listes sont bornées et paginées. Le serveur applique les droits avant pagination et avant émission d'événements.
Une préférence d'affichage ne porte aucune autorité.

## Migration et installation

1. Inventaire en lecture seule du maître, des clients, workers, services, données et tâches actives.
2. Sauvegarde cohérente des données avec manifestes privés ; copie de fichier SQLite ouvert interdite.
3. Validation dans une fédération isolée du nouvel emplacement et des attestations restées sur leur poste d'origine.
4. Préparation des services et raccordements, sans activation concurrente.
5. Bascule seulement si les préconditions sont prouvées ; sinon conserver le maître courant et la tâche de migration ouverte.
6. Réconciliation des tâches et des identités après bascule ; aucune relance métier automatique.
7. Installation de la route HTTPS, vérification de l'APK et recette physique Wi-Fi/4G/Mac éteint.
8. Retour arrière vérifié sans restaurer une base sur une autorité active.

## Contrôle constitutionnel

- Français pour les artefacts SpecKit ; documentation publique EN par défaut et FR conservée.
- Développement dans ce worktree serveur dédié.
- Pas de nouveau moteur métier, d'alias de droits ou d'agent synthétique.
- Une dépendance n'est ajoutée qu'après recherche et justification dans reuse-audit.md.
- Complexité bornée, pagination, coût réseau explicite, aucune observation de tout le store à chaque requête mobile.
- Tests ciblés des frontières de droits et de l'idempotence, puis recette réelle.
- Identité Git existante conservée ; commit après les gates de livraison de my-specify-all.
- Les règles génériques TypeScript ne s'appliquent pas à un projet Rust/Kotlin ; checks natifs équivalents.
- Les builds Rust passent par scripts/build.py ; aucun target copié par test.
- Pas d'exécution privilégiée ni de modification réseau tant que le changement concret n'est pas préparé et vérifié.

## Séquence d'implémentation

Fondations et contrats → association et droits → lecture et annuaire → messages et reçus → tâches et questions → application Android → notifications et mise à jour → migration et routage → recette et livraison.

Les tâches indépendantes des accès DNS, notification et téléphone progressent sans attendre ces accès. Le périmètre complet reste visible dans tasks.md.

## Risques à vérifier

| Risque | Observation qui tranche | Effet |
|---|---|---|
| Autorité humaine déduite d'un libellé local | Tests d'un appareil et d'un agent contre la même opération | Refuser l'agent et recontrôler le droit appareil |
| Déplacement d'une preuve loopback | Test d'attestation depuis le serveur permanent | Bloque migration si non portable |
| Réponse perdue | Couper le transport après commit serveur | Une seule commande et remise |
| Duplication d'état métier | Recherche des puits d'écriture et des reçus | Réutiliser le puits existant |
| Attachement sous une autre racine | Test hors portée et taille dépassée | Refus sans lecture du fichier |
| Notification en arrière-plan | Recette sur Android physique | Aucune réussite déduite de l'envoi FCM seul |
| Migration pendant une tâche | Inventaire, backup et reconnexion contrôlée | Conservation et réconciliation, aucun replay métier |

## Maintenance future

Un contrat mobile versionné, des adaptateurs courts vers les puits existants et une application Android. Aucun framework d'orchestration nouveau. Les limites externes et la première tâche ouverte restent visibles. Les détails de déploiement privés sont conservés hors Git.

## Décision utilisateur — 24 octobre 2026

Première livraison : piloter les agents existants et leurs tâches. La création directe de nouveaux agents est reportée comme extension. L'envoi à un agent existant reste possible ; cet agent applique ses droits de délégation. Le téléphone ne reçoit aucun droit de lancement direct par anticipation.

## Complément lecture riche — r10d

Réutiliser le téléchargement authentifié des pièces jointes, sa vérification de
SHA-256 et ses contrôles d’appareil. Rendu natif Markwon 4.6.2 (Markdown et tables),
Highlights 1.0.0 (code), sans service réseau. Les deux bibliothèques remplacent le
parseur partiel gras/code. Android WebView est réservé à l’aperçu HTML isolé.
Limiter l’aperçu à 256 Kio, la coloration à 32 K caractères ; conserver le texte
complet sans couleur au-delà. Calculer le rendu hors du fil UI. Ajouter HTML et
Markdown à la liste fermée des textes UTF-8 transportés. Aucun nouveau droit.
Vérifier compilation, tests ciblés, refus d’accès WebView et circuit Pixel.
