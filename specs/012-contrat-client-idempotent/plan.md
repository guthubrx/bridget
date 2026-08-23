# Implementation Plan : Contrat client idempotent

**Branch**: `session-12-contrat-client-idempotent` (insertion en feuille de
route soumise à validation utilisateur ; ordre recommandé en revue : socle
avant 009/010) | **Date**: 2026-08-22 | **Spec**: [spec.md](./spec.md)
**Input**: spec contre-revue (2 rounds, 16 objections intégrées, passage au
plan autorisé) + réserve de plan consignée dans `exigences-amont.md`

## Summary

Un **socle transactionnel d'idempotence** dans le daemon (`idempotency_key`
abstraite, réservation, hachage canonique, rejeu, `EnvelopeMismatch`,
expiration par `expires_at` propre), consommé par une **remise aval accusée**
pour les envois à `message_id` client — et, si l'utilisateur retient l'ordre
recommandé, par les ordres 009 (`command_id`) et l'outil MCP 010 comme
projections. Négociation de capacités versionnée ; client socket de référence
comme première projection testable.

## Technical Context

**Language/Version**: Rust 2021, workspace `bridget`
**Primary Dependencies**: aucune nouvelle (`rusqlite`, `serde_json`, `sha2` ?
— non : hachage par les primitives déjà présentes, cf. D-605)
**Storage**: tables SQLite du store existant : `idempotency_records` (socle) +
`send_deliveries` (consommateur Send)
**Testing**: `cargo test` ; matrice versionnée de points de crash (SC-001) ;
tests à barrières pour la remise accusée ; client socket de référence
**Target Platform**: macOS et Linux
**Project Type**: extension daemon/protocole/CLI existants
**Constraints**: réservation + canon **avant** toute garde mutable ; zéro
régression de la voie historique (clients non négociés)
**Scale/Scope**: 1 module socle + 1 module store de reçus + rôle `client` +
variantes de protocole, ~900-1200 lignes, 11-12 tâches prévues (aligné sur le
Complexity Tracking)

## Constitution Check

| Article | Statut | Preuve |
|---|---|---|
| I — français | ✅ | messages, refus typés et erreurs en français |
| VII — ADR | ✅ | ADR 007 « socle d'idempotence unique » en première tâche (décision inter-sessions : 009/010 consommateurs) |
| XVIII — complexité | ✅ | réservation = un `INSERT OR` conditionnel indexé ; lookup = un `SELECT` par clé ; purge bornée par index sur `expires_at` |
| XIX — minimalisme | ✅ | un seul mécanisme d'idempotence pour trois usages (la définition même de l'abstraction à ≥3 usages réels) ; zéro dépendance ; store réutilisé |
| XX — responsabilité | ✅ | chaque invariant de la revue (linéarisation, discernabilité post-purge, portée stable) a sa décision et ses tests de crash nommés |

**Point de vigilance** : le socle touche le chemin d'envoi le plus emprunté du
daemon. La voie historique (ID daemon, clients actuels) ne doit **jamais**
traverser le socle — brancher l'idempotence uniquement derrière la négociation
explicite, prouvé par la non-régression complète 007.

## Project Structure

```text
specs/012-contrat-client-idempotent/
├── spec.md, exigences-amont.md, adversarial-review-cxbridget.md, plan.md
├── data-model.md        # tables, canon, machine d'états, table de vérité recovery
├── contracts/
│   └── protocole-idempotent.md   # négociation, Send idempotent, lookup, accusé aval
├── quickstart.md
└── tasks.md             # après reuse-audit (base : branche courante de la feuille de route)

crates/bridget-daemon/src/
├── idempotency.rs       # NOUVEAU : socle (réservation, canon, rejeu, expiration)
├── daemon.rs            # négociation, ordre des gardes, remise accusée
├── store.rs             # tables idempotency_records + send_deliveries
└── cli.rs               # projection CLI (--id / --issued-at) + client de référence en tests

crates/bridget-transport/src/protocol.rs   # variantes : rôle client, ClientHello/Welcome, SendIdempotent, lookup, DeliverAcked/DeliveryIndeterminate
crates/bridget-daemon/src/receipt_store.rs # NOUVEAU : store de reçus deux-phases Seen/Acked (D-602/D-608 — module dédié : corruption/quota/compaction testables hors de la boucle wrapper)
crates/bridget-daemon/src/wrapper.rs       # branchement du receipt_store dans la boucle de livraison

docs/decisions/007-socle-idempotence.md    # NOUVEAU : ADR
```

