# Tasks : contrat client idempotent (012)

**Prerequisites** : spec (2 rounds), plan (4 rounds, D-601..D-608),
data-model, contrat, reuse-audit (base `session-08-attach@61f7072`).
**Séquencement inter-branches (impératif)** : lots 1-2 = fichiers nouveaux
uniquement ; les tâches touchant `daemon.rs`/`wrapper.rs` attendent le feu
vert du référent (T804b/T805 en vol sur la 008).

**Règles** : `cargo test` + `clippy -D warnings` avant chaque commit ; commit
en review immuable ; 1 tâche = 1 commit ; zéro trace IA ; jamais de push ;
auteur ≠ relecteur.

---

## Phase 0 — Fondations (fichiers nouveaux seulement)

- [ ] **T1201** Setup + ADR : worktree `.worktrees/012-contrat-client-idempotent`
  (branche `session-12-contrat-client-idempotent` depuis `session-08-attach`),
  copie des artefacts 012, `docs/decisions/007-socle-idempotence.md` (socle
  unique, consommateurs 009/010, portée exactly-once honnête).
  **Observable** : ADR auto-portant, commit docs(012).

- [ ] **T1202** Socle `idempotency.rs` + table `idempotency_records`
  (data-model : clé composite, `canonical_bytes` autorité, machine
  `Prepared→Dispatching→Terminal` monotone, `issued_at`/`expires_at` figés,
  validation scope 128 bits/longueur/charset).
  **Observable** : tests — réservation atomique (2 threads, 1 gagnant),
  comparaison exacte des octets (divergence d'un octet → mismatch), rejeu,
  `IdempotencyExpired` (y compris premier Send hors horizon), transitions
  monotones refusant la régression, purge par `expires_at` propre (baisse de
  config testée).

- [ ] **T1203** `receipt_store.rs` (wrapper) : `~/.local/state/bridget/receipts/
  <instance_id>/`, marques `Seen`/`Acked` par `delivery_id`, écriture
  atomique+fsync, quotas octets/entrées, purge/compaction par `expires_at`,
  `max_expires_at` atomique séparé, **quarantaine fail-closed** (corruption,
  disparition post-init → jamais de recréation vide).
  **Observable** : tests — deux phases, redélivrance `Acked` rejouée / `Seen`
  → indéterminée, saturation → refus terminal, corruption ciblée d'un `Acked`
  → quarantaine et zéro réinjection, permissions ; **ajoutés en review** :
  deux threads/même `delivery_id` → un seul `Inject` l'autre `Indeterminate`,
  quota concurrent jamais dépassé, sentinel `max_expires_at` disparu →
  quarantaine, répertoire supprimé/recréé vide → quarantaine.

## Phase 1 — Protocole et négociation *(gate référent : après T804b close)*

- [ ] **T1204** Rôle `client` + négociation : variante `Client` dans
  `ConnectionRole`, `ClientHello`/`ClientWelcome` (capacités négociées
  retournées), état `Negotiated(version, issuer_scope)`, matrice fermée
  (client-only / daemon→wrapper / wrapper→daemon, rien sur attach), règle
  unique sans OU.
  **Observable** : tests croisés par variante (attach→ClientHello refusé,
  ClientHello avant RoleAccepted refusé sans mutation, Lookup/SendIdempotent
  avant Welcome refusés sans lecture/réservation, wrapper→DeliverAcked ok,
  Register historique inchangé).

- [x] **T1205** `SendIdempotent` daemon : ordre des gardes (scope →
  réservation+canon → gardes mutables ; issue connue rejouée sans reroutage),
  transaction unique socle+`send_deliveries` (FK, `delivery_id`,
  `recipient_instance_id`/`delivery_generation` figés à la résolution).
  **Observable** : sondes de non-contamination (Register+Send historiques →
  zéro record), rejeu/refus/mismatch bout-en-bout daemon, deux scopes même
  `message_id` → deux `delivery_id`.

## Phase 2 — Remise aval accusée *(gate référent : coordination wrapper avec 008)*

- [x] **T1206a** `PromptDispatched` (acp.rs seul) : émission de l'événement
  interne **après write+flush réussis** de la frame `session/prompt`.
  **Observable** : faux adaptateur à barrière comptant la LECTURE effective
  des frames ; échec avant flush → pas d'événement. *(Scindée en review pour
  garder auteur ≠ relecteur praticable.)*

- [x] **T1206b** Intégration `receipt_store` dans le wrapper :
  `DeliverIdempotent` → `Seen` → injection → `PromptDispatched` → `Acked` →
  `DeliverAcked` ; `DeliveryIndeterminate` sur `Seen` seul ou quarantaine ;
  ciblage génération/instance (accusé obsolète rejeté).
  **Observable** : crash daemon après remise avant issue → redélivrance, un
  seul prompt ; rename/nouveau wrapper même nom → indéterminée.

- [ ] **T1207** Lookup + finalisation `Accepted` sur accusé : résultat calculé
  aux 4 états, `OutcomeUnknown` en vol, `expires_at` retourné, `Accepted`
  écrit seulement sur `DeliverAcked`.
  **Observable** : les 4 états testés + « retry en vol → Unknown rejoué puis
  terminal ».

## Phase 3 — Projections et bancs

- [ ] **T1208** Projection CLI (`--id`/`--issued-at`/`--issuer-scope`
  obligatoires ensemble) + client socket de référence (code de test) ;
  non-régression : suite 007/008 intacte.
  **Observable** : gate 012 de SC-004 (référence ↔ CLI, même issue), corpus
  SC-003 divergences (`EnvelopeMismatch` sans altération).

- [ ] **T1209** Matrice de crash SC-001 : bancs aux 4 points (avant
  réservation / après `Prepared` / après remise avant issue / après issue
  avant accusé client) × redémarrages, N=50 réparti, comptage des prompts.
  **Observable** : 50/50 livraisons uniques, issues rejouées identiques.

- [x] **T1205bis** Câblage daemon de la remise idempotente (correctif de
  production, découvert en préparant la table de vérité) : daemon.rs construit
  et écrit `DaemonToWrapper::DeliverIdempotent{delivery_id,
  recipient_instance_id, delivery_generation, expires_at}` au passage à
  Dispatching ; redélivrance des `send_deliveries` non-`Acked` au redémarrage
  (reprise bornée, ciblage génération/instance).
  **Observable** : tests E2E daemon→wrapper des deux chemins (nominal +
  redémarrage), un seul prompt injecté.

- [x] **T1210** Table de vérité de récupération E2E : **les 5 lignes exactes
  du data-model 012** (socle×`send_deliveries`×reçu wrapper), avec les
  terminaux propres à la 012 (`Accepted` rejoué, finalisation sur
  `DeliverAcked` rejoué, `DeliveryIndeterminate`→`OutcomeUnknown` maintenu,
  redélivrance sûre, reprise/`Rejected` depuis `Prepared`). *(Corrigé en
  review : « 7 lignes », `Recovering` et `Failed` étaient une dérive de copie
  depuis la table 009 — ces notions appartiennent à la machine spawn.)*
  **Observable** : un test par ligne de la table 012, chacun après crash
  daemon simulé.

- [ ] **T1211** Finition : README (section envois idempotents), DEPRECATIONS
  relu, `implementation.md` avec SC-001..SC-006 pointés (SC-004 gate MCP
  explicitement déférée à la 010).
  **Observable** : checklist pointée avec preuves.
