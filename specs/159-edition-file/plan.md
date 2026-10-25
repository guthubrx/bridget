# Plan - Session159 file retenue par le daemon et édition

**Spec** : `specs/159-edition-file/spec.md`
**Branche B** : `session-159B-edition-file`
**Worktree B utilisé (historique après nettoyage)** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file` ; le WT A antérieur est historique et retiré après livraison.
**Lot A livré** : 14/14 tâches closes ; source `53863bd09a4ff364dc2b412519b343c77e163685`, métadonnées `b0a165edfc6fc5476b2330193c7822e0faecef33`. Le lot B part de cette base livrée ; `held_queue` reste fermé. Les bases 158B et les diagnostics avant159A restent historiques.
**Date** : 2026-10-11 (conception refondue après contre-revue ronde 1 : 11 points, racine commune ; ronde 2 : 10 points intégrés)

## Résumé

Le daemon retient en base les messages ordinaires destinés à un agent occupé, puis
les libère un par tour. Les éditions deviennent des transactions d'un seul magasin.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-1).

Critère visé : Guidage 4 vers 5.

## Contexte technique

- Rust 2024, workspace à 3 crates. Aucune nouvelle dépendance.
- SQLite (`bridget.db`), base d’idempotence v7 déjà livrée par159A. Le lotB crée `queue_edits` et ses index par `CREATE IF NOT EXISTS`, sansv8, reconstruction ni nouvelle restauration de migration.
- Quatre magasins ouvrent le même fichier par des connexions distinctes (`daemon.rs:4806-4818`). Toute transaction d'édition reste dans `IdempotencyStore`.
- Tests : recette privée réemployée dans `quickstart.md`, commandes reproductibles depuis la racine canonique après intégration ; les mesures/bases initiales restent des checkpoints historiques.

## Décisions architecturales

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-2).

## Fichiers touchés

### Lot A livré — changements historiques

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/registry.rs` | champ `held_queue` (`#[serde(default)]`) |
| `crates/bridget-daemon/src/wrapper.rs`, `crates/bridget-transport/src/protocol.rs` | `TurnState` véridique et réattesté, champ `turn_attest` de `Register` |
| `crates/bridget-daemon/src/referent_control.rs` | `AutonomousEffect::HeldRelease` |
| `crates/bridget-daemon/src/store_schema.rs` | `MAX_IDEMPOTENCY_SCHEMA_VERSION = 7` |
| `crates/bridget-daemon/src/idempotency.rs` | migration v7 (reconstruction, rang, liens) |
| `crates/bridget-daemon/src/idempotency/send_delivery.rs` | mode `Held`, rang atomique, `recipient_agent`, `turn_gate`, `release_next_held`, `adopt_held_for_agent`, `settle_held`, réaffectation et orphelinage étendus, lecture par rang |
| `crates/bridget-daemon/src/daemon.rs` | décision de rétention, présence (`busy_period_start`, `last_attested_at`), libération et ses déclencheurs, adoption à l'enregistrement, balayage, suivi mémoire (`held_ids`, `pending_replies`), `QueueList` étendu |
| `crates/bridget-daemon/src/mcp.rs` | champs de sortie de `bridget_queue list` |
| `crates/bridget-daemon/tests/held159_e2e.rs` (nouveau) | pannes, ordre, latence |
| `docs/reference-communication.md`, `skills/bridget/*` | réglage, états, avertissement de sens unique |

### Lot B

| Fichier | Changement |
|---|---|
| `crates/bridget-daemon/src/idempotency.rs`, `idempotency/send_delivery.rs` | `queue_edits`, `apply_cancel`, `apply_reorder`, `apply_promote` |
| `crates/bridget-daemon/src/guidance.rs` (156) | trois actions, faits, refus |
| `crates/bridget-transport/src/protocol.rs` | `QueueEditV1`, `QueueEditRequest`/`Result`, `Register.queue_fallback` |
| `crates/bridget-transport/src/codex_app_server.rs`, `claude_stream_json.rs`, `acp.rs`, `crates/bridget-daemon/src/wrapper.rs` | marqueur `steer-or-queue`, déclaration `queue_fallback` |
| `crates/bridget-daemon/src/daemon.rs`, `mcp.rs`, `cli.rs` | traitement, outil, CLI |
| `crates/bridget-daemon/tests/edit159_e2e.rs` (nouveau) | bout en bout |
| `docs/`, `skills/bridget/*`, `docs/decisions/159-file-retenue-et-edition.md` | documentation et ADR |

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-3).

Précisions T006 : La minuterie Instant du heartbeat existant est vérifiée à chaque passage de la boucle, même sous trafic continu. Elle déclenche une réattestation vraie périodique (30 s au plus), sans nouveau scheduler. Pour toute instance turn_attest, la sortie forcée utilise busy_period_start, même si held_queue est fermé. Les instances anciennes non attestées gardent le calcul legacy sur busy_since. La borne par défaut de 2730 s vaut notify_timeout_secs = 2700 s + grâce de 30 s ; notify_timeout_secs reste configurable. La sortie forcée exige aussi plus de 90 s de silence depuis last_attested_at ; une attestation fraîche maintient busy selon FR025/SC005.

