# Tasks : outils MCP Bridget (010)

**Statut** : PROJET — exécutable après (a) le spike-gate D-403 (kit prêt sous
`spike/`, issue consignée exigée) et (b) la clôture des séries amont dont la
010 consomme les livrables (socle 012 pour l'idempotence de `bridget_send`,
registre 007). **Prerequisites** : spec (2 rounds), plan (round 3, D-401..
D-405 + amendement), contrat outils-mcp.md, quickstart.
**Base prévisionnelle** : branche `session-10-mcp` depuis la 009 finale
(ordre de fondation : 012 → 009 → 010).

**Règles** : identiques aux séries précédentes (validations avant commit,
commit en review immuable, auteur ≠ relecteur, zéro trace IA, jamais de push).

---

## Phase 0 — Gate et fondations

- [x] **T1001** **Spike-gate de branchement (D-403/FR-012)** : exécuter
  `spike/protocole-spike.md` avec `spike/fake-mcp-server.py` (auto-testé) sur
  les quatre voies, versions pinnées, consignation complète (commandes,
  `PROBE_OK`, diff de configs VIDE, nettoyage) dans `implementation.md`.
  Gemini : consigner le statut hérité du constat T708.
  **Observable** : issue de gate explicite — 4/4 ou 3/4-avec-Gemini-documenté
  → GO ; sinon STOP et révision de spec avant toute suite.

- [x] **T1002** Setup + ADR : worktree, copie des artefacts,
  `docs/decisions/008-serveur-mcp.md` (façade du protocole daemon, deux
  mondes stdio/socket, pureté stdout).
  **Observable** : ADR auto-portant.

## Phase 1 — Serveur et identité

- [x] **T1003** `mcp.rs` — dispatcher stdio : sous-commande `bridget mcp`,
  lecteur stdin unique + corrélation d'ids (chaîne/nombre), writer stdout
  sérialisé, `initialize` (version pinnée, capacité `tools` seule),
  `initialized`, `tools/list` idempotente rappelable, `ping`, erreurs
  JSON-RPC (méthode inconnue, `tools/call` avant `initialized`), pureté
  stdout absolue (logs stderr).
  **Observable** : matrice de conformité FR-009 (15 cas) en fixtures ;
  golden test « aucun octet non-JSON-RPC sur stdout ».

- [x] **T1004** Identité d'appelant (FR-004) : résolution À CHAQUE appel —
  fichier de nom courant (`BRIDGET_AGENT_NAME_FILE`) → filiation
  `agent-pids/` **typée** (marqueur enrichi : pid + naissance capturée au
  dépôt + `instance_id` + chemin du nom ; wrapper mis à jour pour l'écrire ;
  ancien format nom-nu → erreur `legacy_marker` avec remédiation) → erreur
  explicite. Marche bornée des ancêtres, premier validé gagne.
  **Observable** : tests — rename entre deux appels, deux agents même
  binaire, chaîne npx ×3, pid recyclé (naissance divergente), hors-agent,
  marqueur legacy.

- [x] **T1005** Outils `bridget_send`/`bridget_who`/`bridget_ledger` :
  connexion daemon PAR APPEL (deux phases Register/commande, budget 10 s du
  registre), `bridget_send` **projection du contrat 012** (id métier avant
  connexion via `SendIdempotent` quand le client MCP négocie —
  `outcome_unknown` distinct, retry même id dédupliqué) ; taxonomie FR-011
  (JSON-RPC / `isError` / métier fermé+extensible) ; `bridget_ledger(view:
  messages|requests|both)` sur la couche de lecture **extraite en module
  neutre** (D-404 : `cli.rs` garde ses renderers, golden tests de sortie
  binaire octet pour octet) ; limite d'appels simultanés → `busy`.
  **Observable** : corpus SC-001 (corps riches intacts), SC-002 (4 familles
  de refus en résultat métier), SC-006 (ledger identique outil vs binaire),
  saturation → `busy`, daemon coupé → `isError` immédiat.

## Phase 2 — Branchement et prompt

- [x] **T1006** Branchement déclaratif (FR-005/FR-012) : champ `mcp` au
  registre (schéma étendu), injection **éphémère par session** selon la voie
  validée au spike pour chaque type (mcpServers ACP pour les équipiers ;
  formes CLI constatées pour les interactifs) — **jamais d'écriture de config
  utilisateur** (diff vide vérifié en test).
  **Observable** : SC-004 selon l'issue de la gate T1001, smoke test réel par
  type disponible.

- [x] **T1007** Prompt allégé (FR-006/SC-005) : blocs avant/après versionnés
  en fixtures, réduction ≥ 60 % (comptage Unicode documenté), matrice
  comportementale = quickstart 007 §1-§4 rejoués avec le prompt réduit.
  **Observable** : mesure consignée + matrice au vert.

## Phase 3 — Finition

- [ ] **T1008** Non-régression totale (007/008/009/012), README (« outils
  MCP »), `implementation.md` complet, checklist SC-001..SC-006 pointée.
  **Observable** : checklist avec preuves ; suite complète au vert.
