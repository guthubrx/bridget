# Raccordement des tests lineage natifs — ronde 2 (r2)

Date : 2026-10-10. Sous-agent test GLM, ronde 2. Mission : raccordement des 4
fichiers de tests lineage ronde 1 + adaptation des fixtures à l'API finale GP.
**Compilation : AUCUNE.** Aucun `cargo` (test/fmt/lint/build/metadata) lancé,
aucun dossier `target/` créé, zéro opération Git, zéro cochage de tâche. Le
compilateur reste la propriété exclusive de GLMGP jusqu'au point root.

Résultat global : **préparation complète, exécution zéro**. Les 4 fichiers de
tests sont câblés, alignés sur l'API production libérée, et vérifiés par
lecture seule. Aucune affirmation numérique PASS ne figure dans ce rapport :
aucun test n'a été exécuté en r1 ni en r2. Le GO d'exécution viendra du
principal, après le check production GP et la prise en charge de la baseline
des fixtures.

---

## 1. Périmètre et ownership

Worktree : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`.

Fichiers de tests (ownership exclusif sous-agent, créés r1, relus intégralement
en r2) :

| Fichier | Tests | Rôle |
|---|---|---|
| `crates/bridget-daemon/src/delegation_lineage_tests.rs` | 10 | Magasin projeté : migration, seq, pages, show, cancel, profondeur |
| `crates/bridget-daemon/src/daemon/native_lineage_tests.rs` | 9 | Daemon : gardes, list/show/journal, cancel, watch socket réelle, marqueur, snapshot V2 |
| `crates/bridget-transport/tests/lineage_contract_test.rs` | 6 | Contrat filaire fermé : codec, capacités, événements, codes |
| `crates/bridget-daemon/tests/lineage_cli_test.rs` | 3 | CLI bout en bout : grammaire, e2e daemon réel, watch exit 0 |

Éditions autorisées en r2 (seules modifications de fichiers production) :

```rust
// fin de crates/bridget-daemon/src/delegation_lineage.rs
#[cfg(test)]
#[path = "delegation_lineage_tests.rs"]
mod delegation_lineage_tests;

// fin de crates/bridget-daemon/src/daemon/native_lineage.rs
#[cfg(test)]
#[path = "native_lineage_tests.rs"]
mod native_lineage_tests;
```

Aucune autre ligne production n'a été touchée. Aucun export production n'a été
ajouté pour atteindre une méthode privée : les tests restent des modules
descendants `cfg(test)` et héritent de la visibilité privée.

Sources d'oracle : `specs/149-sous-agents-lineage/contracts/lineage.md` (contrat
fermé v1) et `specs/149-sous-agents-lineage/test-strategy.md` (sections 4-10).
Le contrat n'a jamais été modifié.

---

## 2. Comptage factuel — écart vs ronde 1

Correction factuelle de résultat 1 : le compte annoncé « 27 fonctions test
(store 10 / daemon 8 / wire 6 / CLI 3) » est faux. Comptage par `grep '#[test]'`
sur les 4 fichiers :

| Fichier | Annoncé r1 | Réel |
|---|---|---|
| store (delegation_lineage_tests) | 10 | 10 |
| daemon (native_lineage_tests) | 8 | **9** |
| wire (lineage_contract_test) | 6 | 6 |
| CLI (lineage_cli_test) | 3 | 3 |
| **Total** | **27** | **28** |

Le test daemon supplémentaire est
`native149_snapshot_v2_exclut_le_filtre_avant_normalisation`. L'écart est
consigné tel quel. Aucun test n'a été supprimé ni fusionné pour recoller au
compte annoncé.

Liste exacte des 28 fonctions :

- Store : `delegation149_migration_148_derive_racine_parent_et_horodatages`,
  `delegation149_migration_fermee_cycle_profondeur_et_volume`,
  `delegation149_insert_seq_signal_durable_et_horodatages`,
  `delegation149_lectures_silencieuses_et_save_inchange`,
  `delegation149_conflits_insertion_rollback`,
  `delegation149_page_bornes_ordre_pagination_budget_et_curseur`,
  `delegation149_entree_titre_erreur_etats_fermes`,
  `delegation149_show_bornes_offset_utf8_et_fenetres`,
  `delegation149_cancel_rejeu_mismatch_etats_et_plafond`,
  `delegation149_parent_lineage_profondeur_et_descendants`.
- Daemon : `native149_validate_ferme_les_formes_ouvertes`,
  `native149_gardes_client_et_authorite`,
  `native149_list_show_journal_et_disponibilite_journal`,
  `native149_cancel_recu_rejeu_et_mismatch_via_flux`,
  `native149_watch_ready_changed_silence_resync_et_invalidation`,
  `native149_watch_refuse_sans_socket_ou_sans_capacite`,
  `native149_page_journal_bornes_permissions_et_fermeture`,
  `native149_marqueur_projection_ferme_et_exclut`,
  `native149_snapshot_v2_exclut_le_filtre_avant_normalisation`.
- Wire : `lineage149_version_et_borne_seq_figees`,
  `lineage149_actions_snake_case_roundtrip_et_fermeture`,
  `lineage149_capacite_exigee_par_action`,
  `lineage149_evenements_watch_fermes`,
  `lineage149_codes_erreur_fermes_et_result`,
  `lineage149_enveloppes_wire_humaines`.
- CLI : `lineage_cli_149_grammaire_fermee_sans_daemon`,
  `lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel`,
  `lineage_cli_149_watch_ready_puis_fermeture_propre`.

---

## 3. Correction factuelle ronde 1 appliquée aux fixtures

Ronde 1 affirmait : `#[serde(default)]` couvre les nouvelles clés. C'est vrai
pour le JSON, faux pour les struct literals Rust. Les deux fixtures internes
listent donc explicitement les 26 champs actuels de `Task`
(`crates/bridget-daemon/src/delegation.rs`, `pub(crate) struct Task`) :

