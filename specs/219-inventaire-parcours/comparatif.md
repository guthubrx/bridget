# Comparaison219 — parcours utilisables, pas nombre de fonctions

Date : 30 octobre2026. Demande : identifier les avantages réels de T3 avant d’implémenter l’actualisation des modèles dans Bridget.

## Conclusion utile

Nous n’avons pas démontré que Bridget avait rattrapé T3. Plusieurs fonctions existent des deux côtés, mais les parcours qui les relient sont moins intégrés chez Bridget. Les écarts prioritaires concernent le compte effectif, les modèles réellement observés sur chaque serveur, les composants encore anciens et la résolution d’un incident sans mainteneur. La comparaison ne prouve pas qu’un seul défaut explique tous les incidents passés.

Le besoin produit est : « Sur chaque serveur, voir ce qui est installé, quel compte est connecté, quels modèles ce programme annonce pour ce compte, quand cela a été vérifié et quelle action faire si cette preuve manque. » Une liste déclarée n’est pas une validation d’accès. Une liste issue du fournisseur ne garantit pas l’acceptation du prochain tour.

T3 a aussi des avantages d’interface et de cycle de travail : connexion guidée, transfert de contexte, points de reprise, Git/PR, navigateur et appareils intégrés. Leur présence ne demande pas de les reproduire tous dans Bridget. Ils sont recensés pour décider, sans transformer cette étude en autorisation générale d’implémentation.

## Méthode et limites

- Bridget : source0582c6bbce1b7c10b02f4ced54731347fdf79010, services217, MCP appelant214. La source locale est un miroir ; le checkout de travail est /home/moi/10.Scripts/64.bridget sur Cartae.
- T3 installé :0.1.0-local.20261019.dev. Bundle local SHA25642eeb05baf987f0bfd1329f09955d94968a92bd8b6f6daddb888e78ec0650bc6. Copie de lecture : /Users/moi/.cache/bridget217/t3-comparison/server-source.mjs.
- Documentation amont figée au commit fa4e4677c0ecc437aece2dcea39f76ef31414152. Elle peut décrire des fonctions plus récentes que l’application locale. Les verdicts documentaires le disent.
-48 parcours inventoriés. Source, documentation et catalogue vivant ont été distingués. Ce n’est pas une certification, une comparaison de performances ni un audit de sécurité complet. Aucune note de qualité artificielle.
- Aucun nouveau travail métier, login, changement de compte, modèle, droit ou serveur ; aucun redémarrage pour cette comparaison. Les mutations d’étude sont documentaires.
- « Non identifié » signifie absent des surfaces CLI/MCP, guides et modules inspectés ; cela n’est pas une preuve absolue d’absence dans tout fichier ou toute extension.
- Aucun essai de chacun des48 parcours. Aucun verdict général sur la fiabilité de T3 ou sur son absence de bugs.

## Corrections de nos affirmations précédentes

1. T3 n’est pas seulement local : ses documents décrivent SSH, environnements liés et équilibrage, avec des limites et une bêta pour les liens.
2. Son rafraîchissement Codex géré est relié aux événements de connexion. Cela ne prouve pas une actualisation systématique de tous les comptes externes et de tous les serveurs. Le login CLI externe peut demander un rafraîchissement manuel.
3. Son attente du cache a une limite de trois secondes. Il peut conserver un catalogue en cache avec avertissement. Il accepte aussi des modèles personnalisés. Ni cette liste ni un login réussi ne prouvent l’accès à tous les modèles.
4. L’identité du client peut influencer le catalogue fournisseur. Bridget doit employer sa propre identité officielle ; aucune imitation de T3 n’est autorisée ou nécessaire.
5. Une version compilée, installée ou un serveur redémarré ne prouve pas que chaque composant appelant a changé. Le client MCP214 de ce fil reste une limite réelle et déclarée.

## Priorités proposées

P0 : actualiser et corréler compte/modèles/serveur ; expliquer l’incertitude ; afficher le composant ancien et son geste précis ; rendre les clôtures et reprises exploitables. L’actualisation est expressément demandée. La prévention218 du drainage reste séparément ouverte après libération réelle des tâches.

