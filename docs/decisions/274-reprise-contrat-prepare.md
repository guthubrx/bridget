# 274 : reprendre le contexte effectivement préparé

Statut : correction validée en cours de livraison.

Une sonde de routage porte le dépôt source. La préparation d’un worktree ou
d’une vue de commit adapte ensuite les chemins de la définition et de la
politique enfant au dossier isolé. Ces transformations changent leurs empreintes.
La reprise d’un démarrage refusé comparait encore ces empreintes à la sonde et
pouvait produire routing_connection_changed sans modification de droits.

Pour un dossier isolé dont la préparation est attestée dans la tâche, la reprise
valide la définition canonique conservée et l’empreinte de la politique enfant
contre le placement préparé. Elle conserve les axes figés du routage et les
vérifications fraîches du worker lors de Prepare et Start. Sans dossier isolé,
la comparaison à la sonde reste obligatoire, avec la normalisation de l’effort.
Une préparation absente, une politique absente ou altérée et une définition
altérée restent des refus. Aucun format de stockage ne change.

La correction s’applique aux futurs parcours de reprise. Elle ne relance pas
les tâches annulées, ne remplace pas de mission et ne modifie pas les permissions.
