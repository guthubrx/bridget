# Contre-revue adverse - Session160

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : les six exigences d'entrée sont-elles fermées, la relance est-elle
  sûre aux frontières de crash, l'ordre des horloges, des pauses et du hold
  est-il cohérent.
- **Verdict reçu** : BLOCKED. Défauts d'entrée 1, 2 et 4 fermés ; 3, 5 et 6
  partiels. Les 5 s sont une politique bornée, pas une garantie.
- **Méthode** : chaque objection est une hypothèse, relue dans le checkout
  principal (`1f2c661d`) et dans les specs 152, 157 et 160 du worktree avant
  d'être retenue.
- **Règle de sûreté appliquée** (Art. XIX) : si un point exige une mécanique
  disproportionnée, la tâche échoue de façon explicite et nommée, avec l'avis
  d'échec durable, plutôt qu'une reprise fragile.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | BLOQUANT. Une continuation unique n'est pas une remise unique et complète : crash après la reconstruction, avant la réservation ou la gravure ; le plan soldait sans remise ; l'extraction met `now` dans le canonique et refuse `Reservation::Replayed` | lu `daemon.rs:4361-4418` (`canonical_send(..., now)`, `Ok(other) => error!("réservation interne inattendue")`) ; `idempotency.rs:124-129` (`Replayed` existe) | oui | `deliver_reconstructed` reçoit la date d'émission (observation de la continuation) et un mode ; en mode relance, `Replayed` est repris ; quatre états de remise lus (absente, réservée, gravée, accusée) ; solde seulement après gravure ; six frontières testées (R5, R10, FR-011, FR-011b, T008, T019, T020). **Échec explicite** `native_relaunch_failed` si la remise est non reprenable |
| 2 | BLOQUANT. Enfant absent : l'intervalle ouvert `[since, maintenant]` fait avancer l'échéance effective, `mission_reply_timeout` ne peut plus expirer | raisonné sur la formule de `frozen_overlap` (data-model) et sur FR-014 | oui | intervalle quota borné à `reset_at + 30 s`, indépendant de la connexion ; gel de pause distinct ; après la borne, le temps court (R7, FR-014, FR-016, SC-010, T012, T020, T023) |
| 3 | HAUTE. La reprise ordinaire de 152 peut concurrencer la reprise quota : l'exécution `starting` créée est « active » et la garde 152 autorise une tâche `working` de la chaîne | lu `execution_store.rs:1466-1467` (`recoverable_execution_ids_for_agent` inclut `starting`), 152 T015 (`may_resume`) | oui | `may_resume` refuse toute chaîne dont l'entrée de quota est ouverte ; tests de reconnexion aux frontières c à f (R10, FR-011c, SC-012, T032, T034) |
| 4 | HAUTE. `quota_hold` non protégé des horloges ni ordonné avec la pause : expiration possible pendant les 5 s ; le saut des tâches pausées de 152 empêche la résolution | lu 152 T012 (saut de pause en tête de `tick`) et T017 (échéance effective) | oui | ordre fixe : annulation, résolution du hold, saut de pause, expiration, relance ; le hold inhibe expiration et rappels ; résolution indépendante du parent et de la pause ; l'attente commence à l'instant de l'échec (R9, FR-004, FR-004b, T016, T023, T033) |
| 5 | HAUTE. Effets d'échec sans preuve durable : un rejeu du message direct après une décision différée peut être perdu ou doublé après un crash | lu `daemon.rs:18108` (`deliver_to_agent` direct), `native_delegation.rs:1048-1068` (avis durable `native-failure-<task_id>`, `failure_sent`) | oui | **Choix : échec explicite avec l'avis durable existant.** Aucun rejeu du message direct, aucune extraction `notify_delivery_failure`. Tests de crash avant et après `failed`, réservation, gravure, solde (R8, FR-006, T015) |
| 6 | HAUTE. `DeliveryRejected` ne porte que l'identifiant de mission, identique après une reprise : un refus ancien fait échouer la mission relancée | lu `wrapper.rs:6256` (`id: message_id`), `native_delegation.rs:810-825` | oui | `StaleIgnored` : pour une tâche qui a des entrées, `rejected()` n'agit que si la feuille de chaîne est `failed` (le wrapper publie la transition `failed` avant le refus) ; pas de nouvelle corrélation dans le protocole (R15, FR-006b, SC-011, T033, T035) |
| 7 | MOYENNE. `resume_on_limit` en `bool` : absent et `false` explicite fusionnent alors que le contrat exige `envelope_mismatch` | lu T009 et `contracts/reprise160.md` (version précédente) | oui | `Option<bool>`, omis seulement pour `None` (data-model, contrat, T009, T025) |
| 8 | HAUTE. Annulation pendant `relaunching` incomplète : continuation existante, identifiant pas encore porté par la tâche ; purge 150 sous pression avec parent suspendu | lu data-model (entrée fermée) et 152 T014, T016 ; `send_delivery.rs:529` (`mark_delivery_indeterminate`) | oui | réconciliation ordonnée : relire l'identifiant, clore la feuille de chaîne, marquer la remise `indeterminate`, fermer l'entrée ; sinon `cancelling` avec `native_cancel_unverifiable:quota_relaunch` ; tests frontières c et d, remise non livrée, purge d'un arbre en attente (non terminal, donc conservé) et après annulation (R16, FR-017, T024) |

