# Contrat interne - project-ui-v1

## But

Frontière entre interface Bridget et autorités existantes. Ce contrat n'est ni
un contrat MCP, ni une API réseau générale, ni une permission de shell.

Chaque requête passe par le relais UI local authentifié, est versionnée et porte
un command_id si elle peut écrire.
Le relais est lié à la boucle locale du serveur. Bridget Desktop le consomme
exclusivement à travers le tunnel SSH de SPEC-074 lorsqu'il est attesté; aucun
port UI ne peut être exposé sur le réseau pour ce contrat.

## Lectures

| Intention | Résultat minimal |
|---|---|
| Réglages projets | Génération de politique, racines, défauts et disponibilité. |
| Parcourir une racine | Dossiers autorisés, chemin canonique, diagnostic Git minimal et statut connu. |
| Lister projets | Identité opaque, nom, chemin, liaison, coordinateur, agents et dernier audit. |
| Lire projet | Projection complète, conversation coordinateur et historique lisible. |
| Prévisualiser | Chemin final, conflit, état Git, mutation Git proposée, configuration et préconditions. |

Aucune lecture ne retourne secret, contenu de fichier, environnement ou chemin
hors racine autorisée.

## Écritures

| Intention | Préconditions | Résultat |
|---|---|---|
| Modifier racines | UI authentifiée, génération attendue et validation | Nouvelle génération après reload attesté. |
| Créer ou importer | Prévisualisation valide, configuration compatible, confirmation | Dossier éventuel, saga SPEC-065, état coordinateur réel. |
| Initialiser Git | Option visible et confirmée | Dépôt vide seulement, sans commit ni fichier ajouté. |
| Reconnecter | Projet path_missing, nouveau chemin autorisé, confirmation | Rebind et audit SPEC-065. |
| Retirer | Confirmation et cycle SPEC-075 si actif | Disable, historique conservé. |
| Réactiver | Projet retiré, dossier confirmé et racine validée | Mutation idempotente vers active, même identité, liaison active et audit. |
| Découverte | Coordinateur prêt et profil valide | Un DiscoveryRun borné. |

## Refus obligatoires

- chemin relatif, inexistant, non dossier, trop large ou hors racine;
- lien symbolique qui sort d'une racine;
- prévisualisation expirée ou différente de la confirmation;
- dossier existant dans un parcours de création;
- projet actif déjà connu;
- configuration non attestée ou incompatible;
- changement d'outil, fournisseur ou modèle du coordinateur;
- approbation de profil, extension ou secret;
- intention inconnue, champ additionnel ou corps non versionné;
- appel provenant de MCP, d'un agent ou d'une API générique.

## Idempotence et audit

Le même command_id avec les mêmes octets rend le même résultat. Un corps
divergent est refusé. Une panne après création de dossier ou liaison laisse un
état de reprise, jamais une suppression compensatoire.

Chaque mutation produit un audit non sensible : type, project_id, génération,
issue, instant et prochaine action. Les événements système conversationnels
sont des projections de ces verdicts, jamais une seconde vérité.

## Routes réalisées v1

- GET /v1/projects/settings : racines, génération, options attestées et défaut.
- POST /v1/projects/settings : génération attendue, racines et défaut futur.
- GET /v1/projects : projection des projets et de leurs états coordinateur.
- POST /v1/projects/preview et /v1/projects/confirm : création ou import.
- POST /v1/projects/disable, /rebind et /activate : transitions explicites.
- POST /v1/projects/discovery/continue : nouveau créneau seulement après la
  borne précédente et confirmation humaine.

Les corps d'écriture sont fermés, versionnés et refusent les command_id
invalide. Les routes ne comprennent ni profil, ni extension, ni secret, ni
commande libre. La liste projets est aussi portée par le snapshot UI existant,
sans boucle de polling dédiée.

## Extension de projection realisee

La projection de chaque projet porte aussi le dernier ProjectAuditEvent sous
forme reduite : operation, issue, generation et instant. Elle ne contient ni
command_id, ni racine precedente, ni contenu. Les agents restent filtres
uniquement par ProjectReference explicite ; un agent historique sans ce fait
reste visible dans Toute la flotte et non enregistre dans un projet.
