# Bridget

**🇫🇷 Français** · [🇬🇧 English](README.en.md)

> Faire travailler ensemble des agents IA en ligne de commande — sur une machine,
> ou sur plusieurs.

Vous avez ouvert Codex dans un terminal, Claude Code dans un autre, peut-être un
troisième agent sur un serveur distant. Chacun travaille seul, et c'est vous qui
faites le facteur : copier une question, coller une réponse, se souvenir de qui
attend quoi.

Bridget donne à ces agents un moyen de se parler directement. Un daemon local
route les messages, tient un annuaire de qui est là, et se charge de ce qui rend
la coordination pénible : relancer celui qui ne répond pas, rattraper une
connexion coupée, empêcher deux agents de boucler indéfiniment.

```console
$ /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget who
Agents connectés :
  NOM      TYPE    HÔTE         OS     TRANSPORT  DOMAINE    MODÈLE         EFFORT  ÉTAT
  agent-1  claude  poste-local  macOS  unix       bridget    claude-opus-5  high    connected
  agent-2  codex   poste-local  macOS  unix       projet-b   gpt-5.6-terra  xhigh   dnd
  distant  claude  serveur      Linux  ssh        projet-b   claude-opus-5  high    connected

$ /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget send --to distant --reply "Peux-tu relire crates/bridget-core ?"
OK: envoyé à « distant » (id=fa09fa7800694, hops=4) [réponse attendue]
```

Trois agents, deux machines, un seul annuaire — et une question dont Bridget se
souvient jusqu'à ce qu'elle obtienne une réponse.

## Ce que Bridget apporte

**Plusieurs machines, un seul annuaire.** Un agent sur votre portable et un agent
sur un serveur apparaissent côte à côte et se parlent comme s'ils étaient
voisins. Le socket local est publié par un tunnel SSH inverse : aucun port à
ouvrir, aucun second daemon à administrer, aucun certificat à gérer.

**Une question sans réponse ne s'oublie pas.** Un message envoyé avec `--reply`
devient une demande suivie, avec une échéance. À un tiers du délai, Bridget
relance discrètement le destinataire ; aux deux tiers, il insiste ; à l'échéance,
il prévient l'émetteur que la demande a échoué. Personne n'attend indéfiniment
une réponse qui ne viendra pas, et rien ne se perd en silence.

**Les coupures ne cassent rien.** Quand le réseau tombe, l'agent distant continue
de travailler. Son wrapper se reconnecte seul, avec un délai croissant, et
retrouve son nom — même s'il avait été renommé entre-temps. Pendant ce temps
l'annuaire le montre `unreachable` plutôt que disparu, ce qui distingue une perte
de réseau d'un agent arrêté volontairement.

**Vous savez à qui vous confiez quoi.** L'annuaire n'affiche pas seulement
`claude` ou `codex`, mais le modèle réellement en service et son niveau d'effort,
tenus à jour quand vous en changez en cours de session. Un domaine, déduit du
dépôt de travail, dit sur quel projet chacun est occupé.

**Le silence se demande.** Un agent en pleine tâche longue peut refuser les
interruptions. Les messages qui lui sont adressés sont alors refusés avec la
raison et le temps restant, et l'émetteur décide de la suite plutôt que
d'attendre sans savoir.

**Des garde-fous contre l'emballement.** Deux agents enthousiastes peuvent
s'envoyer des messages jusqu'à épuisement du budget. Un disjoncteur, une
déduplication par contenu et un budget de sauts arrêtent la boucle avant vous.

Le protocole est indépendant de son transport — tmux aujourd'hui, socket réseau
demain — et tient dans trois crates Rust sans dépendance exotique.

## Démarrage rapide

```bash
# Compiler
cargo build --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml --release

# Lancer le daemon
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget daemon

# Lancer un agent dans tmux
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget codex

# Dans un autre terminal, envoyer un message
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget send --to codex-1 "Analyse ce fichier" --reply

# Voir les agents connectés
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget who
```

## Coordonner avec Maicie

Maicie est le compagnon CLI optionnel de Bridget pour les objectifs et
délégations durables. Elle reste déterministe : aucun LLM, scheduler ou
interprétation de texte libre ne choisit à la place de l'utilisateur.

```bash
cargo build --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml --release -p maicie
```

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie delegate \
  --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json \
  --goal "Relire le plan de reprise" \
  --to reviewer \
  --duration normale \
  --idempotency-key revue-plan-1 \
  --json

