# Relevés 260 - mesures de référence

Mesures faites le 10 novembre 2026 sur le commit `065f54f0`. Entre `065f54f0` et `b3949c93`, seuls `cli.rs`, `delegation_mcp.rs` et `mcp.rs` ont changé.
Les numéros de ligne sont indicatifs. Chercher toujours par symbole avec `rg`.
Abréviations : `D/` = `crates/bridget-daemon/src/` ; `T/` = `crates/bridget-transport/src/`.

## 1. Volumes

- 285 fichiers `.rs` dont 111 de test ; 408 922 lignes ; production 161 988 lignes.
- 3813 `#[test]`, 66 `#[ignore]` dont 8 sans motif.
- 4696 fonctions de production : 255 de plus de 100 lignes, 43 de plus de 300, 3 de plus de 1000.
- 47 `#[allow(clippy::too_many_arguments)]`.
- SQL : `prepare(` 144, `prepare_cached(` 0, `query_row(` 338, `execute(` 419.
- Version du workspace : 0.8.26. Chaîne Rust : 1.92.0 (`rust-toolchain.toml`), donc `#[expect(..., reason = "...")]` est disponible.
- Ligne de base des tests sur Mac (main `c48c30fb`) : 2853 PASS, 40 FAIL hérités, 77 ignorés. Ligne de base Linux inconnue.

## 2. Format et clippy

- Format : `cargo fmt --all -- --check -l` signale 120 fichiers sur 309.
- Clippy sans `test-support` : `python3 scripts/build.py cargo clippy --workspace --all-targets --locked --offline --message-format=json` sort en 101. 3 erreurs, 385 avertissements (daemon 342, transport 46, core 0).

Les 3 erreurs :

| Règle | Lieu | Traitement |
|-------|------|------------|
| `clippy::never_loop` | `D/daemon/native_lineage.rs` ~1659, `while receiver.recv().is_ok() {` (production) | `while` devient `if`, comportement constant (D2) |
| `clippy::non_octal_unix_permissions` | `D/daemon/native_workspace154_tests.rs` ~1897, `from_mode(0)` | écrire `0o000` |
| `clippy::reversed_empty_ranges` | `D/daemon/mobile_operations.rs` ~1928, `1=>1..=0` (test) | plage vide volontaire : `#[allow(clippy::reversed_empty_ranges)]` avec commentaire. Ne pas appliquer la suggestion |

Avertissements par règle : collapsible_if 110 ; too_many_arguments 40 ; nonminimal_bool 34 ; needless_borrows_for_generic_args 26 ; needless_borrow 22 ; type_complexity 19 ; bool_assert_comparison 15 ; result_large_err 11 ; useless_conversion 10 ; redundant_field_names 9 ; large_enum_variant 8 ; cloned_ref_to_slice_refs 7 ; let_and_return 5 ; zombie_processes 5 ; 4 chacun : empty_line_after_doc_comments, question_mark, let_unit_value, unnecessary_get_then_check, assertions_on_constants, identity_op ; expect_fun_call 3 ; 2 chacun : ptr_arg, manual_is_multiple_of, needless_option_as_deref, unnecessary_unwrap, manual_range_patterns, items_after_test_module, useless_format, single_element_loop ; 1 chacun : empty_line_after_outer_attr, suspicious_assignment_formatting, int_plus_one, redundant_pattern_matching, unnecessary_mut_passed, clone_on_copy, redundant_closure_call, manual_strip, format_in_format_args, manual_div_ceil, implicit_saturating_sub, cmp_owned, needless_as_bytes, single_char_add_str, if_same_then_else, extra_unused_lifetimes, drop_non_drop, err_expect, unnecessary_to_owned, field_reassign_with_default, useless_vec.

Fichiers les plus touchés : native_delegation.rs 61 ; daemon.rs 35 ; remote_worker.rs 19 ; native_permissions.rs 14 ; wrapper.rs 14 ; daemon/specialists222.rs 12 ; t3code.rs 12 ; codex_app_server.rs 9 ; lineage_contract_test.rs 9 ; remote_worker_read.rs 9 ; provider_catalogue220.rs 8 ; workspace_git.rs 7.

