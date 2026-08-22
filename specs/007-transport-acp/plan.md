# Implementation Plan : Transport ACP pour agents équipiers

**Branch**: `session-07-transport-acp` | **Date**: 2026-08-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-transport-acp/spec.md`

## Summary

Deuxième implémentation du trait `Transport` existant : au lieu d'injecter les
messages dans un pane tmux et d'exiger des règles de prompt, le wrapper lance
l'agent comme sous-processus **ACP** (JSON-RPC 2.0 sur stdio), livre chaque
message comme un tour de prompt, capture la réponse à la fin du tour et la
route automatiquement. Les types d'agents ACP sont déclarés dans un **registre
de configuration** (Codex d'abord, puis Claude, puis Gemini — l'ajout d'un type
ne demande aucune modification de code). Le transport tmux reste le repli
inchangé pour les agents interactifs.

Arbitrages utilisateur actés en conversation (2026-08-22) :

1. Priorité des agents : **Codex > Claude > Gemini**.
2. Ouverture : ajouter un agent a posteriori = une entrée de configuration,
   zéro code.
3. Facturation : abonnements CLI uniquement — refus de lancement si clé API
   dans l'environnement.
4. Propreté : tout chemin hérité rendu obsolète est supprimé ou inscrit au
   registre des dépréciations (`docs/DEPRECATIONS.md`, créé par cette session).

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget` (3 crates existantes)
**Primary Dependencies**: aucune nouvelle crate (D-201) ; `npx` requis à
l'exécution pour les adaptateurs Codex/Claude (versions pinnées en config)
**Storage**: registre d'agents en fichier JSON (`~/.config/bridget/agents.json`,
lu avec `serde_json` déjà présent — arbitrage au reuse-audit), journaux de
session en JSONL sous `~/.cache/bridget/sessions/`
**Testing**: `cargo test` (unitaires + tests de conformité sur fixtures JSON-RPC
enregistrées) ; test d'intégration manuel scénarisé dans `quickstart.md`
**Target Platform**: macOS et Linux
**Project Type**: CLI/daemon existant — extension d'un trait
**Constraints**: zéro régression tmux (SC-006) ; pas de runtime async ; corps de
messages intacts octet pour octet (SC-002)
**Scale/Scope**: 1 module transport nouveau, 1 module registre, ~800-1200
lignes, 12 tâches (périmètre élargi par la contre-revue : file bornée, matrice
de conformité JSON-RPC, corrections D-208/D-209 du daemon et du protocole,
gate fédération dédié)

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | artefacts, messages d'erreur et journaux en français |
| VII — ADR | ✅ | ADR 003 « ACP comme transport de livraison » à créer en tâche 1 (décision structurante : nouveau protocole externe) |
| IX — recherche préalable | ✅ | baseline `04-architectures-patterns.md` + validation live du 2026-08-22 consignée dans `research.md` (R-001..R-004, sources datées) |
| XVIII — complexité | ✅ | tout est linéaire en nombre de messages ; la file d'attente par équipier est un `VecDeque` (push/pop O(1)) ; aucune boucle imbriquée sur collections |
| XIX — minimalisme | ✅ | zéro nouvelle dépendance (D-201 : sous-ensemble JSON-RPC maison ~7 méthodes, justifié dans research R-004) ; réutilisation du trait `Transport`, du protocole wrapper↔daemon et du cycle de vie des demandes ; le registre JSON remplace une liste en dur (suppression nette) |
| XX — responsabilité | ✅ | chaque décision D-2xx porte sa justification et sa limite ; le compromis « client maison vs crate officielle » est écrit et réversible |

**Point de vigilance** : le tour ACP introduit un état « en cours » entre la
livraison et la réponse. Un tour qui ne se termine jamais (adaptateur gelé)
transformerait la file d'attente en trou noir — d'où D-206 (le daemon reste
l'autorité des échéances, le transport n'annule que le tour actif expiré, les
notifications ont leur timeout de transport) et FR-009 (échec motivé, jamais un
silence).

## Project Structure

### Documentation (this feature)

```text
specs/007-transport-acp/
├── plan.md              # ce fichier
├── research.md          # inconnues résolues, sources datées
├── data-model.md        # registre d'agents + journal de session
├── contracts/
│   └── livraison-acp.md # mapping message Bridget ↔ tour ACP
├── quickstart.md        # scénario de validation manuelle pour les devs
└── tasks.md             # généré en phase tasks
```

### Source Code (repository root)

```text
crates/bridget-transport/src/
├── acp.rs               # NOUVEAU : AcpTransport + client JSON-RPC minimal
├── tmux.rs              # inchangé (fallback)
├── transport.rs         # trait — extension pour la capture de réponse (D-204)
├── protocol.rs          # variante(s) wrapper→daemon pour réponse + état de tour
└── lib.rs               # export du nouveau module

crates/bridget-daemon/src/
├── registry.rs          # NOUVEAU : lecture/validation du registre agents.json
├── wrapper.rs           # branchement du transport ACP + garde clé API
│                        #   + suppression de la livraison stderr (FR-013)
├── cli.rs               # sous-commandes : remplacement de la liste en dur
│                        #   par le registre ; flag de lancement équipier
└── daemon.rs            # relances informées par l'état de tour (FR-008)

docs/
├── decisions/003-transport-acp.md   # NOUVEAU : ADR
└── DEPRECATIONS.md                  # NOUVEAU : registre des dépréciations

tests/  (dans chaque crate, style existant)
└── fixtures JSON-RPC enregistrées pour les tests de conformité
```

