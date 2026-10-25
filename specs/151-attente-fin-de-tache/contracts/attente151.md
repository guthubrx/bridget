# Contrat - Attente et boîte de fin de tâche 151

Compatible avec les contrats 148 (`specs/148-identite-delegation/contracts/delegation.md`),
Lineage 149 et 150 (`specs/150-socle-delegation/contracts/delegation150.md`).
Un appel sans champ nouveau produit le même comportement qu'en 150. Les clés de
réponse existantes ne changent pas. `bridget_send` et `bridget_delegate` ne changent pas.

Règle commune : les outils d'attente et `bridget_task_status` sont des lectures
pures. Ils n'écrivent ni `result_sent`, ni `failure_sent`, ni `delivery_acked`, ni
nettoyage, ni message. Le message de résultat part toujours par `tick`.

## Sonde de support (avant toute attente)

Le client MCP envoie `DaemonIdentityRequest` (budget 3 s). La réponse
`DaemonIdentityReport` porte `features`. Sans `"wait_v1"` : refus
`wait_unsupported` aussitôt, aucune trame nouvelle envoyée.

## `bridget_task_wait` - outil nouveau

Annotations : `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`.

| Champ | Type | Obligatoire | Bornes | Défaut |
|---|---|---|---|---|
| `task_ids` | tableau de chaînes | oui | 1 à 16 éléments, sans doublon | - |
| `wait_timeout` | entier (secondes) | non | 1 à 3600 | 600 |

Tout autre champ : refus `InvalidParams` (liste blanche fermée).

### Réponse

```json
{
  "version": 1,
  "timed_out": false,
  "waited_secs": 7,
  "ready": [ { "...entrée de tâche..." } ],
  "pending": ["<task_id>", "..."]
}
```

- `ready` : tâches terminées à l'instant du retour, dans l'ordre de `task_ids`.
- `pending` : identifiants non terminés, dans l'ordre de `task_ids`.
- `timed_out: true` seulement si l'attente a expiré avec `ready` vide.
- Retour immédiat si au moins une tâche est déjà terminée à l'appel.

### Entrée de tâche (`ready[]`, et réponse de `bridget_task_status`)

Clés 148 à 150 inchangées : `version`, `task_id`, `status`, `child_agent_id`,
`message_id`, `agent_type`, `model`, `effort`, `cwd`, `posture`,
`mission_deadline_at`, `result`, `error`.

Clés ajoutées :

| Clé | Type | Valeur |
|---|---|---|
| `work_state` | chaîne | `working`, `waiting_for_children`, `result_available`, `failed`, `cancelled` |
| `summary` | chaîne ou null | début du résultat, 2048 octets au plus ; `null` hors `result_available` |
| `summary_truncated` | booléen | vrai si le résultat dépasse le résumé |
| `result_bytes` | entier ou null | longueur du résultat en octets |
| `detail` | objet ou null | `{"tool":"bridget_task_status","task_id":"…"}` pour `result_available` et `failed` |
| `mailbox` | objet ou null | `{"state":"pending\|claimed\|delivered\|acknowledged"}` ; `null` si pas de boîte |

Dans `ready[]` de l'attente, la clé `result` est absente : le texte entier se lit par
`bridget_task_status` (qui garde `result` complet) ou par le message de résultat.

### Refus (code dans `native_delegation_refused`)

| Code | Quand |
|---|---|
| `wait_unsupported` | le daemon n'annonce pas `wait_v1` |
| `wait_timeout_out_of_range` | `wait_timeout` hors 1 à 3600 |
| `wait_capacity` | 64 attentes déjà actives dans le daemon, ou 4 attentes déjà ouvertes dans la façade MCP du processus |
| `task_unavailable` | un `task_ids` inconnu, purgé ou d'un autre parent |
| `invalid_request` | `task_ids` vide, plus de 16, doublon, mauvais type |

Ordre côté daemon : `invalid_request`, `wait_timeout_out_of_range`, `wait_unsupported`,
`task_unavailable`, `wait_capacity`. Aucun refus n'écrit quoi que ce soit. La capacité de façade peut être refusée avant la validation des arguments, car aucun exécutant n'est alors lancé.

### Comportement pendant l'attente

