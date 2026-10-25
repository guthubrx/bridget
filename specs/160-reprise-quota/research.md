# Recherche - Session160 reprise après une limite d'usage

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/160-reprise-quota/research.md#extrait-1).

## R1 - Les deux chemins par lesquels une mission échoue sur un quota

- **Chemin A (refus de remise)** : le wrapper reçoit `DeliveryRejected` du
  fournisseur, publie la transition d'exécution `failed` (raison
  `provider_failed`), puis envoie `DeliveryRejected` au daemon
  (`wrapper.rs:6218-6262`). Le daemon appelle `native_delegation::rejected()`
  (`daemon.rs:18080`), qui passe la tâche `failed` (`native_delegation.rs:810-825`).
  Le gestionnaire général continue : `pending_replies.retain`, `get_request`,
  puis `deliver_to_agent("Échec de livraison de la demande ...")` au parent
  (`daemon.rs:18084-18121`).
- **Chemin B (état d'exécution)** : `tick` lit l'exécution `failed` et passe la
  tâche `failed` (`native_delegation.rs:1197`).
- Le chemin A publie d'abord la transition d'exécution, donc B peut s'exécuter
  avant ou après A. Les deux doivent mener à la même décision.
- **Non vu en réel** : quel chemin un vrai refus de quota prend, et avec quelle
  raison. T002 le cherche dans le journal (lecture seule). Sans exemple, tous les
  tests de cette session sont synthétiques et la validation le dit.

## R2 - Modèle adaptateur de bureau et ce que Bridget reprend

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/160-reprise-quota/research.md#extrait-2).

## R3 - Défaut 1 : compteur de rondes durable

- La conception retirée portait `round` dans `limit_resume`, effacé après
  l'incrément. Le compteur disparaissait avec l'attente.
- **Décision** : la tâche porte `quota_waits`, liste où l'on ajoute sans jamais
  retirer. Rondes = longueur de la liste. Une entrée est ajoutée au début de
  l'attente. On ne compte donc que par excès : un crash ne peut pas sous-compter.
- Alternative rejetée : un entier `quota_rounds` en plus d'un fait d'attente
  effaçable. Deux champs à garder cohérents, sans gain : la liste porte aussi les
  intervalles dont le gel (R7) a besoin.

## R4 - Défaut 2 : reconstruire une exécution `failed`

- La sélection de `reconstruct_active_for_agent` n'admet que `queued`,
  `starting`, `running`, `waiting_approval`, `waiting_user_input`, `interrupting`
  (et les lignes de pause ouvertes) (`execution_store.rs:1327`). Ses mises à jour
  répètent la liste (`:1378`, `:1395`). `recoverable_execution_ids_for_agent`
  aussi (`:1467`). Étendre `may_resume` ne suffit donc pas : la requête ne rend
  jamais une exécution `failed`.
- La fonction ordinaire sélectionne par agent cible avec `LIMIT 2` et refuse
  l'ambiguïté. Elle écrit la raison de continuation en dur
  (`'reconstructed', 'daemon_restart'`, `:1427`). Elle ferme le parent en
  `unreachable`. Aucun de ces comportements ne convient à un parent `failed`.
- **Décision** : ne pas toucher la reprise ordinaire. Ajouter
  `reconstruct_quota_failed(parent_execution_id, identity, agent_instance_id,
  child_execution_id, observed_at)` : sélection par identifiant exact, autorisée
  seulement si une ligne de qualification existe pour ce parent et cette identité
  (R5), si le parent est `failed` et feuille de sa chaîne, et si l'identité n'a pas
  servi. Le parent reste `failed`. Une continuation `reconstructed` de raison
  `usage_limit_reset` est insérée, et l'identité est reliée à la continuation.
  Tout est dans une transaction.
- Complexité : accès SQLite par clé O(log N), plus coût de feuille de chaîne
  de `latest_in_chain` livré ; voir le plan actuel.

## R5 - Défaut 3 : qualification, identité et frontières de crash

- **Table** `execution_quota_resumes` dans le magasin d'exécutions :
  `execution_id` (clé primaire, l'exécution échouée), `identity` (unique),
  `reset_at`, `qualified_at`, `resume_execution_id` (nul jusqu'à la relance).
  `execution_continuations` a une clé primaire sur l'enfant et un champ `reason`
  libre mais pas de champ d'identité (`execution_store.rs:1588`). Une table dédiée
  est donc plus simple et plus sûre qu'un texte dans `reason`. Elle suit le motif
  des migrations numérotées (`:1602-1619`, prochaine version 12).
- Les deux magasins (tâches, exécutions) ne partagent pas de transaction
  (contre-revue 152, objection 2). Ordre d'écriture et frontières :

| Frontière | État durable | Reprise après crash |
|---|---|---|
| a) qualification écrite, tâche pas encore | ligne sans entrée de tâche | la décision en attente est résolue en échec normal (ou requalifiée) ; la ligne orpheline ne sert jamais |
| b) entrée `waiting` écrite | tâche + ligne | attente normale |
| c) entrée `relaunching` écrite, reconstruction pas encore | `relaunching` | tick : reconstruction (idempotente par identité) |
| d) continuation créée, aucune réservation de remise | continuation + `resume_execution_id` | tick : réservation avec le canonique figé, puis gravure |
| e) réservation faite, remise pas gravée | réservation `Prepared` | tick : `reserve` rend `Replayed`, gravure |
| f) remise gravée (`dispatching`), pas émise ou pas accusée | remise `dispatching` | tick : attendre. Si l'instance se reconnecte, le daemon la marque `indeterminate` : échec explicite `native_relaunch_failed` |
| g) remise `acked`, entrée pas soldée | remise `acked` | tick : solde, sans seconde remise |
| h) remise `indeterminate` ou `orphaned` | remise absorbante | échec explicite `native_relaunch_failed` |

