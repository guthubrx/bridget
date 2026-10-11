# Recherche - Session152 reprise des missions natives

Date : 2026-10-11. Source : lecture du checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`), de la contre-revue
Codex de la 150 (`specs/150-socle-delegation/adversarial-review-codex.md`,
objections 1 à 4 et 7 de la ronde 1) et de T3 (`RestartContinuation.ts`,
`EffectWorker.ts`, via `research-t3-bridget.md` A3 et A4). Les lignes citées sont
celles de cette base. `N` = `crates/bridget-daemon/src/daemon/native_delegation.rs`,
`D` = `crates/bridget-daemon/src/daemon.rs`, `E` = `execution_store.rs`.

## R1 - Reproduire d'abord : deux chemins d'échec sous pause

- **Constat** : la pause appelle `interrupt_executions_for_pause` (`D:9626-9673`).
  Elle écrit une ligne `control_pause_interruptions` puis émet `Interrupt`. Le
  wrapper répond par un terminal `Cancelled`. Il publie `interrupted`
  (`wrapper.rs:6182`) puis envoie `DeliveryRejected "tour annulé par le pilote"`
  (`wrapper.rs:6202-6207`).
- **Chemin A** : `D:18078-18081` appelle `native_delegation::rejected`. Cette
  fonction ne regarde que la tâche, pas la cause (`N:810-828`). Elle force
  `failed` avec la raison reçue. C'est l'objection 1.
- **Chemin B** : `tick` lit `execution-{mission}`. Si l'état vaut `failed`,
  `unreachable` ou `interrupted`, la tâche passe `failed` (`N:1193-1203`). C'est
  la lecture de l'état `interrupted`.
- **Décision** : les deux chemins sont reproduits par des tests rouges avant tout
  code (T003). Le correctif distingue l'interruption de pause de toute autre
  cause par la ligne de pause (`E:1488-1505`).
- **Raison** : la ligne de pause est la preuve durable déjà écrite avant
  l'interruption. Aucun nouveau signal n'est nécessaire.
- **Alternatives** : filtrer sur le texte de la raison seul (rejeté : fragile, le
  texte vient du wrapper) ; ajouter un état `paused` (rejeté : change le contrat
  Lineage 149).

## R2 - Ordre des événements : `TurnState(false)` avant `interrupted`

- **Constat** : dans `TurnFinished`, le wrapper envoie `TurnState(false)`
  (`wrapper.rs:6141`) avant `publish_execution_transition(... "interrupted")`
  (`:6182`). Le daemon voit donc l'agent libre pendant que l'exécution est encore
  `interrupting`. `resume_executions_after_pause` ne demande qu'une présence non
  `busy` (`D:9675-9703`). `reconstruct_active_for_agent` accepte `interrupting`
  (`E:1396`) et la ferme en `unreachable` avec la raison `daemon_restart`. La
  reprise peut donc précéder le terminal de l'interruption (objection 2).
- **Décision** : pour une mission native, la reprise exige l'état `interrupted`.
  `tick` la pilote chaque seconde et rappelle `schedule_execution_recovery`
  quand l'état est prêt. L'événement de levée seul ne suffit plus.
- **Raison** : `tick` tourne déjà chaque seconde (`D:4694-4705`). Aucun
  mécanisme d'attente nouveau.
- **Alternatives** : retarder `TurnState(false)` dans le wrapper (rejeté :
  touche tous les agents, hors périmètre) ; attendre par minuterie (rejeté :
  non déterministe).

## R3 - Garde de reprise et chaîne d'exécutions

- **Constat** : la garde `D:4251-4258` compare `candidates.first()` à
  `execution-{task.mission}` et sort. Après une reprise, l'exécution active est
  `execution-recovery-<uuid>` (`D:4343`), reliée à la précédente par
  `execution_continuations` (`E:1423-1433`). Quatre endroits ne connaissent que
  `execution-{mission}` : la garde, `tick` (`N:1186`), `close_lost_execution`
  (`N:900-905`) et `descendants_busy` (`N:1349-1368`). C'est l'objection 3.
- **Décision** : une fonction `latest_in_chain(racine)` dans `execution_store.rs`
  suit les continuations avec le CTE récursif du budget (`E:818`). Les quatre
  endroits l'utilisent. `resume_execution_id` (`E:1422`) et la continuation sont
  écrits dans la même transaction. Un test vérifie qu'ils désignent la même
  exécution. Un seul parcours couvre la panne et la pause.
- **Alternatives** : lire seulement `resume_execution_id` (rejeté : absent pour
  une reprise après panne, qui n'a pas de ligne de pause) ; stocker l'id courant
  dans la tâche (rejeté : état redondant, calculable).

## R4 - Horloges : geler pendant la pause, pas décaler après

- **Constat** : `tick` teste l'échéance avant l'interruption (`N:1013-1023`).
  Décaler l'échéance à la levée est trop tard : la tâche aurait déjà échoué
  (objection 4). Le suivi de demande est séparé : `PendingReply` a son
  `Instant` de création et ses paliers (`D:6828-7012`). `collect_reminder_actions`
  diffère les paliers 1 et 2 en pause mais émet le palier 3 (`D:6853-6871`,
  `:6900-6916`). Une demande reconstruite après reconnexion recalcule son temps
  écoulé depuis `tracked_requests.created_at` (`D:3860-3866`).
- **Décision** :
  1. Garde de pause en tête de la boucle de `tick`, avant l'échéance. Elle pose
     `suspended_at` (persisté) sur les tâches non terminales d'états
     `queued`, `starting`, `mission_pending`, `working`.
  2. À la levée, en une seule sauvegarde : `request_frozen_secs += pause` (si la
     tâche est `working`), `mission_deadline_at += pause` (si présent),
     `suspended_at = None`. Un rejeu après crash ne double pas le décalage, car
     `suspended_at` est vidé dans la même écriture. Correction R15 : seule la
     pause qui chevauche la demande est comptée.
  3. Suivi de demande : un champ `frozen_secs` dans `PendingReply`. Il est
     affecté (jamais incrémenté) depuis `request_frozen_secs` à chaque tick et à
     la reconstruction. Le temps écoulé vaut `now - created_at - frozen_secs`. Les
     demandes d'une tâche suspendue sont ignorées par `collect_reminder_actions`.
  4. La table `tracked_requests` n'est pas modifiée. Son index sur
     `deadline_at` n'est lu par aucun balayage (`store.rs:177`, seule
     occurrence). Ce point est prouvé par T005.
- **Raison** : l'affectation absolue est idempotente. Un décalage relatif ne
  survit pas à un rejeu.
- **Alternatives** : réécrire `tracked_requests.deadline_at` (rejeté : double
  écriture non atomique avec la tâche) ; suspendre la demande en la marquant
  annulée (rejeté : perd l'historique des relances).

## R5 - Enveloppe de lancement figée

- **Constat** : `spawn_task` construit l'ordre avec `issued_at = task.created_at`
  et `deadline_at = created_at + 300` (`N:566-570`). Ces deux valeurs entrent
  dans les octets canoniques (`fleet.rs:1662-1696`). Rejouer la même clé avec
  d'autres valeurs donne `spawn_envelope_mismatch`. Le flotte refuse un ordre dont
  `now >= deadline_at` (`fleet.rs:961`) et expire une saga au même seuil
  (`fleet.rs:1111`, `:1436`). Après une pause de plus de 300 s, une tâche `queued`
  ne peut plus se lancer (objection 7).
- **Décision** : champ `launch_issued_at: Option<i64>`. L'ordre utilise
  `launch_issued_at.unwrap_or(created_at)`. À la levée, seule une tâche `queued`
  que la garde de pause a retenue, et sans saga de lancement existante pour
  `command_id = task_id` (`spawn_commands.rs:313`), reçoit
  `launch_issued_at = now`. Une saga existante n'est jamais modifiée.
- **Limite documentée (FR-012)** : une saga déjà en cours au début de la pause
  garde son seuil de 300 s. Le démarrage d'un processus fournisseur est un travail
  de la machine, pas de l'agent. La pause ne l'arrête pas.
- **Alternatives** : changer le canon pour retirer `deadline_at` (rejeté : touche
  toute la flotte et les clés déjà écrites) ; nouveau `command_id` au lancement
  (rejeté : casse `restart_identities`, `N:830-875`).

## R6 - Reprise après panne : retirer `failed`, réutiliser la reprise de flotte

- **Constat** : `prepare_restart` passe `starting|mission_pending|working` à
  `failed/unreachable`, clôt l'exécution, puis arrête l'entrée de flotte
  (`N:922-952`). `reserve_managed_recoveries` saute les enfants natifs
  (`D:3892`, `:3914-3916`). `prepare_recovery` rend un `PreparedSpawn` dont
  l'environnement peut être complété (`lifecycle.rs:313-339`), comme le fait
  `spawn_task` pour la mission et la politique (`N:585-603`).
- **Décision** :
  1. `prepare_restart` garde la tâche et l'exécution quand : identité vérifiée
     (`restart_identities`, `N:830-875`), échéance non dépassée,
     `resume_attempts < 2`. Sinon, échec nommé (FR-016).
  2. Ces enfants sortent de l'ensemble `native_children` et passent par
     `prepare_recovery`. Une fonction `native_child_env` est extraite de
     `spawn_task` pour compléter l'environnement. Elle a deux usages :
     `spawn_task` et la reprise.
  3. Quand le wrapper se reconnecte, `schedule_execution_recovery` reconstruit
     l'exécution avec le message d'origine (`E:1349-1360`, même `message.id`
     donc même identifiant de mission). Le compteur est dérivé de la chaîne,
     sans écriture séparée (correction R14). Correction R13 : cette étape ne
     couvre que le chemin d'un enfant en cours de lancement ; le chemin d'un
     enfant connecté est décrit en R13.
  4. Une tâche `starting` ou `mission_pending` n'a rien reçu : le tick suivant
     rejoue le lancement ou la remise de façon idempotente (`N:1133-1190`).
- **À prouver (H4, T004)** : le bootstrap natif lie `instance_id` au bail
  (`wrapper.rs:470-490`, `:3988-3990`). Le test rouge dit si l'instance du bail
  de reprise reste la même. Sinon, `native_child_env` réécrit le bootstrap avec le
  nouvel `instance_id`.
- **Alternatives** : relancer par `submit_relaunch_from_resolved`
  (`lifecycle.rs:278`) en plus (rejeté tant que H4 ne l'exige pas) ; rejouer la
  mission dans un nouveau fil enfant (rejeté : change l'identité de mission).
- **Borne** : 2 reprises, parce qu'une mission qui tue le daemon deux fois de suite
  est suspecte. La valeur est la question 2 de la spec.

## R7 - Annulation en cascade vérifiée d'abord (T3 A3)

- **Constat** : `cancel_tree` collecte les noms (tâches, liens de flotte) puis
  appelle `prepare_managed_stop` et `await_managed_stop` pour chacun, en ordre
  inverse (`N:1380-1486`). `prepare_managed_stop` a des effets de bord :
  `invalidate_for_stop`, `mark_stopped`, ajout à `managed_terminal_instances`
  (`D:9255-9318`). Il n'existe donc pas de vérification préalable. Un membre
  `NotManaged` laisse la moitié de l'arbre arrêtée. Le nouvel essai rejoue tout
  (`NotFound` compte comme succès, donc sans danger mais sans économie).
- **Décision** : extraire de `prepare_managed_stop` une classification pure
  `classify_stop(st, name)` : supervisé, marqueur, déjà arrêté, introuvable, non
  géré. `prepare_managed_stop` l'utilise. `cancel_tree` classe tout l'arbre avant
  le premier arrêt (FR-018, FR-019). Les succès sont écrits dans
  `cancel_stopped` de la tâche racine (FR-020).
- **Raison** : T3 vérifie d'abord récursivement puis arrête. La classification
  existe déjà dans le code, fondue avec l'effet. On la sépare, on ne la réécrit pas.
- **Alternatives** : annulation sans vérification mais avec journal (rejeté :
  laisse un arbre à moitié mort) ; verrou global d'annulation (rejeté : non
  demandé).

## R8 - Reçus de refus (T3 A4)

- **Constat** : l'admission rejoue une clé existante (`by_agent_request`,
  `N:416-422`). Un refus n'écrit rien. Un refus qui dépend de l'état du monde
  (existence du `cwd`, registre) peut changer entre deux rejeux.
- **Décision** : table `native_delegation_refusals(owner, request_id, canonical,
  code, refused_at)`. Un refus définitif y est écrit, un rejeu avec le même
  canonique rend le même code, un autre canonique rend `envelope_mismatch`.
  Liste fermée à cinq codes (correction R18) : `cwd_outside_parent_project`,
  `cwd_unavailable`, `model_unavailable`, `effort_unavailable`,
  `agent_type_unknown`. Les refus purement
  syntaxiques (`invalid_request`, `reply_timeout_out_of_range`) se rejouent déjà à
  l'identique : pas de reçu. Les refus transitoires (`referent_paused`,
  `task_limit`, `retention_saturated`, `identity_unavailable`,
  `reply_timeout_exceeds_parent`, `parent_*`, `delegation_grant_required`,
  `permission_*`) ne sont jamais écrits. T029 inventorie les codes de registre
  réels avant le code.
- **Purge** : 30 jours, dans le passage d'entretien de la 150.
- **Alternatives** : réutiliser la table de pierres tombales de la 150 (rejeté :
  elle prouve un arbre purgé, pas un refus ; mélanger les deux brouille les
  lectures) ; garder tous les refus (rejeté : figerait un refus transitoire).

## R9 - Annulation pendant la pause

- **Constat** : une ligne de pause non reprise garde l'agent dans
  `recoverable_execution_ids_for_agent` (`E:1456-1485`). Si la tâche est annulée
  pendant la pause, la levée pourrait relancer son exécution.
- **Décision** : `may_resume` refuse une tâche `cancelling` ou `cancelled`. Une
  fonction de magasin `abandon_pause_interruption(execution_id)` ferme la ligne
  (`resumed_at` posé, `resume_execution_id` nul).

## R10 - Pas de nouvel état, pas de nouvelle table pour la suspension

- **Décision** : la suspension est un fait de la tâche (`suspended_at`), pas un
  état. Les champs vivent dans le payload JSON de la tâche avec `#[serde(default)]`
  (`delegation.rs:15-58`). Aucune migration.
