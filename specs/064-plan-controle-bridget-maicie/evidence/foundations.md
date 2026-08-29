# Preuves des fondations

- Date : 2026-08-29

## Commandes exécutées

- `/home/moi/.cargo/bin/cargo test -p bridget-core --quiet`
- `/home/moi/.cargo/bin/cargo test -p bridget-transport --quiet`
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_store_test --quiet`
- `git diff --check`

## Verdicts observables

- `bridget-core` : 51 tests réussis.
- `bridget-transport` : 214 tests réussis, 1 ignoré.
- Magasin SQLite d'exécution : 5 tests réussis, incluant migration additive, reprise après crash et refus d'un événement tardif.
- Aucun espace final, erreur de whitespace ou service de production touché.


## Checkpoint US1 partiel

- `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_store_test --test work_submission_test --test execution_lifecycle_test --test managed_wrapper_test --quiet` : 1, 7, 3 et 1 tests réussis.
- `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib ui::tests::projection_ui_distingue_connexion_vitalite_tour_attente_et_file --quiet` : 1 test réussi.
- `node -e` ciblé sur `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/assets/ui/app.js` : normalisation et présentation des âges, tour et file vérifiées.
## Checkpoint US1 - admission, projection et observabilité

- /home/moi/.cargo/bin/cargo check --workspace --quiet : succès. Les avertissements préexistants de visibilité ManagedStopControl, fonctions non appelées et imports Maicie restent présents.
- /home/moi/.cargo/bin/cargo test -p bridget-daemon --test execution_store_test --test execution_lifecycle_test --test work_submission_test --test managed_wrapper_test --quiet : 1, 8, 4 et 1 tests réussis. Le nouveau scénario wrapper prouve une issue bornée après silence fournisseur et une saturation provider_queue_full sans contamination du premier tour.
- /home/moi/.cargo/bin/cargo test -p bridget-daemon --test ui_relay_test relais_ui_expose_separement_connexion_vitalite_tour_attente_et_file --quiet : 1 test réussi. Il passe par le daemon réel, l'admission QueueOnly et TriggerTurn, puis vérifie les projections busy, vitalité, running, waiting_approval, âge du progrès et profondeur de file.
- /home/moi/.cargo/bin/cargo test -p bridget-daemon --lib metrics_execution_restent_bornees_et_sans_labels_de_contenu --quiet : 1 test réussi. Les compteurs sont bornés, ne portent aucun corps de message et couvrent admission, démarrage, latence, refus, échec de remise et saturation.
- /home/moi/.cargo/bin/cargo test -p bridget-transport --quiet : 214 tests réussis, 1 ignoré.
- /home/moi/.cargo/bin/cargo test -p bridget-core --quiet : 51 tests réussis.
- git diff --check : succès.

Limite constatée hors code : quatre anciens tests UI qui lancent le binaire avec son port par défaut échouent car 127.0.0.1:17888 est déjà occupé par le relais existant sur l'hôte. Le nouveau test utilise explicitement le port 0 et reste vert. Aucun service ni configuration de production n'a été modifié.

T025, T028 à T032 et T035 restent ouverts. Le point de blocage fonctionnel est documenté dans le handoff : les intentions SteerCurrent, InterruptAndStart et ControlOnly ne disposent pas encore d'un contrat de commande corrélé de bout en bout. Elles sont donc encore refusées, ce qui évite toute transformation silencieuse.

- La double écriture est toujours inactive par défaut : aucune de ces écritures ne modifie le comportement productif sans bascule admise.
- Les avertissements Rust historiques `ManagedStopControl` et fonctions UI non appelées sont présents, mais aucune erreur de compilation ou de test ciblé n'est observée.

## Contrat ControlExecution v1 - complément US1

- Commande publique : `version`, `command_id`, `execution_id`, `generation`, `revision`, `operation` et un `message` canonique obligatoire uniquement pour `SteerCurrent`.
- Réponse initiale : `OutcomeUnknown` signifie que Bridget a écrit la commande au wrapper mais ne connaît pas encore son résultat. Le wrapper retourne ensuite `ControlExecutionReported`; Bridget persiste `accepted` ou une raison structurée, notifie le client actif et rend le même résultat par `Lookup(operation_kind=execution_control)` après reconnexion.
- Sûreté : la cible est relue depuis l'exécution durable, le wrapper accusant doit être celui de cette cible, et le rejeu aux mêmes octets ne réémet pas l'action. Les anciens envois sans origine, intention ni référence conservent leurs octets canoniques; ces nouveaux champs rendent une commande distincte.
- Preuves : `cargo test -p bridget-daemon --lib daemon::temoin_commande_controle_est_recue_puis_resolue_par_le_wrapper_cible -- --exact` (1 succès), `cargo test -p bridget-daemon --test execution_store_test` (9 succès), `cargo test -p bridget-transport` (214 succès, 1 ignoré).
