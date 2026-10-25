# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-lineage-tests-wire-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-lineage-tests-wire-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Symbole | Preuve | Usage test |
|---|---|---|
| `Task.permission_snapshot: Option<NativePermissionSnapshot>` + `#[serde(default)]` | `delegation.rs` (struct Task, 26 champs) | fixtures internes, baseline None |
| `Task.effective_posture: Option<SpawnPosture>` + `#[serde(default)]` | idem | idem |
| `Delegate { posture: Option<SpawnPosture>, effort default }` | `protocol.rs` (variant Delegate, tag `operation`, `deny_unknown_fields`) | fixtures store/CLI |
| `NativeDelegationRequest` tag `operation` snake_case | idem | roundtrip wire |
| `lineage_protocol.rs` inclus | `protocol.rs` : `#[path]` + `pub use lineage_protocol::*` | imports wire |
| `human_lineage(socket, request, output, cancelled)` | `lineage_client.rs`, inclus via `communication/client.rs:17` | CLI e2e/watch |
| `T3ThreadBindingFact { version: u16, t3_thread_id: String }` | `protocol.rs:2843` | enregistrement wrapper T3 (CLI + daemon) |
| `Register` complet (identity_version, channel `ChannelReport: From<Option<String>>`, journal_available…) | `protocol.rs` | CLI `register_t3_wrapper` |
| `ClientCapability` snake_case filaire | `protocol.rs` | wire : `human_lineage_view_v1` / `watch_v1` / `cancel_v1` |

<a id="extrait-2"></a>

## Extrait 2

| Symbole | Preuve | Usage test |
|---|---|---|
| `handle_register_with_channel` (15 paramètres) | `daemon.rs:5975` | enregistrement canal négocié |
| `NegotiatedClient { version, issuer_scope, capabilities }` | `daemon.rs:1030` | gardes client |
| `spec094_live_test_connection(&mut st, "t3")` | `daemon.rs:31136` | fixture live |
| `announce_communication_project(&state, root, source, host, None)` | `daemon.rs:8176` | fixture autorité |
| `handle_connection(UnixStream, Arc<Mutex<DaemonState>>)` | `daemon.rs:4947` | watch socket réelle via `UnixStream::pair` |
| `handle_wrapper_message` appelle `native_lineage::invalid_message` en tête | `daemon.rs:14038` | routage trame invalide (lecture) |
| `DaemonState.native_permission_refusals` | `daemon.rs:689` (champ) | API GP prise en compte, non used directement par les tests L |
| `state_with_registered_agent` `pub(super)` | `presence_tests` (`daemon.rs:20099`) | fixture état enregistré |
| `push_control_message_until_checked`, `CANCEL_NOTIFICATION_BUDGET` | `daemon.rs:1746` / `1736` | flux watch |

<a id="extrait-3"></a>

## Extrait 3

| Symbole | Preuve | Usage test |
|---|---|---|
| `ProjectionMutation { generation, seq }` | `delegation_lineage.rs:10` | publish direct divergent |
| `initialize_lineage`, `lineage_page`, `lineage_show`, `human_cancel`, `parent_lineage`, `descendants`, `projection_meta`, `take_changes` | `delegation_lineage.rs` | scénarios store |
| `lineage_journal_page(directory, task_id, after, limit)` | `attach.rs:34` | page journal daemon |
| `lineage_journal_follow` | `attach.rs:90` | lecture seulement, non testé (§8) |
| `is_native_projection(&Value)` | `t3code_contract.rs:197` | marqueur fermé |
| `parse_snapshot(&str)` filtre le marqueur avant normalisation | `t3code_contract_v2.rs:55` | snapshot V2 |
| `stable_uuid` `pub(crate)` + `NAMESPACE_098` | `t3code.rs:316` / `:60` | réplique CLI octet à octet |
| `canonical_uuid` `pub(crate)` | `threads.rs:350` | UUID canoniques |
| `current_host_date` | `bridget-transport::journal` | nom de fichier journal CLI |
| deps `uuid`, `rusqlite`, `libc` | `crates/bridget-daemon/Cargo.toml` (lignes 24/29/34, workspace) | tests CLI |

<a id="extrait-4"></a>

## Extrait 4

- **Grammaire fermée sans daemon** : 19 casses → exit 2,
  `code = "invalid_request"`, `retryable = false` (arguments absents,
  `--json` manquant ou dupliqué, racine relative, flag inconnu, action
  inconnue, bornes limit/offset/after-seq dépassées, UUID non canoniques,
  flag superflu par action). Socket absente → exit 3,
  `code = "binding_unavailable"`.
- **Bout en bout** : daemon réel + wrapper T3 réel (Register identity_version
  2, CommunicationProjectFact, T3ThreadBindingFact) + semis SQLite direct.
  List → seq 0 au semis, une entrée ; show → résultat exact « Sortie CLI 149
  éè », total en octets ; mauvais fil → `binding_unavailable` ; mauvaise
  racine → `project_mismatch` ; offset 16 au milieu de `é` →
  `result_offset_invalid` ; journal inconnu → `task_unavailable` ; journal
  semé (0700/0600, deux événements) → page complète `caught_up` ; cancel →
  reçu `{"version":1,"task_id":…,"status":"cancelling"}` rejoué à
  l'identique ; mismatch → `envelope_mismatch` ; seconde demande sur une
  tâche terminale → reçu sans mutation.
- **Watch CLI** : poll 15 s → ligne `ready` seq 0 ; fermeture du stdout →
  POLLHUP → exit 0, jamais une erreur.
- **Semis SQL seq 0** : le `seed_task` insère sans passer par la projection.
  C'est un oracle de fixture (état initial connu, marqueur seq 0 distinct de
  toute écriture projetée), pas une mutation de production.
- **Réplique stable_uuid** : v5 sur `NAMESPACE_098` présenté v4, octet pour
  octet identique à `t3code.rs:60`. Golden de comportement public : toute
  dérive du namespace casse les tests visiblement. Pas d'export décoratif
  exigé.

<a id="extrait-5"></a>

## Extrait 5

- Aucun `cargo test|fmt|lint|build|metadata`, aucun processus de compilation.
- Aucun dossier `target/` créé (le lock reste détenu par GLMGP).
- Aucune opération Git (commit, branch, stash), aucun cochage de tâche.
- Aucun restart, aucune config ni base de production touchée.
- Aucun fournisseur réel lancé, aucune nouvelle app T3.
- Seules écritures : les 2 déclarations `cfg(test)` autorisées + ce rapport.
