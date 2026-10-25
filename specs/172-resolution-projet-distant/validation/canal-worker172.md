# Livraison du canal worker et des erreurs lisibles — 21 octobre 2026

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/canal-worker172.md#extrait-1).

## Source et composants actifs

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/canal-worker172.md#extrait-2).

Le worker ancien a été arrêté hors commande active, avant le remplacement du
maître. Le worker corrigé a été démarré seulement après le maître corrigé.
Aucun mélange ancien maître/nouveau worker. Le retour arrière suit l'ordre
inverse : worker/client/unité anciens avant maître ancien, hors mission active.
Les sauvegardes restent dans :
/Users/moi/.cache/bridget-message-errors172/bridget-before
/home/moi/.cache/bridget-message-errors172/worker-unit-before.service

## Validation

34 tests distincts du démon PASS, 2 du protocole PASS conservés car protocole
inchangé après r5 ; le dernier test rejoué recoupe les 34. Aucune suite complète.
Les noms exacts, les tours de test et les empreintes compilées sont conservés dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/canal-worker172-tests.json

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/canal-worker172.md#extrait-3).

Cette recette qualifie ce parcours réel. Elle ne qualifie pas tous les comptes,
tous les modèles ou d'autres machines. La saturation du modèle montrée dans la
capture du fil relève du fournisseur ; elle ne prouve pas la cause des coupures
du worker. La cause fine de la coupure historique 837972ad reste inconnue.

## Message français confirmé dans le fil propriétaire

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/canal-worker172.md#extrait-4).

```json
{
  "status": "failed",
  "work_state": "failed",
  "error": "remote_start_failed:remote_result_slot:WorkerUnavailable",
  "error_message": "Le service Bridget distant n’a pas confirmé la réservation du résultat. Le travail métier n’a pas été remis à l’agent.",
  "error_recovery": "L’arrêt du démarrage est terminé. Après correction de la cause, une nouvelle demande peut être créée. Rejouer la même demande ne la redémarre pas."
}
```

Le code technique et le reçu n'ont pas été réécrits pour présenter le message.
Les précisions séparent dossier, stockage, droits, capacité et transport.
Une erreur inconnue n'est pas transformée en manque d'espace disque.

## Ancienne admission figée

La demande opus158-20261021-bus-arches-reprise-v1 était chosen sans Task.
Son propriétaire avait rejoué une fois la même enveloppe, sans obtenir de Task.
La récupération existante après changement d'époque l'a soldée :
refused / permission_not_inherited / task_id=null. Aucune Task créée.
Aucune écriture manuelle du reçu, aucun changement de clé ou de fournisseur.
Le propriétaire a reçu ce résultat utile ; aucune nouvelle mission métier
n'a été lancée par cet intégrateur.

## Conservation et limites

Le résultat de recette, les worktrees propres de revue et la source de livraison
sont conservés. Aucun nettoyage Cartae par ce lot. Les paquets r3 refusés n'ont
jamais été activés. Les caches Cargo réutilisés et les sauvegardes sont conservés.
Pas de signature d'auteur supplémentaire dans Git.

La protection d'échéance complète utilise les deux composants corrigés et la
base UTC des baux existants. Les erreurs globales du store restent fermées ;
une commande sans preuve d'arrêt reste Unknown ou en attente. Aucun SIGKILL.
Les autres tâches ouvertes de la session 172 ne sont pas déclarées terminées
par cette livraison ciblée.
