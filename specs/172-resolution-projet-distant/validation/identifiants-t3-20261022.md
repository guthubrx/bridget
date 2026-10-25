# Liaison des identifiants adaptateur de bureau — 22 octobre 2026

Le maître Bridget exécute le correctif `172-31f42307` depuis 2026-10-22T09:36:59.188552+02:00.
Source de production : `31f42307efd5c3d490a2fdb8e2c09c49f0b0a30b`. Le numéro de paquet reste 0.4.1 ;
le build chargé distingue cette correction.

## Défaut et correction

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md#extrait-2).

## Validation et déploiement

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md#extrait-3).

Le premier préflight a refusé une tâche locale encore active. Elle a ensuite terminé
avec `cleanup_done=true`, sans intervention. L'activation a repris après constat de
zéro descendant local. Les commandes distantes déjà `released/native_ready` restent
exécutées par le même runtime worker ; `prepare_restart` conserve leurs identités.
Le contrôle ne prouvait pas la survie des fournisseurs. Le journal ultérieur a révélé
leur arrêt sur coupure du maître ; cette affirmation de continuité est retirée.
Le correctif du wrapper et sa recette complète sont suivis dans le lot suivant.

## Parcours réel et limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/identifiants-t3-20261022.md#extrait-4).

L'ancienne tâche façades `837972ad-ad58-4512-9a71-fd2843ab5be5` reste `failed`,
`cleanup_done=true`, `guard_released=true`, avec son worktree propre conservé.
Elle appartient à chargement ; un autre propriétaire ne gagne pas son accès.
Aucune tâche n'a été recréée.

Preuves privées : /home/moi/.cache/bridget-identifiants172 et
/Users/moi/.cache/bridget-identifiants172. Sauvegarde précédente conservée :
/Users/moi/.cache/bridget-identifiants172/bridget-before.
