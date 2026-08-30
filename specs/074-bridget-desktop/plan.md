# Plan d'implémentation - SPEC-074 Bridget Desktop

## Résumé technique

Créer le premier client Bridget séparé dans `apps/bridget-desktop`. Il conserve le modèle de sécurité actuel : les relais restent en boucle locale sur chaque serveur. Le client macOS ouvre et supervise les tunnels SSH, puis rend l'UI existante de chaque relais dans un panneau sans lui donner de privilèges Tauri.

Le développement des sources et des tests serveur reste sur cartae.app. La génération et la validation du paquet `.app` sont une étape cible macOS, car le serveur Linux ne possède ni WebKit ni Xcode.

## Décisions de planification

1. Réutiliser le client SSH système et les identités déjà gérées par macOS. Le frontend ne reçoit jamais la capacité de lancer une commande arbitraire. Une commande Rust interne construit un appel SSH à partir d'un profil validé.
2. Ajouter au daemon une commande de découverte d'endpoint UI strictement locale, typée et non journalisante. Elle remplace la lecture brute du fichier interne contenant le jeton.
3. Réutiliser le relais UI, son API HTTP, ses flux SSE, ses jetons et l'interface actuelle. Ne pas cloner l'interface de conversation dans Bridget Desktop.
4. Isoler la coque locale Bridget Desktop du contenu distant : la coque gère les profils, les tunnels et les panneaux ; les vues chargées depuis un tunnel n'obtiennent aucune capacité Tauri. Les deux panneaux côte à côte utilisent les webviews enfants Tauri derrière son feature `unstable`, car c'est l'API qui permet de distinguer leur label de celui de la coque et donc leurs permissions.
5. Prévoir deux types de profil concrets, `ssh` et `local`, sans créer une abstraction de transport plus vaste avant d'avoir une troisième utilisation réelle.
6. Limiter la première vue simultanée à deux panneaux. Chaque panneau a une session, un tunnel et une origine explicites.
7. Documenter le navigateur distant comme capacité future par profil, mais ne créer ni tunnel générique, ni VNC, ni navigateur dans ce lot.

## Frontières de propriété

| Zone | Responsabilité SPEC-074 | Réutilisation / exclusion |
|---|---|---|
| `apps/bridget-desktop` | propriétaire du client macOS, profils, tunnel, panneaux et diagnostics | nouveau périmètre client séparé |
| `crates/bridget-daemon/src/cli.rs` | étend une découverte d'endpoint UI minimale | réutilise UI endpoint persistant |
| `crates/bridget-daemon/src/ui.rs` | fournit le contrat endpoint sans changer les routes UI | réutilise relais, jeton et boucle locale |
| `crates/bridget-daemon/assets/ui` | consommé tel quel à travers le tunnel | ne pas copier ni modifier pour Desktop |
| SSH fédéré SPEC-002 | référence de robustesse SSH | ne pas modifier sa fédération inverse d'agents |
| SPEC-063 à SPEC-073 | consommées comme comportements du relais | ne pas modifier leurs fichiers ou leur logique métier |
| navigateur distant | hors périmètre | décision future séparée |

## Phases

### P1 - Contrat serveur de découverte UI

1. Localiser le contrat `UiEndpoint` existant et rendre une commande de lecture explicite accessible uniquement dans le contexte SSH déjà authentifié.
2. Retourner uniquement les données nécessaires à l'ouverture du tunnel et ne rien écrire dans les logs, diagnostics ou erreurs.
3. Ajouter les tests de syntaxe, d'absence de secret dans les erreurs et de compatibilité de version du contrat.

### P2 - Socle Bridget Desktop et profils non secrets

1. Créer `apps/bridget-desktop` avec la configuration Tauri minimale et un paquet frontend local.
2. Définir les profils `ssh` et `local`, persister seulement les métadonnées non secrètes et migrer proprement une configuration vide ou ancienne.
3. Construire les formulaires accessibles d'ajout, édition, retrait et sélection de profil.
4. Rendre visible le serveur sélectionné avant tout panneau de travail.

### P3 - Connexion SSH contrôlée

1. Construire et valider les arguments SSH dans le backend Rust, sans shell libre ni interpolation d'entrée utilisateur.
2. Réutiliser `ssh-agent` ou un chemin de clé explicitement choisi, sans copier son contenu.
3. Demander une approbation explicite de l'empreinte à la première connexion, persister l'approbation non secrète et bloquer tout changement ultérieur.
4. Superviser l'enfant SSH, la fermeture, la reconnexion bornée et les erreurs catégorisées.

### P4 - Relais et panneaux

1. Établir le tunnel local vers le relais distant après découverte de son endpoint.
2. Charger l'UI existante dans une webview enfant externe, sans capacité locale, et exposer les états séparés : SSH, tunnel, relais et flux.
3. Créer les onglets de profils et un mode deux panneaux. Garantir l'origine persistante dans chaque panneau et la séparation des notifications.
4. Connecter un profil local directement au relais local avec les mêmes états, sans SSH.

### P5 - Qualité, sûreté et livraison

1. Tester les profils, la validation d'arguments, les erreurs SSH, le contrat d'endpoint, la séparation d'origine et l'absence de secrets.
2. Ajouter une checklist de diagnostic redacted et les parcours opérateur manuel.
3. Construire le binaire serveur sur cartae.app et le paquet macOS sur le Mac, sans déployer ni redémarrer Bridget serveur pour le client seul.
4. Vérifier l'accessibilité clavier et le comportement à deux panneaux sur macOS.

## Stratégie de tests

- Tests Rust du daemon avant l'extension de commande endpoint.
- Tests Rust du backend Tauri avec exécutable SSH factice : arguments autorisés, empreinte, annulation, reconnexion, nettoyage de processus enfant.
- Tests frontend sur formulaires, focus, états de connexion, séparation d'origine et panneaux.
- Test d'intégration avec un relais HTTP factice puis avec un Bridget réel lié à 127.0.0.1.
- Test manuel macOS : clé via agent, première empreinte, refus d'empreinte changée, coupure réseau, reconnexion, double panneau et profil local absent.
- Contrôles de secret : recherche sur les artefacts, journaux de test et configuration persistée.

## Sécurité et exploitation

- La capacité Tauri de la coque locale vise exclusivement son label. Chaque webview enfant `panel-*` n'a aucune permission Tauri. Les commandes système sont détenues par un backend étroit et typé.
- Les clés privées et jetons restent hors Git, hors logs et hors stockage de profils. Les diagnostics remplacent les valeurs sensibles par une information de présence ou d'état.
- Chaque tunnel est lié à la boucle locale du serveur configuré et est fermé avec sa session.
- Les erreurs de version et d'empreinte sont bloquantes, pas des avertissements contournables automatiquement.
- Le déploiement du client est indépendant du daemon. Une future mise à jour automatique devra être signée avant activation.

## Critères de sortie

- Bridget Desktop se connecte à cartae.app via SSH sans ouvrir le port UI du serveur.
- Un profil local et un profil distant sont distingués dans la vue et dans les erreurs.
- Deux serveurs peuvent être ouverts sans confusion de messages, agents ou notification.
- Les tests ne révèlent ni clé privée ni jeton d'endpoint.
- Le paquet macOS peut être construit et l'ensemble des parcours manuels critiques est vérifié.