/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/maicie status --config /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/.local/maicie/maicie.json --json
```

Chaque commande ouvre la SQLite privée de Maicie puis rend la main. Les voies
qui contactent Bridget réconcilient d'abord les remises non terminales ; les
actions d'objectif restent locales. Il n'existe donc aucun daemon Maicie à
arrêter. Une interruption laisse les transactions déjà commitées récupérables
au prochain appel. Maicie ne lance et n'arrête jamais un agent : les
activations approuvées passent exclusivement par le `SpawnOrder` public de
Bridget, et `approve` reste une frappe humaine locale non exposée à Bridget ou
MCP.

Les deux couches gardent des vérités différentes. Maicie possède objectifs,
décisions et outboxes transactionnelles ; Bridget possède présence, livraison
et snapshot du transport. Un `Gap`, un `End` ou une observation périmée ne
devient jamais une conclusion métier. Après une coupure, Maicie consulte puis
rejoue le même identifiant et les mêmes octets. Pour une activation, le replay
exact du `SpawnOrder` est lui-même le lookup idempotent : il n'existe pas de
surface `SpawnLookup`. `IdempotencyExpired` est terminal et interdit un nouvel
envoi implicite. Le digest reçu dans `SpawnAccepted` est comparé au hash
épinglé lors de l'approbation, sans relire le registre.

Le démarrage, la configuration et les limites détaillées sont documentés dans
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/plugins/maicie/README.md`.

### Ronde portable (lecture seule)

La ronde constate mécaniquement l'annuaire Bridget, les demandes ouvertes et
échues, les objectifs Maicie à évaluer et le registre. Elle n'affecte personne,
n'envoie aucun message et n'écrit aucune donnée d'autorité : le référent reste
seul décideur. Elle produit une ligne courte suivie d'un document JSON ; si
Maicie est indisponible, les observations Bridget restent présentes et la
partie Maicie est explicitement `unavailable`.

```bash
python3 scripts/bridget-ronde.py --config "$HOME/.config/maicie/config.json"
```

Pour archiver les deux formats après chaque ronde et l'exécuter périodiquement
dans le compte utilisateur, la pose est volontairement explicite. Elle crée une
unité `launchd` sur macOS ou un service/timer `systemd --user` sur Linux ; aucun
service n'est nécessaire pour lancer la commande à la main.

```bash
scripts/install-bridget-ronde.sh \
  --config "$HOME/.config/maicie/config.json" \
  --report-dir "$HOME/.cache/bridget/rondes"
```

Les archives portent leur horodatage UTC et sont écrites en `.txt` et `.json`
avec des permissions `0600`. Pour seulement poser l'unité sans l'activer,
ajoutez `--skip-activate`.

### Guichet Maicie

La session 015 ajoute un contrat de guichet pour rendre la cible de service
`maicie` joignable pendant l'absence du compagnon CLI. Bridget tient la boîte
aux lettres de transport ; Maicie relève ensuite en pull-only au début d'une
commande locale, sous budget absolu, puis rend la main. La relève se fait par
`GuichetClaimNext`, au plus une demande à la fois, dans l'ordre FIFO durable.

La capacité `maicie_guichet` est la borne d'autorisation de la relève, des
réponses et des événements du guichet. Elle ne doit jamais être présentée comme
un droit obtenu par le nom déclaré `from: "maicie"`. Limite C5 conservée :
dans le modèle local coopératif v1, cette capacité n'authentifie pas
cryptographiquement un processus hostile du même compte.

Le guichet v1 refuse fermement le texte libre, les opérations inconnues, les
champs inconnus, les enveloppes divergentes, les claims sans capacité et toute
approbation distante. Il n'expose aucune route pour créer une approbation,
consommer une activation de profil ou lancer un agent. Une boucle résidente
`maicie serve` est explicitement hors version livrée et devra passer par une
spécification v2.

Les trois opérations structurées livrées sont `delivery-report`,
`mission-status` et `deadline-question`. Un dépôt est idempotent par la clé
`(issuer_scope, service_request, request_id)` et ses octets canoniques : un
rejeu conserve donc exactement le même triplet, `issued_at` et le même corps.
La relève `GuichetClaimNext` est FIFO durable et loue au plus une demande ; la
réponse exige le propriétaire, le token, la génération et le bail courants.
Bridget garde la demande liée et son événement de cycle ; Maicie garde le reçu
de greffe et la décision de coordination.

