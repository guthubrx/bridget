# Tasks : Transport ACP pour agents équipiers

**Prerequisites** : spec.md, plan.md, research.md, reuse-audit.md (statut OK)

**Règle** : `cargo test` vert et **zéro warning** (`cargo clippy --all-targets`)
sur tout le workspace avant de cocher. Tout chemin hérité touché est supprimé ou
inscrit à `docs/DEPRECATIONS.md` dans la même tâche.

**Rappel délégation** : implémentation déléguée ; l'agent référent (bridget)
valide chaque tâche avant coche. Ne jamais committer automatiquement.

---

## Phase 0 : Fondations

- [X] **T701** [Fondation] **Spike adaptateur Codex hors Bridget** (quickstart §0) :
  script jetable qui lance `npx @zed-industries/codex-acp@0.16.0`, envoie
  `initialize` + `session/new` + un `session/prompt` trivial, et transcrit les
  échanges bruts. Vérifier : login abonnement (aucune demande de clé API),
  `stopReason` reçu. Consigner la transcription dans `implementation.md` et les
  écarts éventuels dans `research.md`.
  **Observable** : transcription complète d'un tour ; **STOP session si échec**
  (l'hypothèse fondatrice tombe, retour à l'utilisateur).

- [X] **T702** [Fondation] Créer `docs/DEPRECATIONS.md` (colonnes : chemin,
  remplacé par, supprimable quand) et l'ADR
  `docs/decisions/003-transport-acp.md` (contexte, décision — ACP comme
  transport de livraison, client JSON-RPC minimal maison —, conséquences
  positives/négatives, statut Accepté), en reprenant research R-001/R-004.
  **Observable** : les deux fichiers existent, l'ADR est auto-portant.

---

## Phase 1 : US1 — Équipier Codex (MVP)

- [X] **T703** [US1] **Registre d'agents** : `crates/bridget-daemon/src/registry.rs`
  — structures sérialisées `serde_json` conformes à `data-model.md`, registre
  par défaut embarqué (codex/claude/gemini, versions R-002), fusion avec
  `~/.config/bridget/agents.json`, validation avec messages en français (type
  inconnu → refus nommant le fichier et les types disponibles). Brancher
  `cli.rs:62-77` et remplacer la liste blanche `wrapper.rs:522` ; inscrire la
  liste supprimée à `DEPRECATIONS.md`.
  **Observable** : tests unitaires (défauts, fusion, type inconnu, clé
  inconnue → avertissement) ; les agents tmux actuels se lancent à l'identique
  via le registre.