## Approche par exigence (résumé)

| Exigence | Mise en œuvre | Fichier |
|---|---|---|
| FR-001 ordre des gardes | pré-dispatch : résolution `issuer_scope` → réservation/lookup+canon → gardes mutables seulement pour les clés nouvelles | `daemon.rs` |
| FR-002 machine durable + remise accusée | `Prepared→Dispatching→Terminal` en SQLite transactionnel ; `Deliver` idempotent aval + **accusé de remise du wrapper** (D-602) avant `Accepted` | `idempotency.rs`, `protocol.rs`, `wrapper.rs` |
| FR-003/004 canon | hachage de l'enveloppe canonique (champs publiés, échéance absolue normalisée une fois, `hops` initial, `issued_at`) | `idempotency.rs`, contrat |
| FR-005 lookup | `SELECT` par (`issuer_scope`, `operation_kind`, `idempotency_key`) → résultat calculé | `idempotency.rs` |
| FR-006 négociation | variantes `ClientHello`/`ClientWelcome { version, horizon, tolérance issued_at }` ; `expires_at` persisté par opération | `protocol.rs`, `daemon.rs` |
| FR-007 projections | CLI : `bridget send --id … --issued-at … --issuer-scope …` (**les trois obligatoires ensemble** — le scope est généré/persisté par le client appelant, jamais par le CLI) ; client socket de référence pour les tests ; MCP = gate 010 ; **amendement T006 de la 011 requis avant implémentation** (l'OutboxDélégation actuelle ne possède PAS encore d'issuer_scope — constat factuel, gate déjà consignée dans exigences-amont.md) | `cli.rs` |
| FR-008 expiration | `issued_at` validé (référentiel daemon, tolérance), `InvalidIssuedAt`, purge par `expires_at` propre | `idempotency.rs`, `store.rs` |
| FR-009/010 socle et portées | `idempotency_key` abstraite + `operation_kind` ; `issuer_scope` durable négocié | `idempotency.rs` |

## Décisions de conception

**D-601 — Le socle est un module, ses consommateurs des tables — propriété
des états fixée.** `idempotency_records` est **seul propriétaire** de la clé,
du canon, de l'`expires_at` et du **résultat public** ; la table consommatrice
(`send_deliveries`, plus tard `spawn_commands`) est seule propriétaire de son
**état métier et de sa récupération** — aucune issue dupliquée. Création des
deux lignes **et** finalisation des deux états dans **une même transaction
SQLite avec clé étrangère**. Pour la 009 : **amender explicitement D-503**
avant la génération de ses tâches (consommation du socle) ; et si l'ordre
utilisateur faisait démarrer la 009 avant la 012, une **migration serait
inévitable et assumée** — la promesse « jamais deux mécanismes » ne vaut que
dans l'ordre socle-d'abord.