Le parcours réel G1504 a vérifié l'ensemble : un wrapper ACP dépose pendant
l'absence de Maicie, une commande Maicie relève, greffe et répond, Bridget
marque la demande `answered` et l'événement est relevé une seule fois. Le
rejeu ne crée ni seconde décision ni second événement ; mesure : **925 ms**
(commit `69ad00d`). Le guide exécutable est
`/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/specs/015-guichet-maicie/quickstart.md`.

## Envois idempotents

Pour une délégation que votre programme pourra rejouer après une coupure,
fournissez les trois valeurs durables ensemble : une portée opaque stable, une
clé métier et son instant Unix d'émission. Bridget ne génère jamais ces valeurs
à la place du client.

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget send --to codex-1 \
  --id delegation-42 \
  --issued-at "$(date +%s)" \
  --issuer-scope "012_scope_aaaaaaaaaaaa" \
  "Analyse ce fichier"
```

Un retry réutilise exactement ces trois valeurs et la même enveloppe. Il
rejoue alors l'issue durable (`Accepted`, refus motivé ou `OutcomeUnknown`) ;
une enveloppe différente avec la même clé est refusée par
`EnvelopeMismatch`. Après l'échéance annoncée par le daemon,
`IdempotencyExpired` interdit toute réémission aveugle. Les trois options sont
obligatoires ensemble ; un envoi historique sans elles reste inchangé. Une
réponse portant `--in-reply-to` négocie toutefois ce contrat automatiquement :
le binaire affiche son `id` et son `issued_at`, à réutiliser ensemble pour lire
le sort d'un envoi non encore consolidé. Sa portée stable vient de l'instance
Bridget courante, comme pour l'outil MCP.

Le premier résultat d'un envoi nominal n'est pas un incident. L'issue durable
`OutcomeUnknown` ci-dessus est celle du protocole ; côté client — binaire comme
outil MCP — elle se lit sous deux statuts distincts, selon qu'un dépôt est
attesté ou non. Un seul des deux décrit un problème :

- **`in_flight` — dépôt attesté**, le destinataire n'a pas encore accusé.
  **Aucune action n'est requise** : c'est un succès. Le daemon répond sans
  jamais attendre l'aval, donc c'est le retour normal d'un premier envoi. Le
  binaire l'annonce `DÉPÔT: in_flight (remise en vol)` et l'outil MCP rend le
  même statut, `"in_flight"`, avec l'identifiant de remise qui l'atteste.
- **`outcome_unknown` — le sort est inconnu**, pour l'une de deux raisons :
  la connexion est tombée avant la réponse, ou le daemon a répondu sans
  attester de dépôt (pas d'identifiant de remise). Dans les deux cas, le
  message a pu partir ou non.

Les deux surfaces nomment le même statut, il n'y a donc rien à traduire de
l'une à l'autre.

Pour lire le sort d'un envoi non encore consolidé — `in_flight` par curiosité,
`outcome_unknown` par nécessité —, rejouez à l'identique : **même `id`, même
`issued_at`, même corps**. C'est une consultation, jamais une seconde émission ;
elle rend `accepted` une fois l'accusé du destinataire consolidé, et ne duplique
jamais.

Cette garantie est disponible sur le protocole local et la CLI.

## Outils MCP

Lancez `bridget mcp` depuis un équipier Bridget pour exposer `bridget_send`,
`bridget_who` et `bridget_ledger` sur stdin/stdout JSON-RPC. Les résultats
structurés sont également fournis en texte MCP ; le retry réutilise `id` et
`issued_at` retournés par le premier envoi.

## Équipiers gérés

Un équipier géré reçoit les livraisons directement par son pilote, sans
injection de texte dans son terminal. Lancez-le avec `--equipier` :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget codex --equipier
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget claude --equipier
```

Les types, commandes et politiques sont déclarés dans
`~/.config/bridget/agents.json`. L’entrée Codex lance nativement
`/opt/homebrew/bin/codex app-server` avec le modèle épinglé
`gpt-5.6-terra`; l’entrée Claude lance
`/Users/moi/.local/bin/claude --model claude-opus-5`. Ces chemins absolus
restent disponibles au daemon lancé par launchd. Bridget refuse par défaut les
clés API déclarées dans `forbidden_env`, pour préserver l’authentification par
abonnement ; `BRIDGET_ALLOW_API_KEY=1` est un contournement explicite à
employer seulement si la facturation API est voulue.

Chaque type lancé déclare aussi `capabilities` : `execution_paths`, puis les
modèles et efforts exacts acceptés. Cette matrice fait partie de la définition
figée et de son digest : un modèle, un effort ou un chemin absent est refusé
avant tout processus, sans interrogation du pilote au démarrage.

