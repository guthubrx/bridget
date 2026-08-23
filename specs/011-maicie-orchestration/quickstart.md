# Quickstart de validation — Maicie v3

Ce document décrit les scénarios à exécuter après implémentation. Les commandes
MVP existent ; le scénario d'activation exige la clôture de T023.

## Préconditions

1. Le MVP P1 ne dépend que de Bridget. Les scénarios runtime exigent la session
   008 et les profils activables exigent le SpawnOrder public session 009.
2. Bridget est lancé avec deux agents de test ; le binaire Maicie est compilé
   et sa configuration privée est disponible.
3. Les durées `courte`, `normale` et `longue` sont configurées explicitement
   dans une fixture locale.

## 1. Délégation fluide

1. Exécuter `maicie delegate --config /chemin/absolu/maicie.json --goal "résumer le risque X" --to prospective --duration courte --json`.
2. Vérifier une corrélation de demande Bridget dans le JSON, puis dans
   `maicie status --config /chemin/absolu/maicie.json`.
3. Envoyer en parallèle un message direct Bridget à Prospective.
4. Vérifier que le message est livré et qu'aucune transition Maicie n'est
   créée par ce message.
5. Faire répondre Prospective ; vérifier que la délégation passe à
   `à_évaluer`, jamais directement à `clos`.

## 2. Reprise après redémarrage

1. Créer une délégation dont la réponse est différée.
2. Interrompre l'invocation Maicie à la barrière de test puis relancer une
   commande avec la même configuration, sans arrêter Bridget.
3. Simuler un crash après acceptation Bridget mais avant retour Ack.
4. Vérifier que la délégation retrouve le même `message_id` et qu'elle consulte
   ou réessaie ce même id, sans second envoi Bridget.

## 3. État ACP honnête

1. Via l'abonnement 008, injecter activité, permission auto-décidée et `Gap`.
2. Vérifier que `maicie status --config /chemin/absolu/maicie.json --json` identifie source, subscription_id, seq,
   fraîcheur et issue de permission, sans attente humaine fictive.
3. Vérifier qu'un `Gap` produit `flux incomplet`, jamais `bloqué`.

## 4. Durées passives

1. Créer une délégation par classe de durée.
2. Vérifier trois timeouts Bridget distincts.
3. Vérifier qu'aucun timer ou relance Maicie n'est créé.
4. Vérifier qu'une issue Bridget ou une consultation utilisateur est nécessaire
   pour modifier la vue de coordination.

## 5. Profil inactif

1. Déclarer un profil Sentry avec la capacité `sécurité-infrastructure`.
2. Confier un objectif correspondant sans agent sécurité connecté.
3. Vérifier que Maicie propose l'activation sans lancer de processus.
4. Refuser, puis recommencer et approuver avec une approbation non expirée.
5. Vérifier que seul SpawnOrder 009 est émis, avec les hashes de profil/contexte
   approuvés, et qu'aucun processus n'est créé par Maicie.
6. Simuler les crashs avant socket, après écriture avant Ack et après Ack avant
   commit ; vérifier le replay exact du même `SpawnOrder` et du même
   `command_id`, qui constitue le lookup idempotent puisqu'aucune surface
   `SpawnLookup` n'existe. `IdempotencyExpired` est terminal et n'est jamais
   rejoué. Le digest retourné par `SpawnAccepted` est comparé au hash épinglé
   dans l'approbation, sans relecture du registre, et aucun double ordre ne doit
   apparaître.

## Résultat attendu

Tous les scénarios sont reproductibles par CLI et JSON. Ils ne requièrent ni
GUI, ni TUI, ni DSH, ni T3 Code.