**Rejetée** : aucune objection.

**Reprise ou échec explicite**
- Reprise conservée, mécanique petite : objections 2, 3, 4, 6, 7 et 8.
- Reprise conservée avec garde-fou : objection 1 (remise reprise à l'état trouvé,
  date figée). Si la remise est non reprenable : échec explicite
  `native_relaunch_failed`. Raison : réparer une enveloppe ou une remise
  incohérente exigerait une mécanique disproportionnée.
- Échec explicite retenu : objection 5 (avis durable, pas de rejeu du message
  direct), et le fait de quota arrivé après les 5 s (aucune relance, le parent est
  informé).

**Effets** : tâches 31 devient 35 (T032 à T035). Une erreur de tâche nouvelle,
`native_relaunch_failed`. Un changement visible : pour une tâche opt-in, l'avis
d'échec durable remplace le message direct « Échec de livraison de la demande ».

**Reste ouvert**
- Les primitives de 152 et 157 sont planifiées, pas livrées (porte T001).
- Le chemin d'échec et l'ordre d'un vrai refus de quota ne sont pas vus en réel
  (T002), y compris l'ordre « transition `failed` avant `DeliveryRejected` » dont
  dépend `StaleIgnored`.
- La relance ne passe pas par la continuation gouvernée ; la session158 devra
  compter ces relances.
- Le gel de l'enfant ne gèle pas son parent.
- Pas de ronde 2 de contre-revue sur les artefacts corrigés.

## Ronde 2 (plan révisé)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 2, 3, 4, 7 et 8 fermées ; 1 et 5 partielles ; 6
  ouverte. Trois points restent.
- **Règle de sûreté** (Art. XIX) : en cas de doute, échec explicite nommé plutôt
  que reprise fragile.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | BLOQUANT. Une remise `indeterminate` était soldée comme une relance, alors que les remises natives retrouvées `dispatching` sont mises en quarantaine à la reconnexion et que l'état est absorbant : crash après gravure avant émission, le message peut ne jamais partir et l'attente être soldée | lu `daemon.rs:4195-4215` (`schedule_idempotent_delivery_recovery` : toute remise dont le message est une mission native passe `indeterminate`) et `send_delivery.rs:270-320` (commentaire : `indeterminate` et `orphaned` sont absorbants) | oui | **Choix : échec explicite.** Solde seulement à `acked` ; `dispatching` = attendre ; `indeterminate` ou `orphaned` = `native_relaunch_failed` avec l'avis durable. Scénarios f (gravée, reconnexion, quarantaine) et h (orpheline) ajoutés à T008 et T020. SC-004 ajusté : message remis une fois OU échec explicite, jamais un message perdu avec attente soldée (FR-011b, R5, R10, data-model) |
