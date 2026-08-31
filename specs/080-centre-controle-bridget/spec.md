# Spécification - SPEC-080 Centre de contrôle Bridget

## Fiche synthèse

Spec: 080-centre-controle-bridget
Titre: Centre de contrôle, réglages de serveur et usage
Statut: In Progress - reprise corrective, centre complet exigé avant clôture
Priorité: P1
Branche: session-080-centre-controle-bridget
Créée: 2026-08-31
Dépendances: SPEC-064, SPEC-065, SPEC-066, SPEC-067, SPEC-068, SPEC-074, SPEC-076, SPEC-077

## Rectification de périmètre - 2026-08-31

La première livraison de cette SPEC a rendu un bouton discret « Réglages du
serveur » et un sous-écran limité aux racines de projets et à l'usage. Cette
tranche ne satisfait pas les User Stories 1 à 6 et ne doit plus être décrite
comme un centre de contrôle terminé.

La clôture de SPEC-080 exige désormais une vérification visuelle sur le relais
réel des éléments suivants :

- une roue reconnaissable, fixe tout en bas de la barre latérale gauche ;
- une navigation globale recherchable : Général, Apparence, Date et heure,
  Typographie, Serveurs, Usage et facturation, Mises à jour et Diagnostics ;
- une page serveur distincte par profil enregistré, avec ses seules capacités
  attestées et ses réglages réellement modifiables ;
- une séparation visible entre les préférences de ce Mac, les réglages de ce
  serveur et les politiques de projet ;
- aucune annonce de livraison tant que ce parcours n'est pas observé dans
  l'interface réellement servie.

## Rectification de finition - 2026-08-31

Le centre doit reprendre la grammaire visuelle de réglages de T3 Code pour les lignes, la typographie, les libellés, sous-titres, contrôles et aperçus. Bridget conserve ses fonds, ses gris, son accent et son overlay. Une approximation utilisant des contrôles système bruts ou des cartes génériques ne satisfait pas FR-8001 à FR-8004.

La SPEC reste ouverte tant que la version de finition servie n'a pas été examinée par l'opérateur et tant que le paquet macOS construit n'a pas été installé par choix explicite, après fermeture de l'application en cours. Les éléments non réalisés, notamment tarification datée et réglages serveur supplémentaires, restent explicitement non livrés.

L'icône de l'application est une partie du produit : le paquet macOS DOIT embarquer l'asset Bridget validé, sur fond noir légèrement dégradé, avec la mascotte agrandie. Une icône générique ou une variante plus ancienne ne satisfait pas cette SPEC.

## Contexte

Bridget pilote plusieurs serveurs enregistrés. L'opérateur doit aujourd'hui raisonner à partir de l'interface principale, de l'état des agents et de la configuration du serveur, sans point unique pour distinguer ce qui relève du poste Mac, du serveur, du projet ou de l'exécution.

Le besoin premier est de pouvoir ouvrir, depuis une roue d'engrenage durablement placée en bas de la barre latérale, un centre de contrôle. Celui-ci doit permettre de consulter chaque serveur enregistré et de modifier uniquement les réglages que ce serveur déclare modifiables. La même surface rend les préférences locales prévisibles, expose l'usage observé et signale l'existence d'une version plus récente, sans devenir un terminal distant ni une console d'administration arbitraire.

Un serveur est une autorité d'exécution distincte. Ses réglages doivent donc être décrits par ses propres capacités et appliqués avec les mêmes garanties que les autres commandes Bridget: prévisualisation, confirmation locale, mutation atomique, version attendue et reçu persistant. Un agent ne doit jamais pouvoir modifier ces réglages.

## Objectifs

- Donner un accès permanent et découvrable aux préférences et à l'administration contrôlée des serveurs.
- Permettre à l'opérateur de consulter et modifier, serveur par serveur, un catalogue fermé de réglages explicitement déclarés sûrs et modifiables.
- Distinguer sans ambiguïté les valeurs locales au Mac, les valeurs du serveur, les politiques de projet et les éléments intentionnellement non modifiables.
- Présenter les jetons et coûts observés par fournisseur, modèle, projet et serveur, sans jamais confondre estimation API et facture réelle.
- Préserver le modèle de sécurité local: aucune exécution shell arbitraire, aucun secret révélé, aucune approbation distante et aucune mutation optimiste mensongère.
- Donner un premier signal sur les mises à jour disponibles, sans installer ou mettre à jour un serveur automatiquement.

