# Journal 177

Démarrage : 2026-10-24 09:52 CEST
Statut : In Progress

## Cadrage et préflight

- my-specify-all chargé depuis le chemin explicite de l'utilisateur.
- ETA initiale 9 425–15 850 minutes de travail ; horizon 5–8 semaines à recalibrer après Tasks.
- Synchronisation SpecKit exécutée sur Mac puis dans le worktree serveur.
- Nouvelle session 177, base be00ace5c2dca3126e420214ec248c54083440d1.
- Worktree dédié : /home/moi/10.Scripts/64.bridget/.worktrees/177-bridget-mobile.
- Code et artefacts écrits sur le serveur ; main Mac et Cartae conservés.
- DNS cartae.app consulté en lecture via l'accès existant ; Caddy constaté. Aucune mutation DNS, aucun remplacement de service.
- JDK, Gradle, adb absents des emplacements usuels du serveur.
- setup-plan.sh demandé par la primitive absent ; protocole de la skill exécuté manuellement.

## Ressources et preuves

Les secrets, données de compte et fichiers de configuration de production restent hors Git.
Les preuves de recherche ou de déploiement privées seront conservées sous un répertoire privé nommé pour 177.

## Plan, Tasks et Analyze

- Audit de réutilisation PASS, 14 items audités et zéro duplication évidente.
- 41 tâches couvrant 27 exigences et 8 scénarios.
- Analyze exécuté directement selon le protocole disponible : cinq écarts corrigés, deuxième lecture sans CRITICAL.
- ETA recalibrée après Tasks : 9 425–15 850 minutes de travail, horizon 5–8 semaines ; principales inconnues : migration, notifications et téléphone.
- Contre-revue non admise ; aucune revue indépendante acquise.

## T001 et T002

Inventaire en lecture seule et contrats d’autorité définis. La base vivante contient des tâches actives ; aucune bascule exécutée. Les faits, le contrat d’origine et l’ADR sont relus. Aucun nouveau moteur, aucun runtime remplacé. Le contrat n’est pas une preuve d’exécution. T001/T002 cochées pour ces seuls livrables.

## T003 et T004

JDK17, SDK35/build-tools35 et Gradle8.11.1 installés ; archives vérifiées, outils invoqués. Les preuves privées sont dans /home/moi/.cache/bridget-mobile-tools. Huit tests d’association sont écrits : RED exécuté par le wrapper Cargo, 1 PASS / 7 FAIL attendus sur les opérations non encore implémentées. Les frontières testées portent sur usage unique, réponse perdue, révocation, expiration, portée et renouvellement. Aucun secret brut dans les preuves.

## T005 et contrôle de T006

- Magasin d’autorisation implémenté et relu : 11 tests PASS, dont fichier privé après réouverture, limite d’appareils et deux consommateurs concurrents d’un même QR.
- Aucun secret brut persisté ; aucun état métier créé dans ce magasin.
- T006 ajoute une capacité administrative dédiée et une réponse dont Debug masque le secret du QR.
- Première compilation : deux matrices fermées demandaient le nouveau refus explicite ; corrigé sans élargir les rôles.
- Premier test socket positif : refus de configuration. Observation runtime avant correction : la fixture créée sans l’initialisation CLI avait des fichiers SQLite 0640 et une socket 0770. Le lancement réel appelle environment::initialize_process et fixe umask 077. La fixture est alignée par permissions explicites de ses seuls fichiers, sans changement de l’umask global ni affaiblissement des gardes.

### Reprise et décision de périmètre

T006 vérifiée : 16 tests mobile177 PASS, aucun échec ni saut, dont administration sur vraie socket Unix et refus d'une connexion d'agent. Aucun binaire actif remplacé. Décision utilisateur : agents existants d'abord ; création directe reportée. Le profil --all n'accorde pas task_create et une portée demandant cette opération est refusée.

### Frontière HTTP initiale

22 tests mobile177 PASS, 0 échec, 0 ignoré. Six scénarios supplémentaires passent sur de vraies connexions TCP : association et rejeu, révocation, rotation et réponse perdue, origine étrangère et en-têtes dupliqués, JSON strict et borne de corps, droits de gestion et refus de création d'agent, saturation avant consommation et maintien de la borne TCP. Route publique non activée. L'annuaire reprend directory_request et projette les seuls champs utiles après portée, avant pagination. Le contrôle d'authentification est relu après l'appel natif.

### Appareil de recette

Utilisateur : Pixel 10, raccordement USB au Mac disponible pour la recette finale. Aucun test physique encore réalisé.

### Premier APK

