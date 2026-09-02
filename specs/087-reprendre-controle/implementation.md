# Journal d'Implémentation - Reprendre le contrôle

## Métadonnées
- **Spec** : 087-reprendre-controle
- **Branche** : 087-reprendre-controle (locale et worktree serveur `/home/moi/bridget-referent/.worktrees/087-reprendre-controle`)
- **Démarré** : 2026-09-02
- **Terminé** : En cours
- **Base** : origin/main `bf89d4c7`
- **Méthode** : édition locale, `rsync` vers le worktree serveur, `cargo fmt` et compilation sur le cache `/home/moi/bridget-referent/bridget/target` (daemon, transport) et `target-maicie` (versant Maicie, agent dérivé), tests joués par périmètre déclaré, comptes annoncés
- **T001, réservation v24** : `git grep "SCHEMA_VERSION: i64 = 24"` sur toutes les branches distantes après `git fetch --all` : aucune ligne ; `origin/main:plugins/maicie/src/store.rs:58` = 23. v24 réservée pour ce lot.
- **Versant Maicie** : journal séparé `implementation-maicie.md`, à fusionner ici à la clôture.

## Progression

### T001 Réservation de la migration v24 : ✅ (preuve ci-dessus, 2026-09-02 04:52 UTC)
### T002 Scénarios Gherkin : ✅ `tests/features/087-reprendre-controle.feature`, 18 scénarios
### T003 ADR 027 : ✅ `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md`

### T004 Protocole : ✅
- `crates/bridget-transport/src/protocol.rs` : constantes, capacités `ControlStateV1` / `HumanInboxV1`, refus fermés, `HumanInboxKind` (`as_sql`, `sql_in_clause`, `from_sql`), `DelegateOrigin`, `DelegateFocus`, `ControlStateFrame`, `ControlEventFrame`, items et décisions de boîte, trames `ControlStateRead/Set/History`, `HumanInbox{Deposit,List,Resolve,Decisions,Ack}` et réponses, `ProjectRoundRefusal::ControlPaused`, `ServiceRefusal::HumanOriginForbidden`, `GuichetRefusalReason::HumanOriginInvalid`, `sha256_hex`, `human_message_content_seal`
- `cargo test -p bridget-transport` : **242 passés, 0 rouge, 1 ignoré** (4 tests neufs `spec_087_*`)

### T005 Initialiseurs `Delegate` : ✅
- 11 initialiseurs mis à jour ; `guichet_request_is_valid` accepte `origin`/`focus` en v2 et les refuse en v1
- `cargo test --workspace --features test-support --no-run` : compile, hors les deux tests d'intégration déjà cassés sur main (`mcp_injection_smoke_test`, `idempotency_crash_test`, champ `name`)

### T006 `referent_control.rs` : ✅ 9 tests unitaires verts
### T007 `human_inbox.rs` : ✅ 8 tests unitaires verts (dont canal externe factice, échec consigné, config 0600 et chemin absolu)

