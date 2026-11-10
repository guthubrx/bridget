# Session 260 - Assainissement du code et du dépôt

Statut : In Progress (artefacts prêts, implémentation non commencée)
Créé le : 10 novembre 2026
Dépendances : aucune session ouverte n'est requise. Les sessions 149 à 259 fournissent le code mesuré. Le programme ne modifie aucune de leurs décisions.
Mesures : relevées sur le commit `065f54f0`, revérifiées sur `b3949c93` (aucun fichier Rust modifié entre `406466cb` et `b3949c93`).

## Demande initiale et cadrage

Demande initiale (François, 10 novembre 2026) : « Je veux que tu fasses toutes les spécifications sans aller jusqu'à l'implémentation pour corriger tout ce qu'il y a à corriger, modifier tout ce qui a à corriger, nettoyer tout ce qui a nettoyé. Je répète, tu fais tous les artefacts speckit, mais tu ne fais pas l'implémentation, celle ci sera déléguée à un modèle beaucoup moins onéreux mais moins intelligent, donc il faut que tu lui paves le chemin pour qu'il n'aie qu'à suivre des consignes. Il est pas débile non plus ».

Demande reformulée : produire un programme d'assainissement complet de Bridget, découpé en 17 lots indépendants. Chaque lot corrige une famille de défauts constatée par l'analyse du 10 novembre 2026. Chaque tâche donne le fichier, le symbole à chercher, la commande exacte, le résultat attendu et la règle d'arrêt. Un agent moins capable doit pouvoir exécuter le programme sans redécouvrir le dépôt et sans prendre de décision d'architecture.

Apport : la formulation initiale demande « tout corriger ». La reformulation ajoute trois garanties. Premièrement, un filet de sécurité arrive avant tout changement de code. Deuxièmement, chaque lot se prouve par une comparaison avant/après, pas par une impression. Troisièmement, chaque choix qui revient à François est isolé dans un tableau de décisions avec une action par défaut.

Exclusion explicite de ce tour : aucune implémentation. Les phases Implement, Converge et Audit de code du pipeline ne s'appliquent pas.

Vocabulaire utile :

- Ligne de base (baseline) : résultat mesuré avant un changement. Il sert de point de comparaison.
- Comportement constant : le programme fait exactement la même chose avant et après. Seule la forme du code change.
- Porte de décision : question réservée à François. Elle porte un numéro `D1` à `D16` dans `decisions.md`.
- Analyse statique (lint) : contrôle automatique du code sans l'exécuter. Pour Rust, l'outil est clippy.

## Acteurs

- François : propriétaire du produit. Il tranche les portes de décision.
- L'implémenteur : agent qui exécute `tasks.md`. Il suit `contracts/regles-implementeur.md`.
- Le mainteneur Bridget : responsable de l'activation des versions sur les machines.
- Le contributeur futur : toute personne ou tout agent qui modifie Bridget après le programme.

## User Scenarios & Testing

### US1 - Filet de sécurité automatique (P1)

Le contributeur futur pousse un changement Rust. Un contrôle automatique compile le code, lance l'analyse statique et signale les dépendances vulnérables. Aujourd'hui, aucun contrôle Rust ne tourne sur le dépôt distant.

Test indépendant : pousser une branche avec une erreur d'analyse volontaire. Le contrôle doit échouer.

Scénarios d'acceptation :

1. Étant donné un commit qui introduit un avertissement d'analyse statique, quand le contrôle tourne, alors il échoue et nomme la règle.
2. Étant donné le dépôt à jour, quand un développeur lance la commande locale unique de contrôle, alors elle se termine sans erreur.
3. Étant donné une dépendance avec une alerte de sécurité publiée, quand le contrôle hebdomadaire tourne, alors il la signale.

### US2 - Dépôt sans résidus (P1)

Le contributeur futur liste les fichiers suivis. Il ne trouve plus de fichiers de travail oubliés à la racine. Les fichiers suivis malgré une règle d'exclusion sont expliqués.

Test indépendant : la commande d'inventaire donne une liste dont chaque entrée est couverte par une exception écrite.

Scénarios d'acceptation :

