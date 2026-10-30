# Session221 — capacités complètes et vérifiables

Statut : In Progress. Date : 30 octobre 2026. Branche : session-221-capacites-completes.

## Demande initiale

« Je veux être le meilleur des deux sur toutes ces catégories : soit égalité, soit le meilleur. On reprend Bridget et ses capacités et on corrige les éventuels manques ou limitations constatés tant sur notre solution que sur l’autre. » Invocation my-specify-all : périmètre complet, implémentation, intégration, push, distribution, activation et nettoyage autorisés.

## Demande reformulée et apport

Faire de Bridget une solution au moins aussi capable dans chacun des50parcours recensés en219. Vérifier les acquis, corriger les manques et les limites communes confirmés, préserver droits et travaux, puis livrer les versions effectivement chargées. L’égalité porte sur le résultat utilisateur et ses garanties. Une supériorité exige un gain mesuré sur le même scénario. La présence de code, un texte commercial ou un outil exposé ne suffisent pas.

Ce cadrage remplace une promesse générale par50critères réfutables. Il ne demande pas de modifier un produit tiers. Les sources comparatives sont réservées à l’étude et aux adaptateurs ; les parcours produit sont définis par Bridget.

## Périmètre et exclusions

Les50catégories sont incluses, y compris interface, mobile/iOS, Windows comme client, navigateurs, appareils et moteurs supplémentaires. Aucune catégorie n’est supprimée au motif qu’elle est nouvelle. Les livraisons intermédiaires restent possibles mais le programme reste In Progress tant qu’un critère manque. Les limites d’accès d’un abonnement ne sont pas des capacités que Bridget peut créer. Tout accès, abonnement ou dépense non autorisé requiert sa décision propre. Aucun service métier Opus2D/NE02/Roadie, ancienne mission d’issue inconnue, compte ou secret ne doit être déplacé ou relancé. Les tests synthétiques et les revues restent séparés des métiers.

## Scénarios utilisateur

- US1/P0 — Choisir programme, compte et modèle sur la bonne machine sans fait périmé (C01–C07).
- US2/P0 — Installer, maintenir et reprendre les composants réellement chargés (C08–C13).
- US3/P0 — Déléguer au bon endroit sur la base exacte, sans copie excessive ni conflit de lecture (C14–C19).
- US4/P1 — Reprendre, bifurquer, importer et restaurer un travail avec son contexte (C20–C24).
- US5/P0 — Annuler, réconcilier et comprendre les refus sans assistance du mainteneur (C25–C27).
- US6/P1 — Coordonner messages, questions, secrets et artefacts (C28–C32).
- US7/P1 — Travailler dans une interface intégrée avec navigateur, appareils, code et terminal (C33–C38).
- US8/P1 — Automatiser de façon bornée et mesurer l’usage réel (C39–C43).
- US9/P1 — Lancer et suivre depuis téléphone, web, bureau et iOS, y compris hors ligne (C44–C46).
- US10/P0 — Garder les restrictions et qualifier les prérequis projet (C47–C48).
- US11/P2 — Couvrir les moteurs et plateformes supplémentaires (C49–C50).

Chaque histoire est acceptée lorsque tous ses critères ci-dessous ont une preuve code/test et un scénario utilisateur vérifié. Un manque non confirmé appelle d’abord une mesure ; il n’autorise pas une reconstruction.

## Exigences fonctionnelles et critères d’acceptation

### FR-001 / C01 — Voir le compte utilisé sur la machine

Résultat requis : Afficher sur la machine choisie le programme, la présence de connexion et une identité de compte opaque ; un état inconnu ne devient jamais connecté.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-002 / C02 — Rafraîchir les modèles après une connexion

Résultat requis : Après connexion gérée, démarrage, reconnexion et signal externe observable, invalider les anciens faits et renouveler les modèles ; publier le délai et la limite du cache fournisseur.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-003 / C03 — Relier modèles, compte et serveur

Résultat requis : Chaque modèle annoncé porte sa machine, sa connexion, son programme et sa preuve de compte ; aucun transfert de fait entre deux serveurs.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-004 / C04 — Savoir si la liste est fraîche

