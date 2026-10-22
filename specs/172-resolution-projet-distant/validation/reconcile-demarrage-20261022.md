# Réponse de statut avant Start — 22 octobre 2026

## Incident réel

Task 7562889b-08ef-4277-bf07-3d1798ca78da : préparation terminée à 1792698322, réservation et refus à 1792698323, échéance de lancement 1792698622. Aucun marqueur fournisseur ni remise métier. Le maître remplace la connexion worker à 19:45:23 UTC, seconde du refus PrepareExpired. La sonde initiale dure 6163 ms ; le binaire optimisé a corrigé la régression précédente de 189 s.

## Défaut reproductible et correction

La maintenance envoie Reconcile sans attente corrélée dès l’état starting. Elle peut précéder Start, avant la création du reçu worker. UnknownExecution revient alors sans entrée pending. Le maître répondait WorkerRejected(CommandMismatch), qui fait fermer le lecteur worker. La coupure annule le bootstrap et peut remonter PrepareExpired. Les journaux établissent la coupure au moment du refus ; ils n’enregistrent pas la trame précise qui l’a provoquée. Le test reproduit le défaut de traitement.

Les réponses Rejected sans attente sont maintenant ignorées uniquement après validation de la connexion, de son époque et de la commande distante connue (instance et génération comprises), comme les observations spontanées. Aucun succès, état de tâche ou droit n’est fabriqué. Les refus corrélés continuent de parvenir à l’appelant.

## Référence T3

/Users/moi/11.Repositories/t3code/packages/effect-codex-app-server/src/protocol.ts, resolvePending : une réponse dont l’identifiant n’est plus attendu produit Effect.void et ne ferme pas le transport. Bridget garde en plus la vérification du worker et de la commande distante, nécessaire au partage du canal entre missions.

## Vérification et livraison

Un seul test ciblé : spec172_untracked_worker_refusal_keeps_known_command_channel. RED avec l’ancienne production. Le cas couvre UnknownExecution et ResourceLimit sans attente, leur transmission avec attente, et le refus d’une autre instance ou d’une époque obsolète. GREEN : 1 test PASS, 0 FAIL. Activation à suivre. Aucune mission métier ou fournisseur lancé par ce test. Aucun élargissement des délais.

Preuves privées : /home/moi/.cache/bridget-reconcile172/red.log et /home/moi/.cache/bridget-reconcile172/green.log.