- [X] **T704** [US1] **Client JSON-RPC minimal + `AcpTransport`** :
  `crates/bridget-transport/src/acp.rs` — spawn du sous-processus adaptateur
  (stdio pipés), `initialize` (négociation de version → erreur explicite si
  incompatible), `session/new`, un seul thread lecteur avec dispatch
  réponses/notifications (motif `wrapper.rs:582`), `session/prompt` composé
  selon `contracts/livraison-acp.md` (en-tête + corps octet pour octet),
  collecte des blocs de réponse, `stopReason`, réponse automatique aux
  `session/request_permission` selon la politique du registre,
  `session/cancel` à l'arrêt. Modèle de threads **imposé par D-204** : un seul
  lecteur propriétaire de stdout (table `request_id → waiter`, requêtes
  serveur→client traitées même pendant un prompt), writer sérialisé, worker de
  tours ; `deliver()` valide, enfile et retourne — jamais bloquant.
  **`AcpTransport` est le propriétaire unique de `turn` et de la file bornée**
  (politique de `data-model.md` : capacité, purge par id sur signal
  d'annulation, échec terminal en dépassement — D-209). Implémente le trait
  `Transport` + méthode d'état de tour.
  **Observable** : tests de conformité sur fixtures JSON-RPC enregistrées —
  client générique + **fixtures Codex capturées au spike T701** — couvrant
  **toute la matrice R-004** (notifications intercalées, requête serveur→client
  pendant prompt, réponses hors ordre, ids chaîne/nombre, objet `error`, EOF,
  ligne invalide, méthode inconnue) plus tour normal/vide/erreur, permission,
  version incompatible ; tests de file : FIFO, dépassement de capacité → rejet
  terminal, purge par id ; corps spéciaux intacts octet pour octet. (La
  fixture de couche 2 Claude est ajoutée par T707 ; la preuve de couche 2 se
  limite à Codex/Claude — Gemini indisponible, constat T708.)

- [X] **T705** [US1] **Branchement wrapper équipier et protocole** : flag de
  lancement (`cli.rs`, nom du flag à fixer et documenter), mode ACP dans
  `wrapper.rs` — pas de prompt « Règles ABSOLUES » (FR-004), `AcpTransport` au
  lieu de `TmuxTransport` ; **le wrapper ne tient aucune file propre** : il
  relaie les signaux au transport (D-204, propriétaire unique). Nouvelles
  variantes de protocole (D-209) : `CancelDelivery { id, reason }`
  (daemon→wrapper, annulation/expiration → purge de file par id) et
  `DeliveryRejected { id, reason }` (wrapper→daemon → échec motivé vers
  l'émetteur, y compris file pleine et mort du processus avec N messages en
  attente). Réponse de fin de tour remise au daemon via
  `WrapperToDaemon::Send` avec `in_reply_to` (réutilisation `protocol.rs:36`)
  selon la table `reply=yes/no` du contrat (D-205). **Correction D-208 dans
  `daemon.rs`** : déplacer la transition `answered` (`daemon.rs:1221-1231`)
  après la livraison réussie ; bypass DND des réponses **uniquement après
  validation non mutante** de la demande référencée (état ouvert,
  destinataire = expéditeur de la réponse, émetteur = destinataire de la
  réponse) — un `in_reply_to` forgé ou incohérent est traité en message
  ordinaire. Supprimer la branche stderr `wrapper.rs:822` (FR-013,
  `DEPRECATIONS.md`). Vérifier que `content_key()` (`message.rs:100`)
  n'écrase pas une réponse légitime par déduplication — test dédié (risque 1
  du reuse-audit).
  **Observable** : quickstart §1-§3 passent avec un équipier Codex réel ;
  demande suivie close dans le ledger sans commande exécutée par l'équipier ;
  tests : réponse refusée (dedup/disjoncteur) ne clôt pas la demande, réponse
  légitime vers un émetteur en DND livrée, `in_reply_to` inconnu ou aux
  participants incohérents → DND non contourné et demande inchangée,
  `CancelDelivery` purge le bon message, rafale > capacité → `DeliveryRejected`
  et échec motivé chez l'émetteur, mort du processus avec N messages en file →
  N échecs motivés.

- [X] **T706** [US1] **Journal de session JSONL — schéma versionné v1** :
  `~/.cache/bridget/sessions/<agent>/<date>.jsonl`, **schéma v1 complet de
  `data-model.md`** (enrichi sur exigence de la contre-revue 008 : `v`, `seq`
  strictement croissant traversant la rotation, `session_id`, `payload` typé
  par événement avec expéditeur dans `turn_start.from`), écriture ligne entière
  + flush, append-only, un fichier par jour.
  **Observable** : quickstart §7 ; tests unitaires — append, rotation par date
  avec continuité de `seq`, **récupération du prochain `seq` après
  redémarrage** (cas : dernier fichier normal, vide, ligne finale partielle,
  ligne finale corrompue), permissions 0700/0600 vérifiées, payloads conformes
  par type (`turn_start.body` complet) ; **fixtures de compatibilité lecteur
  versionnées** (tour complet, tour en erreur, permission, rotation, dernière
  ligne partielle, ligne corrompue) destinées aux tests de la session 008.

**Checkpoint MVP** : à la fin de la phase 1, la User Story 1 est démontrable de
bout en bout (SC-001, SC-002).

---

## Phase 2 : US2 — Ouverture du registre (Claude, puis type inconnu du code)

- [X] **T707** [US2] **Équipier Claude** : valider l'entrée `claude` du registre
  par défaut (`@zed-industries/claude-code-acp@0.16.2`) en déroulant quickstart
  §1-§3 avec un équipier Claude. **Capturer la fixture Claude de couche 2**
  (formes filaires réellement observées : tour nominal, notifications propres
  au harness, chunks — la couche 1 des invariants JSON-RPC est prouvée par les
  fixtures génériques, matrice à deux couches de research R-004), sans
  régression Codex.
  **Observable** : échange complet Claude (demande suivie → réponse → clôture) ;
  matrice de conformité au vert pour les deux adaptateurs.