Dette structurelle (à mettre en `allow` au niveau du workspace avec commentaire, suivie par le lot 9) : too_many_arguments, type_complexity, result_large_err, large_enum_variant.

## 3. Aiguillage des messages (lot 5)

Fichier `D/daemon.rs`, fonction `handle_wrapper_message_with_peer` (~20014, 5011 lignes). Retour : `Option<DaemonToWrapper>`.

Structure :

1. Garde `_update_activity` en tête. Elle reste dans la fonction.
2. Pré-aiguillage (~20040-20078) : série de `if let WrapperToDaemon::X { request } = &msg { return Some(module::handle(...)); }`.
3. Pré-traitement (~20074-20806, 736 lignes). Ce n'est pas une branche du `match`. Il contient trois blocs : `let attach_refusal = { ... };` (28 lignes), `let service_refusal = { ... };` (265 lignes), `let client_refusal = { ... };` (321 lignes).
4. `match msg {` (~20807), expression finale de la fonction. Fin ~25024, juste avant `pub fn get_status`.

Compte : 115 branches. 37 branches d'au moins 30 lignes, soit 3303 lignes.
Calcul de la cible : 5011 - 3303 + 37 = 1745 lignes. Avec `service_refusal` et `client_refusal` extraits : environ 1163 lignes. La cible « moins de 1500 lignes » exige donc les 37 branches et ces deux blocs.

Motif de module enfant à réutiliser (16 modules existants, par exemple `D/daemon/user_input.rs`) : `mod wrapper_dispatch;` en tête de `daemon.rs` ; le fichier `D/daemon/wrapper_dispatch.rs` commence par `use super::*;` ; fonctions `pub(super) fn on_xxx(...) -> Option<DaemonToWrapper>`. Les `return` internes de la branche deviennent les retours de la fonction. `?` sur `Option` reste valide.

Tranches (une branche par commit) :

- Tranche 1 (8) : Send 429 ; ServiceRequest 264 ; RelaunchOrder 203 ; CancelRequest 203 ; SpawnOrder 189 ; ServiceHello 124 ; ControlExecutionReported 112 ; DecommissionOrder 109.
- Tranche 2 (11) : ArtifactPublish 94 ; Register 89 ; GuichetReply 89 ; CoordinationSubscribe 88 ; ClientHello 88 ; QueueEditRequest 81 ; ObservationRequest 78 ; Subscribe 75 ; DisplayNameSet 72 ; DeliveryRejected 72 ; ThreadRequest 71.
- Tranche 3 (9) : DelegatedRuntimeEvent 67 ; NativePermissionFact 65 ; WaitAgentLinks 52 ; T3ThreadBindingFact 50 ; DaemonIdentityRequest 49 ; Subscribed 48 ; ObservedActivity 47 ; TurnState 42 ; Unsubscribe 41.
- Tranche 4 (9) : DeliveryIndeterminate 38 ; ListRequests 37 ; AdoptStoppedOrder 36 ; DisplayNameResolve 35 ; DirectoryScoped 34 ; AttachRejected 34 ; JournalReady 34 ; LiveJournalFragment 32 ; Gap 32.

