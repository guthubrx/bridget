# Implementation Plan : `bridget attach` — voir et piloter un équipier

**Branch**: `session-08-attach` (depuis `session-07-transport-acp`) | **Date**: 2026-08-22 | **Spec**: [spec.md](./spec.md)
**Input**: spec contre-revue (BLOCKED round 1 → 8 objections intégrées ; APPROVE_WITH_CHANGES round 2 → 4 résidus intégrés, dont deux durcissements du contrat de journal poussés dans la 007 avant T706)

## Summary

Une sous-commande `bridget attach <nom>` qui projette le flux d'un équipier
headless dans un terminal : rejeu de l'historique depuis le journal v1 (curseur
`seq`), suivi continu, envoi de messages humains par le chemin existant. Un
seul mécanisme pour local et distant : **abonnement médié par le daemon** — le
wrapper est l'unique lecteur de son propre journal, le daemon multiplexe vers
les vues. Aucune dépendance nouvelle, aucun canal réseau nouveau.

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget`
**Primary Dependencies**: aucune nouvelle (rendu ANSI maison, sanitisation
maison — cf. D-304)
**Storage**: aucun nouveau — lecture du journal v1 (007) ; aucun état client
persistant (FR-004 : reprise inter-invocations exclue)
**Testing**: `cargo test` ; fixtures de compatibilité lecteur livrées par
007-T706 (dont lignes partielles/corrompues) ; fixtures hostiles ANSI/OSC ;
faux adaptateur déterministe pour SC-005 (réutilisé de 007-T704 s'il existe)
**Target Platform**: macOS et Linux
**Project Type**: extension CLI + daemon + wrapper existants
**Constraints**: lecteur pur sur le plan d'observation (FR-003) ; SC-001
p95 < 1 s local à 10 evt/s ; zéro contre-pression sur l'écrivain du journal
**Scale/Scope**: 1 module client nouveau, ~500-800 lignes, 8 tâches prévues

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | messages, bandeaux et erreurs en français |
| VII — ADR | ✅ | ADR 004 « abonnement médié par le daemon » à créer en première tâche (choix structurant : un mécanisme unique local/distant) |
| IX — recherche | ✅ | `research.md` R-101..R-105 (motifs internes réutilisés, aucun composant externe nouveau) |
| XVIII — complexité | ✅ | fan-out daemon = itération sur la liste des vues abonnées (petit n) ; lecture journal incrémentale par offset O(nouveaux octets) ; aucune boucle imbriquée sur collections |
| XIX — minimalisme | ✅ | zéro dépendance, zéro état persistant client, réutilisation : journal v1, chemin `Send`, canal wrapper↔daemon, motif de suivi de fichier de la sonde runtime |
| XX — responsabilité | ✅ | chaque décision D-3xx porte justification et limite ; les compromis (fenêtre de rejeu, sanitisation par liste blanche) sont écrits |

**Point de vigilance** : le daemon devient relais de flux. Un client attach
lent ne doit jamais ralentir ni le wrapper ni le daemon — d'où le tampon borné
par vue avec lâcher signalé (D-305), le pendant côté relais du « lecteur pur ».

## Project Structure

```text
specs/008-attach/
├── plan.md              # ce fichier
├── research.md          # R-101..R-105
├── data-model.md        # abonnement, événement de flux, bornes des tampons
├── contracts/
│   └── abonnement-attach.md   # protocole subscribe/snapshot/follow/end
├── quickstart.md        # scénarios de validation manuelle
└── tasks.md             # après reuse-audit (dépend des variantes livrées en 007-T705/T706)

crates/bridget-daemon/src/
├── attach.rs            # NOUVEAU : client de vue (rejeu, suivi, rendu, saisie)
├── cli.rs               # sous-commande attach
├── daemon.rs            # table des abonnements + fan-out
└── wrapper.rs           # suivi de son journal (motif RuntimeProbe) + push

crates/bridget-transport/src/
└── protocol.rs          # variantes d'abonnement (noms au reuse-audit)

