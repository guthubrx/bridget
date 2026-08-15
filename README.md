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
bridget who                                 Liste les agents connectés
bridget agents --json                       Liste au format JSON
bridget status                              Santé du daemon
bridget ledger                              Historique des messages
```

Une annulation est coopérative : elle n'interrompt pas un outil ou un modèle déjà en train de travailler, mais elle met fin à la demande Bridget, à ses relances et à l'obligation de répondre. `bridget requests` permet à l'émetteur de consulter ses demandes et leurs états.

## Tests

```bash
cargo test          # 36 tests (unitaires + intégration)
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
même nom dès que le socket SSH réapparaît.

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