**Structure Decision** : extension en place des crates existantes — aucun
nouveau crate. `acp.rs` vit dans `bridget-transport` à côté de `tmux.rs`
(symétrie voulue par l'architecture « le protocole est indépendant de son
transport »). Le registre vit dans `bridget-daemon` car c'est le wrapper qui
lance les agents aujourd'hui (le daemon-spawn est la session 09).

## Approche par exigence

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001 | lancement sous-processus adaptateur (stdio pipés), `initialize` + `session/new`, enregistrement daemon inchangé | `acp.rs`, `wrapper.rs` |
| FR-002, FR-005 | un message = `session/prompt` avec en-tête sobre + corps brut (contrat) | `acp.rs`, `contracts/livraison-acp.md` |
| FR-003 | réponse du tour capturée au `stopReason` → remise au daemon → routage + clôture (cycle de vie 003 réutilisé) | `acp.rs`, `protocol.rs`, `daemon.rs` |
| FR-004 | aucun prompt injecté en mode ACP (le bloc « Règles ABSOLUES » reste réservé au mode tmux) | `wrapper.rs` |
| FR-006 | registre JSON : type → commande, args (version épinglée dans la commande), variables refusées, politique permissions | `registry.rs`, `data-model.md` |
| FR-007 | file `VecDeque` **bornée** par équipier (politique de file du data-model : capacité, refus motivé, purge), livraison au prochain tour libre ; garde-fous existants intouchés (ils vivent dans le daemon) | `acp.rs` |
| FR-008 | état de tour remonté au daemon (variante protocole) ; relance différée si tour en cours | `protocol.rs`, `daemon.rs` |
| FR-009 | mort du process / `stopReason` d'erreur / dépassement d'échéance du message actif ou timeout de transport (D-206) → échec motivé vers l'émetteur | `acp.rs`, `daemon.rs` |
| FR-010 | journal JSONL par session : événements de tour horodatés | `acp.rs` |
| FR-011 | garde clé API au lancement, contournement `BRIDGET_ALLOW_API_KEY=1` | `wrapper.rs`, `registry.rs` |
| FR-012 | sélection du transport au lancement : flag équipier → ACP, sinon comportement actuel | `cli.rs`, `wrapper.rs` |
| FR-013 | suppression de la branche stderr (`wrapper.rs:822`) ; liste en dur des binaires remplacée par le registre ; entrées au `DEPRECATIONS.md` | `wrapper.rs`, `cli.rs`, `docs/DEPRECATIONS.md` |
| FR-014 | rien à faire : le transport est local au wrapper, la fédération transporte des messages daemon↔daemon — test d'intégration pour le prouver | `quickstart.md` |

## Décisions de conception

**D-201 — Client JSON-RPC minimal maison, pas la crate officielle.** La crate
`agent-client-protocol` v2.0 impose un écosystème async (`async-io`,
`async-process`, `futures`…) à un code intégralement synchrone. Le sous-ensemble
nécessaire (7 méthodes, R-001) tient dans le style existant : threads,
`BufReader` ligne à ligne, `serde_json`. Compromis écrit et réversible ; le
schéma officiel sert de référence de conformité. (Justification complète :
research R-004.)

**D-202 — Le registre d'agents est de la configuration, pas du code.** Fichier
JSON utilisateur (`agents.json`, format arbitré au reuse-audit : `serde_json`
est déjà dans le workspace, TOML aurait exigé une crate nouvelle), une entrée
par type d'agent : commande, arguments (version épinglée dans la commande,
`@paquet@x.y.z`), variables d'environnement refusées, politique de permissions. La
liste blanche en dur de `wrapper.rs:522` est supprimée au profit du registre
(les types non déclarés sont refusés — même garantie, extensible). Un registre
par défaut embarqué couvre codex/claude/gemini aux versions pinnées de R-002.

**D-203 — Un équipier = une session ACP durable.** `session/new` au lancement,
tous les messages arrivent dans la même session (l'équipier garde son contexte
conversationnel, comme un agent tmux garde son historique). La reprise de
session (`session/load`) est hors périmètre (l'identité persistante — session
006 — couvre le nom, pas le contexte).

**D-204 — La capture de réponse étend le contrat de transport sans le casser,
avec un modèle de threads explicite.** Le point critique (contre-revue
cxbridget, objection 1) : `deliver()` est appelé par le thread d'écoute du
wrapper (`wrapper.rs:817`) — s'il bloquait jusqu'à la fin du tour, le second
message ne pourrait jamais être mis en file, et plusieurs lecteurs de stdout
corrompraient le démultiplexage JSON-RPC. Modèle imposé :

