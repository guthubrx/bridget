# Quickstart opérateur - SPEC-084

## Préconditions

- Bridget Desktop possède au moins un profil serveur enregistré.
- Le serveur expose la capacité de catalogue v2.
- L'exploitant a décidé quels répertoires sont des workspaces et quels chemins sont des projets exacts.

## Parcours de validation

1. Ouvrir Créer un projet.
2. Vérifier que le serveur cible est visible et le changer une fois.
3. Vérifier que la liste des emplacements est rechargée depuis le nouveau serveur.
4. Choisir un workspace et saisir un nom de projet.
5. Vérifier la prévisualisation du chemin final avant confirmation.
6. Ouvrir Importer et vérifier qu'un exact project accepte seulement son propre chemin.
7. Vérifier qu'un emplacement système ne figure dans aucun des deux parcours standards.
8. Simuler une génération de catalogue concurrente et vérifier le refus sans création.

## Migration recommandée sur l'installation actuelle

- Déclarer un workspace choisi par l'opérateur pour les projets ordinaires.
- Classer `/home/moi/bridget-referent/bridget` comme projet exact système dans le lot SPEC-086.
- Ne pas promouvoir automatiquement l'ancienne racine en workspace.

## Résultat attendu

Le serveur et l'emplacement sont toujours visibles. Aucun projet n'est créé sous le checkout Bridget par défaut.

