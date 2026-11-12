# Chantier 284 : continuation-locale

Développement délégué sur demande de François le 12 novembre 2026.
Tâche native : 9ec45865-1e3d-41b2-9c0b-c7c944dfa97e. Le résultat a été lu et accusé.
Responsabilité : daemon/native_delegation.rs et daemon/native_delegation_tests.rs. Aucun commit ni déploiement délégué.

Les cas distants sont déjà couverts. Correction proposée du statut local quand la présence de l’enfant manque, sans écriture du store ni prolongation de garde.

- [x] Examiner le parcours existant et produire un delta ciblé.
- [x] Conserver le diff et les tests proposés, sans toucher aux autres fichiers.
- [ ] Revoir le delta et obtenir une preuve de régression utile.
- [ ] Qualifier dans le runtime cumulatif, sans compiler le programme réservé de main.
- [ ] Intégrer, publier et activer le correctif validé.

État : proposition de code, pas une correction livrée. Les tests Rust n’ont pas
été exécutés par le responsable délégué. Les fichiers modifiés sont conservés
sur SYSLE2 dans /home/moi/.local/share/bridget-projects/bridget/repository.
Copie du delta : /Users/moi/.cache/bridget282-trafic/development-284.patch.