1. Étant donné la racine du dépôt, quand on liste les fichiers suivis, alors aucun fichier de mesure ou de reprise temporaire n'y figure.
2. Étant donné un fichier suivi et exclu, quand on lit le fichier d'exclusions, alors un commentaire explique cette exception.

### US3 - Consignes d'agent lisibles (P1)

Un agent qui ouvre le dépôt lit les règles permanentes en moins de 20 Ko. Les reçus de session restent disponibles dans un fichier séparé, à l'identique.

Test indépendant : comparer le texte déplacé avec l'original, octet pour octet.

Scénarios d'acceptation :

1. Étant donné le fichier de consignes, quand un agent le charge, alors il ne contient que des règles permanentes et un renvoi vers les reçus.
2. Étant donné une consigne vivante qui n'existait que dans un reçu, quand on lit les règles permanentes, alors elle y figure.

### US4 - Tests hors des fichiers de production (P1)

Le contributeur futur ouvre un fichier de production. Il n'y trouve plus des dizaines de milliers de lignes de tests. Les tests existent toujours, dans des fichiers voisins.

Test indépendant : la liste triée des noms de tests est identique avant et après.

Scénarios d'acceptation :

1. Étant donné les 16 fichiers concernés, quand le lot est terminé, alors aucun ne contient un module de tests en ligne de plus de 200 lignes, sauf refus documenté.
2. Étant donné la suite complète, quand elle tourne après le lot, alors la liste des tests en échec est identique à la ligne de base.

### US5 - Aiguillage de messages découpé (P1)

Le contributeur futur cherche le traitement d'un message. Il trouve une fonction nommée par message, au lieu d'une fonction unique de 5011 lignes.

Test indépendant : la fonction d'aiguillage mesure moins de 1500 lignes et la suite complète donne les mêmes résultats.

Scénarios d'acceptation :

1. Étant donné une branche d'au moins 30 lignes, quand le lot est terminé, alors son corps est une fonction nommée et la branche ne contient qu'un appel.
2. Étant donné un message quelconque, quand le daemon le traite, alors la réponse est identique à celle d'avant le lot.

### US6 - Blocs `unsafe` justifiés (P2)

Le relecteur lit un bloc `unsafe`. Un commentaire dit pourquoi le bloc est sûr. Les appels répétés passent par une fonction sûre partagée.

Test indépendant : le contrôle d'analyse refuse un bloc `unsafe` sans commentaire.

Scénarios d'acceptation :

1. Étant donné les 247 blocs `unsafe`, quand le lot est terminé, alors chacun porte un commentaire de sûreté.
2. Étant donné un nouveau bloc sans commentaire, quand l'analyse tourne, alors elle échoue.

### US7 - Concurrence observable (P2)

Le mainteneur veut savoir si la boucle périodique bloque le daemon, combien de connexions sont ouvertes et si un verrou a été récupéré après une panique. Aujourd'hui, ces trois faits sont invisibles.

Test indépendant : un test force une panique sous verrou. Le compteur de récupération augmente et une ligne de journal apparaît.

Scénarios d'acceptation :

1. Étant donné un verrou empoisonné, quand un fil le reprend, alors le daemon continue comme avant et consigne le fait une fois.
2. Étant donné une connexion acceptée puis fermée, quand on lit le compteur, alors il a augmenté puis diminué.
3. Étant donné la boucle périodique, quand un tour dépasse le seuil, alors sa durée est consignée.

### US8 - Erreurs comparées sans fragilité (P2)

Le contributeur futur modifie un message d'erreur. Le comportement du programme ne change pas par accident, parce que le texte comparé et le texte produit viennent de la même constante.

Test indépendant : l'inventaire classe chaque comparaison de texte. Aucune comparaison de classe A ne reste avec un texte écrit deux fois.

Scénarios d'acceptation :

1. Étant donné une comparaison sur un texte produit par Bridget, quand le lot est terminé, alors producteur et consommateur utilisent la même constante.
2. Étant donné une comparaison sur un texte d'un programme externe, quand on lit le code, alors un commentaire nomme la source du texte.

### US9 - Fonctions lisibles (P3)

Le contributeur futur lit une fonction sans faire défiler mille lignes. Il appelle une fonction sans compter quinze arguments de même type.

Test indépendant : le relevé des longueurs et des nombres de paramètres après le lot.

