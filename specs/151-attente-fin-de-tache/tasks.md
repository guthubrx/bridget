# Tâches - Session151 attente et boîte de fin de tâche

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R14), `data-model.md`,
`contracts/attente151.md`, `reuse-audit.md`, `quickstart.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : les tâches de la session 150 sont faites (rétention, `delivery_acked`,
lecture de phase T005, `reply_timeout`). Exécution après 150 et avant 152, dans
un worktree dédié, avec le socle 150 copié sans commit.

Règle : une tâche n'est cochée qu'après son test vert. Les tests marqués
« rouge d'abord » sont écrits et vus en échec avant le code. Résultat observable
indiqué à la fin de chaque tâche. Invariant à ne jamais casser : l'attente et le
statut n'écrivent ni `result_sent`, ni `failure_sent`, ni `delivery_acked`.

## Phase 1 - Préparation et vérifications

- [x] T001 Référence et faits à prouver avant tout code : (a) lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` et `batch` de `crates/bridget-daemon` avec la recette ; (b) relever le nom exact de la lecture de phase de remise livrée par 150-T005 dans `crates/bridget-daemon/src/idempotency/send_delivery.rs` ; (c) lister (`rg`) les tests qui figent les clés exactes de `task_view`, du catalogue de capacités ou de `DaemonIdentityReport`, et tous les sites qui construisent `DaemonIdentityReport` (`crates/bridget-daemon/src/daemon.rs:17844` et tests) ; (d) lire la documentation de Codex et de Claude Code sur le délai d'un appel d'outil MCP ; (e) vérifier par un test jetable qu'une connexion cliente du daemon reste ouverte 15 s sans trame ; (f) vérifier que `DaemonIdentityRequest` est accepté sans enregistrement (`crates/bridget-daemon/src/daemon.rs:14384`) et que `RequestWait` doit figurer dans les deux listes de trames (`:14384`, `:14615`) ; (g) énumérer par `rg "UPDATE tracked_requests"` tous les écrivains de l'état d'une demande suivie (annulation `store/ledger_requests.rs:318`, délai `:362`, réponse `:672`, guichet `store/service_events.rs:380` via `daemon.rs:16074`) pour que chacun incrémente le compteur ; consigner dans `specs/151-attente-fin-de-tache/validation/baseline.md`. Résultat : compteurs PASS/FAIL/ignorés, nom de la fonction de phase, liste des tests à adapter, délais d'hôtes, verdicts (e) et (f), liste des écrivains (g). Si (e) échoue, ajouter une tâche avant T010.

## Phase 2 - Fondations (bloquantes)

