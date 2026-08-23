# Phase 1 — Contrats

**Feature** : 004-runtime-modele-agents

## 1. Message de protocole `Runtime`

Direction : wrapper → daemon. Ajouté à `WrapperToDaemon`
(`crates/bridget-transport/src/protocol.rs:13`).

```json
{"type":"Runtime","agent":"agent-2","model":"claude-opus-5","effort":"high","source":"claude-hook"}
```

| Champ | Type | Obligatoire | Sémantique |
|---|---|---|---|
| `agent` | `string` | **oui** | nom de l'agent observé. **Pas** la connexion émettrice : le hook et `bridget runtime` passent par le client CLI, dont la connexion est éphémère. Résolution par le routeur, comme `Rename` et `CancelRequest` |
| `model` | `string` | **oui** | identifiant du modèle tel que rapporté par l'agent, non normalisé. Une observation sans modèle n'est pas une observation : le producteur n'émet rien |
| `effort` | `string \| null` | non | niveau d'effort tel que rapporté. `null` signifie **observé absent**, pas « inconnu » : il efface une valeur antérieure |
| `source` | énumération fermée | oui | `codex-rollout`, `claude-hook` ou `declared` — journalisation seule. Toute autre valeur rend le message indécodable |

**Réponse** : `Ack { id: "runtime" }` en cas d'acceptation, `Nack` si la
connexion n'est pas enregistrée. Aucune diffusion vers les autres agents : ce
message ne transite pas par le routeur ni par le disjoncteur, il n'est pas un
message inter-agents.

**Règles de traitement daemon** :

1. L'agent nommé doit être connu du routeur et porter une présence ; sinon
   `Nack { reason: "agent introuvable: <nom>" }`.
2. **Remplacement atomique** : le couple `(model, effort)` reçu remplace le
   couple courant en bloc. Un `effort` absent efface l'effort connu, parce qu'il
   décrit une observation réelle et non une lacune (data-model, invariant 1).
3. Une valeur identique à l'état courant est ignorée sans erreur.
4. `model` et `effort` sont tronqués à 100 caractères et refusés s'ils
   contiennent un caractère de contrôle, par cohérence avec la validation
   existante (`cli.rs:36`).

## 2. `AgentInfo` — champs ajoutés

```json
{
  "name": "agent-2",
  "agent_type": "claude",
  "host": "macbook",
  "os": "macOS",
  "transport": "unix",
  "state": "connected",
  "model": "claude-opus-5",
  "effort": "high"
}
```

`#[serde(default)]` sur les deux champs : un client d'une version antérieure
reste décodable, un daemon plus récent reste lisible par un client ancien.

## 3. Sortie de `bridget who`

```text
Agents connectés :
  NOM      TYPE    HÔTE      OS     TRANSPORT  MODÈLE                     EFFORT  ÉTAT
  agent-2    claude  macbook   macOS  unix       claude-opus-5              high    connected
  agent-1  codex   macbook   macOS  unix       gpt-5.3-codex              xhigh   connected
  distant  claude  projet-a    Linux  ssh        —                          —       unreachable
```

Les largeurs de colonnes se calculent sur la valeur la plus longue, comme les
colonnes existantes. Les colonnes actuelles conservent leur nom, leur ordre
relatif et leur sémantique (SC-006) ; `MODÈLE` et `EFFORT` s'insèrent avant
`ÉTAT`.

## 4. `bridget runtime`

```text
bridget runtime --model <modèle> [--effort <niveau>]
```

| Cas | Sortie | Code |
|---|---|---|
| succès | `Runtime déclaré : <modèle> (effort: <niveau>)` | 0 |
| hors agent Bridget | `runtime indisponible hors d'un agent Bridget` | 1 |
| `--model` absent | `usage: bridget runtime --model <modèle> [--effort <niveau>]` | 2 |
| valeur invalide | `erreur: <raison>` | 2 |
| daemon injoignable | `daemon inaccessible: <raison>` | 1 |

## 5. `bridget hook claude-runtime`

Lit le payload de hook Claude Code sur l'entrée standard.

**Entrée attendue** (champs utilisés uniquement) :

```json
{"transcript_path":"~/.claude/projects/<projet>/<session>.jsonl","hook_event_name":"Stop"}
```

**Comportement** :

1. Si `BRIDGET_AGENT_NAME` et `BRIDGET_AGENT_NAME_FILE` sont absents → sortie 0
   silencieuse. Le hook est inerte hors Bridget (FR-012).
2. Si `transcript_path` est absent, illisible, ou ne contient aucune ligne
   assistant exploitable → sortie 0 silencieuse. Un hook ne doit jamais faire
   échouer le tour de l'agent (FR-013).
3. Sinon, émet `Runtime` avec `source: "claude-hook"`.

**Sortie standard** : vide dans tous les cas. Un hook `Stop` bavard pollue la
session de l'agent.
**Code de retour** : toujours 0. Les erreurs partent en `log::debug`.

## 6. `bridget install-hooks`

```text
bridget install-hooks            Installe le hook Stop dans ~/.claude/settings.json
bridget install-hooks --remove   Retire la seule entrée Bridget
```

**Garanties** :

- Sauvegarde `~/.claude/settings.json.bak-<AAAAMMJJ-HHMMSS>` écrite **avant**
  toute modification, et son chemin affiché à l'utilisateur (FR-012).
- Insertion **additive** dans le tableau `hooks.Stop` existant : les entrées
  utilisateur déjà présentes sont préservées à l'identique.
- Idempotence : une seconde exécution détecte l'entrée Bridget et ne fait rien.
- `--remove` ne retire que l'entrée dont la commande est
  `bridget hook claude-runtime`.
- Le fichier est réécrit avec une indentation à 2 espaces, comme l'existant.

## 7. Contrat de la sonde Codex

Interne au wrapper, sans surface utilisateur.

| Aspect | Contrat |
|---|---|
| Résolution du rollout | `lsof -p <pid>`, retenir le `.jsonl` de `mtime` le plus récent |
| Fréquence de résolution | une fois au démarrage, puis à l'expiration de 5 min ou si le chemin disparaît |
| Fréquence de vérification | `mtime` toutes les 20 s |
| Lecture | fenêtre de 256 Kio depuis la fin, agrandie jusqu'à 4 Mio si aucun `turn_context` complet n'y figure, puis abandon |
| Intégrité des lignes | la fenêtre commence presque toujours au milieu d'une ligne : la **première ligne est jetée** sauf si la fenêtre couvre le début du fichier. Le scan remonte les lignes complètes depuis la fin ; une dernière ligne sans saut de ligne final, ou tout JSON invalide, est ignorée sans erreur. L'agrandissement se fait avec recouvrement, jamais en repartant d'un offset arbitraire |
| Observation partielle | un `turn_context` (ou une ligne assistant) sans modèle exploitable ne produit **aucune** émission : on préfère l'inconnu explicite à une valeur inventée (FR-004) |
| Émission | uniquement si `(model, effort)` diffère de la dernière paire transmise |
| Échec | journalisé en `debug`, jamais remonté à l'utilisateur, jamais fatal |
