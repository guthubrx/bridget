# Budget de lecture du connecteur de bureau

Statut : contrôlé, livraison en cours. Programme 52/182 In Progress.

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
