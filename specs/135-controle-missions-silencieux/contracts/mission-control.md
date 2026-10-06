# Contrat — Contrôle de mission

Types de progrès admis : `worker_ack`, `task_assigned`, `source_changed`,
`result_checked`, `dependency_handoff`, `blocker_reported`.

Étapes d'une anomalie :

1. `worker` : rappel direct au responsable.
2. `coordinator` : rappel au coordinateur.
3. `escalation` : information du rôle configuré, ou demande de décision.

Si le destinataire d'une étape est injoignable, le contrôleur passe immédiatement
à l'étape suivante. Il inscrit `recipient_unreachable` dans le journal durable. Il ne
considère jamais un envoi impossible comme un rappel réussi.

Un passage envoie au plus un digest par destinataire. Le digest ne contient que
les identifiants de tâches, l'état, l'âge, l'échéance et l'action demandée.

L'anomalie liée à une action disparaît après sa prise en charge, son progrès,
sa décision de suite, sa relance canonique ou une clôture valide. Une échéance de
résultat dépassée reste indépendante d'un progrès récent. Une étape déjà
notifiée ne déclenche pas un second message. La prochaine échéance d'escalade
ou une nouvelle preuve peut ouvrir une nouvelle étape.
