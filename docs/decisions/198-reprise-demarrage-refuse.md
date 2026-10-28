# ADR198 — Reprendre un démarrage refusé sans doubler la mission

Statut : proposé, qualification en cours.

## Contexte
Une admission distante peut persister sa tâche et son dossier, puis recevoir ResourceLimit avant qu’un reçu de commande soit enregistré. Le rejeu restitue alors indéfiniment starting. Le jeton de préparation et le bail300s peuvent expirer. Prolonger silencieusement le canon ou créer une nouvelle mission ferait perdre les garanties d’identité et d’unicité.

## Décision
Le propriétaire rejoue le canon original de l’admission. Un refus initial éligible ouvre un plan durable sur la même tâche. L’ancienne commande doit être fermée durablement avant naissance par le worker authentifié, sous son contrôle sérialisé. Tout reçu antérieur non dédié à cette clôture, processus retenu ou état incertain bloque la reprise. Une absence de fichier ne vaut jamais Terminal.

Après cette preuve et une nouvelle vérification des autorisations, une tentative native avec identifiant de commande distinct peut être réservée. La tâche, sa clé, son enfant logique, sa mission, sa ronde et son dossier restent les mêmes. Les anciens canons, identités et reçus sont conservés. Le worktree existant est réattesté en lecture seule. Le fournisseur ne reçoit aucun travail avant les gardes de libération habituelles.

Le plan est enregistré avant la première opération distante et avant l’arrêt logique de l’ancien bail. La surveillance ne clôt ni ne relance une tâche portant une reprise en cours. Les réponses perdues retrouvent ce plan ; elles ne fabriquent pas une autre tentative. Un refus de capacité du chemin de clôture laisse ce même plan en attente.

Les nouveaux échanges sont négociés. Les anciens pairs refusent sans effet. Les causes de capacité sont fermées et bornées ; les erreurs historiques sans preuve restent inconnues.

## Conséquences
La reprise exige une demande actuelle du propriétaire, des droits équivalents et les mêmes contraintes de cible. Elle ne s’applique pas à un démarrage ou une libération d’issue inconnue. Une nouvelle commande n’est pas une nouvelle mission et n’est admise qu’après clôture vérifiée de la précédente.

Ce contrat ajoute des états durables qui doivent être testés aux coupures, en concurrence avec l’annulation et après reconnexion. Il n’autorise ni un changement manuel du store, ni un remplacement de tâche, ni une activation hors maintenance admise.
