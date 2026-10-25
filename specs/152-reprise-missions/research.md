# Recherche - Session152 reprise des missions natives

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/research.md#extrait-1).

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
  4. Hypothèse historique corrigée en R27 après livraison de 150 : la table
     `tracked_requests` n'est pas modifiée par ce premier projet de gel. Son index sur
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

## R7 - Annulation en cascade vérifiée d'abord (adaptateur de bureau A3)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/research.md#extrait-2).

## R8 - Reçus de refus (adaptateur de bureau A4)

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/research.md#extrait-3).

## R11 - Garde unique

- **Décision** : `AutonomousEffect::NativeMissionDispatch` dans
  `referent_control.rs:105-133`. La pause donne `Deferred { motif: "pause" }`. Les
  sites : lancement (`spawn_task` appelé depuis `tick`) et remise de mission
  (`mission_pending`). Le suivi de demande réutilise `ReminderNudge`. L'envoi du
  résultat et de l'avis d'échec reste admis (H3).
- **Raison** : l'ADR 027 impose qu'un puits autonome passe par la garde unique.

## R12 - Ce que adaptateur de bureau apporte et ce qui reste différent

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/research.md#extrait-4).

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

## R27 - Audit frais après livraison de 150 et 151

Source : lecture du worktree
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions`, base
`cd2bb015755fde7af26cf87fdb8f4e658172e237`, le 2026-10-11. Aucun test ni service
lancé par cet audit. La comparaison complète figure dans `reuse-audit.md`.

- **Dépendances présentes** : identités 148, filiation et permissions 149,
  délai/rétention/garde 150, attente/accusé 151. Les repères ci-dessous sont ceux
  de cette base, avant tout code 152.
- **Horloge de mission déjà corrigée par 150** : l'échéance brute part désormais
  de l'admission de l'exécution, avec repli sur `created_at`.
  Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1287`.
  Le gel 152 ajoute `pause_overlap` à cette échéance. Les lectures de l'exécution
  d'origine qui prouvent l'admission restent distinctes de `latest_in_chain`, qui
  désigne l'exécution courante.
- **R4 n'est plus une preuve d'immuabilité de la table suivie** :
  `align_request_deadline(st: &DaemonState, task: &Task)` réécrit déjà l'échéance
  durable des missions avec un `reply_timeout` explicite. Il réessaie au tick
  suivant si l'écriture échoue. Le gel 152 conserve cet alignement ; il n'ajoute
  pas de décalage relatif de la ligne à chaque levée.
  Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1780`
  et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/store/ledger_requests.rs:319`.
- **T024 réutilise la fonction de propriété existante** :
  `ownership_from_desired(equipier: &DesiredEquipier) -> Option<SpawnOwnership>`
  est privée au module `fleet`. Le plan prévoit sa visibilité dans le crate,
  sans copier la fonction.
  Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/fleet.rs:1666`.
- **L'API recherchée par R24 existe** :
  `IdempotencyStore::agent_link_by_id(&self, link_id: &str)` lit le lien durable.
  T043 doit réutiliser cette lecture. Une façade `FleetSupervisor` sans règle
  supplémentaire n'est pas nécessaire.
  Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/idempotency/agent_links.rs:96`.
- **Arrêt réel à préserver** : `ManagedStopTarget` porte le classement et les
  effets actuels. Aucun `ResumeStrategy`, `StoppedDesired`, `StopKind` ni
  `classify_stop` n'est livré. Une entrée désirée `Stopped` sans superviseur rend
  déjà immédiatement `Stopped` ; l'extraction T026 garde ce comportement.
  Preuves : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:1243`
  et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:9561`.
