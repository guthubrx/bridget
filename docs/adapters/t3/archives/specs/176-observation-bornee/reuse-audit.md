# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/reuse-audit.md](../../../../../../specs/176-observation-bornee/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Besoin | Existant | Décision |
|---|---|---|
| Observation fichiers ouverts | scripts/build.py busy_paths/opened_paths | Unifier, pas de nouveau service ni scanner /proc maison |
| Captures Git bornées | git_ok ; principe T3 executeRaw | Étendre le runner existant avec budget ; réutilisé pour Git/lsof |
| Motifs | worktree_reasons | Garder un seul moteur pour inventaire et prune |
| Informations communes | integration_reason/age_reason | Mémoire limitée à un passage, aucune base/cache durable |
| Imbrication | comparaison de toutes les paires | Index des ancêtres sur la liste déjà lue |
| Scores | AuditV15 session175 | Même grille, nouvelles preuves et périmètre déclaré |
