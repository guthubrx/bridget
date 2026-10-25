[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/175-cycle-worktrees/reuse-audit.md#extrait-1).

## Précisions après contre-revue du plan
Un seul évaluateur de motifs doit alimenter inventaire et retrait existant. La commande worktrees retourne avant toute purge. Git sans verrou optionnel ; erreurs et délais deviennent inconnu. Le rappel de restitution reste une consigne constante de relecture du statut, jamais un motif dynamique figé. Décisions détaillées : adversarial-review.md.

## Items réutilisés au clavier
- REUTILISER le tuple worktree_entries comme Worktree nommé : ajouter locked/prunable observés, sans second registre.
- EXTRAIRE integration_reason, age_reason, dirty_reason depuis prune_merged_worktrees : règles partagées, contrainte explicite d’un seul évaluateur pour retrait et observation.
- CREER inventory/report_worktrees et table des consignes : aucun rendu d’inventaire existant trouvé ; branche dans la commande d’entretien existante. Pas de dépendance.
- REUTILISER git_ok/busy_paths : erreurs explicites, status sans verrou ni fsmonitor, index des fichiers ouverts par passage.
