# Analyse croisée - SPEC-075

## Résultat

**PASS après auto-correction.** Aucun conflit critique ou majeur ne subsiste
entre `spec.md`, `plan.md`, `data-model.md`, le contrat et `tasks.md`.

## Couverture des exigences

| Bloc | Exigences | Tâches |
|---|---|---|
| Sémantique des trois actions | FR-7501 à FR-7510 | T013 à T021, T026 à T037 |
| Durabilité et reprise | FR-7511 à FR-7515 | T001 à T012 |
| Éligibilité et protocoles | FR-7516 à FR-7519 | T013 à T030 |
| UX et concurrence | FR-7520 à FR-7523 | T021, T028 à T037 |
| Migration héritée | FR-7524 | T022 à T025 |
| Audit et réservation du nom | FR-7525 à FR-7526 | T007, T020, T021, T039 à T043 |
| Non-fonctionnel | NFR-7501 à NFR-7506 | T004, T014, T030, T035 à T043 |

Toutes les exigences disposent d'au moins une tâche d'implémentation ou de
preuve. Aucun mécanisme demandé n'est porté seulement par la documentation.

## Auto-corrections effectuées

1. Remplacement de la suppression totale au décommissionnement par une
   tombstone `decommissioned` cachée, car le nom est la clé logique de
   l'historique.
2. Réservation du nom décommissionné jusqu'à une future purge explicite.
3. Suppression de la reprise automatique de l'ancien parent, mandat ou objectif
   lors d'un relaunch administratif.
4. Clarification que `persistent` et `lifecycle_state` sont deux dimensions
   indépendantes.
5. Clarification que le succès de decommission arrive seulement après arrêt
   confirmé pour un agent actif.

## Cohérence des machines d'états

- `running -> stopped`: stop, mort observée ou réconciliation d'un non
  persistant au redémarrage.
- `stopped -> running`: relaunch connecté avec nouvelle génération.
- `running -> stopped -> decommissioned`: decommission d'un agent actif.
- `stopped -> decommissioned`: decommission d'un agent arrêté.
- `decommissioned`: terminal pour cette spec, caché, non relançable, nom
  réservé.

Il n'existe aucun chemin `stopped -> running` au démarrage automatique. Seules
les entrées `running + persistent=true` sont reprises.

## Cas limites vérifiés dans le plan

- Arrêt pendant lancement ou reprise.
- Processus sans connexion mais marqueur géré présent.
- Timeout d'arrêt pendant decommission.
- Échec de relaunch après réservation.
- Nom devenu actif avant traitement.
- Présence mémoire disparue après redémarrage.
- Agent non persistant au redémarrage.
- Agent externe, TMUX ou sans preuve durable.
- Ancien agent géré arrêté avant le schéma 4.
- Rendu UI pendant polling et action en vol.

## Dégradations acceptées

- Le CLI `specify` absent empêche les scripts officiels de vérifier les
  artefacts. Les contrôles ont été reproduits manuellement et `git diff --check`
  est propre.
- `mem` absent empêche DevKMS. Les sources et décisions sont consignées dans
  `research.md`.
- La contre-revue externe peut ne pas répondre parce que le quota Claude est
  épuisé. La contre-revue locale a déjà produit puis corrigé deux constats
  majeurs.