- **Raison** : un nouvel état changerait la vue Lineage 149 et `pending()`
  (`delegation.rs:169`).
- **Coût accepté** : la vue Lineage T3 montre `working` pendant la pause
  (question 3).

## R11 - Garde unique

- **Décision** : `AutonomousEffect::NativeMissionDispatch` dans
  `referent_control.rs:105-133`. La pause donne `Deferred { motif: "pause" }`. Les
  sites : lancement (`spawn_task` appelé depuis `tick`) et remise de mission
  (`mission_pending`). Le suivi de demande réutilise `ReminderNudge`. L'envoi du
  résultat et de l'avis d'échec reste admis (H3).
- **Raison** : l'ADR 027 impose qu'un puits autonome passe par la garde unique.

## R12 - Ce que T3 apporte et ce qui reste différent

T3 rejoue les événements persistés au démarrage et reprend le tour racine
interrompu. Un enfant non repris « doit encore un résultat ». Bridget reprend la
mission avec le message d'origine et une borne de reprises. Bridget ajoute le gel
des horloges, que T3 n'a pas.

## Corrections issues de la contre-revue 152

Chaque point ci-dessous a été vérifié dans le code avant d'être retenu. Voir
`adversarial-review-codex.md`.

## R13 - Reprise d'un enfant déjà connecté (objection 1, critique)

