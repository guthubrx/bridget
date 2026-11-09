#257 — Refus d’ACK groupé exploitable
FR001 : après contrôle de propriété de tout le groupe, un refus task_not_ready nomme uniquement les tâches non prêtes avec task_id, ronde, état et disponibilité du rapport d’espace.
FR002 : conserver le refus atomique, sans ACK partiel, sans mutation de tâche ni effet de remise. Les tâches étrangères ou absentes restent task_unavailable sans état divulgué.
FR003 : guider la relecture et l’ACK des seuls résultats lus, sans continuation ni remplacement automatique. Anciennes rondes non acquittables après ouverture de la suivante.
FR004 : tests ciblés Mac/Linux, revue indépendante et activation0.8.24 sur les services existants, T3 ouvert ; publication, Git et nettoyage. Estimation20–35min.
