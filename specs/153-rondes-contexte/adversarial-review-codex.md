# Contre-revue adverse - Session153

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, mode lecture seule
- **Question** : sûreté des rondes sur un enfant gardé vivant (accusés, réponses
  tardives, droits, échéance, attente 151, places)
- **Verdict reçu** : BLOCKED
- **Méthode de traitement** : chaque objection est une hypothèse, vérifiée dans le
  code du checkout principal (`/Users/moi/Nextcloud/10.Scripts/64.bridget`, base
  `1f2c661d`) et dans les specs 150 et 151 du worktree, avant toute correction.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | `begin_round` efface `delivery_acked` : un résultat N indéterminé est oublié quand N+1 est accusé, puis la purge passe (BLOQUANTE) | lu spec 150 (R2 : « sans preuve, l'arbre reste conservé » ; recopie par message) et `send_delivery.rs:571-579` cité par 150 ; raisonnement sur le plan : une seule booléenne pour la ronde courante | oui | violation de l'invariant 150. Correction : preuve par ronde (`unacked_rounds`), purge exige la liste vide, recopie 150 étendue (FR-029, T029, SC-011). Le « minimum » proposé (exiger l'accusé avant la ronde) est écarté : le message de résultat n'arrive au parent qu'à la fin de son tour, donc le parent ne pourrait plus enchaîner les rondes ; la preuve par ronde est l'autre solution admise |
| 2 | Réponse tardive à une ronde passée traitée en message ordinaire, réveil du parent hors du mécanisme de résultat (HAUTE) | lu `native_delegation.rs:771-808` (`for_mission` vide → `Ok(false)`) ; `daemon.rs:8724` et `:17562-17578` (`captured` faux → `prepare_dispatch`) | oui | l'index unique `$.mission` (`delegation.rs:77`) ne retient que la mission courante. Avant 153 la même réponse était absorbée (`Ok(true)`). Régression réelle introduite par mon plan (R3 était fausse). Correction : table d'index `native_delegation_past_missions`, absorption authentifiée sans effet (FR-030, T028, SC-012) ; table plutôt que `json_each` pour rester en O(log n) (Art. XVIII) |
| 3 | La revérification de droits ne compare pas la politique actuelle du parent à celle de l'enfant vivant (BLOQUANTE) | lu `spawn_task` `:535-553` (révocations et `recheck_permission_context_sources` seulement), `parent_fact` `:38-66`, `child_policy` (`native_permissions.rs:206`) | oui | au lancement, la politique est calculée à l'instant ; à la ronde, elle est figée depuis la ronde 1. Une réduction de droits sans révocation ne serait pas vue. Correction : relire l'attestation actuelle, recalculer `child_policy`, refuser si la politique figée diffère (`permission_not_inherited`, `permission_attestation_unavailable`) ; grant applicable pour la voie 148 (FR-031, T014, SC-013). Risque de faux refus noté (A10) |
| 4 | Garde expirée encore utilisable : rien ne compare `now` à `idle_deadline_at` à l'admission (HAUTE) | lu le plan : l'expiration n'est levée que par `tick` (cadence 1 s, `daemon.rs:4700`) ; l'admission de `Continue` ne la lit pas | oui | entre l'échéance et le tick, une ronde prolongerait l'enfant au-delà de la borne. Correction : contrôle sous verrou à l'admission et dans `begin_round` (FR-032, I7, T004, T015, SC-014) |
| 5 | Résultat lu par l'attente 151 sans garde posée : la garde n'est posée que par `tick` (HAUTE) (caduque après 151 révisée, voir le réalignement ci-dessous) | lu spec 151 FR-017 et tâche T012 : l'attente pose `result_sent` et `delivery_acked`, « le nettoyage se déclenche, aucun message n'est envoyé » ; branche `result_available && result_sent && !cleanup_done` (`native_delegation.rs:1079`, `:1283`) | oui | le nettoyage arrêterait l'enfant gardé après une lecture par l'attente. Correction : `mark_result_sent` unique, appelé par `tick` et par l'attente ; écriture de l'attente liée à la ronde lue (FR-009, FR-033, I6, T030, SC-015) |
| 6 | Place libérée avant l'arrêt confirmé : `holds_slot()` est faux dès la levée de garde (MOYENNE) | lu `cancel_tree` : `cleanup_done` posé seulement si tous les arrêts réussissent (`native_delegation.rs:1444-1476`) ; `terminal()` vrai pour `result_available` (`delegation.rs:60-65`) | oui, ciblé | l'écart existe déjà en 149 pour toute tâche terminale non nettoyée. Je le corrige seulement pour les tâches avec garde (`guarded && !cleanup_done`), pour ne pas changer le comptage des tâches 149 (SC-009). FR-010, I2, T012, SC-015 |

**Note de l'agent revue (reprise)** : les validateurs 149 ferment la liste des
états sans imposer l'irréversibilité (`task_entry`, `delegation_lineage.rs:35`) :
le retour `result_available` vers `mission_pending` n'est pas bloqué côté
Bridget. La compatibilité du lecteur T3 reste à prouver (A2, non vérifiable dans
ce dépôt). L'expiration pendant une pause est cohérente avec 152 : une tâche
inactive est terminale, donc sans `suspended_at` (R11).

**Artefacts modifiés** : `spec.md` (FR-009, FR-010, FR-016, FR-020, FR-026, FR-029
à FR-033, SC-011 à SC-015, entités), `plan.md` (I2, I3, I6, I7, décisions, fichiers,
ordre des refus, risques), `research.md` (R3, R6, R7 corrigés, R14), `data-model.md`
(`guarded`, `unacked_rounds`, `begin_round`, table, pose de la garde),
`contracts/rounds153.md`, `tasks.md` (T003, T004, T010, T012 à T015 modifiées ;
T028 et T029 ajoutées ; 29 tâches après réalignement sur 151 révisée), `reuse-audit.md`, `analyze-report.md`,
`quickstart.md`.

**Réalignement sur 151 révisée** : la session 151 a rendu l'attente et la lecture
d'état purement lectrices (`tick` seul écrit `result_sent` et `delivery_acked`).
L'objection 5 est donc caduque dans sa forme (plus d'écriture par l'attente) :
FR-033, l'invariant I6 partagé, `mark_result_sent` et T030 sont supprimés. La
pose de la garde reste dans la sauvegarde de `tick` (code local, un seul appelant,
Art. XIX). Effet de bord traité : `continuable` n'est vrai qu'après l'envoi par
`tick` (1 s) ; un `bridget_task_continue` trop précoce rend le refus réessayable
`result_delivery_pending` (FR-019, T013, T015).

Statut du plan : les six objections sont traitées. Ouverts : le lecteur T3
(A2), le risque de faux refus de l'égalité de politique (A10) et la livraison
réelle de 150 et 151 (A1). Pas de
ronde 2 de contre-revue lancée ; elle reste recommandée avant T002.

## Ronde 2 (plan révisé)

- **Agent** : Codex, GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : objections 1, 2, 3 et 6 fermées ; 4 partiellement ; 5 caduque
  (151 révisée : lecture pure). Trois points restent.

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| R2.1 | `now` périmé : T015 passe l'heure d'admission à `begin_round` après les lectures de sources (HAUTE) | lu `spawn_task` `native_delegation.rs:535-553` : les revérifications lisent des fichiers de politique ; le plan transmettait le même `now` | oui | une échéance peut être franchie entre le contrôle et l'écriture. `begin_round` relit l'heure via une horloge injectable ; test « échéance franchie pendant la revérification » (FR-032, I7, T004, T015, SC-017) |
| R2.2 | Annulation ignorée avant l'envoi du résultat, puis garde posée jusqu'à 2 h (HAUTE) | lu `Cancel` `native_delegation.rs:518-530` (aucun effet sur `result_available`) et `human_cancel` `delegation_lineage.rs:235` (`drives = !terminal`) ; `continuable()` faux avant l'envoi, donc T011 ne couvrait pas cette phase | oui | l'annulation agent et humaine devient une preuve durable `guard_released` qui interdit à `tick` de poser la garde et à toute ronde de s'ouvrir (FR-012, FR-033, I8, T008, T010, T011, SC-016) |
| R2.3 | Égalité des politiques : la posture effective remplace la posture demandée `None` et modifie la politique Codex (MOYENNE) | lu `child_policy` `native_permissions.rs:206-213` : `readonly = posture == Some(Discovery)` impose lecture seule sans réseau pour Codex vers Codex ; l'admission passe `*posture` et `spawn_task` passe `None` avec snapshot | oui | recalcul avec la posture demandée et les entrées de lancement ; aucun champ de sécurité retiré ; matrice de tests (FR-031, T014, SC-017) |

Aucune tâche ajoutée : les trois corrections complètent T004, T008, T010, T011,
T013, T014 et T015. Total : 29 tâches. Ouverts : lecteur T3 (A2), stabilité de
l'égalité de politique à confirmer par le test de T014 (A15), livraison réelle de
150 et 151 (A1).