Scénarios d'acceptation :

1. Étant donné les 9 fonctions à 15 paramètres ou plus, quand le lot est terminé, alors chacune reçoit une structure d'arguments nommés.
2. Étant donné les 43 fonctions de plus de 300 lignes, quand le lot est terminé, alors chacune est sous 300 lignes, exemptée par écrit, ou arrêtée avec un motif consigné.

### US10 - Format uniforme (P2)

Le contributeur futur lance le formateur. Il ne produit plus un diff de 120 fichiers sans rapport avec son travail.

Test indépendant : le contrôle de format se termine sans différence.

Scénarios d'acceptation :

1. Étant donné le dépôt après le lot, quand le contrôle de format tourne, alors il ne signale aucun fichier.
2. Étant donné l'historique, quand on cherche l'auteur d'une ligne, alors le commit de format peut être ignoré.

### US11 - Modules nommés par leur rôle (P2)

Le contributeur futur lit un nom de module. Le nom dit ce que fait le module, pas dans quelle session il est né.

Test indépendant : aucun fichier Rust de production ne porte un suffixe de numéro de session dans les 18 cas listés.

Scénarios d'acceptation :

1. Étant donné les 18 modules listés, quand le lot est terminé, alors chacun porte le nom de la table de correspondance.
2. Étant donné une chaîne de caractères durable qui contient un ancien numéro, quand le lot est terminé, alors elle est inchangée.

### US12 - Stockage documenté (P3)

Le mainteneur veut savoir quelles bases existent, comment leur version est suivie et quels réglages de durabilité elles utilisent. Il lit un inventaire et une décision d'architecture proposée.

Test indépendant : chaque ouverture de base du code figure dans l'inventaire.

Scénarios d'acceptation :

1. Étant donné une base du produit, quand on lit l'inventaire, alors on y trouve son fichier, son mécanisme de version et ses réglages.
2. Étant donné la proposition, quand François la lit, alors il peut choisir sans lire le code.

### US13 - Scripts contrôlés (P2)

Le contributeur futur modifie un script. Un contrôle automatique signale les erreurs courantes. Les scripts vivants s'arrêtent à la première erreur non prévue.

Test indépendant : les contrôles de scripts passent localement et sur le dépôt distant.

Scénarios d'acceptation :

1. Étant donné un script Python avec un import inutilisé, quand le contrôle tourne, alors il échoue.
2. Étant donné l'exécution de code fourni par le daemon dans le script sentinelle, quand on lit le code, alors un commentaire énonce l'invariant de confiance.

### US14 - Tests ignorés expliqués (P2)

Le contributeur futur voit un test ignoré. Il lit pourquoi.

Test indépendant : aucun attribut d'exclusion de test sans motif.

Scénarios d'acceptation :

1. Étant donné les 8 tests ignorés sans motif, quand le lot est terminé, alors chacun porte un motif.
2. Étant donné un nouveau test ignoré sans motif, quand l'analyse tourne, alors elle le signale.

### US15 - Proposition de signature des versions publiées (P3)

François veut décider si les fichiers publiés doivent porter une signature indépendante de leur hébergement. Il lit une proposition comparée.

Test indépendant : la proposition nomme la menace, compare au moins deux moyens et recommande un choix.

Scénarios d'acceptation :

1. Étant donné la proposition, quand François la lit, alors il peut répondre à la décision D8.
2. Étant donné l'absence de réponse, quand le programme se termine, alors aucun mécanisme de signature n'a été ajouté.

### US16 - Écrans Android séparés (P3)

Le contributeur futur modifie un écran de l'application Android. Il ouvre un fichier par écran.

Test indépendant : l'application s'assemble et ses tests unitaires passent comme avant.

Scénarios d'acceptation :

1. Étant donné le fichier principal de l'interface, quand le lot est terminé, alors il mesure moins de 300 lignes.
2. Étant donné l'application, quand on l'assemble, alors le résultat des tests unitaires est identique à la ligne de base.

### US17 - Code mort traité (P2)

Le contributeur futur ne rencontre plus de code inutilisé masqué par un attribut de silence sans raison.

Test indépendant : chaque attribut de silence restant porte une raison écrite.