Résultat requis : Distinguer date de lecture locale, expiration et fraîcheur réseau non attestée ; cacher toute liste expirée comme preuve actuelle.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Écart des deux côtés ; ce verdict historique ne vaut pas validation221.

### FR-005 / C05 — Options propres à chaque modèle

Résultat requis : Présenter les options réellement déclarées par modèle, leur type, leur valeur par défaut et leurs incompatibilités ; un choix invalide est refusé avant mission.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-006 / C06 — Plusieurs comptes sans écraser le login principal

Résultat requis : Ajouter un compte isolé sans modifier le compte principal ; choisir explicitement le compte et révoquer ses faits lors de sa suppression.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-007 / C07 — Expliquer un modèle refusé

Résultat requis : Distinguer avant départ, lorsque la preuve existe, modèle absent, accès non confirmé et programme incompatible ; après refus, rendre la cause attestée et la prochaine action.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Moments différents, pas d’équivalence ; ce verdict historique ne vaut pas validation221.

### FR-008 / C08 — Préparer toutes les machines depuis un parcours guidé

Résultat requis : Depuis un parcours guidé, préparer une machine puis vérifier programmes, connexion, projet et essai autorisé ; chaque étape reprend après interruption sans recommencer les mutations acquises.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-009 / C09 — Détecter une version de fournisseur incompatible

Résultat requis : Qualifier les opérations réellement utilisées par version de programme ; distinguer minimum testé et version inconnue, sans prétendre qualifier tous les modèles.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-010 / C10 — Mettre à jour fournisseurs, serveurs et application ensemble

Résultat requis : Préparer et contrôler la mise à jour des composants et programmes nécessaires avant bascule ; rendre le composant restant ancien et son geste de rechargement précis.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-011 / C11 — Reconnaître le composant encore ancien

Résultat requis : Afficher versions sur disque et versions réellement chargées de chaque composant ; aucune livraison complète si un composant requis reste ancien.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-012 / C12 — Reprendre après une mise à jour

Résultat requis : Reprendre après une mise à jour avec mêmes tâches, résultats et clés ; perte de connexion, refus, reprise et arrêt sont distingués sans doublon.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Non départagé ; ce verdict historique ne vaut pas validation221.

### FR-013 / C13 — Ajouter un serveur et s’y connecter

Résultat requis : Ajouter un serveur par le mécanisme officiel existant, vérifier son identité et ses droits, puis montrer son état sans manipulation de secrets dans le dialogue.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-014 / C14 — Répartir automatiquement un travail

Résultat requis : Comparer uniquement les cibles admissibles, expliquer les critères du choix et conserver ce choix au rejeu ; une absence de capacité ne déclenche pas un changement silencieux.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun, stratégies différentes ; ce verdict historique ne vaut pas validation221.

### FR-015 / C15 — Déléguer entre machines

Résultat requis : Exécuter la même admission distante jusqu’au résultat et à la clôture, avec propriétaire, clé, base et reçus corrélés ; les coupures n’inventent pas de fin.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun, non départagé ; ce verdict historique ne vaut pas validation221.

### FR-016 / C16 — Regrouper le même projet sur plusieurs machines

Résultat requis : Retrouver un projet sur plusieurs machines, qualifier leurs mappings et corriger le placement futur sans déplacer les tâches actives.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-017 / C17 — Garantir le commit demandé

Résultat requis : Toute préparation prouve le commit exact demandé ; un commit absent est nommé et ne devient jamais une branche récente de substitution.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-018 / C18 — Extraire un dossier léger

Résultat requis : Extraire la sélection autorisée et matérialiser un fichier manquant sans écraser une modification ; mesurer l’espace, auditer le rapport et préserver l’ancre Git.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Profil Bridget documenté ; ce verdict historique ne vaut pas validation221.

### FR-019 / C19 — Lire une base sans bloquer sur un autre dossier

Résultat requis : Lire un commit publié sans changer le dossier partagé ni bloquer sur un worktree descendant indépendant ; les objets absents et les accès refusés restent explicites.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Limite Bridget établie, T3 non départagé ; ce verdict historique ne vaut pas validation221.

### FR-020 / C20 — Changer de fournisseur en conservant le contexte

