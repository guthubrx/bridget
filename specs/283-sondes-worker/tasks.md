# Chantier 283 : sondes-worker

Développement délégué sur demande de François le 12 novembre 2026.
Tâche native : 2268f373-1e6d-49e4-ba93-7e4447d4bf79. Le résultat a été lu et accusé.
Responsabilité : remote_worker_read.rs et remote_worker_tests.rs. Aucun commit ni déploiement délégué.

Instrumentation mapping/git/readiness proposée ; coût du hachage suspecté mais non mesuré sur KS. Aucun seuil ni garantie de fraîcheur modifié.

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

La mesure en production localise désormais le coût dans `readiness`. Sa réduction reste ouverte ; aucune optimisation de cette phase n’est livrée.
