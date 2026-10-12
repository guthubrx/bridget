# Contrat - Contrôle des délégations 158a

Compatible avec la base157 livrée et les contrats148–157. `bridget_delegate`
ne change pas de schéma en158A : les ajouts151–157, dont `fallback` et
`substitution`, sont conservés. Hors arrêt posé, un appel sans ajout158 produit
les mêmes octets canoniques et le même comportement que cette base. Le reçu157
canonique et ses empreintes seront vérifiés/renseignés par ROOT avant le GO158A ;
aucun SHA157 livré n'est supposé dans cette préparation.

## Refus ajoutés

| Code | Quand | Écriture |
|---|---|---|
| `delegations_halted` | arrêt global posé, nouvelle clé (renvoyé dans `native_delegation_refused`) | aucune ; la clé reste libre |
| `capability_denied` (MCP) | appel hors plafond de l'appelant | aucune |

Ordre des refus de `bridget_delegate` (base157 + 158a) : `identity_unavailable`,
`invalid_request`, `reply_timeout_out_of_range`, rejeu (`envelope_mismatch`,
`request_expired`), décision commune d'admission de Delegate et Continue (`delegations_halted`,
`referent_paused`), gardes de droits du parent149, sélection de cible157
(type/modèle/effort, préflight, politique, options, quota ; listefermée de
remplacement conservée), `task_limit`, `retention_saturated`,
lignée parente, `reply_timeout_exceeds_parent`.

Un rejeu de Delegate ou Continue pendant l'arrêt rend la vue courante de la
tâche existante, sans mutation. `suspended` reflète la pause courante : cette
vue n'est pas immuable. Seuls les reçus `DelegationHaltSet` rendent une
réponse historique enregistrée.

## `bridget_capabilities` - clé ajoutée

`delegations_halted` : booléen, état courant de l'arrêt.

## Protocole de contrôle (socket Unix)

Toutes les requêtes exigent la négociation `ControlStateV1` ET un acteur humain
attesté (`control_client_actor`). `ControlStateV1` seule ne suffit pas : elle
est accordée à tout client qui la demande. Pour une requête correctement
formée, appliquer dans cet ordre : négociation absente →
`negotiation_required` ; capacité absente → `capability_not_negotiated` ;
sinon acteur humain absent → `human_actor_required`. Réemployer les gardes
existantes ; un alias `from=humain` ne fournit aucune autorité.

| Requête | Réponse |
|---|---|
| `DelegationOverviewRead { version, state: active\|all, limit 1..100 }` | `DelegationOverview { halted, since?, by?, reason?, items[], total_active }` |
| `DelegationHaltSet { version, command_id, halted: bool, reason?, cancel_active: bool }` | `DelegationHaltState { halted, since, by, reason, generation, cancelled: n, replayed: bool }` |
| `DelegationCancel { version, task_id, request_id }` | `DelegationOverview` (ligne mise à jour) |

Refus fermés (`DelegationControlRejected { reason }`) : `unsupported_version`,
`invalid_request`, `negotiation_required`, `capability_not_negotiated`,
`human_actor_required`, `task_unavailable`, `store_unavailable`.

`cancel_active` n'est admis que si `halted = true`. `command_id` connu : la
réponse enregistrée est rendue (`replayed: true`), sans réécriture et sans aucun
parcours d'annulation. L'annulation ne s'exécute qu'à la première application
et vise les arbres actifs à cet instant. Elle est au plus une fois.

`cancelled` compte les racines uniques des arbres actifs visés par la demande,
figées avant le commit (zéro sans `cancel_active`). Les racines choisies sont
les tâches actives sans ancêtre actif. Ce nombre ne prouve pas des arrêts
physiques confirmés. Un callback `FnOnce` n'est évalué qu'à la première
application ; le rejeu ne parcourt rien. Le verrou daemon reste tenu jusqu'à
`human_cancel`. La perte partielle après crash reste admise.

Un refus `store_unavailable` à `DelegationHaltSet` peut précéder le commit ou
suivre une annulation refusée après le commit. Il ne prouve donc ni l'absence
ni la présence d'un effet : l'arrêt ou les annulations ont pu être appliqués.
La CLI signale cette incertitude et sort avec le code 1. L'humain consulte
`bridget control delegations`, puis utilise `delegations cancel` pour les
tâches restantes. Le rejeu d'un reçu connu n'émet aucune nouvelle annulation.

Le rejeu porte les valeurs historiques `halted`, `since`, `by`, `reason`,
`generation` et `cancelled`, sans modifier l'état courant. La vue globale et
`control status` donnent les métadonnées courantes ; `since`, `by`, `reason`
sont facultatifs. Pas de variante supplémentaire.

