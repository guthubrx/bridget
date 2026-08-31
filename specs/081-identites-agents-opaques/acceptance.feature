Fonctionnalité: Identités d'agents opaques

  Scénario: Le routeur refuse un nom historique
    Quand un wrapper s'enregistre avec "agent-2" comme agent_id
    Alors l'inscription est refusée

  Scénario: Le nom affiché ne modifie pas le routage
    Étant donné un agent_id UUID avec le display_name "Bibliothécaire"
    Quand son profil est renommé "Archiviste"
    Alors les messages et les reprises restent rattachés au même agent_id
    Et l'interface affiche "Archiviste"

  Scénario: Un fournisseur ne reçoit jamais l'identifiant opaque dans son en-tête
    Étant donné un message routé depuis un agent_id UUID
    Et son display_name vaut "Bibliothécaire"
    Quand le wrapper prépare le prompt fournisseur
    Alors l'émetteur affiché dans le prompt est "Bibliothécaire"
    Et l'agent_id d'origine reste disponible pour répondre

  Scénario: Une cible Maicie retirée exige un nouveau ciblage
    Étant donné une délégation et une outbox qui ciblent un agent supprimé
    Quand la migration d'identités est appliquée
    Alors la cible est marquée requires_retarget
    Et l'outbox n'est pas livrable automatiquement

  Scénario: Un runtime Docker ne reçoit qu'un Agent ID UUID
    Quand un lancement runtime est construit avec un agent_id non UUID
    Alors la commande docker exec est refusée