Gemini reste déclaratif, mais les comptes individuels ne sont pas supportés au
2026-08-22 : Google demande la migration vers Antigravity. Les agents tmux
existants, lancés sans `--equipier`, conservent leur comportement `💬`.

## Équipiers persistants

Le daemon peut devenir propriétaire du cycle de vie d’un équipier géré. Le
terminal qui donne l’ordre peut alors se fermer sans arrêter l’équipier :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget spawn codex --name analyse --persistent
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget who
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget stop analyse
```

`--persistent` demande au daemon de relancer l’équipier après son propre
redémarrage, sous le même nom et avec la même configuration. Un équipier lancé
sans cette option reste actif après la fermeture du terminal donneur d’ordre,
mais n’est pas recréé au prochain démarrage du daemon. `bridget stop` arrête le
groupe complet, descendants `npx` compris, et retire un équipier persistant de
l’état désiré afin qu’il ne revienne pas.

Chaque `spawn` affiche un `command_id`. Si la réponse réseau est perdue,
rejouez exactement l’ordre avec `--command-id <ID>` : Bridget restitue la même
issue sans créer une seconde génération.

## Positionnement et modèle de confiance

Bridget cherche deux choses : un **canal de communication fiable** entre agents
CLI, et un **protocole léger**. Ce n'est pas un produit de sécurité, et il ne
prétend pas l'être.

**Hypothèses de fonctionnement.** Les agents tournent sur une même machine de
confiance, sous un même compte utilisateur, et communiquent par un socket Unix
local. Tous les agents connectés sont considérés comme coopératifs.

**Ce que Bridget garantit :**

- un message est livré, ou refusé avec une raison exploitable — jamais perdu en silence ;
- pas de boucle entre agents, pas de doublon, pas de retransmission ;
- une coupure de connexion est rattrapée automatiquement, sous la même identité ;
- une demande en attente a une échéance, des rappels, et peut être annulée.

**Ce que Bridget ne fait pas :**

- il n'authentifie pas l'émetteur d'un message de contrôle : tout processus local
  pouvant écrire sur le socket peut renommer un agent, changer son domaine ou sa
  disponibilité ;
- il ne chiffre rien et ne signe rien ;
- il n'isole pas des agents qui se méfieraient l'un de l'autre ;
- il ne résiste pas à un processus local hostile.

En conséquence, n'exposez pas le socket à un réseau ou à un compte auquel vous ne
faites pas confiance. La [fédération SSH](#plusieurs-machines-et-fédération-ssh) tunnelise le socket : la
confiance repose entièrement sur SSH, pas sur Bridget.

Ces limites sont des choix de périmètre, pas des oublis. Les lever supposerait un
modèle d'autorisation à part entière — utile le jour où des agents de confiances
différentes devront cohabiter, inutile aujourd'hui, et coûteux en complexité pour
un protocole qui veut rester lisible d'un bout à l'autre.

## Architecture

```
Agent A (CLI)          Agent B (CLI)
    │                      │
    ▼                      ▼
Wrapper A              Wrapper B
    │                      │
    └──────► daemon ◄──────┘
              │
         SQLite (ledger)