Liste complète (ligne indicative, nom, taille) : 20810 WorkerHello 9 ; 20819 WakeRequest 6 ; 20825 QueueEditRequest 81 ; 20906 QueueList 4 ; 20910 CommunicationProjectFact 13 ; 20923 DirectoryScoped 34 ; 20957 ObservationGap 19 ; 20976 ObservationCapabilities 26 ; 21002 HumanLineage 3 ; 21005 HumanThreadViewV1 3 ; 21008 HumanThreadWatchV1 3 ; 21011 T3ThreadBindingFact 50 ; 21061 ThreadNoticeCapability 19 ; 21080 ThreadRequest 71 ; 21151 LedgerSearch 18 ; 21169 LedgerRead 18 ; 21187 ObservationRequest 78 ; 21265 ObservedActivity 47 ; 21312 RoleHandshake 1 ; 21313 DelegationOverviewRead 7 ; 21320 DelegationCancel 11 ; 21331 DelegationHaltSet 15 ; 21346 ControlStateRead 3 ; 21349 ControlFocusRead 3 ; 21352 ControlHistory 3 ; 21355 ControlStateSet 21 ; 21376 HumanInboxDeposit 10 ; 21386 ControlFocusPublish 3 ; 21389 HumanInboxList 7 ; 21396 HumanInboxResolve 8 ; 21404 HumanInboxDecisions 3 ; 21407 HumanInboxAck 4 ; 21411 HumanInboxClose 5 ; 21416 RuntimeIngressHello 6 ; 21422 ArtifactRead 16 ; 21438 ArtifactPublish 94 ; 21532 ProjectRegistryRequest 5 ; 21537 ProjectRegistryAdminRequest 7 ; 21544 ProjectSystemRequest 17 ; 21561 ProjectRoundRequest 5 ; 21566 ProjectRoundDispatch 5 ; 21571 ProjectProfileRequest 5 ; 21576 ProjectRuntimeRequest 5 ; 21581 DelegatedRuntimeEvent 67 ; 21648 DelegatedRuntimeEventAcknowledged 22 ; 21670 WaitAgentLinks 52 ; 21722 ServiceHello 124 ; 21846 CoordinationSubscribe 88 ; 21934 ServiceRequest 264 ; 22198 GuichetClaimNext 22 ; 22220 GuichetClaim 29 ; 22249 GuichetLookup 24 ; 22273 GuichetReply 89 ; 22362 ClientHello 88 ; 22450 RegisterAuxiliary 14 ; 22464 SendIdempotent 26 ; 22490 Lookup 12 ; 22502 ControlExecution 18 ; 22520 ControlExecutionReported 112 ; 22632 DeliverAcked 7 ; 22639 DeliveryIndeterminate 38 ; 22677 RequestWait 11 ; 22688 NativeDelegation 5 ; 22693 NativeDelegationT3 5 ; 22698 NativePermissionFact 65 ; 22763 SpawnOrder 189 ; 22952 StopOrder 14 ; 22966 RelaunchOrder 203 ; 23169 DecommissionOrder 109 ; 23278 AdoptStoppedOrder 36 ; 23314 Subscribe 75 ; 23389 Unsubscribe 41 ; 23430 Subscribed 48 ; 23478 AttachRejected 34 ; 23512 End 22 ; 23534 JournalReadError 24 ; 23558 JournalFragment 25 ; 23583 LiveJournalFragment 32 ; 23615 SnapshotCaughtUp 24 ; 23639 Gap 32 ; 23671 Register 89 ; 23760 DiskSpace 22 ; 23782 JournalReady 34 ; 23816 TerminalSessionReady 17 ; 23833 Unregister 24 ; 23857 SelectRuntime 3 ; 23860 RuntimeSelectionReported 13 ; 23873 RuntimeCatalogueReported 8 ; 23881 Send 429 ; 24310 Heartbeat 11 ; 24321 TurnState 42 ; 24363 ExecutionStateChanged 5 ; 24368 WorkspaceRepairResult 5 ; 24373 ExecutionProviderObserved 5 ; 24378 DaemonIdentityRequest 49 ; 24427 SentinelStatusRead 5 ; 24432 ListAgents 5 ; 24437 DisplayNameResolve 35 ; 24472 DisplayNameSet 72 ; 24544 Runtime 25 ; 24569 ServedModel 5 ; 24574 RateLimit 22 ; 24596 Usage 28 ; 24624 UsageWindow 9 ; 24633 Domain 14 ; 24647 Availability 18 ; 24665 DeliveryRejected 72 ; 24737 CancelRequest 203 ; 24940 ListRequests 37 ; 24977 UserInput 1 ; 24978 ChildRequest 4 ; 24982 ChildRequestAnswered 16 ; 24998 ChildRequestClosed 11 ; 25009 AgentTracking, Specialists, MobileAdministration, MobileOperation 1 ; 25010 LedgerProjection 15.

