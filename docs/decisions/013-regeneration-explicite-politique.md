# ADR 013 — Régénération explicite des instances de politique

**Statut** : Accepté
**Date** : 2026-08-27

## Contexte

La politique du greffe lie chaque droit à une instance de processus. Cette
liaison empêche une instance remplacée de continuer à muter la mémoire du
projet, mais l'instance change à chaque redémarrage normal. Onze principaux ont
été configurés lors de la première mise en service, dont dix sur une machine
distante : une collecte locale ne décrit pas la flotte.

Renouveler automatiquement au `Register` serait une escalade implicite, car
l'identité du `Register` reste déclarée par le client. Supprimer l'instance de
la décision annulerait au contraire la protection obtenue.

## Décision

Le renouvellement est une opération explicite distincte de l'autorisation.
Chaque principal régénérable nomme dans la politique la source exacte de ses
marqueurs : hôte mesuré et répertoire canonique. Un scanner est exécuté sur
chaque hôte, valide localement PID et naissance, puis produit un inventaire
transportable. La régénération exige exactement toutes les sources nommées.

Un principal vivant remplace toutes ses anciennes instances par la nouvelle,
sans changer actions, expiration ni révocation. Un principal approuvé absent
est conservé et signalé : l'absence d'un processus n'est pas une révocation.
Un principal observé mais non approuvé n'est jamais ajouté.

La politique est relue sous verrou, sa génération croît seulement lors d'une
modification, puis elle est remplacée atomiquement en mode 0600. La
prévisualisation est le comportement par défaut. Une erreur antérieure au
renommage conserve l'original. Après le renommage, l'outil relit et valide le
chemin final puis rend une issue indéterminée : il ne promet pas un retour
arrière que la primitive ne peut garantir.

## Conséquences positives

- un redémarrage normal peut être réparé sans ouvrir la politique par défaut ;
- une collecte oubliant les agents distants ne peut pas écrire ;
- l'ancienne instance disparaît et ne reste pas autorisée ;
- un agent arrêté n'est pas révoqué par accident ;
- le scanner distant mesure les PID sur le bon noyau ;
- aucun droit, secret ou expiration n'est créé par l'outil.

## Conséquences négatives

- toute politique régénérable doit être annotée une première fois avec ses
  sources ;
- une source entièrement arrêtée bloque la régénération, conformément au choix
  fail-closed ;
- l'opérateur doit transporter les inventaires distants par SSH ;
- un principal à plusieurs grants divergents exige une décision manuelle ;
- le binaire dédié doit être déployé par un lot d'installation séparé.

## Alternatives rejetées

- **renouvellement automatique au Register** : auto-enrôle une identité
  déclarative et forgeable ;
- **joker d'instance** : supprime la protection contre l'instance remplacée ;
- **effacer les principaux absents** : confond arrêt et révocation ;
- **liste de sources fournie seulement en ligne de commande** : permet encore
  d'oublier silencieusement un hôte ;
- **validation locale de marqueurs distants** : compare les PID au mauvais
  système ;
- **écriture directe** : expose une politique tronquée en cas d'arrêt.
