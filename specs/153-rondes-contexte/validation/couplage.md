# Couplage T001153 — code150/151/152 livré

Base WT153 : ee03ebdcfbba85028f0ba3962dde439919b48d29.
Les200 empreintes Rust/Cargo.lock correspondent à la référence brute T001.
Capture T001 historique : les ajouts153 étaient absents avant le GO production.

## Task de la base152 :33 champs (capture T001)

Source : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:49. Les sept champs153 étaient alors à ajouter ; voir la carte finale ci-dessous.

| Champ existant | Portée pour begin_round |
|---|---|
| task_id | Vie entière : conservé par begin_round |
| root_owner_agent_id | Vie entière : conservé par begin_round |
| parent_task_id | Vie entière : conservé par begin_round |
| updated_at | Avance avec la nouvelle sauvegarde |
| started_at | Vie entière : conservé par begin_round |
| completed_at | None à ouverture, fin courante ensuite |
| owner | Vie entière : conservé par begin_round |
| owner_instance | Vie entière : conservé par begin_round |
| origin_owner_instance | Vie entière : conservé par begin_round |
| parent_execution_id | Vie entière : conservé par begin_round |
| request | Vie entière : conservé par begin_round |
| permission_snapshot | Vie entière : conservé par begin_round |
| effective_posture | Vie entière : conservé par begin_round |
| definition | Vie entière : conservé par begin_round |
| cwd | Vie entière : conservé par begin_round |
| child | Vie entière : conservé par begin_round |
| child_instance | Vie entière : conservé par begin_round |
| mission | UUID neuf de ronde |
| mission_deadline_at | None à ouverture |
| created_at | Vie entière : conservé par begin_round |
| state | Reprend mission_pending |
| result | None à ouverture |
| error | None à ouverture |
| result_sent | false à ouverture |
| cleanup_done | false à ouverture |
| failure_sent | false à ouverture |
| reply_timeout | Vie entière : conservé par begin_round |
| delivery_acked | false, preuve ancienne retenue séparément |
| acknowledged_at | None, accusé de lecture151 propre au résultat |
| launch_issued_at | Vie entière : conservé par begin_round |
| mission_issued_at | None, enveloppe du premier envoi de nouvelle mission |
| restart_resumes | Vie entière : conservé par begin_round |
| cancel_stopped | Vie entière : conservé par begin_round |

`restart_resumes` reste borné2 sur toute la vie de Task. `launch_issued_at` et
`started_at` restent initiaux. `cancel_stopped` et toutes les preuves
d'identité, de filiation et de sécurité sont conservés. Les resets précis sont
I5 du plan ; `round_body` contient toujours la consigne nouvelle sur Continue.

## Couplages et décisions de branchement

| Preuve absolue actuelle | Branché par / décision |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:341 admission_counts ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:127 index state | T003/T012, même index à quatre clés, migration Immediate gardée ; SQL actifs OU terminaux guarded1/cleanup0, O(log n+a), a≤128 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:193 purge_trees ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:223 delete_tree | T012/T029, garde absente et ACK de chaque ronde, missions passées supprimées atomiquement |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:264 acknowledge_tasks | T004, reset accusé de lecture151, distinct de livraison |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:269 save_projected | T004, transaction dédiée (dates/ACK anciens conservés ici), wait_epoch.bump + committed après commit |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:195 current_task_view | T016/T018, ID de ronde courante pour la phase boîte151 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1144 authorize_retained_result | T016, autorisation de l'ID exact courant |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1924 tick envoi du résultat | T010/T016, ID courant ; tick seul pose garde avec remise |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:2419 copy_delivery_acks | T029, courant non accusé OU anciennes rondes, avant purge_expired_sends |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:615 wait_tasks | T018, current_task_view ; lecture pure, aucun cinquième constructeur d'ID |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1246 capture_reply ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:9160 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon.rs:18129 | T028, deux chemins, lookup missions passées indexé, identité pure sans recover_child |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:995 native_launch_registry | T014, autorité et sources existantes ; legacy grant/cwd obligatoires sans snapshot |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:112 parent_fact ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permissions.rs:206 child_policy | T014, politique actuelle entière ; posture demandée None préservée |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1488 prepare_restart | T011, lever garde durable, reprise active152, compteur2 vie entière |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1018 native_child_env | Sans nouvelle primitive ; montage152 déjà partagé |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/referent_control.rs:330 pause_overlap | T015, garde NativeDelegation/journal, dates métier brutes intactes, suspended dérivé |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/execution_store.rs:1024 latest_in_chain | Sans nouvel algorithme ; chaîne de reprise152 réutilisée |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-transport/src/protocol.rs:2717 deserialize_present_optional | T002/T005, omission None, brief:null Some(Null) refusé après vraie désérialisation |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/handoff.rs:460 validate_and_render | T006, même rendu et borne16384 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/mcp.rs:2196 schéma handoff | T017, Value aux deux adaptateurs, pas de copie/helper neuf |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/wrapper.rs:5035 liste sûre | T017,23→24 ; catalogue natif6→7/global27→28 |

