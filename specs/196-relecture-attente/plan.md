# Plan
1. Envelopper le parcours MCP existant uniquement pour task_wait et un échec de transport daemon_unreachable.
2. Une relecture1s réutilise le parcours complet, jamais une permission mémorisée. Aucun constructeur de tâche, changement d’hôte ou modèle.
3. Épreuves IPC avant/après, gardes existantes, revue indépendante, livraison Git/candidat et état d’activation explicite.
Complexité : au plus deux parcours de lecture sur le même lot de16tâches au maximum ; aucun stockage, dépendance ou boucle de sondage.
