# Plan 242

Réutilisation : managed_resume_context produit les deux cartes du wrapper ; managed_resume_worktree reste le contrôle Git existant. Aucun lecteur supplémentaire de manifeste ni nouvelle dépendance. La disponibilité des fichiers ne se déduit plus du résultat Git.

Modifier le seul texte et afficher le chemin existant avec le même échappement. Vérifier une lecture sans Git, le chemin absent et les cartes Git/injection existantes. Revue indépendante du diff, compilation Mac/Linux, distribution native avec tâches conservées, retour 241 compatible, publication, main/push/alignement et nettoyage borné. Réutiliser les preuves 241 hors changement.

Convergence : exigences couvertes par le renderer unique et les tests existants/242. Risques : un état Git indisponible ne certifie pas une vue ni ses fichiers. Le message conditionne la poursuite aux fichiers accessibles de l’espace attesté.