**D-602 — La remise aval accusée, en deux phases durables (réécrite aux
rounds 3 et 4).** Un fichier ne rend pas atomiques « enregistrer » et
« injecter » — le protocole du wrapper est en deux marques, indexées par une
**clé de remise dédiée** : `send_deliveries` attribue un **`delivery_id`
opaque, unique et durable** (jamais `message_id` seul : la clé publique
autorise le même `message_id` dans plusieurs `issuer_scope` — deux clients
légitimes ne doivent jamais se déduire mutuellement) ; `DeliverIdempotent`,
`Seen`/`Acked`, `DeliverAcked` et `DeliveryIndeterminate` portent tous ce
`delivery_id`. **Champs filaires imposés** (round 5) : `DeliverIdempotent
{ delivery_id, recipient_instance_id, generation, expires_at, message }` —
sans eux le wrapper ne peut ni vérifier le ciblage de génération ni borner la
conservation du reçu ; `DeliverAcked` et `DeliveryIndeterminate` **répètent
`delivery_id` + génération**, et le daemon rejette un accusé tardif d'une
instance obsolète. Séquence : **`Seen` persisté + fsync AVANT l'injection** ;
l'« injection réussie » a un **observable défini** — l'événement interne
**`PromptDispatched`**, émis **uniquement après écriture + flush réussis de
la frame `session/prompt` vers l'adaptateur** (ni le retour de `deliver()`
qui ne fait qu'enfiler, ni `TurnStarted` qui précède l'envoi de la frame,
ne conviennent) ; sur `PromptDispatched` seulement : **`Acked` + fsync puis
`DeliverAcked`**. Avant `PromptDispatched`, un échec/arrêt reste `Rejected`
ou `Seen`→`DeliveryIndeterminate` selon la frontière. Redélivrance d'un
`delivery_id` `Acked` → rejouer `DeliverAcked` ; `Seen` seul →
`DeliveryIndeterminate`, jamais réinjecter, `OutcomeUnknown` maintenu jusqu'à
l'`expires_at`. **Portée honnête** : exactly-once couvre les crashs client et
daemon avec wrapper vivant ; un crash wrapper en fenêtre `Seen` dégrade à
« au-plus-une-injection + issue indéterminée » (limite au contrat). Tests :
le faux adaptateur **barre et compte la lecture effective des frames
`session/prompt`** ; deux scopes / même `message_id` → deux injections
distinctes puis retry de chacun sans injection supplémentaire ; crash daemon
après remise avant issue → redélivrance, un seul prompt lu.