## Hors périmètre

- Éditeur libre de fichiers de configuration ou terminal SSH dans l'interface.
- Création, modification ou révélation de secrets, jetons, clés SSH ou profils d'agent.
- Approbation distante d'une action sensible ou contournement d'une confirmation locale.
- Installation, mise à jour ou redémarrage automatique d'un serveur, d'un fournisseur ou d'un agent.
- Facturation, rapprochement bancaire, limite contractuelle d'abonnement ou promesse d'exactitude comptable.
- Modification de la politique d'exécution propre à un projet depuis une valeur globale de serveur.
- Administration de l'hôte, du réseau, des paquets système, de Docker ou du pare-feu.

## Acteurs et responsabilités

- Opérateur: configure son poste, consulte l'état et demande les mutations autorisées sur un serveur enregistré.
- Bridget Desktop: conserve les préférences de présentation locales, affiche les portées et demande les confirmations locales.
- Serveur Bridget: publie ses capacités, les réglages autorisés, l'usage observé et les reçus. Il valide et applique atomiquement les mutations de son catalogue fermé.
- Projet: possède les politiques qui lui sont propres. Elles restent séparées des réglages de serveur.
- Agent: consomme une configuration autorisée par Bridget mais ne peut ni lire les secrets ni modifier les paramètres du centre de contrôle.

## Principes de conception

1. Une préférence porte toujours une étiquette de portée: Ce Mac, Ce serveur, Ce projet ou Lecture seule.
2. Un écran de serveur est une console de contrôle typée, pas une porte dérobée vers l'administration générale de l'hôte.
3. Toute écriture de serveur suit le parcours voir la valeur actuelle, prévisualiser le delta, confirmer localement, appliquer, vérifier le reçu.
4. Les valeurs d'usage manquantes restent inconnues. Elles ne deviennent jamais zéro par défaut.
5. Les coûts sont des estimations explicites fondées sur une grille datée. Les abonnements et valeurs sans prix restent non chiffrés.
6. La version et les diagnostics servent d'abord à informer. Toute opération de maintenance reste une action manuelle distincte.

## User Story 1 - Ouvrir et organiser le centre de contrôle - P1

Comme opérateur, je veux ouvrir les réglages depuis une roue en bas de la barre latérale afin de retrouver les préférences, les serveurs et l'usage sans changer de contexte mental.

### Scénarios d'acceptation

1. La roue d'engrenage est visible en bas de la barre latérale, quel que soit le contenu de la conversation active.
2. Son activation ouvre un centre de contrôle avec une navigation latérale stable et une recherche de réglage.
3. La navigation distingue Préférences du Mac, Serveurs, Usage, Mises à jour et Diagnostics.
4. Les sections non disponibles restent visibles avec une explication et ne donnent pas une impression de panne.
5. Le centre mémorise le dernier écran visité localement, sans masquer une erreur ou une permission devenue différente.
6. Le raccourci macOS `Commande + virgule` ouvre le même centre de contrôle sans changer la conversation affichée.

## User Story 1b - Organiser les projets sans confondre les portées - P1

Comme opérateur, je veux une colonne Projets consacrée aux projets, afin d'ajouter, filtrer, personnaliser ou retirer un projet sans la confondre avec les réglages globaux de Bridget ou du serveur.

### Scénarios d'acceptation

1. La colonne Projets explique qu'elle sert à ajouter, organiser et filtrer, et laisse une marge macOS suffisante sous les boutons de fenêtre.
2. « Toute la flotte » signifie explicitement « Tous projets confondus ».
3. Chaque projet est représenté par une petite tuile carrée, légèrement arrondie, portant une ou deux initiales et une couleur stable.
4. Un clic droit, le bouton d'actions ou le clavier ouvre le menu propre au projet. Ce menu permet au minimum de personnaliser son icône ou de le retirer de Bridget.
5. Les initiales et la couleur sont des préférences de présentation locales à cette interface. Elles ne modifient ni le dossier, ni les agents, ni une politique serveur.
6. La colonne peut être repliée : elle conserve alors les icônes des projets et les actions restent atteignables au clic droit ou au clavier.
7. Aucun bouton textuel « Réglages » ou « Retirer » global ne reste au pied de la colonne Projets.
8. `+` et `Importer` ouvrent un dialogue de prévisualisation. La création ou l'import n'est confirmé qu'après validation de la racine autorisée et inscription durable au registre de projets.

