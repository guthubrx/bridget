# Budget de lecture du connecteur de bureau

Statut : intégré, publié et activé. Programme 52/182 In Progress.

## Incident et corrélation
Signalement opus201-primary-absent-petits1075-20261101-01, tâche
9396b336-49ae-4384-a798-749dec358f26 ; tâche indépendante dcb7073e conservée.
Le journal natif du parent montre ensuite un task_cancel réussi à
00:55:47 UTC, sur le même ID, garde close et continuable=false.
Aucun nouvel appel de clôture n'est requis pour cette preuve déjà acquise.
Aucune mission inconnue, tâche ou production n'a été rejouée.

Le lien causal entre primary_absent et le volume HTTP n'est pas établi.
Le premier diagnostic l'attribuait trop vite ; la correction factuelle a été
transmise au parent et au reviewer. Le refus primaire peut être transitoire
pendant republication ; l'heure de son appel initial reste à corréler.
Ne pas présenter le présent correctif comme preuve de sa cause historique.

## Défaut distinct confirmé
Le pont répète response limit pour le fil 32e1b8f8 depuis sa republication.
Lecture diagnostique autorisée de la route officielle bounded?compactTurnItems=1 :
28 038 589 octets, 17 runs, 6 704 nœuds, 6 510 éléments visibles,
64 checkpoints, payloadBudgetExceeded=true. Le corps brut reste privé.
Le mode compact est déjà demandé. L'API installée ne déclare pas de paramètre
pour réduire cette fenêtre ; elle peut conserver un tour qui dépasse son budget.
La limite générale Bridget de 16 Mio refusait donc ce corps valide avant parse.

## Correction et bornes
Seule la lecture V2 bornée (turn_limit <= 20, route /bounded) peut lire
64 Mio, plus un octet de détection d'excès. Aucun passage automatique à
l'historique complet. Les autres lectures et mutations gardent 16 Mio.
L'agrandissement de page existant garde sa limite précédente.
HTTP reste loopback authentifié, délai 10 s, zéro redirection.
UTF-8, identifiant du fil, rôles, nœuds racines, outils, questions,
correspondance des tours et bornes de collections sont validés comme avant.
Aucune nouvelle autorité, session, mission, clé, écriture métier ou format.

Au-delà de 64 Mio, refus conservé. Ce correctif ne rend pas la lecture illimitée
et ne remplace pas une future pagination officielle. Un grand corps coûte
plus de mémoire pendant son décodage ; le choix reste borné et local à la route.
payloadBudgetExceeded ne devient ni un droit ni une preuve de complétude
historique. Les observations et la corrélation existantes restent inchangées.

## Contrôles ciblés
Deux tests nouveaux PASS sur Linux et Mac : seuils 16/64 Mio, dépassement,
UTF-8 invalide ; HTTP réel de fixture >16 Mio sur la seule route bornée,
projection normalisée identique au parseur existant.
Régression du client V2 existant PASS : découverte authentifiée, lecture,
dispatch et rejeu d'identifiants RPC stables. Aucun fournisseur appelé.
Trois contrôles distincts Linux, deux Mac ; aucune campagne générale.

Le correctif est technique, sans nouveau parcours d'interface :
pas de diaporama requis. Les preuves brutes, journaux et reçus restent privés
dans /Users/moi/.cache/bridget221-thread-budget et
/home/moi/.cache/bridget221-thread-budget.

## Revue et activation
Revue native eda3794b, lecture seule : aucun P1/P2 concret dans le delta.
Sa réserve de concurrence est bornée par Session::call : le mutex partagé
couvre la requête et son parse ; le pont ne lance pas ces lectures en parallèle.
Cela ne supprime pas le coût de décodage du corps. Tests effectivement PASS
chez le responsable ; revue lue/ACK et clôture demandée une fois.

Source exécutable 6b131e83e077734489439058dcb09d37a900b224,
intégrée/poussée. Build 221-thread-budget-r1, version 0.6.5.
Linux : 41 390 928 octets, GLIBC maximum 2.34,
SHA256 b5ddd06b91b965a073ce02995f70b0b1ec28b3446d90505fdb67c09f74b5f1dc.
Mac : 34 172 608 octets,
SHA256 9ef290f2e7a2a8bcefdf96fb486a9a5bafd4a36fd093ca0c34ddb45388d8df29.
356 fichiers de sources comparés aux snapshots : aucune différence.

Sept images réellement chargées et hachées : maître Cartae 3685534,
mobile 3685611, worker Cartae 3686415, worker SYS 1155585, worker KS1527812,
worker Mac 86452, pont Mac86817. Coupure maître 2,13 s.
Wrappers Cartae3405178/naissance300643209 et2500061/naissance270585460
observés conservés. Aucun service métier ni mission historique relancé.

Retour221-continuable-r3 compatible par les deux lecteurs de formats ;
pointeurs rollback Cartae/Mac écrits par l'écrivain natif atomique sous verrou,
sans activation concurrente. Retour haché, updaters repris.
Les anciens clients MCP221-view-sync sont conservés ; pas de nouveau schéma.

Pont repris à01:03:54UTC ; fil monuments exposé à01:04:22UTC,
nom confirmé à01:04:24UTC, état privé actualisé après exposition.
Aucune erreur response limit entre la reprise et la lecture de preuve ;
le même fil échouait toutes les6s avant le passage.
Réattestation native révision15 puis même task_status de revue réussis.
Pas d'annulation ou de mission de qualification supplémentaire.
Les trois parents ont été informés, reçus in_flight conservés ; leur lecture
n'est pas prétendue acquise.

Trois copies binaires de transfert vérifiées retirées :124172784octets.
installed appelé Mac/SYS : staging non enregistré Mac et observation
processus incomplète SYS ; caches conservés sans forcer.
Actif et retour, images encore utilisées et worktrees de revue sous
observation incomplète conservés. Workspace221 reste nécessaire à la suite.
Programme global52/182InProgress, aucune catégorie entière close.