**D-603 — `issuer_scope` : opaque, possédé, borné.** Identifiant **opaque de
128 bits minimum**, généré **une fois** et persisté par le client (l'outbox
de Maicie **devra le persister via l'amendement T006 de la 011** — elle ne le
possède pas encore, gate consignée) ; **la possession vaut identité** — une copie
volontaire ou une mauvaise configuration **fusionne les portées par
contrat** (documenté ; aucune prétention d'authentification). Plusieurs
connexions concurrentes du même scope sont **autorisées** (retries).
Validation de longueur/caractères, **nombre de scopes actifs borné**. Le CLI
reçoit le scope explicitement, MCP le projettera. (Reformulé : « collisions
accidentelles rendues improbables par l'espace de clé », jamais
« impossibles ».)

**D-604 — Négociation ordonnée dans le plan de rôles 008.** La séquence par
connexion est : **sélection/figement du rôle d'abord** (mécanique 008 :
premier message non-handshake = wrapper, handshake attach, et un **nouveau
rôle `client` explicite**) ; `ClientHello { contract_version, issuer_scope,
capabilities }` n'est accepté **que sur un rôle `client`** →
`ClientWelcome { version, horizon_secs, issued_at_tolerance_secs,
capabilities }` (les capacités **négociées** retournées — FR-006) ou refus
motivé. **Matrice de rôles fermée pour toutes les nouvelles variantes** :
`ClientHello`/`SendIdempotent`/`Lookup` = rôle `client` uniquement ;
`DeliverIdempotent` = daemon→wrapper ; `DeliverAcked`/`DeliveryIndeterminate`
= wrapper→daemon ; **aucune** sur une connexion attach (la voie d'envoi
d'attach reste le `Send` humain `reply=false` de la 008). Tests croisés par
variante : attach → `ClientHello` refusé ; wrapper → `DeliverAcked` accepté ;
`Register` historique inchangé. **Règle
unique, sans OU** (round 4) : `RoleHello(Client)` → `RoleAccepted` →
`ClientHello` ; un `ClientHello` reçu avant `RoleAccepted` est **toujours
refusé, sans mutation de rôle ni réservation** — il ne fige jamais rien.

**D-605 — Les octets canoniques SONT l'autorité, pas leur empreinte.** Le
canon est une sérialisation déterministe (ordre de champs fixé, publié au
contrat, **taille maximale bornée**) **stockée telle quelle** ; l'égalité se
prouve par **comparaison exacte des octets** — un digest éventuel
(`DefaultHasher` n'est ni stable inter-versions ni sans collision) ne sert
que de **préfiltre d'index, jamais de preuve** (une collision rejouerait une
issue pour une enveloppe divergente). Le corps/enveloppe n'est stocké
**qu'une seule fois** entre les deux tables (référence croisée). Pas de crate
cryptographique (Article XIX, modèle coopératif).

**D-607 — Variante `SendIdempotent` distincte : la séparation est
structurelle, pas intentionnelle.** Le `Send` historique (`protocol.rs:35`)
reste **inchangé octet pour octet** ; une variante **`SendIdempotent`**
distincte — **ainsi que `Lookup`** (un `Lookup` avant `ClientWelcome` est
refusé **sans lecture ni réservation**, testé) — exige l'état de connexion
`Negotiated(version, issuer_scope)` avant
tout appel au socle. Tests avec sonde sur le store : `Register`+`Send`
historiques créent **zéro** `idempotency_record` ; `SendIdempotent` avant
`ClientWelcome` est refusé **sans rien réserver** ; une trame inconnue ne
bascule jamais silencieusement vers le chemin historique.

**D-608 — Store de reçus dédié, indexé par instance, fail-closed (durci au
round 4).** Jamais le fichier de nom. Store dédié indexé par **identité
stable d'instance de session** (`instance_id`), **jamais par nom affiché**
(un `rename` ou un nouveau wrapper reprenant un nom n'ouvrent jamais un store
vierge exploitable) : **répertoire d'état durable**
`~/.local/state/bridget/receipts/<instance_id>/` — jamais `~/.cache`, un
cache est supprimable et une absence recréée vide autoriserait une
réinjection ; après initialisation, **disparition ou troncature inattendue =
quarantaine fail-closed, jamais recréation vide** — 0700/0600,
écriture atomique + fsync, horizon maximal négociable, quota d'octets/entrées
(dépassement → refus terminal sans effet), purge/compaction atomique.
**Ciblage de génération** : `send_deliveries` persiste
`recipient_instance_id`/génération à la **première résolution** du
destinataire ; toute redélivrance ne vise que cette génération — instance
absente ou remplacée → `DeliveryIndeterminate`, **jamais** de reroutage par
nom. **Corruption : fail-closed** — un store corrompu est **mis en
quarantaine tel quel** (jamais « reconstruit » : la corruption peut avoir
effacé la clé d'un `Acked`, et reconstruire une absence en « nouveau »
autoriserait une réinjection) ; toute livraison de cette génération répond
`DeliveryIndeterminate` jusqu'à une borne sûre (`max_expires_at` conservé
séparément et atomiquement, à défaut `horizon_max` depuis la quarantaine).
Échec de persistance `Seen` avant injection → refus terminal sans effet.
Tests : saturation ; **corruption ciblée d'un `Acked` → redélivrance →
compteur de prompts inchangé** ; redémarrage ; `rename` avant redélivrance ;
nouveau wrapper sur le même nom.

**D-606 — Client socket de référence.** Un petit client de test (code de
test, pas un binaire livré) parle le protocole idempotent nu — c'est lui qui
prouve la gate 012 de SC-004 (croisement avec le CLI) sans attendre la 010.

## Complexity Tracking

Aucune violation : zéro dépendance. Dimensionnement honnête (corrigé au
round 4) : un module socle, **deux tables daemon, un store durable de reçus
côté wrapper (D-608), un nouveau rôle de connexion `client` et les variantes
de protocole associées** (`RoleHello(Client)`, `ClientHello`/`Welcome`,
`SendIdempotent`, lookup, `DeliverIdempotent`/`DeliverAcked`/
`DeliveryIndeterminate`) — ~900-1200 lignes, 11-12 tâches prévues. Le choix
« accusé de remise durable côté wrapper » ajoute un état wrapper — justifié
par l'invariant central (une seule injection par `delivery_id`, y compris à
travers un crash daemon), qui ne peut exister nulle part ailleurs.