## User Story 2 - Régler les préférences du Mac - P1

Comme opérateur, je veux régler l'apparence et la lecture sur mon Mac afin que Bridget respecte mon environnement de travail sans modifier les autres postes ni les serveurs.

### Scénarios d'acceptation

1. Je peux choisir le thème Système, Clair ou Sombre et l'interface se met à jour sans relancer Bridget.
2. Je peux choisir le fuseau Système ou un fuseau IANA précis pour l'affichage local des dates.
3. Je peux choisir une taille de police d'interface et, si proposé par le poste, le contraste et la réduction des animations.
4. La page indique explicitement que ces préférences sont locales au Mac.
5. Une préférence locale corrompue ou indisponible retombe sur une valeur sûre sans empêcher l'utilisation de Bridget.

## User Story 3 - Consulter et régler un serveur enregistré - P1

Comme opérateur, je veux ouvrir la fiche de chaque serveur enregistré afin de voir ses capacités, ses valeurs actives et de modifier les réglages que ce serveur autorise.

### Scénarios d'acceptation

1. La liste Serveurs montre le nom, l'adresse de confiance, la présence, la version, l'heure locale et la dernière synchronisation de chaque serveur.
2. L'ouverture d'un serveur présente les onglets Vue d'ensemble, Fournisseurs, Exécution et limites, Projets, Observabilité, Maintenance et Sécurité, selon ses capacités.
3. Chaque ligne précise si elle est modifiable, en lecture seule, indisponible ou contrôlée par une politique de projet.
4. Les réglages modifiables sont proposés par un catalogue fermé de type, plage, unité, valeur par défaut et conséquence attendue.
5. Les réglages absents sur un serveur ne sont ni inventés ni copiés depuis un autre serveur.

## User Story 4 - Modifier sans perdre le contrôle - P1

Comme opérateur, je veux prévisualiser puis confirmer une mutation de serveur afin de comprendre son impact et d'éviter d'écraser une modification concurrente.

### Scénarios d'acceptation

1. Modifier une valeur ouvre une prévisualisation avec ancienne valeur, nouvelle valeur, portée, effet attendu, avertissements et numéro de génération attendu.
2. Une confirmation explicite et locale est nécessaire avant toute écriture de serveur.
3. Le serveur applique toutes les valeurs d'une même demande atomiquement ou n'en applique aucune.
4. Si la génération a changé entre prévisualisation et confirmation, l'interface bloque l'écriture, recharge l'état et explique le conflit.
5. Après succès, l'interface montre un reçu avec serveur, opérateur, horodatage, réglages changés, génération résultante et identifiant de corrélation.
6. Un refus, un délai ou une perte de connexion ne met pas l'interface dans un état supposé réussi.

## User Story 5 - Comprendre usage et estimation de coût - P1

Comme opérateur, je veux visualiser les jetons et estimations de coût par fournisseur afin de choisir mes capacités d'exécution avec un signal honnête sur leur consommation.

### Scénarios d'acceptation

1. La page Usage permet de filtrer une période, fournisseurs, modèles et projets pour le serveur ouvert; elle identifie toujours ce serveur dans son en-tête.
2. Elle affiche les jetons d'entrée, de sortie, de cache et les unités provider qui sont réellement observées, avec leur couverture temporelle.
3. Elle affiche un total par fournisseur, une évolution journalière et un détail par modèle quand les observations le permettent.
4. Chaque montant est libellé Estimation API, porte la version et la date de la grille tarifaire et signale les observations non tarifées.
5. Les données issues d'un abonnement, d'un modèle inconnu ou d'une mesure incomplète restent identifiées comme non facturables ou partielles.
6. Le choix de fuseau horaire indique clairement s'il découpe les journées selon le Mac, un fuseau choisi ou l'heure du serveur affiché.

## User Story 6 - Voir l'état de maintenance sans automatiser - P2

Comme opérateur, je veux voir si une version plus récente de Bridget est disponible afin de planifier une mise à jour sans déclencher un changement de serveur inattendu.

### Scénarios d'acceptation