docs/decisions/004-abonnement-attach.md   # NOUVEAU : ADR
```

**Structure Decision** : `attach.rs` vit dans `bridget-daemon` (c'est un client
du daemon, comme les sous-commandes CLI existantes) ; aucun crate nouveau.

## Approche par exigence

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001 | sous-commande `attach <nom>` ; résolution par l'annuaire, refus motivé (non-ACP → pointer le pane tmux ; inconnu → lister les équipiers) | `cli.rs`, `attach.rs` |
| FR-002 | rejeu depuis `from_seq` puis suivi ; rendu par type de payload (schéma v1) | `attach.rs` |
| FR-003 | le client ne touche jamais le fichier : tout passe par l'abonnement ; le wrapper lit son journal en incrémental (offset+mtime, période 300 ms, motif sonde runtime) depuis son thread d'écoute — jamais le worker de tours | `wrapper.rs` |
| FR-004 | sélecteur typé `Today`/`Seq`/`Date` résolu par le wrapper ; `SnapshotCaughtUp { through_seq }` rend le rattrapage observable ; jonction prouvable par `seq` | `attach.rs`, contrat |
| FR-005 | saisie → `Send` sur la **connexion attach persistante** (D-308), issue corrélée par `message_id` (accusé, `Nack`, `DeliveryRejected` différé) affichée ; la réponse de l'équipier arrive naturellement par le flux d'événements | `attach.rs`, `daemon.rs` |
| FR-006 | fan-out daemon : chaque événement poussé par le wrapper est relayé à toutes les vues abonnées de l'agent | `daemon.rs` |
| FR-007 | abonnement identique local/distant (le wrapper distant est déjà connecté au daemon par le tunnel 002) ; wrapper absent → message d'indisponibilité défini par la spec | `daemon.rs`, `wrapper.rs` |
| FR-008 | fin d'abonnement typée avec motif (wrapper parti, équipier arrêté) ; resynchronisation après reconnexion : le client se réabonne `Seq(last_seq + 1)` (`Seq` étant inclusif — aligné sur le contrat) | contrat, `attach.rs` |
| FR-009 | rendu texte ANSI minimal maison ; saisie en raw mode minimal termios (D-307) — pas de crate TUI | `attach.rs` |
| FR-010 | liste blanche Unicode précise (hors `Cc`/`Cf`, data-model) ; lignes de continuation indentées (pas de faux préfixes) ; longueur rendue bornée ; garde RAII de restauration ; fixtures hostiles étendues (C1, bidi, zero-width, CR, backspace, combining) | `attach.rs` |

## Décisions de conception

**D-301 — Un seul mécanisme : abonnement médié par le daemon.** Le client ne
lit jamais le fichier, même en local. Justification : un seul code local =
distant, fan-out multi-vues gratuit, « lecteur pur » garanti par construction
(le seul lecteur du fichier est son écrivain). Coût assumé : les événements
transitent par le daemon même en local — volume faible (texte), tampon borné.
Le protocole (détail au contrat) : souscription avec **sélecteur typé de
fenêtre** (`Today`/`Seq`/`Date`, résolu par le wrapper — le client ne peut pas
convertir une date en position puisqu'il ne lit pas le fichier), réponse
portant un **`subscription_id` opaque** présent ensuite sur tout le plan de
contrôle (rejeu, `SnapshotCaughtUp`, `Gap`, `End`) — sans lui, deux vues aux
fenêtres différentes seraient indiscernables et une fin d'ancienne génération
pourrait fermer la nouvelle. Chaque événement = la ligne JSONL v1 telle quelle.
Noms de variantes fixés au reuse-audit, en cohérence avec les variantes 007
livrées.

**D-302 — Le wrapper pousse, il n'est jamais interrogé — via un worker de
relais dédié.** Le wrapper suit son propre journal par offset (il en est
l'écrivain : détection immédiate, pas de course) et pousse au daemon tant
qu'un abonnement est actif. Le relais vit dans un **worker dédié**, commandé
par un canal borné : le thread d'écoute ne fait qu'enfiler la commande et
retourne (rejouer un gros historique depuis le thread d'écoute bloquerait
`Deliver`, le heartbeat et les annulations). Le worker tient un **curseur par
abonnement** et lit par **tranches bornées** avec équité rejeu/suivi. Le suivi
de journal est découplé de l'écrivain JSONL : un relais bloqué ne retarde
jamais une écriture de journal. Zéro trafic sans abonné.

**D-303 — Aucun état client persistant.** La reprise inter-invocations est
exclue par la spec ; le client garde son dernier `seq` en mémoire pour la
resynchronisation de la même invocation, rien sur disque.

**D-304 — Sanitisation par liste blanche, pas par liste noire.** Tout octet de
contrôle issu des *données* est neutralisé sauf `\n` et `\t` (affichés comme
tels) ; les séquences d'échappement sont échappées visiblement (`\x1b` rendu
`␛`). Seul le *renderer* émet des séquences ANSI (couleurs de base). Une liste
noire d'OSC/CSI serait incomplète par construction ; la liste blanche est
courte et prouvable par fixtures hostiles (SC-007).

**D-305 — Isolation réelle d'une vue lente : bornes en octets, écrivain dédié,
jamais d'E/S sous le verrou global.** Une borne en *événements* ne bornerait
rien (le corps est complet, une ligne peut être arbitrairement grosse), et le
motif d'écriture actuel du daemon — E/S sous verrou global avec flush synchrone
(`daemon.rs:1334-1351`) — permettrait à une vue lente de figer le daemon
entier. Règles : tampon par vue borné **en octets** (défaut 1 Mio) ; les événements
volumineux voyagent **fragmentés** (`{seq, offset, final}`, 256 Kio max par
fragment — la ligne v1 « telle quelle » n'est pas bornée, la frame si ; un
fragment lâché abandonne le `seq` entier au `Gap`, jamais de boucle de rejeu
sur le même événement) ; le handler du daemon **ne fait qu'enfiler** ; un
**écrivain dédié par vue** écrit avec délai borné et ferme la vue trop lente
(`End` motivé) ; le lâcher alimente un **`Gap` coalescé tenu hors file**
(inlâchable), émis avant le prochain événement conservé — jamais de trou
silencieux. C'est la traduction relais de « intégrité du journal ≠ fidélité du
rendu ».

**D-307 — Saisie en raw mode minimal (termios/libc), pas de TUI.** La lecture
canonique `read_line` rend FR-010 irréalisable : l'écho du terminal affiche les
contrôles avant toute neutralisation, et la ligne partielle est inconnue de
l'application au moment de redessiner. Le client passe donc en raw mode
minimal via `termios` (`libc` est déjà une dépendance du workspace — zéro
crate nouvelle) : tampon de saisie détenu par `attach`, réaffichage sous
chaque événement. **Sémantique des signaux fixée** : `ISIG` désactivé (avec
`ISIG` actif, SIGINT tuerait le processus sans exécuter les destructeurs — la
garde RAII serait décorative) ; Ctrl-C = octet `0x03` traité dans la boucle,
restauration puis sortie ; la garde RAII reste le filet des sorties par
erreur. stdin non-TTY → mode dégradé sans raw mode. Tests pseudo-TTY imposés
(Ctrl-C, EOF, erreur, non-TTY). Édition volontairement pauvre (imprimables,
retour arrière, Entrée) — c'est un champ de saisie, pas un éditeur. Révision
assumée de la recherche R-104 qui excluait le raw mode.

**D-308 — L'envoi humain passe par la connexion attach, pas par cli-send.**
Une connexion `cli-send` éphémère se ferme après l'accusé (`daemon.rs:
1203-1207`) et ne peut donc jamais recevoir un `DeliveryRejected` différé. Le
`Send` part sur la **connexion attach persistante**, `message_id` conservé, et
l'issue (accusé, refus immédiat, rejet différé) revient corrélée sur cette même
connexion. La connexion multiplexe flux, contrôle et issues : **lecteur socket
unique avec dispatch** (`subscription_id` pour le flux, `message_id` pour les
issues) et **writer sérialisé** — le motif D-204 de la 007 réappliqué côté
client. `message_id` unique par invocation ; une entrée `pending_send` ne se
retire que sur issue terminale, expiration bornée, ou fermeture réelle de la
connexion — **jamais à `End`** (un rejet tardif peut arriver pendant un
réabonnement et doit rester corrélé). Chemin unique, cas « accusé puis rejet
tardif » testé avec événements de flux intercalés, y compris après `End`.

**D-306 — SC-005 outillé par un faux adaptateur déterministe.** La mesure de
non-perturbation (p95 latence d'append, N ≥ 200) s'appuie sur l'adaptateur de
test de 007-T704 (fixtures stdio) rejouant des tours identiques — pas de
modèle réel dans la boucle de mesure.

## Complexity Tracking

Aucune violation : zéro dépendance nouvelle, un fichier source nouveau
(`attach.rs`), variantes de protocole en extension du motif D-209 existant. Le
choix « tout passe par le daemon même en local » ajoute un saut de plus qu'une
lecture directe du fichier — justifié par l'unification local/distant et la
garantie de lecteur pur (Article XIX §5 : lisibilité et sécurité d'un chemin
unique contre micro-optimisation d'un chemin double).
