# Journal d'implémentation 102

Statut : Implemented dans le worktree — 33/33 tâches cochées ; non commité, non fusionné, non déployé. Début 2026-09-17 à 20:39 CEST.
ETA initiale du pipeline (102 → 103 → 104) : 410–1180 min, fin haute 16:20 CEST le 18/09,
hors attente de l'intégration 101, décisions utilisateur et compactions de contexte.

## Socle et base commune (T001)

Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents
Branche : session-102-fils-inter-agents, HEAD 1738a072 (`docs(100)`).
`git status` au démarrage : `.specify/feature.json` et `AGENTS.md` modifiés,
`docs/decisions/038-fils-sollicitations-ciblees.md` et `specs/102-fils-inter-agents/`
non suivis. Aucun fichier Rust modifié, aucune copie manuelle du worktree 101.

Constat sur la 101 (20:37 CEST) : `git worktree list` montre 19 worktrees ;
`.worktrees/101-abonnements-t3` et `.worktrees/105-arret-boucles` sont à 1738a072
avec respectivement 21 et 26 fichiers modifiés non commités, 0 commit d'avance.
Le daemon installé (build-id `1738a072ba22-dirty`, adoption
`~/.cache/bridget-adoptions/105-20260917.bWasBb`, relancé 06:43) provient de la
105, qui englobe la 101. Le binaire installé n'est pas une preuve d'intégration Git.

Action : demande d'intégration envoyée à Bridget-enhance (Codex,
b280f81d-a418-4dbc-bfd8-35c50d8fedd1), propriétaire des deux worktrees, à 20:43 CEST
par `bridget send` idempotent (id b067aec4-74d3-42b2-a632-c7aaf9cdcfd3, remise en
vol). Demande : commiter la 101 puis la 105 (puis la 106 si prête) et fusionner
dans main, renvoyer le SHA de main. Borne annoncée : 21:03 CEST. Le CLI nu a
d'abord été refusé (`--from` requis avec identité attestée par filiation de
processus, portée d'émetteur d'au moins 22 caractères) ; consigné pour la doc.

Règle appliquée d'ici là : seuls des fichiers neufs propres à la 102 sont écrits
(feature Gherkin, harnais de test, ce journal). Aucun fichier partagé n'est édité
tant que la base commune n'est pas établie.

## Préflight

- `python3 ~/.speckit/scripts/sync-project.py` : constitution, standards, index et
  AGENTS.md déjà à jour.
- Primitives disponibles : commandes `/speckit.*` (specify, plan, audit-existing,
  tasks, analyze, implement) ; pas de `converge` : contrat manuel de la skill.
- Artefacts 102 présents et relus : spec, plan, research, data-model, contrat,
  quickstart, reuse-audit (PASS), tasks (0/32), analysis, test-plan, checklist,
  contre-revue bdget. Phases Specify/Plan/Audit Existing/Tasks sautées.
- `bridget who` : Bridget-enhance (codex) joignable pour la contre-revue.
- Préchauffage `cargo check -p bridget-daemon --tests` dans le worktree : OK.

## Progression

### T001 Base commune
- **Statut** : ✅ Complété à 21:13 CEST.
- **Arbitrage** : Bridget-enhance (propriétaire des worktrees 101/105/106) a donné
  à 21:03 son accord écrit explicite pour que j'assure commits et intégration
  « 101 → 105 → 106 dans main », en préservant l'arbre principal. Aucun agent
  concurrent, worktrees 102–104 non touchés par lui.