1. La page Mises à jour distingue version de Bridget Desktop, version de chaque serveur et date de la dernière vérification.
2. Une disponibilité de version n'est affichée que lorsqu'elle est attestée par une source configurée et vérifiée.
3. L'action proposée est Consulter les instructions ou Ouvrir les diagnostics. Elle ne déclenche ni téléchargement ni installation automatique.
4. Les diagnostics exposent un état technique borné, sans secrets, messages utilisateurs ni contenu de commandes.

## Exigences fonctionnelles

- FR-8001: L'interface DOIT offrir une roue d'engrenage persistante en bas de la barre latérale qui ouvre le centre de contrôle au clavier et à la souris. Cette roue est l'unique entrée des réglages du serveur affiché : la carte de profil Desktop ne DOIT PAS rendre d'action Réglages concurrente.
- FR-8002: Le centre DOIT proposer les sections Préférences du Mac, Serveurs, Usage, Mises à jour et Diagnostics dans une navigation recherchable.
- FR-8003: Chaque réglage DOIT afficher sa portée, son état de disponibilité et une description courte de son effet.
- FR-8004: Les préférences de thème, fuseau horaire, taille de police et options d'accessibilité DOIVENT rester propres au Mac et ne jamais être envoyées à un serveur.
- FR-8005: Le fuseau personnalisé DOIT être validé comme identifiant IANA avant persistance; en cas d'échec, le fuseau Système reste actif.
- FR-8006: Le serveur DOIT publier un inventaire versionné de capacités et de réglages autorisés, composé d'identifiants fermés et de schémas typés.
- FR-8007: Bridget Desktop DOIT afficher un écran spécifique pour chaque serveur enregistré et ne DOIT présenter que les catégories déclarées par ce serveur.
- FR-8008: Chaque réglage de serveur DOIT indiquer s'il est modifiable, lecture seule, contrôlé par projet ou interdit, avec sa justification.
- FR-8009: Les réglages de projet DOIVENT rester rattachés à leur projet et ne DOIVENT pas être rendus modifiables par la page globale de serveur.
- FR-8010: Toute mutation de serveur DOIT commencer par une requête de prévisualisation sans effet de bord.
- FR-8011: La prévisualisation DOIT contenir la génération de configuration attendue, le delta typé, les avertissements, les conséquences prévues et les causes de refus éventuelles.
- FR-8012: Toute mutation DOIT exiger une confirmation explicite dans Bridget Desktop et transmettre la génération attendue au serveur.
- FR-8013: Le serveur DOIT refuser une écriture portant une génération obsolète et renvoyer l'état permettant à l'opérateur de refaire sa prévisualisation.
- FR-8014: Une demande multi-réglages DOIT être atomique: succès complet avec une génération nouvelle ou absence de modification.
- FR-8015: Chaque mutation réussie DOIT produire un reçu durable sans secret, comportant l'identité du serveur, la date, le delta, l'ancienne et la nouvelle génération et une corrélation.
- FR-8016: Les commandes de shell, chemins libres, arguments libres, variables d'environnement, identifiants SSH, installation de logiciel, pare-feu, Docker et politiques de secrets DOIVENT être exclus du catalogue modifiable.
- FR-8017: Les secrets, leur valeur, leur source et les approbations sensibles DOIVENT rester hors de cette interface de réglages de serveur.
- FR-8018: Aucun agent, fournisseur ou message transporté ne DOIT pouvoir initier ou confirmer une mutation de réglage de serveur.
- FR-8019: La page Usage DOIT dériver ses totaux d'observations immuables horodatées, étiquetées par serveur, fournisseur, modèle, projet, unité et provenance.
- FR-8020: Une observation d'usage absente ou sans granularité suffisante DOIT rester absente ou partielle, jamais être représentée comme zéro.
- FR-8021: Une estimation de coût DOIT référencer une grille tarifaire versionnée avec devise, date d'effet, source et règle de compatibilité de modèle.
- FR-8022: Les modèles ambigus, abonnements, observations incomplètes ou prix inconnus DOIVENT être comptés séparément des montants estimés et rester clairement signalés.
- FR-8023: Les agrégats quotidiens DOIVENT déclarer le fuseau de découpage et conserver la possibilité de recalculer une période depuis les événements source.
- FR-8024: La page Mises à jour DOIT seulement présenter l'information de version attestée et une orientation manuelle. Elle ne DOIT pas exécuter une mise à jour.
- FR-8025: Les diagnostics DOIVENT être bornés aux métadonnées utiles de santé, version, capacité et dernière synchronisation, sans messages, contenu ou secret.
- FR-8026: Le comportement de lecture, prévisualisation, confirmation, application et reçu DOIT fonctionner au travers du même chemin sécurisé que les autres commandes Bridget vers un serveur enregistré.
- FR-8027: `Commande + virgule` DOIT ouvrir le centre de contrôle de Bridget dans le WebView macOS, au même titre que la roue visible.
- FR-8028: La colonne Projets DOIT être limitée à l'ajout, l'import, le filtrage, le repli et les actions contextuelles sur un projet. Elle ne DOIT PAS proposer un second accès global aux réglages du serveur.
- FR-8029: Chaque projet DOIT posséder une présentation locale avec initiales et couleur validées. Cette présentation NE DOIT produire aucune écriture vers le serveur.
- FR-8030: Les actions de retrait et de personnalisation d'un projet DOIVENT être rattachées au menu contextuel de ce projet, pas à un pied de colonne ambigu.
- FR-8031: Les actions d'ajout, d'import, de retrait, de réactivation et de reconnexion d'un projet DOIVENT emprunter les routes versionnées du relais et la capacité locale `ProjectRegistryV1`. Elles ne DOIVENT pas simuler un catalogue de coordinateurs ni utiliser une route absente.
- FR-8027: Une perte de connexion, une réponse invalide, une capacité retirée ou une erreur de validation DOIVENT conserver la dernière valeur confirmée et fournir une erreur actionnable.
- FR-8028: Les préférences locales DOIVENT être versionnées, validées et tolérer un stockage corrompu ou indisponible.