```

- **Daemon** (`bridget daemon`) — socket Unix locale, route les messages,
  maintient le ledger et les sécurités.
- **Wrappers** (`bridget codex`, `bridget claude`) — lancent l'agent CLI (fork+exec),
  se connectent au daemon, reçoivent les messages en push.
- **CLI client** (`bridget send`) — envoie un message depuis n'importe quel shell.

## Crates

| Crate | Rôle |
|-------|------|
| `bridget-core` | Logique pure : routage, disjoncteur, déduplication, enveloppes |
| `bridget-transport` | Protocole JSON + trait Transport + implémentation tmux |
| `bridget-daemon` | Daemon + CLI (binaire `bridget`) |

## Garde-fous de fiabilité

Ces mécanismes protègent la conversation d'elle-même — boucles, doublons,
attentes sans fin. Ils ne protègent pas d'un tiers hostile : voir le modèle de
confiance ci-dessus.

- **Disjoncteur** — max 8 échanges par conversation en 180s (configurable)
- **Déduplication par contenu** — bloque les doublons d'envoi
- **Quarantaine par ID** — bloque les retransmissions (misroute)
- **Hops** — budget de sauts anti-boucle (défaut : 4)
- **Reply yes/no** — distingue questions et affirmations
- **Auto-envoi interdit** — un agent ne peut pas se parler à lui-même
- **Escalade progressive** — rappels automatiques à T/3, 2T/3 puis notification d'échec à T
- **Timeout configurable** — `--timeout <secondes>` (défaut 60s)
- **Demandes annulables** — une demande `--reply` porte un identifiant et peut être arrêtée par son émetteur

Les deux derniers points sont détaillés dans
[Demandes suivies et relances](#demandes-suivies-et-relances).

## Commandes

### Lancer un agent

| Commande | Effet |
|---|---|
| `bridget codex [ARGS…]` | lance Codex et le connecte au daemon |
| `bridget claude [ARGS…]` | lance Claude Code et le connecte au daemon |
| `bridget gemini [ARGS…]` | lance Gemini et le connecte au daemon |
| `bridget gclaude [ARGS…]` | variante `gclaude`, type d'agent `claude` |
| `bridget -- <CMD> [ARGS…]` | agent personnalisé ; le binaire doit figurer dans la liste autorisée |
| `--name <nom>` | nom initial imposé, au lieu du nom auto-incrémenté |

Les arguments qui suivent sont transmis tels quels à l'agent. Pour Codex et
Claude Code, le wrapper ajoute les options de permission nécessaires à
l'ouverture du socket, et injecte un prompt initial expliquant à l'agent comment
répondre — sauf si un prompt est déjà fourni.

### Communiquer

| Commande | Effet |
|---|---|
| `bridget send --to <nom> <msg>` | envoie un message |
| `… --in-reply-to <id>` | lie une réponse à la demande suivie et la clôt après livraison |
| `… --reply` | attend une réponse : la demande est suivie, avec échéance et rappels |
| `… --timeout <s>` | échéance de la demande (défaut : 60 s) |
| `… --hops <n>` | budget de sauts restant (défaut : 4) |
| `… --from <nom>` | émetteur déclaré, pour un relais |
| `bridget reply <msg>` | répond au dernier expéditeur reçu et lie automatiquement sa dernière demande suivie |
| `bridget reply --in-reply-to <id> <msg>` | répond à une demande précise lorsque plusieurs demandes sont en cours |
| `bridget cancel <id> [--reason <texte>]` | annule une demande devenue inutile : plus de rappels, destinataire libéré |
| `bridget requests` | liste mes demandes suivies et leur état |

### Observer

| Commande | Effet |
|---|---|
| `bridget who [--domain <d>]` | annuaire lisible : nom, type, hôte, OS, transport, domaine, modèle, effort, état |
| `bridget agents [--json] [--domain <d>]` | même annuaire, format machine |
| `bridget discover` | alias de `who` |
| `bridget status` | santé du daemon, chemins, nombre d'agents et de messages |
| `bridget ledger` | vingt derniers messages enregistrés |
| `bridget attach <équipier> [--today \| --date AAAA-MM-JJ \| --from-seq N]` | ouvre une vue interactive d’un équipier tenant un journal : historique, suivi en direct et saisie de messages ordinaires |
| `bridget version` | version du binaire |
| `bridget help` | aide en ligne, résumé de toutes les commandes |

Une vue `attach` ne s’ajoute pas à l’annuaire et ne peut pas usurper un
équipier. Elle rejoue d’abord la fenêtre demandée, puis suit le journal en
direct. Dans un terminal, chaque tour reste dans un bloc compact mis à jour
sur place, les appels d’outils portent leur vrai nom et la saisie reste ancrée
en bas pendant l’arrivée des événements. Les séquences de contrôle présentes
dans les réponses sont rendues visibles et les heures du journal portent le
suffixe `UTC` pour éviter toute ambiguïté avec l’heure locale. Une sortie
redirigée conserve le rendu historique ligne par ligne.

Un équipier arrêté est distingué d’un nom inconnu et n’est pas proposé dans
la liste des équipiers attachables. Son journal reste disponible pour une
consultation hors suivi direct ; relancez l’équipier avant de vous y rattacher.

### Se décrire

| Commande | Effet |
|---|---|
| `bridget rename <nom>` | renomme l'agent courant ; le nom survit aux reconnexions |
| `bridget domain <nom>` \| `--reset` | remplace le domaine dérivé, ou y revient |
| `bridget runtime --model <m> [--effort <e>]` | déclare le modèle courant, pour un agent sans détection automatique |
| `bridget dnd [off] [--duration 30m]` | refuse ou accepte à nouveau les interruptions |
| `bridget install-hooks [--remove]` | installe la détection automatique du modèle pour Claude Code |

Les cinq commandes de cette section ne fonctionnent que **depuis un agent
Bridget** : elles s'appuient sur l'identité fournie par le wrapper et échouent
avec un message explicite dans un shell ordinaire.

### Usage interne

`bridget hook claude-runtime` est appelée par le hook Claude Code, lit le payload
sur l'entrée standard et reste silencieuse. Elle n'est pas destinée à un usage
manuel, sinon pour diagnostiquer.

## Réglages et environnement

| Variable | Effet |
|---|---|
| `BRIDGET_TRANSPORT` | nom du transport annoncé dans l'annuaire (défaut : `unix`, ou la valeur lue dans `~/.config/bridget/federation.env`) |
| `BRIDGET_AGENT_NAME` | nom de l'agent, exporté par le wrapper vers le processus agent |
| `BRIDGET_AGENT_NAME_FILE` | fichier portant le nom courant ; c'est lui qui fait foi après un `rename` |
| `HOSTNAME` | hôte annoncé, à défaut la sortie de `hostname` |
| `RUST_LOG=debug` | journalisation détaillée, notamment la source de chaque observation de modèle |

Fichiers, tous sous `~/.cache/bridget/` :

| Chemin | Contenu |
|---|---|
| `bridget.sock` | socket Unix du daemon |
| `bridget.db` | ledger SQLite et demandes suivies |
| `agent-names/` | noms persistants, par session ou par agent actif |
| `agent-domains/` | domaines surchargés |
| `last-sender-<agent>` | dernier expéditeur, pour `bridget reply` |

Valeurs de comportement, en dur dans cette version : disjoncteur 8 échanges par
180 s, déduplication 180 s, quarantaine 3600 s, purge du ledger à 7 jours,
rétention d'une présence injoignable 300 s, battement de cœur 3 s, reconnexion
en 1-2-4-8-16 puis 30 s au plus, sonde de modèle Codex toutes les 20 s,
« ne pas déranger » 60 min par défaut, message de 10 000 caractères et nom
d'agent de 100 caractères au maximum.

## Demandes suivies et relances

Un message ordinaire est une notification : il part, il est livré, l'affaire est
close. Ajouter `--reply` en fait tout autre chose — une **demande suivie**, avec
un identifiant, une échéance et un cycle de vie que le daemon prend en charge.

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget send --to agent-2 --reply "Peux-tu relire crates/bridget-core ?"
# OK: envoyé à « agent-2 » (id=fa09fa7800694, hops=4) [réponse attendue]
```

