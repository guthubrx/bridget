# Research : Transport ACP pour agents équipiers

**Date** : 2026-08-22 · Validation live effectuée le jour même (sources datées).

## R-001 — Le protocole ACP, côté client

**Décision** : Bridget agit comme *client* ACP ; l'agent (via son adaptateur)
est le *serveur*. Communication JSON-RPC 2.0 sur stdio du sous-processus.

Cycle de vie vérifié (source : agentclientprotocol.com/protocol/overview,
consulté 2026-08-22) :

| Étape | Méthode | Rôle pour Bridget |
|---|---|---|
| Init | `initialize` | négociation de version + capacités — point d'échec explicite si incompatible (FR de version) |
| Auth | `authenticate` (optionnelle) | non utilisée : les adaptateurs portent le login CLI |
| Session | `session/new` | une session par équipier, créée au lancement |
| Livraison | `session/prompt` | un message Bridget = un tour de prompt |
| Flux | notifications `session/update` | progression, appels d'outils → journal + état « tour en cours » |
| Permissions | `session/request_permission` | politique équipier (R-005) |
| Fin de tour | réponse à `session/prompt` avec `stopReason` | capture de la réponse → routage + clôture de la demande suivie |
| Annulation | `session/cancel` | arrêt propre d'un tour (stop volontaire) |

Le tour est la brique exacte dont Bridget a besoin : début connu, flux typé,
fin explicite avec raison. La sémantique « relance différée si tour en cours »
(FR-008) se lit directement de cet état.

## R-002 — Adaptateurs par agent, versions constatées

Vérifié le 2026-08-22 (npm + binaire local) :

| Agent | Voie ACP | Version constatée | Lancement | Statut |
|---|---|---|---|---|
| Codex (priorité 1) | `@zed-industries/codex-acp` (npm, bin `codex-acp`) | 0.16.0 | `npx @zed-industries/codex-acp@0.16.0 -c model="gpt-5.5"` (pin issu du spike T701) | **validé** (T701/spike, quickstart) |
| Claude (priorité 2) | `@zed-industries/claude-code-acp` (npm) | 0.16.2 | `npx @zed-industries/claude-code-acp@0.16.2` | **validé** (T707, sans surcharge) |
| Gemini (priorité 3) | natif dans Gemini CLI | 0.46.0 (brew) et 0.56.0 (npm) testées | `gemini --acp` / `npx @google/gemini-cli@0.56.0 --acp` | **conditionnel/non validé** — voie individuelle fermée par Google (« migrate to the Antigravity suite », constat T708 du 2026-08-22) ; dépendrait d'un compte éligible |

Décision : versions **pinnées** dans le registre d'agents (config), jamais de
`@latest`. La version attendue fait partie de l'entrée de registre et est
vérifiée au `initialize` (version de protocole) + au lancement (version de
paquet quand `npx` le permet).

Note : les adaptateurs Zed sont explicitement publiés pour usage hors Zed
(source : zed.dev/blog/codex-is-live-in-zed, consulté 2026-08-22).

