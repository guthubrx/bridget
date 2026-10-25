# 183 — Raccordement des serveurs OVH

Statut : livraison partielle ; SYS-LE-2 disponible, KS-STOR en attente de réconciliation. Autorisation utilisateur : raccordement, essais réels, notification des trois agents, livraison et nettoyage.

## Résultat attendu
SYS-LE-2 et KS-STOR exécutent une mission native de qualification via le maître Cartae. Le projet est préparé sur le serveur choisi. Les comptes fournisseurs existants restent propres à chaque machine. Aucun travail métier Opus2D n’est rejoué.

## Correction nécessaire
Le registre global du maître contient plusieurs machines. Il ne prouve pas que le worker demandé fonctionne sur cet hôte. La préparation utilise d’abord le registre du worker local ; un autre serveur doit répondre machine_mismatch. Le registre du tunnel reste compatible lorsqu’aucun registre local n’existe.

## Validation
Tests Python sur les registres global/local, suite de préparation, compilation embarquant le helper, puis admission et résultat sur chaque machine. Les empreintes et reçus publics ne contiennent aucun secret.

## Correction de reprise — admission choisie sans tâche

Le reçu KS-STOR reste `chosen` dans l'époque courante après un délai de préparation distante. Le rejeu ne fait que lire `admission_pending`, tandis que la reprise périodique exige une époque précédente. Cette combinaison bloque aussi la maintenance.

Un rejeu explicite de la même clé et de la même enveloppe doit reprendre un reçu V2 `chosen` sans nouvelle décision. Il conserve le candidat, le modèle, l'effort et la base. Une exclusion temporaire par propriétaire et clé couvre le parcours choisi ; un appel concurrent reçoit `admission_pending`. Le parcours relit l'état durable après acquisition et garde les contrôles existants de source, de droits et de connexion. Un refus durable ne se rouvre pas.

Aucun changement de protocole, de schéma durable, de garde de maintenance ni de commande administrative. La reprise périodique des anciennes époques reste inchangée. L'ancienne version active ne peut pas recevoir cette correction sans activation ; ce blocage de livraison reste distinct de la preuve par tests.