- **Constat** : `prepare_recovery` ne traite que `recovery_candidates()`, c'est-à-dire
  les lancements encore en cours (`D:3910-3935`). Un enfant déjà connecté a une
  entrée désirée `Running`. La boucle `desired.equipiers` (`D:3945-4020`) lui
  crée une commande neuve : `command_id: "recovery-<uuid>"`, `ownership: None`.
  La nouvelle génération n'a donc ni lien natif ni `delegation_id`. Les contrôles
  liés à `task_id` échouent ensuite : `parent_fact` (`N:43-45`), l'admission d'un
  descendant (`N:462-469`), `restart_identities` (`N:850`, `:857`, `:868`),
  `queued_owner_lost` (`N:890`). Un deuxième redémarrage ferait même échouer
  `restart_identities` pour toutes les tâches (`native_restart_identity_mismatch`).
- **Décision** :
  1. Deux chemins explicites dans T024 : (a) candidat en cours, via
     `prepare_recovery` (le bail garde `command_id` et `ownership`,
     `fleet.rs:1695-1713`) ; (b) enfant connecté, via un ordre de reprise qui passe
     `ownership: ownership_from_desired(&equipier)` (`fleet.rs:1649`) pour les
     entrées natives.
  2. Champ `fleet_command_id: Option<String>` sur la tâche : commande de la
     génération courante, `None` signifie `task_id`. Une fonction
     `Task::fleet_command()` remplace `task_id` dans les comparaisons de
     commande (`managed_by_instance`, `resolved_definition_for_command`,
     `entry.command_id`). Les comparaisons de `delegation_id` gardent `task_id`
     (le lien porte déjà `task_id`).
  3. Le champ est écrit à la reprise, avant l'envoi au superviseur, et relu au
     démarrage suivant.