## Exigences non fonctionnelles

- NFR-8001: L'ouverture locale du centre de contrôle et la navigation entre ses sections DOIVENT rester inférieures à 150 ms hors attente réseau.
- NFR-8002: Le serveur NE DOIT effectuer aucune mutation lors d'une lecture, d'un rafraîchissement ou d'une prévisualisation.
- NFR-8003: Les écritures de réglages doivent être idempotentes dans la fenêtre de corrélation définie par le protocole existant.
- NFR-8004: Les données d'usage et les reçus ne doivent contenir ni texte de conversation, ni prompt, ni sortie d'agent, ni valeur de secret.
- NFR-8005: Les pages doivent rester utilisables au clavier, annoncer les erreurs de validation et respecter la réduction d'animation du système quand elle est active.
- NFR-8006: Les opérations sur un serveur indisponible doivent expirer de façon bornée et ne jamais empêcher l'utilisation d'un autre serveur ou des réglages locaux.
- NFR-8007: Toute collecte et agrégation d'usage doit avoir une complexité linéaire en nombre d'événements de la période ou s'appuyer sur des agrégats indexés équivalents.
- NFR-8008: Aucun paquet, service externe ou nouvelle dépendance ne peut être ajouté sans preuve de besoin, comparaison avec les composants existants et décision documentée.

## Cas limites

- Le poste ne possède aucun serveur enregistré.
- Un serveur est hors ligne, présente un hôte inconnu ou change de capacité pendant la consultation.
- Deux opérateurs prévisualisent puis tentent de modifier le même réglage.
- Une mutation comprend un réglage autorisé et un réglage devenu interdit.
- Le serveur contient une valeur historique hors de la plage proposée par le nouveau catalogue.
- Une période d'usage contient plusieurs fuseaux d'origine, une horloge serveur décalée ou un modèle renommé.
- Le fournisseur ne remonte que des jetons cumulés sans événement daté.
- La grille tarifaire ne connaît pas un modèle ou la devise n'est pas disponible.
- Le stockage local des préférences est refusé ou contient une version inconnue.
- Une version déclarée disponible ne possède pas encore d'instructions de mise à jour compatibles avec le serveur.

## Données et confidentialité

Les préférences de présentation restent locales au Mac. L'inventaire de capacités et les reçus de mutation ne conservent que des identifiants de réglage, valeurs non sensibles autorisées, générations et métadonnées d'audit. Les événements d'usage ne contiennent que des compteurs et dimensions techniques nécessaires à l'agrégation. Les contenus de conversation, prompts, sorties, secrets, identifiants d'authentification et chemins d'hôte ne font pas partie des écrans ni des nouveaux stockages.

