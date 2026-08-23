# Revue hostile finale T026 — plugins/maicie — 2026-08-23

Relecteur : fable-reviewer (équipier géré, désigné T026). Tête revue : 2c7ae32
(delta rustfmt non commité vérifié sans effet comportemental). 12 modules lus
intégralement (~7,6k lignes), contrats croisés avec le producteur réel
(bridget-transport/protocol.rs, bridget-daemon/daemon.rs + idempotency.rs),
tests exécutés.

## Premier verdict : NON MERGEABLE EN L'ÉTAT

Deux bloquants (C1, C2), quatre constats à corriger ou documenter (C3-C6),
le reste mineur. Axes outbox délégation, TOCTOU d'activation, fuite de
contexte et permission ACP jugés solides d'emblée.

### Bloquants

- **C1 [HIGH — chaîne US4 injoignable]** main.rs:404-408 : l'enum Command
  n'exposait que delegate/status/objective. Aucun chemin CLI vers
  propose_profile_activation/approve_profile_activation (app.rs:269/317) ;
  reconcile_activation_startup_at (reconcile.rs:275-381) jamais appelé par le
  binaire. Toute ActivationOutbox pending n'aurait jamais été réconciliée en
  production.
- **C2 [HIGH — harnais non portable]** ack_lost_recovery.rs:474 :
  reprise_lente échouait 6/6 sur la machine du relecteur (QoS arrière-plan,
  sleeps 20 ms mesurés 220-250 ms) ; budget global 100 ms épuisé pendant la
  négociation. Produit conforme, harnais à 40 ms de marge intenable.

### À corriger ou documenter

- **C3 [MEDIUM]** store.rs:337-377 apply_objective_decision : UPDATE sans
  garde d'état ; close() lisait l'objectif hors transaction — un close
  concurrent pouvait écraser une issue terminale commitée.
- **C4 [MEDIUM]** apply_participant_decision : aucune garde d'état — décision
  enregistrable sur objectif déjà Clos.
- **C5 [MEDIUM]** SpawnOrder émis via connexions rôle wrapper fraîches (le
  rôle client négocié ne les admet pas) : l'approbation locale Maicie n'est
  pas opposable côté Bridget. Limite v1 à documenter.
- **C6 [MEDIUM]** reconcile.rs:299 : terminal Rejected{idempotency_expired}
  décidé sans consulter le transport alors que la fenêtre
  [retry_until, dedup_retained_until) permettait un replay-lookup ; digest
  divergent enregistré Rejected alors que le spawn a eu lieu.

### Mineurs

- **C7 [LOW]** pas de budget global de passe d'activation.
- **C8 [LOW]** ResolvedAgentDefinition deny_unknown_fields, miroir strict du
  contrat Bridget — évolution côté Bridget casserait le décodage.
- **C9 [LOW]** display_name/command/args sans contrôle des caractères de
  contrôle dans l'écran d'approbation.
- **C10 [LOW]** N+1 objective_snapshots sans annotation de complexité
  (borné SC-008).
- **C11 [LOW]** ~40-60 lignes mortes (allow(dead_code) périmés,
  RuntimeNature::Disponibilite/Idle jamais produits).

### Vérifié sain (contre-preuves)

- **V1** Outbox délégation : prepared avant I/O, lookup-puis-rejeu octets
  exacts, tombstone daemon retenu jusqu'à issued_at+horizon
  (idempotency.rs:459-484) = retry_until négocié. Aucune fenêtre de
  perte/duplication.
- **V2** TOCTOU activation : scellement parameters == spawn_order_bytes,
  revalidation hash in-tx, consommation mono-usage + issue terminale même
  transaction.
- **V3** Contrats 008/009 : trames conformes au producteur réel.
- **V4** Fuite de contexte : telemetry ids-seulement, 0600/0700 exacts,
  minimized_details whitelist. RAS.
- **V5** Permission ACP : aucune « permission humaine en attente » fabriquée,
  Maicie ne répond jamais.

## Correctifs livrés

- Lot A (coderBridget) : 34c74c0 + 5932ca5 — chaîne US4 câblée CLI
  (profile propose/approve), approbation TTY-only à saisie explicite « oui »
  sans aucun drapeau scriptable (STOP intermédiaire de prospective sur
  --confirm, vérifié et appliqué), gardes transactionnelles C3/C4 avec
  preuves de mutation, écran neutralisé C9, annotation C10.
- Lot B (prospective) : ecc850f — harnais C2 recalibré et discriminant,
  C6 replay jusqu'à dedup_retained_until puis terminal, divergence de digest
  journalisée véridique (agent lancé sans suivi), C8 commenté, C11 purgé.
- Lot C (cxbridget) : abcf14f — limite C5 documentée (ADR 003, README),
  candidat v2 « capacité spawn négociée » consigné, C7 suivi mineur.

## Contre-vérification finale (même relecteur) : MERGEABLE

- C1 FERMÉ (Command::Profile + réconciliation au démarrage et après
  approbation ; test CLI vert).
- C2 FERMÉ (recalibrage vérifié sur la machine du diagnostic : 6/6 verts là
  où l'ancien harnais était 6/6 rouge).
- C3, C4, C6, C8, C9, C10 FERMÉS (dont C9 au-delà du constat : approbation
  devenue TTY-only à saisie explicite).
- C5, C7 COMPROMIS-ACCEPTABLES documentés, suivis v2 tracés.
- C11 RÉSIDU mineur non bloquant (~4 lignes, vocabulaire fermé v1).
- Re-tests : suite maicie complète 118/0 à 5932ca5.

Merge dans main : 1dbb57f… puis 8e10dad (clôture session, 26/26).