P1 : parcours guidé de préparation des machines et comptes ; réglages par projet/serveur ; intégration d’usage/coût ; transfert de contexte et suivi Git/PR si ces parcours sont retenus par l’utilisateur.

P2 : interface de navigateur/appareils, webhooks, nouvelles plateformes, points de reprise et forks. Ce sont des extensions possibles, pas des prérequis de l’actualisation demandée.

## Inventaire détaillé

### 01 — Voir le compte utilisé sur la machine

Famille : Comptes et modèles. Verdict : **Avantage T3**. Priorité : P0.

T3 : État connecté, type de compte et offre intégrés au statut de l’instance.

Bridget : Le catalogue de flotte expose une déclaration et entitlement=unknown. La connexion réelle a demandé une investigation séparée.

Preuves : T1,T2,B1,R1. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 02 — Rafraîchir les modèles après une connexion

Famille : Comptes et modèles. Verdict : **Avantage T3**. Priorité : P0.

T3 : Le Codex géré réactualise son snapshot après les événements du contrôleur de connexion.

Bridget : Pas de lien automatique entre changement de compte et catalogue de flotte.

Preuves : T1,T2,B1. Limite : Code du Codex géré vérifié. Le login effectué hors de ce contrôleur peut demander un rafraîchissement manuel.

### 03 — Relier modèles, compte et serveur

Famille : Comptes et modèles. Verdict : **Avantage T3**. Priorité : P0.

T3 : Instances configurées séparément ; lecture account/read puis model/list dans le contexte de chacune.

Bridget : Connexions et modèles déclarés par serveur ; pas d’observation fraîche corrélée au compte dans fleet.options.

Preuves : T1,T3,B1. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 04 — Savoir si la liste est fraîche

Famille : Comptes et modèles. Verdict : **Écart des deux côtés**. Priorité : P0.

T3 : Attend un cache plus récent que le démarrage. Après trois secondes, peut garder une liste en cache avec avertissement.

Bridget : observed_at et provenance portent la déclaration. Ils ne prouvent pas la fraîcheur d’une lecture fournisseur.

Preuves : T1,B1. Limite : La date d’un fichier partagé ne prouve pas à elle seule la corrélation au compte ou au client. Aucun catalogue ne garantit l’acceptation du prochain tour.

### 05 — Options propres à chaque modèle

Famille : Comptes et modèles. Verdict : **Avantage T3**. Priorité : P1.

T3 : Le catalogue expose effort, vitesse, contexte et valeurs par défaut selon les capacités retournées ou déclarées.

Bridget : Le catalogue de flotte normalise surtout modèle et effort. Des réglages supplémentaires existent dans certains adaptateurs, sans contrat uniforme de flotte.

Preuves : T1,R2,B1,B2. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 06 — Plusieurs comptes sans écraser le login principal

Famille : Comptes et modèles. Verdict : **Avantage T3**. Priorité : P1.

T3 : Profils et espaces Codex distincts ; connexion gérée et configuration de home séparée.

Bridget : Des définitions distinctes sont possibles, mais préparation et connexion restent surtout opérateur/CLI.

Preuves : T2,D1,B3. Limite : Ne pas copier les secrets ni reprendre l’identité d’un autre client pour reproduire ce parcours.

### 07 — Expliquer un modèle refusé

Famille : Comptes et modèles. Verdict : **Commun partiel**. Priorité : P0.

T3 : L’authentification et le catalogue aident avant le lancement ; pas de preuve d’absence de tout refus tardif.

Bridget : 217 explique model_not_found attesté et demande de vérifier compte/offre. Les anciennes erreurs non conservées restent inconnues.

Preuves : T1,B4. Limite : La correction 217 est active. Aucun échec volontaire fournisseur lancé pour cette comparaison.

### 08 — Préparer toutes les machines depuis un parcours guidé

Famille : Installation et versions. Verdict : **Avantage T3**. Priorité : P1.

T3 : Assistant qui sélectionne les ordinateurs, vérifie les agents et prépare l’installation ou la connexion correspondante.

Bridget : Installateur avec choix des programmes et fédération SSH. Plusieurs gestes opérateur restent séparés.

Preuves : D2,B3,B5. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 09 — Détecter une version de fournisseur incompatible

Famille : Installation et versions. Verdict : **Commun partiel**. Priorité : P1.

