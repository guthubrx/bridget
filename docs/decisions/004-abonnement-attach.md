# ADR 004 — Abonnement attach médié par le daemon

**Date** : 2026-08-22
**Statut** : Accepté

## Contexte

Les équipiers ACP de la session 007 sont headless : leur journal versionné
constitue la source d'observation, mais une vue cliente doit pouvoir le suivre
en local comme à travers la fédération SSH. Un client attach ne doit pas se
faire passer pour un équipier : l'inscrire dans l'annuaire ou lui laisser les
messages de wrapper étendrait inutilement la frontière de confiance. Les
envois humains doivent, eux, conserver le chemin normal du daemon et ses
garde-fous, sans créer de demande suivie ni de relance.

Le journal 007 dispose déjà d'un écrivain et d'une lecture de compatibilité.
Le relais attach exige cependant une lecture incrémentale bornée, avec les
positions nécessaires au rejeu et au suivi, sans introduire un second parseur
du schéma v1.

## Décision

Le client attach ouvre une connexion à rôle dédié, séparée des wrappers : elle
reste hors annuaire et hors routage d'équipier. Sa matrice de messages est
fermée : seuls `Subscribe`, `Unsubscribe`, `Send` avec `reply=false` et le
ping (`Heartbeat`) sont admis. Toute variante réservée aux wrappers —
notamment `Register`, `Rename`, `Runtime`, `Domain`, `Availability`,
`TurnState`, `DeliveryRejected`, `CancelRequest` et `ListRequests` — est
refusée par un motif typé « message hors rôle attach ».

Un envoi depuis la vue réutilise `Send` sur cette connexion persistante, avec
une identité humaine imposée par le daemon et `reply=false`. La réponse de
l'équipier est donc observée par le flux d'abonnement ; aucun suivi de demande
ni cycle de relance n'est créé. Cette réécriture est appliquée dès la
négociation du rôle, avant le raccordement complet des abonnements.

La lecture du journal est étendue par tranches dans `journal.rs`. Le nouveau
lecteur partage le parseur v1 et ses fixtures avec `valid_events`, mais expose
les offsets et les erreurs de lignes requis par le relais. Il ne crée ni
parseur JSONL concurrent ni changement de comportement de T706.

La compatibilité sans handshake des wrappers 007 est **coopérative**, non une
frontière d'authentification : le premier message non-handshake fige le rôle
`Wrapper`, mais un client local reste dans le même modèle de confiance et peut
choisir ce rôle. Un handshake ultérieur est refusé afin qu'une connexion active
ne puisse pas changer de privilèges.

## Décisions associées du plan

- **D-303** : le client ne conserve son curseur qu'en mémoire ; la reprise
  inter-invocations demeure hors périmètre.
- **D-304** : les données journalisées passent par une liste blanche Unicode ;
  seules les séquences ANSI émises par le rendu sont fiables.
- **D-305** : chaque vue lente est isolée par un tampon borné et un écrivain
  dédié ; aucune E/S socket ne tient le verrou global du daemon.
- **D-307** : le terminal passe en raw mode minimal via `libc`, avec
  restauration RAII et traitement explicite de Ctrl-C.
- **D-308** : `Send(reply=false)` reste sur la connexion attach persistante ;
  les `Ack`, `Nack` et `DeliveryRejected` y restent corrélés par `message_id`.

## Conséquences positives

- La même connexion applicative couvre les vues locales et fédérées sans
  accès au système de fichiers distant.
- Un client attach ne peut ni devenir une cible d'annuaire ni émettre les
  commandes d'administration d'un wrapper.
- Les envois restent soumis aux garde-fous existants tout en évitant des
  relances inutiles pour une réponse visible dans le flux.
- Le schéma v1 et ses règles de récupération demeurent centralisés dans un
  unique module de journal.

## Conséquences négatives

- Le daemon doit négocier et contrôler explicitement un rôle supplémentaire.
- La matrice fermée doit être mise à jour avec intention pour toute nouvelle
  commande attach.
- L'extension de `journal.rs` doit préserver les usages et fixtures de la
  session 007.
