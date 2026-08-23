# Data Model : contrat client idempotent

## `idempotency_records` (socle, SQLite — propriétaire : clé, canon, résultat public)

| Colonne | Rôle |
|---|---|
| `issuer_scope` | opaque ≥128 bits, validé (longueur/caractères) |
| `operation_kind` | `send` (012) ; `spawn` (009 consommatrice, après amendement D-503) |
| `idempotency_key` | projetée : `message_id` (send) / `command_id` (spawn) |
| `canonical_bytes` | **autorité d'égalité** (comparaison exacte ; taille bornée ; stockés une seule fois — la table consommatrice référence) |
| `state` | `Prepared` \| `Dispatching` \| `Terminal` (monotone) |
| `public_result` | `Accepted{...}` \| `Rejected{catégorie, motif}` (seulement en `Terminal`) |
| `issued_at` | immuable, au canon, validé (référentiel daemon + tolérance négociée, sinon `InvalidIssuedAt`) |
| `expires_at` | `issued_at + horizon négocié`, **figé** — la purge n'a jamais lieu avant, quelle que soit la config courante |

Index unique (`issuer_scope`, `operation_kind`, `idempotency_key`). Résultat
de lookup **calculé** : `Terminal` → son `public_result` ; non terminal →
`OutcomeUnknown` ; absent/expiré → `IdempotencyExpired` (avec la règle
`issued_at` hors horizon au premier `Send` → `IdempotencyExpired`, FR-008).

## `send_deliveries` (consommatrice — propriétaire : saga de remise)

| Colonne | Rôle |
|---|---|
| `delivery_id` | **opaque, unique, durable** — la clé de TOUTE la voie aval (jamais `message_id` seul : collision inter-scopes) |
| FK → `idempotency_records` | création et finalisation **dans la même transaction** que le socle |
| `recipient_instance_id` | figé à la **première résolution** du destinataire |
| `delivery_generation` | **jeton durable propre à la remise** (précision round 6) : attribué par `send_deliveries` à la première résolution, persisté, répété par les accusés — ce n'est **pas** la génération 009 (inexistante chez les wrappers-terminal 007) ; sa seule source est cette table, et le couple (`recipient_instance_id`, `delivery_generation`) reste vérifiable après redémarrage du daemon |
| `phase` | remise en cours / accusée / indéterminée |

## Store de reçus (wrapper — `~/.local/state/bridget/receipts/<instance_id>/`)

Répertoire d'**état** (jamais `~/.cache`), 0700/0600, écriture atomique +
fsync. Entrées : `delivery_id → Seen | Acked` + `expires_at` (transmis par
`DeliverIdempotent`). Quotas octets/entrées ; purge/compaction atomique après
`expires_at` ; `max_expires_at` tenu séparément et atomiquement. Corruption
ou disparition post-init → **quarantaine fail-closed** (tout `delivery_id` de
la génération → `DeliveryIndeterminate` jusqu'à la borne sûre), jamais de
recréation vide.

## Variantes filaires (matrice de rôles fermée)

| Variante | Rôle émetteur → destinataire | Champs |
|---|---|---|
| `RoleHello(Client)` / `RoleAccepted` | client ↔ daemon | — |
| `ClientHello` | client (post-`RoleAccepted`) | version, `issuer_scope`, capacités |
| `ClientWelcome` | daemon | version, `horizon_secs`, `issued_at_tolerance_secs`, **capacités négociées** |
| `SendIdempotent` | client (post-`Welcome`) | enveloppe + `message_id`, `issued_at` |
| `Lookup` / réponse | client (post-`Welcome`) | (`operation_kind`, `idempotency_key`) → résultat calculé + `expires_at` |
| `DeliverIdempotent` | daemon → wrapper | **`delivery_id`, `recipient_instance_id`, `delivery_generation`, `expires_at`, message** |
| `DeliverAcked` / `DeliveryIndeterminate` | wrapper → daemon | **`delivery_id` + `delivery_generation`** (accusé d'instance obsolète rejeté) |

Aucune de ces variantes n'est admise sur une connexion attach ; `Send` et
`Register` historiques strictement inchangés.

## Table de vérité de récupération (crash daemon)

| socle | `send_deliveries` | reçu wrapper | Récupération |
|---|---|---|---|
| `Terminal` | accusée | `Acked` | rejouer `public_result` |
| `Dispatching` | remise en cours | `Acked` (accusé perdu) | à la redélivrance : `DeliverAcked` rejoué → finaliser `Terminal(Accepted)` |
| `Dispatching` | remise en cours | `Seen` | `DeliveryIndeterminate` → `OutcomeUnknown` maintenu jusqu'à `expires_at` |
| `Dispatching` | remise en cours | absent (jamais reçu) | redélivrance sûre (le wrapper n'a rien vu) |
| `Prepared` | — | — | reprise du dispatch ou `Rejected` motivé selon la frontière |
