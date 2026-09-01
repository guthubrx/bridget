# Feature Specification: Projets multi-serveurs et catalogue d'emplacements

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 084-projets-multiserveurs-emplacements
Titre: Projets multi-serveurs et catalogue d'emplacements
Statut: Ready for implementation
Priorité: P1
Tâches: 18/34 (53%)

Résumé:
- Contexte: Bridget Desktop connaît plusieurs serveurs, mais la création et l'import de projets ne rendent pas toujours le serveur explicite et le daemon confond aujourd'hui racines autorisées et emplacements de création.
- Objectif: choisir clairement le serveur et un emplacement déclaré, sans jamais transformer un checkout existant en répertoire parent par défaut.
- Dépendances: SPEC-065, SPEC-074, SPEC-076, SPEC-080, SPEC-081-flotte-globale-sources-ui

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-084-projets-multiserveurs-emplacements`
**Created**: 2026-08-31
**Status**: In progress
**Priority**: P1
**Dependencies**: SPEC-065, SPEC-074, SPEC-076, SPEC-080, SPEC-081-flotte-globale-sources-ui

## Contexte

Bridget Desktop agrège déjà plusieurs sources locales ou distantes et sait ouvrir le panneau du serveur sélectionné. Le daemon sait créer ou importer un projet sous une politique de racines autorisées. Cependant, la politique v1 est une simple liste de chemins et le même contrôle accepte aussi bien un projet exact qu'un descendant. Le checkout `/home/moi/bridget-referent/bridget` peut donc être présenté comme parent de création alors qu'il ne doit représenter que le projet Bridget existant.

La correction doit préserver l'architecture distribuée actuelle: chaque daemon reste autorité de ses projets, Bridget Desktop compose la flotte, et aucune base centrale de projets n'est ajoutée.

## Objectifs

- Rendre le serveur cible visible et modifiable avant toute création ou importation.
- Remplacer la liste ambiguë de racines par un catalogue d'emplacements typés et nommés.
- Distinguer un espace de travail autorisant des descendants d'un projet exact uniquement importable.
- Réserver les emplacements système aux parcours experts et les exclure des créations standard.
- Conserver une identité globale non persistée côté daemon sous la forme `(source_id, project_id)` dans Bridget Desktop.
- Migrer la politique v1 sans ouvrir silencieusement de nouveaux droits de création.

## Hors périmètre

- L'exécution Docker d'un projet, traitée par SPEC-085.
- Le projet système Bridget et le dogfooding, traités par SPEC-086.
- Une base centrale ou un daemon fédérateur de projets.
- La copie automatique d'un dépôt entre serveurs.
- La création d'un serveur ou d'un profil SSH depuis le dialogue projet.
- La découverte automatique de tous les répertoires du serveur.
- Un chemin de création codé en dur tel que `/home/moi/projects`.

## User Story 1 - Choisir explicitement le serveur - P1

Comme opérateur, je veux voir et pouvoir changer le serveur cible avant de créer ou importer un projet afin de savoir où l'opération sera exécutée.

### Scénarios d'acceptation

1. Le dialogue de création ou d'import affiche le nom et l'état de la source cible, même lorsqu'il a été ouvert depuis une source déjà sélectionnée.
2. L'opérateur peut choisir une autre source connectée avant confirmation sans perdre les champs compatibles.
3. Une source déconnectée ou sans capacité projet reste visible mais ne peut pas recevoir l'opération, avec une raison précise.
4. La commande finale est envoyée uniquement au daemon de la source confirmée.
5. Deux projets portant le même `project_id` sur deux sources restent distingués par le couple `(source_id, project_id)` dans le Desktop.

## User Story 2 - Choisir un emplacement sûr - P1

Comme opérateur, je veux choisir un emplacement nommé dont les capacités sont claires afin de ne pas créer un projet dans le checkout d'un autre projet.

### Scénarios d'acceptation

1. Un emplacement `workspace` accepte la création d'un enfant et l'import d'un descendant canonique.
2. Un emplacement `exact_project` accepte uniquement l'import ou la liaison de son chemin exact.
3. Un emplacement `system_only` n'apparaît pas dans le parcours standard.
4. Le chemin final est prévisualisé avant confirmation sans autoriser sa saisie libre si le catalogue ne l'autorise pas.
5. Les liens symboliques, traversées et chemins hors catalogue sont refusés après canonicalisation.

## User Story 3 - Administrer le catalogue sans élargissement implicite - P1

Comme exploitant, je veux modifier le catalogue depuis le centre de contrôle avec génération et prévisualisation afin de maîtriser les zones accessibles aux projets.

### Scénarios d'acceptation

1. Chaque emplacement possède un identifiant opaque stable, un libellé, un chemin absolu canonique, un type et des marqueurs fermés.
2. Une mise à jour nécessite la génération attendue, une prévisualisation et une confirmation explicite.
3. Une politique v1 reste lisible mais n'autorise aucune création d'enfant tant qu'elle n'est pas migrée explicitement en `workspace`.
4. Le retrait d'un emplacement n'efface ni ne délie les projets existants.
5. Les entrées imbriquées ou incohérentes sont refusées lorsqu'elles rendent l'autorité ambiguë.

## Exigences fonctionnelles

- FR-08401: Bridget Desktop DOIT afficher la source cible dans tous les parcours de création et d'import.
- FR-08402: La source cible DOIT rester modifiable jusqu'à la confirmation finale.
- FR-08403: Seules les sources connectées déclarant la capacité projet compatible DOIVENT accepter la confirmation.
- FR-08404: La commande DOIT être exécutée par le daemon de la source confirmée, sans mutation d'une autre source.
- FR-08405: Le Desktop DOIT utiliser `(source_id, project_id)` comme identité de composition inter-serveurs sans modifier l'identité locale du daemon.
- FR-08406: Le daemon DOIT exposer un catalogue versionné d'emplacements au lieu d'une liste de racines sans sémantique.
- FR-08407: Un emplacement DOIT contenir `location_id`, `label`, `canonical_path`, `kind`, `system_only` et `default_creation`.
- FR-08408: Les seules valeurs v2 de `kind` DOIVENT être `workspace` et `exact_project`.
- FR-08409: Un `workspace` DOIT autoriser uniquement la création d'un enfant direct validé et l'import d'un descendant canonique.
- FR-08410: Un `exact_project` DOIT refuser toute création d'enfant et n'autoriser que son chemin exact.
- FR-08411: Un emplacement `system_only` DOIT être absent des parcours standard et inaccessible à une création standard.
- FR-08412: Au plus un emplacement non système par serveur PEUT être marqué `default_creation`.
- FR-08413: L'absence d'emplacement de création par défaut DOIT conduire à un choix explicite, jamais à un chemin inventé.
- FR-08414: Les chemins DOIVENT être absolus, existants lorsque requis, canonicalisés et protégés contre les liens symboliques ou traversées hors périmètre.
- FR-08415: La création DOIT prévisualiser le chemin final et refuser collision, répertoire non vide non prévu et nom invalide avant effet.
- FR-08416: L'import DOIT préserver le dépôt existant et ne DOIT déplacer, copier ni initialiser implicitement son contenu.
- FR-08417: La politique v1 DOIT être chargée en compatibilité comme autorité d'import exact uniquement, sans droit de création descendant.
- FR-08418: La migration d'une entrée v1 vers `workspace` DOIT être explicite, prévisualisée et confirmée.
- FR-08419: Le catalogue DOIT réutiliser la génération, l'écriture atomique et les reçus du centre de contrôle existant.
- FR-08420: Retirer un emplacement du catalogue NE DOIT supprimer, arrêter, déplacer ni délier un projet déjà enregistré.
- FR-08421: Les emplacements ambigus, dupliqués ou imbriqués avec capacités contradictoires DOIVENT être refusés.
- FR-08422: Les réponses UI NE DOIVENT exposer que les chemins nécessaires à l'opérateur autorisé et aucune donnée secrète.
- FR-08423: Aucune base globale, synchronisation de fichiers ou nouvelle autorité réseau NE DOIT être ajoutée.
- FR-08424: La prévisualisation de migration DOIT lister les projets déjà liés sous chaque racine v1 afin que le passage restrictif ne soit jamais présenté comme neutre.
- FR-08425: Le changement de source DOIT être orchestré par Bridget Desktop avant l'envoi au relais cible; un panneau serveur NE DOIT piloter directement aucun autre serveur.

## Exigences non fonctionnelles

- NFR-08401: Le changement de source dans le dialogue doit mettre à jour les emplacements en moins de 500 ms hors latence du tunnel.
- NFR-08402: Toute mutation du catalogue doit être idempotente et atomique.
- NFR-08403: Les contrats doivent être versionnés et refuser les champs inconnus.
- NFR-08404: Les refus doivent être fermés, localisables et ne jamais contenir de secret.
- NFR-08405: Les parcours doivent être utilisables au clavier et annoncer source, emplacement et validation.

## Entités clés

- **Source Desktop**: profil local ou distant déjà connu, état de connexion et capacités attestées.
- **Emplacement projet**: autorité serveur nommée sur un chemin canonique et un type fermé.
- **Sélection projet**: source, emplacement, mode création ou import et prévisualisation.
- **Projet local au serveur**: identité opaque et génération maintenues par le daemon concerné.
- **Identité composée Desktop**: couple `(source_id, project_id)` utilisé seulement pour agréger plusieurs serveurs.

## Cas limites

- La source se déconnecte après la prévisualisation.
- Le catalogue change entre prévisualisation et confirmation.
- Deux sources ont le même libellé ou le même `project_id` local.
- Le chemin canonique change à cause d'un lien symbolique.
- Un emplacement v1 contient un checkout existant utilisé en production.
- Le répertoire enfant apparaît entre validation et création.
- Un emplacement est retiré alors qu'un projet lié y existe.

## Critères de succès

- SC-08401: Cent pour cent des confirmations de création/import affichent une source et un emplacement attestés.
- SC-08402: Cent pour cent des tentatives de création sous `exact_project`, `system_only` ou une entrée v1 non migrée sont refusées sans effet.
- SC-08403: Cent pour cent des commandes sont routées vers la source confirmée et aucune mutation croisée n'est observée.
- SC-08404: Les tests couvrent canonicalisation, liens symboliques, collision, génération concurrente, migration v1, source déconnectée et identités locales identiques.
- SC-08405: Le checkout Bridget existant peut être déclaré `exact_project` et n'est plus proposé comme parent de création standard.

## Décisions actées

- Le modèle reste distribué par serveur.
- Le serveur est toujours visible dans le dialogue, même s'il est présélectionné.
- Aucun chemin de workspace n'est codé en dur.
- La compatibilité v1 est restrictive et nécessite une promotion explicite pour créer des descendants.
