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

## Commandes

```
bridget daemon                              Lance le daemon
bridget codex                               Lance Codex + connexion daemon
bridget claude                              Lance Claude + connexion daemon
bridget send --to <nom> <msg>               Envoie un message
bridget send --to <nom> --reply <msg>       Envoie en attendant une réponse
bridget send --to <nom> --reply --timeout 300 <msg>  Timeout personnalisé
bridget reply <msg>                         Répond au dernier expéditeur reçu
bridget who                                 Liste les agents connectés
bridget agents --json                       Liste au format JSON
bridget status                              Santé du daemon
bridget ledger                              Historique des messages
```

## Tests

```bash
cargo test          # 36 tests (unitaires + intégration)
```

## Déploiement distant

```bash
./scripts/deploy-remote.sh <host> [port]
```

Installe Rust, compile, déploie le binaire et configure systemd sur le serveur.

## Licence

MIT
