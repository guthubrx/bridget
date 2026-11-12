# Chantier 284 : continuation-locale

Développement délégué sur demande de François le 12 novembre 2026.
Tâche native : 9ec45865-1e3d-41b2-9c0b-c7c944dfa97e. Le résultat a été lu et accusé.
Responsabilité : daemon/native_delegation.rs et daemon/native_delegation_tests.rs. Aucun commit ni déploiement délégué.

Les cas distants sont déjà couverts. Correction proposée du statut local quand la présence de l’enfant manque, sans écriture du store ni prolongation de garde.

- [x] Examiner le parcours existant et produire un delta ciblé.
- [x] Conserver le diff et les tests proposés, sans toucher aux autres fichiers.
- [x] Revoir le delta et obtenir une preuve de régression utile.
- [x] Qualifier dans le runtime cumulatif, sans compiler le programme réservé de main.
- [x] Intégrer, publier et activer le correctif validé.

État : intégré dans main, qualifié puis activé avec la version 0.8.48
(`284-sondes-continuation-r1`). Les propositions déléguées ont été revues et
corrigées par le mainteneur ; 31 tests ciblés par plateforme, Clippy strict
et releases ont réussi sur Mac et Debian 12. Les sources SYS sont propres et
alignées. Les propositions originales restent conservées comme preuves.

Publication : https://github.com/guthubrx/bridget/releases/tag/v0.8.48
Commit du code : `1e917ec4ae279ee90d32f80d02ae8de11ded1c44`.
