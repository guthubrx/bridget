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
### T014 et T056 Continuations : ✅

- Le producteur réel est `schedule_execution_recovery`, appelé au réenregistrement d'un wrapper sans tour actif. Avant toute reconstruction, il relit `control_state` et passe par `reserve_governed_continuation`.
- Le contexte fermé `RecoveryAfterIdleWrapper` autorise uniquement cette reprise à réserver un parent encore actif dans SQLite, car l'absence de tour vient d'être attestée par le wrapper. Les continuations ordinaires conservent leur exigence de parent inactif.
- Pause, limite de budget, course ou manque de faits arrêtent la reprise avant la création de la génération fille. Seules `Reserved` et le rejeu de la même réservation autorisent la reconstruction.
- Preuves : `cargo test -p bridget-daemon --test execution_budget_test` : **2 passés, 0 échec** ; `cargo test -p bridget-daemon --lib --features test-support daemon::presence_tests::spec_087_pause_differe_la_continuation_de_reprise -- --exact` : **1 passé, 0 échec**.
- Le scénario historique `spec_079_reprise_register_livre_le_message_exact_une_seule_fois` reste rouge avant d'atteindre la reprise : sa fixture enregistre `agent-2`, invalide depuis l'adoption des UUID v4. C'est le même défaut préexistant déjà consigné pour T043, hors du chemin T056.

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

### T028 et T051 Producteurs de boîte humaine : ✅
- `chain_exhausted` est déposé pour `chaine_preautorisee_epuisee`, `intervention_required` couvre l'autre motif ; la saturation budget conserve son dépôt unique existant. La queue durable `human_inbox_outbox` est le sous-outbox dédié au protocole `HumanInboxDeposit` et est relevée par `reconcile_human_inbox_with_limits`.
- Un rapport de revue accepté avec verdict dépose `review_verdict_pending` (`review-verdict:<delegation_id>`) et une proposition d'activation dépose `activation_approval` (`activation:<approval_id>`), chacun dans la transaction qui enregistre son fait source. Les options sont fermées.
- La migration des identités exclut le destinataire réservé `human`, y compris dans les enveloppes JSON.
- Preuves ciblées sur le serveur : `coordination_store_integration chaine_epuisee_depose_un_unique_item_humain_durable` : **1 passé, 0 échec** ; `guichet_greffe_integration verdict_concordant_termine_la_revue_sans_clore_l_objectif` : **1 passé, 0 échec** ; `approval_atomicity_integration approbation_et_activation_sont_atomiques_et_les_octets_restent_immuables` : **1 passé, 0 échec** ; `identity_migration` : **1 passé, 0 échec**. La cible intégrale `guichet_greffe_integration` garde deux rouges préexistants de migration (`migration_v6_vers_v7_preserve_les_agregats_et_ajoute_les_recus`, `parcours_v16_v17_v18_v19_v20_conserve_les_refus_et_motifs_de_revue`) sur `duplicate column name: deferred_reason`, hors scénario T028.

### T032 et T052 Décisions humaines : ✅
- La relève encadre désormais explicitement les frontières `AfterFetchBeforeApply` et `AfterApplyBeforeAck`. Elle marque la décision dans la même transaction que son effet, puis seulement l'acquitte côté daemon.
- `ack`, `cancel`, `raise_budget` et `reassign:<agent_id>` sont couverts. Une réassignation après intervention humaine crée une génération ouverte, un épisode de rappel et une demande durable vers l'agent choisi ; un rejeu ne crée pas de seconde génération.
- Preuves : `cargo test -p maicie --test controle_referent_087` : **11 passés, 0 échec** ; `coordination_store_integration chaine_epuisee_depose_un_unique_item_humain_durable` : **1 passé, 0 échec**. Le crash simulé entre relève et application relit la décision, l'applique une seule fois, puis l'acquitte.

### T033 et T053 Fermeture automatique des items humains : ✅

- `HumanInboxClose { item_id, reason }` et `HumanInboxClosed` sont classés comme messages service dans les deux matrices daemon. Le bras dédié appelle `human_inbox::close_self` après contrôle de version.
- Maicie conserve désormais le `item_id` reçu lors du dépôt. La migration v25 remet les anciens dépôts attestés en préparation une seule fois afin de récupérer cet identifiant via le dépôt idempotent. Chaque relève ferme un item dont `objective_id` est clos/absent ou dont `delegation_id` est annulée/absente, avec le motif strict `object_vanished`, puis conserve `closed_at` pour ne pas répéter l'appel.
- Preuves serveur : trame transport **1 passé, 0 échec** ; unité daemon `human_inbox` **8 passés, 0 échec** dont le dépôt répété (`occurrences = 2`) et `close_self` idempotent ; scénarios Maicie objectif clos et délégation annulée **2 passés, 0 échec** ; contrat client service **1 passé, 0 échec** ; suite `controle_referent_087` **13 passés, 0 échec** ; `cargo check -p bridget-daemon` réussi.

