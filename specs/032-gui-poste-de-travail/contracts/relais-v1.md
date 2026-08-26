# Contrats du relais UI — v1

**Figés le 2026-08-25 21h30.** Tout lot qui produit ou consomme ces formes s'y
tient. **Une divergence est un défaut, pas une variante.**

> **Principe qui commande tout : LE RELAIS TRANSMET DES FAITS, LA PAGE DÉCIDE
> DE L'AFFICHAGE.** Aucune de ces formes ne contient de mise en forme, de
> couleur, de classe CSS ni de texte destiné à être affiché tel quel.

## Existant — ne pas redéfinir

| Route | Méthode | Rôle |
|---|---|---|
| `/v1/snapshot` | GET | état complet (`UiSnapshotV1`) |
| `/v1/watch` | GET (SSE) | poussée temps réel |
| `/v1/journal` | GET | journal d'un agent (`UiJournalEventV1`) |

**Authentification** : jeton `UiRelayConfig.token` — **existe déjà**, s'applique
aux nouvelles routes sans rien concevoir.

## C1 — Envoyer un message *(lot L1)*

```
POST /v1/send
```

```json
{
  "version": 1,
  "to": "rc1",
  "body": "texte libre",
  "reply": false
}
```

| champ | type | contrainte |
|---|---|---|
| `version` | entier | `1` |
| `to` | chaîne | nom d'agent **existant** ; grammaire validée en amont |
| `body` | chaîne | non vide |
| `reply` | booléen | `true` crée une **demande suivie** qui peut expirer |

**Réponse 202** :

```json
{ "version": 1, "delivery_id": "...", "issued_at": 1787670000, "status": "in_flight" }
```

**Erreurs** — code stable, jamais de prose libre :

| code | quand |
|---|---|
| `unknown_recipient` | destinataire absent de l'annuaire |
| `invalid_body` | corps vide ou champ manquant |
| `agent_stopped` | agent arrêté — la page propose de le relancer |
| `daemon_unavailable` | socket injoignable |

> **`status: "in_flight"` signifie INJECTÉ, pas LU.**
> Le contrat 012 garantit *exactly-one-injection*, **pas** *exactly-one-consumption*.
> **Interdit d'afficher « reçu ».** Vocabulaire imposé : **injecté / en vol**.
> Voir `etude/preuve-de-remise-honnete-quatre-niveaux` au registre.

## C2 — Trace inter-agents *(lot L2)*

Ajout à `UiSnapshotV1` et poussé par `/v1/watch`. **Pas de route nouvelle.**

La projection est relative à l'agent dont le fil est affiché : le paramètre
`agent` est donc **requis pour calculer `peer_exchanges`**. Sur
`GET /v1/snapshot` sans `agent`, la clé est omise (`non calculé`) ; avec
`agent`, elle est toujours présente, y compris sous la forme `[]` (`calculé,
aucun échange`). `/v1/watch` exige déjà `agent` et pousse la même projection.

```json
{
  "version": 1,
  "kind": "peer_exchange",
  "at": 1787670000,
  "peer": "jc6",
  "direction": "in" | "out" | "both",
  "count": 2,
  "delivery_ids": ["...", "..."]
}
```

| champ | rôle |
|---|---|
| `at` | **place chronologique** dans le fil — décisif : on doit voir qu'un agent a consulté quelqu'un **avant** de répondre |
| `direction` | `in` → « Message de X » · `out` → « Message à X » · `both` → « N messages avec X » |
| `count` | **toujours présent**, y compris à 1 |
| `delivery_ids` | clés de corrélation vers les corps déjà publiés par `/v1/journal` |

**Le relais ne formule pas la phrase.** Il donne direction et nombre ; la page
écrit « 2 messages avec jc6 ».

**Contenu du dépli** : la page indexe les `turn_start` et
`prompt_dispatched` du journal par `message_id`, puis résout les
`delivery_ids` dans leur ordre. Le dépli rend les **textes**, jamais les
identifiants. Pour un échange sortant ou bidirectionnel, la page lit aussi le
journal du pair : le corps est journalisé chez le destinataire, sans nouvelle
forme réseau ni enrichissement du relais. Les lectures utilisent `from_seq=0`
pour traverser les rotations de date déjà prises en charge par le journal.
Si une ancienne clé n'existe dans aucun des deux journaux, la page rend
« Contenu indisponible » ; elle ne remplace jamais le corps absent par la clé.

**Seuil de dépli** : `count <= 3` → sur place ; sinon panneau latéral.
**Paramétrable côté page**, valeur provisoire.

## C3 — Niveaux de détail *(lots L3, L4)*

