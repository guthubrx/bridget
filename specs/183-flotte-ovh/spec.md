# 183 — Raccordement des serveurs OVH

Statut : livraison partielle ; SYS-LE-2 disponible, KS-STOR en attente de réconciliation. Autorisation utilisateur : raccordement, essais réels, notification des trois agents, livraison et nettoyage.

## Résultat attendu
SYS-LE-2 et KS-STOR exécutent une mission native de qualification via le maître Cartae. Le projet est préparé sur le serveur choisi. Les comptes fournisseurs existants restent propres à chaque machine. Aucun travail métier Opus2D n’est rejoué.

## Correction nécessaire
Le registre global du maître contient plusieurs machines. Il ne prouve pas que le worker demandé fonctionne sur cet hôte. La préparation utilise d’abord le registre du worker local ; un autre serveur doit répondre machine_mismatch. Le registre du tunnel reste compatible lorsqu’aucun registre local n’existe.

## Validation
Tests Python sur les registres global/local, suite de préparation, compilation embarquant le helper, puis admission et résultat sur chaque machine. Les empreintes et reçus publics ne contiennent aucun secret.
