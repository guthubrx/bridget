# Qualification235 — en cours

Incident du5novembre2026, demandes session206-memoire-24go-sys-20261105-01/02.
Aucun rejeu métier ni admission de diagnostic.

## Faits acquis

- Sonde SYS terminée en6,7–6,8s ; réponse maître worker_unavailable.
- Le worker ne redémarrait pas : une reconnexion changeait son époque.
- Diagnostic natif fermé : WorkerRejected/command_mismatch reçu toutes les
  33–35s ; aucune cause réseau, compte ou version fournisseur n’est établie.
- Les335 reçus SYS correspondent aux tâches, commandes, placements et liens
  durables. Contrôle en lecture seule, sélection dans l’ordre natif incluse.
- Lecteur Rust exact sur la base maître :1959 commandes,0erreur,0,11s.
  L’hypothèse d’une ligne illisible empoisonnant la collection est écartée.
- Les huit slots SYS sans disposition ne contiennent aucun résultat à rejouer.
- Le lecteur actuel relit1959 commandes pour chaque observation : coût confirmé
  par lecture, mais lien causal avec le refus non établi. Ne pas l’annoncer corrigé.

## Activation de diagnostic seulement

SYS :235-channel-observe-r1/source7082049, PID2243992, SHA
c6b6c07f9baed464ffc80d535ec1a4ff5f85099f42e09354e9cc934c68c21188.
Version0.8.2 ; retour234-r3 compatible et pointeur atomique qualifiés.
Deux relances natives SYS sans commande active ; aucun processus métier arrêté.
Les autres composants restent234-r3 à ce stade.
Un diagnostic maître limité aux identifiants du refus est en préparation.

## Reçus et preuves privées

/Users/moi/.cache/bridget235-probe-connection/
SYS :/home/moi/.cache/bridget235-probe-connection/
Avis de courte maintenance au propriétaire :mcp-37511-6aecda4a-1,
remise642a6fea-e98b-4c60-9135-a0b402b182d7,in_flight ; lecture non attestée.
Aucun avis « corrigé » envoyé. La clé01 reste refusée durablement sans tâche.
La clé02 reste non enregistrée ; le propriétaire attend l’avis de correction.

## Cause et correctif candidat

Le diagnostic maître activé à19:37UTC identifie la commande de préparation
2fb96ec5-07ca-404e-a322-7cd359626b85,génération0,instance vide. Il s’agit d’une
ancienne revue Bridget cancelled, sans enfant ni workspace publié, avec liaison
de conservation durable. Le refus se produit hors attente corrélée. La réponse
tardive d’entretien ne doit ni clore une autre demande ni fermer la connexion.
Le code conserve la validation de l’époque et du worker, la capacité négociée,
la liaison exacte de la vue et les gardes terminal/continuation/réservation.
Une attente présente reçoit toujours sa réponse ; aucune clôture n’est inventée.

Les observations relisaient jusqu’à trois fois1959 commandes sous le verrou
partagé pour chacun des335 reçus SYS. Les lectures sont désormais bornées à la
clé primaire ou filtrées par nom/instance avant décodage ; même ordre et mêmes
contrôles de placement et de lien parent. La réconciliation générale reste
stricte et inchangée.

Mesure native sur la même base maître, lecture seule,20commandes SYS :
lectures complètes2236635µs ; deux lectures ciblées par commande44737µs.
Rapport49,997, environ50fois ; aucun chiffre de latence d’admission déduit.
Les1959commandes sont inchangées. Benchmark temporaire hors code livré.
Trois contrôles235 passent : projection fermée, lecture ciblée/identités et
refus tardif sans mutation/avec gardes. Deux premiers essais ont échoué sur des
hypothèses de fixture (scope trop court, stamps puis réservation immuable),
corrigées après lecture des faits de sauvegarde ; aucun défaut de production
masqué. Les quatre régressions utiles passent : conservation exacte, réponse de préparation tardive/corrélation et deux transactions atomiques de spawn.
