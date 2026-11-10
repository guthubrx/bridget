# Modèle des données 258

La tâche, sa ronde, son enfant, sa présence et les reçus restent natifs Bridget.
La projection T3 reste une vue ; ses identifiants de rendu ne deviennent pas
des identifiants d'exécution.

La boîte conserve son brouillon dans le store existant. Une intervention porte
une clé de commande, la tâche et le contexte racine. Le reçu distingue refus,
effet confirmé et issue inconnue. Une sélection modèle/effort en attente ne
remplace pas la sélection constatée. Les capacités bornent les actions visibles.
Le protocole fermé vérifie corps, références et versions avant toute mutation.

L'arbitrage consigné dans reuse-audit.md ajoute une table de reçus dans le
DelegationStore existant : clé UUID globale, racine, tâche, canon de requête et
réponse fermée. La réservation conserve une issue inconnue avant tout effet.
Un rejeu lit le reçu et les accusés existants, sans nouvelle remise. La purge
de l'arbre retire ces reçus avec les autres preuves de commande devenues
purgeables. Les payloads de tâche et leurs résultats restent inchangés.
