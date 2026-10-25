# Contrat - Reprise sur limite d'usage (160)

Compatible avec les contrats 148, 149, 150, 152 et 157
(`specs/157-catalogue-quotas/contracts/delegation157.md`). Un appel sans
`resume_on_limit` produit les mêmes octets canoniques et le même comportement
qu'avant.

## `bridget_delegate` - champ ajouté

| Champ | Type | Obligatoire | Défaut |
|---|---|---|---|
| `resume_on_limit` | booléen | non | absent = inactif |

Le champ fait partie de l'enveloppe idempotente. Absent et `false` explicite sont
deux enveloppes différentes au rejeu (`envelope_mismatch`). Les deux inactivent la
fonction ; seul `true` l'active.

## `bridget_capabilities` - clé ajoutée

`features.resume_on_limit` : `true`. `features.resume_max_wait_secs` : `21600`.
`features.resume_max_rounds` : `2`.

## Réponse d'une tâche - champ ajouté

`quota_wait` : `null`, ou `{round, phase, resumes_at}`.

| Phase | Sens |
|---|---|
| `waiting` | la tâche attend l'heure de retour ; `resumes_at = retour + 30 s` |
| `relaunching` | la relance est en cours |

La tâche garde l'état `working`. Une fois l'attente soldée, `quota_wait` vaut
`null`. Les rondes faites restent comptées dans la tâche (pas dans la vue).

## Erreur de tâche ajoutée (champ `error` d'une tâche `failed`)

| Code | Quand | Détail |
|---|---|---|
| `usage_limit` | épuisement attesté mais reprise impossible : retour à plus de 6 h, retour passé, ou 2 rondes déjà faites | `limit_reset_at` : instant de retour |
| `native_relaunch_failed` | la remise du message de la relance est incertaine ou perdue (`indeterminate`, `orphaned`), ou non reprenable (enveloppe différente, remise invalide) | aucun ; échec explicite avec l'avis durable |
| `native_refusal_uncorrelated` | refus reçu sans identifiant d'exécution après au moins une relance : il ne peut pas être rattaché à l'exécution courante | aucun ; échec explicite avec l'avis durable |

Les erreurs de 152 restent : `mission_reply_timeout` (aussi rendu si l'enfant
reste absent jusqu'à l'échéance effective), `native_resume_exhausted`,
`unreachable`, `native_restart_identity_mismatch`.

## Garanties de comportement

| Situation | Comportement |
|---|---|
| Échec d'une tâche opt-in, fait qualifiant connu | attente immédiate, aucun avis d'échec, suivi conservé |
| Échec d'une tâche opt-in, aucun fait | décision en attente 5 s (politique bornée, pas une garantie d'ordre) ; ensuite échec avec l'avis d'échec durable (`native-failure-<task_id>`) et retrait du suivi. Le message direct « Échec de livraison » n'est pas rejoué |
| Pendant la décision en attente | échéance et rappels inhibés ; résolution avant le saut de pause ; indépendante du parent |
| Refus ancien après une relance | sans effet si son identifiant d'exécution ou sa génération diffère de la feuille de chaîne (`StaleIgnored`), même quand la nouvelle feuille est déjà `failed` |
| Refus sans corrélation après une relance | échec explicite `native_refusal_uncorrelated` |
| Injection synchrone refusée, réponse vide | corrélation d’abord ; absents après relance = native_refusal_uncorrelated ; sinon erreur d’origine si la feuille n’est pas failed |
| Tout échec d'une tâche opt-in | un seul avis, l'avis d'échec durable ; aucun message direct |
| Réponse vide sans DeliveryRejected | attente du timeout de mission existant, aucun refus inventé |
| Reprise ordinaire de 152 | refusée pour toute chaîne dont l'entrée de quota est ouverte |
| Fait qualifiant arrivé dans les 5 s | attente |
| Pause du référent | cause de pause (152) prioritaire ; relance différée pendant une pause |
| Annulation pendant l'attente | exécutée ; l'entrée est fermée ; aucune relance |
| Annulation pendant `relaunching` | réconciliation de la continuation et de la remise, puis fermeture ; sinon `cancelling` avec `native_cancel_unverifiable:quota_relaunch` |
| Redémarrage du daemon pendant l'attente | tâche gardée, enfant non reconstruit avant l'heure, `restart_resumes` inchangé |
| Redémarrage pendant la décision en attente | échec normal avec avis même si un fait était vu en mémoire volatile avant arrêt |
| Enfant absent à l'heure de retour | réessai à chaque tick ; le gel quota s'arrête à `retour + 30 s`, puis `mission_reply_timeout` à l'échéance |
| Troisième échec de quota | `usage_limit` avec `limit_reset_at` ; cause Past ou RoundsDone vérifiée séparément par les oracles existants |

La borne porte sur l’observation du fait : `observed_at <= quota_hold.since + 5`. Un tick retardé peut résoudre le hold plus tard, mais ne rend jamais admissible un fait observé après cette borne. La classification emploie `now` réel au tick de décision, pas `hold.since+5` comme horloge fictive. La borne observée s’applique à chaque fenêtre avant sélection : observed_at<=since+5. Un résultat différent quand now change est prévu ; un fait observé à10:42 pour un hold commencé à10:40 dépasse5s et est exclu.