T3 : Politique de compatibilité par fournisseur et version de T3, avec version recommandée.

Bridget : 209 contrôle l’inventaire et les minima de support avant bascule. Pas de matrice complète modèle/fonction/version.

Preuves : T4,B6. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 10 — Mettre à jour fournisseurs, serveurs et application ensemble

Famille : Installation et versions. Verdict : **Avantage T3 documenté**. Priorité : P0.

T3 : Parcours global qui nomme machines ignorées/échouées et composants encore anciens, puis réactualise les catalogues.

Bridget : Mise à jour des services et contrôle des images chargées existent. Un ancien MCP peut rester chargé après la bascule et réclamer un geste externe.

Preuves : D3,B6,R3. Limite : Parcours décrit dans les documents amont figés ; pas de mise à jour T3 exécutée pendant cette étude.

### 11 — Reconnaître le composant encore ancien

Famille : Installation et versions. Verdict : **Commun partiel**. Priorité : P0.

T3 : Contrôle de version de protocole et consigne désignant le côté à mettre à jour.

Bridget : Négociation et avertissements existent ; services217 et client appelant214 coexistent ici. Le rechargement du composant hôte reste distinct.

Preuves : D3,B6,R3. Limite : Ne pas imposer une égalité de numéro de build si les protocoles sont compatibles.

### 12 — Reprendre après une mise à jour

Famille : Installation et versions. Verdict : **Non départagé**. Priorité : P0.

T3 : Continuation portable des fils pris en charge ; commandes et travaux de fond peuvent être interrompus et signalés.

Bridget : Reconnexion des wrappers et conservation des tâches distantes existent. Plusieurs incidents historiques et le défaut218 montrent des limites réelles.

Preuves : T5,D3,B7,R4. Limite : Aucun redémarrage comparatif contrôlé des deux produits. Pas de verdict général de fiabilité.

### 13 — Ajouter un serveur et s’y connecter

Famille : Serveurs et projets. Verdict : **Avantage T3 sur le parcours**. Priorité : P1.

T3 : Ajout d’environnement, installation/réutilisation du serveur SSH, tunnel et association intégrés.

Bridget : Fédération SSH et identités de worker existent. Paramétrage du projet et des profils séparé.

Preuves : D4,B5. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 14 — Répartir automatiquement un travail

Famille : Serveurs et projets. Verdict : **Commun, stratégies différentes**. Priorité : P1.

T3 : Équilibrage de nouveaux fils documenté ; choix stabilisé, préférences par machine.

Bridget : Admission native compare ressources, charge, droits et continuité, puis fige son choix.

Preuves : D4,B8. Limite : Le mécanisme T3 documenté ne signifie pas que l’équilibrage est activé dans cette installation.

### 15 — Déléguer entre machines

Famille : Serveurs et projets. Verdict : **Commun, non départagé**. Priorité : P1.

T3 : Environnements liés documentés en bêta, avec plafond d’accès et contrôle explicite.

Bridget : Missions natives distantes, projet source et placement vérifiés, propriétaire et reçus conservés.

Preuves : D4,B7. Limite : Aucun lien T3 distant créé. Ne pas prétendre que T3 est seulement local, ni que sa bêta est qualifiée ici.

### 16 — Regrouper le même projet sur plusieurs machines

Famille : Serveurs et projets. Verdict : **Avantage T3 sur le parcours**. Priorité : P1.

T3 : Import et regroupement de dépôts par identité ; paramètres par projet et environnement avec origine de valeur visible.

Bridget : Mappings Git contrôlés et préparation distante automatique existent. Leur cohérence a encore demandé des interventions de configuration.

Preuves : D2,D5,B9. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 17 — Garantir le commit demandé

Famille : Serveurs et projets. Verdict : **Commun partiel**. Priorité : P0.

T3 : Gestion Git et préparation avant lancement ; contexte du fil attaché au dossier.

Bridget : expected_base_commit, preuve Git source et placement distant font partie du contrat d’admission.

Preuves : T6,B9. Limite : Les exigences sont différentes. Aucune preuve que T3 offre exactement la même garantie de SHA entre machines.

### 18 — Extraire un dossier léger

