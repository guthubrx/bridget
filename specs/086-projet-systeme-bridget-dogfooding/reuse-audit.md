# Audit de réutilisation - SPEC-086

## Verdict

EXTEND - Le dogfooding est une composition d'un rôle projet protégé, d'un réglage fermé et de la mutabilité des mounts existants.

## Inventaire audité

| Besoin | Composant existant | Décision |
|---|---|---|
| Projet unique | Registre de projets SPEC-065 | Ajouter un rôle et une contrainte d'unicité. |
| Emplacement protégé | Catalogue SPEC-084 | Utiliser `exact_project + system_only`. |
| Isolation | Runtime partagé SPEC-066/085 | Exiger Docker et étendre le résolveur de mounts. |
| Mutabilité | `ProjectMount.writable` | Dériver du mode de dogfooding, sans nouveau type de volume. |
| Transition | Epoch/recreate du runtime | Réutiliser avec publication tardive et compensation. |
| Réglage expert | Centre de contrôle SPEC-080 | Ajouter une clé fermée avec preview/apply. |
| Concurrence | Git linked worktrees et réservations de travail | Ajouter une réservation canonique de worktree, pas un lock de dépôt. |

## Preuves de lacune

- Le domaine projet actuel ne porte aucun rôle système.
- Aucun réglage `dogfooding` ou `system_project` n'existe.
- Tous les mounts code/worktrees actuels sont marqués writable pour les environnements Docker ordinaires.
- Aucun invariant central n'interdit encore le checkout Bridget à un autre projet.
- Les worktrees existent mais ne sont pas réservés comme unité de travail active.

## Éléments à ne pas dupliquer

- Nouveau dépôt Maicie: Maicie est dans `plugins/maicie` du même checkout.
- Nouveau runtime ou conteneur: utiliser celui du projet.
- Nouveau service de rôles/secrets: inutile.
- Copie ou synchronisation de dépôt: Git/worktree reste la source de vérité.
- Workflow de livraison: explicitement hors périmètre.

## Nouveaux éléments justifiés

- `ProjectRole::BridgetSystem` pour l'unicité et les invariants.
- Attestation locale de checkout sans exécution.
- `dogfooding.bridget` pour la décision opérateur.
- Digest de mounts système et réservation de worktree.

## Conclusion

Le modèle minimal est un projet système unique et un mode binaire. Une matrice de permissions par agent, une durée ou un daemon de self-improvement n'ajouteraient aucune garantie nécessaire à ce stade.

