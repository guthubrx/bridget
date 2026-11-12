# 281 : nettoyage des configurations MCP par leur propriétaire

Statut : adopté et actif en 0.8.46, le 12 novembre 2026.

La purge globale ne dispose d’aucune preuve de fin des fournisseurs, et les
wrappers distants peuvent utiliser le namespace du maître. Un nom de fichier
ne prouve pas un abandon. Retirer cette purge au démarrage et à l’entretien.
Conserver le retrait du fichier par le garde du wrapper à sa fermeture normale.
Les fichiers abandonnés après crash restent conservés tant qu’un entretien
autorisé ne peut pas prouver leur absence d’usage. Ne pas ajouter un registre
ou un nouveau protocole pour ce correctif.

Un diagnostic de démarrage reçu tardivement peut préciser une tâche déjà en
échec avec le seul code remote_start_failed, si le tuple de commande courant
est toujours validé et si la mission n’a pas été remise. Ne pas rouvrir la tâche,
ni écraser une cause précise, ni modifier l’état d’une tâche qui a travaillé.