- [x] T002 [P] Dans `crates/bridget-transport/src/protocol.rs` : ajouter `NativeDelegationRequest::Wait { task_ids: Vec<String>, wait_timeout: Option<u64> }` (`#[serde(default, skip_serializing_if = "Option::is_none")]` sur `wait_timeout`), `WrapperToDaemon::RequestWait { request_id, wait_timeout }`, `DaemonToWrapper::RequestWaitResult { state, timed_out_wait, waited_secs, reply, reply_reason }` et `features: Vec<String>` (`#[serde(default)]`) à `DaemonIdentityReport` ; tests : les variantes existantes gardent exactement leurs octets, un aller-retour de chaque trame réussit, un `DaemonIdentityReport` sans `features` se décode avec une liste vide, un champ inconnu de `Wait` est refusé. Résultat : tests de protocole verts.
- [x] T003 [P] Ajouter dans `crates/bridget-daemon/src/delegation.rs` le type `WaitEpoch` (`Arc<(Mutex<u64>, Condvar)>` avec `get`, `bump`, `wait_changed(seen, tranche)`), le créer dans `DelegationStore::open`, l'exposer par un accesseur, et l'incrémenter dans `save_projected` et `insert_projected` de `crates/bridget-daemon/src/delegation_lineage.rs` après le commit, que `visible_changed` soit vrai ou non ; notifier aussi après `human_cancel` et `delete_tree` ; écrire en commentaire l'ordre des verrous (état puis compteur, jamais l'inverse) ; annoter la complexité O(1) ; tests `native151_attente_compteur_*` : incrément à chaque écriture, réveil d'un dormeur, retour à l'expiration de la tranche (`wait_changed` reçoit la durée de la tranche en paramètre ; la boucle passe `min(5 s, délai restant)`), `get` ne bloque pas pendant un `wait_changed` d'un autre fil. Résultat : tests du compteur verts.
- [x] T004 [P] Ajouter la colonne `reply_summary TEXT` à `tracked_requests` dans les deux définitions (`crates/bridget-daemon/src/idempotency.rs:580`, `crates/bridget-daemon/src/store.rs:165`) avec migration gardée par `pragma table_info` ; faire écrire le résumé dans le même `UPDATE ... SET state = 'answered'` de `mark_answered_in_transaction` (`crates/bridget-daemon/src/store/ledger_requests.rs:665`, nouveau paramètre `reply_summary: Option<&str>`, avec extension du wrapper `Store::mark_answered`, JSON `{id, from, summary, summary_truncated}` à 1024 octets sur frontière UTF-8) ; le calculer depuis le message de réponse dans `crates/bridget-daemon/src/idempotency/send_delivery.rs:748` et dans le chemin hérité `crates/bridget-daemon/src/daemon.rs:17778` ; le guichet (`store/service_events.rs:380`) passe `None` ; tests `native151_envoi_resume_fige_*` : ancienne base migrée, résumé écrit à la réponse, `NULL` pour le guichet, deux scopes différents avec le même identifiant de message vers le même destinataire gardent chacun leur réponse, un remplacement de la ligne du ledger ne change pas le résumé. Résultat : tests verts, tests de demandes suivies existants verts.
- [x] T005 [P] Annoncer `features: ["wait_v1"]` dans la réponse à `DaemonIdentityRequest` (`crates/bridget-daemon/src/daemon.rs:17844`, adapter les constructions listées en T001(c)) et ajouter dans `crates/bridget-daemon/src/communication/client.rs` la fonction `daemon_supports_wait(socket)` (une connexion, `DaemonIdentityRequest`, budget 3 s ; faux si `features` ne contient pas `wait_v1`) ; passer `registered_connection_until` en `pub(crate)` ; tests : daemon qui annonce, daemon simulé sans le champ, daemon qui ne répond pas dans les 3 s. Résultat : tests verts.

## Phase 3 - US2 Résultat structuré (P1)

**But** : un état de travail simple, un résumé stable, une référence au détail.
**Test indépendant** : filtre `native151_vue`.

- [x] T006 [US2] Rouge d'abord : écrire dans `crates/bridget-daemon/src/daemon/native_delegation.rs` les tests `native151_vue_*` : table `status` vers `work_state` (neuf statuts) ; résumé à 0, 2047, 2048 et 2049 octets ; caractère de trois octets à la coupe ; `summary` nul hors `result_available` ; deux lectures donnent le même résumé ; `detail` présent pour `result_available` et `failed` ; toutes les clés de `task_view` d'avant 151 sont présentes et inchangées. Résultat : tests vus en échec.
- [x] T007 [US2] Étendre `task_view` (`crates/bridget-daemon/src/daemon/native_delegation.rs:68-86`) avec `work_state`, `summary`, `summary_truncated`, `result_bytes`, `detail` ; adapter les tests listés en T001(c). Résultat : T006 vert, `native148` et `native149` verts.

## Phase 4 - US3 Boîte de fin de tâche (P1)

**But** : rendre lisible l'état de la remise, sans y toucher.
**Test indépendant** : filtre `native151_boite`.
**Dépend de** : Phase 3.

