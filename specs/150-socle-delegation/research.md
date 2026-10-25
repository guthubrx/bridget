# Recherche - Session150 socle de la délégation native

Date : 2026-10-11. Source : lecture du code du worktree
`.worktrees/150-socle-delegation` (base `1f2c661d`), puis contre-revue Codex
(`adversarial-review-codex.md`). Les lignes citées sont celles de cette base.

## R1 - Unité de purge : l'arbre d'une tâche sans parent

- **Décision** : un arbre = une tâche dont `parent_task_id` est vide, plus tous
  ses descendants (`descendants()`, `delegation_lineage.rs:142-145`).
- **Raison** : purger tâche par tâche laisse des `parent_task_id` orphelins. Un
  enfant vivant dont la tâche disparaît deviendrait racine à l'admission
  (`native_delegation.rs:460-472`). L'index `root_page` regroupe par
  propriétaire racine, pas par arbre (`delegation_lineage.rs:102`) : deux
  missions indépendantes d'un même parent ne doivent pas se bloquer.
- **Alternatives** : purge par tâche (rejetée, orphelins) ; purge par
  propriétaire racine (rejetée par la contre-revue, objection 8).

## R2 - Conditions de purge, contrôlées au moment de la suppression

Un arbre est purgeable si chaque tâche :

1. est terminale (`terminal()`, `delegation.rs:60-65`) et a `cleanup_done` ;
2. si elle est `result_available` ou `failed`, sa remise au parent
   (`native-result-<task>` ou `native-failure-<task>`) a été **accusée**, et
   cette preuve est durable dans la tâche (`delivery_acked = true`). Une tâche
   `cancelled` n'attend aucune remise. Une tâche dont l'avis n'est jamais parti
   (`failure_sent = false`, `native_delegation.rs:2110`) n'est pas purgeable.
   - `result_sent` et `failure_sent` ne suffisent pas : ils deviennent vrais
     sur `OutcomeUnknown` (`native_delegation.rs:1059`, `:1231`).
   - L'accusé se lit dans `send_deliveries.phase = 'acked'` pour
     `issuer_scope(task_id)` et `idempotency_key = message.id`
     (`idempotency.rs:527-540`, `native_delegation.rs:747`). La lecture existante
     `send_delivery()` masque les phases autres que `dispatching`
     (`send_delivery.rs:311`) : une lecture explicite de la phase est ajoutée.
   - `purge_expired_sends` efface les remises quelle que soit leur phase
     (`send_delivery.rs:571-579`). L'entretien recopie donc l'accusé dans la
     tâche (`delivery_acked`) pour toutes les tâches en attente, sans lot (total
     borné à 4096), et avant `purge_expired_sends` dans le même passage. Sans
     preuve, l'arbre reste conservé.
3. n'a pas de sous-agent vivant ni relançable :
   - aucune présence connectée au nom `task.child` ;
   - aucun lancement en cours : aucun `managed_spawns` dont `lease.name ==
     task.child` (`daemon.rs:16667`), aucune saga `requested/reserved/starting`
     sous `fleet.supervisor_scope()` (`spawn_commands.rs:313`) ;
   - aucune entrée de flotte désirée `Running`.
   Juste avant la suppression, l'entrée de flotte de chaque enfant est
   décommissionnée (`fleet.decommission`, `fleet.rs:545`). Une entrée `Stopped`
   resterait sinon relançable (`request_relaunch`, `fleet.rs:880`) après la
   purge. Si la décommission échoue, l'arbre est conservé.

Les contrôles 2 et 3 sont refaits sous le verrou du daemon, juste avant la
suppression.

- **Âge** : maximum de `completed_at` des tâches de l'arbre (`updated_at` si
  absent, tâches 148).
- **Conséquence assumée** : une remise indéterminée bloque son arbre. Si ces
  arbres remplissent la borne, l'admission refuse `retention_saturated`.

## R3 - Borne totale : 4096 tâches conservées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research.md#extrait-1).

## R4 - Clé purgée : pierre tombale de 30 jours

- **Décision** : table `native_delegation_tombstones(owner, request_id,
  purged_at, PK(owner, request_id))`, écrite dans la transaction de purge. Le
  rejeu d'une clé présente rend `request_expired`. Les pierres de plus de
  30 jours sont purgées à l'entretien.
- **Raison** : `bridget_delegate` n'a pas d'`issued_at`
  (`protocol.rs:2013-2023`). Le mécanisme `IdempotencyExpired` de l'ADR 007
  (`idempotency.rs:872-876`) n'est donc pas réutilisable sans changer le
  contrat. La grâce de 30 jours reprend `purge_expired_sends`
  (`daemon.rs:7169`).
