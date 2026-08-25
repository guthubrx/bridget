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
| `delivery_ids` | permet le dépli sans nouvelle requête |

**Le relais ne formule pas la phrase.** Il donne direction et nombre ; la page
écrit « 2 messages avec jc6 ».

**Seuil de dépli** : `count <= 3` → sur place ; sinon panneau latéral.
**Paramétrable côté page**, valeur provisoire.

## C3 — Niveaux de détail *(lots L3, L4)*

Attaché à un message d'agent.

```json
{
  "version": 1,
  "duration_ms": 2820000,
  "acts": [
    { "at": ..., "kind": "intent",  "text": "Je vais comparer..." },
    { "at": ..., "kind": "command", "text": "cargo test -p maicie", "detail": "313 passés" },
    { "at": ..., "kind": "file",    "text": "ui.rs" },
    { "at": ..., "kind": "tool",    "text": "..." },
    { "at": ..., "kind": "plan",    "text": "..." },
    { "at": ..., "kind": "peer",    "peer": "rc1", "direction": "both", "count": 2 }
  ],
  "reasoning": { "available": true, "summary": "...", "raw": "..." }
}
```

**`kind` — ensemble fermé** : `intent` · `command` · `file` · `tool` · `plan` ·
`peer` · `approval`.
La page rend `intent` en **blanc**, tout le reste en **gris**.

**`reasoning.available: false`** → la page affiche **« raisonnement non
fourni »**, **jamais un vide**.

> Un vide se lirait « il n'a pas réfléchi ». Cas mesuré : chez Gemini le flux
> de pensée **n'est jamais émis** ; chez Claude un défaut amont le supprime.

**Correspondance des sources** — voir `spec.md` §4.4 :

| `kind` | Codex | Cursor (ACP) |
|---|---|---|
| `intent` | `item/agentMessage/delta` | `agent_message_chunk` |
| `command` | `item/commandExecution/outputDelta` | `tool_call` |
| `file` | `item/fileChange/patchUpdated` | `tool_call_update` |
| `plan` | `item/plan/delta` | — |
| `approval` | `item/*/requestApproval` | — |
| `reasoning` | `item/reasoning/*` | `agent_thought_chunk` |

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