Lecture et reprise : T009 list et dispatching_deliveries_for_instance trient tous deux par queue_rank. Le même index non unique idx_send_deliveries_recipient_phase est étendu à (recipient_instance_id, phase, queue_rank), sans second index recipient redondant.

## Étapes d'implémentation

1. Lot A livré 14/14, source53863bd : référence et vérifications (T001, T002) ; fondations (T003 à T005) ; signal de tour véridique (T006) ; libération, balayage, lecture (T007 à T009) ; pannes et mesure (T010) ; répétition de la restauration (T011) ; documentation et clôture (T012, T013).
2. Le lot A est livré avec `held_queue` fermé. Une éventuelle ouverture pour `codex` attend les mesuresSC004/SC005 et leur décision ; elle ne fait pas partie de la livraisonA fermée.
3. Lot B : fondations (T014 à T016) ; retrait (T017, T018) ; déplacement (T019) ; promotion (T020, T021) ; outil, bout en bout, documentation, ADR, clôture (T022 à T027).

## Contrôle constitution

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-4).

### Minimalisme (Article XIX)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-5).

Hypothèse historique : limiter B à cancel changeait le besoin validé, ce n’était pas une suppression à comportement constant. T027 final : 0 ligne suppressible démontrée sur ses 197 hunks lus ; pas un minimum théorique.

### Responsabilité future (Article XX)

- Non vérifiable par lecture : latence réelle de la libération, fiabilité de `TurnState` de chaque porteur (T002(1), T010).
- La migration v7 à sens unique et sa restaurationT011 appartiennent au lotA livré. Le lotB ajoute seulement `queue_edits` par `CREATE IF NOT EXISTS`, sans nouvelle version ; aucune répétition fictive de migration n’est promise.

## Stratégie de test

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-6).

## Risques

| Risque | Parade |
|---|---|
| Faux repos (refus de remise) : second message libéré en plein tour | signal véridique, réservation de tour durable, tests de refus de guidage et de déclencheurs concurrents (T006, T007, SC-012) |
| Signal de repos perdu | réattestation périodique sous trafic continu ; sortie forcée des instances `turn_attest` depuis `busy_period_start`, délai configurable `notify_timeout_secs + 30 s` (2730 s par défaut) ; `busy_since` legacy conservé |
| Redémarrage du daemon puis du porteur : retenus introuvables | `recipient_agent` et adoption par agent (T004, SC-015) |
| Libération pendant une pause | `HeldRelease` différé ; reprise de pause d'abord (T007, SC-013) |
| Demande close entre le balayage et la libération | revérification dans la transaction (T007, SC-014) ; suivi mémoire retiré (T008) |
| Perte après libération (`Seen` avant injection) | déclarée, testée (T010) ; garantie du lot A limitée aux messages retenus (R17) |
| Restauration de sauvegarde non éprouvée | répétition isolée avec l'ancien binaire avant livraison (T011) |
| Latence ajoutée à la libération | mesurée avant et après (T001, T010), décision sur le réglage par défaut |
| Migration v7 supprime les liens d'exécution | copie avant `DROP`, réinsertion, test, `foreign_key_check` |
| Migration sans retour | sauvegarde obligatoire, précontrôle relevé en même temps, ADR |
| Régression du chemin de remise par défaut | réglage fermé par défaut ; test d'égalité d'octets (SC-001) |
| Expiration d'une demande pendant la rétention | balayage de parité avec les causes d'écart du pont (T007) |
| Ordre cassé par une réaffectation ou une reconnexion | rang global, reprise triée par rang, `held` inclus dans la réaffectation (T004) |
| Message libéré par erreur pendant une édition | un seul magasin, transactions `Immediate` |
| Perte ou doublon à la promotion après libération | garanties réduites et déclarées : file ordinaire pleine, erreur ambiguë de Codex ; tests (T020, T021) |
| Courses de profondeur du porteur (deux messages libérés) | le second est engagé, non éditable, honnête ; pas de perte |
| Conflit d'édition avec 157 et 158 | exécution séquentielle, diff par lot |

## Livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/159-edition-file/plan.md#extrait-7).

