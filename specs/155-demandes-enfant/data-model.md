# Modèle de données - Session155 (révisé après contre-revue)

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `child_requests` | `Option<String>` (`"deny"` / `"forward"`) | `deny` | Mode choisi à la délégation, figé |

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/155-demandes-enfant/data-model.md#extrait-1).

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
  offered_json TEXT NOT NULL,            -- choix internes fermés et offres explicites si la méthode en porte ; ne pas inventer availableDecisions
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
  donne `void`. Avant chaque renvoi `sent`, le daemon refait FR027. Expiration et fermeture sont traitées avant le renvoi ; une invalidité envoie Cancel, jamais l’accord. Le renvoi encore valide reste identique (idempotent côté wrapper).
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
| forme de réponse par méthode | commande/élicitation refusée : choix fermés ; user_input : answers enveloppé, refus JSON-RPC |
| identifiants de question | pour valider `answer` |

Au plus 4 entrées ouvertes. Vidée à la fin du tour, à l'interruption et à l'arrêt. Une réponse
du daemon qui ne correspond à aucune entrée exacte donne `applied = false`.

## Projection dans `bridget_task_status`

```json
"pending_requests": [{"request_id":"…","kind":"user_input","summary":"…","verdict":"parent","deadline_at":0}],
"recent_requests":  [{"request_id":"…","kind":"…","outcome":"answered_by_parent","decision":"accept","decided_at":0,"delivery_state":"delivered"}]
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

## Verdicts (`classify_request`, fonction pure sur des faits structurés et chemins cités)

Entrées : politique figée du parent (`snapshot.parent.provider_policy`), politique
figée de l'enfant (`snapshot.child_policy`), demande normalisée avec chemins cités et normalisés lexicalement, sans
résolution filesystem ni preuve de confinement (FR-011), indicateurs de complétude. Sortie : `Parent`, `Human` ou
`Forbidden(code)`.

| Cas | Verdict |
|---|---|
| question user_input avec isSecret=false (défaut normatif), parent vivant/prouvé/non révoqué | Parent |
| question, parent absent ou révoqué | Forbidden(`parent_unavailable`) |
| formulaire MCP tiers, quel que soit type/contenu/meta | Forbidden(`approval_method_unsupported`) |
| question à secret | Forbidden(`secret_not_forwardable`) |
| élicitation en mode URL | Forbidden(`url_mode_unsupported`) |
| méthode `item/permissions/requestApproval` ou autre méthode inconnue | Forbidden(`approval_method_unsupported`) |
| approbation, détails incomplets, tronqués ou champ d'effet inconnu | Forbidden(`details_incomplete`) |
| approbation, enfant `readOnly` | Forbidden(`child_restricted`) |
| approbation, politique du parent incomplète, `readOnly` ou `externalSandbox` | Forbidden(`parent_limit_unknown`) |
| écriture de fichier, parent `workspaceWrite`, quel que soit le chemin | Forbidden(`file_confinement_unproven`) |
| faits synthétiques complets d’écriture, durée once, parent dangerFullAccess | Human dans le classifier pur ; transport fileChange0.162.1 réel refusé en amont |
| réseau, parent `workspaceWrite` avec `networkAccess` vrai, ou `dangerFullAccess` | Human |
| réseau, `networkAccess` faux ou absent | Forbidden(`network_not_granted`) |
| commande, parent `dangerFullAccess` | Human |
| commande, parent `workspaceWrite` | Forbidden(`command_unbounded`) |
| demande mixte (commande + réseau + écriture) | le plus restrictif de ses effets |
| durée autre que `once` | Forbidden(`duration_unsupported`) |

Cette table est le contrat testé par T006 et T008 (au moins 24 cas).

## Précisions de la contre-revue plan R1

La décision historique et sa remise ont deux contrôles distincts. L’écriture de
la décision exige un CAS state = open et deadline_at > now. Le claim de remise
exige delivery_state = pending et FR027 valide. Chaque opération vérifie rowcount ;
zéro ligne modifiée n’autorise aucun envoi. Un état terminal ne rouvre pas, mais
sa delivery_state peut encore être réglée par un accusé de fermeture.

Une fermeture fournisseur non confirmée envoie ChildRequestCancel même pour
none ou pending. void signifie accord invalidé, sans prouver que le fournisseur
a quitté son attente. Closed confirme le règlement ; sans réponse, l’issue ne
prétend pas l’annulation confirmée. Une répétition terminale rend la réponse
persistée ; aucun accord void n’est réappliqué.

Pendant une pause, une décision humaine est acquittée sans exécution. La reprise
ne remet pas cette décision ; une nouvelle demande sera nécessaire.

L’attente151 reste une lecture d’état. ready contient uniquement les tâches
terminales et pending les identifiants nonterminaux. La clé pending_requests à la
racine du résultat wait contient les demandes Parent/open/nonexpirées/actionnables
avec task_id et projection155. Une question déjà ouverte peut rendre immédiatement
la main. Human est exclu de cette liste et du prédicat de retour, tout en restant
visible via status. timed_out est faux si ready ou pending_requests est non vide ;
il est vrai seulement à l’échéance wait avec les deux listes vides.
Aucune mémoire persistante de waiter, aucun marqueur de consommation. WaitEpoch
est notifié après commit Parent ; le compteur est lu avant l’état, comme en151.
« Nouvellement ouverte » décrit une notification, pas un filtre des demandes.

Le checkpointR1 demandait la lecture elicitation/approval/_meta ; le schéma 0.162.1 est désormais lu, avec T002 final PASS attestée dans validation/gate-schema155.json. Un formulaire tiers
vide ou marqué approval, ou une métadonnée d’effet non établi, reste Forbidden ;
l’exception du formulaire vide Bridget reste distincte. Ces variantes sont
intégrées à la table existante d’au moins 24 cas, pas à une nouvelle série de tests.

## Bornes du schéma courant 0.162.1

fileChange réel :approval_method_unsupported en v1, duration_unsupported si
grantRoot non nul. Params n’a ni paths ni availableDecisions. Les notifications
changes/path/diff/kind existent, sans preuve de confinement ni route Human
automatique ; aucune table/cache de corrélation supplémentaire. Le verdict pur
Human pour facts synthétiques complets ne signifie pas support du transport réel.

permissions est refusée par erreur JSON-RPC, jamais decision:decline. Pour
user_input, la réponse fournisseur est {answers:{questionId:{answers:[string]}}}.
La réponse interne du parent reste answer[questionId]=[strings] ; le transport
adapte chaque valeur. Un refus ou une expiration user_input utilise l’erreur
JSON-RPC -32000 child_request_refused, car aucun decline/cancel n’est défini.

Elicitation0.162.1 définit cinq modes : form,openai/form,openaiForm,url et
openai/userVerification. Aucun formulaire MCP tiers n’est routable Parent : approval_method_unsupported, même pour form typé ; _meta non vide ou d’effet non établi et modes enrichis sont refusés.
_meta est un JSON arbitraire, pas une clé approval/durée attestée. Aucun champ
structuré fictif n’est ajouté aux faits. Exception vide Bridget historique distincte.
serverRequest/resolved ne porte pas turnId ; rapprochement par la table existante.

Le schéma form MCP n’a pas de drapeau isSecret standard : ne pas déduire une absence de secret de ce seul silence. Réponse de refus fileChange/permissions/user_input : erreur JSON-RPC -32000 child_request_refused sans result, motif interne distinct. Closed atteste le règlement sérialisé du wrapper ; l’erreur JSON-RPC seule ne prouve aucune fermeture runtime du fournisseur.
