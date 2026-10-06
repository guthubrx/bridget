# language: fr
@spec138
Fonctionnalité: Collaborer dans son projet et choisir les échanges interprojets
  Les suggestions privilégient le projet attesté de l'émetteur.
  Un échange volontaire avec un autre projet reste possible.
  Les historiques existants restent exacts et ne sont pas rejoués.

  Contexte:
    Étant donné un vrai daemon temporaire avec une socket et des données isolées
    Et A1 et A2 attestés dans le projet A
    Et B1 attesté dans le projet B
    Et U dont le projet est inconnu

  @US1 @FR13801 @FR13802 @SC13801
  Scénario: L'annuaire de communication propose les agents locaux par défaut
    Quand A1 consulte l'annuaire et les suggestions sans option de portée
    Alors seuls A1 et A2 sont rendus comme agents du même projet
    Et B1 et U ne sont pas proposés automatiquement

  @US1 @FR13802 @FR13805 @SC13801 @SC13802
  Scénario: La vue globale volontaire ne vaut pas mandat d'envoi
    Quand A1 choisit volontairement la vue globale
    Alors A1 et A2 apparaissent locaux et B1 apparaît extérieur
    Et U apparaît avec un projet inconnu
    Quand A1 tente ensuite un nouvel envoi à B1 sans motif interprojets
    Alors aucun message n'est déposé et B1 n'est pas notifié

  @US1 @FR13801 @FR13804 @SC13801
  Scénario: L'absence locale ne provoque aucun repli extérieur
    Étant donné aucun candidat local disponible pour le travail demandé
    Quand A1 demande une suggestion automatique
    Alors l'absence de candidat est expliquée
    Et B1 et U ne sont pas proposés comme remplaçants

  @US1 @FR13804 @SC13801
  Scénario: Un émetteur inconnu ne reçoit aucune fausse suggestion locale
    Quand U consulte l'annuaire local
    Alors aucun agent n'est déclaré du même projet que U
    Et le résultat explique que le projet de U est inconnu

  @US1 @FR13803 @FR13813 @SC13805 @SC13806
  Scénario: Le diagnostic global historique conserve son contrat
    Quand un client de diagnostic consulte ListAgents
    Alors la projection globale historique reste disponible
    Et le domaine d'affichage n'est pas utilisé comme preuve d'appartenance

  @US1 @FR13803 @FR13804 @FR13814 @SC13805
  Scénario: Le client de fond établit son propre contexte validé
    Étant donné un run du projet A sans identité T3 héritée
    Quand son client négocié présente la racine explicite et l'hôte au daemon
    Alors le daemon valide son propre contexte de communication
    Et aucun agent temporaire n'est enregistré par Register
    Et aucune identité T3 n'est empruntée
    Et le contexte d'un auxiliaire déjà attesté ne peut pas être remplacé

  @US2 @FR13805 @FR13806 @SC13802 @SC13803
  Scénario: Un envoi extérieur sans choix volontaire est refusé avant tout effet
    Quand A1 envoie un nouveau corps à B1 sans motif interprojets
    Alors le résultat exige un motif interprojets volontaire
    Et aucun corps n'est déposé ni remis
    Et aucune notification n'est créée

  @US2 @FR13805 @SC13802
  Plan du scénario: Le motif interprojets invalide est refusé
    Quand A1 envoie à B1 avec un motif <motif>
    Alors le résultat indique un motif invalide
    Et aucun dépôt ni notification n'est créé

    Exemples:
      | motif                                |
      | vide                                 |
      | composé uniquement d'espaces          |
      | null explicite                       |
      | supérieur à 512 octets UTF-8          |
      | contenant un NUL ou un contrôle       |

  @US2 @FR13805 @FR13806 @SC13803
  Scénario: La demande explicite est suffisante pour envoyer entre projets
    Quand A1 envoie à B1 le corps exact avec un motif interprojets valide
    Alors l'avertissement émetteur est établi avant le dépôt et la notification
    Et une seule notification est créée
    Et le résultat de l'outil contient l'avertissement et le motif
    Et le corps remis reste identique octet pour octet
    Et aucune confirmation humaine supplémentaire n'est requise

  @US2 @FR13807 @SC13803
  Scénario: La réponse réellement corrélée réutilise le mandat accepté
    Étant donné une demande suivie OPEN de A1 à B1 acceptée avec un motif
    Quand B1 répond à A1 en référençant cette demande
    Alors la réponse reprend le motif de cette demande pour ces participants inversés
    Et elle ne demande aucune confirmation humaine supplémentaire

  @US2 @FR13803 @FR13807 @SC13802 @SC13805
  Scénario: Une fausse référence de réponse n'autorise aucun destinataire
    Étant donné une demande suivie entre d'autres participants
    Quand A1 tente d'envoyer à B1 en référençant cette demande sans motif
    Alors le mandat de cette demande n'est pas hérité
    Et aucun dépôt ni notification extérieure n'est créé

  @US2 @FR13805 @FR13808 @FR13811 @SC13802 @SC13806
  Scénario: Un fil mixte silencieux est partagé avec tous ses membres
    Quand A1 crée un fil avec B1 sans motif interprojets
    Alors la création est refusée sans écriture partielle
    Étant donné ensuite un fil historique partagé entre A1 et B1
    Quand A1 publie un nouveau corps avec une liste de notification vide sans motif
    Alors le dépôt est refusé car B1 peut lire le corps
    Et aucune entrée ni notification n'est ajoutée

  @US2 @FR13804 @FR13805 @FR13808 @SC13802
  Scénario: Le membre inconnu n'annule pas la règle pour le membre extérieur connu
    Étant donné un fil partagé entre A1 et B1 et U
    Quand A1 publie un nouveau corps silencieux sans motif interprojets
    Alors le projet inconnu de U ne supprime pas la divergence attestée entre A1 et B1
    Et aucun corps n'est déposé

  @US2 @FR13807 @FR13808 @SC13803
  Scénario: Le caller réutilise le motif volontaire pour la même audience
    Étant donné un mandat explicite de collaboration entre A1 et B1 dans un fil
    Quand A1 publie deux opérations distinctes avec ce motif structuré
    Alors chaque opération garde son motif dans son canon
    Et le fil ne crée aucun consentement général depuis son historique
    Et aucun autre destinataire ni mission n'est autorisé implicitement

  @US3 @FR13809 @FR13810 @SC13804
  Scénario: Un local occupé ne déclenche pas un recrutement dans un autre projet
    Étant donné A2 occupé et une boucle du projet A
    Quand la boucle cherche un nouvel agent automatiquement
    Alors elle conserve son périmètre local
    Et elle ne recrute pas B1 ni U

  @US3 @FR13809 @FR13810 @SC13804
  Scénario: Le rôle ROOT configuré ne suffit pas à créer un mandat extérieur
    Étant donné une boucle du projet A avec B1 configuré comme ROOT sans mandat
    Quand une escalade nécessite une décision ROOT
    Alors une décision à prendre est rendue visible
    Et B1 n'est pas notifié automatiquement

  @US3 @FR13807 @FR13810 @SC13804
  Scénario: Les trois rôles déjà mandatés conservent leurs rappels
    Étant donné un responsable et un coordinateur et un ROOT extérieurs explicitement mandatés
    Quand plusieurs contrôles de la même mission déclenchent les rappels prévus
    Alors chaque rôle reçoit son rappel prévu avec son motif structuré
    Et aucune nouvelle confirmation humaine n'est exigée à chaque passage
    Et aucune nouvelle mission ou cible n'hérite de ce mandat

  @US3 @FR13807 @FR13812 @SC13806
  Scénario: La reprise conserve le motif et la source figés avant l'échec
    Étant donné une outbox avec le corps et le destinataire et la racine et le motif figés
    Et une première tentative d'envoi échouée
    Quand la racine ou le mandat du run est modifié avant la reprise
    Alors la reprise envoie l'ancienne enveloppe figée
    Et elle ne remplace pas le motif ni la source ni le destinataire depuis le run courant

  @US4 @FR13803 @FR13804 @SC13805
  Scénario: Dépôt et worktree et lien symbolique conservent le même projet
    Étant donné un dépôt Git attesté et son worktree et son lien symbolique
    Quand leur fait de communication est établi
    Alors ils partagent la même racine commune Git et le même hôte attesté
    Et ils sont reconnus comme appartenant au même projet

  @US4 @FR13803 @FR13804 @SC13805
  Scénario: Homonymie et identité forgée ne prouvent pas une appartenance
    Étant donné deux dépôts distincts portant le même nom et le même domaine
    Quand un message ou une connexion auxiliaire tente de fournir un faux projet
    Alors les deux projets ne sont pas fusionnés
    Et l'identité déclarée ne remplace pas la preuve du propriétaire vivant

  @US4 @FR13804 @FR13806 @FR13813 @SC13803 @SC13806
  Scénario: L'ancien client inconnu peut envoyer avec avertissement
    Quand un client historique sans projet prouvé envoie selon son contrat existant
    Alors l'envoi compatible reste possible
    Et le résultat émetteur signale l'incertitude du projet
    Et cet avertissement n'est pas ajouté au corps remis

  @US4 @FR13813 @SC13802 @SC13806
  Scénario: Le client ne transmet pas un motif à un serveur sans capacité
    Étant donné un serveur ancien qui n'annonce pas la capacité de portée138
    Quand le client prépare un nouvel envoi portant un motif interprojets
    Alors le client refuse avant de transmettre l'enveloppe nouvelle
    Et il ne retire pas le motif pour tenter un envoi dégradé

  @US4 @FR13811 @FR13812 @SC13806
  Scénario: Le reçu ancien reste stable après changement des faits et reprise
    Étant donné une opération acceptée avec son résultat durable
    Quand les faits projet changent puis le daemon temporaire redémarre
    Et le caller rejoue exactement la même opération
    Alors le résultat durable antérieur est rendu à l'identique
    Et aucun nouveau dépôt ni notification n'est créé

  @US4 @FR13811 @FR13812 @SC13806
  Scénario: Une opération ancienne acceptée en cours de remise garde son contrat
    Étant donné une ancienne opération acceptée mais encore en cours de remise
    Quand la règle de portée devient disponible et le daemon reprend
    Alors cette opération termine selon son contrat accepté
    Et aucune nouvelle notification n'est créée par un balayage de l'historique

  @US4 @FR13812 @SC13806
  Scénario: Le motif participe au conflit de rejeu de message et de fil
    Étant donné une clé d'opération acceptée avec un corps et un motif
    Quand le caller réutilise cette clé avec un motif ou un corps différent
    Alors le conflit idempotent est rendu pour le message ou le dépôt de fil
    Et aucun effet supplémentaire n'est produit
    Et une opération sans motif conserve ses octets canoniques historiques

  @US4 @FR13811 @FR13812 @FR13814 @SC13806
  Scénario: Lire et publier l'historique silencieux ne réveille personne
    Étant donné un fil historique mixte accessible à ses membres
    Quand un membre le lit selon ses accès existants
    Alors les corps exacts restent récupérables sans nouvelle notification
    Quand un membre autorisé publie un historique silencieux selon le contrat136
    Alors aucune notification n'est créée
    Et aucune mission existante n'est migrée ni réaffectée

  @US2 @FR13805 @FR13806 @FR13813 @SC13802 @SC13803
  Scénario: Une consigne SteerCurrent extérieure exige son motif avant injection
    Étant donné un client attesté du projet A et une exécution active du projet B
    Quand il demande SteerCurrent.message sans motif interprojets
    Alors la demande est refusée avant toute injection dans le contexte de B
    Quand il soumet une nouvelle commande avec un motif structuré valide
    Alors la garde établit le warning cross_project avant l'injection autorisée
    Et le résultat caller porte ce warning hors du corps exact de la consigne
    Et aucune deuxième confirmation humaine bloquante n'est demandée

  @US4 @FR13806 @FR13812 @FR13813 @SC13803 @SC13806
  Scénario: Le résultat de consigne accepté garde son warning après restart
    Étant donné une commande SteerCurrent acceptée avec un warning durable
    Quand les faits projet changent et le daemon temporaire redémarre
    Et le caller rejoue exactement la même commande
    Alors le résultat accepté et son warning sont rendus avant la garde mutable
    Et aucune nouvelle injection n'est effectuée
    Et canonical_bytes reste comparé exactement sans contenir le warning
    Et refusal_reason conserve uniquement sa responsabilité de refus

  @US4 @FR13812 @FR13814 @SC13806
  Scénario: Le contrôle ancien conserve son canon et migre ses warnings à vide
    Étant donné une base existante sans colonne project_warnings de contrôle
    Quand la migration compatible ajoute cette colonne à execution_control_commands
    Alors les anciennes commandes rendent un tableau de warnings vide
    Et leurs octets canoniques restent inchangés
    Et Interrupt sans message garde son contrat existant

  @US3 @FR13812 @FR13814 @SC13806
  Scénario: Une publication concurrente garde son chemin canonique avant archive
    Étant donné une mission dont la première réservation CAS vient de réussir
    Quand un résultat est publié avant la tentative d'archivage
    Alors la publication et l'archivage utilisent le même verrou existant
    Et le résultat publié conserve son fichier, son result_path et son verdict
    Et cette publication n'est pas déplacée comme une ancienne archive