Famille : Serveurs et projets. Verdict : **Point fort Bridget vérifié**. Priorité : P1.

T3 : Aucun équivalent au profil compact207 identifié dans les parcours inspectés.

Bridget : Profil compact-v1 par worktree, audit des drapeaux et conservation de l’ancre Git ; mesures historiques documentées.

Preuves : B10. Limite : Absence limitée au périmètre inspecté ; aucun benchmark T3/Bridget à entrées identiques.

### 19 — Lire une base sans bloquer sur un autre dossier

Famille : Serveurs et projets. Verdict : **Limite Bridget établie, T3 non départagé**. Priorité : P0.

T3 : Dossier par fil et worktree possible ; pas de preuve d’un lecteur immuable indépendant sans extraction.

Bridget : Un rafraîchissement shared peut être refusé si un worktree descendant a un reçu actif. Incidents réels constatés.

Preuves : B11,R4. Limite : Ne pas supprimer la protection de travail actif pour imiter un parcours plus permissif.

### 20 — Changer de fournisseur en conservant le contexte

Famille : Conversations et reprise. Verdict : **Avantage T3**. Priorité : P1.

T3 : Transfert de messages attribués, budget et références vers l’historique omis ; demande nouvelle conservée.

Bridget : Passations et fils existent. Pas de parcours uniforme de transfert de conversation entre moteurs identifié dans les interfaces inspectées.

Preuves : T7,D6,B12. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 21 — Créer une bifurcation depuis un tour

Famille : Conversations et reprise. Verdict : **Avantage T3**. Priorité : P2.

T3 : Fork et contexte de point de reprise reliés au fil.

Bridget : Pas de bifurcation native de conversation avec état du fournisseur identifiée.

Preuves : T8,B12. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 22 — Revenir à un point de reprise avec les fichiers

Famille : Conversations et reprise. Verdict : **Avantage T3**. Priorité : P2.

T3 : Capture et restauration ; la restauration des fichiers exige l’isolation du dossier.

Bridget : Rapports Git et branches conservées ; pas de restauration conversation+fichiers par point de reprise identifiée.

Preuves : T9,B10. Limite : Fonction potentiellement destructive : aucune restauration exécutée.

### 23 — Importer des conversations existantes

Famille : Conversations et reprise. Verdict : **Avantage T3 documenté**. Priorité : P2.

T3 : Import borné de l’historique Claude/Codex avec suivi des éléments déjà importés.

Bridget : Reprise des sessions natives et journaux propres ; pas d’import de projets/historiques équivalent identifié.

Preuves : D2,B12. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 24 — Continuer une mission après son résultat

Famille : Conversations et reprise. Verdict : **Commun avec limite différente**. Priorité : P1.

T3 : Fils ordinaires continuables et tâches déléguées rattachées à un parent.

Bridget : Rondes natives sur enfant gardé, durée et plafond de huit rondes explicités.

Preuves : T10,B13. Limite : Pas d’équivalence exacte de cycle de vie. Le plafond Bridget n’est pas un bug en soi.

### 25 — Annuler et savoir si tout est arrêté

Famille : Conversations et reprise. Verdict : **Non départagé, défaut Bridget constaté**. Priorité : P0.

T3 : Interruption et reprise de session gérées ; pas de recette physique identique réalisée.

Bridget : Arrêt attesté et cleanup_done distincts du résultat. L’incident e3 a nécessité une maintenance ciblée après blocage du drainage.

Preuves : T10,B14,R4. Limite : La correction préventive218 n’est PAS livrée. Ne pas annoncer que la présence d’un bouton Stop suffit dans l’un des produits.

### 26 — Rejouer sans créer de doublon

Famille : Conversations et reprise. Verdict : **Commun, non départagé**. Priorité : P0.

T3 : Reçus de commandes et suivi des effets dans l’orchestration inspectée. Certains outils n’ont pas de clé de retry.

Bridget : Canon durable, clé d’admission, issue inconnue et réconciliation de la même tâche.

Preuves : T11,B15. Limite : Ne pas comparer seulement les mots idempotence ou accepted. Pas de campagne commune de pertes de réponses.

### 27 — Comprendre une erreur sans appeler le mainteneur

Famille : Conversations et reprise. Verdict : **Progrès Bridget, écart produit restant**. Priorité : P0.

