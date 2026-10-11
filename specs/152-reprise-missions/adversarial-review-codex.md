# Contre-revue adverse - Session152

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : réalisabilité de la reprise (panne et pause), atomicité, gel des
  horloges, annulation en cascade, reçus de refus.
- **Verdict reçu** : BLOCKED
- **Note de l'agent** : objections 150 n° 2 et 7 bien fermées ; n° 1, 3, 4
  partiellement ouvertes ; sauvegarde unique du décalage jugée correcte.
- **Méthode de vérification** : chaque objection est une hypothèse. Elle est
  relue dans le checkout principal `/Users/moi/Nextcloud/10.Scripts/64.bridget`
  (base `1f2c661d`) avant d'être retenue.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | CRITIQUE : l'enfant déjà connecté n'est pas couvert ; la flotte crée une commande sans filiation | lu `daemon.rs:3910-4020` : `recovery_candidates()` ne couvre que les lancements en cours ; la boucle `desired.equipiers` crée `command_id: recovery-<uuid>` et `ownership: None` ; lu `native_delegation.rs:43-45`, `:462-469`, `:850`, `:857`, `:868`, `:890` : tous liés à `task_id` ; lu `fleet.rs:900-915` et `:1649` (`ownership_from_desired` existe) | oui | confirmée. Le second redémarrage ferait échouer `restart_identities` pour toutes les tâches. R13, T024 en deux chemins, T043, T004 étendu (deux redémarrages, imbriquée) |
| 2 | Compteur de reprises non atomique | lu `execution_store.rs:1443` : `tx.commit()` avant toute écriture de la tâche, qui vit dans un autre magasin ; la reconstruction écrit déjà la continuation et `resume_execution_id` (`:1422-1433`) | oui | confirmée. Solution plus simple que la correction demandée : compte dérivé de la chaîne, aucune écriture séparée, donc atomique par construction. R14, T010, T025 |
| 3 | Panne pendant une pause : échéance brute avant décalage ; `interrupting` sans terminal | lu `native_delegation.rs:1013-1023` (échéance brute) et le plan T023 ; lu le commentaire `:940` (groupes réconciliés avant `prepare_restart`) ; lu `execution_store.rs:1396` (`interrupting` accepté mais fermé en `unreachable`) | oui | confirmée. `settle_suspension` partagé, `interrupting` réconcilié en `interrupted`. R15, T041, T042, T038 |
| 4 | Chemin A encore visible : le gestionnaire général notifie et retire le suivi | lu `daemon.rs:18078-18125` : après `rejected()`, `pending_replies.retain`, `get_request`, puis `deliver_to_agent("Échec de livraison de la demande ...")` | oui | confirmée. `rejected()` rend `pause_interruption`, le gestionnaire sort avant tout effet. R16, T044, T002/T003 étendus à l'événement complet |
| 5 | `latest_in_chain` peut choisir une ancienne continuation ; borne de chaîne fausse | lu `execution_store.rs:1402` (génération `parent + 1`) et `created_at` en secondes ; mon tri `created_at, execution_id` dépendait d'un UUID aléatoire ; les pauses ne comptent pas dans la borne de 2 | oui | confirmée. Feuille de plus grande `generation`, complexité O(c) sans borne constante. R17, T010 |
| 6 | Gel du suivi trop large : pauses antérieures à la demande soustraites | lu `daemon.rs:8535-8560` et `:8816-8824` : la demande naît à l'envoi de la mission (après `queued`) ; lu `daemon.rs:3860-3866` (temps écoulé recalculé à la reconnexion) | oui | confirmée, avec un écart voisin trouvé : l'envoi utilise `issued_at = task.created_at` (`native_delegation.rs:1148-1156`), donc l'échéance du suivi serait déjà passée après une pause en `queued`. `request_frozen_secs` (pause en `working` seulement) et `mission_issued_at`. R19, T017, T018, T039, T045 |
| 7 | Contrôle global d'annulation contournable au tick suivant | lu `native_delegation.rs:1390-1394` (descendants marqués `cancelling` avant les arrêts) et `:994-996` (chaque `cancelling` devient une racine d'annulation) | oui | confirmée. Contrôle avant toute mutation, `tick` sur les racines seulement, reprise coordonnée par `cancel_stopped`. R20, T027, T046, T040 |
| 8 | Liste fermée des reçus de refus indéterminée ; texte dynamique | lu `registry.rs:157` (texte avec chemin et liste des types) et `:326`, `:328` (`model_unavailable`, `effort_unavailable` stables) | oui | confirmée. Liste exacte de cinq codes ; le texte de `registry.get` est ramené à `agent_type_unknown`, jamais stocké. R18, T029 |
| 9 | Couverture déclarée trop forte ; conservation des états non repris non testée | relu `analyze-report.md` et `tasks.md` T023 : aucun test de conservation ; scénarios rouges manquants | oui | confirmée. Trois tests de conservation dans T023, rouges T038 à T040, rapport d'analyse corrigé. R21 |