- **À prouver (test rouge T004)** : la réservation d'un lien avec un parent
  d'instance disparue (mission imbriquée) est acceptée par la flotte. Si elle est
  refusée, la reprise d'une mission imbriquée échoue avec `unreachable`, et la
  limite est documentée.
- **Alternatives** : réutiliser `command_id = task_id` pour la reprise (rejeté :
  la saga de `task_id` est déjà `Connected`, un rejeu rendrait l'issue ancienne
  sans lancer) ; supprimer les comparaisons de commande (rejeté : elles prouvent
  l'identité du parent, ADR 148).

## R14 - Compteur de reprises dérivé, atomique (objection 2)

- **Constat** : `reconstruct_active_for_agent` valide sa transaction
  (`E:1443`, `tx.commit()`) avant toute écriture de la tâche, qui vit dans un autre
  magasin (`DelegationStore`, connexion propre). Un crash entre les deux perd le
  compte ; compter dans une garde `may_resume` double le compte au rejeu.
- **Décision** : le compte est dérivé, pas stocké. La reconstruction écrit déjà,
  dans sa transaction, une ligne `execution_continuations` (mode `reconstructed`)
  et, pour une pause, `resume_execution_id` (`E:1422-1433`). Le nombre de
  reprises après panne = nombre d'exécutions de la chaîne, hors racine, qui ne
  sont la cible d'aucun `control_pause_interruptions.resume_execution_id`. Le
  champ `resume_attempts` disparaît de la tâche ; `task_view` le calcule.
  `may_resume` lit le compte sans rien écrire. La borne (`< 2`) est testée avant
  la reconstruction ; la reconstruction fait croître le compte dans sa propre
  transaction.
- **Raison** : zéro écriture à coordonner, atomicité par construction.
- **Alternatives** : champ stocké dans la tâche, sauvegardé après coup (rejeté :
  non atomique) ; colonne neuve dans `executions` (rejeté : migration sans gain).

## R15 - Panne pendant une pause (objection 3)

- **Constat** : la décision de reprise de T023 teste `mission_deadline_at` brut.
  Une tâche `suspended_at` non réglée au moment de la panne serait jugée expirée
  à tort. Par ailleurs, une exécution `interrupting` n'obtiendra jamais son
  terminal : l'ancien wrapper est mort.
- **Décision** :
  1. Fonction `settle_suspension(task, now)` partagée par `tick` et
     `prepare_restart`. Si la pause est encore active, la tâche reste suspendue :
     aucun contrôle d'échéance. Si la pause a été levée pendant l'arrêt, la durée
     est calculée jusqu'à la date de levée lue dans `control_events`
     (`referent_control.rs:162`, colonnes `at`, `kind`), à défaut `now`. Le
     décalage est appliqué avant le contrôle d'échéance.
  2. Dans `prepare_restart`, la réconciliation des groupes est déjà achevée (le
     commentaire de `N:940` le dit). Une exécution `interrupting` qui a une ligne
     de pause non reprise passe alors à `interrupted` par
     `transition_if_current` (raison `pause_restart_reconciled`). T015 n'attend
     plus un terminal absent.
- **Alternatives** : attendre un délai puis forcer (rejeté : non déterministe) ;
  ignorer l'échéance au redémarrage (rejeté : une mission sans pause doit
  toujours expirer).

