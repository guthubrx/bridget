# Spécification 125 - Fin observable d'un tour aussitôt suivi d'un autre

## Fiche synthèse
Spec: 125-fin-de-tour-rapide | Statut: Implemented | Priorité: P2 | Date: 2026-09-25
Branche: session-125-fin-de-tour-rapide | Constat de l'utilisateur sur le fil coordinateur.

## Problème observé
Le 25/09 à 07:49:05.9Z, GLM4 a clos un tour ; 0,6 s plus tard un message du coordinateur en a
lancé un autre. T3 n'expose que le dernier tour et le pont lit toutes les ~3 s : il n'a jamais vu
le premier clos comme dernier. Il l'a signalé « continuité non garantie » (lacune) au lieu de
« turn_ended », alors qu'il avait relevé son origine pendant qu'il tournait.

## Exigences
- **FR-001** : un tour qui n'est plus le dernier, clos, dont l'origine a été relevée, donne une fin
  observable (`stop_reason: ended`) et ne compte pas comme lacune.
- **FR-002** : origine inconnue : lacune signalée, fin non observable (inchangé).
- **FR-003** : origine = notification Bridget : aucun fait observable (anti-boucle, inchangé).

## Critères de succès
- **SC-001** : test reproduisant la scène de 07:49 pour les trois cas ; il échoue sur l'ancien code.