Scénarios d'acceptation :

1. Étant donné les 9 attributs listés, quand le lot est terminé, alors chacun est retiré, converti avec une raison, ou restreint aux tests.
2. Étant donné le code réservé à une évolution future, quand le lot est terminé, alors il est conservé avec sa raison.

### Cas limites

- La branche `main` avance pendant un lot : le lot se réaligne avant son intégration. En cas de conflit sur un fichier déplacé, la règle d'arrêt s'applique.
- Un test échoue seulement sous charge : il est rejoué seul trois fois avant d'être compté comme régression.
- Une commande d'un guide échoue par erreur de syntaxe : l'implémenteur l'ajuste et consigne l'ajustement.
- Une décision bloquante n'est pas tranchée : le lot concerné attend. Les autres lots continuent.
- L'espace disque passe sous 12 Gio : l'implémenteur s'arrête et le signale.
- Un autre agent ajoute un reçu dans les consignes pendant le lot 3 : le lot repart de la version distante la plus récente.

## Requirements

### Exigences fonctionnelles

- FR01 : le dépôt DOIT offrir une commande locale unique qui lance le contrôle de format et l'analyse statique.
- FR02 : le dépôt distant DOIT lancer la compilation et l'analyse statique à chaque changement de code Rust, et les rendre bloquantes.
- FR03 : le dépôt distant DOIT lancer la suite de tests en observation, puis en bloquant après la décision D16.
- FR04 : un contrôle périodique DOIT signaler les dépendances Rust portant une alerte de sécurité.
- FR05 : l'analyse statique DOIT se terminer sans erreur ni avertissement en fin de lot 1.
- FR06 : chaque fichier suivi malgré une règle d'exclusion DOIT être couvert par une exception écrite, ou ne plus être suivi après la décision D3.
- FR07 : les fichiers de travail résiduels de la racine DOIVENT être archivés, sans perte de contenu.
- FR08 : le fichier de consignes d'agent DOIT contenir les règles permanentes, sans les reçus de session, et mesurer moins de 20 Ko.
- FR09 : les reçus de session DOIVENT être conservés à l'identique dans un fichier séparé.
- FR10 : les modules de tests en ligne de plus de 200 lignes des 16 fichiers listés DOIVENT être placés dans des fichiers voisins, sans changer un seul test.
- FR11 : chaque branche d'au moins 30 lignes de l'aiguillage de messages DOIT devenir une fonction nommée, sans changer une réponse.
- FR12 : chaque bloc `unsafe` DOIT porter un commentaire de sûreté, et l'analyse DOIT refuser un bloc sans commentaire.
- FR13 : la lecture de l'identité de l'utilisateur du système DOIT passer par une fonction sûre unique.
- FR14 : la récupération d'un verrou empoisonné DOIT être consignée et comptée, sans changer la poursuite du service.
- FR15 : le nombre de connexions ouvertes et son maximum DOIVENT être mesurés.
- FR16 : la durée de chaque tour de la boucle périodique de délégation DOIT être mesurée, et consignée au-delà d'un seuil.
- FR17 : chaque comparaison de texte d'erreur DOIT être classée. Les comparaisons sur un texte produit par Bridget DOIVENT utiliser une constante partagée.
- FR18 : les fonctions à 15 paramètres ou plus DOIVENT recevoir une structure d'arguments nommés.
- FR19 : chaque fonction de production de plus de 300 lignes DOIT être réduite, exemptée par écrit, ou arrêtée avec un motif.
- FR20 : le code Rust DOIT être formaté en un commit isolé, et le contrôle de format DOIT devenir bloquant ensuite.
- FR21 : les 18 modules nommés par numéro de session DOIVENT être renommés selon la table de correspondance, sans changer une chaîne de caractères.
- FR22 : un inventaire DOIT décrire chaque base, son mécanisme de version et ses réglages de durabilité, avec une décision d'architecture proposée.
- FR23 : les scripts Python et shell DOIVENT passer un contrôle automatique, localement et sur le dépôt distant.
- FR24 : chaque test ignoré DOIT porter un motif.
- FR25 : une proposition de signature des versions publiées DOIT être écrite. Aucun mécanisme n'est ajouté sans la décision D8.
- FR26 : les deux fichiers Android de plus de 1000 lignes DOIVENT être découpés par écran, dans la limite de la décision D15.
- FR27 : chaque attribut qui masque du code inutilisé DOIT être retiré, justifié par écrit, ou restreint aux tests.
- FR28 : aucun lot NE DOIT modifier le protocole entre composants, un format durable, une chaîne de caractères durable ou un droit.
- FR29 : chaque lot à comportement constant DOIT prouver l'égalité de la liste des tests en échec avec la ligne de base.
- FR30 : chaque tâche DOIT donner le fichier, le symbole ou la commande de localisation, la commande de vérification, le résultat attendu et la règle d'arrêt.
- FR31 : chaque choix réservé à François DOIT figurer dans `decisions.md` avec une action par défaut et son caractère bloquant ou non.
- FR32 : l'implémenteur DOIT consigner chaque tâche dans un journal, avec le commit et les preuves.

