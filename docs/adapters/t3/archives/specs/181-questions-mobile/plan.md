# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/plan.md](../../../../../../specs/181-questions-mobile/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le contrat central appartient à Bridget. Trois entrées rejoignent le même magasin : outil MCP natif de question humaine (disponible en terminal comme dans les hôtes MCP), callbacks natifs structurés des moteurs gérés, adaptateur facultatif des questions T3. Android et une commande terminal humaine répondent au même identifiant. T3 n’est pas modifié ni redémarré pour ce lot. Les brouillons sont locaux ; aucune synchronisation de saisie T3.

<a id="extrait-2"></a>

## Extrait 2

Réutiliser protocol.rs/daemon.rs pour identité de connexion et matrice de rôle ; communication client pour la preuve MCP ; callbacks ChildProviderRequest/ChildRequestAnswer existants ; mobile_questions.rs et les routes HTTP mobiles pour autorisation/reçus ; LocalStore pour brouillon et commandes persistantes ; BridgetMessagingService pour poussée générique/authentifiée ; t3code_contract_v2.rs et le client RPC pour l’adaptateur T3.

<a id="extrait-3"></a>

## Extrait 3

T3 : lire runtimeRequests et les éléments user_input_request de la projection V2, réemployer la demande native, puis envoyer runtime-request.respond avec commandId stable. Ne pas ajouter de stockage, endpoint ou composant dans T3. Une question terminée sur T3 ferme sa projection Bridget. Une demande non reprenable ne devient jamais répondable. Le protocole T3 reste confiné à cet adaptateur.

<a id="extrait-4"></a>

## Extrait 4

Mobile : prolonger les routes/handlers Questions et commandes idempotentes pour les formulaires. Le volet conversation reprend les choix, descriptions et navigation T3, adapté à Compose. Un agrandissement affiche le même brouillon. Champ libre toujours disponible pour les questions de clarification qui l’autorisent ; choix+complément conservés ensemble. Dictée par RecognizerIntent, résultat éditable, annulation sans effet, absence de moteur explicitée. L’envoi reste un geste distinct.

<a id="extrait-5"></a>

## Extrait 5

Tests Rust : validation, propriété/périmètre, concurrence, rejeu, perte de reçu, reprise daemon, disparition callback, pas de gain de droits155, parseur T3 et commande RPC stable. Tests JVM : choix/simple/multiple/libre+complément, sérialisation/draft, identité et péremption notifications. Tests instrumentés : rotation/fermeture, agrandissement, notification/RemoteInput, dictée (résultat contrôlé et moteur réel si disponible).

<a id="extrait-6"></a>

## Extrait 6

Recette native réelle par le parcours natif Bridget : agent/producteur natif contrôlé et commandant terminal, téléphone émulé associé avec un droit borné de recette. Aucun message aux vrais fils. Recette adaptateur T3 contrôlée quand possible, sans arrêter l’application active. Rapports et captures privées ; diaporama versionné. Comparaison indépendante du plan puis du code via Bridget, autre fournisseur.

<a id="extrait-7"></a>

## Extrait 7

Code écrit et testé sur Cartae dans le worktree181 ; Mac réservé à l’émulateur et à la compilation/signature Mac. Wrapper Cargo obligatoire. Aucun nouveau SDK vocal ; AndroidSDK officiel installé sur disque externe pour le test. Mode0.6.2 prévu, binaire actif plus retour0.6.1 vérifié. Maintenance native contrôlée existante, tâche historique e595 préservée ; aucune relance T3. Main/forks explicites, commit de preuve avec correction, push vérifié par ls-remote. Nettoyage avec gardes existantes, conservation des usages/brouillons/preuves/autres travaux.

<a id="extrait-8"></a>

## Extrait 8

- La recette par le parcours natif Bridget utilise un daemon/namespace de test sans adaptateur ou connexion T3 ; l’application T3 de l’utilisateur reste préservée ailleurs sur le poste.
- La commande de réponse humaine réutilise le principal bridget-control-cli, jamais une preuve d’agent. La preuve MCP déléguée ne peut créer une question qui contourne le parent155 ; ce refus doit avoir un test.
- Une notification directe n’est offerte que pour une demande simple supportée ; la demande complexe ouvre son formulaire. La liaison appareil, la demande et la révision sont figées à la construction et contrôlées à l’envoi.
- Les anciens clients MCP déjà chargés peuvent conserver leur ancien catalogue. Le binaire installé et les nouveaux lancements seront qualifiés séparément ; pas de redémarrage de T3 pour forcer les clients existants.
