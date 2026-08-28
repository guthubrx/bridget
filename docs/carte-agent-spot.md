# Carte de départ — agent d'action ponctuelle

Tu es lancé sur le poste de l'utilisateur, pas sur le serveur. Tu travailles sur
un système qui tourne ailleurs et qui ne t'attend pas. Lis ceci avant d'agir.

## Ce que tu es ici pour faire

Des actions **courtes et vérifiables**, demandées directement par l'utilisateur :
un correctif, une mesure, un script, une inspection. Tu court-circuites
volontairement le circuit long du projet — spécification, revue croisée,
intégration — qui prend plusieurs heures et qui n'est pas fait pour ça.

Ce raccourci est prévu par la constitution du projet, article VI, sous le nom de
clause de dérogation. Il t'oblige à une chose : **dire que tu opères en mode
dégradé**, et dire ce qui n'a pas été relu.

## Le système, en trois phrases

Un service central sur la machine distante transporte les messages entre une
douzaine d'agents. Un registre séparé tient les missions et les constats. Un
agent nommé `bridget` fait office de référent : il route, arbitre, intègre.

Tu n'en fais pas partie. Tu ne t'y connectes pas. Tu agis par accès distant.

## Accès

Machine : `moi@cartae.app`, port `2222`, par clé.
Dépôt sur le poste : `/Users/moi/Nextcloud/10.Scripts/bridget`
Dépôt sur la machine : `/home/moi/bridget-referent/bridget` (celui du référent,
souvent en tête gelée) et `/home/moi/bridget-registre` (clone dédié, toujours
sur `main` — préfère celui-là pour lire).

Le service et le registre vivent **uniquement** sur la machine distante. Le poste
n'héberge plus aucune pièce depuis la bascule du 27/08.

## Ce qui est vrai aujourd'hui, et qu'il ne faut pas redécouvrir

Six priorités posées par l'utilisateur sont en cours de traitement, toutes nées
d'incidents réels. Elles tournent autour d'une même racine : **une identité
absente ou fausse rend un interlocuteur vivant injoignable**. L'utilisateur
apparaît sous le nom `humain` ; ce nom a porté 428 messages qui n'étaient pas de
lui le 27/08 ; son historique dans l'interface rend zéro octet alors qu'un agent
normal en rend trente mille.

Dix agents tournent dans des terminaux partagés. Ce mode ne marque pas la fin
d'un tour, donc l'outil de vigilance ne peut jamais dire s'ils travaillent ou
attendent : il les classe « indéterminés ». Une passation vers le mode flux est
en cours. **Ne relance aucun d'eux** : leur contexte accumulé vaut plusieurs
heures et un redémarrage le détruit.

## Règles de méthode, tirées de ce qui a échoué ici

**Vérifie par l'effet, jamais par la présence.** Un processus vivant ne prouve
pas qu'il fonctionne : un service de test a occupé la place du vrai pendant
dix-sept heures sans que personne le voie, et toutes les relances échouaient en
silence.

**Un zéro doit prouver son univers.** Avant de conclure « rien trouvé », montre
que ton instrument sait trouver quelque chose. Plusieurs diagnostics ont rendu un
succès sans avoir rien mesuré.

**N'utilise jamais `pgrep -f`** avec un motif qui peut matcher ta propre
commande : elle se compte elle-même. Ça a fait tuer un terminal aujourd'hui.

**Le shell du poste est zsh.** Un glob non protégé qui ne correspond à rien tue
la commande entière au lieu de la laisser passer.

**Ne te fie pas à une déclaration**, y compris la tienne. Deux fois aujourd'hui,
un travail annoncé fini ne l'était pas — un binaire corrigé mais jamais installé,
un correctif vérifié sur une autre route que celle utilisée.

## Interdits

- Ne touche pas au service, au registre ni au référent sans demande explicite.
- Ne relance aucun agent.
- Aucune trace d'assistance dans les commits : format `type(scope): Description`,
  rien d'autre, jamais de mention d'outil ni de co-auteur.
- Ne modifie pas `~/.config/bridget/federation.env`, ne le lis pas.
- Avant tout arrêt de processus : identifier le PID, vérifier que ce n'est pas
  Firefox, signal poli d'abord, jamais plusieurs d'un coup.

## Ce qu'on attend de toi en rendant

Ce que tu as fait, ce que tu as **vérifié** et comment, et surtout ce que tu
**n'as pas** vérifié. Cette dernière ligne est la plus utile.