### T054 Refus de plafond structuré : ✅

- `GuichetRefusalReason::BudgetReached { cap, open }` remplace le faux `mutation_invalid`. Maicie conserve le motif `budget_reached` dans son reçu durable et porte les valeurs attestées dans la trame ; à un rejeu de lease, la charge existante est reprise sans recalculer le budget courant.
- Preuves serveur : sérialisation et rejeu **1 passé, 0 échec** ; sélection de la variante depuis `DelegateError::BudgetReached` **1 passé, 0 échec** ; protocole guichet **1 passé, 0 échec** ; `cargo check -p bridget-daemon` réussi.

### T049 Focus guichet en attente : ✅

- Le cas « agents correspondants tous temporairement indisponibles » traverse le guichet avec le statut fermé `waiting_for_agent`, un `objective_id` et aucun identifiant de délégation, de message ou de participant. La validation stricte du reçu, le daemon et la projection MCP reconnaissent ce troisième état sans affaiblir `created`.
- `spec_087_focus_guichet_sans_agent_disponible_attend_et_alerte` prouve le chemin complet : focus durable, aucune délégation fictive, item `focus_waiting_agent` après 60 s et rejeu sans second focus.

### T050 Cible Git gelée du focus : ✅

- `review_git::freeze_origin_default_review_target` lit uniquement les références locales rapatriées (`refs/remotes/origin/HEAD` puis sa tête), sans accès réseau ni chemin de requête. Un focus sans `review_target` exige le projet de revue configuré correspondant à `focus.project_id`.
- `spec_087_focus_guichet_gel_la_branche_origin_par_defaut` prouve que la délégation durable porte `origin/main` et le SHA observé. `spec_087_focus_gel_la_branche_par_defaut_origin` couvre la primitive Git isolée.

### T041 et T055 Projection du focus : ✅

- Maicie publie à chaque relève une projection minimale du focus sur sa connexion de boîte humaine. Bridget la persiste dans `control_focus_projection` et l'interface la relit par `ControlFocusRead` : le daemon ne lit jamais la base Maicie et le rafraîchissement de l'interface ne lance jamais `maicie status`.
- La colonne Projets affiche les objectifs ouverts, avec le focus toujours premier et visuellement distinct. Si la projection de mission est temporairement en retard, le focus reste néanmoins visible depuis sa projection Bridget.
- Preuves : `node --test crates/bridget-daemon/assets/ui/app.js` : **136 passés, 0 échec** ; `cargo test -p maicie --test greffe_central_channel_integration` : **3 passés, 0 échec** ; `cargo test -p bridget-transport protocol::control_and_inbox_contract_tests::spec_087_trames_etat_de_controle_font_l_aller_retour -- --exact` : **1 passé, 0 échec** ; `cargo check -p bridget-daemon` : réussi.

### T043 Gates finales : ✅

- `cargo fmt --all -- --check` : propre.
- `cargo test -p bridget-transport` : **242 passés, 0 échec, 1 ignoré**.
- `cargo test -p bridget-daemon --lib --features test-support` : `referent_control::` **9**, `human_inbox::` **8**, `ui::` **81**, `project_` **38**, `matrice_roles` **11** - soit **147 passés, 0 échec**.
- `cargo test -p maicie --lib` : `control::` **5**, `routines::` **0** (aucun test ne porte ce préfixe), `guichet::` **5**. Intégrations touchées : `greffe_central_channel_integration` **3** et `review_git_integration` ciblé **1** - soit **14 passés, 0 échec**.
- `node --test crates/bridget-daemon/assets/ui/app.js` : **136 passés, 0 échec**.
- Clippy avec `-D warnings` : les trois refus restants sont également présents sur `main` nu : `AcpEvent` trop asymétrique, `ProjectRole::default` dérivable et `format!` inutile dans un test Docker. La quatrième alerte initiale venait de la variante `Delegate` de ce lot : elle est traitée localement par une justification ciblée, sans modifier les octets ni l'API publique de la trame. Aucun nouveau refus Clippy n'est introduit par ce lot.
- Comparaison de base : le premier rouge de `matrice_roles` venait d'une fixture qui utilisait `child-events`, identifiant devenu invalide car les agents sont désormais des UUID v4. La fixture a été alignée et la matrice est verte. La base `main` nue n'a pas atteint ce test : sa compilation s'arrête avant sur `ServiceRequestPayload::Delegate` dont les champs `origin` et `focus` sont absents, état antérieur de synchronisation des branches.