## R16 - Chemin A complet (objection 4)

- **Constat** : `native_delegation::rejected` n'est que la première étape de
  `DeliveryRejected` (`D:18078-18081`). Le gestionnaire retire ensuite la demande
  de `pending_replies` et envoie `Échec de livraison de la demande #id` à
  l'émetteur (`D:18095-18125`). Ignorer `rejected()` protège la tâche mais le parent
  reçoit l'avis d'échec et le suivi disparaît.
- **Décision** : `rejected()` rend un booléen `pause_interruption`. Quand il est
  vrai, le gestionnaire retourne `None` aussitôt : suivi conservé, aucun avis. Le
  test envoie l'événement complet `WrapperToDaemon::DeliveryRejected`.

## R17 - Choix de la dernière exécution (objection 5)

- **Constat** : le tri `created_at DESC, execution_id DESC` n'exprime pas la
  filiation : `created_at` est en secondes et `execution-recovery-<uuid>` est
  aléatoire. Chaque reconstruction crée une exécution de génération
  `parent + 1` (`E:1402`). Les pauses ne comptent pas dans la borne de 2 : la
  chaîne n'est pas bornée par 3.
- **Décision** : la dernière exécution est la feuille de plus grande
  `generation` de la chaîne (`ORDER BY generation DESC LIMIT 1`). Une feuille est
  une exécution sans enfant dans `execution_continuations`. La complexité est
  O(c), c = nombre de pauses et de reprises de la mission, sans borne constante
  mais petit en pratique.