## Trame wrapper `DeliveryRejected` (compatibilité)

`DeliveryRejected { id, reason, execution_id?, generation? }`. Les deux champs
sont optionnels et omis s'ils sont absents.

| Daemon | Wrapper | Effet |
|---|---|---|
| nouveau | nouveau | corrélation complète |
| nouveau | ancien | champs absents : refus courant tant que la tâche n'a jamais été relancée, sinon `native_refusal_uncorrelated` |
| ancien | nouveau | champs ignorés (`WrapperToDaemon` sans `deny_unknown_fields`) ; comportement actuel |

Le wrapper capture les champs depuis sa liaison AVANT la transition terminale qui la retire (refus asynchrone, réponse vide, refus de pause). Injection idempotente synchrone : liaison présente si le contexte d’exécution est fourni, et absence de transition failed. DeliverExecution synchrone : liaison avant injection puis transition failed sur erreur ; capturer avant son retrait. Sans contexte/lien, champs absents.

## Identité et rejeu de la relance

Identité : `limit-resume:<task_id>:<ronde>:<retour>`. Elle est unique dans le
magasin d'exécutions et reliée à la continuation (`usage_limit_reset`). Rejouer la
relance rend la continuation existante, puis reprend la remise à l'état trouvé
(réservation absente, réservée, gravée, accusée). La date d'émission du message
est figée (`observed_at` initial de la continuation), conservée et relue sans réécriture au rejeu. Le solde n'a lieu qu'à partir de
« remise accusée » (`acked`). Une remise `indeterminate` ou `orphaned` donne
`native_relaunch_failed`.

## Primitives de 152 et de 157 utilisées

| Primitive | Source |
|---|---|
| `latest_in_chain` (feuille de plus grande génération) | 152 T010 |
| `AutonomousEffect::NativeMissionDispatch` | 152 T009, T019 |
| `rejected()` qui qualifie la pause, sortie du gestionnaire `DeliveryRejected` | 152 T013, T044 |
| `pause_overlap` (journal des pauses `control_events`) | 152 T050 |
| `effective_deadline`, `frozen_secs` du suivi de demande | 152 T017, T018 |
| `restart_resumes`, décision de reprise de `prepare_restart` | 152 T008, T023, T025 |
| `native_child_env`, `current_command` | 152 T022, T043 |
| `provider_limits`, `Quota`, épuisement attesté par fenêtre | 157 T004, T014 |

## Lineage adaptateur de bureau

Aucun changement de forme. La tâche en attente apparaît `working`.

## Précisions retenues R1

La corrélation FR006b précède toute classification FR006c/quota. Après une relance, des champs absents donnent donc `native_refusal_uncorrelated`, même pour un refus synchrone. Une réponse vide termine l’Execution en `completed`, tandis que la Task demeure `working` jusqu’au refus : elle échoue avec l’erreur d’origine si le refus est corrélé. Les Task déjà `result_available` ou `cancelled` sont préservées ; une Task déjà `failed` ne reçoit pas de nouvel avis.

Lecture160 conservatrice : `attested_exhausted_since(type,since,now,past_since:Option<i64>)` garde la preuve enregistrée par157 et son admission inchangée. Observation strictement après `since`, puis reset futur OU (`past_since=hold.since` et observation>=past_since). Pendant le hold, filtrer chaque fenêtre par `observed_at <= past_since + 5` avant toute agrégation de reset/observation. Hors résolution du hold, past_since=None ; ancienne observation expirée=NoFact. Le cas fait observé pendant le hold puis expiré à la décision donne `Past{reset_at}`/usage_limit. Une fenêtre rejetée à son enregistrement n’est jamais inventée.

Toute sortie d’échec depuis `relaunching` réconcilie la feuille et la remise avant Task.failed/cleanup, en réemployant `close_lost_execution` et T024 : cible child vérifiée, CAS de la feuille vers `unreachable`, remise dispatching mise en quarantaine. Une erreur de clôture se propage ; l’entrée reste `relaunching` pour réessayer le settlement. Aucun nouvel arrêt physique. Les résultats reçus pour Task.failed sont déjà ignorés par capture_reply (ACK transport seulement). Cela ne garantit pas l’unicité des effets externes.

FR003 est une politique de corrélation temporelle, pas une preuve que le quota cause cet échec. Un fait admissible antérieur à l’échec peut qualifier une panne sans rapport et faire répéter des effets de la mission. L’opt-in, les bornes6h/2rondes et l’idempotence de la relance ne garantissent ni causalité ni exactly-once des effets externes. Aucune fenêtre W60 ou test supplémentaire n’est ajouté.

R2 : waiting suit la garde retour+30 ; relaunching se reprend à chaque tick sans nouveau gate horaire, avec pause/échéance conservées. Si Task.save échoue après qualification, réutiliser sa même identity/reset pour finir le même intent Task ; ne jamais reconstruire sans TaskQuotaWait. Si DB indisponible, conserver checkpoint et propager l’erreur pour retry, sans promettre de fermer le hold. Past ne concerne que la preuve admissible observée pendant le hold ; ancienne observation périmée=NoFact.