- Réveil par événement (compteur d'écritures de tâche). Filet interne de 5 s.
- Le client MCP garde sa connexion `wait_timeout + 10` secondes.
- Fermeture du client, annulation MCP de l'appel ou arrêt du daemon : l'attente se
  termine en 6 s au plus. Une annulation MCP ferme le socket daemon de l'appel.
- Pendant la pause du référent : l'attente fonctionne.
- Redémarrage du daemon : l'appel échoue (`daemon_unreachable` côté client). Le
  client peut rappeler. Les messages de résultat repartent par `tick`.
- Résultat vu deux fois : l'attente rend le résumé ; le message de résultat arrive
  ensuite. Il n'est jamais supprimé.

## `bridget_request_wait` - outil nouveau

Annotations : `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`.

| Champ | Type | Obligatoire | Bornes | Défaut |
|---|---|---|---|---|
| `request_id` | chaîne | oui | l'`id` rendu par `bridget_send` | - |
| `wait_timeout` | entier (secondes) | non | 1 à 3600 | 600 |

### Réponse

```json
{
  "version": 1,
  "state": "answered",
  "timed_out_wait": false,
  "waited_secs": 4,
  "reply": { "id": "<message>", "from": "<agent>", "summary": "…", "summary_truncated": false },
  "reply_reason": null
}
```

| `state` | Sens |
|---|---|
| `answered` | la demande a une réponse accusée |
| `open` | `wait_timeout` expiré ; la demande reste ouverte ; `timed_out_wait: true` |
| `timed_out` | l'échéance `reply_timeout` de la demande est passée |
| `cancelled` | la demande a été annulée (`bridget_cancel`) |

`reply` : objet pour `answered` avec résumé figé ; sinon `null`.
`reply_reason` : `"reply_unavailable"` si `state` vaut `answered` et `reply` est
`null` (réponse via guichet, ligne antérieure à 151, enveloppe manquante) ; sinon `null`.
`summary` : 1024 octets au plus, frontière UTF-8, lu dans `tracked_requests.reply_summary`
(écrit avec le passage à `answered`). Aucune lecture du ledger.

Refus : `wait_unsupported`, `wait_timeout_out_of_range`, `wait_capacity` (même
borne de 64), `request_unavailable` (la demande n'est pas à l'appelant ou
n'existe pas), `invalid_request`.

Reprise : si l'hôte coupe l'appel, rappeler `bridget_request_wait` avec le même
`request_id`. Aucun envoi n'a lieu. La réponse est visible aussi par le chemin normal
d'injection (limite connue).

## `bridget_task_ack` - outil nouveau (P2, informatif)

Annotations : `readOnlyHint: false`, `destructiveHint: false`, `idempotentHint: true`.

| Champ | Type | Obligatoire | Bornes |
|---|---|---|---|
| `task_ids` | tableau de chaînes | oui | 1 à 16, sans doublon |

Écrit `acknowledged_at` (une fois) sur chaque tâche `result_available` ou `failed`
du propriétaire. Refus pour tout l'appel : `task_unavailable`, `task_not_ready`
(tâche non terminée, `cancelled`), `wait_unsupported`, `invalid_request`.
Réponse : `{"version":1,"acknowledged":[ids],"already":[ids]}`.
Effet : `mailbox.state = acknowledged`. Aucun autre : ni message, ni nettoyage, ni
purge, ni Lineage.

## Trames de protocole (`bridget-transport`)

| Trame | Contenu |
|---|---|
| `NativeDelegationRequest::Wait` | `task_ids: Vec<String>`, `wait_timeout: Option<u64>` |
| `NativeDelegationRequest::Ack` (P2) | `task_ids: Vec<String>` |
| `WrapperToDaemon::RequestWait` | `request_id: String`, `wait_timeout: u64` |
| `DaemonToWrapper::RequestWaitResult` | `state`, `timed_out_wait`, `waited_secs`, `reply: Option<…>`, `reply_reason: Option<String>` |
| `DaemonToWrapper::DaemonIdentityReport` | champ ajouté `features: Vec<String>` (`#[serde(default)]`) |

Les variantes existantes gardent leurs octets canoniques. Un ancien daemon ignore
sans réponse les trames nouvelles : c'est pourquoi la sonde les précède.

## `bridget_capabilities` - clés

| Clé | Valeur |
|---|---|
| `mission_reply_limit_secs`, `mission_reply_min_secs`, `mission_reply_max_secs` | inchangées (150) |
| `wait_timeout_default_secs` | 600 (nouveau) |
| `wait_timeout_max_secs` | 3600 (nouveau) |

## Lineage adaptateur de bureau

Aucun changement de forme ni de génération. `acknowledged_at` n'est pas projeté.

Précisions vérifiées lors de la contre-revue :
- Une phase `orphaned` ou absente après un envoi engagé rend `claimed`, sans
  prétendre prouver la remise.
- Une demande encore `open` dont l'échéance est dépassée rend `timed_out`
  calculé. L'attente ne modifie pas la ligne.
- Le premier passage à `answered` fixe le résumé ; si le guichet clôture avant
  une réponse de message, le résumé reste absent (`reply_unavailable`).
- Le socket de sonde est annulable au même titre que le socket d'attente.
