# Contre-revue adverse — cxbridget — session 012

**Croisement** : relecture par un moteur distinct de celui de l'auteur

## Round 1 (spec.md)

**Date** : 2026-08-22 · **Verdict** : `BLOCKED` — 10 objections, **toutes
retenues (10/10)**. Le relecteur valide la séparation 011/012 (outbox chez
Maicie, contrat côté daemon).

| # | Objection | Correction appliquée |
|---|---|---|
| 1 | « une seule livraison » sans point de linéarisation SQLite↔remise ; `OutcomeUnknown` sans convergence pour `reply=false` | FR-002 : machine interne durable `Prepared→Dispatching→Terminal` à transitions monotones, frontières de crash définies, remise aval idempotente par `message_id` **accusée par le wrapper** avant `Accepted` (convergence de `reply=false` par l'accusé de remise) |
| 2 | ID post-purge indiscernable d'un ID neuf → recréation possible | FR-008 : `issued_at` immuable au canon, validé contre la borne daemon ; premier `Send` hors horizon → `IdempotencyExpired` ; tests aux trois frontières de purge |
| 3 | identité résolue = nom courant → `EnvelopeMismatch` au `rename` ; nom réattribué héritant des issues | FR-010 : `issuer_scope` stable et durable distinct du nom, survivant au crash client, partagé entre projections ; index (`issuer_scope`, `operation_kind`, `message_id`) ; nom = instantané de traçabilité hors hash |
| 4 | espaces d'ID insuffisants ; « ne peut ni lire ni écraser » = fausse garantie de sécurité | espace de clé complet + lookup borné à la portée ; garantie reformulée en **isolation accidentelle** (modèle coopératif) |
| 5 | `reply_timeout` relatif non canonique ; `hops` ambigu | FR-004 : normalisation unique en échéance Unix absolue persistée avant effet, comparée sans recalcul ; `hops` = valeur cliente initiale |
| 6 | quatre « états » mélangeant stocké et calculé | FR-002 : deux modèles distincts — état interne durable vs résultat de lookup ; transitions monotones, source de convergence, comportement post-redémarrage |
| 7 | horizon mutable → promesse mensongère pour une outbox existante | FR-006 : `expires_at` par opération, persisté au premier `Send`, retourné, purge interdite avant lui même si la config diminue |
| 8 | FR-009 ne doit pas absorber la saga `spawn_commands` | unification limitée au **socle transactionnel** (réservation + hash + rejeu + mismatch + expiration, `operation_kind`) ; 009/010 consommateurs spécialisés ; ordre recommandé : socle d'abord |
| 9 | ordre des gardes absent → issue divergente au retry | FR-001 : réservation/lookup + canon **avant toute garde mutable** ; issue connue rejouée sans reroutage |
| 10 | SC-001 sans points de crash ; SC-004 dépendant de la 010 absente | SC-001 : matrice versionnée de 4 points de crash avec répartition explicite ; SC-004 scindé (gate 012 : client socket de référence + CLI ; gate de conformité MCP portée par la 010) |

## Round 2

**Verdict** : `APPROVE_WITH_CHANGES` — **BLOCKED levé** (linéarisation et
reconnaissance post-purge jugées exprimées). 6 corrections locales, toutes
retenues (6/6) : (1) Assumptions alignée (issuer_scope au canon, nom hors
canon) ; (2) Key Entities séparée en enregistrement interne durable vs
résultat de lookup calculé ; (3) validation d'`issued_at` fixée (référentiel
daemon, tolérance future négociée, `InvalidIssuedAt`, `expires_at = issued_at
+ horizon`) ; (4) SC-005 réécrit sur l'`expires_at` propre (baisse de config
testée) ; (5) socle nommé `idempotency_key` avec projections
`message_id`/`command_id`, durabilité pour toutes les projections (amendement
documenté de la portée éphémère 009) ; (6) **gate de synchronisation 011
conditionnelle** consignée (OutboxDélégation : `issued_at`+`issuer_scope`
persistés avant E/S) et communiquée à `prospective`. Réserve pour le plan
consignée (sémantique de l'accusé wrapper + dedup aval, test à barrière).

**Bilan spec 012 : 2 rounds, 16 objections, 16 retenues, 0 rejetée. Passage au
plan autorisé.**

## Round 3 (plan.md)

**Verdict** : `BLOCKED` — 7 objections, toutes retenues (7/7) : (1) D-602
réécrit en **deux phases durables `Seen`/`Acked`** avec `DeliveryIndeterminate`
et portée honnête d'exactly-once (crash wrapper dans la fenêtre `Seen` =
au-plus-une-injection, limite documentée), test par comptage des
`session/prompt` ; (2) D-604 ordonné dans le plan de rôles 008 (rôle `client`
explicite, `SendIdempotent` refusé sur attach, `DeliverAcked` wrapper-only,
tests croisés) ; (3) D-601 : propriété des états fixée (socle = clé/canon/
expires_at/résultat public ; consommateur = état métier), transaction unique
avec FK, amendement D-503 explicite, migration assumée si l'ordre inverse ;
(4) D-607 créé : variante `SendIdempotent` distincte exigeant `Negotiated`,
`Send` historique intact, sondes de non-contamination ; (5) D-605 : les
octets canoniques sont l'autorité (comparaison exacte, digest = préfiltre,
`DefaultHasher` disqualifié comme preuve), taille bornée, stockage unique ;
(6) D-603 : `issuer_scope` opaque ≥128 bits, possession=identité, portées
fusionnables par contrat, scopes actifs bornés ; (7) D-608 créé : store de
reçus dédié (jamais le fichier de nom), 0700/0600, quotas, purge atomique,
dégradation sûre sur corruption.