- La remise du message repris utilise la clé `recovery-<exécution>`. Le code
  actuel met `now` dans le canonique (`canonical_send(..., now)`) et refuse tout
  résultat de `reserve` autre que `Prepared` (`daemon.rs:4408-4418`,
  « réservation interne inattendue »). Un rejeu après crash échouerait donc, ou
  changerait le canonique. Contre-revue 160, objection 1 : retenue.
- **Décision** : `deliver_reconstructed` reçoit la date d'émission en paramètre
  (reprise ordinaire : `now`, comme avant ; relance de quota : `observed_at` de la
  continuation) et un mode de reprise. En mode relance, `Replayed` n'est pas une
  erreur : on lit la phase de la remise (lecture explicite de 150 T005) et on
  reprend à l'état trouvé (table de `data-model.md`). Le solde n'a lieu qu'à
  partir de `acked` (contre-revue 160, ronde 2, objection 1). Au démarrage ou à la
  reconnexion de l'instance, le daemon marque `indeterminate` toute remise native
  retrouvée `dispatching` (`daemon.rs:4195-4215`) et cet état est absorbant
  (`send_delivery.rs:294`) : solder à `dispatching` perdrait le message sans
  échec. `indeterminate`, `orphaned`, `EnvelopeMismatch` ou remise invalide :
  échec explicite `native_relaunch_failed` avec l'avis durable (Art. XIX : pas de
  réparation de remise).

## R6 - Défaut 4 : cause distincte du compteur de pannes

- 152 incrémente `restart_resumes` seulement dans la décision de reprise de
  `prepare_restart`, avant toute reprise (152 `data-model.md` I4, T023, T025).
  Une relance de quota passe par une autre voie et n'y touche pas.
- La qualification durable (ligne R5, continuation `usage_limit_reset`) rend la
  cause lisible après coup.
- **Risque** : `prepare_restart` doit ne pas reconstruire une tâche en attente
  (sinon l'enfant serait relancé avant l'heure et le compteur de panne consommé).
  T021 ajoute l'exception et un test.

## R7 - Défaut 5 : gel cumulé sans double comptage

- 152 calcule le temps de pause avec `pause_overlap(conn, from, to)`, somme des
  pauses du journal `control_events` plus l'intervalle ouvert (152 T050). Il
  l'utilise pour l'échéance effective (T017), le suivi de demande (T018) et la
  décision de reprise (T023). `mission_deadline_at` n'est jamais réécrite (I2).
