# Diagnostic 268 : journaux et observations

Les signalements du 11 novembre sont rapprochés des états durables sans rejouer les missions.

- Les tâches b8a3bce2, 57e85b90 et 0cd79968 sont cancelled, cleanup_done et garde libérée. La liste T3 conserve une projection working/seq708, car la synchronisation du lineage est portée par la lecture ou le suivi du parent visible. Cette projection ne remplace pas l’état durable de la tâche. Aucun changement des tâches ni relancement.
- Les quatre agents refusés AgentUnknown correspondent à des tâches result_available et cleanup_done. Le daemon ne cherchait que la présence volatile, perdue après disparition du routeur. Hypothèse vérifiable : le refus peut être qualifié depuis la tâche durable après arrêt confirmé, parce que la présence en mémoire ne conserve pas tout l’historique.
- La coupure Vie des habitants coïncide avec le chargement de 267 : core à 14:37:44Z, connecteur Mac renouvelé vers 14:38, lien de ce responsable réinscrit à 14:39:02Z. Une seule tâche fd4460c8 existe pour la clé rejouée strictement, encore working. Aucun droit changé ni mission doublée.
- La copie privée de fca64289 contient la base 1892791d demandée, pas les deux autres commits cités dans le corps. Le placement worktree transfère sa base, pas les objets arbitraires cités dans le texte, les réserves externes ou les alias SSH. SYS n’a plus que 4 790 724 Kio disponibles au relevé. Son propriétaire doit fournir les prérequis autorisés ; aucune mission de remplacement ici.
- Il n’existe pas de mode auto pour send. Le parcours entre responsables est queue ; steer exige filiation ou mandat de boucle spécifique. Les reçus de diagnostic mcp-86189-6af482ad-10 et mcp-86189-6af482af-11 sont conservés sans réveil, pas présumés lus.

Périmètre du correctif : diagnostic du journal après disparition des présences. Conserver les gardes d’accès, les tâches actives, les codes du protocole et les résultats. Ne pas qualifier un enfant working ou non nettoyé d’arrêté. Une route active reste prioritaire sur l’historique. Pas de redémarrage de T3.

Le test exécuté avant correction échoue sur AgentUnknown au lieu de AgentStopped, puis passe avec la recherche durable. Le premier passage sans test après copie interrompue ne compte pas comme preuve ; le chemin temporaire macOS a aussi dû être raccourci pour la socket de la fixture.

Revue indépendante acd3a93d-992e-4653-a31d-0b589a5055e0 : accord sur le diagnostic sous réserve de ne pas prétendre que le résultat est livré. Le message existant ne le prétend pas. L’objection sur les homonymes est écartée par l’index UNIQUE native_delegations_child, déjà présent dans la source cumulative, main et la base active. Le test est complété pour vérifier ce refus d’homonyme et l’absence de faux arrêt en cas d’erreur de lecture du magasin. Le résultat est lu, ACK et la garde propriétaire close ; vue conservée.

Le contrôleur n’est plus routable lors du retour mcp-86189-6af483aa-12 (unknown_recipient, effet non confirmé). Ce retour n’est pas présumé reçu et n’est pas renvoyé par une autre route. Son diagnostic est conservé ici et sera partagé au propriétaire racine encore connecté avec le bilan.
