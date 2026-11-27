# Plan d'implémentation 313 — Graphes de convergence bornée et découvrabilité MCP

## 1. Vue d'ensemble architecturale

L'extension s'insère directement dans le moteur existant `bridget_delegate_graph` sans créer de nouvel outil MCP, de nouvelle table SQLite ni de nouveau daemon :

1. **Protocole (`crates/bridget-transport/src/protocol.rs`)** :
   - Ajout de `GraphConvergenceLoop` (`verify: AutoDelegateV2`, `max_iterations: u8`, `stop_on_stagnation: bool`) et du champ optionnel `convergence: Option<GraphConvergenceLoop>` sur `DelegateGraphStep`.
   - Constantes partagées `MIN_CONVERGENCE_ITERATIONS = 2`, `MAX_CONVERGENCE_ITERATIONS = 5`, `DEFAULT_CONVERGENCE_ITERATIONS = 3`.
2. **Stockage durable (`crates/bridget-daemon/src/delegation.rs`)** :
   - Extension additive (`#[serde(default)]`) de `NativeGraphStepRecord` avec `convergence: Option<GraphConvergenceLoop>` et `convergence_state: Option<NativeGraphConvergenceState>` (`current_iteration`, `phase`, `producer_task_id`, `producer_expected_round`, `verifier_task_id`, `outcome`, `iterations: Vec<NativeGraphConvergenceIteration>`).
   - Aucune migration SQLite requise : `native_delegation_graphs` stocke le payload JSON sérialisé de `NativeGraphRecord`.
3. **Moteur de convergence (`crates/bridget-daemon/src/daemon/native_graphs.rs`)** :
   - Validation DAG + bornes `convergence` dans `validate_and_order_graph`.
   - Parseur déterministe `parse_convergence_verdict(&str)` (lecture depuis la fin du dernier objet JSON fermé `verdict ∈ {approve, revise, escalate}`, tolérance aux blocs Markdown, empreinte SHA-256 normalisée des `blocking_points` + `reason`).
   - Machine d'état par étape de convergence pilotée dans `handle_submit`, `sync_steps_from_tasks` et `tick` :
     - `waiting_dependencies` → (racine ou dépendances terminées) → `launching` (phase `producing`, itération 1, `keep_alive_secs >= 900`).
     - Producteur `result_available` (avec `task.round >= producer_expected_round`) → enregistrement du `producer_commit` et détection stagnation commit (si `k >= 2`) → transition vers phase `verifying`, lancement asynchrone de `convergence.verify` (ancré sur le dernier `producer_commit` si applicable, préfixé par les faits du tour + le contrat de verdict fermé).
     - Vérificateur `result_available` → `parse_convergence_verdict` :
       - `Approve` → étape `completed` (`outcome = "approved"`), `result_excerpt` + `factual_summary` propagés aux étapes aval ;
       - `Escalate` → étape `failed` (`outcome = "escalated"`, `error = "convergence_escalated: ..."`) ;
       - `Revise` → si `stop_on_stagnation` et même `blocking_fingerprint` qu'au tour précédent → `failed` (`outcome = "stagnated"`) ; sinon si `current_iteration >= max_iterations` → `failed` (`outcome = "max_iterations_reached"`) ; sinon incrément `current_iteration += 1`, phase `producing`, reprise du producteur via `NativeDelegationRequest::Continue` sur `producer_task_id` (avec repli automatique sur `AutoDelegateV2` ancré sur le dernier `producer_commit` si `Continue` est refusé) ;
       - Verdict absent/invalide → `failed` (`outcome = "verdict_unparseable"`).
   - Exposition dans `graph_view` (`convergence_state`) et dans `format_graph_summary_message` (issue + nombre de tours).
   - Annulation (`handle_cancel` et `CancelGraph`) : annule `task_id` actif (`producer_task_id` ou `verifier_task_id` en cours).
4. **Pré-validation et schémas MCP (`crates/bridget-daemon/src/delegation_mcp.rs` + `crates/bridget-daemon/src/mcp.rs`)** :
   - Refactoring minimal de `parse_delegate_graph` : extraction de `parse_auto_delegate_step_payload` réutilisée pour `step.delegation` et `step.convergence.verify` (Article XIX : élimine la duplication et applique `parse_workspace` + `parse_brief` + 40-hex SHA + `commit_view` aux deux !).
   - Pré-validation locale de la topologie DAG (unicité `step_id`, existence des `depends_on`, absence de cycle par Kahn) et de `convergence` (`max_iterations ∈ 2..=5`, champs fermés) avant toute connexion.
   - Enrichissement du schéma JSON complet (`delegation_mcp.rs`) et du schéma compact Antigravity (`mcp.rs` : `description` claire sur `steps`, aucune `properties` imbriquée sur `object`).
5. **Adoption — Skills & Documentation** :
   - `/Users/moi/.agents/skills/bridget/SKILL.md` : section concise sur les graphes DAG et les boucles de convergence bornée + matrice de décision + exemple minimal.
   - `/Users/moi/.agents/skills/graph_it/SKILL.md` : lien opérationnel vers `bridget_delegate_graph` (et `convergence`).
   - `docs/delegation-native.fr.md` et `docs/delegation-native.md` : documentation complète du champ `convergence`, du verdict fermé, de la stagnation et des états de sortie.

## 2. Contrôle constitutionnel (Articles XVIII, XIX, XX)
- **Article XVIII (Complexité)** : $O(V+E)$ ($V \le 16$) pour Kahn, $O(L)$ sur la sortie vérificateur, zéro polling, zéro boucle SQL N+1.
- **Article XIX (Minimalisme)** : 0 nouvel outil MCP, 0 nouvelle table SQLite, 0 nouvelle dépendance Cargo ; réutilisation intégrale de `AutoDelegateV2`, `NativeDelegationRequest::Continue`, `parse_workspace`, `parse_brief`, `DelegationStore::save_graph`.
- **Article XX (Charge cognitive & explicabilité)** : contrat fermé à 3 verdicts (`approve`, `revise`, `escalate`), états explicites dans `bridget_graph_status`, aucune heuristique floue sur texte libre.
- **Branche unique (`AGENTS.md`)** : développement direct sur `main` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget`, builds via `scripts/build.py` / `make release`, déploiement 5 hôtes via `scripts/deploy-fleet.py`.
