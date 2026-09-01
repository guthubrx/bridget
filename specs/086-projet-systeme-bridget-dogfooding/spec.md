# Feature Specification: Projet système Bridget et dogfooding expert

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 086-projet-systeme-bridget-dogfooding
Titre: Projet système Bridget et dogfooding expert
Statut: Ready for implementation
Priorité: P1
Tâches: 0/33 (0%)

Résumé:
- Contexte: les agents peuvent développer des projets isolés, mais le checkout Bridget ne doit être modifiable que dans un projet système explicitement activé par un opérateur expert.
- Objectif: créer une autorité unique par installation, bornée à Bridget, avec lecture seule par défaut et écriture partagée après avertissement explicite.
- Dépendances: SPEC-065, SPEC-066, SPEC-067, SPEC-080, SPEC-084, SPEC-085

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-086-projet-systeme-bridget-dogfooding`
**Created**: 2026-08-31
**Status**: Ready for implementation
**Priority**: P1
**Dependencies**: SPEC-065, SPEC-066, SPEC-067, SPEC-080, SPEC-084, SPEC-085

## Contexte

Le projet Bridget est particulier: son checkout contient le code du daemon, du Desktop et du plugin Maicie. Il doit pouvoir être développé avec les mêmes agents et les mêmes worktrees que les autres projets, sans donner à tous les projets le droit de modifier le système qui les exécute. Le contrôle ne doit pas reposer sur une liste pénible d'agents autorisés ni sur une permission temporaire. L'opérateur expert active ou désactive une capacité au niveau d'un unique projet système par installation.

Le dogfooding autorise l'édition, les tests et les commits dans des branches/worktrees. Il n'autorise jamais automatiquement le merge vers `main`, l'installation du binaire, le redémarrage du daemon ou le déploiement.

## Objectifs

- Déclarer au plus un projet système Bridget par serveur/install.
- Réserver le checkout Bridget et ses worktrees à ce projet.
- Fournir un réglage expert désactivé par défaut avec avertissement clair.
- Appliquer une permission homogène à tous les agents du projet système.
- Conserver le checkout de référence en lecture seule et rendre uniquement le worktree attribué modifiable lorsque le mode est activé.
- Recréer proprement l'environnement après un changement de mode.
- Permettre la coexistence avec des agents externes travaillant dans d'autres worktrees.

## Hors périmètre

- Une permission par agent, par mission ou limitée dans le temps.
- L'auto-amélioration autonome ou permanente.
- Le merge automatique, le push, l'installation, le redémarrage ou le déploiement de Bridget.
- L'accès au checkout Bridget depuis un projet standard.
- Plusieurs projets système Bridget sur un même serveur.
- La découverte automatique d'un checkout système.
- Le remplacement de Git/worktree comme mécanisme de concurrence.

## User Story 1 - Enregistrer le projet système - P1

Comme exploitant, je veux déclarer le checkout Bridget comme projet système unique afin que sa nature et ses droits ne puissent être confondus avec ceux d'un projet standard.

### Scénarios d'acceptation

1. L'installation peut déclarer un chemin source absolu explicite et l'enregistrer comme emplacement `exact_project` et `system_only`.
2. S'il n'existe pas ou n'est pas un checkout Bridget attesté, le projet système reste indisponible.
3. Un projet standard ne peut pas être promu projet système depuis un menu ordinaire.
4. Un second projet système est refusé sans modifier le premier.
5. Le checkout système n'apparaît jamais comme parent de création d'un projet standard.

## User Story 2 - Activer le dogfooding expert - P1

Comme opérateur expert, je veux activer ou désactiver le dogfooding avec un avertissement explicite afin de choisir si les agents du projet système peuvent modifier Bridget.

### Scénarios d'acceptation

1. Le réglage est désactivé par défaut et décrit les droits supplémentaires ainsi que les actions toujours interdites.
2. L'activation nécessite une confirmation explicite portant la génération courante du projet système.
3. Tous les agents du projet système obtiennent la même capacité de travailler dans un worktree attribué en écriture après recréation confirmée de l'environnement.
4. Aucun agent d'un autre projet ne reçoit un montage du checkout Bridget.
5. La désactivation restaure des montages en lecture seule sans annuler ni supprimer l'historique Git.

## User Story 3 - Coexister avec les développements externes - P1

Comme développeur, je veux utiliser des branches et worktrees distincts depuis l'intérieur ou l'extérieur de Bridget afin que les travaux concurrents restent compatibles.

### Scénarios d'acceptation

1. Le dépôt et tous ses worktrees liés sont montés aux mêmes chemins absolus dans le conteneur du projet système.
2. Un worktree créé par un agent externe est visible après réconciliation sans copier le dépôt.
3. Deux agents travaillant dans des worktrees différents n'écrasent pas leurs index ou HEAD.
4. Bridget n'autorise pas deux propriétaires à modifier le même worktree et signale le conflit.
5. Les agents peuvent construire et tester dans le conteneur, mais la livraison sur l'hôte reste une étape séparée et explicite.

## Exigences fonctionnelles

- FR-08601: Une installation Bridget DOIT accepter au plus un projet de rôle `bridget_system`.
- FR-08602: Le projet système DOIT provenir d'un chemin source absolu configuré par l'exploitant, jamais d'une découverte générale.
- FR-08603: Le chemin DOIT être attesté comme checkout Bridget avant l'enregistrement.
- FR-08604: Son emplacement DOIT être `exact_project`, `system_only` et interdit comme parent de création.
- FR-08605: Un projet standard NE DOIT pouvoir être promu, cloné ou remplacé en projet système par une route projet ordinaire.
- FR-08606: Le centre de contrôle DOIT exposer un réglage expert `dogfooding.bridget` avec les valeurs fermées `disabled` et `enabled`.
- FR-08607: La valeur par défaut et la valeur de migration DOIVENT être `disabled`.
- FR-08608: L'activation DOIT présenter un avertissement explicite et nécessiter une confirmation liée à la génération du projet système.
- FR-08609: La permission DOIT s'appliquer à tous les agents du projet système, sans rôle d'agent, durée ou mission supplémentaire.
- FR-08610: En mode `disabled`, checkout Bridget, métadonnées Git et worktrees DOIVENT être montés en lecture seule dans l'environnement du projet système.
- FR-08611: En mode `enabled`, le checkout de référence DOIT rester en lecture seule; le worktree attribué au travail et les métadonnées Git nécessaires aux commits PEUVENT être montés en écriture dans le seul environnement du projet système.
- FR-08612: Aucun projet standard NE DOIT recevoir le checkout Bridget, son répertoire Git commun ou ses worktrees comme montage.
- FR-08613: Le changement de mode DOIT être refusé lorsqu'un agent du projet système est actif.
- FR-08614: Le changement confirmé DOIT incrémenter l'epoch d'environnement et recréer le conteneur avant de publier le nouveau mode.
- FR-08615: Un échec de recréation DOIT conserver l'ancien mode confirmé et ne produire aucun fallback Host silencieux.
- FR-08616: Le projet système DOIT utiliser le backend Docker de SPEC-085 pour appliquer la frontière de montage aux agents.
- FR-08617: Les chemins du checkout, du répertoire Git commun et des worktrees DOIVENT être identiques sur Host et dans le conteneur.
- FR-08618: Les worktrees existants DOIVENT rester compatibles avec les agents externes et aucun dépôt ne doit être copié dans un volume opaque.
- FR-08619: Le système DOIT détecter et refuser l'attribution simultanée d'un même worktree à plusieurs travaux actifs.
- FR-08620: Le dogfooding PEUT autoriser lecture, édition, tests, builds et commits sur la branche du worktree.
- FR-08621: Le dogfooding NE DOIT autoriser automatiquement ni merge vers `main`, ni push, ni installation, ni redémarrage, ni déploiement.
- FR-08622: Une opération de livraison système DOIT rester un workflow hôte séparé, explicitement demandé et vérifié.
- FR-08623: La désactivation NE DOIT supprimer ni branches, ni worktrees, ni commits, ni caches utiles.
- FR-08624: L'état UI DOIT distinguer indisponible, désactivé, transition, activé et dégradé.
- FR-08625: Les événements d'activation, refus et recréation DOIVENT être corrélés sans journaliser secrets ou contenu de source.
- FR-08626: Maicie reste un plugin du checkout Bridget et ne reçoit pas de projet système indépendant.
- FR-08627: Le mode activé DOIT être présenté comme une frontière de confiance par projet et non comme une isolation entre agents du même conteneur; les agents peuvent techniquement observer les autres worktrees montés du projet.
- FR-08628: L'avertissement expert DOIT indiquer que la possibilité de committer exige l'écriture de métadonnées Git partagées et ne constitue donc pas une protection contre un agent malveillant du projet système.
- FR-08629: Bridget DOIT attribuer un worktree non principal, sur une branche distincte de `main`, avant tout lancement d'agent en mode activé.

## Exigences non fonctionnelles

- NFR-08601: La vérification d'attestation du checkout doit être déterministe, locale et sans exécuter le code du dépôt.
- NFR-08602: L'activation et la désactivation doivent être atomiques et idempotentes.
- NFR-08603: Toute projection doit omettre secrets, contenu de fichiers et commandes Git.
- NFR-08604: L'UI doit rendre le risque lisible sans masquer les actions irréversibles derrière un libellé vague.
- NFR-08605: Aucun nouveau service de secrets, orchestrateur ou système de rôles ne doit être ajouté.

## Entités clés

- **Projet système Bridget**: projet unique protégé et lié au checkout de l'installation.
- **Réglage de dogfooding**: décision serveur expert, génération et mode confirmé.
- **Politique de montage système**: lecture seule ou écriture selon le mode.
- **Epoch d'environnement**: version imposant la recréation du conteneur partagé.
- **Attribution de worktree**: réservation d'un worktree à un travail actif.

## Cas limites

- Le checkout configuré est déplacé ou remplacé.
- Le mode change pendant qu'un agent démarre.
- La recréation Docker échoue après arrêt de l'ancien conteneur.
- Un agent externe crée ou retire un worktree pendant la réconciliation.
- Le daemon actif est lui-même remplacé pendant une livraison séparée.
- Le projet système est retiré alors que le dogfooding est activé.

## Critères de succès

- SC-08601: Aucun projet standard ne peut lire ou écrire le checkout Bridget par un montage ajouté par Bridget.
- SC-08602: En mode désactivé, cent pour cent des tentatives d'écriture depuis le conteneur système échouent au niveau du montage.
- SC-08603: En mode activé, un agent peut modifier, tester et committer dans son worktree sans permettre un déploiement automatique.
- SC-08604: Cent pour cent des transitions avec agent actif ou génération obsolète sont refusées sans changement de mode.
- SC-08605: Des agents internes et externes peuvent travailler simultanément dans des worktrees distincts sans copie du dépôt ni collision d'index.
- SC-08606: Après désactivation, les sources redeviennent lecture seule et tout l'historique Git reste intact.
- SC-08607: Aucun agent géré en mode activé n'est lancé dans le checkout principal ou sur la branche `main`.

## Décisions actées

- Une seule autorité projet système existe par installation.
- Tous les agents de ce projet partagent la même permission.
- Le dogfooding est désactivé par défaut et n'est pas temporisé.
- L'édition est séparée de la livraison du système en production.