**Corrections faites au passage**
- `resume_attempts` n'est plus un champ de la tâche. C'est une valeur dérivée (objection 2).
- `suspended_total_secs` devient `request_frozen_secs` (objection 6).
- Trois champs ajoutés : `mission_issued_at`, `fleet_command_id` (et `launch_issued_at` existait déjà).
- Tâches : 37 devient 46 (T038 à T046 ajoutées dans leurs phases).

**Restant ouvert**
- Le lien de flotte avec un parent d'instance disparue (mission imbriquée) est-il accepté par la réservation ? À prouver par T004 (d).
- La date de levée de pause lue dans `control_events` n'a pas été vérifiée en détail (colonnes `at`, `kind` lues dans le schéma seulement). T041 le prouve.
- Le recalcul de `timeout_secs` à la reconnexion (`deadline_at - created_at`) reste fragile hors pause quand l'envoi est tardif ; T045 le corrige seulement pour les missions envoyées après la mise à jour.

Pas de ronde 2 demandée dans cette session : les neuf objections sont retenues et traitées dans les artefacts.

## Ronde 2 (plan révisé)

- **Agent** : Codex GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 4, 5, 7, 8, 9 fermées ; 1, 3, 6 partiellement
  ; 2 : compte atomique pour une chaîne existante mais borne incomplète.
  Quatre points restent, tous aux frontières de crash.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | Borne inopérante avant l'envoi de mission | lu `native_delegation.rs:608` (`spawn_task` pose `starting` sans exécution) et `execution_store.rs:1298-1440` (une continuation naît seulement d'une exécution existante) | oui | le compte dérivé reste à 0 en `starting` et `mission_pending`. `restart_resumes` incrémenté à la décision de reprise, sauvegardé avant toute reprise : il ne peut que sur-compter. R22, T023, T025, T047 |
| 2 | CRITIQUE : commande, instance et entrée de flotte changent séparément ; démarrage bloqué | lu `native_delegation.rs:845-868` (`Err` sur tout écart) et `daemon.rs:4553-4555` (`restart_identities(&st)?` dans le démarrage) | oui | confirmée : un seul écart empêche le daemon de démarrer. Choix **échec explicite** : pas de champ `fleet_command_id` (commande dérivée du lien attesté), réconciliation limitée à ce que le lien atteste, sinon tâche `failed` nommée et démarrage continué. R24, T043, T048 |
| 3 | Pause durable sans `suspended_at` | lu `daemon.rs:9598-9626` (`set` puis `interrupt_executions_for_pause`, deux étapes) ; lu `referent_control.rs:162-166` (`control_events.at`, `kind`) ; les tests de `referent_control.rs` montrent `pause_on` et `pause_off` | oui | confirmée. Le temps de pause devient une fonction du journal (`pause_overlap`) : aucune écriture à perdre. `suspended_at` et `settle_suspension` sont supprimés. R25, T050, T017, T041, T049 |
| 4 | `working` ne prouve pas le début du suivi | lu `daemon.rs:8816-8824` (ligne de suivi créée dans la transaction d'admission) et `native_delegation.rs:1154`, `:1188` (état `working` écrit après) | oui | confirmée. Le début vient de la ligne durable `tracked_requests`. R23, T018, T049 |

**Choix reprise ou échec explicite**
- Reprise conservée pour : état `starting`, `mission_pending`, `working`, pause, panne.
- Échec explicite choisi pour : identité irréconciliable (R24). Raison (Art. XIX) :
  modéliser chaque ordre de sauvegarde est disproportionné, et un daemon qui ne
  démarre pas coûte plus qu'une tâche `failed` nommée.

**Simplification obtenue** : quatre champs ou fonctions retirés (`suspended_at`,
`request_frozen_secs`, `fleet_command_id`, `crash_resume_count`), une fonction
ajoutée (`pause_overlap`). Tâches : 46 devient 50 (T047 à T050).