### Self-review Article XIX/XX - T049/T050

- Pourquoi cette solution est nécessaire : la réponse du focus sans agent ne pouvait pas être persistée sous `created`, et l'absence de cible Git laissait une revue de focus non gelée.
- Pourquoi elle est plus simple ou plus maintenable : un statut fermé supplémentaire exprime le fait réel sans rendre permissif `created`; une seule fonction `review_git` réutilise la capture Git bornée existante.
- Hypothèses prises : la branche par défaut `origin` a déjà été rapatriée et `origin/HEAD` pointe vers elle; son absence reste une erreur technique rejouable.
- Vérifications réalisées : `cargo test -p maicie --test review_git_integration spec_087_focus_gel_la_branche_par_defaut_origin -- --exact` : **1 passé, 0 échec**; `cargo test -p maicie --test greffe_central_channel_integration` : **3 passés, 0 échec**; `cargo test -p bridget-transport protocol::tests::service_guichet_messages_roundtrip_et_restent_hors_attach -- --exact` : **1 passé, 0 échec**; `cargo check -p bridget-daemon` : réussi.
- Non vérifié : parcours opérateur réel de T045, volontairement hors de ce lot.
- Code supprimé ou évité : aucune lecture réseau Git, aucune délégation ou outbox fictive, aucun relâchement de validation pour `created`.
- Complexité ajoutée et justification : un état de protocole et ses trois validateurs, nécessaires pour rendre le reçu durable, rejouable et non ambigu.
### T029 Dette de réponse : ✅ palier 3 d'une demande du référent ⇒ item `reply_debt`
### T030 Canal externe : ✅ `human-channel.json` 0600, chemin absolu, commande avec résumé borné sur stdin, échec consigné
### T031 Rappel : ✅ `remind_overdue` (module) ; **branchement dans le thread horaire** : voir Convergence
### T034 CLI inbox : ✅
### T035 Interface, boîte : ✅ panneau « En attente de toi », compteur, notification native (permission accordée, page cachée), routes `/v1/inbox`, `/v1/inbox/<id>/resolve`
### T036 Plafond : ✅ `bridget control budget <n>` ; descripteur de réglage serveur : **non fait**, l'interface expose le plafond directement dans « Paramètres du serveur » (T038)
### T038 Interface, plafond : ✅ section « Objectifs automatiques », un bouton « Enregistrer »
### T039 Bandeau complet : ✅ pause, plafond, compteur d'items ; **focus courant dans le bandeau** : fait par projection Bridget publiée par Maicie, sans appel UI à `maicie status`.
### T040 Historique : ✅ `bridget control status --history` (trame `ControlHistory`)
### T042 Essai adverse : ✅ test unitaire `spec_087_un_agent_par_ses_outils_declares_ne_mute_rien` (client MCP-like sans capacité, client avec capacité mais périmètre non humain, origine forgée depuis un agent enregistré) ; fichier d'intégration prévu remplacé par ce test, même propriété
### T048 Alerte de route humaine : ✅ `spec_087_reprise_d_une_route_humaine_vivante_est_signalee`

### Non cochées côté daemon, avec raison
- T045 : validation opérateur, exige le référent
- T046 : mise en service ; le pipeline interdit le commit automatique, donc pas de déploiement de code non commité

## Mutants

| Mutant | Tests attendus rouges | Résultat brut |
|---|---|---|
| `admit_autonomous_effect` forcé à `Admitted` (server, copie restaurée ensuite) | ronde, relances, continuation, table de vérité | `test result: FAILED. 0 passed; 4 failed` : `la_pause_globale_rend_la_continuation_paused`, `garde_unique_table_de_verite`, `spec_087_pause_differe_les_relances_mais_pas_le_palier_trois`, `spec_087_pause_differe_la_ronde_puis_la_reprise_la_livre` |
| `autonomy_runtime_for_control` renvoie `observed` pendant une pause (copie restaurée ensuite) | `spec_087_pause_differe_la_continuation_de_reprise` | `test result: FAILED. 0 passed; 1 failed` : la remise de reprise était présente malgré la pause. Garde restaurée puis test vert. |

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
