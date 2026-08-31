Fonctionnalité: Environnement Docker partagé par projet
  Pour isoler les ressources d'un projet sans fragmenter Bridget ni Maicie
  En tant qu'opérateur Bridget
  Je veux préparer, utiliser et retirer un environnement Docker réversible par projet

  Contexte:
    Étant donné un daemon Bridget sur l'hôte
    Et un registre de projets SPEC-065 actif
    Et une politique runtime Docker hôte v1 valide

  Scénario: Préparer un conteneur avec une image immuable
    Étant donné le projet "projet-a" lié à une racine canonique
    Et une image fixture référencée par son identifiant sha256 immuable
    Quand je prépare son environnement Docker
    Alors son état devient "ready"
    Et le conteneur atteste project_id, binding_generation, digest et version de contrat
    Et sa racine système est en lecture seule
    Et il ne reçoit ni le HOME hôte, ni le socket Docker, ni un port publié

  Scénario: Refuser une politique ou une image non fiable sans fallback
    Étant donné le projet "projet-a" lié au backend docker
    Et une politique runtime absente, invalide ou modifiable par un autre compte
    Quand une préparation ou une admission Docker est demandée
    Alors l'opération échoue avec une raison Docker structurée
    Et aucun agent host n'est lancé
    Et le backend host historique des autres projets reste inchangé

  Scénario: Partager un conteneur entre agents du même projet
    Étant donné l'environnement "ready" du projet "projet-a"
    Et deux worktrees rattachés au même dépôt de "projet-a"
    Quand je lance deux agents gérés dans ces worktrees
    Alors ils utilisent le même container_id
    Et ils conservent des générations Bridget distinctes
    Et chacun conserve son cwd propre
    Et steering, interruption et incidents délégués gardent leur ProjectReference

  Scénario: Isoler deux projets distincts
    Étant donné les environnements Docker des projets "projet-a" et "projet-b"
    Quand j'inspecte les montages et ingress des deux conteneurs
    Alors aucun conteneur, state root ou montage écrivable n'est commun
    Et un wrapper de "projet-a" ne peut pas se présenter sur l'ingress de "projet-b"

  Scénario: Retirer Docker sans détruire les données du projet
    Étant donné le projet "projet-a" sans agent actif
    Et une empreinte de son dépôt, de ses worktrees et de son state root
    Quand je bascule explicitement son backend vers host puis retire son conteneur
    Alors le prochain agent suit le lancement host historique
    Et l'empreinte du dépôt, des worktrees et du state root est inchangée
    Et aucune suppression de conteneur ne détruit de donnée durable
