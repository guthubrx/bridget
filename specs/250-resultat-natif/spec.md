#250 — Refus des résultats manuels correctement remis

N2 a reçu outcome_unknown malgré deux refus remote_result_requires_worker observés. La réservation restait Prepared et aucun send_delivery n’existait. Le résultat natif distinct est accepté et disponible. Transformer ce refus constaté de SendIdempotent en IdempotencyResult::Rejected durable/corrélé, avec consigne de réponse finale native et lecture de la même tâche. Aucun remplacement du contratWorkerResult, aucun rejeu métier, T3 reste ouvert.
