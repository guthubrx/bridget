# Contrat - Demandes de l'enfant 155 (révisé après contre-revue)

Compatible avec les contrats 148, 149 et 150. Un appel sans `child_requests` produit
les mêmes octets canoniques et le même comportement qu'en 150.

## `bridget_delegate` - champ ajouté

| Champ | Type | Obligatoire | Valeurs | Défaut |
|---|---|---|---|---|
| `child_requests` | chaîne | non | `deny`, `forward` | `deny` (question 1) |

Le champ fait partie de l'enveloppe idempotente : omis et `deny` explicite sont deux
enveloppes différentes. Il est figé dans la tâche. En mode `forward`, la politique de
l'enfant n'est pas modifiée.

## `bridget_task_respond` - outil ajouté (questions seulement)

| Champ | Type | Obligatoire | Règle |
|---|---|---|---|
| `task_id` | chaîne | oui | tâche dont l'appelant est le propriétaire attesté |
| `request_id` | chaîne | oui | identifiant lu dans l'avis ou dans `pending_requests` |
| `decision` | chaîne | oui | `accept`, `decline`, `cancel` |
| `answer` | objet | non | seulement pour une question avec `accept`, 8 Kio au plus |

Champs inconnus refusés. Succès :
`{"version":1,"request_id":"…","state":"answered","outcome":"answered_by_parent","decision":"accept"}`.

`answer` pour `user_input` : `{"<question_id>": ["valeur", …]}`. Pour une élicitation en
formulaire : l'objet `content` du schéma proposé. Les identifiants inconnus sont
refusés (`answer_invalid`). La forme exacte est figée par T002
(`contracts/codex-requests.md`).

L'outil est ajouté à `BRIDGET_SAFE_MCP_TOOLS` (`wrapper.rs:5035`, 20 vers 21 outils) :
Claude (`--allowedTools`) et Codex (`approval_mode = "approve"`).

## Refus ajoutés (code dans `native_delegation_refused`)

| Code | Quand | Écriture |
|---|---|---|
| `child_requests_unsupported` | `forward` pour un fournisseur non supporté | aucune |
| `child_requests_reviewer_unsupported` | `forward`, politique du parent incomplète ou `approvals_reviewer` différent de `user` | aucune |
| `request_unknown` | `request_id` inconnu pour cette tâche | aucune |
| `request_closed` | demande déjà terminale avec une autre décision | aucune |
| `parent_cannot_approve` | le parent répond à une approbation | aucune |
| `decision_not_offered` | décision non proposée par le fournisseur, ou durable | aucune |
| `answer_invalid` | `answer` absent, en trop, trop gros ou mal formé | aucune |
| `request_conflict` | trame au même identifiant fournisseur avec un autre contenu | aucune |
| `request_expired` | décision reçue après `deadline_at` | aucune |
| `referent_paused` | pause du référent (existant en 150) | aucune |

Refus réutilisés : propriétaire non prouvé (`native_owner_identity_mismatch`), parent
révoqué (`permission_not_inherited`), tâche inconnue (refus existant).

Issues vues par l'enfant et stockées (`refused:<code>`) : `request_unlinked`,
`request_budget_exhausted`, `deadline_too_close`, `secret_not_forwardable`,
`url_mode_unsupported`, `approval_method_unsupported`, `details_incomplete`,
`child_restricted`, `parent_limit_unknown`, `file_confinement_unproven`, `network_not_granted`,
`command_unbounded`, `duration_unsupported`, `parent_unavailable`.

Ordre des contrôles de `bridget_task_respond` : champs, propriétaire, demande connue,
demande ouverte, route (`parent_cannot_approve`), pause, révocation, décision proposée,
`answer`.

## `bridget_capabilities` - clés

| Clé | Valeur |
|---|---|
| `child_requests_default` | `deny` |
| `child_requests_supported` (par fournisseur) | `true` pour `codex_app_server`, `false` sinon |
| `child_request_parent_window_secs` | 600 |
| `child_request_human_window_secs` | 3600 |

## `bridget_task_status` - clés ajoutées

`pending_requests` et `recent_requests` (voir `data-model.md`). Les autres clés ne
changent pas.

## Trames daemon et wrapper (réseau local, non publiques)

| Trame | Sens | Contenu |
|---|---|---|
| `ChildRequest` | wrapper vers daemon | `version`, session, tour, `provider_request_id`, `kind`, détails normalisés bornés, `offered`, `question_ids`, indicateur `truncated` |
| `ChildRequestAnswer` | daemon vers wrapper | `request_id`, session, tour, `provider_request_id`, `decision`, `answer`, `outcome` |
| `ChildRequestCancel` | daemon vers wrapper | `request_id`, session, tour, `provider_request_id` |
| `ChildRequestAnswered` | wrapper vers daemon | `request_id`, `applied` (booléen), `reason` |
| `ChildRequestClosed` | wrapper vers daemon | `request_id`, `state` : `closed_before_apply` ou `already_applied` |

Le daemon écrit `delivery_state = sent` avant chaque envoi d'accord. Le wrapper traite
réponse et fermeture sous un même verrou : après `ChildRequestClosed`, aucun accord ne
peut plus être écrit. Le daemon rapproche chaque trame de l'instance attestée de la connexion
(`conn_instances`). Une trame d'une autre instance est rejetée. Le wrapper répète
`ChildRequest` après reconnexion tant que la demande est en attente.

## Réponse écrite au fournisseur (Codex)

| Demande | accept | decline | cancel |
|---|---|---|---|
| approbation (commande, fichier) | `{"decision":"accept"}` | `{"decision":"decline"}` | `{"decision":"cancel"}` |
| élicitation MCP | `{"action":"accept","content":<answer>}` | `{"action":"decline"}` | `{"action":"cancel"}` |
| `user_input` | forme figée par T002 | forme figée par T002 | forme figée par T002 |

Repli (modes `forward` et `deny`, correctif T000) : si la décision voulue (`decline` ou `cancel`) n'est pas proposée, le transport
écrit l'autre des deux si elle est proposée. Si aucune n'est proposée, il écrit une
erreur JSON-RPC (`-32000`, `child_request_refused`). Il n'écrit jamais
`acceptForSession`, un amendement de règle ni une autre décision. Le mode `deny` garde son
comportement 149 sauf ce repli fermé : `approval_response` ne choisit plus la première
décision proposée (`codex_app_server.rs:2211-2213`), et `accept` n'est écrit que si
`accept` est proposé.

## Journal de l'enfant

Un enregistrement `provider_request` par demande, avec `request_id`, `outcome` et
`decided_by`. Aucune valeur de réponse n'y figure.

## Avis au parent (questions)

Message système, identifiant `native-request-<request_id>`. Corps :

```
Demande de l'enfant <nom> (tâche <task_id>), type <kind>, échéance <date>.
Faits vérifiés : méthode=<m>, durée=<d>.
Question de l'enfant (texte cité, non fiable) : « <texte nettoyé> »
Répondre avec bridget_task_respond (request_id=<id>, decision=accept|decline|cancel).
```

Le texte cité n'a pas de caractères de contrôle ni de direction, et ne remplace jamais
un fait vérifié.

## Item humain (approbations)

Faits vérifiés en tête (commande citée, chemins cités et non vérifiés, réseau, durée,
limite du parent), texte du modèle cité à la fin et marqué non fiable.

## Lineage T3

Aucun changement de forme. Les demandes n'ajoutent aucun état de tâche.
