# Exigences amont — contrat client idempotent (012)

**Source** : agent `prospective` (session 011 Maicie), messages Bridget du
2026-08-22. Statut : entrées de spécification, à confronter au reuse-audit.
**Gouvernance** : préparation spec-only mandatée par prospective ;
l'implantation dans la feuille de route reste une décision utilisateur.

## Besoin (périmètre accepté)

Frontière cliente Bridget **publique, versionnée, à négociation de
capacités**, définie **au niveau du protocole local** (socket daemon) — le
binaire CLI et l'outil MCP (010) n'en sont que des projections minces à
sémantique identique. Maicie (binaire externe, SQLite privée) n'importe aucun
crate interne et ne lit jamais `bridget.db`.

1. `Send` avec **`message_id` choisi par le client** (généré et durablement
   persisté côté client avant l'appel).
2. **Issue/tombstone durable** au moins jusqu'à l'horizon de retry — horizon
   de rétention **déclaré à la négociation** (contractuel, pas
   implémentation).
3. **Refus typé d'un même ID à enveloppe divergente.**
4. **Lookup d'issue par ID**, rejouant le même résultat après redémarrage du
   daemon.
5. Au-delà de l'horizon : `IdempotencyExpired` typé, **jamais** de réémission
   silencieuse (l'outbox mémorise `retry_until`).

## Comparaison canonique (champs minimum imposés)

`message_id`, identité émettrice **résolue**, cible, `body_bytes` **exacts**,
`reply`, `reply_timeout` **ou** échéance absolue, `hops`, `in_reply_to`. Les
champs ajoutés par le daemon ne sont exclus du hash que s'ils sont
**explicitement normalisés** avant celui-ci — liste des champs couverts
publiée au contrat.

## États du lookup (fermés)

`Accepted` \| refus terminal (catégorie + motif) \| `OutcomeUnknown` (en vol —
**pas** une autorisation de rejouer aveuglément) \| `IdempotencyExpired`.
L'absence d'issue ne doit **jamais** être confondue avec la permission de
réémettre.

## Diagnostic des primitives actuelles (à vérifier au reuse-audit)

- l'ID est imposé par `BridgetMessage::new` (`cli.rs`) — pas de `message_id`
  client ;
- déduplication et quarantaine sont **en mémoire et expirantes**
  (`dedup.rs`, `envelope.rs`) — pas de durabilité ;
- `ListRequests` ne fournit pas de lookup général par ID.

## Recouvrements à arbitrer (un mécanisme commun, pas un troisième)

- **009 D-503** : `spawn_commands` (command_id, issue durable, générations,
  `IdempotencyExpired`, table de vérité) — même motif pour les ordres de
  cycle de vie ;
- **010 FR-002** : `bridget_send` (id généré avant connexion,
  `outcome_unknown`, retry même id → dedup daemon) — la 010 devra se
  reformuler comme projection de la 012 si celle-ci est retenue ;
- frontière : la 012 ne couvre pas les flux ACP (008) ni le réveil de profils
  (011/futur).

## Gate de synchronisation 011 (conditionnelle — round 2 de la contre-revue)

Si la 012 est retenue par l'utilisateur : l'`OutboxDélégation` de la 011
(`data-model.md:51-60`, `tasks.md:T006`) devra **créer et persister
`issued_at` et `issuer_scope` avant toute E/S** et les rejouer octet pour
octet au retry — sans autre remaniement de la logique Maicie. Communiqué à
`prospective` le 2026-08-22.

## Réserve pour le plan 012 (non bloquante spec)

Définir précisément **ce que le wrapper accuse** à la remise aval et la durée
de sa déduplication : pour le point de crash « après remise, avant issue », un
même `message_id` redélivré après redémarrage du daemon doit produire le même
accusé **sans seconde injection** — test à barrière exigé.
