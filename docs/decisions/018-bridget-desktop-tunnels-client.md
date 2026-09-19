# ADR-018 - Bridget Desktop possède les tunnels client

**Statut** : Accepté
**Date** : 2026-08-30

## Contexte

Le relais Bridget UI actuel est local au serveur et son exposition publique contredirait son modèle de sécurité. L'opérateur veut une application macOS capable de piloter plusieurs serveurs ainsi qu'un relais local éventuel, tout en gardant les agents à leur emplacement d'exécution.

## Décision

Créer un client séparé nommé Bridget Desktop dans `apps/bridget-desktop`. Il possède les profils, ouvre les tunnels SSH depuis le Mac, et charge les relais distants à travers ces tunnels. Les relais serveur restent liés à la boucle locale.

Le client utilise le programme SSH système via un backend local étroit. Le contenu provenant d'un relais ne reçoit aucune permission Tauri de lancer des commandes, lire des fichiers ou accéder aux clés. Les identités SSH restent gérées par le système et ne sont pas recopiées dans la configuration de l'application.

Les deux panneaux demandés sont rendus dans deux webviews enfants Tauri, chacune chargée avec une URL externe de boucle locale et un label `panel-*`. La capability de la coque locale ne cible jamais ces labels. Cette API nécessite actuellement le feature Cargo `unstable`; son coût est explicitement limité à l'agencement des panneaux, sans conséquence sur le protocole de tunnel ou les données de profil.

Un profil local se connecte directement à son relais et n'ouvre pas de tunnel. Chaque panneau reste attaché à une origine ; la première version autorise deux panneaux simultanés.

La visualisation et le contrôle d'un navigateur serveur sont exclus. La future architecture devra les rattacher à une session d'exécution isolée, avec arbitrage explicite entre contrôle agent et humain.

Cette future session utilisera un second tunnel SSH de boucle locale, séparé du tunnel de l'UI Bridget et rattaché au seul profil sélectionné. Elle ne réutilisera ni cookies, ni contexte navigateur, ni droits de contrôle d'une autre exécution. Le navigateur continuera de s'exécuter sur le serveur, et tout transfert de contrôle agent/humain sera explicite et durable.

## Conséquences

### Positives

- Aucun port UI ou navigateur supplémentaire n'est publié sur Internet.
- Le modèle existant SSH, agent SSH et hôtes connus reste réutilisable.
- Le client peut gérer plusieurs serveurs sans fusionner leurs données métier.
- Une compromission de contenu distant est contenue hors des privilèges locaux de l'application.

### Négatives

- Le client doit superviser les processus SSH, la reconnexion et la durée de vie des tunnels.
- La génération et la validation du paquet macOS exigent un Mac, pas seulement le serveur Linux.
- Le jeton UI actuel traverse temporairement la mémoire du client ; les logs et le stockage doivent l'exclure strictement.
