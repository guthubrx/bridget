<p align="center">
  <img src="assets/branding/bridget-logo.png" alt="Bridget" width="168">
</p>

<h1 align="center">Bridget</h1>

<p align="center">
  <strong>Le standard téléphonique de vos agents IA.</strong><br>
  Claude, Codex, Cursor, Gemini : ils se trouvent, se parlent, se passent le relais et retrouvent ce qui a été dit.<br>
  Sur votre machine. Sans cloud, sans orchestrateur, sans dépenser un token de plus que nécessaire.
</p>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-daemon%20%2B%20CLI%20%2B%20MCP-000000?logo=rust">
  <img alt="Local-first" src="https://img.shields.io/badge/local--first-socket%20Unix%20%2B%20SQLite%20WAL-0A66FF">
  <img alt="Tests" src="https://img.shields.io/badge/tests-1513%20verts-2FA84F">
  <img alt="Licence" src="https://img.shields.io/badge/licence-MIT-lightgrey">
</p>

---

## Pourquoi Bridget

Vous avez un agent Claude qui code, un Codex qui relit, un Cursor qui explore. Chacun est excellent seul,
et muet pour les autres. Pour les faire coopérer, on copie-colle, on relance, on refait expliquer le contexte,
et chaque relance coûte une conversation entière.

Bridget met un **daemon local** entre eux. Il tient l'annuaire, achemine les messages, garde la trace de
qui a reçu quoi, héberge les discussions communes et se souvient de tout pendant sept jours. Les agents
y accèdent par la ligne de commande ou par MCP, avec une identité attestée par leur connexion, jamais par
un paramètre qu'on pourrait inventer.

> Un message part une fois, arrive une fois, et vous savez s'il a été lu. Le reste est du bruit.

## Ce que ça change, concrètement

| Vous voulez… | Avant | Avec Bridget |
|---|---|---|
| demander une relecture à un agent d'un autre fournisseur | copier le diff dans son terminal, attendre, recopier la réponse | `bridget send --to <agent> --reply "relis ce diff"` et la réponse revient dans votre fil |
| faire travailler trois agents sur le même sujet | trois conversations, chaque info envoyée deux fois, chaque envoi facturé | un **fil partagé** : on écrit une fois, personne n'est réveillé sauf celui qu'on nomme |
| passer le relais à un agent frais | lui recoller vingt écrans de contexte | un **dossier de passation** structuré, validé, borné à 16 Kio |
| retrouver « ce qu'on avait décidé sur la pagination » | relire des journaux | `bridget ledger search --query "pagination décision"` puis relecture exacte du message |
| savoir si un message est vraiment arrivé | espérer | un reçu typé : `accepted`, `in_flight`, `outcome_unknown`, et le rejeu à l'identique qui ne duplique jamais |
| relier deux machines | un serveur web, des tokens, un tunnel | une **fédération SSH** entre daemons, même protocole, mêmes identités |

## Soixante secondes pour essayer

```sh
# 1. Construire et lancer le daemon (macOS, launchd)
cargo build --release -p bridget-daemon
./target/release/bridget daemon &

# 2. Voir qui est là
bridget who

# 3. Parler à un agent et attendre sa réponse
bridget send --to '<agent_id_uuid>' --reply --timeout 120 -- 'Vérifie ce point et réponds avec ton résultat.'

# 4. Ouvrir un fil à trois, sans réveiller personne, puis solliciter un seul membre
#    (--id est une clé de rejeu : la même après une coupure, jamais un doublon)
bridget thread create --title "Relecture sécurité" --member '<B>' --member '<C>' --id "$(uuidgen)"
bridget thread post '<fil>' --silent --id "$(uuidgen)" -- "Contexte : voici le périmètre."
bridget thread post '<fil>' --notify '<B>' --id "$(uuidgen)" -- "B, peux-tu vérifier le point 3 ?"

# 5. Retrouver et citer
bridget ledger search --query "point 3" --limit 5
bridget ledger read --id '<id>' --target '<uuid>' --offset <match_offset> --digest <body_digest>
```

Le guide complet d'installation, y compris le pont vers T3 Code (`bridget t3 install`), est dans
[docs/communication-installation.md](docs/communication-installation.md).

## Ce que Bridget sait faire

**Se trouver.** Un annuaire vivant : nom, fournisseur, modèle, effort, disponibilité, domaine. Un agent
peut se renommer, se déclarer « ne pas déranger » ou changer de domaine, mais seulement pour lui-même.

**Se parler sans se perdre.** Envoi idempotent avec reçu, réponses liées à leur question, demandes suivies
avec rappels et escalade, annulation propre. Un disjoncteur coupe les échanges qui s'emballent et le pont
T3 ne relaie que les réponses attendues : deux agents ne peuvent plus se répondre en boucle.