`task_view` pur reste distinct de l'enrichissement `current_task_view`.
Aucun champ suspended_at existe. Journal pause152 : échéance métier gelée,
dates brutes conservées ; minuterie d'appelant RequestWait fondée sur Instant
et garde terminale153 inchangées. Les gardes identité/CAS/stockage restent strictes.

Levée de garde : guard_released=true et idleNone, même dans cancel_tree/
prepare_restart encore actifs. Refus terminal avant result_delivery_pending.
Présence d'enfant : route live, conn_instances=child_instance, Desired Running
de la commande native attestée. Réponse tardive : aucune recover_child/save.
tree_settled exige unacked vide hors de l'alternative cancelled/delivery_acked.
Seule écriture d'un refus : fin de garde déjà expirée/enfant absent, après
propriété confirmée ; aucune nouvelle ronde, mission ou clé.


## Raccordement observateur interactif — FR-031 / T017

L'observateur natif existant de Claude atteste chaque Continue, comme Delegate.
Matcher PreToolUse dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer.rs:45,
liste acceptée à74, request_id obligatoire à82, fait courant corrélé à90.
Sources/session/nonce/pair/cwd/overlay restent contrôlés. Aucun outil, helper
ou observateur neuf ; comparaison de politique complète inchangée.

Les dix tests existants, dont trois ajustés dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/native_permission_observer149_tests.rs,
passent au lot diagnostic-et-observer R1. Cas206 et282 : Delegate/Continue
et absence de request_id ; cas178 : RAII/matcher. Preuve locale du hook,
fait et ACK ; aucune recette de modèle réel. T017 attend l'intégration T025.


## Finding T026 — checkpoint historique avant correction FR-019

Lecture du principal :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1055.
recover_owner peut sauvegarder owner_instance avant un refus ou un rejeu
Continue, si le même owner utilise une nouvelle instance et l'ancienne est
absente. FR-019 interdit cette mutation hors fin durable de garde admise.
Le constat est confirmé par lecture ; correctif et preuve ciblée restent
à produire après le workspace gelé en cours. Ne pas transformer cette limite
en exception au contrat. T015 rouverte en phase Implement : checkpoint19/29.
T017 reste aussi ouverte : oracle MCP de catalogue27 au lieu de28 en diagnostic.
Aucun statut Implemented ou Converged ; aucun second workspace demandé.

## Carte finale T026 — 29/29

Task :40 champs, dont sept153 (round, idle_deadline_at, round_body, rounds,
guarded, guard_released, unacked_rounds). Task neuve round=1 ; legacy0 accepté,
round_no=1. begin_round remet à vide les champs de ronde I5, dont completed_at/
acknowledged_at/mission_issued_at ; conserve identité, droits, restart_resumes,
launch_issued_at, started_at et cancel_stopped. Aucun suspended_at ajouté.

| Raccordement final | Source |
|---|---|
| Places/index existant | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation.rs:416 ; deux INDEXED BY, O(log n+a), a≤128 |
| Purge/anciens ACK/historique | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:35 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:230 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:2669 |
| Quatre ID courants | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:201 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1347 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1787 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:2669 |
| CAS/horloge fraîche/commit | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/delegation_lineage.rs:323 |
| Garde/reprise/pause152 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:1698 ; pause_overlap/latest_in_chain réutilisés |
| FR-019 pur | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:277 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/153-rondes-contexte/crates/bridget-daemon/src/daemon/native_delegation.rs:467 |

FR-019 corrigé : contrôle de propriété pur et comparaison WaitEpoch sous même
verrou ; seul un nouveau begin_round committé entraîne tick. Aucun payload,
owner_instance ou epoch modifié sur refus/rejeu. Réconciliation de tick conservée
après admission. Deux RED réels R2 puis GREEN final40+1.
T017 : catalogue28/safe24, matrice15 PASS R2, dix observer149 et intégration
Claude workspace PASS. Gardes de politique complète, cible/CAS/stockage inchangées.
Statut/TaskWait consultent l’ID courant ; pureté testée sur current_task_view
commun à Status, sans prétendre une trame Status envoyée ; vrai TaskWait traversé.
