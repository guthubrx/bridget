# 275 : qualification des signalements tardifs

Vérification dans la nuit du 11 au 12 novembre 2026, après activation du lot 274.
Ce lot ne modifie aucun exécutable. Il ne crée ni remplaçant métier, ni nouveau
moteur, ni arrêt de T3. Les sources actives restent en version 0.8.40.

## Revues antérieures

Les revues 4320 et 4327 portent sur la base 7388a3c4 et sur le delta 272 fourni.
Les deux défauts confirmés sont déjà corrigés : le CAS conserve la cause terminale
qui a gagné, et une annulation survenue pendant la création d’un worktree rend
admission_cancelled avec le dossier et la branche conservés. Le texte de résolution
existe, sans prétendre supprimer ce dossier. Le raccord T3 atteint le puits natif
après contrôle d’identité et de projet ; aucun filtre d’opération ne bloque le
nouveau contrôle. Les tests et la limite du catalogue T3 encore ancien sont dans
la qualification 272. Les résultats de ces revues étaient déjà lus et accusés.

La revue 4344 du lot 273 était déjà traitée : le relancement persistant exclut
explicitement les missions natives, la consigne prévoit le blocage constaté et
current_task_view appelle task_view. La revue 4358 du lot 274 est la R1 déjà
traitée ; la R2 et les tests finaux sont consignés dans la qualification 274.
Aucun de ces doublons n’appelle une nouvelle compilation.

## Liaison de Mise en production

Le passage 272 arrête le maître à 22:22:25,014 Paris et recharge l’image à
22:22:27,938. Le connecteur constate la fin du primaire concerné à 22:22:30,
puis le réinscrit à 22:23:13. Le refus primary_absent tombe dans cette fenêtre.
L’éligibilité de l’enfant encore vivant ne prouve pas la présence du lien primaire
utilisé pour autoriser un nouvel appel. Le rejeu strict de la même continuation
a ensuite été admis, sans nouvelle tâche.

Le passage 273 arrête le maître à 22:59:04,784 et le recharge à 22:59:07,328.
Le refus de lecture préliminaire « daemon fermé sans réponse », signalé vers
22:59, coïncide avec cette maintenance. Son reçu indique que la commande de cet
appel n’a pas été transmise. La lecture suivante réussit. Aucun effet métier
n’est rejoué pour tester cette liaison.

## Continuations

| Tâche | Fait durable | Fin worker | Conclusion |
| --- | --- | --- | --- |
| c5667da1 | garde demandée de 1200 s ; résultat de ronde 3 à 22:42:10 | terminal à 23:02:18 | Continue vers 23:04 arrive après la garde |
| bdb1229f | aucune garde demandée ; guarded=false ; résultat à 22:47:45 | terminal à 22:47:57 | aucune ronde suivante possible sur cette commande |

Toutes les heures sont celles de Paris, le 11 novembre. Les deux résultats et
leurs dossiers sont conservés, cleanup_done=true. Le compteur rounds_remaining
indique la borne des rondes ; il n’ouvre aucune garde. Le statut continuable et
continuation expose l’éligibilité observée. Une lecture avant l’échéance ne
promet pas que la commande restera ouverte au prochain appel. Pour les missions
futures qui doivent être poursuivies, la garde se demande dès l’admission ;
reply_timeout ne la remplace pas. Aucun changement implicite du contrat existant.

## Zombies pendant une mission Linux

La tâche c1890f6c a rendu son résultat et est nettoyée. Son ancien wrapper
3694003 n’existe plus au contrôle. Aucun zombie sous ce parent n’est observé.
Quatre zombies node/git sont visibles sous un autre wrapper encore vivant.

La lecture du code et la revue indépendante 4ff22b87 confirment le contrat :
PR_SET_CHILD_SUBREAPER adopte les descendants ; la récolte a lieu après la
destruction des propriétaires Child du transport. Le drainage du runtime cumulatif
274 conserve ce parcours. Un zombie peut donc rester une entrée de processus
jusqu’à la fermeture de cette mission. Aucune panne de clôture ni consommation
CPU du programme mort n’est établie par cette observation.

Il n’existe pas de registre complet des PID détenus par des Child. Une récolte
concurrente globale ou fondée sur le nom/PGID pourrait consommer un statut encore
attendu par un transport. Une récolte en cours de mission exige un nouveau contrat
de propriété ; elle n’est pas ajoutée comme correctif improvisé. Aucun signal ni
wait depuis un processus extérieur n’est envoyé pour ce diagnostic. Les missions
autorisées continuent avec leur parcours natif de fermeture.

## Clôture

Revue indépendante en lecture seule, résultat lu et accusé. Les trois responsables
ayant des faits nouveaux ont reçu un avis information conservé sans réveil, avec
les heures, les gardes et les limites exactes. Un reçu stored n’est pas une preuve
de lecture. Les preuves privées restent dans le dossier d’incident local et les
journaux natifs ; aucun brief métier ni secret n’est ajouté à Git.

Pas de changement applicatif : pas de batterie, de build, de publication ni de
redémarrage supplémentaire. Les contrôles portent sur le code, les reçus SQLite,
les enregistrements worker, les processus et les journaux des passages précédents.
