# Tâches — Session 026

## Phase 1 — Spécification et témoins sans collision

- [x] T2601 Formaliser les propriétés, la frontière ADR 011 et la définition de contradiction attestable dans `specs/026-operations-greffe-central/spec.md`.
- [x] T2602 Documenter l'architecture à une porte, la décision SQL/Rust, v19 et l'ordre de composition 021→v18→026 dans `specs/026-operations-greffe-central/plan.md`.
- [x] T2603 [P] Écrire deux contre-tests autonomes distincts pour `profile_approve` et `routine_approve` dans `plugins/maicie/tests/guichet_forbidden_operations_contract.rs`; observable : les deux fonctions de test sont exécutées et comparent exactement outcome, opération et motif.
- [x] T2604 Exécuter `cargo test --workspace --no-run`, puis le binaire de test T2603 et consigner les deux rouges attendus sans les attribuer à 021 dans le jalon de branche.

## Phase 2 — Fondations après admission de 021 et v18

- [ ] T2605 Rebaser sur la tête admise contenant 021, relire le vocabulaire de refus unifié et adapter `specs/026-operations-greffe-central/plan.md` sans réintroduire de projection manuelle.
- [ ] T2606 Composer la migration v18 admise avant toute écriture de v19 dans `plugins/maicie/src/store.rs`; observable : une base v17 traverse v18 puis v19 sans saut de version.
- [ ] T2607 Écrire les témoins v19 rouges dans `plugins/maicie/tests/schema_migration_guard.rs` : injection SQL inconnue fail-closed, round-trip exact de `ALL`, bijection enum↔corpus et conservation des `CHECK` d'état.
- [ ] T2608 Implémenter v19 dans `plugins/maicie/src/store.rs` en ouvrant seulement les vocabulaires SQL et en conservant contraintes d'état, unicité et corrélation ; observable : T2607 vert depuis v17/v18.
- [ ] T2609 Créer une source Rust unique pour opérations et motifs dans `plugins/maicie/src/domain.rs` et supprimer les projections manuelles devenues redondantes dans `plugins/maicie/src/guichet.rs` et `plugins/maicie/src/store.rs`.

## Phase 3 — US2601 : validation partagée (P1)

**But** : le défaut local cesse avant l'ouverture des capacités fédérées.

**Test indépendant** : le même corpus rend exactement les mêmes décisions via
la CLI locale et un appel de cas d'usage destiné au guichet.

- [ ] T2610 [P] [US2601] Écrire le corpus exact des contradictions, références et ambiguïtés dans `plugins/maicie/tests/contract/f36_f37_suite_citations.rs`; observable : chaque fixture attend `coherent`, `refused` ou `signaled`, jamais une sous-chaîne.
- [ ] T2611 [US2601] Implémenter le validateur unique au début de `delegate` dans `plugins/maicie/src/app.rs`, sans logique équivalente dans `plugins/maicie/src/main.rs`.
- [ ] T2612 [US2601] Persister refus et signaux idempotents dans `plugins/maicie/src/store.rs`; observable : refus sans objectif créé, signal atomique avec la réservation et rejeu sans ligne double.
- [ ] T2613 [US2601] Brancher la CLI locale uniquement par le cas d'usage existant dans `plugins/maicie/src/main.rs`; observable : le cas mesuré `suite=aucune + depends_on` est refusé et une référence de revue est seulement signalée.

## Phase 4 — US2602 : délégation fédérée (P1)

**Test indépendant** : un dépôt perdu puis rejoué crée un seul objectif et une
seule délégation dans la base centrale, jamais dans une base leurre locale.

- [ ] T2614 [P] [US2602] Ajouter les fixtures canoniques `delegate` valides, divergentes et contradictoires au corpus de contrat sous `specs/026-operations-greffe-central/contracts/`.
- [ ] T2615 [US2602] Étendre les charges et réponses fermées dans `crates/bridget-transport/src/protocol.rs`; observable : toute variante de `ALL` a exactement une fixture et aucun champ inconnu n'est accepté.
- [ ] T2616 [US2602] Persister dépôt, claim, refus et résultat terminal dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/daemon.rs` sans interprétation métier.
- [ ] T2617 [US2602] Convertir le claim vers `DelegateRequest` et appeler le cas d'usage partagé dans `plugins/maicie/src/guichet.rs` et `plugins/maicie/src/app.rs`; observable : IDs et état d'attente exacts dans le reçu.
- [ ] T2618 [US2602] Fournir configuration, candidats, horloge et politiques centraux via `plugins/maicie/src/reconcile.rs` et `plugins/maicie/src/main.rs`; observable : aucune valeur sensible ne vient du payload distant.

## Phase 5 — US2603 et US2604 : registre et clôture (P1)

- [ ] T2619 [P] [US2603] Écrire les témoins `registre_add` append, rejeu et divergence dans `plugins/maicie/tests/catalogue_session_gate.rs`, avec un chemin local leurre explicitement inchangé.
- [ ] T2620 [US2603] Implémenter la conversion fermée et l'append central dans `plugins/maicie/src/guichet.rs`, `plugins/maicie/src/reconcile.rs` et `plugins/maicie/src/catalogue.rs` seulement si la primitive existante ne suffit pas.
- [ ] T2621 [P] [US2604] Écrire les témoins de clôture attestée, preuve absente, objectif déjà clos et rejeu exact dans `plugins/maicie/tests/integration/cli_objective.rs` ou un nouveau test autonome non concurrent.
- [ ] T2622 [US2604] Implémenter `objective_close` sans motif libre dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/guichet.rs`; observable : seule une livraison centrale corrélée autorise la clôture.

