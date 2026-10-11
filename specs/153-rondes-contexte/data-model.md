# Modèle de données - Session153

Une seule table nouvelle (index des missions passées). Aucun état nouveau. Les ajouts vivent dans la charge utile
JSON de la tâche (`delegation.rs`, struct `Task`). Tous sont facultatifs à la
lecture : un payload 149 ou 150 se relit sans erreur.

## Tâche native - champs ajoutés

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `round` | `u32` | 0 (lu comme 1) | Numéro de la ronde courante (1 à 8) |
| `idle_deadline_at` | `Option<i64>` | `None` | Garde posée : instant d'arrêt de l'enfant inactif |
| `round_body` | `Option<String>` | `None` | Corps de la mission de la ronde courante (brief + consigne). `None` : la consigne seule de `request.task` |
| `rounds` | `Vec<RoundRecord>` | vide | Rondes 2 et suivantes déjà ouvertes, pour le rejeu |
| `guard_released` | `bool` | `false` | Fin explicite durable : interdit toute pose de garde et toute ronde. Posée par l'annulation (agent ou humaine) d'une tâche `guarded` en `result_available` |
| `guarded` | `bool` | `false` | Vrai si la délégation porte `keep_alive_secs` (fixé à l'admission, jamais modifié) |
| `unacked_rounds` | `Vec<u32>` | vide | Rondes passées dont le résultat n'est pas accusé (au plus 7) |

`RoundRecord` : `{ round: u32, request_id: String, digest: String }`.
`digest` = SHA-256 hexadécimal des octets canoniques de la requête `Continue`.
Taille au plus 7 entrées (ronde 1 = la délégation). Environ 130 octets chacune.

Champs de `request` (enveloppe de la délégation, `protocol.rs`) :

| Champ | Type | Rôle |
|---|---|---|
| `keep_alive_secs` | `Option<u64>` | 60 à 7 200 ; absent = pas de garde |
| `brief` | `Option<Value>` | Brouillon de dossier de passation v1 |

Les deux sont omis de la sérialisation quand ils sont vides : les octets
canoniques d'un appel 150 ne changent pas. Le projet n'active pas directement la fonction
`preserve_order` de `serde_json` (aucune occurrence dans les `Cargo.toml`). Une
dépendance pourrait l'activer : la parité d'ordre n'est donc pas supposée. Le
rejeu exige seulement que le même objet reçu produise les mêmes octets (test
T005 : deux sérialisations du même brief sont identiques).

## Méthodes dérivées

| Méthode | Définition |
|---|---|
| `round_no()` | `max(round, 1)` |
| `result_message_id()` | ronde 1 : `native-result-<task_id>` ; ronde N >= 2 : `native-result-<task_id>-r<N>` |
| `holds_slot()` | `!terminal() \|\| (guarded && !cleanup_done)` |
| `continuable()` | `state == result_available && result_sent && idle_deadline_at.is_some() && !guard_released` |

## États et transitions

États inchangés (neuf valeurs, contrat 149).

| De | Vers | Déclencheur |
|---|---|---|
| `result_available` + garde posée | `mission_pending` | `bridget_task_continue` admis (`begin_round`) |
| `result_available` + garde posée | `result_available`, garde levée | annulation, arrêt humain, expiration, nettoyage du parent, redémarrage |
| `mission_pending` / `working` / `waiting_for_children` | comme en 149 | ronde N |
| `working` | `failed` | échéance, refus, perte de l'enfant : fin de tâche |

La garde se pose dans la même sauvegarde que `result_sent = true`
(sous le verrou d'état), si `request.keep_alive_secs` est présent.

## `begin_round(task_id, expected_round, ...)`

Transaction unique sur la ligne de la tâche :

1. relire la ligne ; refuser si `round_no() != expected_round`, si l'état n'est pas
   `result_available`, si la garde n'est pas posée ou si `now >= idle_deadline_at`
   ou si `guard_released` ; `now` est **relu par `begin_round` juste avant
   l'écriture** (paramètre d'horloge injectable pour les tests), pas repris de
   l'admission ;
2. insérer l'ancienne mission dans `native_delegation_past_missions` ; si
   `delivery_acked` est faux, ajouter `expected_round` à `unacked_rounds` ;
3. écrire : `round = expected + 1`, `mission = <nouvel UUID>`, `round_body`,
   `rounds += {round, request_id, digest}`, `result = None`, `result_sent = false`,
   `delivery_acked = false` (150), `error = None`, `mission_deadline_at = None`,
   `completed_at = None`, `idle_deadline_at = None`, `cleanup_done = false`,
   `state = "mission_pending"` ; remettre à zéro les champs par ronde ajoutés
   par 151 et 152 ;
4. incrémenter la projection (mutation visible) ;
5. signaler le propriétaire racine après le commit (`committed()`).

`started_at` reste celui de la ronde 1.

## Missions passées (nouvelle table d'index)

```sql
CREATE TABLE IF NOT EXISTS native_delegation_past_missions(
  mission TEXT PRIMARY KEY,
  task_id TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS native_delegation_past_missions_task
  ON native_delegation_past_missions(task_id);
```

Écrite par `begin_round` (au plus 7 lignes par tâche). Lue par `capture_reply`
quand `for_mission` ne trouve rien : lecture O(log n). Supprimée avec l'arbre par
la suppression atomique de SPEC-150 (par `task_id`). Une réponse authentifiée
(expéditeur = enfant, destinataire = parent, expéditeur autorisé) à une mission
passée est absorbée : `capture_reply` rend `true`, rien n'est écrit, aucun
dispatch.

## Preuve d'accusé par ronde

Purge 150 : `delivery_acked` (ronde courante) ET `unacked_rounds` vide. La
recopie de l'entretien (T010 de 150) lit la phase de chaque message
`result_message_id(task, r)` pour `r` dans `unacked_rounds` et retire les rondes
accusées. La recopie précède `purge_expired_sends`, comme en 150.

## Pose de la garde

Dans `tick`, la sauvegarde qui pose `result_sent = true` pose aussi
`idle_deadline_at = now + keep_alive_secs` quand la tâche est `guarded` et que
`guard_released` est faux. Aucune
fonction partagée : `tick` est le seul écrivain de la remise (SPEC-151 : l'attente
et la lecture d'état sont des lectures pures). Tant que `tick` n'a pas envoyé le
message, `continuable` est faux et `bridget_task_continue` rend
`result_delivery_pending` (réessayable).

## Identifiants de message

| Message | Identifiant | Remarque |
|---|---|---|
| Mission, ronde N | `task.mission` (UUID neuf par ronde) | clé idempotente de l'envoi de mission |
| Résultat, ronde 1 | `native-result-<task_id>` | octets 149 |
| Résultat, ronde N >= 2 | `native-result-<task_id>-r<N>` | `in_reply_to` = mission de la ronde |
| Avis d'échec | `native-failure-<task_id>` | inchangé ; une seule fois par tâche |

## Rétention et purge (SPEC-150)

Le prédicat de purge d'un arbre exige `cleanup_done`, aucun enfant vivant et
`unacked_rounds` vide. Il ajoute `idle_deadline_at.is_none()` comme ceinture. Le
comptage des places (128 au total, 16 par instance) utilise `holds_slot()` : la
place reste tenue tant que le nettoyage d'une tâche avec garde n'est pas confirmé.
En SQL, `guarded` est un champ de la charge utile (pas de chemin dans `request`).
