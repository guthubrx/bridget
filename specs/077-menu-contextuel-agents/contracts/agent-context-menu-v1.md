# Contrat UI - Menu contextuel agent v1

## Déclencheurs équivalents

| Déclencheur | Ancrage | Résultat |
|---|---|---|
| clic trois points | bouton | menu unique de l'agent |
| clic droit ligne | position du pointeur | même menu |
| Menu contextuel ou Maj+F10 | ligne focalisée | même menu |

Le navigateur natif est empêché uniquement sur les lignes d'agents.

## Ordre des commandes

1. Ouvrir la conversation
2. Épingler ou Désépingler
3. Marquer comme lu
4. Masquer de la barre ou Afficher dans la barre
5. Arrêter
6. Relancer
7. Décommissionner

## Matrice de cycle de vie

| État | Arrêter | Relancer | Décommissionner |
|---|---|---|---|
| géré actif | disponible | indisponible | disponible |
| géré arrêté | indisponible | disponible | disponible |
| géré en transition | indisponible | indisponible | indisponible |
| non géré | indisponible | indisponible | indisponible |

Les raisons viennent de agentLifecycleEligibility. Les confirmations et appels HTTP restent ceux de SPEC-075.

## Contrat clavier

| Touche | Effet |
|---|---|
| Entrée ou Espace sur trois points | ouvre et focalise le premier item |
| Menu contextuel ou Maj+F10 sur ligne | ouvre et focalise le premier item |
| Flèche bas | item suivant |
| Flèche haut | item précédent |
| Début | premier item |
| Fin | dernier item |
| Échap | ferme et restaure le déclencheur |
| Tab ou Maj+Tab | ferme puis poursuit la navigation normale |

Les actions indisponibles restent dans le parcours des flèches avec `aria-disabled=true` et une raison annoncée, mais Entrée et Espace ne les exécutent jamais.

## Contrat de stockage

Clé: bridget.ui.agent-sidebar-preferences.v1

Structure:
- version égale à 1;
- pinned liste de noms;
- hidden liste de noms;
- readThrough objet d'horodatages.

Toute valeur invalide produit des préférences vides ou ignore seulement l'entrée invalide. Une erreur d'accès au stockage n'empêche aucune action de la session.
