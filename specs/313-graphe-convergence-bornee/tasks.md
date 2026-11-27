# Tâches — 313 : Graphes de convergence bornée et découvrabilité MCP

## Lot 1 — Protocole et modèle durable (Fondations)

- [x] **T001** — Ajouter `GraphConvergenceLoop`, `MIN_CONVERGENCE_ITERATIONS` (`2`), `MAX_CONVERGENCE_ITERATIONS` (`5`), `DEFAULT_CONVERGENCE_ITERATIONS` (`3`) et le champ rétrocompatible `convergence: Option<GraphConvergenceLoop>` sur `DelegateGraphStep` dans `crates/bridget-transport/src/protocol.rs`.
  - *Résultat observable* : `bridget-transport` compile, les anciens JSON sans `convergence` désérialisent avec `None`, et les champs inconnus sont refusés (`deny_unknown_fields`).
- [x] **T002** — Étendre `NativeGraphStepRecord` avec `convergence: Option<GraphConvergenceLoop>`, `convergence_state: Option<NativeGraphConvergenceState>` et `NativeGraphConvergenceIteration` (`#[serde(default)]`) dans `crates/bridget-daemon/src/delegation.rs`.
  - *Résultat observable* : les anciens enregistrements JSON SQLite `native_delegation_graphs` restent désérialisables sans migration destructive.

## Lot 2 — Moteur de convergence bornée (US-1, US-2, US-3)

- [x] **T003** — Étendre `validate_and_order_graph` dans `crates/bridget-daemon/src/daemon/native_graphs.rs` pour valider `convergence` (`max_iterations ∈ 2..=5`, `verify.task` non vide ≤ 65 536 octets, `verify.cwd` non vide, `verify.expected_base_commit` 40 hex si présent).
  - *Résultat observable* : une borne invalide (`1`, `6`) ou un `verify` vide est refusé avant persistance.
- [x] **T004** — Implémenter `parse_convergence_verdict`, la normalisation d'empreinte SHA-256 (`blocking_fingerprint`), la détection de stagnation (commit identique ou `blocking_fingerprint` identique à partir du tour 2), le préambule vérificateur (faits + contrat JSON fermé) et le préambule de reprise producteur dans `crates/bridget-daemon/src/daemon/native_graphs.rs`.
  - *Résultat observable* : `approve`, `revise`, `escalate` sont parsés depuis la fin (y compris sous bloc ```` ```json ````) ; toute sortie sans verdict JSON fermé renvoie `Unparseable` (fail-closed).
- [x] **T005** — Intégrer le cycle `producing ↔ verifying` dans `handle_submit`, `sync_steps_from_tasks`, `tick`, `handle_cancel`, `graph_view` et `format_graph_summary_message` dans `crates/bridget-daemon/src/daemon/native_graphs.rs` :
  - forcer `keep_alive_secs >= 900` (borné à `7200`) sur le producteur au tour 1 ;
  - propager le `producer_commit` vers `verify` (`expected_base_commit` / `worktree.base_ref` lorsque pertinent) ;
  - relancer le producteur au tour `k >= 2` via `NativeDelegationRequest::Continue` (`producer_expected_round = k`), avec repli automatique sur `AutoDelegateV2` ancré sur le dernier commit si `Continue` est refusé ;
  - arrêter immédiatement sur `approve`, `escalate`, `stagnated`, `max_iterations_reached` ou `verdict_unparseable`, et exposer l'historique par itération dans `bridget_graph_status` et dans le bilan final.
  - *Résultat observable* : un graphe `A → B (convergence) → D` exécute la boucle jusqu'à verdict puis déclenche `D` ou propage `on_failure`.
- [x] **T006** — Écrire les tests unitaires complets dans `crates/bridget-daemon/src/daemon/native_graphs.rs` (`mod tests`) couvrant : validation `convergence`, parsing verdict (`approve`/`revise`/`escalate`/Markdown/fail-closed), stagnation commit & motifs, cycle complet 2 tours `revise → approve` débloquant l'étape aval, escalade immédiate, plafond `max_iterations`, et annulation en phase `producing`/`verifying`.
  - *Résultat observable* : tous les tests `daemon::native_graphs::tests` passent via `scripts/build.py cargo test`.

## Lot 3 — Pré-validation locale MCP et schémas complet/compact (US-4)

- [x] **T007** — Factoriser `parse_auto_delegate_object` et étendre `parse_delegate_graph` dans `crates/bridget-daemon/src/delegation_mcp.rs` pour pré-valider localement `steps`, `depends_on`, absence de cycle DAG (Kahn), `convergence` (`verify`, `max_iterations`, `stop_on_stagnation`), `workspace` et `brief` avant connexion au daemon (sans consommer `request_id`), avec exemple minimal dans les messages d'erreur ; mettre à jour le schéma JSON complet de `bridget_delegate_graph`.
  - *Résultat observable* : toute erreur sur `steps`, cycle ou `convergence` renvoie `ClientError::InvalidParams` sans ouvrir de socket.
- [x] **T008** — Mettre à jour la projection compacte Antigravity de `bridget_delegate_graph` dans `crates/bridget-daemon/src/mcp.rs` (`compact_tool_description` + `compact_field_hint` sur `steps`, sans `properties` imbriquées sur `object`) et compléter les tests dans `crates/bridget-daemon/src/delegation_mcp.rs`, `crates/bridget-daemon/src/mcp.rs` et `crates/bridget-daemon/src/mcp_tests.rs`.
  - *Résultat observable* : `compact318_tests`, `spec309_compact_discovery_tools` et `delegation_mcp::` passent ; `bridget mcp` compact reste sous budget avec description explicite de `steps` et `convergence`.

## Lot 4 — Adoption : Skills et documentation (US-5)

- [x] **T009** — Documenter `bridget_delegate_graph` (DAG + convergence bornée, matrice de choix, contrat de verdict fermé, exemple JSON) dans `/Users/moi/.agents/skills/bridget/SKILL.md` et relier `/Users/moi/.agents/skills/graph_it/SKILL.md` à `bridget_delegate_graph`.
  - *Résultat observable* : un agent lisant `bridget/SKILL.md` ou `graph_it/SKILL.md` dispose des clés exactes et du contrat complet.
- [x] **T010** — Mettre à jour `docs/delegation-native.fr.md` et `docs/delegation-native.md` avec le contrat `convergence`, les états d'arrêt et le suivi `bridget_graph_status`.
  - *Résultat observable* : documentation FR/EN alignée sur le code livré.

## Lot 5 — Vérification, livraison flotte et nettoyage

- [x] **T011** — Lancer les tests ciblés (`bridget-transport`, `daemon::native_graphs`, `delegation_mcp`, `mcp::compact318_tests`, `spec309_compact_discovery_tools`), exécuter Converge + Audit + contre-revue adverse finale.
- [ ] **T012** — Committer sur `main`, pousser vers `github` et `origin` (avec vérification `git ls-remote`), lancer les builds release déportés (`macos-arm64` sur `macbook-m1`, `linux-x86_64-glibc2.36` sur `sysle2`), déployer sur les 5 hôtes (`scripts/deploy-fleet.py`), vérifier `bridget mcp` compact + `bridget status` en direct, prévenir les orchestrateurs, puis nettoyer (`target/`, `menage.py`).
