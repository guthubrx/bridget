# Audit de réutilisation - SPEC-075

## Verdict

**PASS avec corrections de sémantique.** Le socle nécessaire existe déjà. La
feature ne justifie ni nouveau service, ni nouvelle base, ni nouveau mécanisme
de terminaison.

## Éléments réutilisés

| Besoin | Existant | Décision |
|---|---|---|
| Arrêter un groupe | `ManagedStopControl`, marqueurs et superviseur | Réutiliser sans signal UI direct |
| Corréler un arrêt | `StopOrder`, `StopResult`, `command_id` | Conserver et corriger la persistance |
| Lancer une génération | saga `SpawnOrder` et `submit_spawn_from_resolved` | Réutiliser pour relaunch |
| Conserver la définition | `DesiredEquipier.resolved_definition` | Réutiliser comme autorité figée |
| Écriture durable | `DesiredStateStore` | Étendre au schéma 4 |
| Afficher l'identité | `AgentInfo` et fiche SPEC-071 | Conserver |
| Actions locales | relais UI SPEC-073 | Étendre à deux routes |
| Mise à jour dynamique | polling incrémental existant | Conserver, aucun reload global |
| Historique | journaux et sessions existants | Ne pas toucher |

## Écarts constatés

1. `invalidate_for_stop` retire `fleet.json` et le roster. C'est incompatible
   avec un arrêt réversible.
2. `fleet.json` ne contient que les agents persistants à reprendre. Il ne peut
   pas projeter les agents arrêtés ou les agents non persistants après restart.
3. `persistent: Option<bool>` sert actuellement de preuve de gestion et de
   politique de reprise. Le schéma 4 doit conserver la valeur même arrêté.
4. `agent_infos` dépend d'une présence mémoire pour afficher `stopped`; cette
   présence disparaît au redémarrage.
5. La compensation d'échec retire une entrée par nom sans vérifier la
   génération. Une relance échouée pourrait effacer la définition précédente.
6. L'UI nomme « Décommissionner » l'appel à `/v1/agents/stop`.
7. Aucun ordre de relance ou de décommissionnement réel n'existe.
8. Le nom est l'identité logique et la clé de consultation de l'historique. Un
   nom décommissionné doit donc rester réservé tant qu'une purge séparée n'a pas
   eu lieu.

## Code à ne pas dupliquer

- Arrêt de processus ou groupe, délais et issue forcée.
- Validation de registre, environnement, `cwd`, provider et permissions.
- Écriture atomique et permissions de `fleet.json`.
- Gestion de fenêtre, focus et confirmation de la fiche.
- Rafraîchissement de snapshot et stabilité du scroll.

## Inconnues levées

- La dernière génération gérée et sa définition sont disponibles dans SQLite.
- L'historique est distinct de `fleet.json` et peut survivre au retrait.
- La relance peut utiliser `submit_spawn_from_resolved`.
- Un agent arrêté n'est aujourd'hui durable que tant que le daemon ne redémarre
  pas, ce qui impose la migration pré-déploiement.