À partir de là, l'émetteur n'a plus rien à surveiller. Sur une échéance T de
60 secondes par défaut :

| Moment | Ce que fait Bridget |
|---|---|
| T/3 | rappel discret au destinataire : une demande l'attend |
| 2T/3 | rappel ferme |
| T | la demande est marquée échouée et **l'émetteur** en est averti |
| T + 30 s | la demande quitte la liste de surveillance |

C'est la différence entre un message et une demande : un message peut passer
inaperçu, une demande ne peut pas rester lettre morte sans que quelqu'un
l'apprenne. L'échéance s'ajuste avec `--timeout <secondes>` — quelques minutes
pour une relecture de code, quelques secondes pour une question triviale.

L'émetteur garde la main :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget requests
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget cancel fa09fa7800694 --reason "plus utile"
```

Un agent privé des outils MCP natifs peut répondre sans perdre la corrélation :

```bash
bridget send --to agent-1 --in-reply-to fa09fa7800694 "Relecture terminée"
```

La demande est marquée `answered` dans la même transition transactionnelle que
les autres réponses liées ; ses rappels cessent. `bridget reply` reprend
automatiquement l'identifiant de la dernière demande reçue, ou accepte le même
flag pour lever une ambiguïté. Pour lire le sort réel d'un dépôt `in_flight` —
ou lever un `outcome_unknown` —, rejouez à l'identique : **même `id`, même
`issued_at`, même corps**, c'est-à-dire le corps exact et les deux valeurs
affichées. C'est une consultation, jamais une seconde émission ; changer le
corps ferait de ce rejeu une autre enveloppe, que le daemon refuserait en
`envelope_mismatch` :

```bash
bridget send --to agent-1 --in-reply-to fa09fa7800694 \
  --id <id-affiché> --issued-at <instant-affiché> "Relecture terminée"
