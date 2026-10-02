# Spécification 130 - Moins de réveils, moins de contexte consommé

## Fiche synthèse
Spec: 130-moins-de-reveils | Statut: Implemented | Priorité: P2 | Date: 2026-10-02
Branche: session-130-moins-de-reveils | Retour d'expérience opus2D, points 1 et 6.

## Problème observé
471 notifications de fin de tour ou d'observation en dix jours, chacune ouvrant un tour chez
l'abonné. Relances qui recopient les règles et lots longs saturant le contexte des GLM.
Relecture : la condition de la 129 (`batchable_prefix() > 0`) était toujours vraie, ce qui aurait
remis aussi demandes suivies et notifications en plein tour.

## Exigences
- **FR-001** : les notifications successives en tête de file partagent un tour (même bornes que
  les lots de messages), avec une enveloppe propre ; jamais mêlées aux messages.
- **FR-002** : seul un message sans réponse attendue pilote un tour en cours (correction 129).
- **FR-003** : la skill demande des relances courtes et la publication des contenus longs.

## Hors périmètre
Refuser l'abonnement d'une session « sans mission » : non défini de façon vérifiable ; la durée
de vie par défaut d'un abonnement (1 h) existe déjà.
