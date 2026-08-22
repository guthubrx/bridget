# Data Model : `bridget attach`

Révisé après contre-revue du plan (round 3) : identité d'abonnement, bornes en
octets, écrivain dédié par vue, saisie raw mode, liste blanche Unicode précise.

## Abonnement (état du daemon, en mémoire)

| Champ | Type | Rôle |
|---|---|---|
| `subscription_id` | opaque, unique | identité de génération — portée par tous les messages du plan de contrôle (contrat) |
| `agent` | nom d'annuaire | équipier observé |
| `conn_id` | connexion attach persistante | destinataire du relais **et** canal de l'issue des envois humains |
| `window` | `Today` \| `Seq(n)` \| `Date(j)` | sélecteur typé, résolu par le wrapper |
| `buffer` | file bornée **en octets** (défaut 1 Mio) ; frames = **fragments** `{seq, offset, final}` de 256 Kio max (contrat, fragmentation filaire) | tampon par vue ; en dépassement, lâcher des plus anciens **par `seq` entier** (jamais de fragment orphelin), comptabilisé dans le `Gap` coalescé tenu **hors file** (inlâchable) |
| `gap` | plage `from_seq..to_seq` coalescée | émise avant le prochain événement conservé |

Isolation d'une vue lente (impératif — le motif actuel du daemon écrit sous
verrou global avec flush synchrone, `daemon.rs:1334-1351`, et le reproduire
figerait le daemon) :

- le handler du daemon **ne fait qu'enfiler** dans le tampon de la vue —
  aucune E/S socket sous le verrou d'état global ;
- chaque vue a son **écrivain dédié** (thread) avec délai d'écriture borné ;
  une vue qui ne consomme pas dans le délai est fermée (`End { reason:
  "vue trop lente" }`) ;
- un relais wrapper→daemon bloqué ne bloque **jamais** l'écrivain JSONL du
  wrapper (le suivi de journal est découplé de l'écriture, cf. plan D-302).

Cycle : créé à la souscription, détruit à la déconnexion du client, à la
demande, à la fermeture pour lenteur, ou à la disparition du wrapper (fin typée
avec motif et `subscription_id`).

## Côté wrapper : relais de journal

| Élément | Rôle |
|---|---|
| worker de relais dédié | seul à lire le journal pour le relais ; commandé par un **canal borné** depuis le thread d'écoute (qui ne fait qu'enfiler et retourne — jamais de rejeu dans le thread d'écoute : il bloquerait Deliver, heartbeat et annulations). **Politique de saturation** : `try_send` plein → refus **typé** de la souscription (remonté au client, jamais de perte silencieuse) ; les commandes d'arrêt/désabonnement passent par un canal de contrôle séparé (jamais refusées) — pas d'abonnement fantôme, pas de fuite. Tests : canal plein, arrêt pendant saturation |
| curseur par abonnement | position de rejeu propre à chaque `subscription_id` (deux fenêtres différentes coexistent) |
| lecture par tranches bornées | équité entre rejeu d'un gros historique et suivi live ; le rejeu ne monopolise pas le worker |

## État du client (mémoire seulement, D-303)

| Champ | Rôle |
|---|---|
| `subscription_id` | génération courante ; messages d'autres générations ignorés |
| `last_seq` | dernier `seq` rendu — resynchronisation `Seq(last_seq+1)` ; s'il n'existe pas encore (snapshot vide), la resynchronisation réutilise le **sélecteur initial mémorisé** |
| `initial_window` | sélecteur demandé à la souscription, conservé pour la reprise sans `last_seq` |
| `caught_up` | positionné par `SnapshotCaughtUp` — état affiché et testable |
| `pending_send` | `message_id → texte` des envois en attente d'issue sur la connexion attach (accusé, `Nack`, `DeliveryRejected` différé) ; `message_id` unique par invocation. Une entrée n'est retirée que sur **issue terminale**, **expiration bornée explicite**, ou **fermeture réelle de la connexion** — jamais à `End` (qui termine un abonnement, pas la connexion : un rejet tardif d'un envoi déjà accusé peut arriver pendant un réabonnement et doit rester corrélé ; test imposé : `End` → nouvel abonnement → rejet tardif de l'ancien `message_id`) |
| `input` | tampon de saisie détenu par le client (raw mode) |

**Modèle de concurrence de la connexion attach** (elle multiplexe flux,
contrôle et issues d'envoi) : un **lecteur socket unique** qui dispatch par
`subscription_id` (flux/contrôle) et `message_id` (issues d'envoi), un
**writer sérialisé** — le motif D-204 de la 007, réappliqué. Test imposé :
événements de flux intercalés avec « accusé puis rejet tardif ».

Aucune écriture disque côté client.

## Saisie et terminal (raw mode minimal)

La saisie canonique (`read_line`) ne permet ni de préserver une ligne partielle
pendant les événements, ni de neutraliser l'écho de caractères de contrôle :
le client passe le terminal en **raw mode minimal** via `termios` (`libc`,
déjà dans le workspace — pas de crate TUI) :

- tampon de saisie détenu par `attach` ; à chaque événement live, la ligne en
  cours est réaffichée sous l'événement ;
- **sémantique des signaux fixée** (round 4) : `ISIG` **désactivé** — avec
  `ISIG` actif, SIGINT terminerait le processus sans exécuter les destructeurs
  Rust et la garde RAII serait illusoire. Ctrl-C arrive donc comme l'octet
  `0x03`, traité dans la boucle de saisie : restauration du terminal **puis**
  sortie. La garde RAII reste le filet pour les sorties par erreur/panique ;
- tests imposés : pseudo-TTY avec Ctrl-C, EOF, erreur ; et stdin **non-TTY**
  (pipe) → pas de raw mode, mode dégradé lecture ligne sans saisie interactive ;
- édition minimale : caractères imprimables, retour arrière, Entrée, Ctrl-C /
  Ctrl-D. Rien d'autre (pas d'historique, pas de curseur libre).

## Rendu (liste blanche précise)

- **Scalaires admis** depuis les données : catégories Unicode imprimables hors
  `Cc`/`Cf` (contrôles **et** caractères de format : bidi, zero-width…), plus
  `\n` et `\t`. Tout le reste est rendu visible (`\x1b` → `␛`, C1 → `␡`-style,
  zero-width → `·`).
- Les sauts de ligne **dans** un contenu ne peuvent pas forger un faux préfixe
  d'événement : chaque ligne de continuation est indentée sous le préfixe de
  son événement.
- Longueur rendue par événement bornée (le journal reste intègre ; la vue
  tronque en le signalant).
- Fixtures hostiles imposées : ESC/CSI/OSC, contrôles C1, bidi (RLO/LRO),
  zero-width, CR isolé, backspace, newline forgée, combining excessifs.
- Préfixe par événement : heure locale courte + expéditeur ou type
  (`10:42 codex-1 →`, `10:42 [outil] rg`, `10:43 [erreur]`).