- [x] T008 [US3] Rouge d'abord : écrire `native151_boite_*` dans `crates/bridget-daemon/src/daemon/native_delegation.rs` : table de vérité des états (`pending`, `claimed` avec phase `dispatching` puis `indeterminate`, `delivered`, `null` pour tâche active ou `cancelled`) ; `native151_boite_status_n_ecrit_rien` (champs de livraison identiques avant et après `status`). Résultat : tests vus en échec.
- [x] T009 [US3] Ajouter la clé `mailbox` à `task_view` par une fonction de dérivation unique qui lit la phase de remise (nom relevé en T001(b)) pour `native-result-<task>` ou `native-failure-<task>` sous `issuer_scope(task_id)` ; annoter la complexité O(1). Résultat : T008 vert.

## Phase 5 - US1 Déléguer puis attendre (P1)

**But** : `bridget_task_wait`, lecture pure, sans polling, sans perte, sans annulation à l'expiration.
**Test indépendant** : filtres `native151_attente`, `native151_facade`.
**Dépend de** : T002 à T005, T007, T009.

- [x] T010 [US1] Rouge d'abord : écrire `native151_attente_*` dans `crates/bridget-daemon/src/daemon/native_delegation.rs` : retour immédiat si une tâche est déjà terminée ; retour sur événement en moins de 2 s ; expiration à 1 s avec `timed_out` rendue en moins de 1,5 s, enfant toujours actif et tâche inchangée ; retour sur événement avec une tranche réglée à 60 s (preuve du réveil sans filet) ; `native151_attente_lecture_pure` (avant/après une attente : `result_sent`, `failure_sent`, `delivery_acked`, `cleanup_done` identiques, pour résultat publié, remise en vol et remise indéterminée) ; `native151_boite_message_apres_attente_expiree` et `native151_boite_message_apres_attente_rendue` (exactement un message, attente ouverte, expirée, fermée ou ayant rendu le résultat) ; tâche d'un autre parent ou inconnue : `task_unavailable` pour tout l'appel ; `task_ids` vide, de 17 éléments ou en doublon : `invalid_request` ; `wait_timeout` à 0 et à 3601 : `wait_timeout_out_of_range` ; 65e attente : `wait_capacity` ; client fermé : attente libérée en 6 s ; résultat publié entre deux lectures, 100 répétitions ; attente permise pendant la pause. Résultat : tests vus en échec.
- [x] T011 [US1] Implémenter l'attente : traiter `NativeDelegationRequest::Wait` dans `handle_with_fact` (`crates/bridget-daemon/src/daemon/native_delegation.rs:280`) avant la prise du verrou ; boucle : `seen = epoch.get()`, verrou d'état, lecture des tâches avec `can_own_task`, relâche, puis `wait_changed(seen, min(5 s, délai restant))` (tranche passée en paramètre de la boucle pour les tests) ; compteur de 64 attentes avec garde RAII dans `crates/bridget-daemon/src/daemon.rs` (partagé avec T016) ; plomberie du socket depuis `handle_connection`, détection de fermeture par `recv(MSG_PEEK | MSG_DONTWAIT)` sans consommer un octet ni modifier O_NONBLOCK ; sortie à l'arrêt du daemon ; annoter la complexité (O(k), k au plus 16) ; aucune écriture. Résultat : T010 vert.
- [x] T012 [US1] Exposer l'outil et limiter la façade : dans `crates/bridget-daemon/src/mcp.rs` ajouter `MAX_WAITING_TOOL_CALLS = 4` (la cinquième attente rend `wait_capacity` sans occuper un créneau, `:33`, `:259`) ; dans `crates/bridget-daemon/src/delegation_mcp.rs` ajouter `bridget_task_wait` à `tools()` (annotations du contrat) et à la liste blanche, interroger `daemon_supports_wait` avant toute trame (`wait_unsupported`), ouvrir la connexion avec `connect_until(now + wait_timeout + 10 s)` ; ajouter le nom au dispatch (`mcp.rs:475`) et au catalogue de test (`:2597-2625`) ; dans `crates/bridget-daemon/src/wrapper.rs` agrandir `BRIDGET_SAFE_MCP_TOOLS` (`:5035`), ajouter `tool_timeout_sec=3700` au montage Codex (`:5252-5270`) et adapter les tests `:8117`, `:8149`, `:8171` ; ajouter `wait_timeout_default_secs` et `wait_timeout_max_secs` au catalogue (clés figées dans `native_delegation_permissions149_tests.rs`) ; tests `native151_facade_*` : 4 attentes ouvertes, la 5e refusée, `bridget_task_cancel` et `bridget_task_status` aboutissent ; test de charge `native151_facade_progression_64_attentes` (64 attentes, écritures étrangères continues, appels de contrôle sous 500 ms). L'annulation MCP et l'EOF sont traités par T026. Résultat : tests verts ; `delegation_mcp` compte 6 outils après T025 (4 avant ; mettre à jour le test `tools().len()` à chaque ajout).
- [x] T013 [US1] Test de bout en bout `native151_e2e_deleguer_puis_attendre` dans `crates/bridget-daemon/tests/native_delegation_e2e.rs` : parent et enfant synthétiques, délégation puis attente, état et résumé rendus, puis exactement un message de résultat remis et accusé (`mailbox` passe de `pending` à `claimed` puis `delivered`), enfant nettoyé par `tick` ; variante : attente expirée puis un seul message. Résultat : test vert (SC-001, SC-002, SC-003, SC-006).
- [x] T026 [US1] Annulation MCP et fin d'entrée (tâche ajoutée après la ronde 2, garde son numéro) : rouge d'abord dans `crates/bridget-daemon/src/mcp.rs` avec un exécuteur injecté qui ouvre une paire `UnixStream` : `native151_facade_annulation_avant_enregistrement` (la notification précède l'enregistrement : l'exécuteur voit le drapeau et s'arrête sans bloquer), `native151_facade_annulation_pendant` (socket fermé, échange bloquant interrompu, créneau libéré), `native151_facade_annulation_apres` (sans effet), `native151_facade_eof_ferme_les_attentes` (deux appels bloqués, EOF : `serve` rend la main en moins de 6 s), `native151_facade_enregistrement_apres_eof_refuse` ; puis implémenter le registre unique `appel -> {annulé, socket}` sous un verrou (entrée créée par `serve` avant `scope.spawn`, `:185`), l'enregistrement du socket dès la connexion et avant l'authentification auxiliaire et la sonde dans `crates/bridget-daemon/src/delegation_mcp.rs` et `mcp.rs` (vérification du drapeau dans la même section critique), la prise en compte de la notification `notifications/cancelled` (`:147-158`) et une garde `Drop` dans la fermeture de `thread::scope` (`:131`) qui appelle `cancel_all` avant la jonction. Dépend de T012. Résultat : tests verts (SC-010, SC-012).

## Phase 6 - US4 Regroupement (P2)

**Test indépendant** : filtre `native151_groupe`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/151-attente-fin-de-tache/tasks.md#extrait-1).

## Phase 7 - US5 Attendre la réponse d'un envoi (P2)

**Test indépendant** : filtre `native151_envoi`.
**Dépend de** : T002 à T005, T012.

- [x] T015 [US5] Rouge d'abord : écrire `native151_envoi_*` : `answered` avec résumé figé ; `reply_unavailable` (réponse du guichet, ligne antérieure à 151) avec `reply: null` ; `native151_envoi_guichet_reveil_sans_filet` ; `timed_out` quand `reply_timeout` passe sans réponse ; `open` à l'expiration de `wait_timeout` ; `cancelled` après `bridget_cancel` ; rappel après coupure sans nouvel envoi ; demande d'un autre émetteur : `request_unavailable` ; `wait_unsupported` face à un daemon sans `wait_v1` ; `bridget_send` inchangé (réponse identique à 150). Résultat : tests vus en échec.
- [x] T016 [US5] Traiter `RequestWait` dans `crates/bridget-daemon/src/daemon.rs` : l'ajouter aux deux listes de trames (Client autorisé, Service refusé) (`:14384`, `:14615`, selon T001(f)), identité de l'émetteur, un seul SELECT de `tracked_requests` filtré par sender (état, `deadline_at`, `reply_summary`), même boucle que T011 sur `WaitEpoch` (tranche `min(5 s, délai restant, échéance restante)`), fin à `deadline_at`, même borne de 64 attentes ; ajouter `bump()` après commit à tous les écrivains relevés en T001(g) : sites `answered` (`:7672`, `:17778`), expiration après `claim_timeout`, annulation de demande, et réponse du guichet au site d'appel `st.store.reply_guichet(...)` (`:16074`) ; test `native151_envoi_guichet_reveil_sans_filet` (réponse du guichet, filet réglé à 60 s, réveil en moins de 2 s). Résultat : tests de T015 côté daemon verts.
- [x] T017 [US5] Dans `crates/bridget-daemon/src/mcp.rs` ajouter l'outil `bridget_request_wait` (schéma, liste blanche, sonde `daemon_supports_wait`, connexion à délai propre, même compteur de 4 attentes que T012 et même registre d'annulation que T026), au catalogue de test et à `BRIDGET_SAFE_MCP_TOOLS` ; ne pas toucher `execute_send`. Résultat : T015 entièrement vert (SC-009).

