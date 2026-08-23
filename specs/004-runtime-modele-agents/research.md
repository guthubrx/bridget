# Phase 0 — Recherche : source de vérité du modèle et de l'effort

**Date** : 2026-08-17
**Méthode** : observation directe des artefacts runtime des deux agents ciblés
sur la machine de travail (macOS 25.2, `codex 0.147.0`, `claude 2.1.223`).
Aucune de ces conclusions ne provient d'une supposition sur la documentation.

---

## D-001 — Où vit le modèle courant d'un agent Claude Code

**Décision** : la dernière ligne `type=assistant` du transcript JSONL de la
session, filtrée sur `isSidechain != true`.

**Preuve empirique** :

```text
~/.claude/projects/-Users-user-Nextcloud-10-Scripts-bridget/0e52d7aa-….jsonl
  message.model = "claude-opus-5"   effort = "high"      isSidechain = false
~/.claude/projects/-private-tmp/4f552261-….jsonl   (lancé --model haiku)
  message.model = "claude-haiku-4-5-20251001"   effort = absent   isSidechain = false
```

**Rationale** : c'est la seule valeur qui décrit ce que le modèle a réellement
servi, par opposition à `~/.claude/settings.json` (`model`, `effortLevel`) qui
n'est qu'un défaut et reste faux dès que l'humain fait `/model` ou lance avec
`--model`.

**Conséquences** :

- `effort` est **absent** sur les modèles sans réglage d'effort (constaté sur
  Haiku 4.5). L'effort est donc optionnel, ce qui justifie l'inconnu explicite
  de FR-004 plutôt qu'une valeur par défaut inventée.
- Le filtre `isSidechain` est obligatoire : un sous-agent tourne fréquemment sur
  un autre modèle que la session principale, et sa ligne serait la plus récente.

**Alternatives écartées** :

- `~/.claude/settings.json` : périmé dès le premier `/model`.
- Demander à l'agent de se déclarer : l'agent ne connaît pas son niveau d'effort
  et n'est pas notifié de façon fiable d'un changement décidé par l'humain.

---

## D-002 — Comment relier un agent Claude à son transcript

**Décision** : un hook Claude Code `Stop`, qui reçoit `transcript_path` dans son
payload et hérite de l'environnement du processus agent.

**Preuve empirique** — payload réel capté sur un hook `Stop` :

```json
{
  "session_id": "4f552261-b47f-4e64-af06-0f618a07ad3f",
  "transcript_path": "~/.claude/projects/-private-tmp/4f552261-….jsonl",
  "cwd": "/private/tmp",
  "permission_mode": "dontAsk",
  "hook_event_name": "Stop",
  "last_assistant_message": "ok"
}
```

Le payload **ne contient pas** le modèle : le hook doit donc lire le transcript.
Second test, environnement vu par le hook :

```text
BRIDGET_AGENT_NAME=test-agent-2
BRIDGET_AGENT_NAME_FILE=~/.cache/bridget/agent-names/active-agent-2
```

**Rationale** : `BRIDGET_AGENT_NAME_FILE` est exactement ce que
`current_agent_name()` (`crates/bridget-daemon/src/cli.rs:216`) utilise déjà
pour `bridget rename`. Le hook n'introduit donc aucun nouveau mécanisme
d'identification d'agent : il réutilise celui de la spec `001-renommer-agent`.

`Stop` se déclenche à la fin de chaque tour de l'agent, donc juste après que la
ligne assistant portant le nouveau modèle a été écrite. C'est le seul événement
qui garantit une lecture fraîche, et il satisfait FR-006 sans aucune scrutation
périodique.

**Alternatives écartées** :

- **`lsof` sur le PID de Claude** : testé, ne rend rien. Claude ouvre et referme
  le transcript à chaque écriture, contrairement à Codex. Écarté sur preuve.
