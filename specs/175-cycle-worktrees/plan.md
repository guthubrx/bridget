# Plan — Session 175
## Contexte technique
Rust/serde pour la projection de tâche ; Python standard pour scripts/build.py. Aucun protocole worker, schéma SQLite ou réglage de droits modifié.
## Constitution
Réutilisation, preuves ciblées, chemins absolus dans les rapports, développement Cartae, versions publiques EN/FR, sources T3 lues. Nettoyage sans force et sans perte. Pipeline autonome autorisé par my-specify-all.
## Choix
1. Étendre task_view et la restitution de fin de tâche avec une explication de conservation dérivée des champs existants (continuable, cleanup_done, rapport courant, dirty, commits). Projection seule : aucun nouveau champ dans WorkspaceRecord, qui refuse les champs inconnus.
2. Étendre scripts/build.py par un inventaire worktrees en lecture seule pour un --repo explicite, --base-ref (main par défaut) et --json. Réutiliser worktree_entries, git_ok, busy_paths et les conditions de prune_merged_worktrees. Montrer la capacité du cwd sélectionné et distinguer le registre entier.
3. Partager les observations de nettoyage avec le chemin existant quand cela supprime une divergence, sans nouvelle autorité de retrait. Les candidats ne certifient pas l'absence de tâches ou de fils ; les contrôles restent conservateurs.
4. Mettre à jour les guides de délégation et d'entretien EN/FR existants. Choix shared explicite seulement. Pas de changement du rangement des espaces existants.
## Réutilisation
- crates/bridget-daemon/src/daemon/native_delegation.rs : task_view, workspace_report_ready, append_workspace_result.
- crates/bridget-daemon/src/delegation.rs : Task, WorkspaceRecord inchangés.
- scripts/build.py : worktree_entries, busy_paths, parse_open_paths, git_ok, prune_merged_worktrees.
- scripts/tests/test_build_cleanup.py : fixtures Git et processus réels.
- crates/bridget-daemon/src/daemon/native_workspace154_tests.rs : fixtures de tâche et restitution.
- docs/delegation-native.md et docs/delegation-native.fr.md.
## Découpage
ROOT possède Rust, artefacts, docs, intégration et livraison. Un livrable distinct Python peut être confié via Bridget après audit de réutilisation. Une contre-revue en lecture seule examine plan puis diff.
## Validation
Tests Rust ciblés de projection autorisée ; tests Python avec vrais dépôts et motifs, seuils, locked/nested/ignored et échec des contrôles ; recette réelle d'inventaire en lecture seule sur Cartae. Pas de mission métier supplémentaire, pas de campagne globale.
## Charge future et complexité
Une observation par worktree, inventaire processus une fois. Aucun scan dans task_view, aucun cache dérivé persisté, aucun nouveau service. Les motifs sont fermés et documentés. Le rangement séparé de T3 est différé : il casserait les ancrages de permissions et nécessiterait une migration.
## Livraison
Commit ciblé, intégration main, push github vérifié, release optimisée Mac/Cartae avec procédures existantes et services T3 préservés. Nettoyage seulement du lot avec preuves d'absence d'usage, sinon conservation motivée.

Précision capacité : observation des inscriptions Git dans le périmètre choisi ; les réservations en cours et dossiers non enregistrés restent contrôlés par l’admission, sans équivalence de total promise.

## Précisions après contre-revue du plan
Un seul évaluateur de motifs doit alimenter inventaire et retrait existant. La commande worktrees retourne avant toute purge. Git sans verrou optionnel ; erreurs et délais deviennent inconnu. Le rappel de restitution reste une consigne constante de relecture du statut, jamais un motif dynamique figé. Décisions détaillées : adversarial-review.md.
