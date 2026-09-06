# Plan 091

Complément : rendu dans attach (équipier Terra), contrôle du modèle dans le
protocole/daemon/wrapper/pilote (principal). Pas d'édition simultanée d'attach :
la commande `/model` sera raccordée après livraison du rendu. Le Codex installé
0.153.4 expose `thread/settings/update` après négociation `experimentalApi`.
Validation par `model/list` borné puis mise à jour du fil NATIF : aucun double
état Bridget, aucun `turn/start` vide, aucun changement de sandbox/approbation.
La notification `thread/settings/updated` atteste le réglage dans l'annuaire ;
elle ne prouve pas encore une inférence exécutée avec ce modèle. La recette HTTP
privée vérifie séparément le modèle et l'effort effectivement employés au tour suivant.
Canal de contrôle attach existant, corrélation
opaque générée par daemon et liée à la connexion wrapper, réponse bornée.
Les scripts SpecKit et mémoires projet sont absents de ce dépôt extrait : le
workflow est appliqué à ces artefacts sans réinstaller la mécanique globale.

Trois lots disjoints : lancement autorisé (protocole/daemon/registre/CLI), communication et corrélation (wrapper/MCP), observation (attach). Worktrees séparés puis intégration validée. Les droits sont vérifiés à l'autorité, jamais par une instruction au modèle. Toute intervention humaine imposée par un garde reste humaine.

Attach réutilise ListAgents pour sa ligne de statut, sans nouveau canal sémantique ni lecture SQLite. Requête bornée indépendante de la lecture du journal ; arrêt du lecteur de statut à la fermeture. Rendu sur le thread déjà propriétaire du terminal. Hors terminal : aucune ligne de statut périodique.

La documentation et les tests doivent refléter le comportement réel. Le défaut de transmission MCP et la perte de corrélation se diagnostiquent sur les trames avant modification. Les permissions de développement sont définies dans le lot lancement et testées sans modifier la configuration globale de production.
