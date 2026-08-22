# Tasks : équipiers gérés par le daemon (009)

**Prerequisites** : spec (3 rounds), plan (4 rounds, D-501..D-506 + amendement
D-503→socle 012), data-model (table de vérité 7 cas, `StopOutcome`), contrat
ordres-cycle-de-vie, quickstart, reuse-audit (base `e028311`, branche de
départ **session-12** pour consommer le socle).
**Base pinnée** (review tasks) : la branche 009 est partie de session-12 —
consigner dans `implementation.md` le commit exact de base ; la gate T911
n'intègre que les commits 012 REVUS requis (socle : 6df60cc et descendants
validés) puis la 008 finale.
**Gates inter-branches** : les tâches marquées *(gate daemon)* attendent le
feu vert du référent si des reviews 008/012 touchent encore `daemon.rs`.

**Règles** : cargo test + clippy -D warnings avant commit ; commit en review
immuable ; 1 tâche = 1 commit ; auteur ≠ relecteur ; zéro trace IA ; jamais de
push ; tests de crash RÉELS (processus tués), jamais simulés.

---

## Phase 0 — Fondations

- [ ] **T901** Setup + ADR : worktree `.worktrees/009-daemon-spawn` (branche
  `session-09-daemon-spawn` depuis `session-12-contrat-client-idempotent`),
  copie des artefacts, `docs/decisions/006-daemon-spawn.md` (ADR : daemon
  lance le wrapper, bootstrap à octet, réconciliation, consommation du socle).
  **Observable** : ADR auto-portant, commit docs(009).