**Constat spike T701 (2026-08-22, trois tentatives, transcriptions extraites
dans `implementation.md`)** : le cœur Codex embarqué dans `codex-acp` 0.16.0
refuse les modèles récents de la config utilisateur (`gpt-5.6-terra` **et**
`gpt-5.6-sol` → erreur 400 « requires a newer version of Codex »). La
résolution retenue (tentative 3, validée en pair review) : conserver le vrai
`~/.codex` et **épingler un modèle compatible dans les args de l'adaptateur**
(`-c model="gpt-5.5"`) — auth abonnement conservée, ni `OPENAI_API_KEY` ni
`CODEX_API_KEY` présents ou demandés, `stopReason=end_turn` reçu. La piste
« home isolé » (tentative 2) fonctionnait aussi mais a été écartée : inutile
dès lors que la surcharge de modèle suffit, et porteuse de risques propres
(sécurité de la copie d'`auth.json`, rotation de jetons). Le retour aux modèles
récents pour les équipiers = bump du pin `codex-acp` quand Zed publie une
version au cœur plus récent — une ligne de registre.

## R-003 — Facturation : abonnements, jamais de clé API

Constats (sources : zed.dev/acp/agent/codex-cli, README claude-code-acp,
consultés 2026-08-22) :

- `codex-acp` : « Codex owns its own authentication and billing » — l'adaptateur
  utilise le login du CLI Codex (compte ChatGPT).
- `claude-code-acp` : s'appuie sur le Claude Agent SDK avec la session loggée de
  Claude Code (abonnement).
- Gemini CLI : login Google du CLI.

**Risque identifié** : une variable d'environnement de clé API ferait basculer
silencieusement certains SDK en facturation au token. Garde (FR-011) : refus de
lancement si l'une de ces variables est présente pour le fournisseur concerné :

| Fournisseur | Variables refusées |
|---|---|
| Codex/OpenAI | `OPENAI_API_KEY`, `CODEX_API_KEY` |
| Claude/Anthropic | `ANTHROPIC_API_KEY` |
| Gemini/Google | `GEMINI_API_KEY`, `GOOGLE_API_KEY` |

Contournement volontaire : variable `BRIDGET_ALLOW_API_KEY=1`, documentée dans
le message de refus. Les variables refusées par type d'agent vivent dans
l'entrée de registre (pas en dur).

## R-004 — Client JSON-RPC : crate officielle ou sous-ensemble maison

Crate officielle : `agent-client-protocol` v2.0.0 (Apache-2.0, Zed). Dépendances
vérifiées le 2026-08-22 : `async-io`, `async-process`, `blocking`, `futures`,
`futures-concurrency`, `schemars`, `tracing`, `uuid`, `rustc-hash`… — un
écosystème async complet.

Bridget est intégralement **synchrone** (threads std, `BufReader` ligne à
ligne, `serde_json` — cf. `wrapper.rs`, `protocol.rs`). Le README revendique
« trois crates Rust sans dépendance exotique ».

**Décision (D-201 au plan)** : implémenter un client JSON-RPC **minimal et
bloquant** dans `bridget-transport`, limité au sous-ensemble du tableau R-001.
Justification Article XIX : la crate officielle imposerait un runtime async et
~10 dépendances transitives ; le sous-ensemble tient dans le style existant du
code (mêmes primitives que le protocole wrapper↔daemon). Risque assumé :
suivre l'évolution du protocole à la main — mitigé par la négociation de
version à l'`initialize` et le pinnage des adaptateurs. Ce compromis est
documenté et réversible (le schéma officiel de la crate sert de référence de
conformité).

**Borne de conformité obligatoire** (ajoutée après contre-revue, objection 2 ;
précisée en review T707 : **matrice à deux couches**). Couche 1 — *invariants
du client JSON-RPC*, prouvés une fois par les fixtures génériques et les tests
du dispatcher (ils n'appartiennent à aucun adaptateur) : hors-ordre, ids
mixtes, `error`, EOF, JSON invalide, méthode inconnue, requête serveur→client,
écritures concurrentes. Couche 2 — *formes filaires réellement observées par
adaptateur* (fixture par adaptateur, capturée au réel) : tour nominal,
notifications propres au harness (ex. `available_commands_update` de Claude),
chunks vides/concaténés, `sessionId`, `stopReason`. On ne fabrique pas de
captures artificielles pour la couche 1 :

| Cas JSON-RPC | Comportement exigé |
|---|---|
| notifications intercalées pendant une requête en attente | dispatchées sans perturber le waiter |
| requête serveur→client pendant un `session/prompt` en cours | traitée et répondue par le thread lecteur |
| réponses hors ordre (ids non séquentiels) | corrélées par la table `request_id → waiter` |
| id numérique **et** id chaîne | acceptés indifféremment |
| objet `error` en réponse | propagé comme échec motivé, jamais avalé |
| EOF sur stdout | fin de session propre, échec des tours en attente |
| ligne non-JSON ou JSON invalide | journalisée et ignorée, sans panique ni corruption d'état |
| message inattendu (méthode inconnue) | journalisé, ignoré ; répondre `method not found` si c'est une requête |
| écritures concurrentes | sérialisées par le writer — aucun entrelacement possible |

## R-005 — Permissions en mode équipier headless

Aujourd'hui, le wrapper lance déjà les agents avec contournement complet des
permissions (`--yolo` Codex, `--dangerously-skip-permissions` Claude — cf.
`wrapper.rs:483-505`). Le mode équipier ACP reprend cette **parité** : réponse
automatique « autoriser » aux `session/request_permission`, consignée au
journal de session. La politique est un champ de l'entrée de registre
(`permissions = "allow" | "deny"`), défaut `allow` pour parité avec l'existant.
Durcir cette politique est explicitement hors périmètre (modèle de confiance du
README inchangé).

## R-006 — Livraison des métadonnées de message

Le message Bridget (expéditeur, `reply`, id, corps) doit arriver dans le tour
sans protocole d'emoji. Décision : en-tête textuel sobre et stable en tête du
prompt (`De : <expéditeur> — réponse attendue : oui/non`), suivi du corps brut.
Pas de JSON dans le prompt : l'agent n'a rien à parser, et le corps reste
intact (SC-002). Le mapping exact est spécifié dans `contracts/livraison-acp.md`.

## R-007 — Chemin de retour de la réponse vers le daemon

Le wrapper capture la réponse à la fin du tour et doit la remettre au daemon.
Le protocole wrapper→daemon existant (`WrapperToDaemon` dans `protocol.rs`)
est le canal naturel. La phase tasks précisera s'il existe une variante d'envoi
réutilisable (voie du CLI `bridget send`) ou s'il faut une variante nouvelle —
à trancher au reuse-audit avec preuve `fichier:ligne`.
