# Implémentation - SPEC-086 Projet système Bridget et dogfooding expert

## Invariants de livraison

- Le projet système est unique, explicite et reste séparé de tous les projets standard.
- Le mode dogfooding est désactivé par défaut et n'est jamais déduit d'un chemin ou d'un nom.
- Le checkout principal Bridget reste lecture seule. Seul un linked worktree non-main, attribué par lease, peut être écrivable.
- La lease est exclusive, persistée et libérée au terme de l'agent qui la détient.
- Le mode expert n'autorise jamais automatiquement merge, push, install, restart ni deploy.

## Statut de preuve

Les sections sont complétées uniquement après code et tests ciblés. Aucun daemon actif n'est modifié par cette SPEC.

- `ProjectRole` est une projection fermée, rétrocompatible avec les anciennes
  projections qui deviennent `standard` par défaut. Le registre SQLite réserve
  un seul rôle `bridget_system` et refuse toute seconde déclaration.
- L'attestation structurelle vérifie le checkout principal sur `main`, un
  worktree lié non-main, le même git common dir et la présence du worktree
  dans l'inventaire Git. Elle n'exécute aucun fichier du dépôt.
- Les plans de mounts gardent le checkout principal lecture seule. En mode
  disabled, le worktree et le common dir sont également lecture seule; en mode
  enabled, ils deviennent le domaine explicitement coopératif et le digest de
  topologie change.
- Preuve Docker ciblée exécutée avec l'image
  `bridget-project-runtime:spec085-v4` : écriture refusée dans le worktree en
  mode disabled, écriture refusée dans le checkout principal en mode enabled,
  écriture effective dans le linked worktree en mode enabled. Le conteneur et
  le checkout Git temporaires ont été supprimés après le test.
- Preuve multi-worktree exécutée avec la même image : un processus dans le
  conteneur a écrit uniquement dans `feature-bridget`, tandis qu'un processus
  hôte a écrit uniquement dans `feature-external`. Les deux branches et leurs
  index Git ont gardé des statuts distincts. Le dépôt temporaire et le
  conteneur éphémère ont été supprimés après le test.

## Validation opérateur restant volontairement manuelle

Le parcours du `quickstart.md` contre le projet système réel reste à valider
par l'opérateur avant toute livraison. Il implique une décision explicite
d'activation du mode expert et ne doit pas être simulé par un daemon actif.
