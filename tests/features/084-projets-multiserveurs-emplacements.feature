Fonctionnalité: Projets multi-serveurs et catalogue d'emplacements

  Scénario: Une création utilise un workspace explicite
    Étant donné une source connectée avec un emplacement workspace
    Quand l'opérateur confirme la création d'un enfant valide
    Alors le daemon calcule le chemin final depuis cet emplacement

  Scénario: Un projet exact ne devient pas un parent de création
    Étant donné un emplacement exact_project
    Quand l'opérateur demande une création
    Alors le daemon refuse sans créer de dossier

  Scénario: La politique historique reste restrictive
    Étant donné une politique v1 existante
    Quand l'opérateur demande une création sous une ancienne racine
    Alors le daemon refuse tant qu'une migration v2 explicite n'a pas eu lieu

  Scénario: Deux sources conservent leurs projets distincts
    Étant donné deux sources ayant le même identifiant de projet local
    Quand Bridget Desktop affiche la flotte
    Alors chaque ligne est identifiée par la source et l'identifiant local