Rangs A/B : INSERT sans queue_rank ; AUTOINCREMENT conserve la monotonie après purge. Les permutations B gardent les valeurs existantes, avec des rangs provisoires négatifs uniquement dans leur transaction ; aucun CHECK positif incompatible. Coût d’écriture lié aux index et à SQLite, pas de mesure de latence supposée. Références : [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html), [SQLite Foreign Keys](https://www.sqlite.org/foreignkeys.html).

Garde de libération (décision ROOT) : conserver le mutex State de la lecture du repos jusqu’au commit de Held vers Dispatching. La même transaction Immediate d’IdempotencyStore lit control_state et control_pause_interruptions dans config.db_path, partagé par les magasins existants. Préserver l’ordre de verrous existant ; aucun writer socket ni autre connexion SQL dans cette transaction. La libération est linéarisée au commit. Le push différé se fait hors verrou ; une pause validée après ce commit ne reprend pas une remise déjà libérée. Pas de table miroir. Les oracles concurrents de T007 déjà prévus couvrent l’ordre pause/TurnState true ; aucun résultat runtime nouveau n’est annoncé.

Portée de SC001 : égalité des octets canoniques du message et de la remise, de la phase et de la poussée du chemin d’envoi avec held_queue fermé. FR023 exige séparément le flux auxiliaire nouveau de réattestation vraie ; cette exigence ne promet pas l’identité des octets de tout le flux auxiliaire. busy_since legacy reste réservé aux instances sans turn_attest. L’exigence de suites vertes et l’interdiction de nouvelle régression sont conservées, sans dérogation implicite.

Les lectures control_state et control_pause_interruptions sont exécutées après BEGIN IMMEDIATE, dans cette même transaction. En cas de SQLITE_BUSY, la remise reste held : aucun refus, aucune libération ni push ; le prochain déclencheur réessaie. L’attente SQL peut durer busy_timeout (2 s) sous State ; ce risque de latence sera mesuré, sans nouvelle garantie. Les chemins TurnState et pause prennent ce même mutex ; les cas concurrents T007 prévus vérifient l’ordre.

## Précisions d’exécution du lotB après revueR1

T020 route la copie portant `steer-or-queue` avant les refus actuels de `SteerCurrent`. Si cette copie doit se replier, il pose `steering_requested = false` ET `interrupt_requested = false` avant les earlyrejects ; un message humain avec tour actif ne doit pas devenir un interrupt. Le test de saturation porte alors sur `queue.messages`, jamais sur `queue.steer`. Sur Codex, interface interactive, absence de tour actif ou `steering_open` fermé conduisent à la FIFO ordinaire pour cette seule copie marquée. Claude et ACP réemploient leur même FIFO. Aucun interrupt implicite. La saturation garde son rejet explicite. Une remise sans ce marqueur garde le comportement actuel. Les cas prescrits de T020 couvrent ces branches sans nouveau nom de test.

`apply_promote` garde State de la lecture des faits du tour jusqu’au commit. Après `BEGIN IMMEDIATE`, sa propre connexion relit `control_state`, `control_pause_interruptions`, la réservation, la saga et la demande. Les droits `Steer` de156 refusent un agent pendant la pause ; l’identité humaine attachée attestée conserve l’exception. La reprise en attente bloque les libérations automatiques `HeldRelease`, pas la commande humaine explicite autorisée. Ordre : State et transaction, commit, relâchement de State, puis writer/push. Aucun writer de socket ou autre connexion SQL n’est appelé sous cette transaction. Une pause validée après le commit ne reprend pas l’effet déjà validé.

Le reçu `applied` prouve le commit d’édition, pas l’injection ni la consommation. Un rejet ou `indeterminate` après promotion n’a pas d’avis universel pour `reply=false`. Les avis existants couvrent les chemins attach et demande suivie. `orphan_emitter_notices` sert au retrait/à l’orphelinage, pas à tout rejet du fournisseur. Aucun nouvel avis durable ni nouvel outil de consultation n’est ajouté. T020 et T021 conservent les garanties réduites déclarées.

### Ancres fraîches du codeA pour préparer159B

Base livrée53863bd, métadonnéesb0a165ed. Les anciens numéros T002/T006 décrivent le diagnostic avant le lotA. Ils ne décrivent pas un défaut encore présent.

| Responsabilité | Source actuelle |
|---|---|
| Enregistrement et adoption | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:8106 ; adoption8472 après identité établie |
| Envoi et rétention | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:11423 ; should_hold11719 ; mémoire après commit11905 |
| Relances et filtre Held | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:8953 ; palier3 avant filtre9089 |
| Balayage partagé | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:9362 |
| Libération ordinaire | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:6093 |
| Liste durable et droits | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/daemon.rs:11111 |

Lecture indépendante ciblée : /Users/moi/.cache/bridget-anchors159B-20261012/fresh-anchors159B.md. Sources wrapper495b706b et daemonabf654 stables, corps et limites dans le manifeste associé. Cette relocalisation ne vaut pas un nouveau test exécuté.

Précision T014 : le DDL `queue_edits` et ses deux index se trouvent dans `IdempotencyStore::init_schema`, réemployé à chaque `open`/`in_memory`. Il n’est pas enfermé dans la branche de migration versv7. Une basev7 du lotA sans `queue_edits` doit créer la table à la réouverture puis accepter sa première édition. Les cinq noms existants `edit159_receipt_store_*` couvrent cette fixture initiale et le rejeu sansv8 ; aucun nouveau scénario ou nom.

Signal terminal déjà livré : en mode `automatic_reply`, la branche terminale de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file/crates/bridget-daemon/src/wrapper.rs:6635` envoie `current_turn_in_progress.unwrap_or(false)`, comme le bras de refus6723. Ce n’est pas un false constant. Cette lecture précise le codeA actuel ; elle ne déclenche pas un nouveau runT006.
