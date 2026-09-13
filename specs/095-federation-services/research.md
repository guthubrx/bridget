# Réemploi 095

- Montage historique prouvé : maître Mac, SSH inverse -R, Cartae client-only. Source historique scripts/federate-ssh.sh et recette specs/002-federation-ssh/analyze.md.
- Régression : commit 9066f10a du 05/09/2026 retire install/status/remove et launchd ; run sécurisé reste présent.
- Installation historique supprimait la socket distante avant bind : non réutilisable telle quelle, car un daemon Linux vivant utilise aujourd'hui ce chemin.
- Choix : nouvelle racine privée pour la socket cliente, désactivation explicite de l'ancienne autorité après sauvegarde, nouveau service installé depuis le paquet actuel. Pas de dépendance sur com.bridget.federation.cartae historique.
- launchd KeepAlive/RunAtLoad et systemd utilisateur Restart contrôlent le processus au premier plan. La persistance Linux hors connexion dépend de linger ; constater, ne pas modifier une politique privilégiée implicitement.
- Les trois commandes administratives restent dans le script actuel, évitant une nouvelle commande Rust qui ne ferait que relayer le shell.
