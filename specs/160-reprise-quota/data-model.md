# Modèle de données - Session160

## Requête `Delegate` (`bridget-transport/src/protocol.rs`)

| Champ | Type | Sérialisation |
|---|---|---|
| `resume_on_limit` | `Option<bool>` | omis seulement si `None` |

`None` = fonction inactive. `Some(false)` explicite est sérialisé : c'est une
autre enveloppe, donc `envelope_mismatch` au rejeu d'une même clé. Les octets
canoniques d'un appel sans ce champ sont inchangés.

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

Champs ajoutés, optionnels, pour garder les payloads 148 à 157 lisibles :

| Champ | Type | Défaut | Rôle |
|---|---|---|---|
| `quota_waits` | `Vec<QuotaWait>` | vide | Attentes de quota. On ajoute, on ne retire jamais. Sa longueur est le nombre de rondes |
| `quota_hold` | `Option<QuotaHold>` | `None` | Échec non encore classé, 5 s au plus |

```
QuotaWait { since: i64,            // début de l'attente (instant de la qualification)
            reset_at: i64,         // heure de retour du quota
            round: u8,             // 1 ou 2
            identity: String,      // limit-resume:<task_id>:<round>:<reset_at>
            execution_id: String,  // exécution échouée qualifiée
            phase: "waiting" | "relaunching" | "done",   // done sans resume_execution_id : annulée
            until: Option<i64>,    // fin de l'attente, posée au solde
            resume_execution_id: Option<String> }
QuotaHold { since: i64, error: String, execution_id: String }
```

Valeurs dérivées, jamais stockées :

