# Lot capacité — 23 octobre 2026

Décision utilisateur : 128 et organiser le nettoyage. Deux contrôles de production réunis autour de MAX_WORKTREES, sans changement de protocole ou de permissions. Portée conservée : `.worktrees` du checkout cible, sous-dossiers compris. Aucun paramètre par projet ajouté dans ce lot.

Quatre tests distincts réussis : diagnostic public workspace_limit ; préflight avec 127 worktrees puis un imbriqué ; union Git/tâches/réservations à la 128e place ; 40 demandes concurrentes après 120 worktrees existants (au plus huit admissions). Le test préflight échoue sur le plafond ancien. Pas de suite complète.

Revue indépendante : logique de production conforme ; deux comptes du test ancien restaient à 33 et 30. Ils sont devenus 129 et 126. Le test concurrent a été renforcé avec 120 emplacements occupés, puis sa somme finale adaptée et rejouée avec succès. Le harnais a nécessité un TMPDIR court pour ses sockets Unix ; aucun changement de production pour ce point.

Nettoyage : inventaire privé de 48 worktrees secondaires Opus2D. Aucun retrait automatique sûr établi : cinq chemins utilisés, sept modifiés/non suivis, 24 contenant des fichiers ignorés, 45 non fusionnés dans main ; catégories recouvrantes. Plan établi pour clôture propriétaire, intégration/conservation des commits, archivage des sorties et retrait Git sans force. Aucune suppression.

Livré : source b38c96023f2d1d04eb9e546813b59b9717a32e7e poussée sur GitHub ; release optimisée 174-b38c9602 active sur maître Mac et worker Cartae. Maître PID 75404, worker PID 3998948. Processus distants existants et application T3 conservés. Client Cartae aligné. Avis envoyés aux responsables BUS, monuments et intégration Opus2D ; reçus de remise en cours, sans prétendre à une lecture. L’audit général 174 reste ouvert ; ces quatre tests ne sont pas sa conclusion.
