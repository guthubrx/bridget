# Feature Specification: Pilotage des rondes par projet dans l'interface

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 081-pilotage-rondes-projet-ui
Titre: Pilotage des rondes par projet dans l'interface
Statut: Implemented
Priorité: P1
Tâches: 22/22 (100%)
Tests: 12/12 (100%)

Résumé:
- Contexte: la politique de ronde par projet existe, mais l'opérateur ne peut pas la piloter ni en connaître l'état depuis l'interface.
- Objectif: permettre une activation ou désactivation sûre par projet et rendre son état réellement confirmé compréhensible.
- Dépendances: SPEC-079, SPEC-080

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-081-pilotage-rondes-projet-ui`
**Created**: 2026-08-31
**Status**: Implemented
**Priority**: P1
**Dependencies**: SPEC-079, SPEC-080

## Contexte

La SPEC-079 a établi une politique de ronde distincte pour chaque projet, désactivée par défaut et liée à la génération courante de ce projet. La SPEC-080 a établi une colonne Projets et un menu contextuel accessible par trois points, clic droit et clavier. Pourtant, l'opérateur doit encore utiliser une commande technique pour connaître ou modifier la ronde.

L'interface doit exposer cette autorité existante sans créer un second mécanisme de planification, sans modifier la cadence globale et sans laisser croire qu'une action est appliquée avant la confirmation de Bridget.

## Objectifs

- Activer ou désactiver la ronde indépendamment pour chaque projet actif.
- Montrer l'état confirmé de la ronde dans le contexte naturel du projet.
- Expliquer le dernier passage connu et le prochain passage attendu sans promettre un horaire non attesté.
- Conserver les garanties de génération, d'idempotence et de neutralité fournisseur de la SPEC-079.

## Hors périmètre

- Un interrupteur global de toutes les rondes.
- Une cadence configurable ou différente selon le projet.
- Un bouton de déclenchement manuel immédiat.
- La création d'un timer par projet.
- L'arrêt, la relance ou l'annulation d'un agent ou d'un travail.
- Une règle différente pour Claude, Codex, Cursor, Gemini ou tout autre fournisseur.

## User Story 1 - Piloter la ronde d'un projet actif - P1

Comme opérateur, je veux activer ou désactiver la ronde depuis le menu d'un projet afin de choisir quels projets reçoivent les réveils périodiques.

### Scénarios d'acceptation

1. Depuis les trois points, le clic droit ou `Maj + F10`, le menu d'un projet actif présente une action « Ronde de vigilance » avec son état confirmé.
2. L'activation confirmée ne modifie que le projet et la génération affichés.
3. La désactivation confirmée empêche les passages futurs sans interrompre un agent ni un travail déjà accepté.
4. Une réponse refusée ou indisponible conserve l'état précédent et affiche une erreur actionnable.
5. Un projet inactif ou dont le dossier manque présente l'action comme indisponible et en explique la cause.

## User Story 2 - Comprendre l'état sans agir - P1

Comme opérateur, je veux voir rapidement si la ronde est active et quand elle est susceptible de repasser afin de ne pas confondre attente normale et panne.

### Scénarios d'acceptation

1. La ligne d'un projet actif indique discrètement lorsque sa ronde est activée.
2. Le menu distingue une politique absente, une politique désactivée et une politique activée.
3. Le menu montre le dernier passage connu avec son résultat fermé : déposé, refusé ou indéterminé.
4. Si la ronde est activée, l'interface indique qu'un prochain passage est attendu au prochain cycle global, au plus tard dans sept minutes, sans afficher une heure exacte non attestée.
5. Après reconnexion d'un projet, l'ancienne décision n'est jamais présentée comme applicable à la nouvelle génération.

## User Story 3 - Conserver une commande sûre et générique - P1

Comme exploitant, je veux que l'action de l'interface utilise le même contrôle Bridget que la commande existante afin de conserver les protections et la compatibilité entre fournisseurs.

### Scénarios d'acceptation

1. Toute mutation porte un identifiant de commande, le projet et sa génération exacte.
2. Le rejeu de la même commande et de la même enveloppe ne produit pas un second effet.
3. Une génération divergente, un projet absent ou inactif est refusé sans modification.
4. Aucune racine hôte, secret, profil ou donnée fournisseur n'est exposé par le contrôle de ronde.

## Exigences fonctionnelles

- FR-08101: L'opérateur DOIT pouvoir activer ou désactiver la ronde d'un projet actif depuis son menu contextuel existant.
- FR-08102: L'action DOIT être accessible par les trois points, le clic droit et `Maj + F10`.
- FR-08103: L'interface DOIT afficher uniquement un état confirmé par Bridget et NE DOIT PAS appliquer de changement optimiste.
- FR-08104: Une politique absente DOIT être présentée comme désactivée et non configurée pour la génération courante.
- FR-08105: Une mutation DOIT être liée au projet, à sa génération courante et à un identifiant de commande stable.
- FR-08106: Un projet absent, inactif ou d'une génération divergente DOIT être refusé sans second effet.
- FR-08107: Après une nouvelle liaison, l'ancienne politique DOIT rester inapplicable jusqu'à une nouvelle décision explicite.
- FR-08108: Désactiver la ronde NE DOIT arrêter, relancer, annuler ni empêcher la reprise d'aucun agent ou travail.
- FR-08109: La ligne projet DOIT signaler discrètement une ronde activée sans ajouter de commande globale.
- FR-08110: Le menu DOIT exposer le dernier passage connu, son instant et un résultat fermé : déposé, refusé ou indéterminé.
- FR-08111: En l'absence de passage connu, l'interface DOIT l'indiquer explicitement sans inventer de résultat.
- FR-08112: Lorsque la ronde est activée, le prochain passage DOIT être décrit comme attendu au prochain cycle global dans un délai maximal de sept minutes, sans heure exacte calculée localement.
- FR-08113: Une erreur de transport, de capacité ou de validation DOIT conserver la dernière valeur confirmée et fournir un message actionnable.
- FR-08114: Le contrôle DOIT réutiliser la politique de ronde existante et NE DOIT créer ni second registre, ni second planificateur, ni timer par projet.
- FR-08115: Le contrôle DOIT être identique pour tous les fournisseurs et transports d'agents.
- FR-08116: Le contrôle NE DOIT exposer ni racine hôte, ni secret, ni contenu de message, ni configuration fournisseur.
- FR-08117: La cadence, le déclenchement manuel et l'activation globale DOIVENT rester hors de cette interface.

## Entités clés

- **Projet lié** : identité opaque, génération courante et état actif ou inactif.
- **Politique de ronde** : décision explicite de la génération courante, état activé ou désactivé, révision et date de mise à jour.
- **Dernier passage** : occurrence la plus récente effectivement traitée, résultat fermé et date d'observation.
- **Commande de politique** : intention idempotente d'activer ou désactiver une génération précise.

## Cas limites

- Le projet est retiré entre l'ouverture du menu et la confirmation.
- Le projet est reconnecté et change de génération pendant que le menu reste ouvert.
- La même commande est envoyée deux fois après une perte de réponse.
- Le daemon redémarre pendant la mutation.
- Une occurrence est rejouée par le timer global.
- Aucun passage n'a encore eu lieu depuis l'activation.
- Le dernier passage a une issue inconnue malgré une remise identifiée.

## Hypothèses

- La cadence globale reste fixée à sept minutes par la SPEC-079.
- L'interface est reliée au daemon local déjà authentifié et n'introduit pas de nouvelle permission distante.
- Le menu projet établi par la SPEC-080 reste la surface de commande canonique.

## Critères de succès

- SC-08101: En trois interactions ou moins depuis la liste des projets, l'opérateur peut connaître l'état confirmé et demander son changement.
- SC-08102: Cent pour cent des mutations testées avec une génération obsolète, un projet inactif ou une réponse refusée conservent l'état affiché avant la demande.
- SC-08103: Cent pour cent des projets activés sont signalés dans leur ligne et présentent un prochain passage attendu dans un délai maximal de sept minutes.
- SC-08104: Les tests automatisés couvrent activation, désactivation, rejeu identique, enveloppe divergente, nouvelle génération, projet inactif, capacité absente et perte de réponse.
- SC-08105: Aucun scénario de désactivation ne stoppe, ne relance ou n'annule un agent ou un travail.
- SC-08106: La projection rend cent pour cent des passages observés avec un résultat fermé ou indique explicitement qu'aucun passage n'est connu.