- [X] **T708** [US2] **Test d'acceptation d'ouverture — type inconnu sans
  code de production** *(révisée le 2026-08-22, gate de support : Gemini
  indisponible pour les comptes individuels — deux tentatives réelles 0.46.0
  et 0.56.0, constat versionné en `0800d0a`)* : test d'intégration automatisé
  — fixture de registre versionnée ne contenant initialement que Codex/Claude,
  ajout dynamique d'une entrée de type **inconnu du code de production**
  pointant vers l'adaptateur de test stdio (T704), lancement de l'équipier via
  ce fichier, échange complet (livraison → tour → réponse → clôture).
  L'entrée `gemini` reste au registre par défaut, documentée indisponible
  (comptes individuels) dans research.md et README.
  **Observable** : SC-003 révisé prouvé — le test d'intégration vit sous
  `crates/*/tests/` (ou diff `.rs` strictement limité à `cfg(test)`), et un
  **contrôle mécanique** vérifie qu'aucune ligne hors test/fixture n'a changé
  (`git diff` filtré, consigné) ; l'échange passe de bout en bout via le
  registre. Si un changement de production s'avère nécessaire, la tâche échoue
  (défaut d'ouverture → retour T703/T704).

---

## Phase 3 : US3 — Relances informées et facturation protégée

- [X] **T709** [US3] **État de tour et échecs motivés** : variante légère
  `WrapperToDaemon` d'état de tour (arbitrage reuse-audit), transitions
  remontées à chaque début/fin de tour ; dans `daemon.rs` : relance différée si
  tour en cours avec événement consigné (FR-008), échec motivé vers l'émetteur
  sur mort du processus, `stopReason` d'erreur ou dépassement de l'échéance de
  la demande (FR-009, D-206 : le daemon reste l'autorité unique du cycle de
  vie ; le transport n'annule que le tour actif, timeout de transport
  configurable pour les notifications), état annuaire selon la table de
  transitions de `data-model.md` (`busy`/`connected`/`stopped`/`unreachable`,
  re-déclaration de l'état de tour au `Register` après reconnexion).
  **Observable** : tests unitaires daemon — relance différée pendant un tour
  (SC-004), relance normale sur équipier inactif, échec motivé sur les trois
  causes, **reconnexion wrapper pendant un tour sans perte de l'état `busy`,
  arrêt propre → `stopped` (pas `unreachable`), pas de double signalement
  d'expiration daemon/transport** ; quickstart §4.

- [X] **T710** [US3] **Garde de facturation** : au lancement d'un équipier,
  refus si une variable de `forbidden_env` du registre est présente, message en
  français nommant la variable et le contournement `BRIDGET_ALLOW_API_KEY=1`
  (`wrapper.rs`, D-207).
  **Observable** : quickstart §5 ; tests unitaires refus/contournement (SC-005).

---

## Phase 4 : Finition

- [X] **T711** Non-régression locale et documentation : suite `cargo test`
  complète + agents tmux réels inchangés (SC-006) ; section « Équipiers ACP »
  dans `README.md` et `README.en.md` ; `DEPRECATIONS.md` relu — aucun chemin
  hérité non étiqueté (SC-007) ; `implementation.md` complet ; `bridget who`
  montre le transport `acp`.
  **Observable** : checklist SC-001..SC-007 (hors FR-014) pointée une à une
  dans `implementation.md` avec preuve.

- [X] **T712** **Validation fédération (gate FR-014)** : équipier ACP joignable
  à travers la fédération SSH — réutiliser `scripts/test-federate-ssh.sh`
  (boucle SSH locale) ou une machine distante réelle, et dérouler l'échange
  complet (quickstart §2) vers l'équipier distant.
  **Observable** : échange fédéré réussi et consigné. **Cette tâche ne peut pas
  être « passée » : sans environnement disponible, elle reste non cochée et la
  session reste `In Progress`** (round 2 de la contre-revue, objection 6 — un
  « non vérifié : raison » ne satisfait pas FR-014).