## Phase 6 — US2605 et fermeture de sécurité (P1)

- [ ] T2623 [US2605] Étendre `GuichetResult` et le lookup dans `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/daemon.rs` pour rendre issue et charge terminales durables.
- [ ] T2624 [US2605] Étendre le CLI client dans `crates/bridget-daemon/src/cli.rs` avec erreur avant dépôt si tunnel absent et reprise stricte des mêmes id/date/octets.
- [ ] T2625 [US2605] Rendre verts les deux contre-tests de `plugins/maicie/tests/guichet_forbidden_operations_contract.rs`; observable : deux refus durables exacts et zéro mutation métier.
- [ ] T2626 [P] [US2605] Ajouter le quatrième oracle enum↔corpus et des mutants de comparaison exacte dans les tests de contrat Bridget et Maicie ; observable : ajout d'une variante sans fixture et assertion `contains` mutée font rougir.

## Phase 7 — Validation et livraison

- [ ] T2627 Exécuter `cargo test --workspace --no-run` avant comptage, puis les suites ciblées, le parcours central réel, formatage et clippy ; consigner SHA, passés, rouges, ignorés et imputation de chaque rouge.
- [ ] T2628 Mesurer les compositions 021, v18, 025/v20 et les autres têtes gelées touchant protocole/guichet/store ; observable : aucune divergence sémantique silencieuse.
- [ ] T2629 Relire le diff complet selon les articles XIX/XX, supprimer les duplications et documenter précisément le code évité et les surfaces non mesurées.
- [ ] T2630 Geler, pousser et livrer la tête de `session-026-operations-greffe-central` avec message Git strict et sans trailer.

## Dépendances et stratégie

- T2601–T2604 sont indépendantes des fichiers de 021.
- T2605 et T2606 bloquent tout code de production.
- T2607–T2609 bloquent les trois opérations.
- Le paquet A est T2610–T2613 et doit être livrable séparément.
- Le paquet B est T2614–T2626 ; US2603 et US2604 peuvent préparer leurs tests
  en parallèle après gel du contrat, mais leurs écritures centrales restent
  séquentielles.
- Le MVP sûr comprend T2601–T2625. Aucun MCP direct ni runtime Maicie résident
  n'est requis.

## Mesure du jalon documentaire/TDD

**SHA de base réellement mesuré** :
`b6eea777facf929d99a9c4f9ae75fb50e06dc2fd`, avec le diff des quatre fichiers
neufs du jalon 026.

- `cargo test --workspace --no-run` : compilation verte, aucun binaire de test
  absent.
- `cargo test -p maicie --test guichet_forbidden_operations_contract` :
  **0 passé, 2 rouges, 0 ignoré**.
- Rouge `profile_approve_est_refuse_et_persiste_exactement` : imputé au défaut
  TDD attendu ; le code courant rend `UnsupportedOperation` sans refus terminal
  durable.
- Rouge `routine_approve_est_refuse_et_persiste_exactement` : même imputation,
  exercée par une fonction distincte.
- `--features test-support` : non joué ; cette mesure n'en a pas besoin et le
  gate Linux `kqueue`/`kevent` reste hors périmètre connu.
- `cargo fmt --all --check` : rouge sur la dette antérieure à 026 dans
  `plugins/maicie/src/routines.rs`, `plugins/maicie/src/store.rs`,
  `plugins/maicie/src/main.rs` et
  `plugins/maicie/tests/contract/routines.rs`; aucun hunk ne vise le témoin 026.

Un premier lancement a été arrêté par le harnais avant l'oracle métier, car le
répertoire temporaire n'était pas en `0700`. Le fixture a été corrigé, recompilé
avec `--no-run`, puis rejoué ; ce rouge de harnais n'est pas compté comme un
défaut produit.

## Self-review Article XIX/XX du jalon

- **Pourquoi cette solution est nécessaire** : douze agents fédérés ne peuvent
  pas appliquer les trois mutations centrales et la valeur gratuite `aucune`
  masque déjà des relations mesurées.
- **Pourquoi elle est plus simple ou maintenable** : elle étend le guichet et
  les cas d'usage existants, sans second accès MCP au store, daemon Maicie ni
  validation dupliquée dans le CLI.
- **Hypothèses prises** : 021 fournira le vocabulaire de refus unifié et v18
  sera admise avant l'écriture de v19.
- **Vérifications réalisées** : lecture des schémas et chemins d'appel,
  compilation workspace, exécution séparée des deux témoins, formatage ciblé,
  `git diff --check` et validation du format des tâches.
- **Non vérifié** : implémentation, migration v18→v19, parcours central réel,
  suites `test-support` Linux et composition finale.
- **Code supprimé ou évité** : aucun code de production à ce jalon ; un client
  MCP direct, un repli SQLite local, un interprète de prose et deux validateurs
  parallèles sont explicitement évités.
- **Complexité ajoutée et justification** : la prélecture d'un nom d'opération
  interdit et l'audit `refused|signaled` sont nécessaires pour enregistrer ce
  qui a été refusé sans autoriser cette opération.
