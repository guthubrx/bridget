# Plan

Android seulement : réutiliser /v1/agents, /v1/tasks et /v1/tasks/{id}.
Préférences chiffrées liées à l'association : 50 identifiants récents ordonnés,
migration depuis la conversation sélectionnée. Pas de copie de corps de message.
Lecture groupée des favoris/récents par ids (lots de 50), bornée à 100 contacts.
Tri O(n log n), index O(n). Aucune lecture réseau par ligne.
Lecture des délégués à l'ouverture seulement, pagination native et filtrage sur
parent_agent_id, sans inférence d'identité. Présenter les titres de tâches.

Publication Android 0.8.53/8053 réservée. Ne pas faire passer la publication
Android pour une activation des corrections serveur encore en attente.
