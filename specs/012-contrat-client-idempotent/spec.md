# Feature Specification : Contrat client idempotent — envois durables à `message_id` choisi

**Feature Branch**: `session-12-contrat-client-idempotent` (insertion dans la
feuille de route **soumise à validation utilisateur** — spec préparée sur
mandat de l'agent `prospective`, session 011 Maicie)
**Created**: 2026-08-22
**Status**: Draft — en attente de contre-revue adverse
**Input**: exigences amont de `prospective` (011) consignées dans
`exigences-amont.md` : un client externe doit pouvoir déléguer puis survivre à
un crash ou à la perte d'un accusé **sans jamais émettre deux fois la même
délégation**, en ne dépendant que d'un contrat public local.

## Contexte et problème

Un client externe (Maicie 011, ou tout orchestrateur tiers) qui envoie une
demande et perd l'accusé — crash, coupure, timeout — est aujourd'hui devant un
dilemme impossible : réémettre (risque de doublon : l'ID est **imposé par le
daemon** via `BridgetMessage::new`, la déduplication est **en mémoire et
expirante**) ou abandonner (risque de perte silencieuse). Aucun lookup général
d'issue par ID n'existe (`ListRequests` ne couvre que les demandes suivies).
Cette session définit la brique manquante : des **envois idempotents à
`message_id` choisi par le client, aux issues durables et consultables**, au
niveau du **protocole local public** — les adaptateurs (CLI, outil MCP 010) en
deviennent des projections minces à sémantique identique.

## Dépendances et frontières

- **Recouvrements à unifier** (exigence du reuse-audit à venir) : le motif
  « id avant connexion + issue durable + `IdempotencyExpired` » existe déjà
  dans deux conceptions validées — 009 D-503 (`spawn_commands`) et 010 FR-002
  (`bridget_send`). La 012 doit aboutir à **un** mécanisme commun ; la 010 se
  reformulera comme projection de la 012 si elle est retenue.
- **Hors périmètre** : flux ACP (008), réveil de profils (011+), toute logique
  côté client (l'outbox de Maicie reste chez Maicie).
- **Modèle de confiance inchangé** (README) : l'idempotence est un contrat de
  fiabilité entre pairs coopératifs, pas une authentification.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Réémettre sans risque après un accusé perdu (Priority: P1)

Un client envoie une demande avec un `message_id` qu'il a généré et persisté.
L'accusé se perd (crash client, coupure). Au redémarrage, le client réémet le
**même** `message_id` avec la **même** enveloppe : le daemon ne livre pas une
seconde fois — il **rejoue l'issue** de l'envoi initial (accusé, refus motivé,
ou `OutcomeUnknown` si l'envoi initial est encore en vol).

**Why this priority** : c'est le cœur du besoin — l'outbox testable.

**Independent Test** : envoyer avec ID choisi, tuer le client avant lecture de
l'accusé, réémettre le même ID → une seule livraison chez le destinataire,
issue identique rejouée ; vérifiable au ledger.

**Acceptance Scenarios**:

1. **Given** un envoi accepté dont l'accusé s'est perdu, **When** le client
   réémet le même `message_id` à enveloppe identique, **Then** aucune seconde
   livraison n'a lieu et l'issue `Accepted` initiale est rejouée (mêmes id,
   données).
2. **Given** un envoi refusé (DND, disjoncteur…), **When** réémission du même
   ID, **Then** le refus initial est rejoué à l'identique — le retry ne
   « retente » pas l'envoi.
3. **Given** le daemon a redémarré entre l'envoi et le retry, **Then** le
   comportement des scénarios 1-2 est inchangé (durabilité).
4. **Given** un même `message_id` réémis avec une **enveloppe divergente**
   (corps, cible, `reply`… — comparaison canonique), **Then** refus typé
   `EnvelopeMismatch`, l'issue initiale restant intacte.

---

### User Story 2 - Consulter l'issue d'un envoi par son ID (Priority: P2)

Le client interroge le daemon avec un `message_id` : il obtient l'un des
quatre états fermés — `Accepted` (avec les données d'origine), refus terminal
(catégorie + motif), `OutcomeUnknown` (en vol — **pas** une autorisation de
rejouer aveuglément), ou `IdempotencyExpired` (horizon dépassé). L'absence
d'issue n'est jamais ambiguë.

**Why this priority** : la moitié « lecture » du contrat — sans elle, le
client ne peut pas réconcilier son outbox après crash.

**Independent Test** : lookup après chaque famille d'issue + lookup d'un ID
inconnu → `IdempotencyExpired` ; lookup pendant qu'un envoi est en vol →
`OutcomeUnknown`.

**Acceptance Scenarios**:

1. **Given** un envoi accepté puis un redémarrage du daemon, **When** lookup,
   **Then** `Accepted` avec les données d'origine.
2. **Given** un envoi en cours de traitement, **When** lookup, **Then**
   `OutcomeUnknown` — et le contrat dit explicitement que ce n'est pas une
   permission de réémettre en aveugle.
3. **Given** un ID au-delà de l'horizon de rétention (ou jamais vu), **Then**
   `IdempotencyExpired`.

---

### User Story 3 - Négociation de capacités et versionnement (Priority: P3)

À la connexion, un client public négocie : version du contrat, **horizon de
rétention des issues** (déclaré par le daemon — c'est lui que l'outbox utilise
pour `retry_until`), et capacités (envoi idempotent, lookup). Un client d'une
version inconnue reçoit un refus motivé, jamais un comportement silencieusement
dégradé.

**Why this priority** : c'est ce qui rend le contrat *public* — un tiers peut
cibler une version, pas une implémentation.

**Independent Test** : négociation nominale (horizon reçu et cohérent avec le
comportement observé) ; version future → refus motivé.

**Acceptance Scenarios**:

1. **Given** un client qui négocie, **Then** il reçoit version + horizon +
   capacités, et l'horizon annoncé correspond au comportement réel (une issue
   est consultable jusqu'à l'horizon, `IdempotencyExpired` après).
2. **Given** une version de contrat non supportée, **Then** refus motivé
   nommant les versions supportées.

---

### Edge Cases

- **Deux réémissions concurrentes du même ID** : une seule livraison, les deux
  reçoivent la même issue (pas de course observable).
- **Retry pendant que l'envoi initial est en vol** : `OutcomeUnknown` rejoué
  jusqu'au terminal — jamais une deuxième livraison, jamais un blocage.
- **Enveloppe divergente subtile** (même corps, `reply` différent ; champ
  daemonisé non normalisé) : la comparaison canonique publie la liste exacte
  des champs couverts et des normalisations — tout écart → `EnvelopeMismatch`.
- **Purge de rétention pendant un retry** : l'`expires_at` **persisté par
  opération** fait foi (jamais la configuration courante) ; à cheval sur la
  frontière, le comportement est déterministe et testé (avant / à / après) ;
  jamais de réémission silencieuse déclenchée par l'expiration, et un premier
  `Send` à `issued_at` déjà hors horizon est reconnu (`IdempotencyExpired`,
  FR-008).
- **Collisions d'ID** (entre clients, ou avec la voie historique) : l'espace
  de clé (`issuer_scope`, `operation_kind`, `message_id`) isole les portées —
  deux clients choisissant le même `message_id` ne se voient pas, et le
  lookup est borné à la portée de l'appelant. Garantie d'**isolation
  accidentelle** entre pairs coopératifs — pas une protection contre un pair
  local malveillant (modèle de confiance documenté).
- **Volumétrie** : la table d'issues est bornée par la rétention ; un client
  qui émet en masse ne peut pas la faire croître sans limite au-delà de
  l'horizon (purge testée).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** : le protocole local public DOIT accepter un envoi portant un
  `message_id` **choisi par le client**, avec la même sémantique de routage et
  de garde-fous que la voie historique — l'idempotence s'ajoute, elle ne
  remplace rien. **Ordre des gardes fixé** (round 1) : après résolution de
  l'`issuer_scope`, la clé est réservée/recherchée et le canon comparé
  **avant toute garde mutable** (DND, dedup contenu, résolution de cible…) —
  une issue connue est **rejouée sans reroutage** ; seules les clés nouvelles
  traversent les garde-fous.
- **FR-002** : chaque envoi idempotent suit une **machine d'états interne
  durable** `Prepared → Dispatching → Terminal(Accepted | Rejected)`, à
  transitions **monotones**, survivant au redémarrage, avec chaque frontière
  de crash définie et testée. `Accepted` n'est atteint qu'après une **remise
  aval elle-même idempotente par `message_id`, accusée par le wrapper
  destinataire** — c'est le point de linéarisation qui fonde « une seule
  livraison » ; la convergence d'un envoi `reply=false` est assurée par cet
  accusé de remise (pas par une réponse applicative). Le **résultat de
  lookup** est un modèle distinct calculé : `Accepted` \| `Rejected`
  (catégorie fermée + motif) \| `OutcomeUnknown` (état interne non terminal)
  \| `IdempotencyExpired` (synthétisé hors horizon).
- **FR-003** : la réémission d'un `message_id` connu à enveloppe
  **canoniquement identique** DOIT rejouer l'issue sans nouvel effet (zéro
  seconde livraison) ; à enveloppe **divergente** → refus typé
  `EnvelopeMismatch` sans altérer l'issue initiale.
- **FR-004** : la **comparaison canonique** couvre : `message_id`,
  `issuer_scope` (FR-010), cible, `body_bytes` exacts, `reply`, l'**échéance
  Unix absolue** (le délai relatif est normalisé **une seule fois** à la
  première réception, persisté avant tout effet, comparé au retry **sans
  recalcul**), `hops` (valeur cliente **initiale**, avant décrément daemon),
  `in_reply_to`, et `issued_at` (FR-008) ; les champs ajoutés par le daemon
  sont exclus **seulement s'ils sont explicitement normalisés avant le
  hachage** ; la liste des champs couverts est publiée au contrat.
- **FR-005** : un **lookup par (`issuer_scope`, `message_id`)** DOIT retourner
  l'un des quatre résultats fermés ; `OutcomeUnknown` est documenté comme « en
  vol — pas une autorisation de rejouer » ; ID inconnu ou hors horizon →
  `IdempotencyExpired`. Le lookup est **implicitement borné à la portée** de
  l'appelant.
- **FR-006** : la **négociation de capacités** DOIT précéder l'usage :
  version du contrat, horizon de rétention, capacités ; version inconnue →
  refus motivé. **L'horizon devient contractuel par opération** : un
  `expires_at` est calculé au premier `Send` selon l'horizon négocié,
  **persisté**, retourné dans `Accepted` et au lookup — **aucune purge avant
  `expires_at`**, même si la configuration courante a diminué entre-temps.
- **FR-007** : les adaptateurs existants et futurs (binaire CLI, outil MCP
  010) DOIVENT projeter ce contrat **sans en modifier la sémantique** — un
  même (`issuer_scope`, `message_id`) traverse indifféremment les voies avec
  le même résultat ; la voie historique (ID généré par le daemon) reste
  inchangée pour les clients qui ne négocient pas.
- **FR-008** : l'expiration est **passive et reconnaissable** :
  l'enveloppe porte un **`issued_at` immuable, inclus au canon** — un premier
  `Send` dont `issued_at` est déjà hors horizon répond `IdempotencyExpired`
  (jamais une création) : un ID ancien purgé est ainsi **discernable** d'un ID
  neuf. **Validation d'`issued_at` fixée** : le référentiel temporel est
  l'horloge du daemon ; une tolérance future maximale est déclarée à la
  négociation (défaut au plan) — au-delà, refus typé `InvalidIssuedAt` ;
  **`expires_at = issued_at + horizon contractuel`**, sans autre formule.
  Tests : `issued_at` futur au-delà de la tolérance, dérive admise dans la
  tolérance, et frontières exactes de purge (juste avant, à, juste après).
- **FR-009** : aucune dépendance nouvelle ; durabilité par le store existant.
  L'unification avec 009/010 porte sur le **socle transactionnel seul** —
  réservation de clé, hachage canonique, rejeu, `EnvelopeMismatch`,
  expiration — dont la clé abstraite est nommée **`idempotency_key`**,
  projetée en `message_id` pour `Send` et en `command_id` pour `SpawnOrder`,
  paramétrée par `operation_kind` ; les enregistrements du socle sont
  **durables pour toutes les projections** (la portée « éphémère à la vie du
  daemon » des spawns non persistants de la 009 devra être ré-exprimée comme
  une rétention courte du socle, amendement documenté) ; `spawn_commands`
  (009) et la table de remise (012) restent des **consommateurs spécialisés**
  (générations, fleet et reprise restent propres à la 009). Ordre
  d'implémentation recommandé si la 012 est retenue : **le socle avant 009 et
  010**, qui en deviennent consommateurs — jamais deux mécanismes transitoires.
- **FR-010** : l'identité d'idempotence est un **`issuer_scope` stable et
  durable, distinct du nom affiché** : il survit au crash du client, est
  partagé explicitement entre les projections (CLI, MCP), et indexe les
  issues avec le `message_id` ; le nom résolu au moment de l'envoi n'est
  qu'un **instantané de traçabilité** (ledger), jamais une composante du
  hash — un `rename` entre émission et retry ne change rien, et un nom
  réattribué n'hérite d'aucune issue. Espace de clé complet :
  (`issuer_scope`, `operation_kind`, `message_id`). Garantie reformulée :
  **isolation accidentelle** entre clients coopératifs — pas une protection
  contre un pair local malveillant (modèle de confiance inchangé).

### Key Entities

- **Envoi idempotent** : envoi dont l'ID appartient au client ; enveloppe
  canonique hachée.
- **Enregistrement interne durable** : état `Prepared` \| `Dispatching` \|
  `Terminal(Accepted | Rejected)`, transitions monotones, `expires_at`
  propre — l'entité stockée.
- **Résultat de lookup** : valeur **calculée** — `Accepted` \| `Rejected`
  (catégorie + motif) \| `OutcomeUnknown` (interne non terminal) \|
  `IdempotencyExpired` (synthétisé hors horizon) — jamais stockée telle
  quelle.
- **Négociation** : version, horizon de rétention, capacités — le contrat
  public que Maicie et tout tiers ciblent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001** : sur N = 50 cycles « envoi → perte d'accusé simulée → retry même
  ID », le destinataire reçoit exactement 50 livraisons (zéro doublon), et
  chaque retry rejoue l'issue initiale à l'identique. La répartition des
  perturbations est fixée par une **matrice versionnée de points de crash** :
  avant réservation, après `Prepared`, après remise aval avant issue, après
  issue avant accusé client — avec redémarrage du daemon sur un sous-ensemble
  explicite de chaque point.
- **SC-002** : les quatre états du lookup sont observés chacun par un test
  dédié, plus le cas « retry en vol → `OutcomeUnknown` rejoué puis terminal ».
- **SC-003** : 100 % d'un corpus de divergences d'enveloppe (corps modifié
  d'un octet, cible changée, `reply` inversé, champ daemonisé non normalisé)
  produit `EnvelopeMismatch` sans altérer l'issue initiale.
- **SC-004** : scindé en deux gates (la 010 n'existant pas encore, elle ne
  doit pas devenir une dépendance bloquante) : **gate 012** — le même
  (`issuer_scope`, `message_id`) envoyé via un **client socket de référence**
  puis retenté via le **binaire CLI** (et inversement) rejoue la même issue ;
  **gate de conformité 010** (portée par la 010 à son implémentation) — même
  preuve avec l'outil MCP.
- **SC-005** : après purge, la table ne contient aucun enregistrement **au-delà
  de son `expires_at` propre** (jamais « de l'horizon courant » : une baisse
  de configuration conserve les opérations anciennes jusqu'à leur échéance
  promise — testé), et un lookup post-purge répond `IdempotencyExpired`
  (jamais un rejeu partiel).
- **SC-006** : un client de version inconnue est refusé avec motif ; un client
  sans négociation conserve exactement le comportement historique (suite 007
  au vert, non-régression).

## Assumptions

- Le modèle de confiance est inchangé : c'est l'**`issuer_scope` stable** qui
  entre au canon ; le nom courant résolu reste **hors canon** (instantané de
  traçabilité) — et rien de tout cela n'est une authentification.
- L'horizon de rétention par défaut est aligné sur celui des demandes suivies
  (même politique de purge) ; sa valeur exacte est un choix du plan, déclaré à
  la négociation.
- L'implémentation n'est **pas** engagée par cette spec : l'insertion en
  feuille de route (avant/après la 010) est une décision utilisateur ; la
  contrainte d'unification FR-009 garantit qu'aucun travail n'est perdu quel
  que soit l'ordre.
