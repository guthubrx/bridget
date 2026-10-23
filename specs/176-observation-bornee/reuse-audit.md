# Audit de réemploi176
Statut : PASS pour plan, traitement de montage soumis à revue avant implémentation.

| Besoin | Existant | Décision |
|---|---|---|
| Observation fichiers ouverts | scripts/build.py busy_paths/opened_paths | Unifier, pas de nouveau service ni scanner /proc maison |
| Captures Git bornées | git_ok ; principe T3 executeRaw | Étendre le runner existant avec budget ; réutilisé pour Git/lsof |
| Motifs | worktree_reasons | Garder un seul moteur pour inventaire et prune |
| Informations communes | integration_reason/age_reason | Mémoire limitée à un passage, aucune base/cache durable |
| Imbrication | comparaison de toutes les paires | Index des ancêtres sur la liste déjà lue |
| Scores | AuditV15 session175 | Même grille, nouvelles preuves et périmètre déclaré |

## Gate avant tasks
- [x] Fonctions existantes et responsabilités recherchées.
- [x] Pas de nouveau service/table/dépendance.
- [x] Toute nouvelle aide interne doit porter une borne ou une règle observable.
