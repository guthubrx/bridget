# Plan 258 — Conversation et commandes communes

## Contexte et responsabilité

Bridget : Rust, protocoles fermés, daemon maître, worker, pilote Codex existant.
T3 : TypeScript/Effect, React, composants de conversation partagés. Base T3
75c7dafe51 ; base Bridget e41b75fe. Deux worktrees isolés, aucun changement Parix.

## Réutilisation de l'existant

1. Conserver ThreadTimeline/MessagesTimeline et projectBridgetTimeline acquis.
   Raccorder les faits du journal au suivi et à la position du même fil.
2. Réutiliser ChatComposer, ComposerSurface, ComposerModelEffort, les stores de
   brouillon, éditeur, raccourcis et contrôles ; séparer le transport natif de
   l'interface sans copier la boîte ni autoriser thread.start sur une projection.
3. Étendre le protocole humain HumanLineage et les ports BridgetReader/Lineage.
   Autorité par conversation racine + projet + task_id ; jamais un enfant arbitraire.
4. Réutiliser les puits de remise idempotente, contrôle d'interruption et
   SelectRuntime existants. Le pilote garde la validation des modèles/efforts.
   Les reçus existants sont préférés à toute nouvelle table.
5. Publier les capacités et états utiles à cette boîte ; distinguer état
   confirmé, sélection en attente et issue inconnue. Aucun fallback local.

## Livrables et surfaces

Crates protocole/daemon/CLI : contrat additif, gardes, tests locaux/distants.
T3 contracts/client-runtime/server/web : RPC autorisée, transport et UI communs,
tests de contrat, service et navigateur. Adapter les consommateurs mobiles si le
contrat partagé les affecte. Pas de nouveau moteur ni catalogue parallèle.
Documentation du parcours et preuves-corrections dans le dépôt T3.

## Risques à résoudre avant code

Le catalogue doit provenir de la cible, pas du parent. Un contrôle livré mais
sans réponse reste inconnu. Aucune commande ne survit à un changement de ronde
sans vérifier la cible. Les événements du journal ne sont jamais des commandes.
Les pièces jointes suivent les capacités de transport réelles et leurs chemins.
Les sélections futures ne réécrivent pas les faits historiques d'un tour.

## Vérification et livraison

Tests ciblés Rust via scripts/build.py ; même cache entretenu avant/après.
Tests de contrat/Effect/UI, types et lint T3 par sa chaîne actuelle ; réutiliser
les preuves encore valides. Converge après le dernier changement. Audit V15 avec
décision calculée ; pas de succès annoncé sur un gate bloqué/inconclusif.
Contre-revue plan puis code quand un autre fournisseur est joignable.
Intégration sur main et push des deux dépôts, sans force ; distributions natives
Bridget si son exécutable change. Préparer le paquet T3 sans fermer l'application.
Préserver travaux étrangers, missions, preuves, actif et retour vérifié.

## Estimation initiale

1 440–2 400 minutes. Début : 2026-11-10 06:51 CET. Recalibrer après Tasks.

Lotincident associé10nov : conserver le diagnostic OAuth fournisseur dans le classifier existant et partager le guidage entre le statut et les notifications. Trois usages d’un helperfermé, aucune table ni dépendance supplémentaire. Reconnexion du compte par l’utilisateur ; aucune migration de compte, substitution de modèle ou mission rejouée.