- **Forcer `--session-id` depuis le wrapper** : donnerait le chemin exact sans
  toucher à la configuration utilisateur, mais casse au premier `/clear` (la
  session change d'identifiant) et entre en conflit avec `--resume`.
- **Fichier le plus récemment modifié dans le dossier projet** : ambigu dès que
  deux agents Claude tournent dans le même dépôt, ce qui est le cas nominal ici.

**Risque accepté et documenté** : un `claude` lancé depuis le shell d'un agent
Bridget hérite du `BRIDGET_AGENT_NAME` de son parent et déclarerait donc le
modèle du parent. La contre-revue `agent-1` a demandé de rejeter cette
ambiguïté ; aucun discriminant fiable n'existe, un processus enfant héritant de
tout l'environnement de son parent et partageant ses ancêtres. Verrouiller le
`session_id` au premier hook casserait le cas nominal du `/clear`, qui change
d'identifiant. Le risque est donc conservé, avec une atténuation de diagnostic :
le `session_id` du payload est journalisé en `debug` à chaque mise à jour, ce qui
rend le cas identifiable s'il se produit. Voir
[adversarial-review-agent-1.md](./adversarial-review-agent-1.md), objection P0-3 bis.

---

## D-003 — Où vit le modèle courant d'un agent Codex

**Décision** : le dernier événement `type=turn_context` du fichier rollout de la
session, localisé par les descripteurs de fichiers ouverts du processus.

**Preuve empirique** :

```text
$ lsof -p 25987 | grep jsonl
codex 25987 user 37u REG … /Volumes/DATA/workflow/codex/sessions/2026/08/09/rollout-….jsonl
codex 25987 user 39u REG … /Volumes/DATA/workflow/codex/sessions/2026/08/17/rollout-….jsonl

turn_context.payload → { "model": "gpt-5.3-codex", "effort": "xhigh", "summary": "auto" }
```

**Rationale** : Codex maintient son rollout ouvert en permanence, ce qui donne
un lien **exact** processus → session, sans heuristique. Le point décisif est
que le chemin observé n'est pas `~/.codex/sessions/` mais un volume externe :
deviner le chemin aurait produit une détection systématiquement vide sur cette
machine. `lsof` n'est pas une commodité, c'est la seule méthode correcte.

**Conséquences** :

- Deux rollouts peuvent être ouverts simultanément (session reprise). Retenir
  celui dont le `mtime` est le plus récent.
- Un rollout observé pèse 945 Mo. La lecture doit partir de la **fin** du
  fichier par fenêtre bornée, jamais d'un parcours complet — Article XVIII.

**Coût mesuré** : `lsof -p <pid>` = 134 ms. Appelé en boucle courte, ce serait
un gaspillage ; d'où D-004.

**Alternatives écartées** :

- Chemin conventionnel `~/.codex/sessions/AAAA/MM/JJ/` : faux sur cette machine,
  prouvé ci-dessus.
- `session_meta` au lieu de `turn_context` : ne porte pas l'effort et n'est
  écrit qu'une fois, donc insensible à un changement en cours de session.

---

## D-004 — Cadence de la sonde Codex

**Décision** : résolution du chemin par `lsof` **une seule fois** au démarrage
(et re-résolution seulement si le fichier disparaît, ou au plus toutes les
5 minutes), puis surveillance par `mtime` toutes les 20 secondes. Lecture et
émission uniquement si `mtime` a changé **et** si la valeur lue diffère de la
dernière valeur transmise.

**Rationale** : ramène le coût récurrent d'un `lsof` à 134 ms toutes les
5 minutes plus un `stat` toutes les 20 s, soit un surcoût négligeable pour
l'agent observé (FR-013), tout en tenant les 60 s de FR-006. Satisfait aussi
FR-007 et SC-003 : un agent inactif n'émet rien du tout.

**Alternatives écartées** :

- `lsof` à chaque cycle : 0,7 % d'un cœur en permanence par agent, pour une
  donnée qui ne change qu'à chaque tour de travail.
- Surveillance d'événements du système de fichiers : dépendance supplémentaire
  (`notify`) pour un gain nul à cette granularité — Article XIX.

---

## D-005 — Forme du point d'entrée de mise à jour

**Décision** : un unique message de protocole `Runtime { model, effort }`
wrapper → daemon, alimenté par trois producteurs : la sonde Codex intégrée au
wrapper, la sous-commande `bridget hook claude-runtime` appelée par le hook, et
la déclaration explicite `bridget runtime --model … --effort …`.

**Rationale** : un seul chemin d'écriture dans l'annuaire, trois collecteurs.
Le parsing des deux formats JSONL vit dans le binaire Rust, pas dans un script
shell : il devient testable par `cargo test` sur fixtures (SC-005) et ne fait
apparaître aucune dépendance externe, `serde_json` étant déjà au workspace.

**Alternatives écartées** :

- Script shell + `jq`/`python3` pour le hook : ajoute une dépendance runtime non
  déclarée, sort la logique du périmètre testé, et duplique le parsing.
- Trois messages de protocole distincts selon la source : aucune règle métier ne
  diffère entre eux — wrapper passthrough interdit par l'Article XIX.

---

## D-006 — Placement du hook et réversibilité

**Décision** : `bridget install-hooks` insère l'entrée `Stop` dans
`~/.claude/settings.json`, après avoir écrit une sauvegarde horodatée
`settings.json.bak-<AAAAMMJJ-HHMMSS>`. `bridget install-hooks --remove` retire
la seule entrée Bridget. La commande est idempotente.

**Preuve de compatibilité** : `~/.claude/settings.json` porte déjà des hooks
utilisateur sur `UserPromptSubmit`, `Stop`, `PostToolUse` et `SessionEnd`.
L'insertion doit donc **ajouter** une entrée au tableau `Stop` existant sans
toucher aux autres, et non réécrire la clé.

**Rationale** : couvre FR-012. Le hook est inerte hors Bridget — il sort
immédiatement si `BRIDGET_AGENT_NAME` n'est pas défini, donc aucun impact sur
les sessions Claude ordinaires de l'utilisateur.

---

## Synthèse des inconnues levées

| Inconnue initiale | Levée par | Statut |
|---|---|---|
| Le payload de hook porte-t-il le modèle ? | test de capture réel | Non — lire le transcript (D-002) |
| Peut-on relier un PID Claude à son transcript ? | `lsof` sur PID Claude | Non (D-002), oui pour Codex (D-003) |
| Le hook hérite-t-il de l'identité Bridget ? | test de capture d'environnement | Oui (D-002) |
| L'effort est-il toujours présent ? | comparaison Opus / Haiku | Non, optionnel (D-001) |
| Le chemin des sessions Codex est-il conventionnel ? | `lsof` sur PID Codex | Non, volume externe (D-003) |
| Coût de la détection | mesure `time lsof` | 134 ms, à amortir (D-004) |

Aucune inconnue `NEEDS CLARIFICATION` ne subsiste à l'entrée de la Phase 1.