- **un seul thread lecteur**, propriétaire exclusif du stdout de l'adaptateur,
  qui démultiplexe réponses (table `request_id → waiter`), notifications et
  requêtes serveur→client — y compris `session/request_permission` reçue
  *pendant* un prompt en cours ;
- **un writer sérialisé** (mutex) — jamais d'écriture entrelacée sur stdin ;
- **un thread worker de tours** qui dépile la file et exécute les
  `session/prompt` séquentiellement ;
- **`deliver()` ne fait que valider, enfiler et retourner** — jamais bloquant.

**Propriétaire unique** (round 2 de la contre-revue, objection 1) :
`AcpTransport` possède seul `turn` et `queue`. Le wrapper ne tient aucune file
propre : il branche le transport et lui relaie les signaux de cycle de vie
(annulation, échéance) — deux files casseraient l'ordre FIFO, l'annulation et
la contre-pression.

La remontée de réponse et d'état de tour passe par le canal wrapper→daemon
existant. Le trait `Transport` gagne au plus une méthode d'interrogation d'état
— pas de callback, pas de généricité spéculative (Article XIX).

**D-209 — Annulation et refus de livraison deviennent des messages typés.**
(Round 2, objection 2.) Le daemon accuse réception à l'émetteur dès le push
vers le wrapper — un refus « immédiat » de file pleine est donc impossible, et
l'annulation actuelle n'est qu'un texte livré, impurgeable d'une file par id.
Deux variantes de protocole sont ajoutées :

- `CancelDelivery { id, reason }` (daemon→wrapper) : retire un message de la
  file du transport (annulation ou expiration d'une demande) ;
- `DeliveryRejected { id, reason }` (wrapper→daemon) : remonte un échec
  terminal (file pleine, purge sur échéance, mort du processus) ; le daemon le
  transforme en échec motivé vers l'émetteur (mécanisme FR-009).

Le dépassement de capacité est ainsi un **échec asynchrone terminal motivé**,
pas un refus synchrone.

**D-205 — Réponse = texte final du tour, notifications = pas de réponse.** Un
message `reply=yes` attend le texte de fin de tour et clôt la demande. Un
message `reply=no` est livré comme tour mais sa réponse éventuelle n'est pas
routée (parité avec la sémantique actuelle « notification »). Le texte du tour
reste au journal dans les deux cas.

**D-206 — Le daemon reste l'autorité unique du cycle de vie ; le transport ne
gère que le tour actif.** (Reformulé après contre-revue cxbridget, objection
3 : la version initiale « échéance la plus contraignante » était fausse — un
tour `reply=no` n'a pas d'échéance, une demande plus courte en file ne doit pas
annuler le tour actif antérieur, et le daemon marque déjà les demandes expirées,
d'où un risque de double échec.) Règles :

- le daemon garde seul la responsabilité d'expirer une demande et de prévenir
  l'émetteur (mécanisme 003 inchangé) ;
- le transport n'annule (`session/cancel`, avec délai de grâce borné) que le
  tour **actif** dont le message porteur a dépassé sa propre échéance ;
- un message en file dont l'échéance est dépassée est **retiré** avant tour et
  jamais livré en retard (contrôle de deadline au dépilage) ;
- les tours de notification (`reply=no`) relèvent d'un timeout de transport
  configurable (valeur par défaut au registre), indépendant du ledger.

**D-208 — La clôture d'une demande suit la livraison réussie de la réponse,
pas sa réception.** Défaut préexistant confirmé dans le code (contre-revue
cxbridget, objection 7) : `daemon.rs:1221-1231` exécute `mark_answered` avant
disjoncteur, déduplication, quarantaine, budget de sauts, DND et routage — une
réponse ensuite refusée clôt quand même la demande, et l'émetteur ne la reçoit
jamais. Correction (dans le périmètre car le flux ACP automatise les réponses
et amplifierait le défaut) : la transition `answered` est déplacée **après** la
livraison réussie. Décision associée : un message portant `in_reply_to` est une
réponse *sollicitée* — il traverse le refus DND de l'émetteur d'origine (sinon
une demande posée juste avant de passer en DND resterait ouverte et relancerait
un équipier qui a déjà répondu). **Garde anti-forgeage** (round 2, objection
3) : le bypass DND n'est accordé qu'après validation **non mutante** de la
demande référencée — état ouvert, destinataire de la demande = expéditeur de la
réponse, émetteur de la demande = destinataire de la réponse. Un `in_reply_to`
inconnu ou aux participants incohérents est traité comme un message ordinaire
(DND applicable, demande inchangée). Tests dédiés dans les tâches, y compris id
forgé.

**D-207 — Garde clé API dans le wrapper, avant le spawn.** Vérification des
variables refusées de l'entrée de registre au moment du lancement — le seul
endroit où l'environnement du futur processus est connu et modifiable. Refus
motivé en français avec le nom de la variable et le contournement documenté.

## Complexity Tracking

Aucune violation à justifier : zéro dépendance nouvelle, deux fichiers source
nouveaux (`acp.rs`, `registry.rs`), le reste est extension en place. La seule
« duplication » assumée est le client JSON-RPC maison face à une crate
officielle — arbitrée en D-201/R-004 au profit du minimalisme de dépendances.
