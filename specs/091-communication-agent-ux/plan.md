# Plan 091

Trois lots disjoints : lancement autorisé (protocole/daemon/registre/CLI), communication et corrélation (wrapper/MCP), observation (attach). Worktrees séparés puis intégration validée. Les droits sont vérifiés à l'autorité, jamais par une instruction au modèle. Toute intervention humaine imposée par un garde reste humaine.

Attach réutilise ListAgents pour sa ligne de statut, sans nouveau canal sémantique ni lecture SQLite. Requête bornée indépendante de la lecture du journal ; arrêt du lecteur de statut à la fermeture. Rendu sur le thread déjà propriétaire du terminal. Hors terminal : aucune ligne de statut périodique.

La documentation et les tests doivent refléter le comportement réel. Le défaut de transmission MCP et la perte de corrélation se diagnostiquent sur les trames avant modification. Les permissions de développement sont définies dans le lot lancement et testées sans modifier la configuration globale de production.