T3 : Statuts et actions intégrés aux comptes, environnements, tours et mises à jour.

Bridget : Guidages199–217 améliorés, mais des incidents ont encore exigé la lecture manuelle de plusieurs autorités.

Preuves : T1,T10,D3,B4,R4. Limite : Les historiques ne prouvent pas que tous les défauts sont encore présents. Cause inconnue reste inconnue.

### 28 — Éditer, ordonner et supprimer la file

Famille : Messages et coordination. Verdict : **Commun partiel**. Priorité : P2.

T3 : Édition du texte et des pièces jointes, ordre et promotion en intervention.

Bridget : 159B permet annuler, ordonner, promouvoir sous droits ; pas de remplacement du texte dans ce contrat.

Preuves : T12,B16. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 29 — Intervenir pendant le travail

Famille : Messages et coordination. Verdict : **Commun, droits différents**. Priorité : P1.

T3 : Message de pilotage lié au tour ; règles d’autorité du fil.

Bridget : Steer natif sous droits parent/humain ; not_parent est une protection, pas un défaut à contourner.

Preuves : T12,B16. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 30 — Répondre aux questions et permissions

Famille : Messages et coordination. Verdict : **Commun partiel**. Priorité : P1.

T3 : Questions et approbations dans le fil ; leur portée reste distincte.

Bridget : Demandes enfant relayées sous opt-in ; questions propriétaire et approbations humaines séparées.

Preuves : T10,B17. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 31 — Saisir un secret sans le donner au modèle

Famille : Messages et coordination. Verdict : **Avantage T3**. Priorité : P1.

T3 : Carte privée ; référence consommable une fois, valeur hors historique du modèle.

Bridget : Pas de port natif de demande/consommation de secret équivalent identifié ; recours aux connexions officielles sur la cible.

Preuves : T13,B17. Limite : Ce manque n’autorise jamais à demander une clé dans le chat ou à copier des identifiants.

### 32 — Joindre et lire des artefacts

Famille : Messages et coordination. Verdict : **Commun partiel**. Priorité : P2.

T3 : Pièces jointes au fil, visualisations et lecteurs intégrés.

Bridget : Artefacts versionnés, empreintes, droits et lecture bornée ; présentation mobile disponible.

Preuves : T14,B18. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 33 — Parcourir le web ensemble sur le serveur

Famille : Interface et outils. Verdict : **Avantage T3, extension de périmètre**. Priorité : P2.

T3 : Navigateur partagé humain/agent, contrôle, captures et enregistrements, par environnement.

Bridget : Pas de navigateur partagé intégré identifié dans les surfaces inspectées. Les outils externes du fournisseur restent possibles.

Preuves : T14,D4,B19. Limite : Outil T3 exposé dans ce fil ; aucune navigation métier exécutée pour le comparer.

### 34 — Afficher et contrôler simulateurs et émulateurs

Famille : Interface et outils. Verdict : **Avantage T3, extension de périmètre**. Priorité : P2.

T3 : Panneau d’appareils et hôtes SSH avec commandes bornées.

Bridget : Pas de panneau d’appareils équivalent identifié.

Preuves : T15,B19. Limite : Présence du port et du code vérifiée ; aucun appareil démarré.

### 35 — Revoir les modifications et gérer les PR

Famille : Interface et outils. Verdict : **Avantage T3**. Priorité : P1.

T3 : Diffs, commentaires, guides et demandes de fusion reliés aux conversations.

Bridget : Passage par Git, CLI/outils tiers et comptes rendus ; pas de parcours de revue graphique équivalent identifié.

Preuves : T16,D7,B19. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 36 — Être réveillé par un changement de PR

Famille : Interface et outils. Verdict : **Avantage T3**. Priorité : P2.

T3 : Suivi des contrôles et commentaires avec réveil du fil.

Bridget : Réveils génériques présents, pas de suivi natif spécialisé des PR identifié.

Preuves : T16,B20. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 37 — Utiliser un terminal lié au projet

Famille : Interface et outils. Verdict : **Avantage T3 sur l’interface**. Priorité : P2.

T3 : Terminal du projet accessible sur l’environnement.

