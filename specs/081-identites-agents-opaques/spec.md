# SPEC-081 - Identités d'agents opaques

## Contexte

Bridget utilise encore un même champ name pour l'adressage technique, l'identité durable et le nom affiché. Cette confusion fait fuiter les noms de routage dans les prompts, les messages, Maicie et certaines vues. Le produit est en pré-production : une migration cassante et atomique est explicitement acceptée.

## Décision produit

Pour chaque agent, il ne reste que deux identifiants :

- agent_id : identifiant opaque, stable, unique, employé par les contrats, le routage et les stockages. Il n'est pas montré dans les surfaces utilisateur.
- display_name : nom humain modifiable, unique parmi les agents actifs, employé dans les vues, notifications, transcripts et prompts.

Le nom technique historique disparaît du modèle vivant. Il peut être lu une seule fois par la migration pour préserver les liens, mais il n'est plus exposé, accepté par les APIs ni maintenu comme alias de routage.

Les principaux non agents restent distincts : human, system:bridget, system:maicie et system:resume.

## Objectifs

1. Le daemon route exclusivement avec agent_id.
2. Les agents et leurs interlocuteurs sont désignés par display_name dans les prompts.
3. Bridget Desktop et le relais web n'affichent que display_name pour les agents.
4. Maicie persiste et cible des agent_id. Toute référence à un agent supprimé devient requires_retarget sans être effacée ni livrée à un autre agent.
5. Une migration unique transforme Bridget et Maicie, puis refuse le contrat historique.

## Hors périmètre

- Personae et bibliothèques de personnalités.
- UI complète de création et suppression d'agent.
- Compatibilité d'un ancien wrapper ou client après migration.
- Secretisation de agent_id : il est opaque, pas secret.

## Histoires utilisateur

### US1 - Nom visible cohérent - P1

En tant qu'utilisateur, je peux modifier le nom affiché d'un agent et ne voir que ce nom dans les listes, conversations, recherche, notifications et réglages.

Critères :
- Modifier display_name ne modifie ni agent_id, ni délégation, ni message, ni exécution.
- Aucun nom de routage historique ou agent_id n'apparaît dans une surface utilisateur normale.
- Les prompts injectés utilisent les display_name des agents.

### US2 - Routage durable par agent_id - P1

En tant qu'opérateur, je peux relancer un agent sans dépendre d'un nom modifiable.

Critères :
- Handshake wrapper, message, annuaire, présence, flotte, exécution et cycle de vie portent agent_id.
- Un routage vers un ancien nom est refusé explicitement.
- Les agents actifs avant migration sont relancés avec leur agent_id. Un agent explicitement stopped reste arrêté.

### US3 - Maicie conservée et sûre - P1

En tant qu'utilisateur, je ne perds pas une mission Maicie parce qu'un agent a été supprimé ou recréé.

Critères :
- Les profils Maicie lient agent_id et conservent display_name pour l'affichage.
- Chaque référence connue est convertie atomiquement ou marquée requires_retarget avec raison.
- L'outbox ne livre jamais une délégation à retargeter.
- Une commande de rapport expose les références qui nécessitent un retarget.

### US4 - Migration contrôlée - P1

En tant qu'opérateur, je peux exécuter un préflight puis une migration sans deviner les données touchées.

Critères :
- Le préflight énumère identités, messages, exécutions, flotte et références Maicie convertis ou mis en attente.
- L'exécution est idempotente, journalisée et protégée par sauvegarde SQLite.
- Une ambiguïté ou incohérence bloque avant toute écriture partielle.

## Exigences

- FR-8101 : les contrats v2 exposent agent_id et display_name, jamais name, profile_ref ni routing_name.
- FR-8102 : display_name est la seule identité humaine dans prompts, UI et notifications.
- FR-8103 : les messages conservent from/to exclusivement comme principaux agent_id ou système documentés.
- FR-8104 : le routeur ne résout aucun alias historique.
- FR-8105 : les stores de flotte et d'exécution utilisent agent_id.
- FR-8106 : Maicie persiste agent_id pour profils, délégations, routines et outbox, avec requires_retarget.
- FR-8107 : la migration convertit Bridget puis Maicie avec préflight, sauvegarde et transaction par base.
- FR-8108 : les tables et colonnes d'alias sont supprimées ou inaccessibles après migration.
- FR-8109 : la recréation donne un nouvel agent_id et ne réattribue aucune ancienne tâche.
- FR-8110 : les endpoints profil utilisent agent_id dans leur chemin et display_name au rendu.
- FR-8111 : les tests couvrent renommage, livraison, suppression/recréation, migration de délégation et outbox orpheline.

## Non fonctionnel

- Atomicité : aucune migration partielle dans une base SQLite.
- Observabilité : journal versionné et compteurs convertis, retargetés, refusés.
- Sécurité : un identifiant opaque ne remplace pas les autorisations existantes.
- Réversibilité : sauvegarde privée obligatoire avant opération destructive.
- Qualité : aucun alias de compatibilité durable.

## Dépendances

- SPEC-078 : profils, étiquettes, instructions, notifications.
- SPEC-080 : contrôle central et projection UI.
- Maicie : délégations, routines et outbox.