Résultat requis : Passer à un autre fournisseur sur décision autorisée, avec passation visible et restrictions conservées ; aucune prétendue reprise d’état natif non portable.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-021 / C21 — Créer une bifurcation depuis un tour

Résultat requis : Bifurquer depuis un tour choisi en conservant une filiation et un contexte borné ; les fichiers et effets externes ne sont pas rejoués automatiquement.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-022 / C22 — Revenir à un point de reprise avec les fichiers

Résultat requis : Prévisualiser le retour à un point de reprise conversation/fichiers et protéger les modifications concurrentes ; les effets externes non réversibles sont listés.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-023 / C23 — Importer des conversations existantes

Résultat requis : Importer un historique pris en charge sans exécuter ses outils ni instructions ; préserver provenance, ordre, pièces et trous, avec prévisualisation et déduplication.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-024 / C24 — Continuer une mission après son résultat

Résultat requis : Continuer une mission après résultat via le même enfant lorsque valide ; toute limite de rondes ou durée reçoit une voie de suite explicite et contrôlée.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun avec limite différente ; ce verdict historique ne vaut pas validation221.

### FR-025 / C25 — Annuler et savoir si tout est arrêté

Résultat requis : Confirmer l’arrêt natif des descendants avant libération ; la présence de zombies ou une preuve manquante est visible et réconciliable sans SIGKILL ni fausse clôture.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Non départagé, défaut Bridget constaté ; ce verdict historique ne vaut pas validation221.

### FR-026 / C26 — Rejouer sans créer de doublon

Résultat requis : Toute mutation rejouable conserve clé, canon et issue ; une réponse perdue n’autorise ni duplication ni remplacement automatique.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun, non départagé ; ce verdict historique ne vaut pas validation221.

### FR-027 / C27 — Comprendre une erreur sans appeler le mainteneur

Résultat requis : Chaque refus donne d’abord action utile, portée, cause attestée ou vérification manquante, références utiles et décision humaine si requise ; aucune attente globale induite par un seul refus local.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Progrès Bridget, écart produit restant ; ce verdict historique ne vaut pas validation221.

### FR-028 / C28 — Éditer, ordonner et supprimer la file

Résultat requis : Afficher, corriger le texte, ordonner, promouvoir et supprimer un message encore en file sous les droits existants ; refuser toute édition après remise ou version périmée.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-029 / C29 — Intervenir pendant le travail

Résultat requis : Autoriser une intervention pendant un tour uniquement au parent ou à l’humain habilité ; un pair ordinaire reçoit la voie de communication autorisée sans contournement.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun, droits différents ; ce verdict historique ne vaut pas validation221.

### FR-030 / C30 — Répondre aux questions et permissions

Résultat requis : Présenter les questions et permissions au bon responsable ; appliquer une seule réponse corrélée et rendre les choix devenus périmés.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-031 / C31 — Saisir un secret sans le donner au modèle

Résultat requis : Saisir un secret hors du modèle puis ne transmettre qu’une référence à usage borné ; aucun secret dans messages, journaux, artefacts ou arguments de processus.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-032 / C32 — Joindre et lire des artefacts

Résultat requis : Joindre, prévisualiser et relire les formats pris en charge avec empreinte et droits ; un contenu actif ne prend pas les droits de l’interface.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-033 / C33 — Parcourir le web ensemble sur le serveur

Résultat requis : Ouvrir et parcourir un navigateur partagé sur la machine choisie, avec propriétaire de contrôle visible, observations et artefacts ; ne pas employer une autre session connectée sans autorisation.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Fonction T3 exposée, usage non comparé ; ce verdict historique ne vaut pas validation221.

### FR-034 / C34 — Afficher et contrôler simulateurs et émulateurs

Résultat requis : Afficher les appareils disponibles puis contrôler un simulateur ou émulateur autorisé, avec état, captures et reprise de session ; aucun appareil manquant inventé.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Fonction T3 exposée, usage non comparé ; ce verdict historique ne vaut pas validation221.

### FR-035 / C35 — Revoir les modifications et gérer les PR