- `permission_snapshot: None` et `effective_posture: None` dans les deux
  fixtures internes (baseline legacy 148 voulue).
- `root_owner_agent_id`, `parent_task_id`, `updated_at`, `started_at`,
  `completed_at` renseignés explicitement, avec valeurs cohérentes par
  scénario (migration 148 : dérivation attendue par `initialize_lineage` ;
  scénarios 149 : racine et parent déjà posés).
- Les payloads JSON v148 (fixture `v148_payload` du store, `seed_task` du CLI)
  omettent volontairement les champs 149. `#[serde(default)]` les tolère à la
  désérialisation : c'est la preuve de backcompat JSON voulue par le contrat.
- `NativeDelegationRequest::Delegate` : `posture: None` et `effort: None`
  posés explicitement dans les fixtures qui construisent la variante Rust.
  Les payloads JSON omettent ces clés, tolérées par `#[serde(default)]`.

Aucune fixture ne prétend compiler « grâce au default » : chaque clé nouvelle
est écrite ou explicitement omise avec le motif.

---

## 4. Vérification statique de l'API — preuves par lecture

Chaque symbole utilisé par les tests a été confirmé par lecture, avec
signature et visibilité. Aucune compilation.

### 4.1 Production libérée GP

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

### 4.2 Daemon

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

Visibilité : les champs privés de `DaemonState` sont lisibles depuis
`daemon::native_lineage::native_lineage_tests` (module descendant). Le glob
`use super::*` traverse les deux niveaux. L'import explicite de `Value`/`json`
prime sur le glob, sans ambiguïté.

### 4.3 Store et annexes

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

---

## 5. Inventaire des fixtures Task hors ownership — lecture seule

GP est propriétaire du compilateur et des fixtures production. Inventaire
demandé, sans modification :

| Localisation | Forme | Correction mécanique nécessaire |
|---|---|---|
| `src/daemon/native_delegation.rs:473` | Seul struct literal `Task` complet hors tests | **Aucune.** Le literal inclut déjà `permission_snapshot` et `effective_posture` (source GP libérée) |
| `tests/native_delegation_e2e.rs` | Construction via saga / payloads JSON sérialisés | Aucune. `#[serde(default)]` tolère l'absence des champs 149 à la désérialisation |
| `tests/native_delegation_real_glm.rs` | idem, via serde | Aucune |
| Autres fichiers | Aucun autre semis SQL `native_delegations` ni `to_value(&task)` | Rien à faire |

Conclusion : zéro correction mécanique en attente côté fixtures production.
Le seul literal complet est déjà conforme à l'API 149.

---

## 6. Oracles et bornes par scénario

Bornes détaillées, tracées au contrat `contracts/lineage.md`. Ce sont des
oracles d'assertion, pas des résultats d'exécution.

### 6.1 Store (`delegation_lineage_tests.rs`)

- **Migration 148 → 149** : `initialize_lineage` dérive `root_owner_agent_id`
  et `parent_task_id` depuis les seules identités native148 durables
  (correspondance `owner`/`child_instance`) ; `updated_at` recalé à
  `created_at`. Refus fermés : cycle détecté (`visited`), profondeur > 8,
  volume > 4096 enregistrements.
