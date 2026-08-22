# Research : Outils MCP Bridget (session 010)

**Date** : 2026-08-22 · Rédigé par anticipation pendant l'implémentation 007 ;
à confronter au verdict de contre-revue de la spec.

## R-201 — Le sous-ensemble MCP nécessaire

MCP est du JSON-RPC 2.0 sur stdio (côté serveur ici — symétrique du client ACP
de la 007, mêmes primitives maison, zéro dépendance). Sous-ensemble requis :

| Méthode | Rôle |
|---|---|
| `initialize` | négociation de version/capacités ; déclarer `tools` seulement |
| `notifications/initialized` | accusé du client |
| `tools/list` | déclarer les 3 outils, schémas JSON en français ; **peut être rappelée à tout moment** par le harness — réponse stable et idempotente |
| `tools/call` | exécuter ; résultat `content` structuré. `isError` est réservé aux échecs **techniques** (daemon injoignable, timeout) — un refus Bridget (`Nack` DND, disjoncteur…) est un **résultat métier** structuré, pas une erreur (aligné FR-011) |
| `ping` (optionnelle) | répondre si le harness l'utilise |

Conformité : fixtures contre le schéma officiel (même doctrine que la matrice
R-004 de la 007) ; capacités non déclarées (resources, prompts) absentes de
`initialize` — un harness ne doit rien attendre d'autre.

## R-202 — Branchement par type d'agent (formes constatées)

| Type | Forme de branchement |
|---|---|
| Claude Code (interactif) | config MCP au lancement (`--mcp-config` / `.mcp.json` projet) — clé `mcpServers.bridget = { command, args }` |
| Codex CLI (interactif) | `~/.codex/config.toml`, table `mcp_servers.bridget` — le wrapper peut passer l'équivalent en `-c` au lancement (motif validé au spike T701 pour `model`) sans toucher au fichier utilisateur |
| Gemini CLI (interactif) | `settings.json`, clé `mcpServers` |
| Équipier ACP (tous) | paramètre `mcpServers` de `session/new` — **constaté dans les transcriptions du spike T701** ; c'est la voie la plus propre : par session, sans fichier |

Décision proposée : la forme de branchement est un champ de l'entrée de
**registre** (007) — `mcp = "acp-session" | "cli-config" | "none"` avec les
détails par type — pas de code par agent hors registre. À vérifier par un spike
de branchement par type avant implémentation (l'injection `-c` Codex et le
`--mcp-config` Claude notamment).

## R-203 — Identité : env var, puis filiation pid, à travers npx

Chaîne réelle d'un équipier Codex : wrapper → `npx` (pid enregistré dans
`agent-pids/` par le wrapper) → `node`/`codex-acp` → (harness) → `bridget mcp`.
La variable `BRIDGET_AGENT_NAME` est posée par le wrapper mais **peut être
filtrée** par le harness au lancement des serveurs MCP (constat documenté dans
`wrapper.rs:561-565`, raison d'être d'`agent-pids/`).

Résolution par filiation : le serveur MCP remonte ses ancêtres (`ppid` répété
via `libc`/sysctl sur macOS, `/proc/<pid>/stat` sur Linux) jusqu'à trouver un
pid présent dans `agent-pids/`. Les processus intermédiaires (`npx`, `node`,
shells) sont traversés naturellement — c'est le pid **enregistré** qui compte,
pas la profondeur. Cas limites à couvrir :

- ancêtre jamais trouvé (serveur lancé hors agent) → erreur explicite (spec) ;
- pid réutilisé par l'OS après mort de l'agent → l'entrée `agent-pids/` doit
  être nettoyée au désenregistrement (vérifier ce nettoyage en reuse-audit ;
  sinon, tâche) ;
- renommage : l'entrée `agent-pids/<pid>` (format enrichi D-402 : naissance,
  `instance_id`, **chemin du fichier de nom courant**) pointe vers le fichier
  que `bridget rename` réécrit — le serveur relit ce fichier à chaque appel
  d'outil, pas au démarrage (parité avec la résolution de nom du wrapper).

## R-204 — Reconnexion daemon

Le binaire ouvre une connexion par commande ; le serveur MCP fera pareil par
appel d'outil (`FR-008 sans état`) : pas de connexion longue à réparer, le cas
« daemon redémarré » se réduit à « connexion refusée → erreur explicite de
l'appel ». Simplicité > pool de connexions (Article XIX) ; à re-mesurer
seulement si la latence par appel devenait perceptible (elle est en
microsecondes sur socket Unix local).

## R-205 — Prompt allégé : quoi reste, quoi part

| Bloc actuel | Sort avec MCP |
|---|---|
| « réponds TOUJOURS avec bridget send » | **supprimé** (équipiers ACP : déjà supprimé en 007 ; interactifs : l'outil remplace la commande) |
| syntaxe de commande, interdiction d'accuser réception | **supprimés** (portés par la description des outils) |
| sémantique `💬` des messages entrants (agents tmux) | **conservé** (la livraison reste du ressort du transport) |
| `reply=yes/no` | **conservé**, reformulé court |

La mesure SC-005 (≥ 60 % de réduction en caractères) se fait sur le bloc
interactif tmux, le seul qui garde un prompt.
