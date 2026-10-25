# SPEC145 — Consultation native des fils Bridget dans adaptateur de bureau

Date : 2026-10-07. Statut : Implemented ; 20/20 tâches vérifiées. Non installé, non activé. Branche : `session-145-bridget-thread-viewer`.

## Besoin et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-2).

L'utilisateur a autorisé la session145 et le travail isolé. La dernière session livrée est143. Les répertoires144 ne prouvent aucun développement. Les travaux142 et les processus actifs restent hors périmètre. Aucun commit, fusion, push, installation, déploiement ou redémarrage n'est autorisé par cette spécification.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-3).

## Histoires utilisateur

### US1 — Trouver les fils de l'agent sélectionné (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-6).

### US2 — Lire le contenu exact de plusieurs fils (P1)

Comme utilisateur, je veux sélectionner un fil et parcourir ses messages afin de comprendre l'échange réel.

Test indépendant : ouvrir deux fils avec des auteurs distincts, des types différents, du texte multiligne et un historique supérieur à une page. Chaque page conserve son ordre. Le texte et la copie correspondent à la source. Le changement de fil n'affiche pas l'historique du précédent.

1. Étant donné un fil accessible, quand je le sélectionne, alors son titre, ses membres et une première page bornée de messages apparaissent.
2. Étant donné un historique plus long, quand je demande la page suivante, alors une page bornée s'ajoute sans doublon ni trou dans un jeu de données stable.
3. Étant donné un message, quand je le lis ou le copie, alors son auteur, sa date, son type et son corps exact sont disponibles. Les noms absents ne sont pas inventés.
4. Étant donné une révocation d'appartenance, quand une nouvelle lecture a lieu, alors l'accès est refusé et les données désormais non autorisées disparaissent du panneau.

### US3 — Chercher et rafraîchir sans agir sur les agents (P2)

Comme utilisateur, je veux filtrer les données chargées et les rafraîchir manuellement afin de retrouver un échange sans réveiller un agent.

Test indépendant : chercher un mot présent dans une page chargée, vider la recherche, puis rafraîchir. Observer les appels et l'état Bridget avant et après. Seules les lectures humaines prévues ont lieu. Les curseurs agents et les missions restent identiques.

1. Étant donné des données chargées, quand je saisis une recherche, alors seuls ces titres, membres, auteurs et corps chargés sont filtrés localement ; la portée est indiquée.
2. Étant donné une recherche sans résultat, quand elle est appliquée, alors un état sans résultat permet de l'effacer.
3. Étant donné le bouton Rafraîchir, quand je l'utilise, alors la liste et le fil courant sont relus selon l'identité attestée actuelle, sans doublon ni émission.
4. Étant donné le panneau fermé, quand j'attends ou change de conversation, alors aucune lecture périodique ni génération de modèle n'est déclenchée par ce panneau.

### US4 — Utiliser un panneau natif fiable et accessible (P2)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-7).

Test indépendant : ouvrir l'onglet, parcourir les fils, charger une page, rafraîchir et fermer au clavier. Simuler un daemon absent, une réponse incompatible et un délai dépassé. Chaque situation affiche un état utile sans modifier les agents.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-8).

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-9).

## Limites et hypothèses

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/spec.md#extrait-10).

La recherche porte sur les pages chargées. La session n'ajoute ni composition de message, ni ACK humain, ni abonnement, ni notification, ni action de mission. La vue ne revendique pas une lecture humaine de l'ensemble d'un fil tant que toutes ses pages ne sont pas chargées.

Les assistants et modèles ne reçoivent aucun nouveau contexte par l'ouverture de cette surface. L'absence de consommation de jetons vise les appels de modèle provoqués par la consultation ; elle ne prétend pas mesurer le coût de processus indépendants déjà actifs.

Les helpers et templates SpecKit attendus sont absents du worktree et du runtime utilisateur inspectés. Les artefacts sont produits par application documentaire du protocole lu par le principal. Aucun helper exécuté ni synchronisation installée n'est prétendu.
