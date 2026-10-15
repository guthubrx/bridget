# Quickstart de validation SPEC168

Statut : procédure prévue, pas recette déjà passée. Exécuter après implémentation et autorisation de livraison.
Aucun nom de modèle dans les exemples ne garantit son accès sur un compte donné.

## 1. Préparer les capacités sans changer les comptes

Sur chaque machine, relever le binaire et ses capacités réelles.
Déclarer une connexion par environnement déjà authentifié, pas un profil par modèle.
Ne copier aucun credential. Le deuxième environnement Codex Cartae absent reste inactif
tant qu’une authentification native autorisée n’a pas été réalisée hors Bridget.
Claude Cartae déjà authentifié selon ROOT ne demande pas un nouveau login.

Lire le catalogue horsT3 : bridget delegate --catalogue --json ; utiliser --machine pour filtrer.
Sans --json, vérifier le même inventaire lisible depuis un terminal humain sans identité d’agent.
La garde utilise sameUIDUnix/Role::Client/version négociée, pas le bindingT3 de human_authority_guard.
Cette commande ne crée aucune Task et ne lance aucun modèle pour chaque entrée.
Comparer modèle, efforts et machine. Conserver la distinction « exposé » / « lancement effectivement réussi ».
Vérifier les options de sûreté du petit décideur sur Claude Mac2.1.300 et Cartae2.1.251.
Utiliser --safe-mode avec --restricted et outils/MCP désactivés ; --bare est interdit pour OAuth.
Prouver --json-schema en print/stream-json réel ; stdout borné, stderr non conservé.
Les aides n’attestent pas que tous leurs efforts sont acceptés par chaque modèle : défaut compatible
obligatoire, ou null/liste vide pour défaut natif sans flag. Ne pas inventer medium.
Un flag absent, un réglage administré incompatible ou un compte indisponible désactive ce niveau explicitement.

## 2. Configurer la chaîne et les contraintes

Dans le registre privé déjà utilisé, activer la section168 versionnée pour les machines voulues.
Conserver les valeurs de configuration hors dépôt ; les exemples utilisent seulement des IDs opaques.
Autoriser service et petit modèle séparément, avec adresse/accessibilité propres à la machine du coordinateur.
Choisir le petit modèle réel et sa connexion autorisée ; ne pas imposer une nouvelle cléAPI.
Configurer les cinq axes, les seuils et le budget total. telemetry.weights de la machine coordinatrice
fixe uniformément les poids cpu/ram/load/continuity/compilation/quota/reset pour toutes les cibles :
défauts et maxima20/20/20/20/10/5/3, entiers non négatifs. Les termes CPU/RAM/charge sont arrondis
au plus proche ; la pénalité100 de saturation reste fixe. Seuils et fraîcheur restent propres à chaque cible.
Sans politique168 active, le daemon sert encore les anciennes enveloppes163, mais refuse
explicitement une nouvelle demande automatiqueV2 ; aucun retour silencieux versV1.
Le nouveau client automatique CLI/MCP émet toujoursV2, sans flag de version ni choix selon ON/OFF.
workload=general reste le type de travail par défaut, pas un commutateur de protocole.
Le chemin explicite bridget_delegate/NativeDelegate et le wire AutoDelegateV1 restent compatibles
au titre deFR-034. Un toggle ne change donc pas la version du canon d’un même request_id.

Préparer le SHA exact du projet sur les machines autorisées à exécuter du code.
Un coordinateur peut effectuer un fetch explicitement autorisé ; aucun fetch du daemon.
Pour Shared Git comme Worktree, fournir AutoDelegateV2.expected_base_commit égal à ce SHA ;
Worktree.base_ref doit résoudre le même SHA. Sans base attendue : workspace_base_required avant modèle.
La CLI delegate lit le HEAD courant de la source et le fige dans expected_base_commit avant
création de l’enveloppe ; jamais de HEAD distant implicite. Un nouveau commit source change donc
le canon : la même --request-id reçoit envelope_mismatch. Pour retrouver la demande précédente,
utiliser bridget task status avec son identifiant, sans soumettre une nouvelle délégation.
Shared horsGit avec workload=general peut omettre la base. workload=general|compilation est
un champV2 séparé du briefHandoffV1, général par défaut, sans classification par mots-clés.

## 3. Vérifier par les façades existantes

Flags vérifiés dans le code :
- Catalogue : bridget delegate --catalogue [--machine ID] [--json].
- Mission : bridget delegate MISSION [--machine-id ID] [--engine MOTEUR]
  [--connection-id ID] [--model MODELE] [--effort EFFORT] [--workload general|compilation]
  [--project-id ID] [--workspace JSON] [--request-id ID] [--cwd CHEMIN] [--json].
- Suivi : bridget task status TASK_ID --json ; attente : bridget task wait TASK_ID
  --timeout 120 --json ; arrêt : bridget task cancel TASK_ID --json.