- **Alternatives** : tri par `created_at` (rejeté : égalités) ; id courant stocké
  dans la tâche (rejeté : état redondant et non atomique).

## R18 - Reçus de refus : liste exacte (objection 8)

- **Constat** : `registry.get` renvoie un texte dynamique avec chemin de registre
  et liste des types (`registry.rs:157`). `for_delegation` renvoie des codes
  stables `model_unavailable` et `effort_unavailable` (`registry.rs:326`, `:328`).
- **Décision** : liste fermée de cinq codes (`cwd_outside_parent_project`,
  `cwd_unavailable`, `model_unavailable`, `effort_unavailable`,
  `agent_type_unknown`). Le texte de `registry.get` est ramené au code
  `agent_type_unknown` à l'admission. Le texte n'est jamais stocké. Les autres
  erreurs du registre ou des permissions (`delegation_protocol_unavailable`,
  politique, posture) ne sont pas gardées.

## R19 - Gel du suivi limité à la pause qui chevauche la demande (objection 6)

- **Constat** : la demande suivie est créée à l'envoi de la mission, donc après
  `queued`, `starting`, `mission_pending`. Une pause subie avant l'envoi ne
  chevauche pas la demande. La cumuler puis la soustraire à une demande plus
  récente raccourcit son temps écoulé.
