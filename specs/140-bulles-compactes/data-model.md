# Modèle140 — Projection de présentation, sans migration

## Message source

Le texte, l'identifiant, le fil T3, les pièces jointes et les actions existants restent les données de référence. Aucun changement SQL ou de stockage T3. La copie conserve le texte source, pas le résumé.

## Métadonnée de remise Bridget

BridgetMessage reçoit thread_display_title facultatif au niveau racine. Champ absent : défaut None et omission à la sérialisation. Résolution uniquement lors de la remise pour le destinataire concerné, après lecture autorisée Store.thread_show. Cette valeur n'est pas écrite dans le message persistant ou son canon.

ThreadNotice ne change pas. La branche idempotente et la branche classique de remise appliquent la même règle. En cas d'absence, d'échec de lecture ou de refus d'appartenance, aucune valeur de titre n'est remise.

## Projection T3 mémoire

Une projection pure d'une enveloppe complète fournit : famille, libellé, nom d'émetteur connu, compteur de lot connu, titre facultatif du fil, corps lisible et texte brut original. Elle ne fournit ni agent destinataire supposé ni type de publication inféré. Une sollicitation sans titre a le libellé « Fil partagé · Nouveautés ».

Les cinq familles sont direct, thread-notice, observation, direct-batch et observation-batch. Une entrée non reconnue ne produit pas de projection Bridget et suit le rendu ordinaire.

## États de présentation

État local ouvert/replié, par identité composée fil T3 + message. Valeur initiale : replié. Les détails techniques peuvent être ouverts seulement depuis la bulle. Changer de fil ou recycler une ligne ne transmet pas l'état à un autre message. L'état n'est pas une prise en charge ou un ACK Bridget.

## Invariants

- Le brut reste identique au texte reçu.
- L'ouverture n'envoie ni message ni ACK, et ne change pas une mission.
- Les suffixes techniques ne sont retirés du corps lisible que s'ils correspondent exactement à un suffixe final déclaré.
- Le canon et le journal ne contiennent pas l'enrichissement éphémère du titre.
- Un titre inconnu, refusé ou absent ne peut pas être inventé.
- La détection d'enveloppe sert uniquement à la présentation.
- Aucun état persistant, nouveau registre de noms ou dépendance n'est ajouté.