## Round 4 (plan)

**Verdict** : `BLOCKED` — 7 objections, toutes retenues (7/7) : (1) dedup aval
par `message_id` seul = collision inter-scopes → `delivery_id` opaque durable
sur tous les messages aval ; (2) « injection réussie » sans observable
(`deliver()` enfile, `TurnStarted` précède la frame) → événement
`PromptDispatched` après écriture+flush de la frame, seul déclencheur
d'`Acked` ; (3) corruption « reconstruite en Seen inconnus » impossible (une
clé `Acked` effacée redeviendrait injectable) → **fail-closed** : quarantaine,
`DeliveryIndeterminate` jusqu'à borne sûre ; (4) store par nom mutable →
indexation par `instance_id`, génération du destinataire figée à la première
résolution, jamais de reroutage par nom ; (5) « OU » du test de rôle supprimé
(règle unique `RoleHello→RoleAccepted→ClientHello`) ; (6) `--issuer-scope`
obligatoire au CLI + correction factuelle (l'outbox 011 n'a pas de scope,
amendement T006 requis) ; (7) références D-607/D-608 démêlées, dimensionnement
honnête (~900-1200 lignes, 11-12 tâches).

## Rounds 5-6 (plan)

- **Round 5** : `APPROVE_WITH_CHANGES` — 6 corrections (store sous
  `~/.local/state` avec quarantaine sur disparition, `ClientWelcome` avec
  capacités + matrice complète, champs filaires de la remise, correction
  factuelle 011, suppression du « OU » résiduel, module `receipt_store.rs` +
  invariant `delivery_id`).
- **Round 6** : **`APPROVE`** — invariants bloquants déclarés fermés ; 3
  nettoyages intégrés (dimensionnement aligné, `Lookup` exigeant `Negotiated`,
  `delivery_generation` définie comme jeton propre à la remise dans le
  data-model).

**Bilan conception 012 : spec 2 rounds (16 obj.) + plan 4 rounds (23 obj.) =
39 objections, 39 retenues, 0 rejetée.** Paquet complet : spec, plan,
data-model, contrat, quickstart, exigences amont. Reuse-audit et tasks à la
décision d'insertion (confirmation utilisateur attendue) ; gate T006/011
consignée.

## Review implémentation T1205 (2026-08-22, commit a7aa961, auteur prospective)

Verdict : **STOP** — 6 points, dont 1-3 vérifiés par le référent dans le code
avant relais (canon incluant `from`/`reply_timeout` contraire à FR-004 ;
`delivery_generation: 1` codé en dur ; FK `send_deliveries` sans cascade →
purge impossible). Points 4-6 : refus terminal en deux transactions,
garde-fous historiques perdus sur la voie nouvelle (D-208, invariants reply,
circuit_breaker par `message.from`), quatre tests manquants. Correctifs en
commits par-dessus a7aa961 (immuable), assignés à prospective.
