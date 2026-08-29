# Contrat de capacités fournisseur

## Objectif

Permettre à Bridget de choisir une commande compatible avec la session
réellement exécutée et de rendre tout repli visible.

## Preuve d'identité

Une session fournisseur publie :

- type de fournisseur ;
- chemin de binaire résolu ;
- version observée ;
- empreinte de release ou du binaire si disponible ;
- version de contrat testée ;
- date d'observation.

## Capacités

Chaque capacité possède :

- nom stable ;
- état `Supported`, `Unsupported`, `Experimental` ou `Unknown` ;
- source `Negotiated`, `VersionContract` ou `Observed` ;
- restrictions éventuelles, par exemple tour normal uniquement ;
- stratégie de repli autorisée ;
- date et révision.

## Règles

1. `Unknown` ne vaut jamais `Supported`.
2. Une capacité expérimentale exige une activation explicite.
3. Une restriction de type de tour est vérifiée avant la commande.
4. Un refus runtime met à jour l'observation, mais ne réécrit pas le contrat de
   version sans réconciliation.
5. Les octets du refus fournisseur sont conservés avec la projection canonique.
6. Un changement de version invalide les observations de la session précédente.

## Matrice minimale

| Capacité | Repli permis |
|---|---|
| `start_thread` | Aucun si le fournisseur ne peut pas démarrer |
| `resume_thread` | Reconstruction explicite |
| `fork_thread` | Nouveau thread avec ascendance Bridget, sans ascendance native, déclaré |
| `start_turn` | File en attente ou refus |
| `steer_turn` | Interruption puis nouveau tour si politique autorisée |
| `interrupt_turn` | Annulation de session ou refus, jamais kill implicite |
| `client_message_correlation` | Issue indéterminée ou repli, jamais faux acquittement |
| `structured_approval` | Politique de refus ou attente externe déclarée |
| `usage_reporting` | Usage inconnu, jamais zéro inventé |