- **Alternatives** : ajouter `issued_at` au contrat (rejetée, hors périmètre) ;
  pierre sans fin (rejetée, croissance non bornée).

## R5 - Ouverture du magasin

- **Décision** : retirer le refus `records.len() > 4096` de
  `initialize_lineage` (`delegation_lineage.rs:80`). Garder les refus de cycle
  et de profondeur. Ne migrer que les lignes 148 (`root_owner_agent_id` vide).
- **Raison** : l'admission plafonnait déjà à 4096, et la nouvelle borne aussi.
  Une base de plus de 4096 lignes n'existe que par écriture externe. Une
  migration par lots serait du code pour un cas impossible (objection 9
  partiellement retenue : garde retirée, lots rejetés).

## R6 - Admission

- **Décision** : remplacer le chargement complet `tasks()`
  (`native_delegation.rs:441-451`) par des `COUNT` SQL sur l'index d'état :
  128 actives au total, 16 par instance. Puis, si le total atteint 4096, un
  passage de purge borné, puis `retention_saturated` si le total reste à 4096.
- **Complexité** : actives par l'index d'état ; total par `COUNT(*)`, O(n) avec
  n ≤ 4096 (borne de rétention), sans charger les payloads.
- **Écart d'implémentation accepté** : sous pression, la purge de borne à
  l'admission n'est pas limitée à 128 arbres ; elle s'arrête quand le total
  repasse sous 4096. Les accusés sont recopiés juste avant. L'entretien garde
  la limite de 128 arbres.

## R7 - Pause à l'admission

- **Décision** : ajouter `AutonomousEffect::NativeDelegation` à l'enum fermée
  (`referent_control.rs:105-113`) et appeler `admit_autonomous_effect` à
  l'admission, après le contrôle de rejeu (`native_delegation.rs:419`), avant
  `parent_fact`. Refus `referent_paused`, aucune écriture.
- **Raison** : l'ADR 027 impose une garde unique par puits. L'ordre garde le
  test `native148_replay_precedes_grant_revocation_and_definition_change`.
- **Transmis à 152** : lancement, remise de mission et missions en cours
  pendant une pause (objections 1 à 4 et 7).

## R8 - Délai de réponse par tâche

- **Décision** :
  - `Delegate.reply_timeout: Option<u64>`, avec
    `skip_serializing_if = "Option::is_none"`. Les appels 149 gardent les mêmes
    octets canoniques. Nom aligné sur `bridget_send`.
  - Bornes 60 à 86 400 s, défaut 3600. Refus `reply_timeout_out_of_range`.
  - Un délai **explicite** d'enfant est au plus `parent.mission_deadline_at -
    now`, si le parent a une échéance. Refus `reply_timeout_exceeds_parent`.
    Sans délai explicite, le comportement 149 reste intact (3600 s, aucun
    contrôle) : sinon toute délégation imbriquée existante serait refusée
    (contre-revue ronde 2, point 5).
  - Stocké dans la tâche (`reply_timeout`, absent = 3600). Remplace
    `MISSION_REPLY_LIMIT_SECS` aux lignes `:1007`, `:1170`, et le littéral
    `reply_timeout = Some(3600)` (`:1143`) du suivi de demande.
  - `bridget_capabilities` garde `mission_reply_limit_secs` et ajoute
    `mission_reply_min_secs` et `mission_reply_max_secs`. Le test qui fige les
    clés (`native_delegation_permissions149_tests.rs:195`) est mis à jour.
- **Conséquence documentée** : « omis » et « 3600 explicite » sont deux
  enveloppes différentes au rejeu (`envelope_mismatch`).

## R9 - Projection après purge

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/150-socle-delegation/research.md#extrait-2).

## R10 - Points d'appel de la purge

- Pas de purge dans `DaemonState::new` : présences et lancements y sont vides
  et la réconciliation des anciens processus vient après (`daemon.rs:4552-4561`).
  Le premier passage d'entretien a lieu 120 s après le démarrage.
- Entretien horaire : `run_maintenance` (`daemon.rs:7177-7240`), 128 arbres au
  plus par passage (patron `disk_hygiene::MAX_ENTRIES_PER_PASS`).
- Admission : un passage borné si le total atteint 4096.

## R11 - Hors périmètre constaté

- Tables d'exécution, d'idempotence et de flotte liées aux tâches purgées
  (`execution_store.rs:1533-1605`, `spawn_commands`, `agent_links`) : dette
  existante, non traitée.
- Chargements complets `tasks()` au démarrage et dans `cancel_tree` : bornés à
  4096 lignes par la rétention.
