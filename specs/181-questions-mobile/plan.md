# Plan181 — Questions natives Bridget

Statut : livré avec conservations justifiées.25/25tâches ; ConvergeP1/P2/P3 ; auditpass14/14 ; source0.6.2et release publiées. Contre-revue indépendante sans verdict (api_error/délai), réserves et reçus de livraison conservés.

## Architecture et réutilisation

Le contrat central appartient à Bridget. Trois entrées rejoignent le même magasin : outil MCP natif de question humaine (disponible en terminal comme dans les hôtes MCP), callbacks natifs structurés des moteurs gérés, adaptateur facultatif des questions T3. Android et une commande terminal humaine répondent au même identifiant. T3 n’est pas modifié ni redémarré pour ce lot. Les brouillons sont locaux ; aucune synchronisation de saisie T3.

Réutiliser protocol.rs/daemon.rs pour identité de connexion et matrice de rôle ; communication client pour la preuve MCP ; callbacks ChildProviderRequest/ChildRequestAnswer existants ; mobile_questions.rs et les routes HTTP mobiles pour autorisation/reçus ; LocalStore pour brouillon et commandes persistantes ; BridgetMessagingService pour poussée générique/authentifiée ; t3code_contract_v2.rs et le client RPC pour l’adaptateur T3.

Créer un module natif user_input.rs et une table additive de demandes structurées, car human_inbox est limité aux décisions offertes et native_child_requests est strictement lié aux délégations155 et à leurs permissions héritées. Ne pas affaiblir ces contrats pour transporter une réponse libre d’un agent principal. Aucun nouveau daemon, service, file réseau, dépendance ou modèle vocal.

## Contrat et état

Identifiant UUID de demande durable, agent/instance/projet issus de la connexion authentifiée, questions bornées non secrètes, empreinte canonique, origine et callback natif exact. Réponse structurée : sélections et texte pour chaque question ; un seul choix si multiSelect=false. Le complément n’est pas interprété. Canon, commande et réponse sont immuables après consignation.

Transitions : pending -> submitted -> applied ; pending -> resolved_elsewhere/expired/cancelled ; submitted -> unknown/closed si la capacité native disparaît. submitted atteste l’enregistrement, applied nécessite le reçu du producteur. Pas de réponse réinjectée en prompt ordinaire. Rejeu exact rendu depuis reçu ; même clé/autre enveloppe refusée. Compétition de deux réponses traitée en transaction SQLite. Instance/projet/capacité revalidés avant remise.

Conservation7jours ;32demandes ouvertes par agent,2048au total ; corps total64Kio maximum ;64questions/options cumulées bornées ; texte libre10Kio par réponse,32Kio pour l’ensemble. Afficher les refus sans tronquer. Index par agent/état et clé de commande. Coût O(octets+questions/options) pour validation, O(log n) pour idempotence et mutation, lecture bornée.

## Interfaces

Outil MCP bridget_ask_user : création idempotente, état/attente/reprise/annulation propres au producteur. La commande humaine bridget questions permet liste, lecture et réponse sur le même contrat, sans T3. L’agent n’obtient jamais le droit de répondre à sa propre question comme humain.

Les callbacks codex_app_server existants sont réemployés pour les questions d’agents principaux ; les demandes de délégation155 conservent leur routage et leurs permissions. Le transport Claude stream-json reconnaît uniquement AskUserQuestion comme question libre ; ses autres permissions restent refusées selon le contrat actuel. Les TUI opaques sans callback utilisent l’outil MCP Bridget, sans simulation de touches. Le guidage de lancement doit nommer ce chemin compatible.

T3 : lire runtimeRequests et les éléments user_input_request de la projection V2, réemployer la demande native, puis envoyer runtime-request.respond avec commandId stable. Ne pas ajouter de stockage, endpoint ou composant dans T3. Une question terminée sur T3 ferme sa projection Bridget. Une demande non reprenable ne devient jamais répondable. Le protocole T3 reste confiné à cet adaptateur.

Mobile : prolonger les routes/handlers Questions et commandes idempotentes pour les formulaires. Le volet conversation reprend les choix, descriptions et navigation T3, adapté à Compose. Un agrandissement affiche le même brouillon. Champ libre toujours disponible pour les questions de clarification qui l’autorisent ; choix+complément conservés ensemble. Dictée par RecognizerIntent, résultat éditable, annulation sans effet, absence de moteur explicitée. L’envoi reste un geste distinct.

Notification : conserver la poussée FCM sans contenu ; relire les questions via HTTPS authentifié avant rendu privé. Ajouter choix courts et RemoteInput pour une question simple ; ouvrir le bon formulaire pour les demandes complexes. Receiver non exporté, PendingIntent explicite et identité de liaison, question et révision figées ; vérifier l’état au traitement et conserver la commande en cas d’incertitude. Les réponses de notification et de l’écran utilisent la même règle métier. Les heures silencieuses restent appliquées.

## Vérification

Tests Rust : validation, propriété/périmètre, concurrence, rejeu, perte de reçu, reprise daemon, disparition callback, pas de gain de droits155, parseur T3 et commande RPC stable. Tests JVM : choix/simple/multiple/libre+complément, sérialisation/draft, identité et péremption notifications. Tests instrumentés : rotation/fermeture, agrandissement, notification/RemoteInput, dictée (résultat contrôlé et moteur réel si disponible).

Recette native réelle sans T3 : agent/producteur natif contrôlé et commandant terminal, téléphone émulé associé avec un droit borné de recette. Aucun message aux vrais fils. Recette adaptateur T3 contrôlée quand possible, sans arrêter l’application active. Rapports et captures privées ; diaporama versionné. Comparaison indépendante du plan puis du code via Bridget, autre fournisseur.

## Constitution, livraison et maintenance

Code écrit et testé sur Cartae dans le worktree181 ; Mac réservé à l’émulateur et à la compilation/signature Mac. Wrapper Cargo obligatoire. Aucun nouveau SDK vocal ; AndroidSDK officiel installé sur disque externe pour le test. Mode0.6.2 prévu, binaire actif plus retour0.6.1 vérifié. Maintenance native contrôlée existante, tâche historique e595 préservée ; aucune relance T3. Main/forks explicites, commit de preuve avec correction, push vérifié par ls-remote. Nettoyage avec gardes existantes, conservation des usages/brouillons/preuves/autres travaux.

## Fallback des primitives

Skills Speckit disponibles et lues sur Mac. Les scripts setup-plan.sh et check-prerequisites.sh ont été tentés dans le checkout181 : No such file or directory. Les templates projet manquent aussi. Protocole des skills exécuté directement, artefacts complets conservés ; pas de fausse invocation de commande slash. Analyze et Converge seront exécutés selon leur protocole manuel, avec résultats tracés.

## Précisions après Analyze

- La recette sans T3 utilise un daemon/namespace de test sans adaptateur ou connexion T3 ; l’application T3 de l’utilisateur reste préservée ailleurs sur le poste.
- La commande de réponse humaine réutilise le principal bridget-control-cli, jamais une preuve d’agent. La preuve MCP déléguée ne peut créer une question qui contourne le parent155 ; ce refus doit avoir un test.
- Une notification directe n’est offerte que pour une demande simple supportée ; la demande complexe ouvre son formulaire. La liaison appareil, la demande et la révision sont figées à la construction et contrôlées à l’envoi.
- Les anciens clients MCP déjà chargés peuvent conserver leur ancien catalogue. Le binaire installé et les nouveaux lancements seront qualifiés séparément ; pas de redémarrage de T3 pour forcer les clients existants.