Les refus de négociation/capacité réutilisent `ClientRejected` ; les autres
refus de contrôle réutilisent `DelegationControlRejected`. Aucun nouvel ID
de test ou scénario. L'arrêt bloque les nouvelles Delegate/Continue, pas les
missions déjà queued/en cours sauf `cancel_active`. Les demandes d'enfant
ouvertes et leurs réponses gardent la garde de pause seule.

## CLI

```text
bridget control delegations [list] [--state active|all] [--limit N] [--json]
bridget control delegations cancel <task_id>
bridget control halt-delegations [--reason <texte>] [--cancel-active]
bridget control resume-delegations
bridget control status           # ajoute la ligne d'arrêt s'il est posé
```

`halt-delegations`, `resume-delegations` et `delegations cancel` exigent un
terminal interactif. Codes de sortie : 0 succès, 1 refus ou erreur daemon, 2 usage.
`control status` sort aussi avec le code 1 si la lecture des données d'arrêt
échoue, même après l'affichage de l'état de contrôle087. Un affichage partiel
ne confirme pas le succès de la commande.

## Portées MCP (`mcp_scopes.rs`)

| Outil | Base | Lecture | État propre |
|---|---|---|---|
| `bridget_capabilities`, `bridget_who`, `bridget_status`, `bridget_control_status`, `bridget_task_status`, `bridget_read_artifact`, `bridget_ledger`, `guichet_request_status` | `reads` | toutes | - |
| `bridget_rename`, `bridget_dnd`, `bridget_domain`, `bridget_runtime` | `writes_own_state` | - | toutes |
| `bridget_events` | `writes_own_state` | `types`, `list` | `sub`, `unsub` |
| `bridget_thread` | `writes_threads` | `list`, `show`, `history` | `read`, `ack` |
| `bridget_journal` | `sends` (abaissé à `reads` sans `to`) | sans `to` | - |
| `bridget_handoff` | `sends` | `preview` | - |
| `bridget_send`, `bridget_cancel` | `sends` | - | - |
| `bridget_publish_artifact` | `publishes` | - | - |
| `bridget_delegate`, `bridget_task_cancel`, `guichet_delegate`, `guichet_registre_add`, `guichet_objective_close` | `delegates` | - | - |
| outils ajoutés par 151 à 157 | à classer en T015 | | |

Cette table est indicative. Le test exhaustif fait foi. Aucune ligne n'a de
portée « contrôle humain ».

`readOnlyHint` = vrai si et seulement si la portée de base est `reads`. Pour
`bridget_journal`, la base est `sends` : `readOnlyHint` est faux.

Plafond `read_only` (enfant `discovery`) : seuls `reads` et `writes_own_state`.
Ruptures voulues (politique FR-029) : refusés `bridget_thread` `post`, `create`,
`add_members`, `close` ; `bridget_journal` avec `to` ; `bridget_publish_artifact` ;
`bridget_send`, `bridget_cancel`, `bridget_handoff` en envoi ; toute délégation
(`capability_denied`, « outil hors du plafond de cette tâche, posture
discovery »). Restent permis : `read` puis `ack` de fil (pas de rejeu de page),
`bridget_read_artifact`, et la réponse finale automatique au parent, qui ne
passe pas par un outil MCP.

## Compatibilité

- Un appelant sans tâche `discovery` garde tous ses outils.
- La vue Lineage T3 ne change pas.

## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.

## Plafond interne, sans outil MCP ajouté

`NativeDelegationRequest::CallerScope {}` : identité inscrite exigée ; lookup
indexé dans le magasin du daemon visé. L'enveloppe `NativeDelegationResult`
rend un objet `result: { version: 1, caller_read_only: bool }` : vrai pour `read_only`, faux pour `standard` vérifié.
Ce champ n'a aucun défaut : RPC indisponible, réponse invalide ou erreur SQL
ferment chaque appel MCP avec effet ou lecture du daemon avant son effet.
Exception pure locale : `bridget_handoff` `action=preview` réemploie son
parseur strict et son rendu avant le RPC. Aucun accès DB, aucun identifiant
créé et aucun transport ; Reads est permis pour toutes les postures, donc
aucune tâche ou plafond n'est lu. C'est le contrat local103 conservé, pas un
repli après panne. Aucun autre appel de lecture dépendant de la socket
n'échappe à CallerScope. Pas d'accès DB côté client, pas
de repli standard. La CLI du même compte garde la borne ADR011 et ne devient
pas confinée par ce plafond MCP. Pour `bridget_journal`, `to` absent ou nul
abaisse à Reads ; toute autre valeur, même vide, garde Sends. Les annotations
`readOnlyHint` dérivent de la base ; leur absence n'équivaut pas à vrai.