- **Invariants 151** : les mutations de tâches conservent `acknowledged_at` et
  passent par `save_projected`, qui signale le guichet d'attente après commit.
  T008 ne crée aucun champ de suspension dérivé.
  Preuve : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/delegation_lineage.rs:269`.

H4 reste à prouver par T004 sur les deux chemins de reprise. La lecture ne vaut
pas validation d'un enfant relancé ni d'un modèle actif.

## R28 - Ajustements de la contre-revue Claude et de l’analyse fraîche

La contre-revue du plan, coordonnée par le principal, rend
APPROVE_WITH_CHANGES. La trace détaillée est dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/adversarial-review-claude.md.

- Le bloc d’environnement de `spawn_task` est après ses contrôles de révocation,
  de sources et de cwd. L’extraction ne les remplace pas : T022/T023/T024 les
  réutilisent avant la reprise, avec les cas T004 déjà prévus.
- La réconciliation appelle `self.load` pour tous les noms de marqueurs, puis
  vérifie l’identité native. Une simple exclusion de la table d’identités ne
  ferme donc pas le démarrage non fatal. T043/T048 vérifient le marqueur natif lisible avant
  publication et prouvent le traitement de son incohérence ; un marqueur
  illisible n’est jamais une identité attestée. Les fichiers corrompus/illisibles
  restent hors FR-029 et gardent le refus sûr existant. `load` existe à la
  ligne1171 et `remove` à1385 ; cette suppression ne répare pas une identité et
  n’est sûre qu’après disparition prouvée du groupe ou PID recyclé.
- `RequestWait` 151 observe `deadline_at` brut et calcule l’expiration sans
  modifier la ligne. T018 applique l’échéance métier native effective, en
  gardant le budget de l’appelant inchangé. T017 applique le temps restant
  effectif à l’admission d’un descendant. La date brute exposée reste durable.
- `current_task_view`, qui reçoit le daemon, calcule `suspended`. `task_view`
  reste pur ; aucune nouvelle lecture globale ni écriture de suspension.
- L’annulation conserve les erreurs et résultats déjà terminaux ; un bail de
  lancement expiré garde un échec nommé. Les cas sont intégrés à T027/T024.
- `NativeMissionDispatch` partage la logique de garde existante, tout en nommant
  un effet distinct de l’admission. `pause_overlap` utilise l’index temporel
  du schéma existant : O(log N + P), au lieu d’ignorer le coût de sélection.

Preuves actuelles :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:824.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:289.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:126.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:165.

## Preuve d’exécution T003 — premier RED, 2026-10-11

Le principal a compilé les trois tests de pause puis observé 0 PASS/3 FAIL,
RC101, 33,45 s. La production native/daemon est confirmée inchangée par lui.
Chemin A : tâche failed, suivi supprimé et avis d’échec réel au parent.
Chemin B : interrupted provoque failed. Ordre inverse : absence de reprise
avant interrupted, puis failed et aucune reprise après son arrivée.
Les trois assertions d’état échouent avec failed au lieu de working.
T002/T003 restent non cochées jusqu’au GREEN. Les autres assertions ne sont
pas toutes atteintes ; le reçu distingue logs empiriques et assertions.

Journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
Preuves brutes : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-pause152.log et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-pause152.json.
Racine privée réelle : /var/folders/82/29vmttnx4pz9mc73kzzfvw1r0000gn/T/b.aczqbxyd (mode 0700 observé).
Écart : /var/folders au lieu de la recette courte /tmp/b.XXXX. Le principal
corrigera les prochains runs ; cette preuve initiale n’est pas réécrite.

## Preuve d’exécution — fondations et RED étendu, 2026-10-11

La nouvelle passe du principal est documentée dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
Source brute :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-fondations152-results.json.

Fondations : 5 PASS/0 FAIL, tous RC0, répartis entre la chaîne (2),
le payload 151 (1), pause_overlap (1) et la table de garde existante (1).
T008/T009/T010/T050 sont cochées avec l’autorisation du principal ;
les autres cases sont conservées. La progression est 5/50.

RED étendu : compilation réussie, puis 0 PASS/7 FAIL, RC101.
Six FAIL sont des RED valides, dont les trois T003 déjà observés.
T005 s’arrête sur un témoin incorrect : expired attendu contre timed_out
du contrat réel. L’oracle du témoin doit être corrigé et le comportement
ancien rejoué ; T005 ne constitue pas encore un RED définitif.
Le reçu conserve les sept résultats bruts et les assertions réellement
atteintes, sans annoncer de succès global. La racine de ce run est
/tmp/b.2ykydeej, mode 0700 observé ; l’écart du premier run reste historique.

Rien n’est livré ni activé. La contradiction H6/quickstart sur la relance
est conservée comme limite documentaire, à traiter par le principal lors
de la livraison, après observation des travaux et processus actifs.

## Reproduction complète — compilation R1 et exécution R2

R1 n’est pas un RED comportemental : erreur de compilation E0061/E0600
dans la fixture register_connected, RC101 en 3,92 s, aucun test exécuté.
Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-results.json.

R2 a compilé puis exécuté 14 RED natifs et 5 RED E2E, tous en échec attendu.
T005 atteint maintenant l’assertion native après le témoin timed_out corrigé ;
sa réserve d’oracle antérieure reste historique. Les deux témoins ordinaires
passent, chaque run RC0, et permettent de cocher T011 sur autorisation du
principal. La progression est 6/50 ; les tâches RED restent décochées avant
leur GREEN. Ces résultats ne sont pas additionnés aux précédents runs
pour produire un total global.

Preuves :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/red-complet152-r2-results.json.
Le JSON atteste la stabilité des six empreintes sources pendant cette passe,
pas le gel du code lors du GO d’implémentation suivant. Le journal détaille
les quatre commandes, les SHA des preuves, les diagnostics et leurs limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

T032 impose une correspondance test149 → FR pour l’adaptation du contrat
de reprise FR-013/FR-014/FR-015/FR-027. Les échecs hérités ne servent pas à
masquer ces écarts volontaires. Les refus de droits ou d’identité invalides
et les états protégés de FR-016/FR-017/FR-029 gardent leurs oracles.
T033/T035 ne sont pas publiées ; rien n’est livré ni activé.

## US1/US2 R1 et preuves complémentaires

US1/US2 : 13 PASS/3 FAIL ; T029 : 2 RED attendus séparés.
Les horloges, le lancement retenu 600 s, le suivi après reconnexion, la vue
additive et la projection Lineage passent. Les douze clôtures prouvées sont
T005/T006/T039/T049/T012/T013/T016/T018/T020/T021/T044/T045 : progression 18/50.
T002/T003/T014/T015 restent ouvertes pour les reprises ; T019 pour le résultat ;
T017 pour l’admission d’un descendant. Aucun cas ni ID nouveau n’est ajouté.
Source :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r1-results.json.

Le diagnostic de réception reproduit 0 PASS/1 FAIL, RC101. Sa donnée réelle
montre phase150 dispatching, exécution starting et ACK absent dans la fixture.
La façade corrige celle-ci avec le vrai DeliverAcked ; aucun GREEN de reprise
n’est déduit avant le rejeu. Source :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/diagnostic-reception-152-r1-results.json.

Le magasin de refus compte 12 PASS/0 FAIL, trois RC0, cinq empreintes stables
pendant ces runs. T030 reste ouverte jusqu’au branchement API et au GREEN
des refus ; T029 normalisation/record_refusal et T031 purge restent à brancher.
Source :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/store-refus-152-r1-results.json.

Les commandes, SHA, données réelles, modes privés et limites sont conservés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
Les preuves antérieures restent historiques ; aucun total global n’est inventé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/152-reprise-missions/research.md#extrait-5).

## US1/US2 R2 puis R3 T019 — 18 PASS ciblés, reprise validée

R2 : 17 PASS/1 FAIL sur sept filtres ; les trois reprises deviennent GREEN
après le vrai DeliverAcked initial de la fixture. Le descendant est borné
par l’échéance effective : 60 s admis, 120 s refusé. T002/T003/T014/T015/T017
sont cochées avec ces preuves. Source :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r2-results.json.

Le FAIL R2 T019 porte sur l’état immédiatement après capture_reply :
waiting_for_children, et non result_available. R3 corrige seulement cet
oracle de fixture et conserve corps durable, remise exacte au parent,
pause active, état final result_available et cleanup. Résultat 1 PASS/0 FAIL,
RC0 ; auto-relecture du test effectuée sans nouvelle exécution. Source :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us1-us2-152-r3-t019-results.json.

T019 est cochée ; progression 24/50. Les preuves actuelles US1/US2 combinent
17 PASS R2 et 1 PASS R3, sans prétendre à un run unique ou à une suite globale.
Les 19 RED historiques natifs/E2E, l’échec de compilation R1 et les premières
traces de fixture restent distincts et conservés. SHA et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

T030 reste ouverte jusqu’à son intégration API et au GREEN des refus ;
T029/T031 ne sont pas clôturées. T033/T035 restent préparées en mémoire,
sans publication avant US3/US4/US5 GREEN et les finitions.
Rien n’est livré ni activé.

## US3 — limite des fixtures T004a/T047 avant leur GREEN

Le principal a identifié une limite du montage initial. fleet.recover_commands
ferme le Starting initial lorsque desiredfleet.json n’a pas encore été écrit.
Les anciennes fixtures T004a/T047 rouvraient le magasin avant cette persistance.
Leurs RED compilés reproduisent bien l’échec natif unreachable ; ils ne prouvent
pas qu’un enfant Launching était récupérable à cette frontière.

La nouvelle fixture représente une frontière réelle avec la primitive existante
register_connected_observed : sauvegarde de desired, puis retour Err avant
l’écriture de l’issue SQLite, tandis que la tâche reste Starting.
Le principal annonce les observations desired, phase, lease et lien dans les
prochains GREEN. Aucun résultat de cette nouvelle frontière n’est encore
attribué à ce reçu ; aucun hook de production ou mécanisme de flotte nouveau
n’est déduit de ce changement de fixture.

Cette précision complète les hypothèses de R6/R13 et les cas déjà prévus
T004a/T047. Les anciennes traces brutes ne sont pas remplacées ni reclassées en
échec de compilation. Leur portée est limitée à l’échec effectivement observé.
T004/T047 restent décochées tant que leurs cas complets et la récupérabilité
réelle ne sont pas prouvés. Source de cette limite : constat empirique transmis
par le principal le 2026-10-11 ; aucune exécution par le propriétaire documentaire.

## US3 R1 — récupérabilité attestée, E2E encore partiels

Sources :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/us3-152-r1-results.json
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t022-regression-152-r1-results.json.

US3 : 8 PASS lib, 2 PASS E2E et 3 FAIL E2E. T004a/T047 montrent maintenant
la vraie frontière : desired=Running, phase Starting, bail et lien attestés,
puis candidat retrouvé à la réouverture. Bootstrap natif présent dans le cas
valide ; révocation, source modifiée et bail expiré refusent toute reprise.
Pour starting et mission_pending, les deux premiers crashes portent le compteur
à 1 puis 2 ; le troisième ferme avec native_resume_exhausted, sans reprise.
Cette preuve complète R6/R13 sans réinterpréter la fixture RED initiale.

Les trois FAIL E2E arrêtent trop tôt leurs oracles de nouvelle génération,
started ou disparition du groupe. Le principal observe ensuite les nouveaux
started et le groupe final gone, sans PID orphelin. Les racines sont
/private/tmp/n148-8c139604cf,
/private/tmp/n148-4af8c04beb et
/private/tmp/n148-40107f7ecc.
Ces observations sont attribuées au principal, pas à une exécution documentaire.
Le propriétaire qualité adapte les E2E sans changement de production pour ces
courses. Le GREEN complet n’est pas encore déduit de ces observations.

Régression T022 native148 --lib : 22 PASS/1 FAIL, RC101. Le test
native148_cut_after_admission_retains_exact_ids_and_frozen_model compare
task_view pur à l’API enrichie par suspended/resume_attempts.
T032 couvre désormais ce changement volontaire FR-024/T021 : égalité stricte
de current_task_view aux trois étapes, avec IDs, modèle et droits conservés.
Aucun échec hérité, nouveau test, ID ou périmètre n’est invoqué pour ce cas.

Progression conservée à 24/50 ; T022/T023/T024 et les autres cas US3 attendent
leur correspondance complète aux GREEN. SHA, diagnostics, compteurs et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
T033/T035 restent en mémoire ; rien n’est livré ni activé.

## US3 R2 — frontières d’identité GREEN, mission imbriquée et gardes connectées ouvertes

Les deux journaux R2 donnent native148 --lib : 23 PASS/0 FAIL, RC0 ; E2E
native152_rouge : 4 PASS/1 FAIL, RC101. Aucun test n’est ignoré. Le JSON atteste
sept empreintes source stables. Les quatre frontières d’identité passent,
ainsi que native148_cut_after_admission_retains_exact_ids_and_frozen_model.
Le FAIL de comparaison R1 reste conservé ; T022/T032 ne sont pas clôturées
par cette seule tranche.

Le cas native152_rouge_panne_mission_working échoue sur nested152 absent pour
3bc90d3a-56d5-4af4-949d-de015e47e530 dans
/private/tmp/n148-d5b7866569. Le principal rapporte un fournisseur Python de
fixture qui refuse --resume et prépare son adaptation. Aucun défaut de
production n’est établi par ce diagnostic ; le GREEN imbriqué manque encore.

Les refus révocation/source modifiée/bail expiré sont prouvés sur le vrai
candidat Starting par native152_rouge_panne_starting_bootstrap_et_gardes.
Les refus révocation/source sur le chemin déjà connecté restent à exécuter
dans le même cas T004/T024, selon l’autorisation du principal. Le bail expiré
ne concerne que Starting. La frontière Starting avec desired/lien attestés
ne réinterprète pas le RED initial sans desired comme récupérable.

Ces preuves distinguent tests lib, daemon isolé avec fixtures, états et
observations runtime transmis par le principal. Elles ne valident aucun
runtime de production ou modèle réel. Aucune nouvelle case US3 n’est cochée :
24/50 reste inchangé. T033/T035 restent en mémoire. Les preuves brutes R1/R2,
les commandes, les durées et les SHA sont consignés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## US3 R3 gardes — refus Starting et Connected prouvés en lib

Le filtre native152_rouge_panne --lib donne 2 PASS/0 FAIL/0 ignoré, RC0,
avec six empreintes source stables. Le test T004 existant contient sept
sous-cas : Starting valide/révocation/source/bail expiré, Connected
valide/révocation/source. T038 pause active/levée passe aussi.

Les deux branches refusent la révocation par permission_not_inherited et
la source modifiée par settings_revision_changed : failed, compteur zéro,
aucune reprise préparée. Starting avec bail expiré rend spawn_timeout,
compteur 1 observé et aucune reprise préparée. Le cas valide prépare une
reprise, compteur 1, avec bootstrap et filiation corrélés. Connected prépare
la génération 2. Les assertions de codes et les observations de compteurs
restent distinguées dans le journal d’implémentation.

Les gardes Connected ouvertes au moment de R2 sont maintenant GREEN en lib.
La source relue conserve la vraie frontière Starting : desired sauvegardé
avant l’erreur de l’issue SQLite. Elle ne transforme pas le RED initial sans
desired en candidat récupérable. Cette preuve lib ne lance pas de fournisseur
et ne ferme pas le FAIL E2E nested152 de R2. Aucun nouveau cochage US3 :
24/50 reste inchangé. T033/T035 restent en mémoire.

Les lignes de code/journal, les états, les compteurs, les SHA et la portée
exacte sont consignés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## US3 R3 E2E — sortie après quarantaine, sans reconstruction de mission

Le filtre E2E native152_rouge donne 4 PASS/1 FAIL/0 ignoré, RC101,
avec sept empreintes source stables. Les quatre frontières d’identité passent.
native152_rouge_panne_mission_working échoue encore sur nested152 absent
pour d0f0af8f-29da-40dc-93d2-b87b7aae8688, dans
/private/tmp/n148-6b23b800e8. Cette preuve R3 est distincte du FAIL R2.

Le diagnostic réel confirmé par le principal, la qualité et la façade
montre --resume correct, initialisation reçue, session enregistrée,
stderr vide et aucun prompt pour la génération 2. La tâche Working et
l’exécution Starting/génération 1/révision 0/instance NULL sont distinctes.
La remise est réaffectée à la génération 2 puis indeterminate ; aucune
continuation n’est créée. schedule_execution_recovery sort après cette
quarantaine au lieu de reconstruire la mission. Le refus d’argument Python
précédemment rapporté pour R2 n’explique donc pas ce FAIL R3.

Le principal autorise une correction minimale de production dans ce chemin
du daemon, avec l’E2E inchangé. L’hypothèse est en cours ; aucun GREEN ni
cochage US3 n’est déduit. Les 2 PASS lib R3 gardes restent une preuve séparée.
Les RED/R1/R2 et la limite du Starting initial sans desired sont conservés.
Progression : 24/50 ; T033/T035 restent en mémoire. Aucun runtime de production
ou modèle réel n’est validé par cette fixture.

Résultat, commande, durées, SHA, portée et diagnostic attribué :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## US3 R4 — reconstruction effective, source de permissions privée à attester

R4 : E2E 4 PASS/1 FAIL, RC101 ; pause lib 6 PASS puis 5 PASS, tous RC0,
aucun ignoré. Sept empreintes source sont stables pendant les trois runs.
Les quatre frontières d’identité passent. Le principal et les propriétaires
constatent l’exécution génération 2/running/provider_accepted : le correctif
du daemon remet réellement la mission. Le nouveau FAIL contient une demande
MCP nested152 refusée par permission_source_unavailable.

Ce refus est correct pour la fixture héritée : permission_snapshot=null,
PATH sans CLI claude et provider.py avec shebang. Il ne justifie aucun
affaiblissement de garde. Le prochain montage réutilise le pattern149 :
.git privé, profil privé, bin/claude dans le PATH et capture réelle du contexte.
La qualité complète seulement le fournisseur privé par un binaire compilé
qui incorpore le Python existant ; le CLI, sa SHA, les settings et l’init
doivent être observés. Aucun snapshot injecté, outil produit, dépendance
nouvelle ou installation n’est prévu. Le compilateur C est déjà requis par
rusqlite bundled. Aucun succès du nouveau montage n’est encore constaté.

Les références149 exactes, la décision T004 et les limites de preuve sont
ajoutées à
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/reuse-audit.md.
Les compteurs, SHA et diagnostics attribués sont consignés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

US3 reste ouverte à 24/50, sans nouveau cochage. Les RED/R1/R2/R3 sont
conservés. T033/T035 restent en mémoire. Aucun modèle réel ou runtime de
production n’est validé par cette fixture.

## US3 R5/R6 — montage attestable puis transport réel accepté

R5 donne 4 PASS/1 FAIL E2E, RC101. Le CLI privé compilé et les settings
sont observés sur disque ; init152 rapporte plan, Read/Glob/Grep, cwd et
--resume natif. Les SHA de ces fichiers ne sont pas celles d’un fact mémoire
publiquement accepté. Le nouveau refus est permission_attestation_unavailable.
La façade identifie definition_presence_fields qui restitue claude_stream_json
et la garde de transports qui l’excluait.

R6 admet ce transport dans la garde existante, avec les autres contrôles
d’identité, driver, révision et structure conservés. Résultat : 5 PASS/0 FAIL,
RC0. Sept sources sont stables pendant chaque run ; seul daemon.rs change
entre R5/R6, l’E2E est byte-identique. Deux redémarrages du magasin réussissent.
Le cas exige les IDs de tâche/mission, nouvelle instance, compteurs 1 puis 2,
bootstrap et filiation ; le MCP admet le petit-enfant après chaque reprise,
avec sa mission et sa filiation conservées. Le fournisseur reste une fixture,
pas un modèle réel ou une validation de production.

Les onze tâches US3 sont reliées aux preuves R1/R2/R3/R4/R6 dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.
T025 dispose déjà des bornes Starting/mission_pending/Working, du sur-compte
avant effet et de la reprise de pause sans incrément. Seul le résultat
rendu une fois après panne attend le GREEN du principal. Le complément est
posé dans le test déjà prévu ; sa lecture ne vaut pas exécution.

Aucun cochage global US3 ni 35/50 n’est encore déclaré : 24/50 reste inchangé.
T032 reste une régression finale distincte. T033/T035 restent en mémoire.
Les RED et chaque diagnostic R1 à R5 gardent leur trace historique.

## Clôture US3 après GREEN T025 R4 — 35/50

T025 passe : native152_panne_resultat_rendu_une_fois, 1 PASS/0 FAIL/0 ignoré,
RC0, sept empreintes source stables pendant le run. Les trois passes
antérieures restent conservées : R1 mauvaise instance/StaleGeneration ;
R2 contrôle post-Registered non exécuté/WouldBlock ; R3 remise SQLite acked
constatée par le principal, mais entretien maintain_stores omis par la fixture.
R4 exécute ces étapes réelles et passe. Les corrections portent seulement
sur la fixture ; le préfixe de production garde son empreinte antérieure.

Le cas final impose une seule reprise, mission/instance corrélées, feuille
d’exécution génération2, vrais ACK de mission et de résultat, puis entretien
du magasin. Trois ticks ne produisent aucune deuxième remise. Le résultat
et les flags result_sent/delivery_acked/cleanup_done sont durables, compteur1.
La borne deux, le sur-compte avant effet et le compteur de pause zéro étaient
déjà prouvés. Le dernier manque T025 est maintenant fermé.

Après le GO explicite du principal, onze tâches US3 sont cochées :
T004/T038/T047/T048 et T022/T023/T024/T025/T041/T042/T043.
Progression actuelle : **35/50 (70 %)**. La matrice R6 et les réserves
antérieures restent historiques ; aucune ancienne preuve n’est remplacée.
T032 reste une régression finale distincte. T033/T035 restent en mémoire
avant US4/US5 GREEN et les finitions. Aucun modèle réel ou runtime de
production n’est déduit des fixtures.

Commandes, compteurs, diagnostics, lignes de code et SHA des huit captures :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Clôture US4 — 41/50, crash SQL et arrêts simulés distingués

US4 R1 : native152_annulation 4 PASS, native152_rouge_annulation 2 PASS,
témoin148 des arrêts profonds 1 PASS ; tous RC0, aucun FAIL/ignoré.
Sept empreintes source sont stables pendant ces trois runs distincts.
Le refus d’un membre non arrêtable conserve les descendants sur plusieurs
ticks. Les réessais partent de la racine, suivent l’ordre profond d’abord,
sautent les confirmations durables et conservent erreur/résultat terminal.

Le crash de fixture est un vrai échec SQL après confirmation, avant le reçu,
puis réouverture du magasin. Desired Stopped permet la reprise sans
superviseur et sans nouvelle demande d’arrêt. Aucun PID/SIG réel n’est testé
par ces unitaires. Le témoin148 reste une non-régression distincte.

Le coût de précontrôle/index locaux est O(m + f + p + n log n), hors
descendants/liens/marqueurs. Les sauvegardes de la liste croissante coûtent
O(n²) octets cumulés. Ces bornes sont commentées dans les sources ; elles
ne constituent pas une mesure de performance.

Après GO du principal, six cases sont cochées : T007/T040/T026/T027/T028/T046.
Progression actuelle : **41/50 (82 %)**. L’histoire et le texte des tâches
sont conservés. US5 et les finitions restent ouverts ; T033/T035 restent
en mémoire. Aucun modèle réel ou runtime de production n’est déduit.

Oracles, lignes de code, commandes, durées et SHA des quatre captures :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Clôture US5 — preuve API et entretien, 44/50

US5 R1 : **15 PASS, 0 FAIL, 0 ignored, RC0**. La preuve magasin
antérieure de 12 PASS reste historique. Les trois tests natifs
supplémentaires exercent l’admission, la normalisation du registre et
le passage réel `maintain_stores`. Le rejeu garde le refus initial
malgré la disparition du cwd et une pause. Une autre enveloppe rend
`envelope_mismatch`. Le reçu est conservé à 30 jours et purgé au-delà.

Le magasin n’admet que cinq codes fixes. Les dix cas transitoires
séparés restent sans reçu. Le texte dynamique du registre n’est pas
enregistré. Les refus et leur purge ne changent ni tâche, ni Lineage,
ni epoch d’attente. La complexité O(R) de la purge est lue dans le code,
sans mesure de performance.

Après GO et vérification, T029/T030/T031 sont cochées :
**44/50 (88 %)**. Les six finitions restent ouvertes à ce stade.
T033/T035 sont autorisées et seront publiées après relecture du code.
Aucun succès de modèle réel, livraison ou runtime de production n’est
revendiqué. Les journaux anciens sont conservés.

Commande, oracles, limites et SHA des deux captures :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Finitions documentaires — 46/50, T032/T034 encore ouvertes

Après GO, les trois documents utilisateur et l’ADR152 sont publiés et relus.
T033/T035 sont cochées ; **46/50 (92 %)**. D1 à D22 sont présentes exactement
une fois. Les formes d’appel et les canoniques restent inchangés.
Le contrat existant ne prétend plus que le comportement152 est celui150.

La documentation distingue le précontrôle d’annulation refusé d’un échec
d’arrêt ultérieur. Elle conserve la limite de lancement300s, la borne2,
la date brute, le budget d’attente et les protections des marqueurs illisibles.
La complexité du plan est alignée sur les commentaires présents ; aucun banc
ni succès de performance n’est revendiqué.

T032 R1 : 217 PASS/16 FAIL/1 ignored, tous FAIL nouveaux.
R2 : 149 65 PASS/7 FAIL, cinq F2 stricts corrigés ;150 51 PASS/0 FAIL.
Les sept restants sont cinq adaptations de contrat et deux vrais défauts de
garde corrigés sans changer leurs oracles. Les six filtres finaux R3 sont en
cours ; T032 reste ouverte. T034 attend également R3 vert et la relecture
finale. T036/T037 restent ouvertes.

Preuves séparées, mappings, six réponses XX, limites et SHA :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## T032 R3/R4 et dernières décisions encore ouvertes

R3 termine les six filtres de référence à 232 PASS/1 FAIL/1 ignored.
Les six intégrations149 ne sont pas exécutées après l’échec du lib.
Le complément152 donne 51 unitaires PASS puis 4 E2E PASS/1 FAIL.

R4 corrige les seules routes de la fixture tick149 : 78 PASS/0 FAIL,
soit 72 unitaires et six intégrations réellement exécutées.
L’agrégation compatible de R3, avec149 remplacé par R4, donne
239 PASS/0 FAIL/1 ignored ; ce n’est pas un seul run.

R4 valide aussi les 51 unitaires152 renforcés. Les mêmes tests prévus
mesurent maintenant les 300 s de SC-002, puis le restant et les deux
événements exacts de SC-003. Leur ancien oracle à 300 ticks reste dans
l’historique ; aucune preuve de durée n’en est déduite.

L’E2E152 initialise g2 sans prompt et reste en diagnostic. T032 attend
la source finale Connected et sa vérification. Le principal rejette H1
de la revue hostile pour sa lecture trop large de FR-029 ; les contrôles
d’autorité149 stricts restent PASS. Les quatre autres objections sont
en contre-analyse, sans défaut reproduit par le reviewer.

46/50 reste inchangé. T032/T034/T036/T037 restent ouvertes.
Preuves, SHA, contre-revue et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Correctif terminal avant R5 — décisions et prototypes alignés

Les sources N/D gelées sont relues après GO du principal. Connected
retente le chemin de reprise existant après identité Fleet attestée et
gardes de connexion/instance/liberté. M2 propage la lecture de pause
illisible ; M3 réserve la conversion en échec à neuf refus nommés ;
M4 fige l’échéance depuis la seule demande durable, ou diffère.

Le booléen de R16 est désormais un `Result<bool, String>` interne.
T013/T044 gardent leurs scénarios prévus. Le prototype T015 a quatre
arguments ; l’enveloppe legacy T045 et l’origine d’échéance sont distinctes.
Plan, ADR et reuse-audit reflètent ces corrections au même contrat.

H1 est rejetée par le principal ; les contrôles stricts d’autorité149
sont conservés. M5 reste une réserve LOW, sans patch ni nouveau scénario.
R5 est en cours. Les anciennes captures ne sont ni remplacées ni requalifiées.
46/50 reste inchangé ; les quatre finitions restent ouvertes.

Preuves de lecture, SHA et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## R5 comparé, T034 relue, revue R2 et patch de demande absente

R5 donne295 PASS/0 FAIL/1 ignored sur sept filtres. Les six filtres de
compatibilité donnent239/0/1 comme la référence ; native152 donne51 unitaires
et5 E2E PASS. Les quatre adaptations149 effectives sont distinguées du
cinquième témoin byte-identique. Les sept défauts production149 corrigés
restent prouvés par leurs oracles stricts. T032/T034 sont clôturées :48/50.

Suite complète R1 :94 cibles,2103 PASS/8 FAIL/72 ignored, RC101 ; mêmes cinq
cibles et huit noms que W004151. Elle précède le patch None. Pas de succès
global ni d'addition à R5. T036/T037 restent ouvertes.

R2 conserve REQUEST_CHANGES. H1 est documentée : cible étrangère/CAS/stockage
peuvent bloquer le démarrage hors FR-029, sans assouplir149. La causalité L5
est réfutée par l'ordre du code : clôtures métier après tous arrêts confirmés.
Leur refus CAS peut laisser une clôture partielle, limite réelle conservée.

Le patch None posé réutilise close_lost_execution puis failed/request_unavailable
pour les deux branches après admission/working sans échéance. SQL Err ou
clôture refusée diffèrent sans mutation de tâche dans ces branches ; le cas
avant admission reste inchangé. R6 :48 PASS/3 FAIL, RC101, aucun E2E exécuté.
Les trois annulations attendent working et observent failed ; diagnostic
fixture/produit en cours, aucune adaptation d'oracle acquise.

Preuves, mappings, six réponsesXX et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md
et
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md.

## Rectification et correctif de pause avant R8

Cinq adaptations149 effectives incluent l'oracle global O4 de FR-029 ; le
comptage antérieur de quatre était incomplet. Le témoin supplémentaire reste
byte-identique. R3 résout H1/M2/M3/M4/L5 et révèle la pause ouverte après
request_unavailable. Une condition réutilise sa consommation après gardes.
R7 montre deux fixtures Working sans Request ; vrais IDs/délai désormais
admis par create_request, oracles inchangés. R8/098 attendus ;48/50 inchangé.