| 2 | BLOQUANT. « `failed` avant `DeliveryRejected` » est faux partout : refus asynchrone (vrai), injection idempotente refusée (refus sans transition), réponse vide (`completed` puis refus) ; T035 pouvait ignorer un refus courant, et une feuille `failed` ne prouve pas que le refus la concerne | lu `wrapper.rs:4461-4475` (refus sans transition), `:6182-6205` (réponse vide : transition `completed` puis refus), `:6244-6262` (échec asynchrone), `protocol.rs:4164` (trame `id`, `reason` seulement) et `:2809-2812` (`WrapperToDaemon` sans `deny_unknown_fields`) | oui | **Corrélation par exécution et génération.** La trame `DeliveryRejected` gagne `execution_id` et `generation` optionnels, remplis depuis `bindings` du wrapper. Compatibilité : un ancien daemon ignore les champs ; un ancien wrapper ne les envoie pas. Sans corrélation : tâche jamais relancée = refus courant ; tâche relancée = **échec explicite** `native_refusal_uncorrelated` (jamais ignoré). Seul un refus courant dont la feuille est `failed` peut qualifier une limite ; injection synchrone et réponse vide échouent avec leur erreur d'origine. Tests : injection synchrone, réponse vide, ancien refus quand la nouvelle feuille est déjà `failed`, refus sans corrélation (FR-006b, FR-006c, R15, T033, T035, T036) |
| 3 | HAUTE. Avis unique défini pour le hold seulement : le chemin `Handled` garde le message direct, alors que l'avis durable existe ; refus immédiats `TooFar`, `Past`, `RoundsDone` et refus tardif après la résolution d'un hold non protégés du double avis | lu `daemon.rs:18108` (message direct par `deliver_to_agent`) et `native_delegation.rs:1048-1068` (avis durable `native-failure-<task_id>`) | oui | sortie dédiée `NativeFailed` pour TOUT échec traité d'une tâche opt-in : suivi retiré, avis durable seul, gestionnaire sorti avant `get_request` et `deliver_to_agent`. `Pass` seulement pour les tâches non opt-in (comportement exact). Rouge T037 puis T015 : un test par cas (hold, refus immédiats, réponse vide, refus tardif, tâche terminale) (FR-006, SC-013, R8) |

**Rejetée** : aucune objection.

**Choix, reprise ou échec explicite**
- Point 1 : échec explicite `native_relaunch_failed` plutôt que de solder une
  remise incertaine.
- Point 2 : corrélation réelle, plus échec explicite `native_refusal_uncorrelated`
  quand elle manque après une relance. Jamais d'« ignorer » sans preuve.
- Point 3 : avis durable seul, sans message direct, pour tout échec opt-in.

**Effets** : tâches 35 devient 37 (T036 trame et wrapper, T037 rouge double avis ;
T008, T015, T019, T020, T033, T035 réécrites). Deux erreurs de tâche nouvelles :
`native_refusal_uncorrelated`, et `native_relaunch_failed` élargi. Une extension
de trame compatible.

**Reste ouvert**
- La liaison d'exécution du wrapper existe-t-elle au refus d'une injection
  synchrone ? T036 le vérifie. Sinon le refus est sans corrélation (règle ci-dessus).
- Un vrai refus de quota par l'injection synchrone ou la réponse vide ne serait pas
  repris (échec avec l'erreur d'origine) ; T002 dira si c'est le cas.
- Les primitives de 152 et 157 sont planifiées, pas livrées (porte T001).
- La relance ne passe pas par la continuation gouvernée ; la session158 devra
  compter ces relances.
- Pas de ronde 3 de contre-revue.