- **rondes faites** : `quota_waits.len()`.
- **en attente** : la dernière entrée a la phase `waiting` ou `relaunching`.
- **instant de relance** : `reset_at + 30 s`.
- **intervalle de gel d'une entrée** : `[since, min(until ou maintenant,
  reset_at + 30 s)]`. Le gel quota s'arrête donc à l'heure de retour plus 30 s,
  quelle que soit la connexion de l'enfant. Après ce point, l'échéance effective
  court de nouveau et `mission_reply_timeout` peut expirer. Le gel de pause reste
  distinct et dure autant que la pause.
- **hold** : tant que `quota_hold` existe, le tick saute l'expiration et les
  rappels de la tâche. Le hold est borné à 5 s par sa résolution, qui passe avant
  la pause dans `tick`.

La borne porte sur l’observation du fait : `observed_at <= quota_hold.since + 5`. Un tick retardé peut résoudre le hold plus tard, mais ne rend jamais admissible un fait observé après cette borne. La classification emploie `now` réel au tick de décision, pas `hold.since+5` comme horloge fictive. La borne observée s’applique à chaque fenêtre avant sélection : observed_at<=since+5. Un résultat différent quand now change est prévu ; un fait observé à10:42 pour un hold commencé à10:40 dépasse5s et est exclu.

### Invariants

- I1 : `quota_waits` est append-only. Aucune écriture ne retire une entrée ni
  ne baisse `round`.
- I2 : au plus une entrée n'est pas `done`. Une entrée n'est ajoutée que si la
  longueur est inférieure à 2.
- I3 : la qualification de l'exécution (table ci-dessous) est écrite AVANT
  l'entrée de tâche. Si cette sauvegarde échoue, relire la qualification persistée et finir le même intent Task avec la même identity/reset ; aucune reconstruction sans TaskQuotaWait. Stockage indisponible : checkpoint conservé, erreur propagée, retry ; aucun hold déclaré fermé sans écriture.
- I4 : `until` est posé une seule fois, au passage à `done`.
- I5 : `quota_hold` et une entrée non `done` n'existent jamais ensemble. Le
  passage du hold à l'attente est une seule sauvegarde, et `since` de l'entrée
  reprend `since` du hold (l'attente commence à l'instant de l'échec).
- I10 : une chaîne dont l'entrée de quota est ouverte n'est jamais reprise par la
  reprise ordinaire de 152 (`may_resume` refuse).
- I11 : le message de la relance a une date d'émission figée, égale à
  `observed_at` initial de la continuation, conservé puis relu sans réécriture au rejeu. Le canonique est identique à chaque rejeu.
- I6 : `restart_resumes` de 152 n'est jamais modifié par ce mécanisme.
- I7 : `mission_deadline_at` n'est jamais réécrite (152 I2). Le gel passe par
  l'échéance effective.
- I9 : une entrée `done` sans `resume_execution_id` a été fermée par une
  annulation. Aucune relance n'a eu lieu.
- I8 : `agent_type`, `model`, `effort` et la définition épinglée de la tâche ne
  changent pas.

## Qualification d'une exécution échouée (magasin d'exécutions)

```sql
CREATE TABLE IF NOT EXISTS execution_quota_resumes(
  execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id),
  identity TEXT NOT NULL UNIQUE,
  reset_at INTEGER NOT NULL,
  qualified_at INTEGER NOT NULL,
  resume_execution_id TEXT REFERENCES executions(execution_id)
);
```

- Migration numérotée (prochaine version, après `execution_store.rs:1619`).
- `resume_execution_id` est posé dans la transaction qui crée la continuation.
- `execution_continuations` ne change pas : la continuation a `mode =
  'reconstructed'` et `reason = 'usage_limit_reset'`.

## Reconstruction ciblée (`execution_store.rs`)

`reconstruct_quota_failed(parent_execution_id, identity, agent_instance_id,
child_execution_id, observed_at) -> QuotaRecoveryOutcome`

Une transaction :

1. lire la ligne de qualification du parent ; absente ou d'une autre identité :
   `NotQualified` ;
2. si `resume_execution_id` est posé : garder `AlreadyResumed(id)` (rejeu), puis relire snapshot/message/continuation existants et son `observed_at` initial, sans DTO supplémentaire ni upsert ;
3. vérifier que le parent est `failed` et feuille de sa chaîne : sinon
   `NotLeaf` ou `NotFailed` ;
4. insérer l'exécution enfant (`starting`, génération `parent + 1`, même
   soumission), la continuation (`reconstructed`, `usage_limit_reset`), puis
   poser `resume_execution_id` ;
5. rendre `Reconstructed(message, snapshot)`.

Le parent reste `failed`. La reprise ordinaire
(`reconstruct_active_for_agent`) n'est pas modifiée.

## Remise de la relance (idempotence)

Clé : `recovery-<resume_execution_id>` (clé interne existante de la reprise).
Quatre états lus, jamais déduits du solde de la tâche :

| État | Lecture | Reprise |
|---|---|---|
| réservation absente | `reserve` rend `Prepared` | réserver avec le canonique figé, puis graver |
| réservée, sans remise | `reserve` rend `Replayed` et la phase de la remise est absente | graver la remise |
| remise en vol | phase `dispatching` | attendre : l'entrée reste `relaunching`, le tick suivant relit |
| remise accusée | phase `acked` | solder |
| remise incertaine ou perdue | phase `indeterminate` ou `orphaned` | échec explicite `native_relaunch_failed` avec l'avis durable |

`reserve` avec une enveloppe différente rend `EnvelopeMismatch` : échec explicite
`native_relaunch_failed`. La phase s'obtient par la lecture explicite de phase
de 150 (T005). Le solde de l'entrée n'a lieu qu'à `acked`. Pourquoi pas à
`dispatching` : le daemon marque `indeterminate` (état absorbant) toute remise
native retrouvée `dispatching` à la reconnexion de l'instance
(`daemon.rs:4195-4215`, `send_delivery.rs:294`). Un crash entre la gravure et
l'émission laisserait un message jamais envoyé avec une attente soldée. Une
remise en vol qui expire devient `indeterminate`
(`settle_expired_dispatching`) : elle mène à l'échec explicite. L'échéance
effective continue de courir après `retour + 30 s`.

## Gel cumulé (`referent_control.rs`)

`frozen_overlap(conn, task, from, to) -> i64` : longueur de l'union des
intervalles de pause (`pause_intervals`, extrait de `pause_overlap` de 152) et
des intervalles de `quota_waits` compris entre `from` et `to`.

```text
intervalles = pause_intervals(from, to) + [e.since .. min(e.until ou to, e.reset_at + 30) pour e dans quota_waits]
rogner à [from, to], trier par début, fusionner les chevauchements, sommer
```

Complexité : lecture SQL du journal H, tri puis union O((P + Q) log(P + Q)), espace O(P + Q), Q au plus2.

Utilisé par : échéance effective (`mission_deadline_at + frozen_overlap(...,
mission_deadline_at - reply_timeout, now)`), `frozen_secs` du suivi de demande,
décision de reprise de `prepare_restart`.

## Machine d'état (résumé)

```text
working --échec, opt-in, fait qualifiant connu--> working + QuotaWait(waiting)   (since = instant de l'échec)
working --échec, opt-in, aucun fait--> working + QuotaHold (5 s)
QuotaHold --fait qualifiant avant 5 s--> working + QuotaWait(waiting)
QuotaHold --5 s écoulées--> failed (erreur d'origine, avis d'échec durable)
QuotaHold --redémarrage--> failed (erreur d'origine, avis durable, même si fait vu avant en mémoire volatile)
working + waiting --retour + 30 s, garde de pause, enfant connecté--> relaunching
relaunching --reconstruction (idempotente)--> relaunching (resume_execution_id posé)
relaunching --chaque tick, sans nouveau gate horaire, pause/échéance conservées--> relire continuation/remise existantes
relaunching --remise réservée puis gravée (états de la remise)--> relaunching
relaunching --remise accusée--> working + done (until posé), exécution enfant en cours
relaunching --remise indeterminate ou orphaned--> failed (native_relaunch_failed, avis durable)
relaunching --remise non reprenable--> failed (native_relaunch_failed, avis durable)
working + waiting|relaunching --annulation--> cancelling (réconciliation continuation et remise, puis done)
working + waiting --redémarrage--> working + waiting (inchangé)
échec qualifiant mais retour > 6 h, passé, ou 2 rondes faites--> failed (usage_limit, limit_reset_at)
```

## Refus ancien (`rejected()`)

Trame wrapper vers daemon : `DeliveryRejected { id, reason, execution_id:
Option<String>, generation: Option<u64> }`, les deux champs ajoutés avec
`#[serde(default, skip_serializing_if = "Option::is_none")]`. Le wrapper les
remplit depuis `bindings.get(message_id)` quand la liaison existe. Compatibilité :
un ancien daemon ignore les champs (l'énumération `WrapperToDaemon` n'a pas
`deny_unknown_fields`) ; un ancien wrapper ne les envoie pas (`None`).

Sortie de `rejected()` : `Pass` (tâche non opt-in ou mission inconnue :
comportement actuel, le gestionnaire continue), `PauseInterruption` (152),
`QuotaHold` (hold ou attente ouverte), `NativeFailed` (échec décidé : le
gestionnaire retire le suivi et sort), `StaleIgnored` (refus ancien : sans
effet, le gestionnaire sort). Toute sortie autre que `Pass` fait sortir le
gestionnaire avant `get_request` et `deliver_to_agent`.

Pour une tâche opt-in, avec `current` = (identifiant, génération) de la feuille
de chaîne :

| Trame | Tâche jamais relancée | Tâche relancée au moins une fois |
|---|---|---|
| champs présents, égaux à `current` | refus courant | refus courant |
| champs présents, différents | `StaleIgnored` | `StaleIgnored` |
| champs absents | refus courant | échec explicite `native_refusal_uncorrelated` |

Un refus courant ne peut qualifier une limite que si la feuille est `failed`
(sinon : `NativeFailed` avec l'erreur d'origine). Les refus d'une tâche opt-in
déjà terminale rendent `NativeFailed` (jamais `Pass`) uniquement pour arrêter le gestionnaire : préserver `result_available`/`cancelled`, ne pas réécrire `failed` ni réserver un nouvel avis.

## Vue de tâche (`task_view`)

Champ additif `quota_wait` : `null`, ou `{round, phase, resumes_at}`. La tâche
reste `working`. `suspended` de 152 reste réservé à la pause.

## Garde (`referent_control.rs`)

Aucune variante nouvelle : la relance utilise `AutonomousEffect::NativeMissionDispatch`
de 152.

## Précisions R1

La corrélation FR006b précède toute classification FR006c/quota. Après une relance, des champs absents donnent donc `native_refusal_uncorrelated`, même pour un refus synchrone. Une réponse vide termine l’Execution en `completed`, tandis que la Task demeure `working` jusqu’au refus : elle échoue avec l’erreur d’origine si le refus est corrélé. Les Task déjà `result_available` ou `cancelled` sont préservées ; une Task déjà `failed` ne reçoit pas de nouvel avis.

Lecture160 conservatrice : `attested_exhausted_since(type,since,now,past_since:Option<i64>)` garde la preuve enregistrée par157 et son admission inchangée. Observation strictement après `since`, puis reset futur OU (`past_since=hold.since` et observation>=past_since). Pendant le hold, filtrer chaque fenêtre par `observed_at <= past_since + 5` avant toute agrégation de reset/observation. Hors résolution du hold, past_since=None ; ancienne observation expirée=NoFact. Le cas fait observé pendant le hold puis expiré à la décision donne `Past{reset_at}`/usage_limit. Une fenêtre rejetée à son enregistrement n’est jamais inventée.

Toute sortie d’échec depuis `relaunching` réconcilie la feuille et la remise avant Task.failed/cleanup, en réemployant `close_lost_execution` et T024 : cible child vérifiée, CAS de la feuille vers `unreachable`, remise dispatching mise en quarantaine. Une erreur de clôture se propage ; l’entrée reste `relaunching` pour réessayer le settlement. Aucun nouvel arrêt physique. Les résultats reçus pour Task.failed sont déjà ignorés par capture_reply (ACK transport seulement). Cela ne garantit pas l’unicité des effets externes.

Refus/non-reprenable de la remise : native_relaunch_failed existant, pas nouvelle phase. AlreadyResumed(id) et relectures existantes conservent le premier observed_at.
