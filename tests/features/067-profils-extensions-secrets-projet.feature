Fonctionnalité: Profils, extensions et secrets bornés par projet
  Pour utiliser des fournisseurs réels sans exposer le home de l'hôte
  En tant qu'opérateur Bridget local
  Je veux approuver un profil projet, attester ses ressources et recréer son environnement sans fuite

  Contexte:
    Étant donné un registre de projets SPEC-065 actif
    Et une politique runtime Docker SPEC-066 attestée
    Et un catalogue hôte fermé de ressources synthétiques

  Scénario: Afficher puis approuver exactement un profil projet
    Étant donné un profil avec trois agents, deux extensions et deux références secrètes
    Quand l'opérateur local affiche la proposition
    Alors il voit les agents, modèles, efforts, capacités, image, limites et destinations
    Et il ne voit aucune valeur secrète ni chemin hôte privé
    Quand il approuve la définition résolue inchangée
    Alors le même digest, la génération de liaison et la version runtime sont épinglés

  Scénario: Refuser une proposition distante ou divergente
    Étant donné une proposition provenant d'un agent, de l'UI ou de MCP
    Quand elle tente une approbation ou une rotation
    Alors aucune route ni commande distante ne peut l'exécuter
    Et une différence de définition, image, capability ou politique refuse l'activation

  Scénario: Monter seulement les extensions approuvées
    Étant donné deux projets avec des extensions distinctes sous des racines autorisées
    Quand leurs environnements Docker sont recréés
    Alors chaque extension est montée en lecture seule dans sa destination fermée
    Et aucun projet ne voit le catalogue de l'autre
    Et une modification après approbation exige recreate avant un nouveau spawn

  Scénario: Utiliser un secret sans le persister
    Étant donné une référence fichier, répertoire et process-env synthétiques approuvées
    Quand le wrapper lance un fournisseur après admission de ses capabilities
    Alors les sources file et directory sont montées sous /run en lecture seule
    Et process-env est lu seulement dans le wrapper et injecté au processus fournisseur
    Et aucun argument Docker, label, store, journal, projection ou incident ne contient la sentinelle

  Scénario: Révoquer ou faire tourner une génération de secret
    Étant donné un profil actif et des agents déjà lancés
    Quand l'opérateur approuve une rotation, une révocation ou un rebind
    Alors les nouveaux spawns sur l'ancienne génération sont refusés avant lecture ou montage
    Et les agents existants terminent sur leur ancienne génération sans arrêt implicite
    Et une nouvelle approbation suivie d'un recreate est requise

  Scénario: Conserver le contrat fournisseur-neutre
