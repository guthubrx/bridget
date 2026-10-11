# Contre-revue adverse - Session150

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, mode plan (lecture seule),
  tâche T3 `s150-contre-revue-plan-r1`
- **Canal** : délégation T3. La délégation Bridget a d'abord été refusée
  (`permission_source_unavailable`) alors que le catalogue annonce
  `discovery: true` : incohérence notée pour la session157.
- **Question** : réalisabilité de la reprise après pause, sûreté de la purge par
  arbre, compatibilité de la génération de projection, enveloppe de lancement,
  autres défauts.
- **Verdict reçu** : BLOCKED

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | L'interruption émet `DeliveryRejected` qui force `failed` hors de `tick` | lu `daemon.rs:18078-18081`, `native_delegation.rs:815-822` | oui | second chemin d'échec réel ; transmis à 152 avec toute la suspension |
| 2 | La reprise peut précéder le terminal de l'interruption | lecture de l'ordre `TurnState(false)` puis `ExecutionStateChanged` (`wrapper.rs:6141,6182`) cohérente | oui | transmis à 152 |
| 3 | La continuation n'est suivie que dans `tick`, pas dans reprise, annulation, descendants | cohérent avec `daemon.rs:4253`, `native_delegation.rs:901` | oui | transmis à 152 |
| 4 | Décaler l'échéance à la reprise ne gèle pas les délais pendant la pause | cohérent avec l'ordre de `tick` (`:1013`) et le suivi de demande | oui | transmis à 152 |
| 5 | `result_sent`/`failure_sent` vrais sur `OutcomeUnknown` : remise non prouvée | lu `native_delegation.rs:1059-1064`, `:1231-1236` | oui | la purge exige la phase `acked` (R2.2) |
| 6 | `cleanup_done` ne prouve pas que l'enfant est arrêté (relance possible) | lu `native_delegation.rs:1474-1477`, `daemon.rs:16653-16700` | oui | contrôle présence + flotte + lancement sous verrou (R2.3) |
| 7 | `spawn_base_at` ne garantit pas la compatibilité des lancements déjà soumis | cohérent avec `fleet.rs:1679-1696` | oui | `spawn_base_at` retiré de 150 ; gel des lancements traité en 152 |
| 8 | `root_page` regroupe par propriétaire, pas par arbre | lu `delegation_lineage.rs:100-103` | oui | arbre = tâche sans parent + descendants (R1) |
| 9 | Ouverture : chargement complet et migration non bornée | lu `delegation_lineage.rs:74-80` et l'admission `:442` | partiel | garde retirée ; lots rejetés car la base ne peut pas dépasser 4096 lignes par le chemin normal (R5) |
| 10 | Nouvelle génération compatible avec 149, publier après commit | lu `native_lineage.rs:144-162` | oui | signal après commit par `committed()` (R9) |

**Correction faite au passage** : l'affirmation « le daemon ne démarre plus
au-delà de 4096 » était fausse en pratique. Le vrai défaut est le refus
définitif après 4096 délégations cumulées. La spec a été corrigée.

## Ronde 2 (plan révisé)

- **Agent** : Codex GPT-6.1-Sol, effort élevé, lecture seule, tâche
  `s150-contre-revue-plan-r2`
- **Verdict reçu** : BLOCKED (objections 8, 9, 10 déclarées fermées ; retrait
  vers 152 jugé cohérent)

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | Ligne de remise absente ≠ accusé : `purge_expired_sends` efface aussi les remises indéterminées | lu `send_delivery.rs:571-579` | oui | preuve durable `delivery_acked` recopiée par l'entretien ; sans preuve, arbre conservé |
| 2 | Avis d'échec jamais envoyé peut disparaître | cohérent avec `failure_sent=false` | oui | `failed` et `result_available` exigent `delivery_acked` |
| 3 | Entrée `Stopped` relançable après purge | lu `fleet.rs:876-884`, `fleet.rs:545` | oui | décommission de l'entrée avant suppression |
| 4 | Purge au démarrage avant la réconciliation des marqueurs | lu `daemon.rs:4550-4561` | oui | purge retirée du démarrage ; entretien à 120 s |
| 5 | Délai implicite casse les délégations imbriquées | raisonnement vérifié sur R8 | oui | contrôle parent seulement pour un délai explicite |
| 6 | `send_delivery()` masque les phases | signalé par l'agent | oui | lecture explicite de phase ajoutée au plan |
| 7 | Lancement en cours : `managed_spawns` + sagas | signalé par l'agent | oui | intégré à R2.3 |

## Ronde 3 (plan révisé)

- **Agent** : Codex GPT-6.1-Sol, effort élevé, lecture seule, tâche
  `s150-contre-revue-plan-r3`
- **Verdict reçu** : BLOCKED sur un seul point ; objections R2 n° 2 à 7
  fermées ; décommission sans effet de bord dangereux (`fleet.rs:545`,
  `desired_state.rs:345`).

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | Recopie limitée à 128 puis purge globale des remises : une tâche accusée peut perdre sa preuve | raisonnement vérifié sur T010 et `send_delivery.rs:571` | oui | recopie sans lot (total ≤ 4096) et exécutée avant `purge_expired_sends` ; test des 129 accusés expirés |
| 2 | Une reprise après plus de 37 jours d'arrêt n'est pas protégée | durées vérifiées par l'agent | noté | même dans ce cas, l'absence de preuve conserve l'arbre ; la perte possible est une purge qui n'a plus lieu, jamais un résultat perdu |

Statut final du plan : objections bloquantes fermées. Pas de ronde 4 :
l'unique point restant a une correction directe et un test dédié.
