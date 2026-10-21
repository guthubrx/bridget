# Livraison du canal worker et des erreurs lisibles — 21 octobre 2026

Correctif installé et actif sur Mac et Cartae. Aucune mission native active au
moment de l'arrêt des composants Bridget. T3 et le pont sont restés ouverts.

## Source et composants actifs

- Source du binaire : e5195b7de80341170196bfd92def8277b2f0df3a. Build : 172-e5195b7d.
- Revue de production APPROVE sur 95c6ec4c, puis garde de sortie du test corrigée
  et test ciblé : 1 PASS. Aucun autre code de production modifié après la revue.
- Maître Mac : PID 3688, SHA256 20a44e11b74ebbe92e118fee4152cec6874270542aa5d6a895bd31e01860511c.
  Chemin : /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
- Worker Cartae : PID 589872, SHA256 2b3f36fc577a08fa41ac55cbb6c69b6e4f925463d688077f6188cb486dc4574c.
  Chemin : /home/moi/.local/lib/bridget-communication/172-e5195b7de803/bin/bridget.
  Service actif, NRestarts=0 après démarrage.
- T3 : PID 91019/91105 ; pont : PID 898. PID, dates de naissance et application
  signée inchangés. Aucun service métier relancé.

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

Délégation réelle depuis ce fil T3 sur le couple actif :
- Clé : recette172-canal-actif-cartae-e5195b7d-20261021-r1.
- Task : e05fd619-83fd-4056-9bea-5a42615783b0.
- Enfant : c7ea25a6-214c-46d7-9d6f-150306a99b8f.
- Route choisie par Bridget : claude / claude-haiku-5-5 sur Cartae.
- Machine cartae, version 0.4.1 et HEAD e5195b7de80341170196bfd92def8277b2f0df3a lus réellement.
- Marqueur : BRIDGET172_CANAL_ACTIF_CARTAE_OK.
- Résultat intégral lu puis acquitté ; rejeu strict unique : même Task et
  même résultat de 1369 octets, sans nouveau lancement.
- cleanup_done=true ; guarded=false (aucune garde acquise), guard_released=false
  conservé tel quel. Worktree propre, résultat et fichiers conservés.

Cette recette qualifie ce parcours réel. Elle ne qualifie pas tous les comptes,
tous les modèles ou d'autres machines. La saturation du modèle montrée dans la
capture du fil relève du fournisseur ; elle ne prouve pas la cause des coupures
du worker. La cause fine de la coupure historique 837972ad reste inconnue.

## Message français confirmé dans le fil propriétaire

L'agent chargement a effectué une seule lecture MCP officielle de la tâche
837972ad-ad58-4512-9a71-fd2843ab5be5 après activation, sans relance des façades.
Message de preuve : t3-7120f9a6-52e9-4cdb-a3ed-4844a2de32b2.

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
