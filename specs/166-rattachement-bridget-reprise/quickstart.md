# Validation rapide — SPEC-166

1. Exécuter le test de rejeu ciblé. Il doit prouver la reprise avec la preuve B avant le tour.
2. Exécuter les tests de l’adaptateur Codex et le typecheck du serveur.
3. Construire et signer l’application T3.
4. Sauvegarder l’application installée.
5. Installer la nouvelle application.
6. Relancer T3 une seule fois après vérification des processus actifs.
7. Reprendre un ancien fil T3.
8. Exécuter un appel Bridget dans son premier tour.
9. Vérifier que l’appel réussit sans réinstallation du pont.
10. Vérifier que le fil conserve son identité et son historique.
