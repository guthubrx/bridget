# Implémentation - SPEC-075 cycle de vie complet des agents gérés

## Statut

Implémentation terminée et vérifiée dans le worktree isolé
`/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents`.

Aucun commit, merge, push, déploiement ou redémarrage du daemon de production
n'a été effectué.

## Résultat utilisateur

- `Arrêter` termine le groupe supervisé, conserve l'agent dans la barre en
  état `stopped` et empêche sa reprise automatique après redémarrage.
- `Relancer` crée une nouvelle génération sous le même nom à partir du runtime,
  du projet, du répertoire et de la politique de persistance figés.
- `Décommissionner` arrête d'abord un agent actif, cache ensuite sa tombstone,
  conserve son historique et réserve son nom.
- Les agents historiques peuvent être adoptés explicitement avec
  `bridget adopt-stopped <nom>...` uniquement si une ancienne génération gérée,
  connectée et complètement définie est prouvée dans le journal.
- La CLI, le protocole daemon, le relais HTTP local et l'interface utilisent les
  mêmes ordres typés et les mêmes verdicts.

## Garanties ajoutées

- Schéma `fleet.json` version 4 compatible en lecture avec les versions 1 à 3.
- États durables fermés `running`, `stopped` et `decommissioned`.
- Écritures atomiques et compensations conditionnées par commande et
  génération.
- Échec de relance sans perte de la dernière définition arrêtée.
- Projection des agents arrêtés depuis le disque sans présence mémoire.
- Réservation temporaire du nom pendant un décommissionnement afin de fermer
  la course avec une relance concurrente.
- Refus explicite d'une action UI inconnue, sans repli vers une route
  destructive.
- Conservation vérifiée du journal après arrêt, relance, décommissionnement et
  réouverture du superviseur.

## Surface modifiée

- Modèle durable et migration: `desired_state.rs`.
- Saga, journal idempotent et adoption: `fleet.rs`, `idempotency.rs`,
  `lifecycle.rs`.
- Routage, supervision, courses et projection: `daemon.rs`.
- Contrat filaire et CLI: `protocol.rs`, `cli.rs`.
- Relais HTTP et interface: `ui.rs`, `app.js`, `theme.css`.
- Preuve de redémarrage: `managed_parity_test.rs`.

## Migration opérateur

1. Avant le premier redémarrage avec le schéma 4, lister les anciens agents
   arrêtés qui doivent rester administrables.
2. Pour chaque nom explicitement validé, exécuter
   `bridget adopt-stopped <nom>`.
3. Les refus `no_managed_history` ou `incomplete_history` doivent rester des
   refus. Il ne faut pas recréer une identité à partir de son seul nom.
4. Construire et déployer le binaire seulement après merge validé.
5. Après redémarrage, vérifier que les agents arrêtés sont visibles, que les
   tombstones ne le sont pas et qu'aucun agent arrêté n'est repris.

## Hors périmètre assumé

- Purge définitive de l'identité et de l'historique.
- Libération d'un nom décommissionné.
- Actions en masse.
- Pause ou drainage d'un tour avant arrêt.
- Renommage, clonage ou changement de fournisseur, modèle ou runtime.
- Extension du cycle de vie aux anciens agents TMUX non gérés.
