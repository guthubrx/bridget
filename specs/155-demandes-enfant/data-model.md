# Modèle de données - Session155 (révisé après contre-revue)

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `child_requests` | `Option<String>` (`"deny"` / `"forward"`) | `deny` | Mode choisi à la délégation, figé |

Aucun nouvel état de tâche. La vue Lineage T3 ne change pas.

## Table `native_child_requests` (base de la saga native)

```sql
CREATE TABLE IF NOT EXISTS native_child_requests(
  request_id TEXT PRIMARY KEY,          -- sha256(task_id | child_instance | provider_session | turn_id | provider_request_id), 16 octets en hex
  task_id TEXT NOT NULL,
  child_instance TEXT NOT NULL,         -- instance attestée de l'enfant (connexion qui a envoyé la trame)
  provider_session TEXT NOT NULL,
  turn_id TEXT NOT NULL,
  provider_request_id TEXT NOT NULL,
  content_hash TEXT NOT NULL,           -- empreinte du contenu normalisé ; un doublon au contenu différent est refusé
  kind TEXT NOT NULL CHECK (kind IN ('approval_command','approval_file','elicitation_form','user_input')),
  verdict TEXT NOT NULL CHECK (verdict IN ('parent','human','forbidden')),
  state TEXT NOT NULL CHECK (state IN ('open','answered','expired','cancelled','refused')),
  outcome TEXT,                          -- answered_by_parent | answered_by_human | expired | cancelled | refused:<code>
  summary TEXT NOT NULL,                 -- <= 400 caractères, nettoyé
  facts_json TEXT NOT NULL,              -- faits vérifiés par Bridget (voir plus bas), <= 4 Kio
  model_text TEXT NOT NULL,              -- texte du modèle cité (reason, question), <= 1 Kio, marqué non fiable
  offered_json TEXT NOT NULL,            -- décisions proposées par le fournisseur, filtrées à accept/decline/cancel
  created_at INTEGER NOT NULL,
  deadline_at INTEGER NOT NULL,
  decision TEXT,                         -- accept | decline | cancel (HISTORIQUE)
  answer_json TEXT,                      -- <= 8 Kio, jamais de secret
  decided_by TEXT,                       -- parent | human | system
  decided_at INTEGER,
  delivery_state TEXT NOT NULL DEFAULT 'none' CHECK (delivery_state IN ('none','pending','sent','delivered','void','unknown')),
  void_reason TEXT,                      -- cancelled | paused | revoked | instance_changed | restart | task_ended | expired | closed_before_apply
  sent_at INTEGER,                       -- posé AVANT l'envoi de la trame
  delivered_at INTEGER,
  inbox_item_id TEXT,
  inbox_settled INTEGER NOT NULL DEFAULT 0,  -- 1 quand l'item est fermé ou acquitté
  UNIQUE(task_id, child_instance, provider_session, turn_id, provider_request_id));
CREATE INDEX IF NOT EXISTS native_child_requests_open ON native_child_requests(state, deadline_at);
CREATE INDEX IF NOT EXISTS native_child_requests_task ON native_child_requests(task_id, state);
CREATE INDEX IF NOT EXISTS native_child_requests_inbox ON native_child_requests(inbox_settled, state);
```

Règles :

- La décision (`decision`) est un fait historique. Seule `delivery_state = pending`
  autorise un envoi. Avant l'envoi, le daemon contrôle FR-027 (tâche active, instance,
  pause, révocation, ÉCHÉANCE) puis écrit `sent` et `sent_at` ; ensuite seulement il
  pousse la trame. Un plantage entre les deux laisse `sent` : jamais `pending`.
- `delivered` : accusé positif (`applied = true`) de la connexion de `child_instance`.
- `void` : posé seulement si l'accord n'a jamais été envoyé (`pending`), ou si le wrapper
  confirme `closed_before_apply`. `void` ne revient jamais à `pending`.
- `unknown` : accord `sent` sans accusé quand la tâche se ferme et que le wrapper ne
  répond plus (processus disparu, redémarrage). Le daemon ne dit jamais « annulé » dans
  ce cas : l'issue stockée est `cancelled_outcome_unknown`.
- Fermeture d'une demande `sent` : le daemon envoie `ChildRequestCancel` et attend
  `ChildRequestClosed` ; `already_applied` donne `delivered`, `closed_before_apply`
  donne `void`. Tant que l'état est `sent`, le renvoi reste identique (idempotent côté
  wrapper).
- Échéance : `now >= deadline_at` interdit tout envoi ; à l'échéance, `pending` devient
  `void` (`expired`), `sent` suit la fermeture ci-dessus.
- Un accusé reçu d'une autre connexion est ignoré et journalisé.
- Au plus 4 lignes `open` par tâche et 32 au total par tâche.
- Un état terminal ne change plus. Une réponse rejouée à l'identique rend la ligne.
- Un doublon de trame au même `content_hash` rend la ligne. Un autre `content_hash`
  rend `request_conflict` et la ligne existante est inchangée.
