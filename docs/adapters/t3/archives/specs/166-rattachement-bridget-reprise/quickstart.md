# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/quickstart.md](../../../../../../specs/166-rattachement-bridget-reprise/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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