Build Android réussi : assembleDebug et testDebugUnitTest, 41 actions, 1 min 11 s ; deux tests unitaires PASS. APK de développement seulement, aucune livraison annoncée. Versions de dépendances figées dans gradle.lockfile. Pixel USB vu depuis le Mac mais non autorisé ; demande de validation Android envoyée à l'utilisateur. Le stockage Android utilise AES-GCM de la plateforme avec clé Keystore ; tests physiques isolés préparés, jamais exécutés sur le fichier de données utilisateur.

### Diagnostic de la fixture Unix et preuve Android

Refus authorization_required dans le test natif : la fixture hérite de conn-1 pour son destinataire mais laisse conn_counter à zéro. Le premier client Unix réel réutilise conn-1 et porte donc une identité d’agent ; le refus est correct. Le compteur de la fixture est avancé, comme les autres tests natifs existants. Aucun contrôle de production assoupli. Sur Pixel 10a/Android17 réel : trois tests instrumentés PASS (réouverture, substitution de ligne chiffrée, perte de clé). Le premier relevé UI n’était pas exploitable car téléphone verrouillé ; aucune preuve visuelle validée n’en est déduite.

### Premières observations visuelles

Accueil réel français affiché sur Pixel, application de test non associée. Défaut constaté : icônes de barre système noires sur fond sombre, car le thème XML forçait le mode clair. Référence relue : le thème natif suit le système ; pas de code Android équivalent dans cette référence de bureau. Réutilisation de l’API standard enableEdgeToEdge d’AndroidX Activity pour accorder les barres au thème ; vérification réelle après rebuild prévue.

### Checkpoint vérifié

28 tests Rust mobile177 PASS, 0 échec, 0 ignoré. Garde finale de projet testée avant tout reçu ; révocation et refus d’identité d’agent ; trois connexions Unix réelles (première réponse jetée, lecture du reçu, rejeu strict) donnent une seule remise durable. Cette preuve simule la réponse perdue en la jetant côté application ; elle ne prouve pas encore une coupure TCP après commit. T008 reste ouverte : annuaire/projets filtrés et paginés réalisés, inventaire des machines hors ligne encore à raccorder. T019 close sur les trois tests instrumentés réels ; T025 reste ouverte pour arrêt du processus et réseau. Aucun commit, push, bascule de maître ni HTTPS public réalisés à ce stade.

### Validation de QR côté téléphone

Défaut confirmé par lecture : une propriété manquante pouvait produire une exception JSON affichée à tort comme un défaut de stockage. Validation stricte partagée entre aperçu et association ; les entrées mal formées donnent invalid_qr, sans contacter le réseau ni modifier l'autorisation. La référence de bureau applique un décodage typé ; aucun équivalent Android dans cette référence. Test instrumenté des types, champs inconnus et origines préparé.

### Origine, reconnexion et coupure réelle

32 tests Rust mobile177 PASS, zéro échec ou saut. T008 close : annuaire/projets autorisés avant pagination et machines natives conservées en stale sans fuite des machines non autorisées. T009 reste ouverte pour la matrice intégrée complète. T013 close sur vraie chaîne HTTP TCP → relais Unix de test → daemon natif : la réponse native est retenue après commit, le client TCP ferme sans lire un octet, puis lecture du reçu et rejeu strict donnent une seule remise durable. Aucun faux reçu injecté. Le premier ajout du test de refus d’origine utilisait un nom SQL de table erroné ; correction de la fixture vers idempotency_records, sans modification de production.

La commande lie désormais device_id dans son enveloppe ; le daemon le compare à la clé vérifiée. L’application garde aussi endpoint et refuse avant réseau une nouvelle origine. Brouillons et renouvellements ont la même séparation. Référence courte relue : cible HTTP explicitement résolue et brouillon lié à environnement/fil dans le code de bureau ; pas de file durable Android équivalente trouvée. Réutilisation du magasin, du chiffrement et des reçus existants.

Accueil français et contraste système vérifiés sur Pixel réel. Preuves visuelles autonomes : preuves-corrections/2026-10-24-177-android-system-bars/preuves.html ; captures originales privées. Parcours scanner non vérifié : le téléphone a changé d’application pendant la tentative. Une action de test a alors touché un écran extérieur ; elle ne fournit aucune preuve Bridget. Les prochaines interactions doivent tester le paquet au premier plan et refuser l’action dans le même bloc de contrôle avant tout appui. Aucun autre appui ni lancement de parcours téléphone après ce constat.

### Checkpoint Android sur appareil réel