## 4. Tests en ligne (lot 4)

16 fichiers (lignes totales / lignes de tests en ligne) : D/daemon.rs 52569 / 31968 ; D/daemon/native_delegation.rs 38228 / 23830 ; D/remote_worker.rs 14026 / 6887 ; D/mcp.rs 9773 / 6675 ; D/wrapper.rs 13581 / 6092 ; T/codex_app_server.rs 10316 / 5524 ; D/attach.rs 9555 / 5249 ; D/cli.rs 12755 / 4870 ; T/protocol.rs 12255 / 4710 ; D/t3code.rs 8578 / 4008 ; D/delegation.rs 5637 / 2861 ; T/claude_stream_json.rs 4613 / 2230 ; T/acp.rs 4251 / 2119 ; D/delegation_mcp.rs 2580 / 1694 ; D/daemon/mobile_operations.rs 2075 / 1644 ; D/registry.rs 4408 / 1539. Total en ligne : environ 112 000 lignes.

Motif existant à copier (`D/daemon/native_delegation.rs` ~36813) :

```rust
#[cfg(test)]
#[path = "native_workspace154_tests.rs"]
mod native_workspace154_tests;
```

Trois formes de `#[cfg(test)]` : (a) module en ligne `#[cfg(test)] mod x { ... }` : à sortir ; (b) déjà sorti par `#[path]` : ne rien faire ; (c) élément isolé : il reste en place. Exemples de (c) : `#[cfg(test)] fn handle_wrapper_message(` (daemon.rs ~19988), `handle_domain`, `handle_availability`, `daemon_identity`, champs `fixture_root` et `guard_checks`, variante `Control`, crochets `B221_VIEW_TEST_*` de remote_worker.rs, tests libres `#[cfg(test)] #[test]` en fin de daemon.rs et de native_delegation.rs.

Règles de l'outil `outils/extraire_module_tests.py` (à écrire, options `--list FICHIER` et `--extract FICHIER LIGNE`) : module de premier niveau (colonne 0) de plus de 200 lignes ; fin du bloc = première ligne `}` en colonne 0 après le début ; nouveau fichier dans le même dossier que la source (préserve `include_str!`) ; nom `<stem>_<mod>.rs`, ou `<stem>_tests.rs` si le module s'appelle `tests`, ou `<mod>.rs` si le nom commence déjà par le stem ; déclaration `#[cfg(test)] #[path = "<fichier>"] <visibilité> mod X;` avec attributs et visibilité conservés ; corps non désindenté ; refus si le corps contient un `mod y;` non en ligne, si le fichier cible existe, ou si le module est imbriqué.

Critères : compilation ; liste triée des noms de tests identique avant et après (`... cargo test ... -- --list`) ; liste des échecs identique à la ligne de base.

## 5. Blocs `unsafe` (lot 6)

- 247 blocs, 244 sans commentaire de sûreté. Les plus chargés : workspace_commit_view.rs 32 ; managed_process.rs 29 ; claude_interactive.rs 18 ; attach.rs 16 ; jsonl.rs 12. Liste complète : `rg -c 'unsafe \{' crates --glob '*.rs' | sort -t: -k2 -nr`.
- Appels libc : geteuid 76 ; kill 54 ; fcntl 48 ; poll 33 ; close 25 ; flock 13 ; write 11.
- Modèle de commentaire existant (`D/disk_hygiene.rs` ~446) : `// SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence.`
- Lieu de la fonction unique `effective_uid()` : `T/fsutil.rs` (primitives privées du système de fichiers). `libc` est déjà une dépendance des deux crates.
- Lint de fin de lot : `clippy::undocumented_unsafe_blocks`.

## 6. Concurrence (lot 7)

