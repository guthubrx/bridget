# Session 259 — Guidage des erreurs avant envoi MCP

Statut : In Progress. Correction autorisée par le mandat permanent de maintenance, incident Roadie du 10 novembre 2026. Base 065f54f05d26224b0e31198a4db48ca208232da7, après 258 ; dépendances 151, 196 et 230.

Le parent a lu le résultat de la tâche 890203d2-7fbc-4ec6-8ff0-3cb070ccdab5. L'appel ACK a rendu uniquement une erreur de délai JSONL. La lecture technique ciblée de l'état durable, sur la tâche et son propriétaire signalés, trouve acknowledged_at=null, ronde1/result_available. Aucun ACK rejoué.

FR01 : une erreur de transport au raccordement ou à la lecture du plafond MCP indique l'étape constatée et que la commande de cet appel n'a pas été transmise. Conserver le code technique, les références et les reçus antérieurs.
FR02 : aucune nouvelle tentative, commande ou connexion après l'échec du contrôle préalable. Aucun ACK de tâche ni changement de droits provoqué par le guidage.
FR03 : refus natifs et issue inconnue après une transmission gardent leurs contrats actuels. Le nouveau texte ne s'applique qu'aux deux échanges préalables concernés.
FR04 : intégrer, pousser, distribuer/activer les services et vérifier leur retour compatible. Préserver les tâches et les applications ouvertes ; les façades déjà chargées restent à distinguer du programme installé.

Cause précise du retard non établie : ne pas inventer une panne de réseau, de droits ou de compte. Aucun travail Roadie modifié. Pas de nouvelle mission métier ou rejeu d'ACK.
