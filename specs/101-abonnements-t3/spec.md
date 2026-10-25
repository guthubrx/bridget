# Session 101 — Abonnements utilisables depuis adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-1).

## Contexte et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-2).

## Scénarios utilisateur et tests

### US1 — Demander une notification depuis sa conversation (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-3).

- Une conversation reconnue crée un abonnement lui appartenant ; une autre
  conversation ne peut ni le consulter ni le supprimer comme sien.
- Deux conversations du même projet ne sont pas confondues.
- Une identité impossible à prouver produit un refus explicite, sans usurpation.

### US2 — Recevoir une fin de tour réellement observée (P1)

Après abonnement à Horizon-3D, un tour achevé produit une notification dans la
conversation demandeuse. La notification dit « tour terminé », pas « projet fini ».

- Les faits reçus après l'abonnement comptent, même produits juste avant.
- L'historique connu à l'installation du pont n'est pas rejoué.
- Un tour actif, un état inconnu ou une simple absence de sortie ne valent pas fin.
- Une notification ne déclenche pas une boucle de notifications de notifications.
- Une répétition du même fait ne produit pas de seconde notification.

### US3 — Connaître ce qui est réellement surveillable (P1)

Les types d'événements disponibles et leurs sources sont consultables. Une
demande précise impossible est refusée immédiatement avec une explication.

- Agent inexistant, source sans événement compatible ou source indisponible :
  aucun succès trompeur.
- Les écritures et permissions ne sont proposées que si elles sont attestées,
  jamais déduites du texte d'une réponse ou d'une commande shell.
- Une surveillance interrompue n'est pas présentée comme active ; après
  reconnexion, son état reste vérifiable et les limites sont explicites.

### US4 — Préserver les usages quotidiens (P2)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-5).

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-6).

## Entités principales

Conversation appelante attestée ; source observable et ses capacités ; fait de
tour terminé ; abonnement appartenant à une conversation ; état de surveillance.
Les titres restent des libellés, jamais des preuves d'identité.

## Cas limites et hypothèses

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-7).

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/spec.md#extrait-8).
