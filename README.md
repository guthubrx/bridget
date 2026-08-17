# Bridget

Protocole de communication inter-agents CLI, pair-à-pair, transport-agnostique.

## Vue d'ensemble

Bridget permet à des agents CLI (Codex, Claude, Gemini) de communiquer entre
eux en temps réel via un daemon central. Le protocole est indépendant du
transport : tmux aujourd'hui, socket réseau demain.

## Démarrage rapide

```bash
# Compiler
cargo build --release

# Lancer le daemon
./target/release/bridget daemon &

# Lancer un agent dans tmux
bridget codex

# Dans un autre terminal, envoyer un message
bridget send --to codex-1 "Analyse ce fichier" --reply

# Voir les agents connectés
bridget who
```

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

## Sécurités

- **Disjoncteur** — max 8 échanges par conversation en 180s (configurable)
- **Déduplication par contenu** — bloque les doublons d'envoi
- **Quarantaine par ID** — bloque les retransmissions (misroute)
- **Hops** — budget de sauts anti-boucle (défaut : 4)
- **Reply yes/no** — distingue questions et affirmations
- **Auto-envoi interdit** — un agent ne peut pas se parler à lui-même
- **Escalade progressive** — rappels automatiques à T/3, 2T/3 puis notification d'échec à T
- **Timeout configurable** — `--timeout <secondes>` (défaut 60s)
- **Demandes annulables** — une demande `--reply` possède un identifiant ; son émetteur peut l'arrêter avec `bridget cancel <id>` afin de supprimer les rappels et de libérer le destinataire de toute réponse.

## Commandes

```
bridget daemon                              Lance le daemon
bridget codex                               Lance Codex + connexion daemon
bridget claude                              Lance Claude + connexion daemon
bridget send --to <nom> <msg>               Envoie un message
bridget send --to <nom> --reply <msg>       Envoie en attendant une réponse
bridget send --to <nom> --reply --timeout 300 <msg>  Timeout personnalisé
bridget reply <msg>                         Répond au dernier expéditeur reçu
bridget cancel <id> --reason <texte>         Annule une demande suivie devenue inutile
bridget requests                            Liste mes demandes suivies et leur état
bridget runtime --model <M> [--effort <E>]  Déclare le modèle courant de l'agent
bridget install-hooks [--remove]            Détection auto du modèle pour Claude
bridget domain <nom> | --reset              Change le domaine de l'agent courant
bridget dnd [off] [--duration 30m]          Ne pas déranger
bridget who [--domain <D>]                  Liste les agents connectés
bridget agents --json                       Liste au format JSON
bridget status                              Santé du daemon
bridget ledger                              Historique des messages
```

Une annulation est coopérative : elle n'interrompt pas un outil ou un modèle déjà en train de travailler, mais elle met fin à la demande Bridget, à ses relances et à l'obligation de répondre. `bridget requests` permet à l'émetteur de consulter ses demandes et leurs états.

## Modèle et niveau d'effort des agents

`bridget who` affiche le modèle et le niveau d'effort courants de chaque agent,
tenus à jour quand l'humain en change en cours de session. Le type d'agent
(`claude`, `codex`) ne dit rien de sa capacité réelle : c'est le modèle qui
détermine à qui confier quoi.

```text
  NOM      TYPE    HÔTE      OS     TRANSPORT  MODÈLE         EFFORT  ÉTAT
  agent-2    claude  macbook   macOS  unix       claude-opus-5  high    connected
  agent-1  codex   macbook   macOS  unix       gpt-5.3-codex  xhigh   connected
  distant  claude  projet-a    Linux  ssh        —              —       unreachable
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
bridget install-hooks            # ajoute un hook Stop dans ~/.claude/settings.json
bridget install-hooks --remove   # le retire
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
  NOM            TYPE    DOMAINE            MODÈLE         ÉTAT
  agent-2          claude  bridget            claude-opus-5  connected
  agent-1        codex   projet-b     gpt-5.6-terra  connected
  agent-3   claude  projet-b     claude-opus-5  dnd
```

Le domaine **range, il ne cloisonne pas** : tous les agents restent visibles et
joignables entre domaines. C'est un repère pour choisir un destinataire, pas un
mécanisme de sécurité — la communication croisée entre projets est un usage
courant, notamment pour faire relire du code par un agent d'un autre dépôt.

```bash
bridget who                       # tous les agents, avec leur domaine
bridget who --domain bridget      # seulement ce domaine
bridget domain revue-croisee      # surcharge, conservée après reconnexion
bridget domain --reset            # retour au domaine dérivé du dépôt
```

Le nom est rendu brut : un répertoire `projet-b` donne le domaine
`projet-b`. Aucune règle d'embellissement implicite ; la surcharge
est là pour ça.

## Ne pas déranger

Un agent en pleine tâche peut refuser les interruptions :

```bash
bridget dnd                    # 60 minutes par défaut
bridget dnd --duration 15m     # ou 90s, 2h
bridget dnd off                # levée immédiate
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
cargo test          # 83 tests (unitaires + intégration)
```

## Déploiement distant

```bash
./scripts/deploy-remote.sh <utilisateur@hôte> [port] [daemon|client-only]
```

Cette procédure cible Linux : elle installe Rust, compile, déploie le binaire et configure
un service systemd utilisateur. Le mode `daemon` (défaut) crée un daemon Bridget autonome.
Le mode `client-only` installe uniquement le client ; il est requis pour un hôte fédéré afin
de ne pas créer un second daemon et un socket concurrent.

## Fédération SSH

Pour enrôler n'importe quelle machine SSH dans le daemon local unique, sans port public :

```bash
./scripts/federate-ssh.sh install projet-a --host exemple.tld --user user --port 2222
./scripts/federate-ssh.sh status projet-a
./scripts/federate-ssh.sh remove projet-a
```

Le tunnel inverse publie le socket Unix du daemon maître sur la machine distante. Les agents
lancés là-bas utilisent le même annuaire Bridget. L'hôte, l'utilisateur, le port, la clé SSH et le
chemin de socket distant sont paramétrables. Sans `--remote-socket`, le script demande le vrai
`$HOME` à l'hôte Linux et y utilise `$HOME/.cache/bridget/bridget.sock` : aucun chemin `/home/...`
n'est supposé. Pour installer le binaire client sur cet hôte sans daemon distant :

```bash
./scripts/deploy-remote.sh user@exemple.tld 2222 client-only
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

## Licence

MIT