## Phase 8 - US6 Compatibilité et finitions

- [x] T018 [US6] Compatibilité avec le vrai ancien daemon : compiler le daemon du commit de base de 150 dans un dossier temporaire hors du dépôt (`git worktree add` vers `/tmp`, jamais de `checkout` dans le worktree), lancer le client 151 contre lui avec la recette ; constater `wait_unsupported` en moins de 5 s pour `bridget_task_wait` et `bridget_request_wait`, et qu'aucune trame nouvelle n'est reçue ; consigner dans `validation/compat-ancien-daemon.md`, puis retirer le dossier temporaire. Résultat : SC-011 prouvé avec le binaire ancien.
- [x] T019 [P] Documenter les outils, `work_state`, le résumé, les états de boîte, le doublon possible, les refus (`wait_unsupported`, `wait_timeout_out_of_range`, `wait_capacity`, `request_unavailable`, `task_not_ready`), le délai d'outil des hôtes et la limite FR-030 dans `docs/delegation-native.md` (section après la ligne 42), `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`. Résultat : relecture sans terme vague.
- [x] T020 [P] Écrire l'ADR `docs/decisions/151-attente-et-boite-de-fin-de-tache.md` (contexte, décisions R1 à R12, pourquoi l'attente est une lecture pure, conséquences, statut Accepté). Résultat : fichier présent.
- [x] T021 [US6] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148/149/150/087 ne passe de vert à rouge ; consigner dans `specs/151-attente-fin-de-tache/validation/tests-151.md`. Résultat : SC-007.
- [x] T022 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests sensibles à la charge avant de conclure ; relever SC-008 (progression à 64 attentes, une lecture par tranche sans événement) ; consigner les compteurs dans `validation/tests-151.md`. Résultat : suite verte ou échecs expliqués.
- [x] T023 Relire le diff complet (Articles XIX et XX : lignes suppressibles, abstractions, chemins d'erreur), vérifier à la lecture qu'aucun chemin d'attente n'écrit `result_sent`, `failure_sent`, `delivery_acked` ni `cleanup_done`, écrire `specs/151-attente-fin-de-tache/implementation.md` et enregistrer `git diff` dans `specs/151-attente-fin-de-tache/validation/diff-151.patch`. Résultat : réponses aux six questions de l'Art. XX.

## Phase 9 - US7 Accusé de lecture explicite (P2, en dernier, supprimable)

**Test indépendant** : filtre `native151_ack`. Ces deux tâches se retirent sans effet sur T001 à T023 si la question 2 est refusée.

- [x] T024 [US7] Rouge d'abord : écrire `native151_ack_*` : accusé d'une tâche `result_available` et d'une tâche `failed` (`mailbox.state = acknowledged`) ; `task_not_ready` pour une tâche active ou `cancelled` ; `task_unavailable` pour tout l'appel si un identifiant n'est pas au propriétaire ; deuxième accusé : `already` ; aucun effet sur `result_sent`, `delivery_acked`, le message de résultat, le nettoyage, la purge 150 ni la génération Lineage ; `wait_unsupported` face à un ancien daemon. Résultat : tests vus en échec.
- [x] T025 [US7] Ajouter `Task.acknowledged_at: Option<i64>` (`#[serde(default)]`) dans `crates/bridget-daemon/src/delegation.rs`, le faire ignorer par `visible_changed` (`crates/bridget-daemon/src/delegation_lineage.rs`), ajouter `NativeDelegationRequest::Ack` (`crates/bridget-transport/src/protocol.rs`) et son traitement (`can_own_task`, écriture sous le verrou d'état), l'outil `bridget_task_ack` (`crates/bridget-daemon/src/delegation_mcp.rs`, liste blanche, sonde, catalogue, `BRIDGET_SAFE_MCP_TOOLS`), l'état `acknowledged` dans la dérivation de `mailbox` (T009), la documentation (T019) ; mettre à jour le test `tools().len()`. Résultat : T024 vert ; `native148`, `native149` et `native150` verts.

## Dépendances

- T001 avant tout.
- T002 à T005 en parallèle (fichiers distincts), avant les phases 3 à 7.
- Phase 3 (US2) avant Phase 4 (US3) : la boîte étend `task_view`.
- T010 à T013 séquentiels : même fichier `native_delegation.rs` ou `mcp.rs`. T026 après T012 ; T017 après T026.
- T014 après T011.
- Phase 7 (US5) après T012 ; peut être reportée sans bloquer US1 à US4 (MVP).
- T018 après T012 et T017. T019 et T020 en parallèle. T021 à T023 en dernier de la partie obligatoire.
- Phase 9 (US7) après T023 ; T019 est complétée par T025 pour l'outil `bridget_task_ack`.

## Exemples de parallélisme

- T002, T003, T004, T005 : fichiers distincts.
- T019, T020 : documentation seule.

## Stratégie

MVP = Phases 1 à 5 (T001 à T013 et T026) : déléguer puis attendre, résultat structuré, boîte
lisible, façade qui ne se bloque pas. Puis regroupement (US4), attente d'envoi (US5),
compatibilité et finitions (US6), enfin l'accusé explicite (US7).

## Charge future (Article XX)

- Réduisent la charge : T011 (plus de relecture de statut par les agents), absence de
  toute écriture de livraison par l'attente (aucune transition à raisonner).
- Ajoutent de la complexité justifiée : T003 (compteur global dont chaque écriture
  dépend, justifié par le canal Lineage à consommateur unique), T004 (résumé figé dans
  la transaction, nécessaire car le ledger n'est pas une clé fiable), T005 (sonde,
  nécessaire car un ancien daemon est muet), T012 (limite et annulation, nécessaires
  car la façade a 8 créneaux et ne ferme pas ses exécutions).

## Convergence

- [x] T027 [US5] Prouver la borne SC-009 après l'accusé normal de réponse : ouvrir `RequestWait` sur une demande encore `open`, puis faire passer sa réponse par le vrai traitement `DeliverAcked` du daemon et mesurer le retour `answered` en 2 s au plus après cet accusé. Vérifier le résumé exact, avec deux réponses de scopes différents qui utilisent le même identifiant de message vers le même destinataire. Réutiliser les fixtures et observateurs de tests existants, sans nouveau hook de production ; ne pas remplacer le traitement de l'accusé par `mark_answered` ni par un `bump` manuel. Exécuter le test ciblé avec la recette et consigner la preuve dans `validation/tests-151.md`. Les tests actuels prouvent séparément la stabilité entre scopes (`idempotency/tests.rs:121`), un retour sur état déjà `answered` (`daemon/native_delegation.rs:1958`) et le réveil guichet (`:2106`), mais ne mesurent pas cette borne après l'accusé normal. Résultat : borne après accusé et bonne réponse prouvées, sans changement du contrat.
