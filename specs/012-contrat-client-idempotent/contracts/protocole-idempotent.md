# Contrat : protocole client idempotent

Extension du protocole local public — la référence que les projections (CLI,
client socket de référence, outil MCP 010) traduisent sans en changer la
sémantique. Le `Send`/`Register` historiques sont hors de ce contrat et
inchangés.

## Séquence obligatoire

1. `RoleHello(Client)` → `RoleAccepted` (mécanique de rôles 008 ; tout
   `ClientHello` antérieur est refusé sans mutation ni réservation).
2. `ClientHello { contract_version, issuer_scope, capabilities }` →
   `ClientWelcome { version, horizon_secs, issued_at_tolerance_secs,
   capabilities }` ou refus motivé nommant les versions supportées.
3. Après `Welcome` seulement : `SendIdempotent` et `Lookup` (avant : refus
   sans lecture ni réservation).

## `SendIdempotent`

Enveloppe canonique (champs publiés, FR-004 de la spec) + `message_id` +
`issued_at`. Traitement daemon dans cet ordre **avant toute garde mutable** :
résolution `issuer_scope` → validation `issued_at`
(`InvalidIssuedAt`/`IdempotencyExpired`) → réservation/lookup de clé +
comparaison **exacte des octets canoniques** (`EnvelopeMismatch` si
divergence ; rejeu si connue). Clé nouvelle → gardes historiques → machine
`Prepared→Dispatching→Terminal` → remise aval accusée (contrat ci-dessous) →
réponse (`Accepted{expires_at}` \| `Rejected{catégorie, motif}` \|
`OutcomeUnknown` si le délai de l'appel expire avant le terminal).

## Remise aval (daemon ↔ wrapper destinataire)

`DeliverIdempotent { delivery_id, recipient_instance_id, delivery_generation,
expires_at, message }` → wrapper : reçu `Seen` persisté+fsync → injection →
`PromptDispatched` (écriture+flush de la frame `session/prompt`) → reçu
`Acked` persisté+fsync → `DeliverAcked { delivery_id, delivery_generation }`.
Redélivrance : `Acked` → rejouer l'accusé ; `Seen` seul →
`DeliveryIndeterminate { delivery_id, delivery_generation }` (jamais de
seconde injection) ; accusé d'une génération obsolète → rejeté par le daemon.
Store en quarantaine → `DeliveryIndeterminate` jusqu'à la borne sûre.

## `Lookup`

`(operation_kind, idempotency_key)` → résultat **calculé** : `Accepted{...}`
\| `Rejected{...}` \| `OutcomeUnknown` (« en vol — pas une autorisation de
rejouer ») \| `IdempotencyExpired` — borné à la portée de l'appelant, avec
`expires_at` retourné quand l'enregistrement existe.

## Portée d'exactly-once (publiée, honnête)

Exactement-une-injection garantie à travers crashs **client** et **daemon**
(wrapper vivant, redélivrances comprises). Crash **wrapper** dans la fenêtre
`Seen` : au-plus-une-injection + `OutcomeUnknown` jusqu'à `expires_at` — un
accusé idempotent du consommateur ACP n'existe pas dans le protocole actuel.

## Cas de test imposés (extraits structurants)

Deux scopes / même `message_id` → deux `delivery_id`, deux injections, retries
sans supplément ; comptage des frames `session/prompt` lues par le faux
adaptateur sur toute la matrice de crash de SC-001 ; `rename` du destinataire
avant redélivrance et nouveau wrapper sur le même nom → `DeliveryIndeterminate`
(jamais de reroutage) ; corruption ciblée d'un `Acked` → quarantaine, compteur
de prompts inchangé ; `Lookup`/`SendIdempotent` avant `Welcome` → refus sans
effet ; sondes : la voie historique ne crée jamais d'`idempotency_record`.
