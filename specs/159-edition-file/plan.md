# Plan - Session159 file retenue par le daemon et édition

**Spec** : `specs/159-edition-file/spec.md`
**Branche** : `session-159-edition-file`
**Worktree prévu A** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159A-edition-file` (exécution séquentielle, après livraison158B)
**Base 159A confirmée** : source 158B e52a96240b6168bf4466712e1bda8852b4ecb43f livré ; HEAD de préparation b235998638b3eadf8f8edc0a6e323b762e95cab0. GO159A acquis par ROOT, implémentation A en cours ; held_queue reste fermé. Le candidat 158B 2124 reste historique ; aucune livraison 159A déduite.
**Date** : 2026-10-11 (conception refondue après contre-revue ronde 1 : 11 points, racine commune ; ronde 2 : 10 points intégrés)

## Résumé

Le daemon retient en base les messages ordinaires destinés à un agent occupé, puis
les libère un par tour. Les éditions deviennent des transactions d'un seul magasin.

- **Lot A (159a)** : phase `held`, rang durable, libération et balayage, lecture
  exacte, migration v7. Réglage `held_queue` par type d'agent, **fermé par défaut**.
- **Lot B (159b)** : `cancel`, `reorder`, `promote` sur les messages retenus, avec
  reçu complet, plus `bridget_queue` et la CLI.
- Hors lots : fils T3 (159c), `edit`.

Critère visé : Guidage 4 vers 5.

## Contexte technique

- Rust 2024, workspace à 3 crates. Aucune nouvelle dépendance.
- SQLite (`bridget.db`), base d'idempotence existante (migrations 2 à 6, la v7 est neuve).
- Quatre magasins ouvrent le même fichier par des connexions distinctes (`daemon.rs:4806-4818`). Toute transaction d'édition reste dans `IdempotencyStore`.
- Tests : recette privée réemployée dans `quickstart.md`, actualisée vers le futur worktree159A ; aucune baseline158B exécutée par cette préparation.

## Décisions architecturales

| Sujet | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Où vit la file | base du daemon (`send_deliveries`, phase `held`) | file mémoire du porteur | une édition devient une transaction, sans saga (R2, R3) |
| Réutiliser `execution_queue` | non | oui | aucun consommateur, autre magasin, pas de lien à la saga (R14) |
| Portée | types d'agents avec `held_queue`, fermé par défaut | tous les agents d'emblée | mesurer avant d'étendre (R14) |
| Fils T3 | hors lot | émettre `TurnState`, déplacer le regroupement | regroupement 114 et file de 64 propres au pont (R14, R16) |
| Libération | réservation de tour durable (`turn_gate`), repos attesté, revérification en transaction | « aucune remise `dispatching` » seule | l'accusé ouvre un trou avant `TurnState true` ; un refus envoie un faux repos (R5) |
| Signal de tour | véridique et réattesté, `turn_attest` | signal actuel | `wrapper.rs:6224` envoie `false` sur un refus ; `busy_since` remis à zéro (R5) |
| Rattachement | `recipient_agent` durable, adoption à l'enregistrement | `recipient_instance_id` seul | présences vides après un redémarrage du daemon (R5) |
| Pause | `HeldRelease` effet autonome différé | libération libre | collision avec la reprise de pause (ADR 027) |
| Déclencheurs | repos attesté, reconnexion, fin d'exécution, levée de pause, tick | fin de tour seule | aucun ne prouve seul le repos : conditions revérifiées |
| Édition | transaction unique + reçu | saga répartie | 11 défauts de la première conception |
| Résultat d'édition | toujours final | `outcome_unknown` | aucun aller-retour avec le porteur |
| Promotion | libération immédiate en guidage + marqueur `steer-or-queue`, garanties réduites | annuler puis renvoyer | pas de perte avant la remise ; file ordinaire pleine et erreur ambiguë de Codex restent des cas déclarés (R9, R17) |
| Porteur ancien | chemin actuel sans rétention faute de `turn_attest` ; `promote` du lot B exige `queue_fallback` | exiger partout | la rétention exige le signal véridique déclaré ; les capacités anciennes restent fermées |
| Rang | queue_rank INTEGER PRIMARY KEY AUTOINCREMENT ; delivery_id UNIQUE NOT NULL | rang recyclé après purge ou compteur métier parallèle | FR008 durable ; INSERT sans rang, anciens rowid copiés ; surcoût SQLite justifié (R6) |
| Migration | v7 à sens unique + précontrôle relevé + sauvegarde | double écriture | `CHECK` de phase ; échec fermé voulu (R6) |
| Reçu d'édition | `queue_edits`, même transaction que l'effet | table de réservation | effet et reçu atomiques |
| `edit` | reporté | implémenter | enveloppe idempotente, attribution (R15) |
| Outil | `bridget_queue` + `action` | outils séparés | un outil, déjà créé en 156 |

## Fichiers touchés

### Lot A

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

Aucun nouveau module Rust. Aucun changement du pont T3 (`t3code.rs`). Aucune
trame daemon vers porteur.

Précisions T006 : La minuterie Instant du heartbeat existant est vérifiée à chaque passage de la boucle, même sous trafic continu. Elle déclenche une réattestation vraie périodique (30 s au plus), sans nouveau scheduler. Pour toute instance turn_attest, la sortie forcée utilise busy_period_start, même si held_queue est fermé. Les instances anciennes non attestées gardent le calcul legacy sur busy_since. La borne par défaut de 2730 s vaut notify_timeout_secs = 2700 s + grâce de 30 s ; notify_timeout_secs reste configurable.

Lecture et reprise : T009 list et dispatching_deliveries_for_instance trient tous deux par queue_rank. Le même index non unique idx_send_deliveries_recipient_phase est étendu à (recipient_instance_id, phase, queue_rank), sans second index recipient redondant.

## Étapes d'implémentation

1. Lot A : référence et vérifications (T001, T002) ; fondations (T003 à T005) ; signal de tour véridique (T006) ; libération, balayage, lecture (T007 à T009) ; pannes et mesure (T010) ; répétition de la restauration (T011) ; documentation et clôture (T012, T013).
2. Le lot A se livre avec `held_queue` fermé. Une éventuelle ouverture pour `codex` attend les mesuresSC004/SC005 et leur décision ; elle ne fait pas partie de la livraisonA fermée.
3. Lot B : fondations (T014 à T016) ; retrait (T017, T018) ; déplacement (T019) ; promotion (T020, T021) ; outil, bout en bout, documentation, ADR, clôture (T022 à T027).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | Prévu | ADR159 initiale A dans T012 avant livraison ; extension B du même fichier dans T025 : /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/159-file-retenue-et-edition.md |
| VIII Forensic | OK | défauts prouvés par lecture ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; tâche cochée après test vert |
| XVI Worktree | OK | worktree partagé |
| XVIII Complexité | OK | R13 |
| XIX Minimalisme | OK | lot A fermé par défaut, deux lots, `edit` et T3 reportés, pas de saga d'édition, pas de nouveau module |
| XX Vertus LLM | OK | chaque décision a sa preuve `fichier:ligne` ; coûts listés |
| XXII Optimisation | OK | latence mesurée avant et après (T001, T010), pas supposée |

### Minimalisme (Article XIX)

- Réutilisés : `send_deliveries` et sa saga, `schedule_idempotent_delivery_recovery`, `defer_idempotent_delivery`, `orphan_emitter_notices`, `release_stale_busy_turns`, `TurnState`, migration par reconstruction (v4), `guidance.rs` et `bridget_queue` (156), `test_sync::checkpoint`.
- Créés : une phase `held`, une phase `cancelled`, une colonne `queue_rank`, un réglage de registre, une table `queue_edits`, deux trames client, un champ optionnel de `Register`.
- Écartés : tombe durable chez le porteur, saga d'édition, snapshot du porteur, trames daemon vers porteur, `execution_queue`, `edit`, extension aux fils T3.

Potentiel minimalisme : environ 60 lignes si le lot B est limité à `cancel` (`reorder` et `promote` sont isolés en Phases 7 et 8).

### Responsabilité future (Article XX)

- Non vérifiable par lecture : latence réelle de la libération, fiabilité de `TurnState` de chaque porteur (T002(1), T010).
- La migration v7 est à sens unique : essayer sur copie, sauvegarder avant la première ouverture (T011, ADR).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | un envoi vers un agent `busy` est poussé tout de suite ; `bridget_queue` refuse les actions d'écriture ; `DROP TABLE` supprime les liens d'exécution ; le précontrôle refuse la version 7 |
| Migration | `held159_migration_*` |
| Rétention et libération | `held159_hold_*`, `held159_rank_*`, `held159_release_*`, `held159_sweep_*`, `held159_list_*` |
| Pannes | `held159_crash_*`, `edit159_cancel_*` avec `test_sync::checkpoint` |
| Édition | `edit159_receipt_store_*`, `edit159_rights_*`, `edit159_cancel_*`, `edit159_reorder_*`, `edit159_promote_*` |
| Transports | `edit159_promote_transport_*` |
| Bout en bout | `held159_e2e.rs`, `edit159_e2e.rs` |
| Mesure | latence fin de tour vers début du message suivant, avant et après |
| Non-régression | `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`, transport |
| Suite complète | recette `--workspace --no-fail-fast` |

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

Livraison par ROOT dans le pipeline152–160 déjà autorisé : commit, intégration,
push, build signé, installation et activation conditionnelle de Bridget, puis
nettoyage. T011/SC006 reste préalable : sauvegarde cohérente et restauration
isolée avec le vrai ancien binaire158B installé, empreinté et conservé avant
reconstruction159. Contrôler zéro mission native non terminale avant activation.
T3 reste sans relance explicite. `held_queue` reste fermé.
Diffs enregistrés par lot (`validation/diff-159a.patch`, `validation/diff-159b.patch`).
La migrationv7 s'applique dès l'ouverture du nouveau binaire, même réglage fermé ;
aucune restauration ou activation n'est réputée acquise par cette préparation.

Rangs A/B : INSERT sans queue_rank ; AUTOINCREMENT conserve la monotonie après purge. Les permutations B gardent les valeurs existantes, avec des rangs provisoires négatifs uniquement dans leur transaction ; aucun CHECK positif incompatible. Coût d’écriture lié aux index et à SQLite, pas de mesure de latence supposée. Références : [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html), [SQLite Foreign Keys](https://www.sqlite.org/foreignkeys.html).

Garde de libération (décision ROOT) : conserver le mutex State de la lecture du repos jusqu’au commit de Held vers Dispatching. La même transaction Immediate d’IdempotencyStore lit control_state et control_pause_interruptions dans config.db_path, partagé par les magasins existants. Préserver l’ordre de verrous existant ; aucun writer socket ni autre connexion SQL dans cette transaction. La libération est linéarisée au commit. Le push différé se fait hors verrou ; une pause validée après ce commit ne reprend pas une remise déjà libérée. Pas de table miroir. Les oracles concurrents de T007 déjà prévus couvrent l’ordre pause/TurnState true ; aucun résultat runtime nouveau n’est annoncé.

Portée de SC001 : égalité des octets canoniques du message et de la remise, de la phase et de la poussée du chemin d’envoi avec held_queue fermé. FR023 exige séparément le flux auxiliaire nouveau de réattestation vraie ; cette exigence ne promet pas l’identité des octets de tout le flux auxiliaire. busy_since legacy reste réservé aux instances sans turn_attest. L’exigence de suites vertes et l’interdiction de nouvelle régression sont conservées, sans dérogation implicite.

Les lectures control_state et control_pause_interruptions sont exécutées après BEGIN IMMEDIATE, dans cette même transaction. En cas de SQLITE_BUSY, la remise reste held : aucun refus, aucune libération ni push ; le prochain déclencheur réessaie. L’attente SQL peut durer busy_timeout (2 s) sous State ; ce risque de latence sera mesuré, sans nouvelle garantie. Les chemins TurnState et pause prennent ce même mutex ; les cas concurrents T007 prévus vérifient l’ordre.