Résultat requis : Afficher les modifications, commentaires et contrôles d’une proposition de changement liée au projet ; l’approbation et la fusion gardent leurs droits distincts.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Fonction T3 exposée, usage non comparé ; ce verdict historique ne vaut pas validation221.

### FR-036 / C36 — Être réveillé par un changement de PR

Résultat requis : Recevoir un changement de proposition de code par un abonnement durable, borné et annulable ; pas de boucle de sondage par l’agent ni réveils dupliqués.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 ; ce verdict historique ne vaut pas validation221.

### FR-037 / C37 — Utiliser un terminal lié au projet

Résultat requis : Ouvrir un terminal lié au bon projet et à la bonne machine, reconnectable et borné ; le terminal reste une capacité d’exécution autorisée, pas un texte injecté dans un agent.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Fonction T3 exposée, usage non comparé ; ce verdict historique ne vaut pas validation221.

### FR-038 / C38 — Retrouver et organiser le travail

Résultat requis : Rechercher, classer, épingler et archiver le travail sans perdre les résultats ; des pages bornées séparent présence actuelle et historique.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3 sur l’interface ; ce verdict historique ne vaut pas validation221.

### FR-039 / C39 — Planifier un réveil

Résultat requis : Planifier un réveil ou une tâche dans le fuseau choisi, avec portée et autorité explicites ; conserver le comportement lors de changement d’heure et de redémarrage.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-040 / C40 — Déclencher une tâche par webhook

Résultat requis : Déclencher une action autorisée par un appel signé et limité, avec protection contre rejeu et doublon ; aucun corps HTTP n’est traité comme nouvelle autorité.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Avantage T3, extension de périmètre ; ce verdict historique ne vaut pas validation221.

### FR-041 / C41 — Reprendre après une limite d’usage

Résultat requis : Après limite attestée, attendre la bonne date et reprendre dans les bornes autorisées, même fournisseur et droits ; annulation et pause gardent la priorité.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun, paramètres différents ; ce verdict historique ne vaut pas validation221.

### FR-042 / C42 — Suivre tokens, coûts et offre du compte

Résultat requis : Afficher usage, offre et estimation de coût avec provenance, unité, devise et date ; distinguer mesure, prix configuré et montant réellement facturé.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Présentation T3 documentée ; ce verdict historique ne vaut pas validation221.

### FR-043 / C43 — Observer la présence d’agents de plusieurs projets

Résultat requis : Observer et contacter les agents autorisés de plusieurs projets via un annuaire paginé ; distinguer remis, lu et travail terminé, avec justification interprojet.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Comparaison non établie ; ce verdict historique ne vaut pas validation221.

### FR-044 / C44 — Lancer un nouveau chantier depuis le téléphone

Résultat requis : Créer une mission depuis le téléphone avec choix explicites, précontrôle, clé stable et suivi du résultat ; une perte réseau ne crée pas un deuxième chantier.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-045 / C45 — Proposer une même interface sur web, bureau et iOS

Résultat requis : Rendre les parcours humains essentiels disponibles sur web, bureau et iOS, avec les mêmes droits et reçus ; vérifier accès clavier, mobile et reconnexion.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-046 / C46 — Notifications et messages pendant une coupure

Résultat requis : Conserver brouillons, reçus et notifications pendant une coupure puis réconcilier une seule fois ; une notification ne prouve pas la lecture ni l’application.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Commun partiel ; ce verdict historique ne vaut pas validation221.

### FR-047 / C47 — Conserver les restrictions entre fournisseurs

Résultat requis : Toute conversion de fournisseur conserve ou réduit les droits attestés ; une restriction non traduisible bloque précisément le candidat concerné.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Non départagé ; ce verdict historique ne vaut pas validation221.

### FR-048 / C48 — Mesurer les prérequis du projet sur chaque serveur

Résultat requis : Qualifier les prérequis déclarés du projet sur chaque serveur avant mission, avec version et chemin réels ; absence, désaccord et déterminisme non prouvé restent séparés.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Lacune transversale à traiter ; ce verdict historique ne vaut pas validation221.

### FR-049 / C49 — Prendre en charge d’autres moteurs

Résultat requis : Prendre en charge les moteurs supplémentaires recensés avec leurs opérations et limites explicites ; tester le contrat de chaque adaptateur sans déduire l’accès de sa présence.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

