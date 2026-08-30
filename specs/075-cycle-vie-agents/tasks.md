# Tasks - SPEC-075 cycle de vie complet des agents gérés

## Règles d'exécution

- Worktree exclusif:
  `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents`.
- Aucun commit, merge, push, déploiement ou redémarrage production automatique.
- Toute terminaison passe par le superviseur existant.
- Une tâche n'est cochée qu'après test ou preuve associée.

## Phase 1 - Contrats et modèle durable

- [x] **T001** Ajouter
  `DesiredLifecycleState { Running, Stopped, Decommissioned }`, les valeurs
  serde et les tests dans `crates/bridget-daemon/src/desired_state.rs`.
- [x] **T002** Passer `FLEET_SCHEMA_VERSION` à 4 et ajouter `persistent` et
  `lifecycle_state` à `DesiredEquipier`, avec valeurs par défaut compatibles
  schémas 1 à 3.
- [x] **T003** Ajouter les mutations atomiques ciblées de `DesiredStateStore`:
  marquage stopped, retrait conditionné par commande et génération, transition
  explicite vers decommissioned.
- [x] **T004** Ajouter les tests schema 1, 2, 3 et 4, validation des états et
  préservation des permissions/écriture atomique.
- [x] **T005** Persister dans `register_connected_observed` tous les agents
  gérés, y compris `persistent=false`, en état `running`.
- [x] **T006** Remplacer les suppressions de compensation par un retrait
  conditionné à la génération possédée afin de préserver une définition
  `stopped` antérieure.
- [x] **T007** Ajouter aux tests fleet les transitions running vers stopped,
  échec de relance sans perte, décommissionnement caché et réservation du nom.

## Phase 2 - Reprise et projection

- [x] **T008** Réconcilier au démarrage `running + persistent=false` en
  `stopped` avant toute reprise.
- [x] **T009** Filtrer les candidats de reprise sur
  `lifecycle_state=running && persistent=true` et ignorer les entrées stopped.
- [x] **T010** Conserver les entrées en stopped sur échec de reprise ou mort
  spontanée au lieu de les retirer par nom.
- [x] **T011** Projeter les entrées stopped de `fleet.json` dans `AgentInfo`
  sans présence mémoire et sans doublon d'un agent live.
- [x] **T012** Tester la visibilité stopped après réouverture du store, la
  non-reprise et la projection de l'identité figée.

## Phase 3 - Protocole et daemon

- [x] **T013** Ajouter `RelaunchOrder`, `RelaunchResult`, `RelaunchOutcome`,
  `DecommissionOrder`, `DecommissionResult` et `DecommissionOutcome` dans
  `crates/bridget-transport/src/protocol.rs`.
- [x] **T014** Ajouter les tests de sérialisation aller-retour et de lecture
  fermée des nouvelles variantes.
- [x] **T015** Corriger `StopOrder` pour persister stopped avant l'attente du
  superviseur et conserver le roster géré.
- [x] **T016** Factoriser l'arrêt supervisé en primitive commune utilisable par
  stop et decommission, sans modifier les délais ou signaux SPEC-009.
- [x] **T017** Implémenter `RelaunchOrder` à partir de la définition figée, avec
  nouvelle génération, même nom et réponse uniquement après Connected.
- [x] **T018** Router succès et échecs de la saga vers `RelaunchResult` sans
  produire `SpawnAccepted` pour un demandeur de relance.
- [x] **T019** Implémenter `DecommissionOrder` pour agent arrêté et actif, avec
  transition cachée seulement après verdict d'arrêt positif.
- [x] **T020** Retirer la présence mémoire stoppée après décommissionnement afin
  que le prochain snapshot ne la réaffiche pas, et refuser un nouveau spawn du
  même nom.
- [x] **T021** Ajouter les logs de transition corrélés et les tests de courses:
  doublon, relance concurrente, arrêt pendant relance, timeout de
  décommissionnement.

## Phase 4 - Adoption héritée

- [x] **T022** Ajouter au store idempotent une lecture de la dernière génération
  gérée par nom, triée explicitement et limitée à une seule issue.
- [x] **T023** Implémenter l'adoption d'un nom arrêté explicitement fourni, avec
  preuve de génération gérée et définition complète, sans heuristique de nom.
- [x] **T024** Ajouter une commande opérateur `bridget adopt-stopped <nom>...`
  et un rapport adopté/refusé sans données sensibles.
- [x] **T025** Tester qu'un nom historique non fourni ou une génération externe
  n'est jamais adopté.

## Phase 5 - CLI et relais HTTP

- [x] **T026** Ajouter `bridget relaunch <nom>` et
  `bridget decommission <nom>` avec `command_id` optionnel et codes de sortie
  non nuls pour les refus.
- [x] **T027** Mutualiser la connexion, la corrélation et l'affichage des
  résultats sans casser `bridget stop`.
- [x] **T028** Ajouter les routes POST `/v1/agents/relaunch` et
  `/v1/agents/decommission`, protégées comme `/v1/agents/stop`.
- [x] **T029** Généraliser validation et mapping HTTP des issues fermées, sans
  retry et sans succès optimiste.
- [x] **T030** Étendre `ui_relay_test.rs` aux méthodes, tokens, versions,
  corrélations, succès, conflits, indisponibilité et timeout.

## Phase 6 - Interface

- [x] **T031** Renommer l'action existante en « Arrêter » et corriger tous les
  textes de confirmation et de résultat associés.
- [x] **T032** Implémenter la matrice d'actions actif, occupé, en reprise,
  arrêté, externe et mutation en cours.
- [x] **T033** Ajouter les confirmations distinctes « Relancer » et
  « Décommissionner », avec nom exact, conséquences et avertissement de travail
  en cours.
- [x] **T034** Généraliser l'état en vol et les retours par action et agent,
  sans doublon de requête ni mutation optimiste.
- [x] **T035** Conserver focus, Échap, clic extérieur, logos, scroll et polling
  incrémental de la barre latérale.
- [x] **T036** Ajouter les styles intégrés des trois actions et l'indication
  stopped sans reload visuel.
- [x] **T037** Étendre les tests Node à la matrice, aux dialogues, aux routes,
  aux erreurs et au rerender pendant une action.

## Phase 7 - Convergence et preuves

- [x] **T038** Exécuter l'analyse croisée spec, plan, modèle, contrat et tâches;
  corriger toute ambiguïté critique ou majeure.
- [x] **T039** Exécuter les tests ciblés protocole, desired state, fleet,
  lifecycle, daemon, UI et relais HTTP.
- [x] **T040** Exécuter la preuve intégrée stop, redémarrage, relaunch,
  decommission, redémarrage et conservation du journal sur daemon temporaire.
- [x] **T041** Exécuter `cargo fmt --check`, `cargo clippy` ciblé si disponible,
  `git diff --check` et les non-régressions SPEC-009, 071 et 073.
- [x] **T042** Faire une contre-revue hostile du diff par un autre fournisseur,
  corriger les constats critiques et majeurs, puis répéter une seconde fois si
  des corrections ont été nécessaires.
- [x] **T043** Produire `implementation.md`, `audit.md` et le handoff final avec
  preuves, limites et procédure de migration, sans commit automatique.

## Dépendances

```text
T001-T004 -> T005-T012 -> T013-T021 -> T022-T030 -> T031-T037
                                              \-> T038-T043
```

Les tâches UI dépendent du contrat HTTP stabilisé. L'adoption héritée peut être
développée après le cycle nominal, mais doit être terminée avant une mise en
production qui redémarrerait le daemon actuel.
