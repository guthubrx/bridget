# Spécification 111 — Une remise attend la fin du tour au lieu de périmer

## Fiche synthèse

Spec: 111-remise-attend-le-tour
Statut: In Progress
Priorité: P1
Tâches: 0/5
Date: 2026-09-19
Branche: session-111-remise-attend-le-tour
Dépendances : adaptateur T3 098, envoi idempotent 099.

## Problème observé

Constaté le 2026-09-19. Un agent a envoyé à un autre une libération de périmètre de travail. Le
destinataire, occupé sur un tour long, ne l'a jamais reçue. Journal du pont, deux minutes après
l'envoi : `remise mcp-47021-6aae971d-1 périmée avant démarrage`. Un relais envoyé ensuite a subi
le même sort, à la même seconde d'écart. Neuf remises ont été jetées ainsi dans la journée.

Cause : le pont retire de sa file toute remise dont l'attente dépasse `turn_wait`, deux minutes par
défaut, quelle que soit l'échéance réelle du message. Ce délai sert de garde-fou mémoire, la file
n'étant bornée par rien d'autre. Un agent qui travaille plus de deux minutes ne reçoit donc rien,
alors que la remise reste valide côté daemon pendant sept jours.

Le destinataire n'était pas en panne : il écrivait encore une minute avant le constat. Il n'a
simplement jamais été inactif au bon moment.

## Scénarios utilisateur

- **US1 (P1) — Recevoir après un tour long.** Un message sans échéance envoyé à un agent occupé
  est remis à la fin de son tour, même plusieurs heures plus tard.
- **US2 (P1) — Respecter une échéance déclarée.** Un message qui porte une échéance ou un délai de
  réponse reste écarté quand elle est dépassée, comme aujourd'hui.
- **US3 (P2) — Ne pas grossir sans fin.** La file d'attente d'un fil reste bornée en nombre ; au
  delà, la plus ancienne remise est écartée avec une trace explicite.

## Exigences fonctionnelles

- **FR-001** : une remise n'est plus écartée au seul motif que l'attente dépasse `turn_wait`.
- **FR-002** : une remise reste écartée si son échéance métier est dépassée, si son délai de réponse
  est écoulé, si sa saga a expiré, ou si la demande suivie n'est plus ouverte.
- **FR-003** : la file d'un fil est bornée à un nombre fixe de remises ; un dépassement écarte la
  plus ancienne avec un avertissement distinct de la péremption.
- **FR-004** : le cache des annulations conserve son propre délai, inchangé.
- **FR-005** : aucune modification du protocole, du format de sérialisation ni de la durée de vie
  d'une remise côté daemon.

## Critères de succès

- **SC-001** : une remise sans échéance attendue plus longtemps que l'ancien délai est toujours en
  file et remise dès que le fil redevient libre.
- **SC-002** : une remise dont l'échéance est dépassée est toujours écartée.
- **SC-003** : au-delà de la borne, la file ne grandit plus et la plus ancienne est écartée.
- **SC-004** : recette complète verte.
- **SC-005** : après livraison, les trois remises en attente pour le fil bloqué sont effectivement
  remises, et le journal ne montre plus de péremption pour elles.

## Hors périmètre

Changer la durée de vie d'une remise côté daemon, notifier l'expéditeur d'une attente longue,
interrompre un tour en cours pour injecter un message.
