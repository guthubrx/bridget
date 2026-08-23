# Maicie v3

Maicie est un compagnon CLI de coordination construit au-dessus des contrats
publics de Bridget. Elle transforme une instruction explicite en objectif et en
délégation durables, sans devenir un moteur de workflow ni un superviseur de
processus.

## Démarrer

Compiler Bridget et Maicie depuis la racine du dépôt :

```bash
cargo build --release -p bridget-daemon -p maicie
./target/release/bridget daemon
```

Dans un autre terminal, créer un fichier de configuration. Tous les chemins
doivent être absolus et la base Maicie doit rester distincte de `bridget.db` :

```json
{
  "version": 1,
  "bridget_socket": "/chemin/absolu/.cache/bridget/bridget.sock",
  "database_path": "/chemin/absolu/.local/state/maicie/maicie.sqlite3",
  "durations": {
    "short_secs": 30,
    "normal_secs": 300,
    "long_secs": 3600
  },
  "status_capture_budget_ms": 250,
  "profiles": [{
    "id": "reviewer",
    "agent_name": "reviewer",
    "agent_type": "codex",
    "model": "modele-epingle",
    "effort": "high",
    "display_name": "Relecture",
    "tags": ["review"],
    "personality_ref": "profiles/reviewer.md",
    "tools": ["bridget_send"],
    "spawn_order_ref": "agents/reviewer"
  }]
}
```

Les commandes sont des invocations courtes. Chacune charge la configuration et
ouvre la SQLite privée. Les commandes qui contactent Bridget réconcilient les
outboxes non terminales avant leur action ; les commandes d'objectif restent
strictement locales. Toutes rendent ensuite la main :

```bash
./target/release/maicie delegate \
  --config /chemin/absolu/maicie.json \
  --goal "Relire le risque de reprise" \
  --to reviewer \
  --duration normale \
  --idempotency-key delegation-review-1 \
  --json

./target/release/maicie status --config /chemin/absolu/maicie.json --json
```

Réutiliser la même clé d'idempotence avec le même contenu rejoue le même
résultat. Réutiliser cette clé avec un contenu différent est refusé.

## Arrêt

Maicie ne maintient aucun service résident, timer ou processus enfant. Une
invocation peut être interrompue avec `Ctrl-C` ; les décisions et outboxes déjà
commitées restent dans sa SQLite et seront réconciliées à l'invocation suivante.
Le daemon et les équipiers restent sous la responsabilité de Bridget
(`bridget stop <nom>` pour un équipier géré).

Si Bridget utilise un registre utilisateur `agents.json`, ce fichier doit être
régulier, non symbolique et avoir le mode `0600` ou plus restrictif ; le daemon
refuse de démarrer avec un registre plus permissif.

## Deux autorités, jamais une vérité fusionnée

| Autorité | Ce qu'elle possède |
|---|---|
| SQLite Maicie | objectifs, délégations, décisions et remises locales durables |
| Bridget | présence, livraison, demandes suivies et snapshot de transport |

Une outbox Maicie décrit ce qu'elle a durablement préparé ou appris d'une issue.
Un snapshot Bridget décrit ce que le transport a observé, avec sa source et sa
fraîcheur. Une coupure, un `Gap` ou un `End` rend le transport incomplet ; cela
ne clôt jamais un objectif et ne réécrit pas l'état métier.

## Outboxes transactionnelles et reprise

La délégation et son enveloppe filaire complète sont écrites dans la même
transaction avant toute I/O. Après une coupure, une ligne `prepared` ou
`outcome_unknown` commence par consulter l'issue durable de son `message_id`.
Sans issue et sous l'horizon contractuel, Maicie rejoue les octets persistés
avec le même identifiant. `IdempotencyExpired` est terminal : aucune nouvelle
émission implicite n'est créée.

L'activation d'un profil suit la même discipline avec
`ActivationOutbox(command_id, spawn_order_bytes)`. Le protocole 009 n'expose
pas de `SpawnLookup` séparé : rejouer exactement le `SpawnOrder` avec le même
`command_id` constitue le lookup idempotent. Après acceptation, Maicie compare
le digest de définition renvoyé par `SpawnAccepted` au hash épinglé dans
l'approbation. Elle ne relit jamais `agents.json` pour refaire cette preuve.

## Limites assumées

- comportement déterministe, sans LLM, interprétation libre ni sélection
  sémantique cachée ;
- sélection explicite ou égalité stricte de tags déclarés ;
- aucun `Child`, `Command::spawn`, arrêt ou redémarrage de processus dans
  Maicie : tout lancement passe par `SpawnOrder` Bridget ;
- `approve` est une frappe humaine locale exclusivement, dans le modèle de
  confiance mono-utilisateur ; cette capacité n'est exposée ni à Bridget ni à
  MCP ;
- les permissions ACP sont des faits déjà auto-décidés par Bridget, jamais une
  attente d'approbation humaine dans Maicie ;
- aucune lecture de `bridget.db`, du journal ACP ou d'un fichier interne
  Bridget : seul le protocole public est consommé.

Les scénarios de validation sont décrits dans
`specs/011-maicie-orchestration/quickstart.md`.