- `inbox_settled = 0` avec un état terminal : le daemon ferme (item ouvert) ou acquitte
  sans exécuter (item résolu) à chaque tick, puis passe à 1. Au plus 512 lignes lues.
- Supprimée avec l'arbre de la tâche, dans la transaction de purge de la session 150.

## Faits vérifiés (`facts_json`)

Calculés par Bridget, jamais copiés tels quels du fournisseur.

| Clé | Contenu |
|---|---|
| `method` | méthode fournisseur retenue |
| `duration` | `once` (seule valeur acceptée en v1) |
| `command` | commande citée, bornée à 1 Kio (approbation de commande) |
| `paths_cited` | chemins cités par le fournisseur, au plus 64, nettoyés, marqués non vérifiés ; ils ne fondent aucune décision (FR-011) |
| `network` | `true` ou `false` |
| `child_posture` | `development` ou `read_only` |
| `parent_sandbox` | `dangerFullAccess`, `workspaceWrite` (avec zones) ou autre |
| `verdict_code` | motif du verdict |

## Table des attentes du transport (mémoire du wrapper)

| Champ | Rôle |
|---|---|
| identifiant JSON-RPC, session, tour | clé de rapprochement avec `ChildRequestAnswer` |
| état `waiting` / `answered` / `closed` | transitions sous un seul verrou avec l'écriture de la réponse |
| anneau des 64 dernières demandes réglées | répondre exactement à un renvoi ou à une fermeture tardive |
| méthode | choisit la forme de la réponse |
| décisions proposées | pour écrire `decline` ou `cancel` seulement |
| identifiants de question | pour valider `answer` |

Au plus 4 entrées ouvertes. Vidée à la fin du tour, à l'interruption et à l'arrêt. Une réponse
du daemon qui ne correspond à aucune entrée exacte donne `applied = false`.

## Projection dans `bridget_task_status`

```json
"pending_requests": [{"request_id":"…","kind":"user_input","summary":"…","verdict":"parent","deadline_at":0}],
"recent_requests":  [{"request_id":"…","kind":"…","outcome":"answered_by_parent","decision":"accept","decided_at":0}]
```

`recent_requests` : 10 dernières issues terminales. Clés ajoutées, aucune retirée. Les
demandes `human` figurent dans la liste (le parent sait qu'une demande attend l'humain)
sans pouvoir y répondre.

## Item de boîte humaine (approbations `Human` seulement)

| Champ | Valeur |
|---|---|
| `kind` | `intervention_required` |
| `dedup_key` | `native-child-request:<request_id>` |
| `subject` | `delegation_id = task_id`, `message_id = mission`, `agent_id = enfant` |
| `context` | faits vérifiés d'abord, puis texte du modèle cité et marqué non fiable (moins de 16 Kio) |
| `options` | `accept`, `decline` |
| `producer` | `daemon` |

## Verdicts (`classify_request`, fonction pure sur des faits déjà résolus)

Entrées : politique figée du parent (`snapshot.parent.provider_policy`), politique
figée de l'enfant (`snapshot.child_policy`), demande normalisée avec chemins déjà
résolus (FR-011), indicateurs de complétude. Sortie : `Parent`, `Human` ou
`Forbidden(code)`.

| Cas | Verdict |
|---|---|
| question `user_input` ou formulaire sans secret, parent vivant, prouvé, non révoqué | Parent |
| question, parent absent ou révoqué | Forbidden(`parent_unavailable`) |
| question à secret | Forbidden(`secret_not_forwardable`) |
| élicitation en mode URL | Forbidden(`url_mode_unsupported`) |
| méthode `item/permissions/requestApproval` ou autre méthode inconnue | Forbidden(`approval_method_unsupported`) |
| approbation, détails incomplets, tronqués ou champ d'effet inconnu | Forbidden(`details_incomplete`) |
| approbation, enfant `readOnly` | Forbidden(`child_restricted`) |
| approbation, politique du parent incomplète, `readOnly` ou `externalSandbox` | Forbidden(`parent_limit_unknown`) |
| écriture de fichier, parent `workspaceWrite`, quel que soit le chemin | Forbidden(`file_confinement_unproven`) |
| écriture de fichier, parent `dangerFullAccess` | Human |
| réseau, parent `workspaceWrite` avec `networkAccess` vrai, ou `dangerFullAccess` | Human |
| réseau, `networkAccess` faux ou absent | Forbidden(`network_not_granted`) |
| commande, parent `dangerFullAccess` | Human |
| commande, parent `workspaceWrite` | Forbidden(`command_unbounded`) |
| demande mixte (commande + réseau + écriture) | le plus restrictif de ses effets |
| durée autre que `once` | Forbidden(`duration_unsupported`) |

Cette table est le contrat testé par T006 et T008 (au moins 24 cas).
