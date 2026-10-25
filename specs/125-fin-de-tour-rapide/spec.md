# Spécification 125 - Fin observable d'un tour aussitôt suivi d'un autre

## Fiche synthèse
Spec: 125-fin-de-tour-rapide | Statut: Implemented | Priorité: P2 | Date: 2026-09-25
Branche: session-125-fin-de-tour-rapide | Constat de l'utilisateur sur le fil coordinateur.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/125-fin-de-tour-rapide/spec.md#extrait-1).

## Exigences
- **FR-001** : un tour qui n'est plus le dernier, clos, dont l'origine a été relevée, donne une fin
  observable (`stop_reason: ended`) et ne compte pas comme lacune.
- **FR-002** : origine inconnue : lacune signalée, fin non observable (inchangé).
- **FR-003** : origine = notification Bridget : aucun fait observable (anti-boucle, inchangé).

## Critères de succès
- **SC-001** : test reproduisant la scène de 07:49 pour les trois cas ; il échoue sur l'ancien code.