Attaché à un message d'agent.

```json
{
  "version": 1,
  "duration_ms": 2820000,
  "acts": [
    { "at": ..., "kind": "command", "text": "cargo test -p maicie", "detail": "313 passés" },
    { "at": ..., "kind": "file",    "text": "ui.rs" },
    { "at": ..., "kind": "tool",    "text": "Read src/main.rs" },
    { "at": ..., "kind": "plan",    "text": "..." },
    { "at": ..., "kind": "approval", "text": "..." }
  ],
  "reasoning": { "available": true, "summary": "...", "raw": "..." }
}
```

**`kind` — ensemble fermé côté PAGE (projection)** : les kinds que les
pilotes ÉCRIVENT réellement dans `payload.kind`, plus les synonymes legacy.

| kind journal | producteur | note |
|---|---|---|
| `command` | Codex `CodexActKind::Command` | mesuré (relec*) |
| `file` | Codex `CodexActKind::File` | producteur réel, 0 émission mesurée |
| `plan` | Codex `CodexActKind::Plan` | producteur réel, 0 émission mesurée |
| `approval` | Codex `CodexActKind::Approval` + event `permission` | mesuré |
| `tool` | ACP `tool_call_journal_payload` (C3, depuis 78d57dc) | |
| `tool_call` | forme LEGACY Cursor encore dominante dans les journaux | projeté en `tool` |

**Retirés de la projection** (aucun `payload.kind` journal) :

- `intent` — le tableau ci-dessous le mappait depuis `agentMessage/delta`,
  mais les pilotes écrivent `kind:text` (bulle de réponse, pas un acte).
- `peer` — les échanges sont des entrées timeline `peer_exchange` (relais),
  jamais un `update.payload.kind`.

La page rend l'ancien `intent` en **blanc** s'il réapparaissait ; tout le
reste des actes en **gris**. Un filtre qui ne matche rien rend une projection
**vide** sans erreur — d'où le témoin `TEMOIN_vue_affiche_acte_present_au_journal`.

**Correspondance des sources** — voir `spec.md` §4.4 :

| `kind` | Codex | Cursor (ACP) |
|---|---|---|
| *(texte, pas acte)* | `item/agentMessage/delta` → `kind:text` | `agent_message_chunk` → `kind:text` |
| `command` | `item/commandExecution/outputDelta` | — |
| `file` | `item/fileChange/patchUpdated` | — |
| `tool` / `tool_call` | — | `tool_call` / `tool_call_update` |
| `plan` | `item/plan/delta` | — |
| `approval` | `item/*/requestApproval` | event `permission` |
| `reasoning` | `item/reasoning/*` | `agent_thought_chunk` |

**`reasoning.available: false`** → la page affiche **« raisonnement non
fourni »**, **jamais un vide**.

> Un vide se lirait « il n'a pas réfléchi ». Cas mesuré : chez Gemini le flux
> de pensée **n'est jamais émis** ; chez Claude un défaut amont le supprime.

## C4 — Ligne d'agent *(lot L5)*

Depuis `UiSnapshotV1.agents`, **enrichi**.

```json
{
  "name": "rc1",
  "type": "codex",
  "host": "cartae",
  "state": "alive" | "busy" | "stopped",
  "last_message_at": 1787670000,
  "last_excerpt": "...",
  "unread": 3
}
```

> **Conception de Grok Bot : ce sont des AGENTS, pas des conversations.**
> D'où `state` et `host`, absents d'une liste de fils. C'est ce qui dit d'un
> coup d'œil **qui travaille et qui est mort** — ce qui a manqué toute la
> journée du 25/08.

**`state`** doit distinguer **arrêté volontairement** de **mort**. Défaut connu
de l'outil de ronde : il confond les deux, et chaque agent éteint proprement
réapparaît comme une panne à réparer.

## C5 — État de la connexion *(lot L6)*

Poussé par `/v1/watch` :

```json
{ "version": 1, "kind": "relay_state", "state": "connected" | "reconnecting" | "lost", "since": ... }
```

**Le daemon a été redémarré deux fois le 25/08.** Sans cet événement,
l'utilisateur regarde une interface muette en croyant que personne ne parle.

## Interdits — quel que soit le lot

| interdit | raison |
|---|---|
| approuver un profil ou une routine | **frappe humaine obligatoire** (ADR 011) |
| exposer modèle, effort, permissions | scellés dans la définition figée de l'agent |
| changer la branche d'un agent | ferait **expirer un verdict** en cours |
| renvoyer du HTML ou des classes CSS | le relais transmet des faits |
| afficher « reçu » pour un envoi | injecté ≠ consommé |