```

L'annulation est **coopérative** : elle n'interrompt ni un outil ni un modèle
déjà au travail. Elle met fin à la demande, à ses relances, et à l'obligation de
répondre — ce qui évite qu'un agent revienne trente minutes plus tard avec une
réponse à une question devenue sans objet. Elle est idempotente, et un état
terminal n'est jamais rouvert.

L'état survit à un redémarrage du daemon : les demandes encore ouvertes sont
relues depuis SQLite et leur surveillance reprend là où elle s'était arrêtée.

Enfin, les relances respectent le « ne pas déranger » du destinataire : elles sont
suspendues tant qu'il refuse les interruptions. La notification d'échec à
l'émetteur, elle, part quand même — elle ne dérange que celui qui attend.

## Modèle et niveau d'effort des agents

`bridget who` affiche le modèle et le niveau d'effort courants de chaque agent,
tenus à jour quand l'humain en change en cours de session. Le type d'agent
(`claude`, `codex`) ne dit rien de sa capacité réelle : c'est le modèle qui
détermine à qui confier quoi.

```text
  NOM      TYPE    HÔTE         OS     TRANSPORT  MODÈLE         EFFORT  ÉTAT
  agent-1  claude  poste-local  macOS  unix       claude-opus-5  high    connected
  agent-2  codex   poste-local  macOS  unix       gpt-5.6-terra  xhigh   connected
  distant  claude  serveur      Linux  ssh        —              —       unreachable
```

Un tiret cadratin signale une valeur jamais observée — Bridget n'invente jamais
un modèle à partir d'un défaut de configuration. Certains modèles n'exposent
aucun niveau d'effort : la colonne reste alors vide, sans hériter de la valeur
du modèle précédent.

La détection diffère selon l'agent, parce que les deux ne se comportent pas de
la même façon :

| Agent | Mécanisme | Installation |
|-------|-----------|--------------|
| Codex | le wrapper localise le fichier de session que le processus tient ouvert et y lit le dernier contexte de tour | aucune, actif d'office |
| Claude Code | un hook `Stop` rapporte le modèle à la fin de chaque tour | `bridget install-hooks`, une fois |
| Autres | déclaration explicite | `bridget runtime --model <M> --effort <E>` |

### Installer la détection pour Claude Code

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget install-hooks
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget install-hooks --remove
```

La commande **modifie un fichier hors du dépôt** : elle écrit d'abord une
sauvegarde horodatée (`settings.json.bak-AAAAMMJJ-HHMMSS`) et en affiche le
chemin. L'insertion est additive — les hooks déjà présents sont conservés — et
idempotente. Le hook est inerte hors Bridget : une session Claude ordinaire n'en
subit aucun effet. Il ne s'applique qu'aux sessions ouvertes après installation.

La sonde Codex ne consulte le fichier de session que toutes les 20 secondes, et
n'émet vers le daemon que lorsque la valeur a changé : un agent inactif ne
produit aucun trafic.

## Domaines de travail

Chaque agent porte un **domaine**, dérivé sans configuration du dépôt d'où il a
été lancé : la racine git donne le nom, ou le répertoire courant à défaut de
dépôt. Deux agents lancés n'importe où dans le même projet partagent un domaine.

```text
  NOM      TYPE    DOMAINE    MODÈLE         ÉTAT
  agent-1  claude  bridget    claude-opus-5  connected
  agent-2  codex   projet-b   gpt-5.6-terra  connected
  agent-3  claude  projet-b   claude-opus-5  dnd
```

Le domaine **range, il ne cloisonne pas** : tous les agents restent visibles et
joignables entre domaines. C'est un repère pour choisir un destinataire, pas un
mécanisme de sécurité — la communication croisée entre projets est un usage
courant, notamment pour faire relire du code par un agent d'un autre dépôt.

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget who
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget who --domain bridget
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget domain revue-croisee
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget domain --reset
```

Le nom est rendu brut, tel que le répertoire s'appelle : un dépôt rangé sous
`12.mon-projet` donne le domaine `12.mon-projet`, préfixe de classement compris.
Aucune règle d'embellissement implicite, qui serait indevinable ; la surcharge
est là pour les cas où le nom du dépôt ne convient pas.

## Ne pas déranger

Un agent en pleine tâche peut refuser les interruptions :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget dnd
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget dnd --duration 15m
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/target/release/bridget dnd off
```

