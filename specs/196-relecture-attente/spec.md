# Session196 — Récupération bornée de lecture d’attente

Correction autorisée du signalement trajets bus mcp-68740-6ae1887a-4. Attente45s sur9cc87183, ronde2, interrompue par délai JSONL ; résultat stable808octets disponible ensuite. Le délai exact a expiré à une étape non journalisée ; aucune saturation de taille établie.

Après une indisponibilité de transport de task_wait, permettre une seule relecture native des mêmes task_ids avec attente1s et identité inchangée. Refaire tous les contrôles, sans cache de droits. Rendre le constat et un marqueur de récupération. Refus métier/authentification, mutations, autres outils et appels interrompus par l’hôte ne doivent pas être rejoués. Si la relecture échoue techniquement, fournir un guidage fermé vers task_status, sans annulation ou nouvelle mission. Aucun seuil transport augmenté.

Tester les vraies trames IPC : coupure au plafond, coupure après envoi Wait, même lot et relecture bornée ; refus du nouveau contrôle conservé. La correction concerne le client commun de toutes plateformes, sans présumer le chargement d’un ancien processus MCP.
