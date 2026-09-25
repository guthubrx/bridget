# language: fr
# Spécification lisible liée à test-map.md : aucun runner Gherkin n'est installé.
# Chaque tag désigne un critère, pas une preuve déjà acquise.
@session_089
Fonctionnalité: Communiquer entre agents sans dépendre de l'interface ni d'un service compagnon
  Bridget transporte des demandes et leurs faits attestés.
  Un accusé de livraison ne prouve pas la réussite intellectuelle d'une mission.

  Contexte:
    Étant donné un paquet de test issu du dépôt indépendant 64.bridget
    Et un home, un cache, un magasin de contenus et une socket réservés à ce test
    Et aucune donnée ni aucun processus de la flotte historique utilisé par le test
    Et un délai global qui fait échouer proprement le harnais en cas de blocage

  @SC_08901 @T015 @T016 @T020 @fournisseurs_reels
  Scénario: Une demande entre deux fournisseurs reçoit une seule réponse liée
    Étant donné deux sessions réelles de fournisseurs distincts autorisés
    Quand la première session envoie une demande suivie à la seconde
    Et la seconde répond avec l'identifiant intégral de cette demande
    Alors une seule injection de chaque message est attestée par le pilote
    Et la demande devient answered dans le magasin maître
    Et chaque message livré possède exactement une entrée de ledger
    Et aucun rappel n'est émis après la clôture attestée
    Mais Bridget ne déclare pas la mission métier réussie

  @SC_08902 @T014 @T022
  Plan du scénario: Un rejeu divergent ne réécrit jamais le premier dépôt
    Étant donné un envoi accepté dont les octets canoniques sont conservés
    Quand le CLI puis l'outil MCP rejouent la même clé avec le même issued_at
    Alors les deux façades retrouvent les mêmes identifiants et la même issue
    Et les octets du record sont identiques à ceux du premier envoi
    Quand la même clé est rejouée en modifiant seulement "<champ>"
    Alors EnvelopeMismatch est retourné
    Et le premier record, ses références et les demandes liées sont inchangés

    Exemples:
      | champ       |
      | body        |
      | cible       |
      | reply       |
      | deadline    |
      | in_reply_to |

  @SC_08903 @T017 @crash_reel
  Plan du scénario: La reprise respecte les faits durables à chaque frontière de crash
    Étant donné une demande à identifiants et octets immuables
    Et un jalon déterministe au point "<frontière>"
    Quand le processus de test est interrompu brutalement à ce jalon
    Et le daemon isolé redémarre puis reçoit le même rejeu
    Alors le résultat correspond aux faits durables de cette frontière
    Et aucun succès n'est inventé à partir de l'absence d'accusé
    Et une éventuelle remise conserve les mêmes octets et la même destination d'instance
    Et le compteur de prompts respecte le domaine de garantie documenté
    Mais une limite de garantie du crash wrapper ou de l'horizon reste explicite

    Exemples:
      | frontière                       |
      | avant réservation               |
      | après Prepared                  |
      | après remise avant issue        |
      | après issue avant accusé client |

  @SC_08904 @T025 @T026 @T027 @deux_serveurs_reels
  Scénario: La fédération SSH rejoint une seule autorité et survit à la coupure
    Étant donné deux machines reliées par un tunnel SSH de test
    Et une session distante enregistrée auprès du daemon maître unique
    Quand who et ledger sont consultés depuis chaque machine
    Alors ils exposent les mêmes données autorisées du maître
    Quand seul le tunnel de test est coupé pendant un échange
    Alors l'absence de connexion n'est pas annoncée comme une livraison
    Quand ce tunnel est rétabli
    Alors les identifiants et la corrélation de l'échange sont conservés
    Et aucun deuxième prompt ni redémarrage du fournisseur n'est induit par la reconnexion

  @SC_08905 @T019
  Scénario: La jonction du rejeu au suivi conserve l'ordre et annonce les lacunes
    Étant donné une attache réelle avec des entrées avant et après une rotation de journal
    Quand le rattrapage atteint SnapshotCaughtUp
    Et seulement ensuite une nouvelle entrée est injectée dans le relais vivant
    Alors chaque séquence attendue est reçue une seule fois dans l'ordre
    Et les octets source inhabituels et leur provenance restent identiques
    Quand une lacune attestée est suivie d'un fragment valide
    Alors le flux et l'observation restent Gap jusqu'à un rattrapage explicitement réussi
    Et Gap reste distinct d'Unavailable
    Et un mutant confondant ces états fait échouer cet oracle

  @SC_08906 @T007 @T009 @T010 @T011 @T013
  Scénario: Le paquet de communication est indépendant du produit élargi
    Étant donné une installation de test sans sources de service compagnon ni interface graphique
    Et sans serveur web, runtime Docker de projet ni binaire tmux
    Quand le paquet est construit puis deux agents communiquent
    Alors l'envoi, la réponse liée, le ledger et l'attache fonctionnent
    Et le graphe de dépendances de production ne contient aucun crate métier ni renderer
    Et une commande retirée reçoit un refus explicite
    Et les contrôles d'accès fournisseur et de contenu restent actifs
    Et aucun chemin historique n'est ouvert par repli

  @SC_08907 @T031
  Scénario: Une migration n'altère que la copie explicitement fournie
    Étant donné des fixtures historiques des magasins Bridget retenus
    Et leurs empreintes calculées avant la migration
    Quand chaque copie est migrée puis rouverte
    Alors identifiants, octets, références et états terminaux sont préservés
    Et la réexécution de la migration ne change pas ces résultats
    Et un schéma futur inconnu est refusé
    Et les empreintes des sources ainsi que les données de production sont inchangées

  @SC_08908 @T029 @T030
  Plan du scénario: Une frontière de sécurité refuse avant l'effet interdit
    Étant donné une opération nécessitant la frontière "<frontière>"
    Quand l'opération viole cette frontière
    Alors un refus typé ou une erreur de transport honnête est retourné selon le contrat
    Et aucun effet interdit, processus non autorisé ni accès indu n'a eu lieu
    Et une corruption n'est pas transformée en refus métier certain

    Exemples:
      | frontière                        |
      | propriétaire et droits du fichier |
      | absence de symlink sensible       |
      | identité et capacité de connexion |
      | taille de la ligne avec son LF    |
      | nombre maximal de connexions     |
      | échéance absolue de la connexion  |

  @SC_08909 @T020 @T021 @fournisseurs_reels
  Plan du scénario: Chaque pilote expose des faits sans changer de fournisseur
    Étant donné une session réelle autorisée par le chemin "<pilote>"
    Quand elle reçoit un tour puis publie ses observations
    Alors le modèle et l'effort affichés proviennent de faits attestés
    Et un signal absent reste inconnu
    Et son journal peut être attaché indépendamment du nom de son protocole
    Quand le fils de test termine pendant un tour actif
    Alors l'attente se termine dans la borne sans processus orphelin
    Et aucune API facturée n'est substituée implicitement

    Exemples:
      | pilote                 |
      | Codex natif            |
      | Claude natif           |
      | ACP                    |
      | GLM via Claude Code    |

  @SC_08910 @T023
  Scénario: La skill suffit à envoyer puis répondre sans connaître le service compagnon
    Étant donné un agent ne disposant que de la skill courte livrée avec le noyau
    Quand il exécute le scénario envoyer, répondre et consulter via CLI ou MCP
    Alors la réponse porte in_reply_to
    Et une seule réponse est rattachée à la demande
    Et les statuts expliqués correspondent aux issues réelles du contrat
    Et in_flight n'est pas présenté comme outcome_unknown
    Et aucune connaissance du magasin ni du métier du service compagnon n'est nécessaire

  @SC_08911 @T003 @T034 @T036
  Scénario: La simplification ne fait pas disparaître une preuve gênante
    Étant donné l'inventaire des fichiers de tests du commit source épinglé
    Quand la suite du paquet extrait est comparée à cet inventaire
    Alors chaque fichier retiré ou déplacé possède une disposition motivée par le périmètre
    Et les oracles communication des fichiers mixtes ont une destination vérifiée
    Et aucun test rouge n'est caché par un nouvel ignore
    Et les résultats fmt, clippy, tests et audit de dépendances sont consignés
    Et un fichier de test oublié fait échouer le contrôle de couverture

  @SC_08912 @T005 @T028 @T033
  Scénario: Le noyau diminue réellement sans dégrader le chemin de communication
    Étant donné une référence et un candidat mesurés sur le même hôte
    Quand 600 événements sont émis à 10 événements par seconde pendant 60 secondes
    Alors les pertes, p95, maximum et ressources sont consignés pour les parcours local et SSH
    Et les budgets historiques applicables restent respectés sans modification de seuil
    Et le biais d'horloge est documenté pour le parcours distant
    Et le nombre de modules et dépendances de production diminue réellement
