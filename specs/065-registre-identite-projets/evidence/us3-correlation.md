# Preuve US3 - Corrélation projets, agents et exécutions

## Oracles exécutés

- Un snapshot avec `ProjectReference` survit à la fermeture et au redémarrage
  du store d'exécutions.
- Une référence absente dans les données historiques reste `None`; elle n'est
  jamais inférée depuis `domain` ou un chemin.
- Un rebind augmente la génération courante de la liaison sans modifier la
  génération déjà portée par une réservation de flotte active.
- Une reprise de flotte, les liens agent-parent et les événements runtime
  délégués conservent la référence initiale.
- Un cwd dans la racine ou dans un worktree Git relié est admis; un voisin est
  refusé avant l'appel provider.
- Maicie refuse une divergence explicite entre le projet d'une délégation et
  celui d'une exécution sans écrire de transition métier.

## Compatibilité et migrations

`ProjectReference` est `Option` sur les trames, ordres, snapshots, projections,
curseurs et événements durables. Les migrations des bases d'exécution et
d'idempotence sont additives. La migration d'idempotence est v6 afin de ne pas
réutiliser v5, déjà réservée aux liens de livraison.

## Résultat

Les tests ciblés listés dans `final-validation.md` passent. La corrélation est
donc durable pour les nouveaux flux, tandis que les flux antérieurs restent
lisibles sans projet.
