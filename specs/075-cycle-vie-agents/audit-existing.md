# Audit de l'existant avant implémentation - SPEC-075

## Verdict global

**PASS avec écarts bloquants identifiés.** Le socle technique est réutilisable,
mais l'implémentation actuelle ne peut pas satisfaire la sémantique demandée
sans faire évoluer la source durable et le protocole.

## Fichiers et comportements audités

| Zone | Faits observés | Verdict |
|---|---|---|
| `desired_state.rs` | Schéma 3, entrées uniquement persistantes, aucune notion `stopped` | À étendre |
| `fleet.rs` | `invalidate_for_stop` supprime l'entrée et le roster | À corriger |
| `lifecycle.rs` | `submit_spawn_from_resolved` sait relancer une définition figée | Réutilisable |
| `daemon.rs` | `StopOrder` attend le superviseur; `stopped` vient seulement de la présence mémoire | À étendre |
| `protocol.rs` | Ordre et issue stop typés; aucune relance ou décommission | À étendre |
| `cli.rs` | `bridget stop` existe | À généraliser |
| `ui.rs` | `/v1/agents/stop` protégé et corrélé | Réutilisable et extensible |
| `app.js` | Panneau, confirmation et retours existent, mais le libellé est faux | À corriger |
| `theme.css` | Styles du panneau et de la confirmation existent | Réutilisable |

## Données runtime observées

- L'agent `priorite-reponse-codex` est présent en mémoire sous `stopped`, sans
  processus résiduel.
- Son champ de gestion n'est plus projeté après l'arrêt, parce que le roster a
  été oublié.
- Sa dernière génération gérée et sa définition résolue existent encore dans
  SQLite.
- Un redémarrage avec le code actuel ferait disparaître cette présence arrêtée,
  car elle n'est pas dans `fleet.json`.

## Compatibilité avec les specs antérieures

- SPEC-009 impose que `stop` retire le marqueur de reprise. La nouvelle
  représentation respecte l'intention, mais remplace la suppression de
  l'identité par `lifecycle_state=stopped`.
- SPEC-073 a donné au stop le libellé de décommissionnement. SPEC-075 le
  supersède explicitement.
- SPEC-071 et l'identité fournisseur ne nécessitent aucun changement de source
  ou d'inférence.
- Le rafraîchissement dynamique récemment livré peut refléter les nouvelles
  transitions sans reload de page.

## Risques de conflit concurrent

- Le worktree part de `origin/main` au commit `aba60f0`.
- Un autre worktree `session-074-bridget-desktop` existe, mais aucun fichier de
  ce worktree n'est modifié ici.
- La livraison future devra intégrer la tête de `main` avant toute fusion.
- Aucun commit, merge, push, installation ou redémarrage production n'est
  autorisé par le pipeline courant.

## Gate de réutilisation

- [x] Aucun second superviseur de processus.
- [x] Aucun second fichier de flotte.
- [x] Aucune nouvelle table nécessaire au fonctionnement nominal.
- [x] Même source d'identité et même snapshot UI.
- [x] Même garde locale et même token de session.
- [x] Compensation par génération exigée avant relance.
- [x] Migration héritée explicitement séparée du fonctionnement nominal.