--machine filtre la lecture ; --machine-id impose la cible d’une mission.
delegate fige le SHA lui-même et n’accepte pas --expected-base-commit.
Le MCP et auto-delegate acceptent expected_base_commit / --expected-base-commit.
Aucun modèle ou effort non déclaré, aucune base distante implicite.

Scénarios ciblés :
1. Lire au moins douze options complètes, chacune avec ses efforts et défaut résoluble ; ni coupe à huit
   ni profil par modèle. Le registre129refuse globalement ; aucun catalogue partiel ne le masque.
2. Fixer seulement le modèle, puis seulement la machine ; vérifier les autres axes automatiques.
3. Imposer une machine saturée : refus, aucune bascule cachée. Avec machine libre alternative,
   demander à un faux décideur de choisir la machine saturée : son ID retiré est refusé.
4. Présenter quotas restants/reset connus proches : bonus borné observable, inconnus neutres et comptes séparés.
5. Lancer deux sous-tâches indépendantes sur deux worktrees, avec base exacte commune vérifiée.
6. Couper le demandeur puis rejouer même clé/enveloppe, avec le même SHA source ; conserver choix
   et unique Task. Après un commit source, consulter bridget task status plutôt que redéléguer.
7. Après choix durable, rendre sa cible indisponible ou saturée : la revalidation peut refuser
   le lancement ; elle conserve le choix, sans autre machine ni nouvel appel de décision.

## 4. Prouver les trois niveaux en recette réelle bornée

Préparer un namespace de recette isolé sans serveur, pont ni configurationT3.
Ne pas arrêter ou redémarrer le T3 de l’utilisateur.
Limiter les appels modèles aux recettes nécessaires, sans essayer cent fois chaque modèle.

- Service décisionON : prouver une sélection réelle sur un ensemble entier qui tient dans ses
  limites attestées (≤8options par prudence initiale), avec tous les faits FR021 et réponse fermée ;
  santé seule insuffisante. Liste12+qui ne tient pas = indisponibilité de capacité explicite, aucune réduction.
- ServiceOFF, petit modèleON : prouver une sélection réelle Haiku léger sur12+options entières,
  avec OAuth déjà authentifié, sans outil ni nouvelle Task décideur.
- Deux niveauxOFF : prouver le score déterministe sur12+options entières, zéro appel de décision.
- Pour chaque résultat distant, vérifier l’hôte cartae et le modèle/connexion/effort effectivement retenus.
- Rejouer la même enveloppe et redémarrer la recette : zéro nouvel appel de décision pour la clé
  déjà choisie. Le rejeu CLI suppose le HEAD source inchangé ; sinon lire bridget task status.

Si un niveau n’est pas qualifié, le laisser désactivé et ne pas le présenter comme livré.
La livraison complète exige sa recette prévue ou un blocage explicite ; le score seul n’est pas le périmètre complet168.

## 5. Livrer et nettoyer

Une campagne workspace finale unique après stabilisation ; uniquement les replays affectés ensuite.
Un seul compilateur Cargo sur Cartae, jobs2 et deux threads de tests ; aucun build concurrent.
Conserver les échecs hérités séparément. Ne pas annoncer un GREEN global depuis des tests ciblés.

ROOT sauvegarde bases cohérentes, registre, unités/plists et binaires avant activation.
Intégrer/pousser les commits, vérifier les SHA sur les machines et installer les paquets utiles.
Récupérer les travaux distants et vérifier l’absence de processus avant retrait des worktrees/branches.
Préserver tout dossier sale/divergent, DB, preuve, rollback et travail étranger.
Version168 compatible ajoutant une fonction : prévoir0.4.0 si la source reste0.3.0.
ROOT ne modifie version/changelog qu’en livraison, puis construit les paquets depuis ce commit exact,
avec tag et release cohérents. Installer utilement Mac/Cartae, préserverT3.
Toute lignev2 interdit le retour à l’ancien binaire même168OFF : garder compatible+OFF
ou restaurer un ensemble cohérent sans perte.
Le reçu de livraison distingue source publiée, binaire actif, configuration activée et faits de recette.


## Configuration et preuves à conserver

routing_policy version2 contient connections, model_catalog et machine_policies dans le
registre privé du namespace BRIDGET_HOME. Les cinq axes sont automatic_axes/fixed_values.
Chaque machine configure séparément decision_service et lightweight_decision avec enabled
et accessible. Le service vise service_id dans decision_services, avec endpoint et model.
Le petit décideur utilise une connexion Claude locale déjà authentifiée ; effort de rôle
absent/null tant que sa compatibilité n’est pas qualifiée. Aucun login ou secret copié.
decision_timeout_ms borne la chaîne à1–30s, service_timeout_ms a un plafond effectif2s ;
le même Instant couvre attente, préparation et lancement, sans budget renouvelé au spawn.

Le reçu final doit distinguer code publié, binaire actif, configuration et recettes réellement
passées. Ce quickstart reste une procédure prévue tant que ces preuves manquent, sans promettre
FULL, activation ou production. Configuration exacte et livraison Git entre machines :
/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/delegation-native.md.