## Hypothèses

- Les serveurs sont déjà enregistrés et joignables par le mécanisme de tunnel et d'identité de Bridget.
- Le Mac est l'origine d'autorité pour confirmer une mutation initiée par l'opérateur.
- Les capacités existantes de projets, fournisseurs, exécutions et observabilité sont réutilisables mais ne suffisent pas à elles seules à constituer un catalogue de réglages de serveur.
- La première version présente une mise à jour comme information et n'effectue aucune maintenance distante.
- Le prix des API est une estimation. Les abonnements fournisseur n'exposent pas nécessairement une facture ou un prix unitaire fiable.

## Décision de périmètre

Le serveur ne devient pas une collection de pages administrables arbitraires. La première version couvre seulement un catalogue de réglages fermé et justifié, avec des catégories attendues telles que disponibilité des fournisseurs, limites d'exécution, intervalles d'observabilité et maintenance informative. Toute nouvelle catégorie exige une capacité dédiée du serveur, son schéma, son impact, son test et son niveau d'autorisation.

## État d'implémentation au 2026-08-31

La tranche initiale est codée dans le worktree de la session. Elle apporte :

- une roue en bas de la barre des agents du panneau relié, ouvrant les réglages du serveur courant et un onglet Usage ;
- un catalogue fermé côté daemon. Seule `project_roots.allowed_roots` est modifiable, les catégories fournisseurs, exécution et maintenance sont explicitement en lecture seule ;
- aperçu sans écriture, confirmation locale, génération attendue et rejeu immédiat du même identifiant de commande pour cette mutation ;
- collecte future du fournisseur et du modèle attestés pour les échantillons d'usage. Les anciennes lignes restent inconnues et aucun coût n'est inventé sans grille tarifaire ;
- des préférences strictement locales à Bridget Desktop : nom d'affichage, thème, fuseau et taille de police.

Les éléments suivants restent délibérément incomplets : plusieurs réglages serveur modifiables, filtres et courbe d'usage, grille tarifaire datée, mises à jour et diagnostics attestés, ainsi que la validation manuelle avec un serveur réel. Ils restent des tâches non cochées afin de ne pas les présenter comme livrés.

## Critères de succès

- SC-8001: Un opérateur atteint la fiche d'un serveur et distingue les valeurs locales, serveur, projet et lecture seule en trois interactions ou moins depuis la vue principale.
- SC-8002: Cent pour cent des écritures de serveur passent par prévisualisation, confirmation locale, génération attendue et reçu.
- SC-8003: Une écriture concurrente ou obsolète ne modifie aucune valeur et mène à une prévisualisation renouvelée explicite.
- SC-8004: Les tests automatisés couvrent les capacités absentes, lecture seule, mutation atomique, génération obsolète, refus, délai et reçu.
- SC-8005: Les tests d'usage couvrent compteur absent, source cumulative, modèle inconnu, prix absent, filtre de période et fuseau de découpage.
- SC-8006: Aucun secret, contenu conversationnel ou commande libre ne peut être consulté ou transmis par les nouveaux contrats.
- SC-8007: Le total affiché de coût porte toujours l'étiquette Estimation API et les observations non tarifées restent séparées.
- SC-8008: La consultation d'une version disponible ne déclenche ni installation ni redémarrage d'un serveur.
- SC-8009: Les tests existants pertinents restent passants et les nouveaux contrats sont documentés avec des exemples de succès et de refus.

## Tests attendus

- Unitaires daemon: catalogue de réglages, validation de type et de portée, génération concurrente, atomicité, reçus et bornes de diagnostics.
- Unitaires transport: encodage versionné des lectures, prévisualisations, confirmations, mutations et résultats d'erreur.
- Unitaires usage: conservation d'événement, agrégation par période, provenance, cache, couverture, prix connu ou inconnu et fuseau.
- Unitaires interface: navigation, préférences locales, accessibilité clavier, états déconnectés et rendu des portées.
- Intégration: Bridget Desktop vers faux serveur, de la lecture jusqu'au reçu, incluant un conflit de génération et une absence de capacité.
- Vérification manuelle: parcours réel avec au moins un serveur enregistré, sans appliquer de mise à jour.
