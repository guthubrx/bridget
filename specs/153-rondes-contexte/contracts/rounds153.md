# Contrat - Rondes et brief 153

Compatible avec les contrats 148, 149 et 150. Un appel sans nouveau champ produit
les mêmes octets canoniques et le même comportement qu'en 150.

## `bridget_delegate` - champs ajoutés

| Champ | Type | Obligatoire | Bornes | Défaut |
|---|---|---|---|---|
| `keep_alive_secs` | entier | non | 60 à 7 200 | absent : pas de garde |
| `brief` | objet | non | schéma du brouillon `bridget_handoff`, corps rendu <= 16 384 octets | absent |

Les deux champs font partie de l'enveloppe idempotente.

`brief` accepte exactement les champs du brouillon de `bridget_handoff`
(`objective`, `summary` obligatoires ; `results`, `decisions`, `questions`,
`next_step`, `references`, `limitations`). Champs inconnus, `null` et chaînes
blanches : refus. Aucune troncature.

Avertissements (rappelés dans la description de l'outil) : les sources citées ne
sont pas vérifiées ; le journal est lisible plus largement que l'enfant ; aucun
secret dans un brief.

## `bridget_task_continue` - nouvel outil

| Champ | Type | Obligatoire | Bornes |
|---|---|---|---|
| `task_id` | chaîne | oui | tâche du même parent |
| `request_id` | chaîne | oui | 1 à 128, **nouvelle clé à chaque ronde** |
| `task` | chaîne | oui | 1 à 65 536 octets |
| `brief` | objet | non | comme ci-dessus |

Réponse : la vue de tâche (`task_id`, `status`, `child_agent_id`, `message_id` =
mission de la ronde, `mission_deadline_at`, `result`, `error`) plus :

| Clé | Valeur |
|---|---|
| `round` | numéro de la ronde courante |
| `idle_deadline_at` | entier ou `null` |
| `continuable` | booléen ; vrai seulement après l'envoi du résultat par `tick` |

Après une ouverture réussie : `status = "mission_pending"`, `round = N`,
`continuable = false`, `result = null`.

Annotations MCP : `readOnlyHint=false`, `destructiveHint=false`,
`idempotentHint=true`, `openWorldHint=true`.

## `bridget_task_status` - clés ajoutées

`round`, `idle_deadline_at`, `continuable`. Aucun nouveau `status`.

## `bridget_task_cancel` - comportement ajouté

Sur une tâche avec garde en `result_available` (résultat envoyé ou non) : pose la
fin explicite durable et lève la garde, l'enfant est arrêté par le nettoyage,
`status` reste `result_available`. Le résultat est quand même remis au parent. Sur une tâche en
ronde : annulation existante (`cancelling`).

## Refus ajoutés (code renvoyé dans `native_delegation_refused`)

| Code | Outil | Quand | Écriture |
|---|---|---|---|
| `brief_invalid` | delegate, continue | brief refusé par le validateur (message : champ et raison) | aucune ; clé libre |
| `keep_alive_out_of_range` | delegate | hors 60 à 7 200 | aucune |
| `keep_alive_not_enabled` | continue | tâche créée sans `keep_alive_secs` | aucune |
| `round_in_progress` | continue | la ronde courante n'est pas finie | aucune |
| `result_delivery_pending` | continue | résultat prêt (`result_available`) mais pas encore envoyé par `tick` (cadence 1 s). **Réessayable** : refaire le même appel après environ une seconde ; la `request_id` n'est pas consommée | aucune |
| `keep_alive_ended` | continue | garde levée, tâche `failed` ou `cancelled`, enfant absent, ou `now >= idle_deadline_at` | aucune |
| `round_limit` | continue | 8 rondes déjà ouvertes | aucune |

Refus réutilisés : `permission_attestation_unavailable` (attestation actuelle absente) et `permission_not_inherited` (politique actuelle du parent plus faible que la politique figée de l'enfant), `referent_paused` (150), `envelope_mismatch`,
`task_unavailable`, `identity_unavailable`, `invalid_request`,
`settings_revision_changed`, `permission_not_inherited`,
`delegation_grant_required`.

Ordre pour `bridget_delegate` : ordre 150, avec `brief_invalid` et
`keep_alive_out_of_range` juste après `reply_timeout_out_of_range` et avant le
contrôle de rejeu.

Ordre pour `bridget_task_continue` : voir `plan.md`, « Ordre des refus ».

Un rejeu (même `task_id`, même `request_id`, même enveloppe) rend la vue
courante de la tâche, y compris pendant une pause. Il ne crée aucune ronde.

## `bridget_capabilities` - clés ajoutées

| Clé | Valeur |
|---|---|
| `keep_alive_min_secs` | 60 |
| `keep_alive_max_secs` | 7 200 |
| `max_rounds` | 8 |
| `brief_max_bytes` | 16 384 |

## Réponses tardives

Une réponse authentifiée de l'enfant à la mission d'une ronde passée est absorbée
sans effet : pas de changement de la tâche, pas de message ordinaire, pas de
réveil du parent. Même comportement sur les deux chemins d'envoi.

## Messages de remise

Mission : corps = `<dossier rendu>\n\n--- Consigne ---\n<consigne>` avec brief, la
consigne seule sans brief. Résultat de la ronde N >= 2 : identifiant
`native-result-<task_id>-r<N>`, `in_reply_to` = mission de la ronde.

## Lineage T3

Aucun changement de forme. Une ronde ouverte est une mutation visible
(`result_available` vers `mission_pending`). Le lecteur relit la tâche. La fin
explicite d'un enfant inactif ne change pas le `status`.