- Additionner les secondes de pause et d'attente compterait deux fois le temps
  commun.
- **Décision** : extraire de `pause_overlap` une fonction `pause_intervals(conn,
  from, to)` qui rend les intervalles, ajouter `frozen_overlap(conn, task, from,
  to)` : fusion des intervalles de pause et des intervalles de `quota_waits`
  (entrées closes `[début, fin]`, entrée ouverte `[début, to]`), longueur de
  l'union. `pause_overlap` devient la longueur des intervalles de pause seuls.
  Les trois sites de 152 appellent `frozen_overlap`.
- Complexité : O(P + Q log Q), P = événements de pause dans la fenêtre, Q = au
  plus 3 entrées. Calculé seulement quand le résultat sert, comme en 152.
- **Borne du gel quota** (contre-revue 160, objection 2) : un intervalle ouvert
  `[début, maintenant]` fait avancer l'échéance effective avec le temps, donc
  `mission_reply_timeout` ne pourrait jamais expirer si l'enfant reste absent.
  L'intervalle est donc borné à `reset_at + 30 s`, indépendamment de la
  connexion. Passé ce point, le temps court. Le gel de pause reste distinct.
- Aucun état écrit : un crash ne peut pas perdre le gel.

## R8 - Défaut 6 : le gestionnaire général de `DeliveryRejected`

- Après `rejected()`, le gestionnaire retire le suivi, lit la demande, applique
  `claim_timeout` pour certaines raisons et notifie le parent
  (`daemon.rs:18084-18121`). Ne pas mettre la tâche `failed` ne suffit donc pas :
  il faut aussi faire sortir le gestionnaire.
- 152 règle le cas de la pause : `rejected()` rend `pause_interruption` et le
  gestionnaire retourne `None` avant tout effet (152 T013, T044).
- **Décision** : `rejected()` rend `Pass` (tâche non opt-in : comportement
  actuel), `PauseInterruption` (152), `QuotaHold`, `NativeFailed` ou
  `StaleIgnored` (R15). Le gestionnaire retourne `None` pour toute sortie autre
  que `Pass`, et retire le suivi pour `NativeFailed`.
- **Un seul avis pour tout échec opt-in** (contre-revue 160, ronde 2, objection
  3) : la sortie dédiée `NativeFailed` couvre tous les échecs traités d'une tâche
  opt-in, pas seulement le hold : refus immédiat (`TooFar`, `Past`, `RoundsDone`),
  réponse vide, refus tardif après la résolution d'un hold, et refus d'une tâche
  déjà terminale. Aucun ne passe par `get_request` ni `deliver_to_agent`
  (`daemon.rs:18108`), donc aucun double avis avec `native-failure-<task_id>`.
- **Pas de rejeu du message direct** (contre-revue 160, objection 5, retenue).
  Le message « Échec de livraison de la demande » est envoyé par
  `deliver_to_agent` sans preuve durable (`daemon.rs:18108`). Le rejouer après une
  décision différée donnerait un avis perdu ou doublé après un crash. Le parent
  d'une tâche en échec reçoit déjà l'avis durable : `tick` envoie
  `native-failure-<task_id>` par l'envoi idempotent quand le propriétaire est
  connecté, et mémorise `failure_sent` (`native_delegation.rs:1048-1068`).
  Une résolution de hold en échec se contente donc de passer la tâche `failed` et
  de retirer le suivi (`pending_replies.retain`, sans notification). Pour une
  tâche opt-in, l'avis durable remplace le message direct. Aucune extraction de
  `notify_delivery_failure` n'est faite.

## R9 - L'ordre d'arrivée et la décision en attente de 5 s

- Le fait de quota arrive par `handle_rate_limit` (`daemon.rs:6600-6660`, 157 le
  copie dans `provider_limits`). L'échec arrive par un autre chemin (R1). Rien ne
  garantit l'ordre.
- **Décision** : à l'échec d'une tâche opt-in, si un fait qui qualifie est déjà
  connu : attente immédiate. Sinon : `quota_hold {depuis, raison, exécution}`,
  tâche `working`, jusqu'à 5 s. Le tick (1 s) reteste. Après 5 s : échec normal
  avec l'avis d'échec durable (R8). Cinq secondes couvrent plusieurs ticks sans faire
  attendre longtemps un échec sans rapport avec un quota. La valeur n'est pas
  mesurée en réel (T002).
