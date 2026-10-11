# Plan - Session159 file retenue par le daemon et édition

**Spec** : `specs/159-edition-file/spec.md`
**Branche** : `session-159-edition-file`
**Worktree** : `.worktrees/150-socle-delegation` (exécution séquentielle, après 156)
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
- Quatre magasins ouvrent le même fichier par des connexions distinctes (`daemon.rs:3058-3071`). Toute transaction d'édition reste dans `IdempotencyStore`.
- Tests : recette `umask 077` de la session 150 (`quickstart.md`).

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
| Porteur ancien | valable sauf pour `promote` (déclare `queue_fallback`) | exiger partout | la rétention n'exige rien du porteur |
| Rang | global, monotone, en base | rang par instance, rang du porteur | une seule copie, pas de collision (R6) |
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

## Étapes d'implémentation

1. Lot A : référence et vérifications (T001, T002) ; fondations (T003 à T005) ; signal de tour véridique (T006) ; libération, balayage, lecture (T007 à T009) ; pannes et mesure (T010) ; répétition de la restauration (T011) ; documentation et clôture (T012, T013).
2. Pause : le lot A se livre, réglage fermé. Ouverture pour `codex`, mesure, décision.
3. Lot B : fondations (T014 à T016) ; retrait (T017, T018) ; déplacement (T019) ; promotion (T020, T021) ; outil, bout en bout, documentation, ADR, clôture (T022 à T027).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 159 prévue (T025) |
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
| Signal de repos perdu | réattestation 30 s ; wrapper muet : sortie forcée de `busy` ≤ 2730 s depuis `busy_period_start` ; SC-005 corrigé |
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

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diffs enregistrés par lot (`validation/diff-159a.patch`, `validation/diff-159b.patch`).
La migration v7 s'applique à la prochaine ouverture de la base, après relance et
sauvegarde décidées par l'utilisateur.