Bridget : Attach dialogue avec l’agent, pas un shell de projet intégré. Le terminal externe reste disponible.

Preuves : D8,B12. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 38 — Retrouver et organiser le travail

Famille : Interface et outils. Verdict : **Avantage T3 sur l’interface**. Priorité : P1.

T3 : Recherche, épinglage, classement, mise en sommeil et métadonnées de fil.

Bridget : Annuaire paginé, fils partagés, registre et journaux ; expérience moins unifiée.

Preuves : T17,D9,B12. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 39 — Planifier un réveil

Famille : Automatisation et mesure. Verdict : **Commun partiel**. Priorité : P2.

T3 : Tâches récurrentes liées ou non au fil.

Bridget : Réveils périodiques ou calendaires durables ; ce ne sont pas tous des créations automatiques de mission.

Preuves : T18,B20. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 40 — Déclencher une tâche par webhook

Famille : Automatisation et mesure. Verdict : **Avantage T3, extension de périmètre**. Priorité : P2.

T3 : Déclencheur HTTP, template de corps et signature selon le contrat.

Bridget : WakeCadence ne contient que Interval et At ; pas de webhook natif identifié.

Preuves : T18,B20. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 41 — Reprendre après une limite d’usage

Famille : Automatisation et mesure. Verdict : **Commun, paramètres différents**. Priorité : P1.

T3 : Réglages de mise en sommeil et reprise à la date de retour.

Bridget : resume_on_limit opt-in, durée et nombre de reprises bornés, même fournisseur et droits.

Preuves : T19,B21. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 42 — Suivre tokens, coûts et offre du compte

Famille : Automatisation et mesure. Verdict : **Avantage T3 sur la présentation**. Priorité : P1.

T3 : Usage par modèle/fournisseur, limites d’abonnement et prix réglables.

Bridget : Quotas et crédits observés dans les instances ; pas de tableau de coût homogène comparable identifié.

Preuves : T1,D10,B21. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 43 — Observer la présence d’agents de plusieurs projets

Famille : Automatisation et mesure. Verdict : **Point fort Bridget**. Priorité : P1.

T3 : Gestion de fils et environnements. Pas de contre-preuve d’une équivalence exacte à l’annuaire interagents Bridget.

Bridget : Annuaire, messages interprojets justifiés, reçus de remise et registre de coordination.

Preuves : T17,B15. Limite : Ne pas qualifier ce point de supériorité globale de fiabilité.

### 44 — Lancer un nouveau chantier depuis le téléphone

Famille : Mobile et accès. Verdict : **Avantage T3 documenté**. Priorité : P1.

T3 : Création et pilotage de fils dans les applications mobiles.

Bridget : Application Android de suivi/pilotage d’agents existants ; pas de création directe de nouvel agent dans le contrat courant.

Preuves : D4,D11,B19. Limite : Lecture des sources et/ou documentation ; pas de nouvel essai métier.

### 45 — Proposer une même interface sur web, bureau et iOS

Famille : Mobile et accès. Verdict : **Avantage T3 documenté**. Priorité : P2.

T3 : Interfaces web, bureau, iOS et Android annoncées et décrites.

Bridget : CLI/MCP et Android ; pas d’interface produit web/bureau/iOS équivalente identifiée.

Preuves : D11,B19. Limite : Pas d’audit d’accessibilité ni d’essai des quatre plateformes pendant cette comparaison.

### 46 — Notifications et messages pendant une coupure

Famille : Mobile et accès. Verdict : **Commun partiel**. Priorité : P1.

T3 : Messages hors ligne, notification et reprise documentés.

Bridget : Brouillons/reçus chiffrés, notifications bornées et relecture après reconnexion.

Preuves : D12,B19. Limite : Pas de mesure comparative de délai ou de taux de remise.

### 47 — Conserver les restrictions entre fournisseurs

Famille : Droits et exploitation. Verdict : **Non départagé**. Priorité : P0.

T3 : Modes et capacités du fil/instance ; les refus restent possibles.

Bridget : Projection de droits et contrôles avant remise ; conversion T3→Codex214 volontairement fermée.

Preuves : T20,B22. Limite : Aucune analyse de sécurité comparative exhaustive. Ne pas obtenir la parité en supprimant un contrôle.

