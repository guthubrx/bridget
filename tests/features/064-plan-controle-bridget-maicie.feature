# language: fr
@SPEC-064
Fonctionnalité: Plan de contrôle Bridget et Maicie
  Afin que mission, remise et exécution restent des vérités distinctes
  Bridget applique le contrôle runtime et Maicie conserve les décisions métier.

  @US1 @SC-001 @SC-002 @SC-003
  Scénario: Une soumission humaine pilote le tour actif sans acquittement prématuré
    Étant donné une exécution Bridget active et une livraison humaine admissible
    Quand Bridget choisit une opération de pilotage supportée
    Alors la soumission, la livraison et l'exécution ont des identifiants distincts
    Et l'acquittement attend une preuve de visibilité fournisseur corrélée
    Et un accusé fournisseur seul ne clôt pas la livraison

  @US1 @SC-002 @SC-004
  Scénario: Un événement tardif ne modifie pas une nouvelle exécution
    Étant donné une ancienne exécution terminée et une nouvelle génération active
    Quand un terminal de l'ancienne génération arrive
    Alors seule l'ancienne exécution reçoit ce terminal
    Et la nouvelle exécution reste inchangée

  @US1 @SC-003
  Scénario: Une demande d'autorisation garde son attente visible
    Étant donné une exécution en attente d'autorisation fournisseur
    Quand un opérateur consulte son état
    Alors la raison d'attente et l'action autorisée sont visibles
    Et aucune transition métier Maicie n'est déduite de cette attente

  @US2 @SC-004
  Scénario: Un enfant possède un parent et un mandat durables
    Étant donné une exécution parente avec une politique de descendants
    Quand Bridget réserve puis démarre un agent enfant
    Alors le lien porte parent, enfant, mandat et génération
    Et un échec de démarrage referme la réservation
    Et aucun enfant orphelin n'est conservé

  @US2 @SC-004
  Scénario: Une limite de profondeur refuse sans modifier la flotte
    Étant donné une demande d'enfant qui dépasse la profondeur autorisée
    Quand Bridget évalue la réservation
    Alors la demande est refusée avec un motif structuré
    Et aucun processus enfant ni lien durable n'est créé

  @US3 @SC-005
  Scénario: Une reprise native conserve l'identité fournisseur
    Étant donné un fournisseur dont la capacité de reprise est attestée
    Quand Bridget reprend une exécution interrompue
    Alors le nouveau tour référence le thread fournisseur d'origine
    Et le mode de reprise est publié comme natif

  @US3 @SC-005
  Scénario: Une reprise non native devient une reconstruction explicite
    Étant donné un fournisseur sans capacité de reprise attestée
    Quand Bridget relance le travail
    Alors une nouvelle identité fournisseur est créée
    Et la carte textuelle de continuité est marquée comme reconstruction

  @US4 @SC-006 @SC-007
  Scénario: Maicie observe une exécution sans en devenir l'autorité
    Étant donné une délégation Maicie liée à une exécution Bridget
    Quand Bridget publie un fait runtime terminé ou indisponible
    Alors Maicie met à jour une projection de fraîcheur versionnée
    Et l'objectif et la délégation ne changent pas sans décision métier explicite

  @US4 @SC-007
  Scénario: Bridget continue quand Maicie est indisponible
    Étant donné une projection Maicie indisponible
    Quand Bridget reçoit puis exécute une soumission admissible
    Alors Bridget maintient sa remise et son exécution
    Et l'indisponibilité est observable sans transition métier implicite

  @US5 @SC-008
  Plan du scénario: Une opération dépend de la capacité observée
    Étant donné le fournisseur "<provider>" par le chemin "<path>" et une version attestée
    Quand Bridget demande l'opération "<operation>"
    Alors Bridget applique la capacité observée ou le repli explicite
    Et l'événement conserve fournisseur, chemin d'exécution et version

    Exemples:
      | provider | path                | operation             |
      | codex    | codex_app_server    | steer_current_turn    |
      | claude   | claude_stream_json  | interrupt_current_turn |
      | cursor   | acp                 | interrupt_current_turn |
      | faux     | inconnu             | steer_current_turn    |

  @US5 @SC-008
  Scénario: Cursor reste un fournisseur ACP sans adaptateur spécialisé
    Étant donné `provider_kind` égal à `cursor` et `execution_path` égal à `acp`
    Quand Bridget publie les capacités observées
    Alors Cursor est distingué dans l'affichage et l'audit
    Et le transport ACP commun exécute la commande
    Et aucun adaptateur Cursor distinct n'est requis

  @US6 @SC-009 @SC-010
  Scénario: Une limite runtime bloque une continuation sans clore la mission
    Étant donné une exécution qui atteint une limite de temps, usage ou descendants
    Quand Bridget évalue une continuation
    Alors Bridget publie une issue technique de limite
    Et aucun nouveau tour concurrent n'est créé
    Et Maicie ne clôt pas l'objectif sur ce seul fait runtime

  @US6 @SC-009
  Scénario: Une continuation politique exige une preuve d'inactivité
    Étant donné du travail restant et une politique de continuation admise
    Et une preuve d'inactivité plus récente que le dernier tour
    Quand Bridget réserve la continuation
    Alors une seule continuation est admise atomiquement

  @SC-011
  Scénario: Une compatibilité héritée possède une sortie documentée
    Étant donné une projection héritée encore consommée
    Quand le lot est validé
    Alors son flag, son rollback et sa date de retrait sont publiés
    Et aucune compatibilité sans consommateur n'est conservée
