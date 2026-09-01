# Quickstart opérateur expert - SPEC-086

## Préconditions

- SPEC-084 et SPEC-085 sont déployées.
- Le checkout Bridget est explicitement connu de l'installation.
- Le projet système est lié en Docker et aucun agent n'est actif.
- Les travaux existants sont dans des branches/worktrees distincts.

## Déclaration

1. Ouvrir les réglages expert du serveur.
2. Déclarer le checkout Bridget comme projet système.
3. Vérifier l'attestation et la classification `exact_project + system_only`.
4. Vérifier qu'il n'apparaît pas comme parent dans Créer un projet.

## Validation disabled

1. Lancer un agent de lecture dans le projet système.
2. Vérifier qu'il peut inspecter sources et worktrees.
3. Vérifier qu'une écriture échoue au niveau du mount.
4. Arrêter proprement l'agent.

## Activation expert

1. Prévisualiser `dogfooding.bridget = enabled`.
2. Lire l'avertissement et confirmer la génération affichée.
3. Attendre la recréation et l'état enabled confirmé.
4. Lancer un agent dans un worktree dédié.
5. Vérifier que le checkout principal reste read-only, puis édition, tests et commit sur la branche non-main du worktree attribué.
6. Vérifier qu'aucune action de merge, installation ou restart n'est proposée automatiquement.

## Coexistence

1. Garder un autre worktree pour un agent externe hôte.
2. Vérifier que les deux travaux utilisent des branches et index distincts.
3. Tenter de réserver le même worktree deux fois et vérifier le refus.

## Désactivation

1. Arrêter tous les agents du projet système.
2. Désactiver le dogfooding et attendre la recréation.
3. Vérifier que les sources redeviennent read-only et que branches, commits et worktrees sont intacts.