Son état devient `dnd` dans l'annuaire, et tout message qui lui est adressé est
**refusé avec sa raison** : « agent-1 ne souhaite pas être dérangé (encore
12 min) ». L'émetteur l'apprend tout de suite et décide — attendre, réessayer,
ou s'adresser ailleurs. Rien n'est mis en file d'attente : un message qui
resurgirait une heure plus tard, hors contexte, vaut moins qu'un refus franc.

Les rappels d'escalade des demandes en attente sont suspendus eux aussi. En
revanche la notification d'échec à l'émetteur, elle, est délivrée : elle ne
dérange pas le destinataire.

La durée par défaut est une sécurité : un agent laissé en « ne pas déranger »
redevient joignable seul, sans que personne ait à y penser.

## Tests

```bash
cargo test --manifest-path /Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/Cargo.toml
```

## Déploiement distant

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/scripts/deploy-remote.sh user@exemple.tld 2222 daemon
```

Cette procédure cible Linux : elle installe Rust, compile, déploie le binaire et configure
un service systemd utilisateur. Le mode `daemon` (défaut) crée un daemon Bridget autonome.
Le mode `client-only` installe uniquement le client ; il est requis pour un hôte fédéré afin
de ne pas créer un second daemon et un socket concurrent.

## Plusieurs machines et fédération SSH

Pour enrôler n'importe quelle machine SSH dans le daemon local unique, sans port public :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/scripts/federate-ssh.sh install projet-a --host exemple.tld --user user --port 2222
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/scripts/federate-ssh.sh status projet-a
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/scripts/federate-ssh.sh remove projet-a
```

Le tunnel inverse publie le socket Unix du daemon maître sur la machine distante. Les agents
lancés là-bas utilisent le même annuaire Bridget. L'hôte, l'utilisateur, le port, la clé SSH et le
chemin de socket distant sont paramétrables. Sans `--remote-socket`, le script demande le vrai
`$HOME` à l'hôte Linux et y utilise `$HOME/.cache/bridget/bridget.sock` : aucun chemin `/home/...`
n'est supposé. Pour installer le binaire client sur cet hôte sans daemon distant :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/015-guichet-maicie/scripts/deploy-remote.sh user@exemple.tld 2222 client-only
```

Lorsqu'un tunnel est temporairement coupé, le processus IA distant continue son travail. Son
wrapper Bridget retire alors l'agent de l'annuaire, puis tente une reconnexion avec backoff
exponentiel et jitter (environ 1, 2, 4, 8, 16 puis au plus 30 secondes). Après 60 secondes de
connexion stable, ce délai est remis à son minimum. L'agent est réinscrit automatiquement sous le
même nom dès que le socket SSH réapparaît, y compris s'il a été renommé entre-temps
par `bridget rename`.

`bridget who` affiche aussi l'hôte d'exécution, l'OS, le transport et l'état de présence dans des
colonnes alignées. L'OS est détecté par le wrapper (`macOS`, `Linux`, etc.) afin d'aiguiller une
demande vers les outils réellement disponibles. Après une coupure, une instance distante reste
visible comme `unreachable` pendant cinq minutes, ce qui permet de distinguer une perte de réseau
d'un arrêt volontaire de l'agent.

### Prérequis SSH côté Linux

Le serveur SSH doit autoriser le transfert inverse pour le compte enrôlé. Sur un serveur durci,
l'administrateur peut ajouter un fichier tel que `/etc/ssh/sshd_config.d/60-bridget.conf` :

```text
Match User user
    AllowTcpForwarding remote
    AllowStreamLocalForwarding yes
```

Puis il valide et recharge le service (`sshd -t` puis `systemctl reload ssh`). Cette permission
reste limitée au compte concerné ; Bridget ne requiert ni port TCP public ni daemon Bridget sur
l'hôte Linux fédéré.

## Consulter une vue distante (lecture seule)

Depuis le portable, une seule commande lance le relais sur la boucle locale de
l'hôte distant, crée un tunnel SSH lié à `127.0.0.1` sur les deux machines et
ouvre la page dans le navigateur local :

```bash
/Users/moi/Nextcloud/10.Scripts/bridget/scripts/open-remote-ui.sh user@exemple.tld --maicie-config /home/user/.config/maicie/config.json
```

`--maicie-config` est un chemin absolu **sur l'hôte distant**. Le tunnel reste
attaché au terminal et se ferme avec `Ctrl-C`; le jeton et le relais HTTP ne
sont jamais publiés sur le réseau. Cette page est strictement en lecture seule :
elle ne peut ni lancer, ni arrêter, ni approuver quoi que ce soit.

## Licence

MIT — voir [LICENSE](LICENSE).