### Entités clés

- Lot : groupe de tâches indépendant. Il a une branche, un espace de travail, une porte de sortie et une priorité.
- Tâche : action unique, vérifiable, liée à un commit.
- Porte de décision : question, action par défaut, caractère bloquant, statut, réponse.
- Ligne de base : liste des tests en échec, liste des noms de tests, comptes d'analyse statique, liste des fichiers non formatés.
- Preuve : fichier texte daté dans le dossier de validation.

## Success Criteria

- SC01 : en fin de lot 1, un changement qui introduit un avertissement d'analyse est refusé par le contrôle distant.
- SC02 : les 3 erreurs et les 385 avertissements d'analyse mesurés sont ramenés à 0, hors dette structurelle documentée.
- SC03 : la racine du dépôt ne contient plus les 15 fichiers résiduels listés.
- SC04 : le fichier de consignes d'agent passe d'environ 125 Ko à moins de 20 Ko.
- SC05 : la part de tests en ligne des 16 fichiers listés passe d'environ 112 000 lignes à moins de 15 000 lignes.
- SC06 : la fonction d'aiguillage passe de 5011 lignes à moins de 1500 lignes.
- SC07 : 247 blocs `unsafe` sur 247 portent un commentaire de sûreté (3 aujourd'hui).
- SC08 : les 76 lectures directes de l'identité de l'utilisateur sont remplacées par une fonction unique.
- SC09 : aucune fonction de production ne dépasse 1000 lignes (3 aujourd'hui).
- SC10 : le contrôle de format signale 0 fichier (120 aujourd'hui).
- SC11 : 0 test ignoré sans motif (8 aujourd'hui).
- SC12 : pour chaque lot à comportement constant, la liste des tests en échec est identique à la ligne de base.
- SC13 : chaque porte de décision a une réponse ou une action par défaut appliquée et consignée.
- SC14 : un agent moins capable exécute un lot complet sans poser de question hors des portes de décision.

## Hors périmètre

- Documentation de masse des fonctions publiques.
- Réduction en masse des attentes fixes dans les tests.
- Épinglage de certificat dans l'application Android.
- Unification réelle des mécanismes de version de schéma (le lot 12 produit seulement l'inventaire et la proposition).
- Mise en place réelle d'une signature des versions publiées (le lot 15 produit seulement la proposition).
- Réécriture de l'historique Git.
- Activation des versions sur les machines, sauf réponse contraire à la décision D1.
- Renommage des fonctions qui portent un numéro de session.
- Tout changement fonctionnel du produit.

## Assumptions

- L'implémenteur dispose d'un accès au dépôt, de la chaîne Rust fixée par le projet et du droit de pousser sur `main` après chaque porte de lot.
- Les mesures peuvent avoir bougé. Chaque tâche localise sa cible par un symbole ou une commande, pas par un numéro de ligne.
- La ligne de base des tests dépend du système. Elle est mesurée sur la machine de l'implémenteur avant tout changement.
- Un seul lot est ouvert à la fois, à cause de l'espace disque et des fichiers partagés.
- Les lots 10 et 11 touchent presque tous les fichiers. Ils demandent un dépôt calme et un accord sur le moment (D9, D10).
- La correction d'une règle d'analyse par l'outil automatique ne change pas le comportement. La suite de tests le vérifie.