Build assembleDebug, assembleDebugAndroidTest et testDebugUnitTest PASS ; deux tests JVM. Les deux APK ont été comparés par SHA-256 entre Cartae et le Mac, puis mis à jour sans désinstallation et sans effacement des données. Sept tests instrumentés réels PASS, zéro échec ou saut : réouverture, substitution de ligne chiffrée, clé perdue, commande liée à son origine, brouillon séparé par origine/appareil/agent, renouvellement lié à son origine, QR strict. Les fixtures utilisent leurs seuls noms de base et alias Keystore temporaires. Aucun flux de production associé, aucun message métier, aucun DNS, aucune bascule, aucun binaire actif Bridget remplacé. T020/T021/T025/T036 restent ouvertes pour leurs parcours réseau et UI complets.

### T007 et T009 — suivi et annuaire intégrés

37 tests Rust mobile177 PASS, zéro échec ou saut. Suivi : flux attach natif réutilisé, repère lié à appareil/agent, 25 s maximum par cycle, quatre cycles par appareil, six au total pour réserver deux travailleurs aux lectures et mutations ordinaires. Revocation active ferme le flux natif avant émission ; test TCP, événement réel de protocole, échéance et requête forgée. Pas d’interrogation périodique du journal worker. Appareils paginés après autorisation. Les avertissements de fixtures portaient sur des champs de construction ou emprunts Rust, corrigés sans changement de garde.

T009 : vrai HTTP TCP vers le client Unix natif et le daemon, voisins de projet/machine présents avant pagination. Le premier agent autorisé reste visible avec limite un ; voisins absents de liste, projet et filtre. Une machine stale reste testée séparément. Aucun envoi produit par ces lectures. Les fichiers de test restent au plus près du gestionnaire au lieu du fichier d’intégration initialement prévu. Self-review : un seul lecteur natif, aucun second annuaire, pas de nouvelle autorité. Non acquis : parcours téléphone connecté et recette distante complète.

## Reprise : adaptations métier et client Android

T011, T012 et T015–T017 vérifiées sur les puits natifs : 44 tests mobile177, 12 tests artefacts. Les fixtures réutilisent les modules privés existants pour éviter un second harnais. Les écrans Android, la file persistante, les pièces jointes protégées et les reçus sont implémentés ; leur qualification connectée reste ouverte. La reprise après arrêt réel du processus a été prouvée sur le Pixel. Voir validation/mobile-native.md. Aucun binaire de production activé et aucune migration effectuée.

## Canal de notification implémenté, recette externe ouverte

Le magasin d’appareils passe en version 2 avec une souscription technique facultative par appareil. Les droits fédéraux de lecture sont relus avant chaque envoi, après révocation/expiration. Le daemon utilise une file éphémère de taille 1, coalescée à 15 secondes, distincte de la boucle métier. La requête FCM ne contient que la version du format et une empreinte de l’association ; aucun texte métier ni identifiant d’agent/projet. OAuth et envoi HTTPS ont des budgets et captures bornés, avec refus des redirections. Le compte de service reste privé et s’administre par la capacité humaine locale existante.

Android possède un service FCM, un choix explicite, la permission Android 13+, un canal système et le rejet des alertes d’une autre association ou contenant des champs métier. Aucun Analytics ajouté et auto-initialisation désactivée. La préférence locale ne devient pas une inscription confirmée sans reçu serveur. Les identifiants publics Firebase sont fournis au build par configuration privée ; aucun compte inventé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/177-bridget-mobile/implementation.md#extrait-1).

## Régression d’origine : fixture locale incohérente identifiée

Le filtre worker a révélé trois échecs après le routage des preuves par hôte : 57 passés et 3 échecs. Observation directe : la fixture native exécutait ses fichiers temporaires sur le coordinateur, mais sa présence primaire annonçait macbook tandis que le coordinateur portait le nom réel Cartae. Une recette changeait encore le coordinateur en coordinator171 sans changer la présence. Le nouveau contrôle reconnaissait correctement ces parents comme distants, puis refusait l’absence de worker d’origine. La fixture locale a été alignée sur son hôte déclaré ; le scénario coordinator171 aligne explicitement sa présence. Aucun contournement de preuve ni repli local ajouté au code de production. Le nouveau test de relais couvre séparément l’origine distante.

Lecture runtime complémentaire de la recette de mapping : host=coordinator171, origin=macbook, présence instance-1=macbook, mapping source_host=coordinator171. La première retouche du même libellé avait aligné une autre recette ; deux autres affectations explicites restaient. Elles sont maintenant alignées dans leurs blocs de fixture. Le diagnostic temporaire a été retiré après observation.