- Boucle périodique : `D/daemon.rs` ~7061-7090, thread `st_reminder`, un tour par seconde. Point de mesure : autour de `native_delegation::tick(&st_reminder)`.
- `tick` : `D/daemon/native_delegation.rs` ~12576, 983 lignes, verrou global tenu. Requêtes par élément ~12778-12832 : `st.store.open_requests()` puis `st.delegation_store.for_mission(&request.id)` ; `failed_for_owner(&owner)` par propriétaire ; boucle `for mut task in tasks.iter().cloned()` avec 17 `save` et 3 `get_request`.
- Acceptation : `D/daemon.rs` ~7356-7379, `listener.accept()` sans plafond, un thread par connexion.
- Compteur jamais câblé : `pub struct Metrics` (daemon.rs ~115), `active_connections`, `increment_connections`, `decrement_connections`.
- Verrous empoisonnés récupérés en silence (`unwrap_or_else(|x| x.into_inner())`) : plus de 600 sites. daemon.rs 221 ; T/codex_app_server.rs 98 ; native_delegation.rs 94 ; T/acp.rs 75 ; T/claude_stream_json.rs 51 ; wrapper.rs 35 ; fleet.rs 25 ; native_lineage.rs 22.

## 7. Erreurs comparées par texte (lot 8)

Motif `(err|error|e|reason|message|msg)\.(contains|starts_with|ends_with)\("` hors tests : 375 occurrences (borne haute). native_delegation.rs 68 ; T/codex_app_server.rs 40 ; daemon.rs 34 ; attach.rs 28 ; cli.rs 24 ; delegation_mcp.rs 22 ; wrapper.rs 16 ; mcp.rs 16 ; reaper.rs 12 ; registry.rs 11 ; native_permission_change.rs 8 ; communication/client.rs 8.

## 8. Fonctions longues et listes de paramètres (lot 9)

43 fonctions de plus de 300 lignes (longueur, lieu, nom) : 5011 D/daemon.rs:20014 handle_wrapper_message_with_peer ; 1371 D/wrapper.rs:4447 launch_session_with_status ; 1101 T/codex_app_server.rs:3493 spawn_reader ; 983 D/daemon/native_delegation.rs:12576 tick ; 796 D/wrapper.rs:2413 launch ; 777 native_delegation.rs:8617 admit_delegate ; 727 D/agent_profile/specialists.rs:384 apply_specialist_inner ; 625 daemon.rs:12320 handle_idempotent_send_with_result_authority ; 569 D/delegation_mcp.rs:171 execute_with_proof ; 555 daemon.rs:6826 run ; 542 wrapper.rs:7364 forward_managed_events_with_redaction ; 539 D/idempotency.rs:531 init_schema ; 533 T/claude_stream_json.rs:1579 spawn_reader ; 517 native_delegation.rs:4108 failure ; 485 daemon.rs:19031 handle_worker_message ; 482 native_delegation.rs:9414 handle_locked_with_remote_proof ; 458 D/remote_worker.rs:6901 run_worker ; 449 native_delegation.rs:1120 observe_routing_candidates_for_project ; 446 D/mcp.rs:2639 tools ; 443 D/mobile/http.rs:507 execute ; 423 daemon.rs:8476 handle_register_with_channel ; 420 daemon.rs:2215 deliver_frozen_wake ; 395 daemon.rs:13005 handle_execution_control_authorized ; 394 T/codex_app_server.rs:387 spawn_endpoint ; 393 wrapper.rs:3525 start_with_clock ; 379 native_delegation.rs:6081 handle_with_fact_inner ; 377 native_delegation.rs:6636 delegate_remote_with_routing ; 369 D/t3code.rs:1189 serve ; 365 native_delegation.rs:4957 task_view ; 362 mcp.rs:712 execute_tool_at_with_identity_once ; 357 D/native_permissions.rs:1230 capture_context_with_env_impl ; 347 daemon.rs:7385 handle_connection ; 344 D/remote_workspace_repair.rs:187 repair_workspace ; 342 daemon.rs:4617 drain_managed_events ; 337 native_delegation.rs:7563 start_remote_task_with ; 337 D/daemon/native_workspace_repair.rs:257 repair ; 334 T/native_permissions.rs:72 validate_permissions ; 328 T/codex_app_server.rs:2524 normalize_child_provider_request ; 319 D/daemon/specialists222.rs:1186 request_role ; 311 D/store.rs:145 init_schema ; 307 T/acp.rs:1155 spawn_reader ; 303 native_delegation.rs:2501 admit_auto_routing ; 301 t3code.rs:3238 project_journal.