- **Décision** : le compteur persisté s'appelle `request_frozen_secs`. Une
  suspension ne change pas d'état pendant sa durée. À la levée, `d` est ajouté à
  `request_frozen_secs` seulement si l'état est `working`.
- **Écart voisin** : la mission est envoyée avec `issued_at = task.created_at`
  (`N:1148-1156`), et le suivi calcule son échéance à partir de cette date
  (`D:8816-8824`). Après une pause en `queued`, l'échéance du suivi serait déjà
  passée. Champ `mission_issued_at`, posé à la première tentative d'envoi et
  persisté avant l'envoi. Les tâches sans ce champ gardent `created_at`.

## R20 - Annulation : contrôle avant toute mutation, reprise depuis la racine (objection 7)

- **Constat** : `cancel_tree` marque les descendants `cancelling` avant tout arrêt
  (`N:1390-1394`). `tick` traite chaque tâche `cancelling` comme une racine
  d'annulation (`N:994-996`). Si la racine est refusée, le tick suivant annule les
  descendants un par un, ce qui contourne le contrôle global.
- **Décision** :
  1. Le plan de classement s'exécute avant le marquage des descendants.
  2. `tick` ne lance `cancel_tree` que pour les racines : une tâche `cancelling`
     dont l'ancêtre est `cancelling` est ignorée tant que sa racine est en cours.
  3. `cancel_stopped` (durable, sur la racine) rend la reprise après un crash
     entre deux arrêts confirmés exacte.

## R21 - Conservation des états non repris (objection 9)

`prepare_restart` ne touche que `starting`, `mission_pending`, `working`. Les
états `waiting_for_children`, `result_available` et terminaux gardent leur
traitement actuel (`mark_stopped` de l'entrée de flotte, `N:945-950`). Un test de
conservation par état le prouve (T023).

## Ronde 2 de la contre-revue (4 points aux frontières de crash)

Les points sont vérifiés dans le code ; voir `adversarial-review-codex.md`,
section Ronde 2. Cette ronde remplace R4 (points 2 et 3), R13 (point 2 : champ
`fleet_command_id`), R14 (compteur dérivé), R15 (point 1 : `settle_suspension`)
et R19 (compteur `request_frozen_secs`). Ces sections gardent leur historique.

## R22 - Compteur de redémarrages survécus (ronde 2, point 1)

- **Constat** : en `starting` et `mission_pending`, aucune exécution de mission
  n'existe (l'exécution naît à l'admission de l'envoi). Les relances de l'enfant
  ne créent aucune continuation. Un compte dérivé de la chaîne reste à 0 et la
  borne ne s'applique jamais (`N:608`, `spawn_task` pose `starting` sans exécution).
- **Décision** : retour à un compteur stocké, `restart_resumes`, mais incrémenté
  et sauvegardé **dans la décision de reprise de `prepare_restart`**, avant
  toute reprise. C'est le seul endroit qui l'écrit.
