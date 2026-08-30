# language: fr
@SPEC-065
Fonctionnalité: Registre et identité des projets
  Afin qu'un projet soit corrélé sans ambiguïté entre Maicie, Bridget et la flotte
  l'opérateur enregistre une identité stable et une liaison hôte réversible.

  @US1 @SC-001 @SC-003
  Scénario: Enregistrer une racine valide crée une identité et une liaison host
    Étant donné une racine Git existante sous une racine hôte autorisée
    Quand l'opérateur enregistre le projet par la commande locale Maicie
    Alors Maicie rend un `project_id` opaque et stable
    Et Bridget crée une liaison active avec backend `host`
    Et aucune donnée du dépôt ni du worktree n'est modifiée

  @US1 @SC-002
  Scénario: Rejouer une commande après redémarrage conserve une seule identité
    Étant donné une commande d'enregistrement durable à une frontière de saga
    Quand Maicie ou Bridget redémarre puis reprend la même commande
    Alors le résultat durable est renvoyé avec le même `project_id`
    Et une seule liaison active est conservée
    Et aucun second événement d'audit n'est créé

  @US1 @SC-003
  Plan du scénario: Une racine invalide est refusée avant écriture durable
    Étant donné une proposition de racine "<racine>"
    Quand Bridget vérifie la politique de racines
    Alors l'enregistrement est refusé avec "<raison>"
    Et aucune identité active ni liaison n'est créée

    Exemples:
      | racine                    | raison                 |
      | chemin/relatif             | invalid_absolute_root  |
      | /racine/inexistante        | root_missing           |
      | /                          | root_too_broad         |
      | /hors/politique/projet     | root_outside_allowed_prefixes |

  @US1 @SC-002 @SC-005
  Scénario: Deux alias de la même racine convergent sans double identité
    Étant donné deux chemins qui se canonisent vers la même racine Git
    Quand deux commandes d'enregistrement distinctes arrivent en concurrence
    Alors une seule identité devient active
    Et l'autre commande converge vers cette identité ou reçoit un conflit durable
    Et l'audit ne contient qu'un événement pour chaque mutation effective

  @US2 @SC-004
  Scénario: Une racine déplacée reste diagnostiquable sans correction implicite
    Étant donné un projet actif dont la racine hôte a été déplacée
    Quand l'opérateur consulte son statut
    Alors la liaison indique `path_missing`
    Et le `project_id` reste visible
    Et aucun chemin n'est corrigé automatiquement

  @US2 @SC-005 @SC-006
  Scénario: Rebind et désactivation préservent le code et les agents actifs
    Étant donné un projet actif avec un agent sur la génération courante
    Quand l'opérateur confirme un rebind vers une racine autorisée puis désactive le projet
    Alors le même `project_id` est conservé et l'ancienne liaison est auditée
    Et l'agent actif termine sur son ancienne génération sans arrêt implicite
    Et toute nouvelle admission attribuée au projet est refusée
    Et l'empreinte du dépôt et des worktrees est identique avant et après

  @US2 @SC-005
  Scénario: Le rapprochement de review_project exige une confirmation explicite
    Étant donné une configuration `review_project` historique
    Quand l'opérateur exécute d'abord un dry-run puis confirme le rapprochement
    Alors le résultat décrit une seule identité et une seule trace d'audit
    Et le démarrage normal ne crée aucun projet à partir de cette configuration

  @US3 @SC-007 @SC-008 @SC-011
  Scénario: Une exécution conserve sa référence projet dans les faits durables
    Étant donné un agent lancé pour un projet enregistré et un agent historique sans projet
    Quand Bridget persiste la génération, le snapshot, la projection et un incident délégué
    Alors les faits du premier agent portent le même `ProjectReference`
    Et les faits historiques portent `None` et restent utilisables
    Et une reprise par curseur ou un acquittement ne redéduit jamais le projet depuis cwd ou domain

  @US3 @SC-009
  Scénario: Une divergence entre délégation et exécution est refusée
    Étant donné une délégation Maicie liée au projet A
    Quand une exécution du projet B est proposée
    Alors Bridget refuse la proposition avant effet runtime
    Et Maicie ne réécrit ni la délégation ni l'objectif

  @US3 @SC-010
  Scénario: Le registre reste une extension locale sans composant externe
    Étant donné un déploiement existant utilisant le backend host
    Quand l'opérateur n'enregistre aucun projet
    Alors les commandes historiques de lancement et de communication restent valides
    Et aucun daemon, crate ou dépendance externe supplémentaire n'est nécessaire
