# Contrat de frontière entre Maicie et Bridget

## Deux vérités

### Maicie fait autorité sur

- objectifs ;
- délégations ;
- dépendances ;
- décisions ;
- évaluations ;
- clôtures ;
- politiques et budgets attachés à la mission.

### Bridget fait autorité sur

- présence observée ;
- instances et générations ;
- parc et liens d'agents ;
- soumissions et livraisons ;
- files et exécutions ;
- sessions, tours et capacités fournisseurs ;
- consommation et usage observés.

## Échanges Maicie vers Bridget

Maicie peut demander une soumission en fournissant :

- objectif et délégation ;
- destinataire ou critère déjà résolu selon ses règles ;
- intention d'exécution ;
- contenu canonique ;
- échéances et politique ;
- identifiant idempotent.

Bridget retourne une décision durable et les références opaques créées.

## Échanges Bridget vers Maicie

Maicie consomme un flux cursored de faits d'exécution. Chaque observation rend :

- curseur ;
- source et génération ;
- fraîcheur ;
- état runtime ;
- raison et références ;
- usage éventuel ;
- gap ou indisponibilité explicites.

## Interdictions

- Bridget ne ferme pas un objectif.
- Maicie ne tue, ne reprend et ne supervise pas directement un processus.
- Bridget n'ouvre pas la base privée Maicie.
- Maicie n'ouvre pas `bridget.db`.
- Une projection périmée ne remplace aucune vérité durable.
- Une fin technique n'est pas un verdict métier.
- Une approbation technique fournisseur n'est pas une approbation de profil
  Maicie et réciproquement.

## Projection publique de mission

Les vues Bridget qui ont besoin du contexte Maicie lisent une projection :

- versionnée ;
- produite atomiquement ;
- en lecture seule ;
- datée ;
- sans secret ni droit implicite ;
- tolérante à l'absence ou au retard.

La projection contient uniquement les informations nécessaires à l'affichage
ou à la carte de reprise, jamais les tables privées complètes.
