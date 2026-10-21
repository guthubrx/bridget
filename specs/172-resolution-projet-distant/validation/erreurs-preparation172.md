# Erreurs de préparation distante — 21 octobre 2026

État : code validé, paquets prêts, activation différée. T044 reste ouverte.

La demande porte sur des messages compréhensibles. Le nettoyage Cartae appartient
à un autre responsable. Ce lot n’a supprimé aucun fichier distant.

## Comportement livré dans les sources

La réponse MCP ajoute error_message et error_recovery, tout en conservant error,
la clé de demande et le TaskID. Une préparation échouée avant la création de
l’enfant est présentée par work_state=failed. Le status durable reste inchangé.

Le message distingue disque plein, accès refusé, échec de préparation Git et reçu
incomplet. Il explique la reprise : corriger la cause, annuler la tâche, puis
créer une nouvelle demande. Rejouer la même demande ne relance pas la préparation.
Les flux Git restent privés et bornés à 1 Mio chacun.

Exemple pour un nouveau diagnostic DiskFull :
« Préparation distante impossible : espace disque insuffisant sur la machine
cible. La tâche reste enregistrée. »

L’ancien reçu ba369015-da43-45c4-b70a-dae9509ea2db garde StoreUnavailable.
Le diagnostic Git précis a été perdu par l’ancien code. Le nouveau texte
l’indique ; il ne présente pas une cause supposée comme une donnée observée.
Son propriétaire a annulé cette tâche ; cleanup_done=true, aucune garde acquise.
Aucune mission de remplacement n’a été lancée dans ce lot.

## Référence et validation

Les mécanismes T3 lus et les adaptations distantes sont consignés dans :
/home/moi/10.Scripts/64.bridget/specs/172-resolution-projet-distant/implementation.md

14 tests distincts PASS, nouveaux tests et régressions ciblées. Aucune suite
complète. Les dix filtres, la liste exacte et les comptes d’exécution sont dans :
/home/moi/.cache/bridget-message-errors172/tests-r2.json
/home/moi/.cache/bridget-message-errors172/test-proof-r2.json

Revue indépendante via le MCP officiel Bridget, choix du moteur non imposé :
r1 Task533592b7-e282-4537-8862-3a116dcc9036 ; objections traitées.
r2 Task832e891b-ec6f-4922-a5d4-a7a5fcdd355b ; APPROVE pour le code.
Les deux résultats ont été lus intégralement et acquittés. Les précisions
documentaires r2 sont intégrées à implementation.md. Deux réserves faibles
héritées y restent explicites, sans prétendre les avoir corrigées.

## Paquets préparés

Source compilée commune : b633e17d8cd56dcc88c1fd719ecf6ba4fe9f5c6c.
Le commit documentaire suivant ne change pas cette source.

Mac : build172-b633e17d, compilation117,14s RC0, signature ad hoc vérifiée.
Binaire : /Users/moi/.cache/bridget-message-errors172/bridget
SHA256 : b0511487ff19fc883da30ed8e2855f59959f461d8662930cb1f068d7d25c3729

Cartae : compilation92,52s RC0.
Binaire : /home/moi/.local/lib/bridget-communication/172-b633e17d8cd5/bin/bridget
SHA256 : b6f81277925f40034c1378bb780ef4d97c022969879504ee04022463b3e7aaf7

Les paquets161df06a précédents sont obsolètes. Ils ne doivent pas être activés.
Les scripts préparés utilisent les reçus r2. Ils n’ont pas été exécutés.

## Activation à reprendre

Au contrôle du 21 octobre, une tâche Cartae travaille encore :
ddebe152-55b0-4b71-9a8a-c89676ea59e3. Aucune admission automatique ouverte.
Elle appartient à un autre responsable ; aucune interruption n’est demandée.

Un abonnement ponctuel turn_ended, once=true, est enregistré sur son enfant
d9b668f5-e23a-45f4-ac14-dc8f87838b09. Identifiant :
f758f6d2-308d-4d28-aa0f-bfd70b134975, expiration1792574512.
Une fin de tour ne prouve ni succès ni fin de mission. À ce signal, relire les
gardes des tâches et admissions. Aucun minuteur ni activation automatique.

Les processus actuels sont conservés : maître16326/source7085b77c,
worker624845/source72a8e8f8, T3 Beta91019/91105, pont898. Les anciens
PID T330609/30660 des reçus du 19 octobre sont historiques.

Avant activation, revérifier PID, dates de naissance, empreintes et absence de
tâche active ou gardée non nettoyée. Activer le maître corrigé, puis seulement
le worker corrigé. Un ancien maître ne décode pas les nouveaux motifs.
Retour arrière : ancien worker, client et unité d’abord, ancien maître ensuite.
Aucun redémarrage T3.

Après activation, demander au propriétaire de ba369015 une seule lecture MCP
officielle de son ancien reçu. Cette lecture vérifie error_message dans son
contexte, sans nouvelle mission ni simulation de panne de disque en production.

Les sources et tests sont validés. La restitution par les services actifs reste
à vérifier ; ce reçu ne revendique ni activation ni validation réelle de ce
dernier parcours.
