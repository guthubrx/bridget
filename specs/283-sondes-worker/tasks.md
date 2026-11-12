# Chantier 283 : sondes-worker

Développement délégué sur demande de François le 12 novembre 2026.
Tâche native : 2268f373-1e6d-49e4-ba93-7e4447d4bf79. Le résultat a été lu et accusé.
Responsabilité : remote_worker_read.rs et remote_worker_tests.rs. Aucun commit ni déploiement délégué.

Instrumentation mapping/git/readiness proposée ; coût du hachage suspecté mais non mesuré sur KS. Aucun seuil ni garantie de fraîcheur modifié.

- [x] Examiner le parcours existant et produire un delta ciblé.
- [x] Conserver le diff et les tests proposés, sans toucher aux autres fichiers.
- [ ] Revoir le delta et obtenir une preuve de régression utile.
- [ ] Qualifier dans le runtime cumulatif, sans compiler le programme réservé de main.
- [ ] Intégrer, publier et activer le correctif validé.

État : proposition de code, pas une correction livrée. Les tests Rust n’ont pas
été exécutés par le responsable délégué. Les fichiers modifiés sont conservés
sur SYSLE2 dans /home/moi/.local/share/bridget-projects/bridget/repository.
Copie du delta : /Users/moi/.cache/bridget282-trafic/development-283.patch.