### FR-050 / C50 — Installer le programme sur Windows

Résultat requis : Installer et utiliser une interface Bridget complète sur Windows pour piloter les serveurs Mac/Linux ; qualifier les parcours, la mise à jour et la reconnexion sur un navigateur Windows réel.

Scénario : état initial connu → action autorisée → observation du résultat → refus ou interruption → reprise contrôlée. La preuve inclut le cas nominal et le cas de limite le plus risqué. Référence initiale219 : Parcours T3 documenté ; ce verdict historique ne vaut pas validation221.

## Critères transversaux de succès

- SC-001 :50/50catégories ont une preuve actuelle et un verdict égalité ou gain mesuré ; zéro verdict issu de la seule documentation. Une comparaison non réalisable reste non établie.
- SC-002 :zéro doublon de mission lors des scénarios de réponse perdue, reconnexion et mise à jour ; zéro élargissement de droits ou secret publié.
- SC-003 :tous les refus nouveaux donnent action, portée et référence utile ; un responsable peut choisir la suite sans intervention du mainteneur à partir du message reçu.
- SC-004 :les parcours courants n’ajoutent pas d’étape humaine inutile ; toute régression de temps supérieure à10% sur les mêmes entrées est expliquée et arbitrée avant verdict de supériorité. Les mesures conservent machine, versions et dispersion, au moins5répétitions pour un gain de performance.
- SC-005 :après chaque livraison exécutable validée, octets installés, processus chargés et lien natif sont vérifiés sur les hôtes concernés. Un composant ancien requis garde le statut de livraison partielle jusqu’à son rechargement constaté.
- SC-006 :chaque tâche non terminée reste visible avec sa prochaine action et son blocage réel ; aucun test simulé n’est présenté comme essai de plateforme, de compte ou de modèle réel.

## Cas limites

Compte connecté sans accès modèle ; identités/APIkey non observables ; vieux client ; permission révoquée entre préparation et effet ; cache périmé ; objet Git absent ; lecture shared concurrente ; fichiers modifiés pendant restauration ; secret expiré ou consommé deux fois ; signature ou nonce webhook invalide ; appareil perdu ; navigateur piloté par l’humain ; coupure pendant mise à jour ; processus descendant d’UID différent ; hôte de qualification manquant ; limite d’abonnement ; reprise de tâche dont l’issue est inconnue.

## Entités utiles

Programme et version ; connexion et compte opaque ; modèle et options ; projet et base ; tâche et tentative ; fil/tour et passation ; point de reprise ; reçu et preuve ; permission ; référence de secret ; appareil et session navigateur ; abonnement d’événement ; mesure d’usage ; composant chargé. Les détails techniques et réutilisations appartiennent au plan.

## Dépendances et acquis

219 reste l’inventaire initial daté ;220 apporte déjà une observation Codex sur serveurs provisionnés, sans garantie d’entitlement ni découverte générale Claude. 218 reste un défaut de drainage à intégrer à C25, pas une tâche métier à relancer. Réutiliser153/158/159/168/172/174/177/181/183/194/198/201/207/208/209/211/214/215/217/220 lorsque leurs preuves restent valides. Les acquis ne sont ni réécrits ni automatiquement déclarés équivalents.

## Hypothèses et décisions

L’objectif porte sur toutes les catégories du rapport219, même celles qui n’étaient que documentées chez l’autre solution. Ces catégories doivent être validées comme besoins utiles, sans gonfler leurs garanties supposées. Les adaptations restent officielles et respectent les licences. L’utilisateur reste seul habilité à connecter un compte ou accepter une dépense. Les fonctions d’interface font partie du besoin, pas seulement des commandes techniques équivalentes.

## Clarification utilisateur —30octobre2026

Choix explicite : « Piloter les serveurs Mac/Linux depuis ces plateformes ». Web, bureau, iOS et Windows disposent de l’interface complète ; Windows n’est pas ajouté comme hôte d’exécution natif. La demande conserve les50catégories. L’interprétation intermédiaire client+hôteWindows était plus large que le besoin et est retirée ; aucun code de portageOS n’a été écrit.