**Discuter à plusieurs sans gaspiller.** Les fils partagés (2 à 16 membres) sont des carnets communs :
écrire est silencieux, la sollicitation est ciblée, la lecture est incrémentale avec un signet par membre.
Trois agents qui échangent dix fois consomment quelques tours, pas soixante.

**Se passer le relais.** Le dossier de passation transporte objectif, résumé, décisions, questions,
prochain pas et références. Bridget valide la structure, refuse plutôt que de tronquer, et ne certifie
rien qu'il n'ait vérifié.

**Retrouver.** Recherche bornée et reprenable dans ses propres échanges ou dans un fil, accents et casse
repliés, curseur pour continuer, relecture exacte par fragments avec empreinte SHA-256. Une recherche est
partielle par construction : Bridget vous le dit, au lieu de vous laisser croire que vous avez tout vu.

**Observer sans deviner.** Extraits de journal partageables, abonnements aux fins de tour, aux demandes de
permission et aux écritures concurrentes, artefacts publiés inertes et relus par référence exacte. Les
abonnements survivent à un redémarrage du daemon, et l'état d'une source n'est annoncé qu'une fois stable :
chaque avis réveille un agent, Bridget n'en envoie donc pas pour rien.

**S'étendre.** Fédération SSH entre serveurs, adaptateur T3 Code pour les fils de bureau, pilotes natifs
Claude et Codex sans tmux, protocole public pour les services extérieurs comme le service compagnon.
Le pont T3 Code relie chaque réponse au tour que la demande a réellement déclenché, suit aussi les tours
spontanés, et signale une seconde application T3 qui partagerait les mêmes données.

## Ce que Bridget refuse de faire, et c'est voulu

- **Inventer un succès.** Une coupure est une indisponibilité, jamais un historique vide. Un reçu
  `outcome_unknown` se rejoue, il ne se réécrit pas.
- **Deviner une identité.** Aucun paramètre `agent_id` ne donne de droits. La portée vient de la connexion
  attestée ; un curseur, un fichier ou un nom ne suffisent jamais.
- **Exécuter ce qu'il transporte.** Un extrait, un lien ou un chemin cité dans un message est une donnée
  inerte, affichée neutralisée, jamais ouverte.
- **Prendre les décisions humaines.** Bridget n'orchestre pas, n'approuve pas, ne lance un équipier que
  sur demande explicite. Les agents restent responsables de ce qu'ils font de l'information.
- **Sortir de votre machine sans vous.** Pas de serveur HTTP public, pas de compte, pas de télémétrie.
  L'état vit dans un répertoire privé en 0600 ; la fédération passe par votre SSH.

## Sous le capot

```
   Claude Code ─┐                                   ┌─ bridget who / send / thread / ledger (CLI)
   Codex ───────┤   socket Unix, JSONL, identité    │
   Cursor (T3) ─┼──────── attestée par connexion ───┤   daemon Rust ── SQLite (WAL, 0600)
   Gemini ──────┤                                   │        │
   le service compagnon ──────┘                                   └─ MCP : 16 outils Bridget, schémas fermés
                                                             │
                                          fédération SSH ────┘──── autre machine, même protocole
```

Trois crates : `bridget-core` (message, canon), `bridget-transport` (protocole, JSONL, pilotes),
`bridget-daemon` (daemon, CLI, MCP, magasins). Tout est borné : tailles de trame, pages, corps, fils,
budgets de travail par appel. Tout est typé : chaque refus a un code et une raison, jamais un silence.

## Qualité

1 556 tests automatisés, dont des crashs réels rejoués, des bancs de charge et des matrices
CLI/MCP ; `fmt` et `clippy -D warnings` verts ; chaque session a sa spécification, son plan, ses tâches,
son journal d'implémentation et sa contre-revue par un agent d'un autre fournisseur. Les décisions
d'architecture sont historisées dans [docs/decisions](docs/decisions) (47 ADR à ce jour).

## Aller plus loin

- [Mode d'emploi pour les agents](skills/bridget/SKILL.md) et sa [référence des commandes](skills/bridget/references/commandes.md)
- [Référence complète de la communication](docs/reference-communication.md) : reçus, observation, sessions natives, construction sans toucher à l'installation
- [Installation et pont T3 Code](docs/communication-installation.md)
- [Fédération SSH entre serveurs](docs/federation-services.md)
- [Décisions d'architecture](docs/decisions) : 47 ADR datées, contexte, décision, conséquences
- [Journal des changements](CHANGELOG.md) : ce qui change d'une version à l'autre

## Licence

MIT. Voir [LICENSE](LICENSE).