### T008 Dispatch daemon et matrices : ✅
- Huit trames classées dans les deux matrices exhaustives ; ensemble service `[HumanInboxV1]` admis ; `ClientHello` admet `ControlStateV1` (défaut trouvé par le test adverse T042 : la capacité était filtrée en silence, l'interface et la CLI auraient été refusées en production)
- `matrice_roles_tests` : 27 verts, 1 rouge préexistant (`attente_de_descendance_derive_le_parent…`, dans la base nue)

### T011 CLI : ✅ `bridget control status [--history] | pause [--reason] | resume | budget <n>`, `bridget inbox list [--all] | resolve <id> <choix>`, terminal interactif exigé pour les mutations, pied de `who` (`emit_control_footer`), `cargo build -p bridget-daemon` vert
### T012 Ronde : ✅ `spec_087_pause_differe_la_ronde_puis_la_reprise_la_livre`
### T013 Relances : ✅ `spec_087_pause_differe_les_relances_mais_pas_le_palier_trois` ; `ReminderAction::Deferred` porte `reason`
### T014 Continuations : ✅ `managed_supervisor::control_pause_tests::la_pause_globale_rend_la_continuation_paused`
- **Note honnête** : aucun appelant de production de `reserve_governed_continuation` n'existe sur main ; la garde est prête, le producteur n'est pas branché (dette antérieure)

### T018 Interface, pause : ✅ routes `GET/POST /v1/control/state` via socket (`control_request`, capacité `ControlStateV1`, périmètre `bridget-ui-control`), bandeau `#control-banner`, projection pure `controlBannerProjection`
### T019 Persistance et `who` : ✅ `spec_087_pause_survit_a_la_reconstruction_de_l_etat`, ligne `Contrôle : …` après `Daemon build-id`

### T020 Origine humaine fabriquée par le daemon : ✅
- `daemon.rs` bras `ServiceRequest` : principal humain = connexion enregistrée `ui` sous `UI_HUMAN_AGENT_ID` ; refus `HumanOriginForbidden` sinon ; fabrication déterministe (`hmo-<sha256(issuer_scope\nrequest_id)[..32]>`, `ts = issued_at`, hash canonique sans origine, scellé transport) ; message humain au ledger ; garde du greffe non appliquée au principal humain (ADR 027)
- Test : `spec_087_origine_humaine_fabriquee_par_le_daemon_et_refusee_aux_agents` (rejeu déterministe, scellé recalculé, ledger, pas d'attestation greffe)

### T024 (partie daemon) : ✅ `ExecutionStore::queue_priority_for`, `FOCUS_QUEUE_PRIORITY = 100`, test `spec_087_une_remise_de_focus_passe_devant_le_travail_ordinaire` ; s'applique à la file `QueueOnly` ; la référence `focus:<objective_id>` est posée par Maicie (demandé à l'agent dérivé)
### T026 Interface, focus : ✅ route `POST /v1/control/focus` par la connexion humaine du relais (`submit_focus`, réponse `GuichetResult`), formulaire « Travaille sur… » avec choix « Après le focus actuel / À la place du focus actuel » en amont (au lieu d'une confirmation après coup), test Node ; **focus en tête de la liste des objectifs** : non fait, l'interface ne liste pas les objectifs Maicie (reporté à T041).
- Preuve de reprise : `node --test crates/bridget-daemon/assets/ui/app.js` : **135 passés, 0 échec** ; `cargo test -p bridget-daemon --lib --features test-support ui::` : **81 passés, 0 échec, 771 filtrés**. Le compilateur signale un `mut` inutile préexistant dans `daemon.rs:23515`, sans échec.
### T027 Focus en attente d'agent : ✅
- `greffe_service.rs` distingue un projet réellement sans agent missionnable (refus explicite) d'agents missionnables tous temporairement indisponibles. Dans ce second cas, `open_focus_waiting_for_agent` ouvre le focus humain sans délégation ni outbox fictive et le place dans `focus_queue`.
- `reconcile.rs::reconcile_focus_waiting_agents` relit le focus actif sans délégation. Après `durations.normal_secs`, il dépose l'unique item durable `focus_waiting_agent`, clé `focus-waiting:<objective_id>`, avant la relève de la boîte humaine.
- Preuve : `cargo test -p maicie --test controle_referent_087` : **10 passés, 0 échec**. Le test `spec_087_focus_sans_agent_attend_puis_avertit_le_referent` constate zéro item à 59 s, un item à 60 s, aucune délégation fictive et aucun doublon au rejeu. `cargo build -p maicie` : **compilation réussie**.
### T029 Dette de réponse : ✅ palier 3 d'une demande du référent ⇒ item `reply_debt`
### T030 Canal externe : ✅ `human-channel.json` 0600, chemin absolu, commande avec résumé borné sur stdin, échec consigné
### T031 Rappel : ✅ `remind_overdue` (module) ; **branchement dans le thread horaire** : voir Convergence
### T034 CLI inbox : ✅
### T035 Interface, boîte : ✅ panneau « En attente de toi », compteur, notification native (permission accordée, page cachée), routes `/v1/inbox`, `/v1/inbox/<id>/resolve`
### T036 Plafond : ✅ `bridget control budget <n>` ; descripteur de réglage serveur : **non fait**, l'interface expose le plafond directement dans « Paramètres du serveur » (T038)
### T038 Interface, plafond : ✅ section « Objectifs automatiques », un bouton « Enregistrer »
### T039 Bandeau complet : ✅ pause, plafond, compteur d'items ; **focus courant dans le bandeau** : non fait (le relais n'appelle pas `maicie status`, qui réconcilie des outboxes à chaque appel ; voir Convergence)
### T040 Historique : ✅ `bridget control status --history` (trame `ControlHistory`)
### T042 Essai adverse : ✅ test unitaire `spec_087_un_agent_par_ses_outils_declares_ne_mute_rien` (client MCP-like sans capacité, client avec capacité mais périmètre non humain, origine forgée depuis un agent enregistré) ; fichier d'intégration prévu remplacé par ce test, même propriété
### T048 Alerte de route humaine : ✅ `spec_087_reprise_d_une_route_humaine_vivante_est_signalee`

### Non cochées côté daemon, avec raison
- T041 : focus en tête de liste ; l'interface ne connaît pas encore les objectifs
- T043 : gates joués partiellement (voir ci-dessous), à rejouer après fusion du versant Maicie
- T044 : `docs/regles-chantier.md` mis à jour, `quickstart.md` relu contre les libellés réels ; ce journal
- T045 : validation opérateur, exige le référent
- T046 : mise en service ; le pipeline interdit le commit automatique, donc pas de déploiement de code non commité

## Mutants

| Mutant | Tests attendus rouges | Résultat brut |
|---|---|---|
| `admit_autonomous_effect` forcé à `Admitted` (server, copie restaurée ensuite) | ronde, relances, continuation, table de vérité | `test result: FAILED. 0 passed; 4 failed` : `la_pause_globale_rend_la_continuation_paused`, `garde_unique_table_de_verite`, `spec_087_pause_differe_les_relances_mais_pas_le_palier_trois`, `spec_087_pause_differe_la_ronde_puis_la_reprise_la_livre` |

## Gates joués (versant daemon, transport, interface)

| Gate | Résultat |
|---|---|
| `cargo fmt --check -p bridget-transport -p bridget-daemon` | propre |
| `cargo test -p bridget-transport` | 242 passés, 0 rouge, 1 ignoré |
| `cargo test -p bridget-daemon --lib -- ui:: project_ referent_control human_inbox spec_087 focus_priority control_pause` | 144 passés, 0 rouge |
| `node --test crates/bridget-daemon/assets/ui/app.js` | 135 passés, 0 rouge (2 tests neufs) |
| `cargo clippy -p bridget-transport --all-targets -- -D warnings` | 4 erreurs, toutes présentes sur origin/main nu (`acp.rs:48`, `protocol.rs` impl dérivable, `format!` inutile dans un test, taille de variantes) ; aucune dans les lignes de ce lot |
| Suite lib complète `bridget-daemon` | à rejouer après fusion Maicie : la compilation échoue par intermittence tant que le versant Maicie est en cours d'édition (deux écrivains sur le worktree serveur) |

Découverte de méthode : la fixture `state_with_registered_agent` tenait `HOME_REGISTRY_LOCK` et paniquait (`InvalidAgentId("agent-2")`, identité non UUID depuis 081) ; le verrou empoisonné faisait échouer en cascade tous les tests suivants. Les cinq prises de ce verrou relisent désormais un verrou empoisonné. À mesurer sur la suite complète.

## Passe d'intégration des deux versants (2026-09-02, 08:40 CEST)

| Gate | Résultat |
|---|---|
| `cargo clean -p bridget-transport -p maicie -p bridget-daemon` | nécessaire : un artefact périmé faisait échouer la compilation de maicie comme dépendance du daemon (21,3 Gio libérés) |
| `cargo test -p bridget-transport` | 242 passés, 0 rouge |
| `cargo test -p maicie --test controle_referent_087` / `human_origin_attestation` / `bridget_client_contract` / `contract spec_087` | 9 / 10 / 13 / 4 passés, 0 rouge |
| `cargo test -p maicie --lib` | 88 passés, 1 rouge préexistant (`agent_retarget_requirements`) |
| `cargo test -p bridget-daemon --lib -- ui:: project_ referent_control human_inbox spec_087 focus_priority control_pause` | 144 passés, 0 rouge |
| Suite lib complète `bridget-daemon` | 699 passés, 146 rouges, 7 ignorés ; noms comparés à la base : 2 différences, toutes deux environnementales (`InvalidAgentId("agent-2")` de la fixture cassée sur main ; « binaire bridget de test absent » après mon `cargo clean`) ; aucun rouge imputable au lot |
| `cargo test --workspace --no-run` | compile, hors `mcp_injection_smoke_test` (champ `name`, préexistant) |
| `node --test app.js` | 135 passés, 0 rouge |
| T031 rappel horaire | branché dans le thread de purge (`remind_overdue`), recompilé, 24 tests 087 verts |