### 48 — Mesurer les prérequis du projet sur chaque serveur

Famille : Droits et exploitation. Verdict : **Lacune transversale à traiter**. Priorité : P1.

T3 : Scripts de préparation et réglages d’environnement ; pas de preuve d’une attestation complète Node/outils/déterminisme.

Bridget : Contrôles Git, ressources et programmes ; égalité Node215 obtenue par intervention ciblée, pas par un contrat projet complet.

Preuves : D5,B10,B23. Limite : Même version de Node ne prouve pas le déterminisme des octets. Rien ne permet de déclarer l’un des produits complet ici.

## Exigences pour l’actualisation autorisée

1. Observer sur le serveur cible et dans la configuration exacte de la connexion ; aucun compte d’une autre machine transposé.
2. Séparer programme installé, authentification observée, modèles annoncés, droits du parent et succès d’une vraie mission.
3. Corréler la preuve à une identité de compte opaque, au programme, à la connexion et à l’époque du worker. Ne pas exposer adresse, jeton ou secret dans le catalogue.
4. Actualiser après un changement de connexion reconnu, au démarrage/reconnexion pertinent et sur action explicite. Détecter aussi un login officiel réalisé hors de Bridget sans lire ni copier ses secrets.
5. Invalider l’observation de l’ancien compte avant de proposer les options du nouveau. Un échec reste inconnu ; aucun ancien choix présenté comme frais.
6. Borner les processus, le temps, la sortie, la pagination et la concurrence. Aucun tour de modèle ni mission de qualification pour simplement découvrir les modèles.
7. Garder droits, choix humains et tâches admises intacts. Aucun changement silencieux de fournisseur/modèle/compte, aucune reprise de mission d’issue inconnue.
8. Pour les fournisseurs sans découverte fiable, nommer la limite au lieu d’inventer un accès. Ne pas annoncer le support de tous les fournisseurs depuis le seul support Codex.
9. Donner à l’agent une prochaine action précise, sa portée et la décision humaine requise si elle sort de ses droits.
10. Qualifier compte changé, compte absent, programme ancien, cache périmé, pagination, coupure et simultanéité ; puis distribuer et vérifier les images chargées sur les hôtes concernés.

## Sources

- **T1** : Bundle installé /Users/moi/.cache/bridget217/t3-comparison/server-source.mjs : CodexProvider.ts, lignes223281–223655 ; account/read, requestAllCodexModels, requestCodexChatGptModels.

- **T2** : Même bundle : ProviderAuthService.ts lignes287564–287710 ; CodexManagedProvider.ts lignes233535–233540. Arrêt des sessions affectées et réactualisation après phase de connexion gérée.

- **T3** : Même bundle : ProviderInstanceRegistry.ts lignes127508–127690 ; corrélation d’instance et configuration propre.

- **T4** : Même bundle : providerCompatibility.ts lignes191126–191176 ; politique liée à la version du produit et du fournisseur.

- **T5** : Même bundle : ProviderRuntimeRecoveryService.ts lignes293170–293684 ; RestartContinuation.ts lignes292480–292593.

- **T6** : Même bundle : ThreadLaunchService.ts lignes294443–295024 ; GitWorkflowService.ts lignes274036–275175.

- **T7** : Même bundle : ContextHandoffService.ts lignes125824–126233 ; ProviderSwitchService.ts lignes278307–278394.

- **T8** : Même bundle : ThreadForkService.ts lignes278459–278529 ; outil t3_thread_fork réellement exposé.

- **T9** : Même bundle : CheckpointRestoreSafety.ts lignes120595–120674 ; CheckpointRollbackService.ts lignes290717–290935.

- **T10** : Même bundle : ProviderTurnControlService.ts lignes291312–291532 ; ports task_status/task_cancel/questions exposés.

- **T11** : Même bundle : CommandReceiptStore.ts lignes120486–120594 ; EffectOutbox.ts lignes126426–126919.

- **T12** : Outils t3_queue_edit/reorder/promote_to_steer exposés ; descriptions et schémas dans le bundle MCP.

- **T13** : Même bundle : SecretRequests.ts lignes295098–295226 ; outil request_secret exposé.