- Le délai ne concerne que les tâches opt-in : les autres échouent sans délai.
- **Politique, pas garantie** (contre-revue 160) : un fait de quota arrivé après
  les 5 s ne relance rien. L'échec est explicite et le parent est informé.
- **Ordre fixe dans `tick`** (objection 4, retenue) : annulation, résolution du
  hold, saut de pause de 152 (T012), expiration de l'échéance, relance. Sans
  cela, une tâche en pause serait sautée avant que le hold ne se résolve, et une
  échéance imminente pourrait expirer pendant les 5 s. Le hold inhibe l'expiration
  et les rappels. Il se résout sans le parent : passer à l'attente écrit l'état,
  passer à l'échec écrit l'état et l'avis durable attend la reconnexion (permis
  pendant la pause, ADR 027, 152 H3). Le début de l'attente reprend `since` du
  hold.
- Redémarrage pendant le `quota_hold` : le fait de quota (mémoire) est perdu et
  le suivi en mémoire aussi. La décision est résolue en échec au démarrage, avec
  l'avis d'échec durable. C'est conservateur.

## R10 - La relance

- **Déclencheur** : `tick`, pour une tâche `working` dont la dernière entrée de
  `quota_waits` est `waiting` et `maintenant >= retour + 30 s`. 30 s absorbent un
  écart d'horloge.
- **Conditions** : tâche non `cancelling`, garde de pause
  (`NativeMissionDispatch`, 152 T009 et T019), enfant connecté et libre.
- **Remise** : la queue de `schedule_execution_recovery` (réservation de la clé
  `recovery-<exécution>`, gravure de la remise, report des contrôles) est extraite
  en `deliver_reconstructed`. Deux usages : la reprise ordinaire et la relance de
  quota. L'enfant reçoit le message d'origine (même identifiant de mission).
- **Gouvernance** : la reprise ordinaire réserve une continuation gouvernée
  (`reserve_governed_continuation`, `managed_supervisor.rs:64`), qui lit les faits
  de budget du parent. Décision : la relance de quota n'y passe pas. Son
  autorisation est la ligne de qualification plus l'entrée de tâche, plus
  stricte. Risque : un budget par tâche de la session158 devra compter ces
  relances. T002 relit `reserve_governed_continuation` et consigne si un parent
  `failed` serait accepté.
- **Chaîne réservée** (objection 3, retenue) : après la création de la
  continuation, l'exécution `starting` est « active » pour la reprise ordinaire
  (`recoverable_execution_ids_for_agent` l'inclut, `E:1467`), et la garde de 152
  autorise une tâche `working` de la chaîne. À la reconnexion de l'enfant,
  `schedule_execution_recovery` pourrait donc reconstruire une seconde fois.
  `may_resume` refuse toute chaîne dont l'entrée de quota est ouverte. Tests de
  reconnexion après crash aux frontières c à f.
- **Enfant absent** : réessai à chaque tick. Le gel quota s'arrête à
  `reset_at + 30 s` (R7) ; ensuite l'échéance effective court et
  `mission_reply_timeout` (existant) expire. Aucun mécanisme nouveau de relance
  de l'enfant : la flotte relance déjà un agent persistant.

## R11 - Bornes et anti-boucle

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/160-reprise-quota/research.md#extrait-3).

## R12 - Pas de substitution

- La tâche porte la définition épinglée à l'admission
  (`scoped.resolved_definition`). La relance la réutilise telle quelle. Le champ
  `fallback` (157) n'est évalué qu'à l'admission : il n'est jamais relu. Test
  d'identité de la cible avant et après.

## R13 - Complexité et coût

- Qualification : O(log A + F), A types et F fenêtres bornées par157.
- Relance : accès SQLite O(log N) et coût de feuille de chaîne du plan.
- Gel : lecture du journal H, tri/union O((P + Q) log(P + Q)), espace O(P + Q), Q au plus2.
- Une table, un champ de liste, aucun module : aucune extraction sans deux
  usages (`deliver_reconstructed`, `pause_intervals`).

## R14 - Constaté hors périmètre

- Le gel d'un enfant ne gèle pas son parent (limite notée).
- La mémoire des limites est volatile : l'entrée `waiting` garde son retour, donc
  la relance ne dépend pas de la mémoire.
- Une relance de quota n'est pas comptée par un budget (session158).

## R15 - Refus ancien et corrélation (objection 6, rondes 1 et 2)

- `DeliveryRejected` ne porte que l'identifiant du message (`protocol.rs:4164`,
  `wrapper.rs:6256`). Après une relance, la mission garde le même identifiant : un
  refus de l'ancien run ferait échouer la mission relancée (`rejected()` ne
  regarde que la tâche et `child_instance`, `native_delegation.rs:810-825`).