- **Seq et génération** : chaque insertion/save visible incrémente `seq` de
  façon durable dans `native_delegation_projection_meta` ; les lectures
  silencieuses et les saves invisibles n'incrémentent pas
  (`visible_changed`).
- **Conflits** : échec d'insertion → rollback complet, meta inchangée.
- **Pages** : bornes limit 1..=100 ; ordre `(created_at, task_id)` ;
  curseur refusé si racine différente, UUID non canonique, ou
  génération/seq divergents (`snapshot_changed`) ; budget 128 KiB − 4 KiB ;
  `next_cursor` seulement si plus de résultats.
- **Entrée** : 20 clés exactes (`ENTRY_KEYS`) ; titre = première ligne non
  vide, contrôles interdits filtrés, plafond 256 caractères ; erreur plafonnée
  à 1024 ; `posture` = `effective_posture.or(posture).unwrap_or(Discovery)` ;
  les 9 états fermés, sinon `store_unavailable`.
- **Show** : limit 1..=16384 ; offset sur frontière de caractère sinon
  `result_offset_invalid` ; fenêtrage UTF-8 descendant ; total > 256 KiB →
  `store_unavailable` ; résultat servi seulement à l'état `result_available`.
- **Cancel** : reçu idempotent rejoué à l'identique ; mismatch
  task/request → `envelope_mismatch` ; plafond 4096 reçus → `resource_limit` ;
  seule une tâche non terminale passe en `cancelling` et mute la projection.
- **Profondeur** : `parent_lineage` refuse au-delà de la chaîne de 8
  (`visited.len() >= 8` → `task_depth_limit`) ; descendants : CTE récursive
  `depth < 8`, `LIMIT 4097` puis refus `descendant_limit` si > 4096.

### 6.2 Daemon (`native_lineage_tests.rs`)

- **Validate** : version != 1 → `unsupported_version` ; fil vide/>2048/
  contrôles → `invalid_request` ; racine relative ou > 4096 refusée ; bornes
  list 1..=100, show 1..=16384, journal 1..=100 et after_seq ≤ MAX_SEQ ;
  UUID canoniques obligatoires.
- **Gardes** : client non négocié ou capacité absente → refus mappé depuis
  `HumanThreadViewError` ; autorité : fil non lié ou projet divergent →
  `binding_unavailable` / `project_mismatch` ; re-garde stable entre deux
  verrous (owner/root figés dans le `Context`).
- **List/Show/Journal** : sous verrou pour la résolution, E/S hors verrou ;
  `journal_available` = présence réelle du dossier `sessions/<child>` ;
  tâche hors racine → `task_unavailable`.
- **Cancel via flux** : reçu complet via `handle`, rejeu sans mutation, drive
  du moteur déclenché seulement si non terminal.
- **Watch socket réelle** : `UnixStream::pair` + `handle_connection` dans un
  thread ; `Ready` version 1 seq 0 (distinct du seq du magasin) ;
  `Changed` après mutation projetée ; silence en l'absence de mutation ;
  `Resync` déclenché par publish direct d'une génération divergente
  (`publish(&mut st, &owner_agent(), &ProjectionMutation { generation:
  "88888888-…", seq: 900 })`) — déterministe, sans dépendance au timing ;
  double liaison → `Error` + EOF ; invalidation de garde → `Error`.
- **Refus watch** : sans socket inscriptible ou sans capacité → `Error`
  filaire, jamais de silence.
- **Page journal** : bornes limit 1..=100 ; permissions 0700 répertoire /
  0600 fichiers exigées ; trou de seq → `journal_gap` ; entrée trop large →
  `entry_too_large` ; > 256 fichiers → `resource_limit` ; budget 16 KiB ;
  dossier absent → `journal_unavailable`.
- **Marqueur** : `is_native_projection` ferme les 7 clés camelCase exactes,
  version 1, UUID canoniques, seq ≤ MAX, 9 états ; fil sans marqueur → faux.
- **Snapshot V2** : le fil marqueur natif est exclu AVANT la normalisation —
  un gabarit identique au helper `shell()` du test 147 existant, sans
  `providerInstanceId`, passe alors que la même forme post-normalisation
  serait refusée.

### 6.3 Wire (`lineage_contract_test.rs`)

- `HUMAN_LINEAGE_VERSION = 1` et `HUMAN_LINEAGE_MAX_SEQ = 2^53 − 1`
  (= 9 007 199 254 740 991) figés.
