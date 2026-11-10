# Implémentation 261

Le pilote relit son quota natif au repos, sur son propre processus. Cadence
bornée à trente secondes, délai de lecture de deux secondes, aucune inférence.
La réponse est publiée par le lecteur existant vers la même instance.
Les gardes de quota, de dépense, de propriétaire et de reprise restent intactes.
Le refus de reprise ajoute les seuls faits de quota de l'enfant autorisé.

Contrôles ciblés Mac : deux scénarios de transport réussis, un scénario de
projection et d'absence de fuite à une autre connexion réussi. Un premier
essai de fixture a refusé un chemin temporaire trop long ; TMPDIR=/tmp corrige
l'environnement du test. La régression de qualification160 est en cours.
Aucune batterie générale n'est lancée pour ce correctif.

Revue indépendante : tâche4c4916be-07e7-4ed0-9259-75827be1bc91,
clé bridget-261-quota-refresh-review-20261110-r1. Le patch soumis est complet ;
la vue de commit est la base245218a3cdd99a12b0eefc45b33ce0da8023d548.

Activation et reprise réelle encore à vérifier. Un ancien pilote en mémoire
n'acquiert pas ce comportement par le seul changement du lien exécutable.
La garde expirée d'une tâche ne sera ni réouverte ni modifiée directement.
