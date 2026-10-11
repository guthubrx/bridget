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
| Injection synchrone refusée, réponse vide | échec avec l'erreur d'origine, sans reprise (la feuille n'est pas `failed`) |
| Tout échec d'une tâche opt-in | un seul avis, l'avis d'échec durable ; aucun message direct |
| Reprise ordinaire de 152 | refusée pour toute chaîne dont l'entrée de quota est ouverte |
| Fait qualifiant arrivé dans les 5 s | attente |
| Pause du référent | cause de pause (152) prioritaire ; relance différée pendant une pause |
| Annulation pendant l'attente | exécutée ; l'entrée est fermée ; aucune relance |
| Annulation pendant `relaunching` | réconciliation de la continuation et de la remise, puis fermeture ; sinon `cancelling` avec `native_cancel_unverifiable:quota_relaunch` |
| Redémarrage du daemon pendant l'attente | tâche gardée, enfant non reconstruit avant l'heure, `restart_resumes` inchangé |
| Redémarrage pendant la décision en attente | échec normal avec avis |
| Enfant absent à l'heure de retour | réessai à chaque tick ; le gel quota s'arrête à `retour + 30 s`, puis `mission_reply_timeout` à l'échéance |
| Troisième échec de quota | `usage_limit` avec `limit_reset_at` |

## Trame wrapper `DeliveryRejected` (compatibilité)

`DeliveryRejected { id, reason, execution_id?, generation? }`. Les deux champs
sont optionnels et omis s'ils sont absents.

| Daemon | Wrapper | Effet |
|---|---|---|
| nouveau | nouveau | corrélation complète |
| nouveau | ancien | champs absents : refus courant tant que la tâche n'a jamais été relancée, sinon `native_refusal_uncorrelated` |
| ancien | nouveau | champs ignorés (`WrapperToDaemon` sans `deny_unknown_fields`) ; comportement actuel |

Le wrapper remplit les champs depuis sa liaison d'exécution quand elle existe
(refus asynchrone, réponse vide, refus de pause). Le refus d'une injection
synchrone peut ne pas avoir de liaison : champs absents.

## Identité et rejeu de la relance

Identité : `limit-resume:<task_id>:<ronde>:<retour>`. Elle est unique dans le
magasin d'exécutions et reliée à la continuation (`usage_limit_reset`). Rejouer la
relance rend la continuation existante, puis reprend la remise à l'état trouvé
(réservation absente, réservée, gravée, accusée). La date d'émission du message
est figée (observation de la continuation). Le solde n'a lieu qu'à partir de
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

## Lineage T3

Aucun changement de forme. La tâche en attente apparaît `working`.