- 7 actions en roundtrip snake_case exact ; fermeture : action inconnue,
  champ en trop (action et requête entière), `deny_unknown_fields`.
- Capacité par action : view pour list/show/journal, watch pour watch,
  cancel pour cancel ; noms filaires exacts `human_lineage_{view,watch,cancel}_v1`.
- 4 événements watch fermés en roundtrip (`ready`/`changed`/`resync`/`error`)
  ; statut inconnu, champ en trop, champ manquant refusés.
- 11 codes erreur fermés + `result()` : `retryable` vrai pour
  `store_unavailable` seul ; 4 clés exactes.
- Enveloppes `HumanLineage` / `HumanLineageResult` / `HumanLineageWatchEvent`
  en roundtrip ; type inconnu refusé.

### 6.4 CLI (`lineage_cli_test.rs`)

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

---

## 7. Risques ronde 1 — statut après r2

| Risque r1 | Statut r2 |
|---|---|
| Profondeur 8 doit découler du contrat | Le contrat dit « La profondeur maximale reste 8 ». Les tests affirment selon la lecture du code : première mission native = profondeur 1 ; chaîne de 8 maillons admise ; un parent au bout d'une chaîne de 8 refuse un 9e maillon (`visited.len() >= 8`). Voir §8. |
| Oracle profondeur à confirmer par l'owner natif | Incertitude consignée §8. Aucune modification du contrat, aucun test n'invente un code filaire (les libellés `task_depth_limit` / `parent_task_unavailable` / `descendant_limit` sont des chaînes d'erreur internes daemon, pas des codes filaires fermés). |
| Resync déterministe, pas de timing fragile | Couvert par publish direct d'une génération divergente du publisher. Le seuil `pending.len() >= 16` n'est PAS testé (§8). |
| stable_uuid répliqué acceptable | Oui : golden de comportement public, justifié §6.4. |
| Semis SQL seq 0 | Oui : oracle de fixture, pas une mutation prod. |
| follow CLI via AttachRelay réel | Non couvert (§8), listé comme preuve future. |
| Trame watch invalide (`invalid_message`) | Non couvert (§8), listé comme preuve future. |

---

## 8. Limites et preuves futures

Rien de tout ceci n'est prétendu couvert :

1. **Exécution** : zéro test lancé en r1 et r2. Aucune affirmation PASS.
   La compilation réelle reste à GLMGP ; le GO d'exécution vient du
   principal.
2. **Seuil `pending.len() >= 16` du Resync** : non testé. Le couvrir par le
   timing de 17 mutations rapides serait fragile. Preuve future possible :
   publish direct de 17 mutations en attente, ou seuil injectable — décision
   à l'owner.
3. **`journal_follow` live (follow CLI via vrai AttachRelay)** : non testé.
   La voie froide (pages) est couverte par daemon et CLI. Preuve future :
   fixture attach réelle dans les tests daemon, si elle reste dans
   l'ownership tests.
4. **`invalid_message` (trame watch invalide)** : non testé directement. Le
   branchement en tête de `handle_wrapper_message` est vérifié par lecture
   (`daemon.rs:14038`). Preuve future : trame inconnue envoyée sur une
   connexion qui porte un watch, attente `Error` + EOF.
5. **Sémantique exacte de la borne de profondeur 8** : les tests affirment
   « chaîne de 8 admise, 9e maillon refusé ». Si l'owner natif tranche autre
   (par exemple profondeur 0-indexée), seuls les cas limites de
   `delegation149_parent_lineage_profondeur_et_descendants` et la partie
   migration fermée changent d'oracle. À arbitrer avant exécution, sans
   modifier le contrat.
6. **`native_permission_refusals` et les API permissions GP**
   (`NativePermissionFact.unavailable_code`, `ClaudeActiveTurn`,
   `CodexReaderContext`, `ManagedSession.provider_permissions`) : prises en
   compte dans la vérification d'API, mais hors périmètre des tests lineage.
   Elles appartiennent aux 5 tests GP préparés par GLMGP.

---

## 9. Interdits respectés

- Aucun `cargo test|fmt|lint|build|metadata`, aucun processus de compilation.
- Aucun dossier `target/` créé (le lock reste détenu par GLMGP).
- Aucune opération Git (commit, branch, stash), aucun cochage de tâche.
- Aucun restart, aucune config ni base de production touchée.
- Aucun fournisseur réel lancé, aucune nouvelle app T3.
- Seules écritures : les 2 déclarations `cfg(test)` autorisées + ce rapport.