- [ ] **T902** `desired_state.rs` : `fleet.json` schéma 1 (clé stable,
  `command_id`+génération), écriture durable via `fsutil` (temp+fsync+rename+
  fsync répertoire), daemon seul écrivain, chargement au démarrage.
  **Observable** : tests — frontière temp→rename prouvée par **processus
  écrivain enfant RÉELLEMENT tué à une barrière** avant/après le rename
  (règle « crash réels » — l'injection de faute est réservée aux erreurs
  d'E/S), réouverture et vérification ancien/nouvel état ; schéma versionné ;
  retrait durable.

- [ ] **T903** `managed_process.rs` : sous-mode `managed-bootstrap` (spawn
  standard, `setsid`, `BootstrapReady{pid,pgid,birth,instance_id,command_id,
  generation}` sur le FD de statut hérité, attente `read_exact(1)` octet
  `RELEASE`, EOF→`_exit`, `exec` wrapper) ; discipline des FDs (RELEASE et
  auxiliaires CLOEXEC, statut seul hérité) ; marqueurs `managed/` durables
  écrits par le daemon entre `BootstrapReady` et `RELEASE`.
  **Observable** : tests — octet-vs-EOF explicites, crash aux trois
  frontières (avant marqueur / après marqueur avant RELEASE / après RELEASE),
  FDs prouvés via un **exécutable-sonde post-exec** (le hook réel du wrapper
  n'existe qu'à T906, qui porte le test d'intégration
  `BootstrapReady`≠`Connected`/`StartupFailed` corrélés), `Ready`≠succès.

## Phase 1 — Ordres et machine d'états *(gate daemon)*

- [ ] **T904** `fleet.rs` orchestration : machine
  `Requested→Reserved→Starting→Connected|Failed|Cancelled` (réservation
  atomique nom+slot+quota sous le verrou d'état, générations), consommation
  du **socle 012** avec la CLÉ COMPLÈTE : un **`issuer_scope` interne du
  superviseur** — réservé, stable et durable à travers les redémarrages,
  distinct de tout scope client, créé avant la première réservation —
  (`operation_kind="spawn"`, `idempotency_key=command_id`) ; ordre
  fleet.json→issue-après-Register-réel→réponse, retry rattaché à la
  génération en vol — jamais de Connected synthétique ; `spawn_commands` =
  saga seule, création ET finalisation dans UNE transaction FK avec le
  socle ; **aucun second lookup/rejeu dans `fleet.rs`** (le socle est seul
  juge).
  **Observable** : tests à barrières — deux spawns simultanés même nom,
  timeout avec `Register` tardif rejeté, retry après chaque point de crash
  D-503 (y compris après redémarrage pour un persistant : même scope interne
  retrouvé, même `command_id` → même issue), `EnvelopeMismatch` sur ordre
  divergent, `IdempotencyExpired`.

- [ ] **T905** Ordres client + refus typés + projection CLI : `SpawnOrder`/
  `SpawnAccepted`, `StopOrder`/`StopOutcome` (5 issues), la table fermée des
  11 refus (contrat) ; **CLI observable par l'humain** : `bridget spawn` /
  `bridget stop` génèrent et AFFICHENT le `command_id`, option
  `--command-id` pour rejouer exactement un ordre (canon identique au
  retry) ; garde de facturation au spawn sur l'environnement **source**
  (D-505), environnement construit — **schéma `pass_env` déclaré dans
  `registry.rs`** avec validation/bornage testés — + `cwd` client
  validé/revalidé.
  **Observable** : matrice SC-003 automatisée (11 familles × motif typé ×
  zéro état résiduel) ; tests CLI parsing/round-trip/réponse-perdue-puis-
  `--command-id` → même issue ; **gate réelle Codex ET Claude lancés en
  environnement nettoyé** (preuve que baseline+pass_env suffisent aux CLIs
  réels — D-505) ; quickstart §3 échantillon.

- [ ] **T906** Canal de statut + supervision : `StartupFailed{kind,reason}`
  via hook `managed-status` du wrapper (FD hérité), `waitpid` non bloquant au
  tick (enfants du daemon courant), mort spontanée → chemins 007
  (DeliveryRejected, états, `End` attach FR-011ter), stderr par équipier :
  fichier dédié **créé 0700/0600 dès l'ouverture, séparé par
  instance/génération, purgé par âge** (politique alignée journaux), jamais
  parsé comme protocole.
  **Observable** : spawn à commande absente → motif exact via canal ; mort
  d'équipier → échecs motivés + `stopped` + `End` aux vues ; stderr
  consultable + tests permissions/rétention ; test d'intégration du hook
  `managed-status` réel (BootstrapReady≠Connected, StartupFailed corrélé).

## Phase 2 — Arrêt et réconciliation *(gate daemon)*

- [ ] **T907** `stop` de groupe : résolution primaire par table superviseur
  nom→génération (`Reserved`/`Starting` couverts), invalidation de génération
  d'abord, handshake d'arrêt wrapper (chemin 007) → attente bornée → escalade
  `killpg` → polling `kill(-pgid,0)` → `StopOutcome` ; refus `NotManaged`
  structurel ; marqueur supprimé sur disparition confirmée.
  **Observable** : tests — stop avant marqueur / bootstrap bloqué / après
  Register / marqueur périmé ; adaptateur ignorant l'annulation ; descendant
  `npx` ; `stop` d'un wrapper-terminal → refus.

- [ ] **T908** Réconciliation + phase `Recovering` : au démarrage — scan
  `managed/`, validation naissance+instance, terminaison des groupes périmés
  (`killpg`+polling borné, `ECHILD` documenté), marqueurs supprimés sur
  disparition confirmée, PUIS réservation ordonnée des reprises avant
  ouverture ; `DaemonRecovering` pour tout spawn nouveau (lookup `command_id`
  AVANT le refus — contrat) ; stop-gagne-sur-reprise (FR-010bis, tombstone).
  **Observable** : SIGKILL réel du daemon → redémarrage → aucun ancien groupe
  + une seule instance par persistant (SC-006) ; stop du 2e élément pendant
  que le 1er démarre (barrière) ; matrice de reprise (succès/échec/conflit/
  quota).

## Phase 3 — Parité et finition

- [ ] **T909** Matrice de parité FR-008 : corpus commun (quickstart 007 §1-§5
  automatisés) exécuté wrapper-terminal PUIS daemon-géré, observables
  comparés (N et tolérances versionnés) ; suite de frames attach comparée sur
  la même fixture dans les deux modes.
  **Observable** : matrice versionnée au vert, chiffres consignés.

- [ ] **T909b** Banc SC-001 réel : 20 spawns en environnement gelé et
  consigné, p95 < 10 s avec timeout global, **fermeture réelle du terminal
  donneur d'ordre**, puis 20/20 échanges suivis complets.
  **Observable** : chiffres consignés, banc reproductible.

- [ ] **T910** Persistance bout-en-bout : `--persistent`, cycles SC-005 (3×
  redémarrage coopératif : persistants 3/3, éphémères 0/3, stop exclut 3/3),
  arrêt coopératif SC-006 avec **N=3 groupes dont descendants `npx`**, liste
  des pgid comparée avant/après (falsifiable), et le cas **SIGKILL réel**
  (T908) rejoué dans la matrice finale SC-006.
  **Observable** : quickstart §4 automatisé, listes de pgid consignées.

- [ ] **T911** Finition : README (« équipiers persistants »), DEPRECATIONS
  relu, `implementation.md` avec SC-001..SC-006 pointés (SC-001 : N=20,
  p95<10 s), gate d'intégration des branches amont (008 finale + 012 socle)
  avant clôture.
  **Observable** : checklist pointée, `git log` prouvant l'intégration.