- **Commits créés par plumbing** (`write-tree` + `commit-tree` + `update-ref`,
  aucun checkout ni modification de fichier dans les worktrees d'autrui) :
  - `9a1329e1` feat(101): Attester les observations T3 et l'identite MCP (parent 1738a072, 32 fichiers) ;
  - `853f2903` fix(105): Arreter les reponses automatiques en boucle (parent 9a1329e1 ; delta pur 105 : t3code.rs, wrapper.rs, envelope.rs, acp.rs, claude_stream_json.rs, codex_app_server.rs, docs ; daemon.rs/observation.rs/store.rs identiques à la 101) ;
  - `a4244c42` build(106): Ajouter le script de build et l'entretien des caches (parent 1738a072 ; la section « Entretien des caches » d'AGENTS.md, présente seulement dans l'arbre principal, y est incluse).
- **Fusions dans main** : `105bbb04` merge(105) puis `f36804ea` merge(106), sans
  conflit. État sale antérieur de main comparé fichier par fichier au résultat :
  identique partout sauf README.md (+8 lignes de la 105) et AGENTS.md (en-tête de
  session 105) ; copie de sécurité `/tmp/main-dirty-RDd3` ; stash temporaire supprimé.
- **Base 102** : commit `2d39a2d7` docs(102) (artefacts de spec, ADR 038, feature.json,
  AGENTS.md) puis `a3e0786d` merge(102) de main ; conflits feature.json/AGENTS.md
  résolus en gardant la 102. `t3code_identity.rs` et `spec101_observation_test.rs`
  présents. Aucune copie manuelle du worktree 101.
- **Non fait** : aucun déploiement, aucun redémarrage, aucune recette complète ;
  vérification cargo ciblée (tests 100/101 intégration, lib spec101/spec105,
  harnais 102) lancée sur cette base, résultat consigné à T030 et ci-dessous.

### T002 Harnais isolé
- **Statut** : ✅ Complété.
- **Fichier** : `crates/bridget-daemon/tests/spec102_threads_test.rs` (créé) ;
  réutilise `tests/support/idempotent.rs` (racine `/tmp/bid-…`, home/état/tmp
  privés, `spawn_daemon`, `register_agent_as`). Ajouts propres : `spec102_root`
  (refuse toute racine hors `/tmp` et toute socket sous `~/.cache/bridget-core`),
  `stop_cooperatively` (SIGTERM au groupe du daemon de test, attente réelle de
  sortie, échec après 10 s, jamais de SIGKILL émis par le test).
- **Test** : `spec102_harness_isolated_daemon_starts_registers_and_stops_without_sigkill`
  — daemon isolé démarré, agent synthétique seul dans l'annuaire, arrêt coopératif,
  socket fermée. Commande : `cargo test -p bridget-daemon --test spec102_threads_test`
  → 1 passed (0,21 s), sur base 1738a072 avant fusion ; rejoué sur la base fusionnée
  (voir T030).

### T003 Scénarios Gherkin
- **Statut** : ✅ Complété (document, non exécutable).
- **Fichier** : `tests/features/102-fils-inter-agents.feature` (créé) — 36 scénarios
  V01–V36, chacun tagué avec l'identifiant V et le nom du test Rust cible ;
  cas négatifs inclus (V07, V14, V16, V17, V19–V23, V31, V32).
- **Liaison** : table scénario → test Rust ajoutée à la fin de test-plan.md.
- **Vérification** : lecture ; aucun moteur Python ajouté.

### T004–T005 Contrat, notice et négociation
- **Statut** : ✅ Complétés.
- **protocol.rs** : `ThreadRequest { version, request }`, `ThreadAction` (8 variantes fermées,
  `deny_unknown_fields`), `ThreadNotify` (`[]` / UUID[] / `"all"`), `ThreadResult { version, result }`,
  variantes `WrapperToDaemon::ThreadRequest`, `WrapperToDaemon::ThreadNoticeCapability { versions }`,
  `DaemonToWrapper::ThreadResult`. Tests `spec102_v33/v05/v22/v27` (4).
- **Écart au contrat, documenté** : la capacité d'alerte n'est pas un champ de `Register` /
  `Registered` mais un fait de connexion `ThreadNoticeCapability` envoyé après `Registered`,
  comme `DiskSpace`/`JournalReady`. Motif : ajouter un champ aux deux variantes aurait touché
  ~105 constructions littérales (43 `Register`, 62 `Registered`) dans 30 fichiers de tests et
  sources pour le même effet ; le fait post-enregistrement conserve « sans nouveau handshake »,
  la sélection liée à la connexion (`DaemonState.thread_notice_versions`, effacée avec la
  connexion) et la renégociation à chaque reconnexion. Un ancien wrapper n'annonce rien et ne
  reçoit aucune alerte ; un ancien daemon ignore la variante. Reporté dans contracts/thread-api.md
  et reuse-audit.md (arbitrage).
- **message.rs** : `BridgetMessage.thread_notice: Option<ThreadNotice>` (absent par défaut, omis
  à la sérialisation, `deny_unknown_fields`) ; 4 constructions littérales mises à jour.
- **communication.rs** : `canonical_send` ajoute à la FIN le domaine `bridget/thread-notice/v1`
  seulement si la notice est présente ; golden historique inchangé ; test V20 par champ.
- **daemon.rs** : matrices de rôles (Service et Client refusent `ThreadRequest`/capacité ;
  attach refuse par défaut) ; handler `ThreadRequest` = `live_connection_identity` sinon
  `identity_unavailable`, puis `threads::handle` ; `DaemonDirectory` (membres connus, faits
  `connected`/version d'alerte) ; capacité effacée à la déconnexion et au remplacement d'instance.
- **client.rs** : `thread_request` sur connexion attestée (même budget 10 s que `observation_request`).

### T006–T007 Schéma et transactions
- **Statut** : ✅ Complétés.
- **store/threads.rs** (créé, raccordé par `Store::init_schema` → `threads::ensure_schema`) :
  six tables + `thread_schema_migrations` (version 1, distincte de la version idempotence 6),
  `PRAGMA foreign_keys = ON` sur la connexion Store, FK membres sur entrées/reçus/sollicitations,
  CHECK `through_seq > base_seq`. Écritures en transaction unique : `thread_create`,
  `thread_post` (rejeu → mismatch → appartenance → clôture → reply_to → cibles → ACK joint →
  quotas → entrée → compteurs → intentions → opération), `thread_close`, `thread_read`,
  `thread_ack`, `thread_history`, `thread_list`, `thread_show` ; API des sollicitations
  (`thread_wake_candidates/reserve/defer/settle/by_delivery/in_flight`) prête pour T013–T014.
- **Tests** : V34 (base synthétique pré-102 avec ledger/demande, double ouverture, FK et CHECK
  effectifs par rusqlite, version idempotence inchangée) ; V31 concurrence (2 auxiliaires × 40
  dépôts → séquences 1..80 sans trou) ; V19/V20 idempotence.
- **Constat** : la rétention existante du ledger purge une ligne datée de 1970 ; la fixture V34
  utilise des horodatages du jour (ce n'est pas un effet 102).

### T008–T009 US1 côté daemon
- **Statut** : ✅ Complétés.
- **threads.rs** (créé) : bornes V1 en constantes, validation (UUID canonique minuscule, titre
  1–160 caractères, corps 1–16 Kio non blanc, entrée JSON ≤ 48 Kio, pages 1–200, liste 1–100),
  normalisation des cibles (soi ignoré avec notice, dédup, tri), empreinte SHA-256 par champs,
  projection des refus en codes du contrat, résultats JSON du contrat, `notice_body` /
  `notice_message_id` pour T013.
- **Tests d'intégration** (`spec102_threads_test.rs`, daemon isolé réel) : V01, V02 (redémarrage),
  V03 (20 dépôts, silence de B/C/D), V19, V20, V21 (erreurs uniformes, aucune fuite, refus
  `not_a_member` sans publication partielle), V22 (rôles nu/client/attach/service refusés,
  auxiliaire attesté agit comme son principal), V31 validation, V31 concurrence, V32 clôture,
  V34 migration. Commande : `cargo test -p bridget-daemon --test spec102_threads_test` → 12/12.

### T010 CLI `bridget thread`
- **Statut** : ✅ Complété.
- **cli.rs** : `thread` dans le répartiteur et l'aide ; `cmd_thread` (identité de session
  attestée requise, sortie JSON, exit 0/2/1) ; `parse_thread_args` strict (huit formes du
  contrat, `--` obligatoire pour le texte, options répétées/inconnues/superflues refusées,
  `--from`/script/chemin impossibles) ; `--member`/`--notify` : UUID canonique transmis tel quel,
  nom explicite résolu par `ListAgents` avec refus si inconnu ou ambigu.
- **Tests** : `spec102_v07_cli_noms_ambigus_refuses`, `spec102_v33_cli_thread_flags_stricts_et_formes_exactes`.

### T011 Outil MCP `bridget_thread`
- **Statut** : ✅ Complété.
- **mcp.rs** : schéma fermé (`additionalProperties:false`, énumérations, bornes), exécution via
  le client commun, `tool_result` marque `isError:true` (+ `code`) pour un résultat
  `status:"error"` en conservant `structuredContent` ; inventaire attendu à 19 outils
  (15 Bridget + 4 Maicie), matrice FR-009 mise à jour (19).
- **wrapper.rs** : `BRIDGET_SAFE_MCP_TOOLS` 14 → 15 et golden Codex mis à jour.
- **Docs minimales** : ligne `thread` dans le tableau 094 de commandes.md (test d'inventaire),
  « quinze outils » dans SKILL.md et commandes.md ; recettes complètes à T024/T025.
- **Tests** : `spec102_v33_bridget_thread_transmet_le_contrat_et_refuse_les_champs_inconnus` ;
  `mcp::tests` + `wrapper::` : 132/132.

### T012–T014 Sollicitations : état coalescé, projection 099, réaffectation
- **Statut** : ✅ Complétés.
- **store/threads.rs** : `thread_wake_candidates` (pending > max(acked, dispatched), fil
  ouvert, aucune remise active non échue ; après issue inconnue échue seule une mention
  strictement supérieure est candidate ; rotation par `last_attempt_at`),
  `thread_wake_reserve` (génération +1, borne/clé/instance/génération 099 figées,
  contrôle de course sur la génération attendue, dernière issue inconnue conservée dans
  `last_uncertain_*`), `thread_wake_defer` (offline/dnd/capability_unavailable/rate_limited),
  `thread_wake_settle` (Dispatched → `dispatched_seq`, Refused, OutcomeUnknown),
  `thread_wakes_expire` (in_flight échue → outcome_unknown), `thread_wake_by_delivery`,
  `thread_wakes_in_flight`.
- **daemon.rs** : `dispatch_thread_wakes` sous verrou (sans E/S fournisseur), appelée après
  chaque post/ack/close réussi et par le tick d'entretien existant (1 s) ; débit 5 départs/s,
  lot 16 ; `project_thread_wake` = scope superviseur + clé `thread-wake:<fil>:<agent>:<gén>`
  + `canonical_send` avec notice + `reserve` (horizon 099 7 jours) + `SendDelivery.expires_at`
  = min(échéance 120 s, expiration de réservation) + `begin_send_delivery` +
  `defer_idempotent_delivery` (poussée hors verrou) ; `recover_thread_wakes_in_flight`
  reprend une réservation figée sans gravure 099 avec la même clé/instance/génération ;
  `settle_thread_wake_by_delivery` corrèle `DeliverAcked` (dispatched) et
  `DeliveryIndeterminate` (outcome_unknown). Message d'alerte : from `bridget`, origine
  System, intention TriggerTurn, reply=false, id `thread-notice:<fil>:<agent>:<gén>`,
  corps neutre, `deadline_at` = échéance d'injection.
- **idempotency/send_delivery.rs** : `reassign_dispatching_deliveries` exclut les enveloppes
  portant `thread_notice` (JSON1 sur l'enveloppe durable, un seul UPDATE transactionnel).
- **Tests** : V04 (20 échanges A↔B, zéro trame chez C/D/A ; générations distinctes ; `all`
  vise B/C/D une fois), V08 (dix mentions → une génération), V09 (enveloppe figée, mention
  concurrente conservée), V10 (absent, retour, DND puis levée), V24 (coupure entre réservation
  et gravure : même delivery_id/génération repris), V25 (issue inconnue, échéance, nouvelle
  mention, accusé tardif refusé), V27 (autre instance : notice jamais réaffectée, capacité
  renégociée). Suite : 20/20.

### T015–T017 Adaptateurs et réponse implicite
- **Statut** : ✅ Complétés.
- **wrapper.rs** : `connect_and_register_at` annonce `ThreadNoticeCapability{[1]}` après
  `Registered` (chemin commun Codex/Claude gérés et interactifs et pont T3) ; le contexte de
  réponse reçoit le marqueur `{"kind":"thread_notice","thread_id":…}` pour une alerte
  (Deliver et DeliverIdempotent), l'ancien format DM reste écrit pour les vrais DM.
- **t3code.rs** : enveloppe dédiée 🧵 (lire/confirmer/publier par `bridget_thread`, aucun
  relais de réponse finale) ; aucune attente `pending` (reply=false, règle 105).
- **cli.rs** : `bridget reply` refuse avec `thread_notice_not_replyable` (JSON, exit 2) quand
  le contexte est un marqueur ; les anciennes lignes `expéditeur[TAB]id` restent lues.
- **Tests** : V28 (`spec102_v28_alerte_de_fil_injectee_sans_attente_ni_relais`), V29 CLI
  (`spec102_v29_marqueur_thread_notice_reconnu…`) et wrapper
  (`spec102_v29_alerte_de_fil_laisse_un_marqueur_type…`).
- **Limite** : la recette des wrappers Codex/Claude réels avec faux fournisseur (V29
  intégration) n'est pas écrite ; la capacité est annoncée par le chemin commun et la remise
  suit le tracker 099 existant.

### T018–T023 US3 : lecture, reçus, confirmation, historique, coupures
- **Statut** : ✅ Complétés (implémentation déjà portée par store/threads.rs et threads.rs
  aux tâches T007/T009 ; ces tâches ajoutent les preuves et corrigent les écarts constatés).
- **Comportements vérifiés** : `read` = reçu actif rejoué à l'identique (même plage, même
  reçu, notice `pending_receipt_replayed`, `requested_limit` d'origine) tant qu'il n'est ni
  confirmé ni échu ; sinon page depuis `acked_seq+1` bornée par nombre (1–200) et par octets
  (48 Kio d'entrées + 12 Kio de réserve ≤ 60 Kio), jamais coupée au milieu d'une entrée ;
  page vide sans reçu ; `ack` = dernier reçu rejouable (`already_acknowledged`), reçu tardif
  encore courant accepté, reçu remplacé `receipt_obsolete`, reçu étranger/autre fil
  `receipt_invalid`, non-membre `thread_unavailable`, avance exactement à `through_seq` ;
  `post` avec `ack_receipt` atomique (refus = ni dépôt ni avance) et rejouable après
  suppression du reçu ; `history` borne figée (`to_seq`), `next_from_seq`, sans reçu ni
  déplacement du repère ; reçu actif retrouvé après redémarrage.
- **Tests** : V11, V12, V13, V14 (Unicode/échappements, pagination par octets, `entry_too_large`
  sur 16 Kio de caractères de contrôle), V15 (deux lectures concurrentes, même reçu), V16
  (matrice ACK), V17, V18, V26. Suite `spec102_threads_test` : 28/28.

### Écarts préexistants corrigés pour une recette propre (hors périmètre 102, tolérance zéro)
- `core_089_skill_test` : l'exemple de réponse de SKILL.md (session 105) dépassait les 60
  caractères affichés par `bridget ledger` → corps raccourci à 59 caractères.
- `claude_interactive_097_test` (6 tests) : le prompt système de la session 105 contenait
  « ; », refusé par le filtre d'arguments du wrapper (`;&|$`) → deux « ; » remplacés par « . »
  et « , » ; texte sinon inchangé.
- `claude_native_permissions_test::spec094_claude_recoit_la_liste_fermee…` : la liste attendue
  n'incluait pas `bridget_events`/`bridget_journal` (sessions 100/101) ; complétée avec
  `bridget_thread`.
- `t3code.rs:2780` : emprunt inutile (clippy) dans la 105.
- **Environnement** : `cli::hook_tests::spawn_roundtrip…` et `cli::spawn_executor_tests::…`
  échouent quand le processus de test descend d'un agent Bridget attesté (la résolution
  d'identité par filiation trouve mon identité T3 dans `~/.cache/bridget-core/agent-pids`
  puis exige une preuve auxiliaire absente du namespace de test) ; ils passent avec
  `BRIDGET_HOME` pointé sur un namespace privé vide. La recette T030 utilise ce réglage.

### T026 Quotas N/N+1 et saturation
- **Statut** : ✅ Complété.
- **Tests** : `spec102_v31_quotas_de_fils_n_et_n_plus_1` (8 créateurs × 32 fils ouverts
  acceptés, 33e refusé, 257e refusé par un neuvième créateur ; la clôture ne libère pas la
  conservation (256 conservés) ; lecture/ACK/clôture toujours possibles ; 100 clôtures rejouées
  avec clés nouvelles laissent 3 opérations pour le fil) ; V31 validation (membres 1 et 17,
  titre 161, corps 16 Kio+1, blanc, reply_to hors plage, limit 201, from_seq 0, to_seq futur) ;
  V14 (entrée indivisible > 48 Kio refusée, pages ≤ 60 Kio) ; V31 concurrence (compteurs).
- **Non testé à l'échelle réelle** : plafonds 10 000 entrées (contrôle unitaire de la
  condition seulement via V35 sous le plafond), 16 Mio par fil et 128 Mio au total (même code de
  contrôle transactionnel, `SUM(body_bytes)` sur ≤ 256 lignes) ; exécution jugée trop coûteuse
  pour la recette locale, consignée comme limite.

### T027 Autorisations, contenu inerte, notice non forgeable
- **Statut** : ✅ Complété.
- **daemon.rs** : `thread_notice` fourni par un client est neutralisé avant le canon dans
  `handle_idempotent_send` et dans le bras `Send` ; seule la projection interne construit une
  notice. Journaux : identifiants, codes et compteurs seulement (aucun corps, titre, reçu).
- **Tests** : V23 (`SendIdempotent` avec notice forgée → remise sans notice, corps inerte,
  autorité de l'expéditeur conservée ; corps hostile stocké tel quel), V21, V22 (rôles), V16
  (reçus étrangers). SC-006 : tous les refus de la recette produisent un code précis sans
  mutation (vérifié par relecture d'historique après chaque refus).

### T028 Parité CLI/MCP et catalogue
- **Statut** : ✅ Complété.
- **Test** : `spec102_v33_parite_cli_mcp` — serveur MCP réel (`bridget mcp`, identité A) et CLI
  réelle (`bridget thread …`, identité B attestée par preuve privée) sur le même daemon isolé :
  `tools/list` expose `bridget_thread` (8 actions, `additionalProperties:false`) ; create MCP,
  post/rejeu CLI, read des deux côtés (mêmes entrées, mêmes champs), ack des deux côtés, list
  identique ; erreurs : `thread_unavailable` en exit 2 côté CLI et `isError:true` + `code`
  côté MCP, drapeaux invalides exit 2, champ inconnu MCP → JSON-RPC -32602 ; close MCP puis
  `thread_closed` CLI ; `bridget reply` refuse `thread_notice_not_replyable` sur marqueur.
- Allowlists : `BRIDGET_SAFE_MCP_TOOLS` (15), golden Codex, liste attendue Claude, inventaire
  094 ; matrice FR-009 à 19 outils.

### T029 Charge locale (SC-005)
- **Statut** : ✅ Complété.
- **Test** : `spec102_v35_charge_10000_entrees` — 9 800 entrées de 256 octets puis 200
  opérations alternées read+ack (50 entrées) / post (1 Kio) avec 8 DM idempotents témoins remis
  pendant la mesure (le disjoncteur 099 borne une paire à 8 échanges / 180 s : les « 20 DM
  témoins » du plan sont réduits à 8 sur une seule paire).
- **Mesure** (poste de recette, build debug, daemon isolé, 2026-09-17) : remplissage 6,8 s ;
  200 opérations p50 = 2,8 ms, p95 = 3,5 ms, max = 4,3 ms ; seuil p95 < 1 s largement tenu.

### T024–T025 Documentation et skill (US4)
- **Statut** : ✅ Complétés.
- **commandes.md** : section « Fils partagés (102) » — contrat et codes, recette à quatre
  participants avec 7 exemples JSON `tools/call` (parsés par un contrôle éphémère), formes CLI,
  lecture/reçus/reprise, recette de synthèse (provenance, plage, désaccords, pas de consensus
  inventé, aucun résumé automatique), alertes/états/limites ; ligne `thread` dans le tableau
  094 ; `bridget_thread` dans la liste des outils ; « quinze outils ».
- **SKILL.md** : description, cinq demandes du quotidien (ouvrir, déposer en silence,
  solliciter, rattraper, résumer), section « Fils partagés : publier, solliciter, lire »
  (choix `[]`/UUID/`all`, publié ≠ injecté ≠ confirmé, conduite à l'alerte, clé de rejeu,
  bornes, compatibilité de catalogue et de capacité), lien vers les recettes. Les exemples JSON
  historiques indexés par `core_089_skill_test` sont inchangés (test vert) ; les nouveaux
  exemples vivent dans la référence, hors du parcours indexé.
- **README.md** : paragraphe 102 après le paragraphe 105.
- **Contrat / audit** : `contracts/thread-api.md` amendé (fait `ThreadNoticeCapability`),
  `reuse-audit.md` : deux arbitrages ajoutés.

### T033 (Convergence) Plafonds réduits et débit borné
- **Statut** : ✅ Complété.
- **Tests** : `store::threads::spec102_quota_tests::spec102_v31_plafonds_d_entrees_et_d_octets_n_puis_n_plus_1`
  (limites injectées : 3 entrées, 10 octets par fil, 15 au total ; N accepté, N+1 refusé par
  code précis, compteurs cohérents, aucune mutation) ;
  `spec102_v10_debit_borne_puis_reprise_au_tick_suivant` (sept membres visés par `all` :
  cinq départs immédiats, deux au tick suivant ≈ 1,45 s, sept alertes au total, une par
  membre, toutes `dispatched`).

## Remise à l'humain (T032)

**Statut de la session** : Implemented (33/33) dans le worktree ; recette T030 verte ; commit,
fusion et déploiement restent à autoriser.

**Ce qui est livré dans le worktree** (diff non commité, base `a3e0786d` = main `f36804ea`) :
- Contrat : `ThreadRequest`/`ThreadAction`/`ThreadNotify`/`ThreadResult`,
  `WrapperToDaemon::ThreadRequest`, `WrapperToDaemon::ThreadNoticeCapability`,
  `DaemonToWrapper::ThreadResult`, `BridgetMessage.thread_notice` (`ThreadNotice`).
- Daemon : `threads.rs` (règles), `store/threads.rs` (six tables, transactions, sollicitations),
  dispatcher de sollicitations dans `daemon.rs` (tick 1 s existant, 5 départs/s, lot 16,
  échéance 120 s, reprise même clé), exclusion des notices de la réaffectation 099,
  neutralisation des notices forgées, préflight du schéma.
- Façades : `bridget thread …` (CLI) et `bridget_thread` (MCP, 15e outil des allowlists
  Codex/Claude) ; `bridget reply` refuse après une alerte.
- Adaptateurs : annonce de capacité après enregistrement (chemin commun), enveloppe T3 dédiée
  sans attente ni relais, marqueur de contexte de réponse.
- Docs : SKILL.md, commandes.md (recettes + JSON), README, ADR 038, contrat amendé.

**Versions et capacités nécessaires à la recette réelle** : daemon, pont T3 et wrappers
issus du même build (le fait `ThreadNoticeCapability` n'est annoncé que par les nouveaux
wrappers ; un serveur MCP déjà vivant garde son ancien catalogue sans `bridget_thread`).
Un membre dont le client est ancien publie et lit mais n'est pas sollicité
(`capability_unavailable`) ; rien n'est dégradé en message direct.

**Commandes de recette** (depuis le worktree, environnement privé) :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/102-fils-inter-agents
umask 077
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
BRIDGET_HOME=/tmp/b102/ns cargo test -p bridget-daemon --lib -- spec102 --test-threads=1
cargo test -p bridget-daemon --test spec102_threads_test
env -i HOME=$HOME TMPDIR=$(mktemp -d /tmp/b.XXXX) BRIDGET_HOME=/tmp/b102/ns \
  PATH=$HOME/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  perl -e 'alarm 1800; exec @ARGV' -- cargo test --workspace --release --no-fail-fast
```

**Adoption éventuelle (non exécutée, à autoriser)** : même procédure que les adoptions
100/101/105 (`~/.cache/bridget-adoptions/<session>-<date>/` : binaire avant, `.backup` de la
base, plists, skill), build isolé en release depuis le worktree, remplacement du binaire
installé, puis `launchctl kickstart -k gui/$UID/com.bridget.daemon` et `com.bridget.t3`.
La migration additive crée six tables au premier démarrage ; retour arrière = binaire
précédent (les tables ignorées ne gênent pas l'ancien daemon). Les sessions T3 déjà
ouvertes gardent leur serveur MCP ancien : `bridget_thread` n'y apparaît qu'après réouverture.

**Ce qui n'est pas fait** : commit, fusion dans main, déploiement ; recette avec agents réels ;
purge/rétention des fils (hors périmètre) ; test à l'échelle des plafonds 16 Mio / 128 Mio.

### T030 Recette complète (état définitif du code, 2026-09-17 22:12–22:18 CEST)
- **Statut** : ✅ Complété.
- **Environnement** : `umask 077`, `env -i HOME TMPDIR=/tmp/b.XXXX BRIDGET_HOME=/tmp/b102/ns
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`, `perl -e 'alarm 1800'`.
- `cargo fmt --all -- --check` : rc 0.
- `cargo clippy --workspace --all-targets -- -D warnings` : rc 0 (aucun avertissement).
- `cargo test --workspace --release --no-fail-fast` : **1460 réussis, 0 échec, 50 ignorés**
  (ignorés : recettes réelles Codex/Claude/T3 conditionnées par variables d'environnement, et
  le worker de performance privé du harnais). Journal : /tmp/b102/recette2-release.log.
- Régressions 089/094/097/098/099/100/101/105 incluses dans le workspace ; les quatre
  écarts préexistants corrigés (skill, prompt 105, allowlist Claude, clippy t3code) font
  partie du vert.
- Une première passe complète (22:04–22:12) avait donné 1458/0/50 avant les deux derniers
  tests de convergence.
- **Passage 3 (22:31–22:40, après la correction issue de la contre-revue)** : fmt rc 0,
  clippy rc 0, `cargo test --workspace --release --no-fail-fast` **1460 réussis, 0 échec,
  50 ignorés** (/tmp/b102/recette3-release.log). État du code figé à ce passage.

### T031 Analyze, Converge et contre-revue adverse
- **Statut** : ✅ Complété.
- Analyze/Converge : analysis.md § « Converge — passage 1 » (table exigence → code → test,
  1 tâche ajoutée T033 puis implémentée ; passage 2 CONVERGED, tasks.md inchangé).
- Contre-revue : Bridget-enhance (Codex) injoignable (deux tours en erreur) ; cursor-listen
  (Cursor) a rendu **APPROVE_WITH_CHANGES** à 22:26 : aucun BLOCKED, 1 correction retenue
  (`thread_post` : absence de ligne de sollicitation après écriture = erreur d'invariant au
  lieu d'un état vide), 1 documentation ajoutée (alerte en vol après lecture = sans objet),
  1 objection non retenue avec justification (rejeu de reçu sans budget d'octets), limites
  de couverture confirmées. Détail : adversarial-review-cursor-listen.md.
- Recette complète relancée après la correction (voir T030 : passage 3).

### T032 Remise
- **Statut** : ✅ Complété — section « Remise à l'humain » ci-dessus, ADR 038 « Accepté »,
  fiche de spec mise à jour. Aucun commit, fusion ni déploiement effectué.