- **La première décision était fausse** : se fier à « la transition `failed`
  précède `DeliveryRejected` ». C'est vrai pour le refus asynchrone
  (`wrapper.rs:6244`). C'est faux pour l'injection idempotente refusée
  (`wrapper.rs:4461` : refus sans transition) et pour la réponse vide (transition
  `completed` puis refus, `wrapper.rs:6182` puis `:6195`). Une feuille `failed` ne
  prouve pas que le refus la concerne.
- **Décision** : corréler à l'exécution et à sa génération. Étendre la trame
  `DeliveryRejected` de deux champs optionnels (`execution_id`, `generation`),
  remplis depuis `bindings.get(message_id)` du wrapper
  (`ManagedExecutionBinding` porte `execution_id` et `generation`,
  `wrapper.rs:5519-5527`).
- **Compatibilité** : `WrapperToDaemon` n'a pas `deny_unknown_fields`
  (`protocol.rs:2809-2812`) : un ancien daemon ignore les champs. Un ancien
  wrapper ne les envoie pas : `None`. Nouveau daemon, ancien wrapper : champs
  absents.
- **Règle sans corrélation** (règle de sûreté) : pour une tâche jamais relancée,
  un refus sans champs est traité comme courant (aucun refus ancien n'est
  possible). Pour une tâche relancée, la corrélation est impossible : échec
  explicite `native_refusal_uncorrelated`, jamais « ignorer », car ignorer
  pourrait masquer un vrai échec et laisser la tâche en attente.
- **Qualification** : seul un refus courant dont la feuille est `failed` peut
  qualifier une limite. L'injection synchrone et la réponse vide échouent avec
  leur erreur d'origine. Limite assumée : si T002 montre qu'un vrai refus de quota
  passe par l'injection synchrone, une session ultérieure l'ajoutera.
- Effort : deux champs, un remplissage dans les chemins d'émission du wrapper,
  une comparaison dans `rejected()`. Pas de nouvelle table.

## R16 - Annulation pendant `relaunching` (objection 8)

- À ce stade, la continuation peut exister sans que la tâche porte encore son
  identifiant, et une remise peut être réservée ou gravée.
- **Décision**, dans cet ordre, idempotente et pilotée par `tick` (152 FR-007) :
  (a) relire `resume_execution_id` dans `execution_quota_resumes` par identité et
  le reporter dans l'entrée ; (b) clore la feuille de chaîne (`latest_in_chain`,
  `close_lost_execution` de 152 T014) ; (c) si la remise `recovery-<exécution>` est
  `dispatching`, la marquer `indeterminate` (`mark_delivery_indeterminate`,
  `send_delivery.rs:529`), pour qu'aucune reprise de remise ne la livre ; (d)
  fermer l'entrée (`done`, sans `resume_execution_id` si aucune continuation). Si
  une étape échoue : la tâche reste `cancelling` avec
  `native_cancel_unverifiable:quota_relaunch` (mécanisme de 152). Échec explicite
  plutôt que réparation.
- La reprise ordinaire ne la reprend pas : `may_resume` refuse `cancelling` et
  `cancelled`.
- **Purge de 150** : une tâche avec une entrée ouverte est `working`, donc non
  terminale : son arbre n'est jamais purgeable, même sous pression de borne (le
  parent suspendu reste conservé ; `retention_saturated` si la borne est
  atteinte). Après annulation, la tâche est terminale et l'arbre redevient
  purgeable. Tests dans T024.