Exemptions naturelles : `tools` de mcp.rs (données) et les deux `init_schema` (SQL).

9 fonctions à 15 paramètres ou plus (structure d'arguments) : 21 T/acp.rs:1478 prompt_request ; 17 daemon.rs:8476 handle_register_with_channel ; 17 native_delegation.rs:8211 resolve_target ; 17 wrapper.rs:6367 reconnect_managed_session ; 16 wrapper.rs:2195 connect_and_register_at ; 15 D/threads.rs:934 post ; 15 wrapper.rs:2089 connect_and_register_with_domain_at ; 15 D/wrapper_workspace_repair.rs:74 apply ; 15 T/claude_stream_json.rs:1579 spawn_reader.

11 fonctions à 12-14 paramètres (seulement documentées) : 14 native_delegation.rs:8471 target_resolver_in_catalogue171 ; 14 T/acp.rs:1155 spawn_reader ; 13 native_delegation.rs:2369 choose_routing_admission ; 13 native_delegation.rs:8252 target_resolver ; 13 D/idempotency/spawn_commands.rs:24 reserve_spawn ; 13 wrapper.rs:2060 connect_and_register ; 12 native_delegation.rs:3586 finalise_auto_choice ; 12 D/delegation.rs:2099 choose_auto_admission_at ; 12 D/execution_store/specialists.rs:130 reserve_specialist_request ; 12 native_permissions.rs:405 project_remote_permissions_in_operation ; 12 wrapper.rs:4214 spawn_managed_session_transport.

## 9. Modules numérotés (lot 11)

Table de correspondance (18) : agent_tracking_sync230.rs → agent_tracking_sync.rs ; daemon/agent_tracking230.rs → daemon/agent_tracking.rs ; daemon/late_worker_prepares240.rs → daemon/late_worker_prepares.rs ; daemon/native_admission_guidance211.rs → daemon/native_admission_guidance.rs ; daemon/native_provider_catalogue220.rs → daemon/native_provider_catalogue.rs ; daemon/native_quota_questions246.rs → daemon/native_quota_questions.rs ; daemon/native_workspace_preparation210.rs → daemon/native_workspace_preparation.rs ; daemon/specialists222.rs → daemon/specialist_requests.rs ; delivery_spool244.rs → delivery_spool.rs ; mcp_directory211.rs → mcp_directory.rs ; provider_catalogue220.rs → provider_catalogue.rs ; remote_worker_capacity234.rs → remote_worker_capacity.rs ; remote_worker_proof198.rs → remote_worker_proof.rs ; reply_outbox244.rs → reply_outbox.rs ; sentinel244.rs → sentinel.rs ; t3code_agent_tracking230.rs → t3code_agent_tracking.rs ; workspace_failure210.rs → workspace_failure.rs ; T/native_hook_profile171.rs → native_hook_profile.rs.

Procédure : `git mv` ; mise à jour de `mod` et de `#[path]` ; identifiants par `perl -pi -e 's/\bANCIEN\b/NOUVEAU/g'` (le `sed` de macOS n'a pas `\b`) ; contrôle `git diff -U0 | rg '^[+-].*"'` : aucune chaîne littérale ne change. Chaînes à ne pas toucher : `format!("spool244-{}", ...)` (delivery_spool244.rs ~247) et `format!("sentinel244-{}", ...)` (sentinel244.rs ~181). Conflit de nom (E0255) : alias ou arrêt. Ne pas modifier `specs/` ni `docs/rapports`. Les noms de fonctions numérotés sont hors périmètre.

## 10. Stockage (lot 12)

`D/store_schema.rs` (`MAX_IDEMPOTENCY_SCHEMA_VERSION = 8`). Trois mécanismes de version : table de migrations, `user_version`, `PRAGMA table_info`. Environ 110 `Connection::open`. WAL pour `store.rs` ; `journal_mode=DELETE; synchronous=FULL` pour `delivery_spool244.rs` et `reply_outbox244.rs`. Le lot produit un inventaire et un ADR proposé, rien d'autre.

## 11. Scripts (lot 13)

- `ruff check scripts --select E9,F,B,S` : 137 signalements. S603 54 ; S607 22 ; S101 18 ; B904 11 ; S108 10 ; F401 4 ; S110 4 ; B905 3 ; B007 2 ; B023 2 ; S105 2 (faux positifs) ; S606 2 ; S102 1 (`scripts/sentinel.py` ~172) ; S310 1 (`scripts/update.py` ~97) ; S324 1 (`scripts/real_hook_profile171.py` ~323).
- `exec` de `scripts/sentinel.py` : la source vient du daemon (`D/sentinel244.rs` ~139-150, `include_str!`). Même origine de confiance : commentaire d'invariant et `# noqa: S102` (D11).
- Scripts sans `set -e` : `scripts/bridget-relance.sh`, `scripts/federate-ssh.sh` (vivant, 14 références), `scripts/install-k1.sh` (cible du Makefile), `scripts/registre-nature-reclasser.sh`, `scripts/test-053-review-witness-selection.sh` (0 référence), `scripts/test-git-pre-push-authorship.sh` (0 référence). Ajouter `set -e` à un script vivant change son comportement : lire le script, lister les commandes dont l'échec est toléré, `bash -n`, sinon documenter.
- Outils locaux : `ruff` et `gitleaks` présents ; `shellcheck`, `actionlint`, `cargo-audit`, `cargo-deny` absents.
- `scripts/install.sh` : l'empreinte SHA-256 vient de la même origine que l'archive. Pas de signature détachée (base du lot 15).

## 12. Tests ignorés et code mort (lots 14 et 17)

- `#[ignore]` sans motif (8) : `D/workspace_commit_view_tests.rs` ~1024 ; `D/daemon.rs` ~649, ~26373, ~32295 ; `D/managed_process.rs` ~2004, ~2041, ~2072, ~2486. Traitement : `#[ignore = "..."]`. Lint : `clippy::ignore_without_reason`.
- `allow(dead_code)` (9) : `D/workspace_git.rs` ~92 ; `D/native_permissions.rs` ~2367 ; `D/provider_accounts.rs` ~829 ; `D/attach.rs` ~556, ~564, ~702 (boucle T806b, D12) ; `D/remote_worker.rs` ~13514, ~13531 (tests) ; `D/daemon/native_delegation.rs` ~3904. Méthode : retirer l'attribut et compiler ; s'il reste nécessaire, `#[expect(dead_code, reason = "...")]` ; élément utilisé seulement par les tests : `cfg(test)`.

## 13. Android (lot 16)

`android/app/src/main/java/app/cartae/bridget/MainActivity.kt` (1154 lignes). Fonctions : MobileApp 136 ; Welcome 233 ; AgentDirectory 250 ; ProjectLabel 343 ; ProjectAvatar 356 ; Conversation 372 (228 lignes) ; activityKey 600 ; isToolActivity 607 ; ToolGroup 610 ; ToolActivity 624 ; hasMessageText 668 ; FilePanel 678 ; TaskPanel 728 ; Questions 766 ; Outbox 806 ; Settings 862 (135 lignes) ; sentinelReason 998 ; sentinelStage 1007 ; Confirm 1017 ; eventText 1023 ; choiceText 1033 ; commandState 1038 ; stateText 1047 ; recoveryText 1061 ; errorText 1075 ; dateText 1123 ; updateStage 1125 ; hostUpdateStage 1139.
Découpage : un fichier par écran (`ConversationScreen.kt`, `SettingsScreen.kt`, `AgentDirectoryScreen.kt`, panneaux, `Texts.kt`) ; `private` devient `internal` ; cible : moins de 300 lignes.
`MobileViewModel.kt` : 1074 lignes, une classe, 95 fonctions (D15 : fonctions pures seulement).
Vérification : `./gradlew` dans `android/` ; confirmer les noms par `./gradlew tasks`. Aucun APK publié par ce lot.

## 14. Hygiène du dépôt et consignes (lots 2 et 3)

- Fichiers suivis mais ignorés (345, `git ls-files -ci --exclude-standard`) : `audits/` 317 ; `specs/159-edition-file/validation/*.log` 26 ; `AGENTS.md` ; `.specify/feature.json`.
- `*.log` est ignoré : les preuves vont en `.txt` dans `validation/`.
- Résidus de la racine : `.wip-reprise-preflight.txt` ; `watch_20260823-175033` ; `.wip-mesures/` (13 fichiers).
- `AGENTS.md` : environ 125 Ko. Six sections de règles en tête. Bloc des reçus : de la première ligne qui commence par `Session` suivi d'un numéro, jusqu'à la ligne qui précède « Les reçus datés des sessions précédentes restent dans specs/. » Destination : `specs/recus-de-session.md`. Contrôle : `diff` octet pour octet du bloc, puis `python3 scripts/check-docs.py`.
- Consignes vivantes à remonter en règles permanentes avant de déplacer le bloc : ne pas arrêter ni remplacer T3 sans assentiment explicite ; repère d'empreinte du registre Mac spécial (dernier connu : session 256, confirmé 257) ; fichier de registre du worker Mac à préserver, parent = namespace attesté ; SYS02 à ne plus reprendre ; pas d'essai Claude ni sentinelle 244 sans demande ; Codex KS désactivé jusqu'à connexion de l'utilisateur ; outil de mise à jour Mac sur Python système 3.9.6 ; Linux commun construit sur Debian 12 / glibc 2.36.

## 15. Build et intégration continue (lot 1)

- `Makefile` : cibles `build`, `release`, `test` via `python3 scripts/build.py cargo ...`. Aucune cible `lint`, `fmt`, `clippy`.
- `scripts/build.py` : liste blanche Cargo = `build, test, check, clippy`. `fmt` et `audit` refusés (appel direct de `cargo fmt` justifié : il ne compile pas). Après un Cargo réussi, `prune_merged_worktrees` retire tout worktree sans commit propre : committer avant le premier Cargo dans un worktree.
- Seul workflow : `.github/workflows/repository-hygiene.yml`. Conventions à reprendre : `permissions: contents: read` ; actions épinglées par SHA (`actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4`, `persist-credentials: false`) ; téléchargements vérifiés par empreinte. Aucun contrôle Cargo aujourd'hui.
- Aucun `[workspace.lints]`, `rustfmt.toml`, `clippy.toml`, `deny.toml`.
- Crochets Git : `commit-msg` seulement.

Recette de test candidate. Non exécutée sous cette forme ; la tâche de fondation T003 la valide :

```sh
umask 077
R=$(mktemp -d /private/tmp/b.XXXX); mkdir -m 700 "$R/state" "$R/tmp"
env -i HOME="$HOME" BRIDGET_HOME="$R/state" BRIDGET_SOCKET="$R/state/bridget.sock" TMPDIR="$R/tmp" CARGO_TARGET_DIR="$PWD/target" PATH="$HOME/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin" python3 scripts/build.py cargo test --offline --locked --workspace --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4 2>&1 | tee "$R/tests.txt"
grep -E '^test .* \.\.\. FAILED' "$R/tests.txt" | sort
```

Tests sensibles à la charge (les rejouer seuls trois fois avant de conclure) : `reprise_daemon_redelivre_les_octets_immuables_a_la_meme_instance` ; `voie_acp_lance_la_session_wrapper_avec_probe_ephemere` ; `managed_parity_test::matrice_fr008_compare_le_meme_corpus_et_les_frames_attach` ; `daemon::presence_tests::stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels`. Échecs hérités connus : 8 tests `core_089_*` et `t3code_098`.