- **T14** : Outils preview_*, html_preview/html_render et pièces jointes exposés ; PreviewBrowser.ts/ServerBrowser.ts dans le bundle.

- **T15** : Même bundle : DeviceService.ts lignes309601 et suivantes ; outils device_* exposés.

- **T16** : Même bundle : PullRequestService.ts, PullRequestWatchReactor.ts lignes298159–298492 ; outils PR exposés.

- **T17** : Même bundle : ThreadManagementService.ts lignes286561–287116 ; ThreadSearch.ts lignes301916–302030.

- **T18** : Même bundle : ScheduledTaskService.ts lignes295405 et suivantes ; outils schedule_task et schémas webhook exposés.

- **T19** : Même bundle : UsageLimitRecoveryWorker.ts lignes287117–287302.

- **T20** : Modes et limites retournés par orchestrator_capabilities ; documentation permission-modes.md.

- **B1** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs : RoutingPolicy/model_catalog et routing_catalogue, lignes306–316,807–859.

- **B2** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs : select_runtime, lignes1187–1260 ; model/list pour un changement humain, pas le catalogue de flotte.

- **B3** : /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/install.sh : prepare_providers et check_clients ; aucun login automatique.

- **B4** : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/217-refus-autonomes/validation.md et code de projection propriétaire associé.

- **B5** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/federation-services.fr.md ; script /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/federate-ssh.sh.

- **B6** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/automatic-updates.fr.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/client_compatibility.rs.

- **B7** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.fr.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_start_recovery.rs.

- **B8** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs : ressources, scores et admission ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/host_resources.rs.

- **B9** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.fr.md : préparation172 et SHA211 ; code lifecycle/workspace_git.

- **B10** : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/207-preparation-econome/validation.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/workspace_compact.rs.

- **B11** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/remote_worker.rs : shared_refresh_allowed, autour2720.

- **B12** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.fr.md ; CLI, MCP, handoff et attach dans les sources correspondantes.

- **B13** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.fr.md : contrat153, gardes et plafond de huit rondes.

- **B14** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs : cancel_tree et cleanup_done ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/managed_process.rs : remote_drain.

- **B15** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.fr.md : reçus, annuaire et échanges ; magasins d’idempotence.

- **B16** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.fr.md : 156 et 159B, lignes175–313.

- **B17** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation_mcp.rs : outils status/respond et child_requests.

- **B18** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/artifact_service.rs ; artifact_store.rs et règles de lecture.

- **B19** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/mobile.fr.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/android/app/src/main/java/app/cartae/bridget/ ; inventaire CLI/MCP inspecté.

- **B20** : /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wake_schedules.rs ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs : WakeCadence, ligne4405.

- **B21** : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.fr.md : quotas157 et reprise160.

- **B22** : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/214-preparation-fiable/validation.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/native_permissions.rs.

- **B23** : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/215-runtime-node/validation.md.

- **R1** : Lecture actuelle de bridget_capabilities : fleet.status=declared,45 options,entitlement=unknown ; aucune requête modèle lancée.

- **R2** : Lecture actuelle orchestrator_capabilities : instances Codex et cx-pro, huit modèles chacune et options ; présence de fournisseurs désactivés ou non authentifiés explicitée.

- **R3** : Preuve de livraison217 : services217 chargés, MCP de ce fil214 conservé. Pas de reconnexion en boucle.

- **R4** : Constats directs218 : e3f4498a avait un wrapper214 vivant,408 zombies dont un UID0. Intervention ciblée puis reçu worker Terminal et cleanup_done=true. Correction préventive non livrée ; preuve privée /Users/moi/.cache/bridget218/diagnostic.json.

- **D1** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/providers-codex.md

- **D2** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/welcome-wizard.md

- **D3** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/updating.md

- **D4** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/remote-access.md

- **D5** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/project-settings.md

- **D6** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/portable-handoffs.md

- **D7** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/source-control.md

- **D8** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/terminal.md

- **D9** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/thread-sidebar.md

- **D10** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/usage.md

- **D11** : https://github.com/pingdotgg/t3code ; outils et documentation mobile amont, pas de test de toutes les plateformes.

- **D12** : https://github.com/pingdotgg/t3code/blob/fa4e4677c0ecc437aece2dcea39f76ef31414152/docs/user/composer.md
