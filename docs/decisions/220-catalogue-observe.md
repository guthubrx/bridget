# 220 — observation locale des modèles, distincte de la politique

Date : 30 octobre 2026. Statut : accepté dans la session220.

## Besoin

Une déclaration statique de modèle ne dit pas quel compte est connecté sur un serveur. Après une connexion officielle différente, une liste ancienne peut induire l’agent en erreur. La découverte ne doit pas lancer une mission ni conférer de droits supplémentaires.

## Décision

Le worker tient l’observation locale Codex. Il utilise le programme et l’environnement de sa définition, sans chemin fourni par le lecteur. Il initialise un processus neuf sous l’identité Bridget puis lit le compte, les modèles paginés et le compte une seconde fois. Aucun fil, tour, login/logout ou outil métier n’est autorisé. Les bornes et le créneau unique sont indépendants de la file des missions.

Le bootstrap géré existant porte la preuve durable avant RELEASE. Une fin incertaine conserve cette preuve et l’autorité sur le processus. Une ancienne preuve ne permet aucun signal sans identité de naissance correspondante. Aucun SIGKILL.

Le maître accepte seulement une observation négociée de la connexion worker et de l’époque courantes, liée au dernier refresh_id et à l’empreinte de définition. La date et le TTL limitent sa visibilité. Le catalogue déclaré reste la politique d’admission ; la liste annoncée est un fait séparé. Aucun profil ni reçu de tâche n’est réécrit.

Un nonce public aléatoire représente localement l’observation du compte. Sa comparaison privée est salée par worker et ne sort jamais. Les métadonnées de fichiers auth/config déclenchent une invalidation, pas une preuve d’identité. Le contenu des fichiers de secrets n’est pas lu par Bridget. Une actualisation explicite est disponible sans login ni mission.

## Conséquences et limites

La lecture du programme peut toujours provenir de son cache. Ni la fenêtre d’attente de trois secondes ni un processus neuf ne prouvent sa fraîcheur réseau. L’accès et l’abonnement restent inconnus avant une exécution réelle. Les connexions désactivées restent désactivées. Les autres adaptateurs ne sont pas prétendus pris en charge. Le choix humain de modèle existant et les restrictions de l’admission restent inchangés.

Le worker est propriétaire de la fraîcheur et de la supervision. Le maître est propriétaire de la corrélation et de la projection. Le client MCP affiche les faits ; sa fréquence de lecture ne provoque aucune sonde. Un ancien client peut afficher les données additives mais doit recharger son connecteur pour obtenir le nouvel outil. Le contrat fournisseur et les schémas devront être revérifiés lors de leur changement.