- **Pourquoi c'est sûr** : la reconstruction d'exécution n'a lieu qu'après le
  démarrage terminé, quand l'enfant se reconnecte. Un crash avant la sauvegarde
  n'a donc rien reconstruit, et un crash après compte déjà. Le compteur ne peut
  que sur-compter. L'objection 2 de la ronde 1 visait un sous-compte
  (commit de la reconstruction avant l'écriture du compteur) : il n'existe plus.
- **Alternatives** : compter les commandes `recovery-<uuid>` de flotte (rejeté :
  absentes pour un lancement en cours, qui garde sa commande) ; garder le compte
  dérivé et ajouter un second compteur (rejeté : deux mécanismes).
- **Retiré** : `crash_resume_count` (T010).

## R23 - Début du suivi lu dans la demande durable (ronde 2, point 4)

- **Constat** : `send_as` admet l'envoi et crée la ligne `tracked_requests` dans
  la transaction d'admission (`D:8816-8824`). La tâche passe `working` plus tard
  (`N:1188`). Entre les deux, la demande est suivie alors que la tâche est
  `mission_pending` : un gel conditionné à `working` ne retire pas la pause.
- **Décision** : le temps de pause d'une demande est `pause_overlap(created_at de
  la ligne tracked_requests, now)`. Il ne dépend plus de l'état de la tâche ni
  d'un compteur accumulé. Il est recalculé à chaque tick et à la reconnexion
  (affectation absolue, donc idempotente).
- **Retiré** : `request_frozen_secs`.

## R24 - Identité au redémarrage : dérivée, et jamais fatale (ronde 2, point 2)

- **Constat** : `restart_identities` rend `Err("native_restart_identity_mismatch")`
  dès qu'une tâche est incohérente (`N:845-868`), et `?` dans le démarrage
  (`daemon.rs:4553-4555`) empêche le daemon de démarrer. `fleet_command_id`,
  `child_instance` et l'entrée de flotte seraient écrits séparément : un crash
  entre les trois écritures produirait cet état.
- **Décision** :
  1. Pas de champ `fleet_command_id`. La commande courante est lue sur l'entrée de
     flotte dont le lien attesté porte `delegation_id == task_id`. Plus de
     deuxième écriture à synchroniser.
  2. Avant le contrôle, `restart_identities` réconcilie l'instance enfant depuis
     le lien attesté de la flotte.
  3. Si la réconciliation échoue, la tâche passe `failed`
     (`native_restart_identity_mismatch`), est exclue de la table d'identités, et
     le démarrage continue. `restart_identities` ne rend plus `Err` pour une
     incohérence de tâche.
- **Choix « échec explicite » (Art. XIX)** : réparer chaque incohérence possible
  demanderait de modéliser chaque ordre de sauvegarde. Un échec nommé d'une
  tâche est borné, testable, et la sûreté du démarrage prime sur l'exhaustivité.
  Les groupes de processus d'une tâche exclue sont terminés par la
  réconciliation existante (`reconcile_stale_groups_with_native`).
- **À prouver (T048, T004)** : l'API de lecture du lien de flotte par identifiant
  de lien. Si elle manque, la réconciliation de l'instance est abandonnée et
  seul l'échec nommé reste.

## R25 - Pause durable sans état intermédiaire (ronde 2, point 3)

- **Constat** : la pause est enregistrée par `referent_control::set` puis
  `interrupt_executions_for_pause` (`D:9598-9626`). Si le daemon tombe avant le
  premier tick, aucun `suspended_at` n'existe. Si la pause est levée pendant
  l'arrêt, le décalage ne peut pas être retrouvé.
- **Décision** : plus de `suspended_at`. Le temps de pause est une fonction du
  journal durable : `pause_overlap` lit `control_events` (`pause_on`, `pause_off`,
  `referent_control.rs:162-166`) et `control_state`. L'échéance effective vaut
  l'échéance enregistrée plus la pause écoulée depuis l'admission. Aucune
  réécriture, donc ni perte ni double décalage possible.
- **Effet de bord favorable** : T012 n'écrit plus rien ; `settle_suspension`
  disparaît ; T017, T018, T041 se simplifient.
- **Coût** : une lecture de `control_events` seulement quand le résultat sert
  (expiration apparente, demande native, décision de reprise). Le journal est de
  l'ordre de quelques événements humains.
- **Alternatives** : persister `suspended_at` dans la même transaction que la
  pause (rejeté : la pause vit dans un autre magasin que les tâches) ;
  réécrire les échéances à la levée (rejeté : non idempotent).

## R26 - Lancement : date posée à la première soumission

Avec le journal des pauses, la date de lancement n'a plus besoin d'être ajustée
à la levée. `launch_issued_at = now` est sauvegardé avant le premier
`submit_spawn` (tâche `queued` sans saga). Une tâche retenue par la pause est
lancée après la levée avec une date récente. Une saga existante garde son
enveloppe.
