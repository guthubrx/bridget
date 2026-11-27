# Contre-revue adverse du plan (Spec 314) — Claude (`claude-fable-5-2`)

- **Date** : 2026-11-27
- **Agent / fournisseur** : `claude-mac` (`claude-fable-5-2`, `task_id=084c9ead-1897-4455-b032-16abe068c3c9`, `posture=discovery`, lecture seule)
- **Question posée** : Challenger `spec.md`, `plan.md`, `data-model.md`, `reuse-audit.md` contre `native_graphs.rs`, `remote_worker.rs` et `mcp.rs` (défauts logiques, sécurité `gate`, interaction `when`/`on_failure`, escalade, rollup SQLite Time-Machine, budget MCP compact ≤ 550 B).
- **Verdict initial reçu** : `BLOCKED` (6 P0, 8 P1, 4 P2 — tous vérifiés et intégrés avant `tasks.md`).

| # | Objection | Vérifiée comment | Retenue | Résolution appliquée dans `spec.md` / `plan.md` / `data-model.md` |
|---|---|---|---|---|
| 1 | `cancel_graph` annule une branche de repli avant son lancement (`native_graphs.rs:578-587`) | Relu `native_graphs.rs:578-587` (confirmé) | Oui | Interdire `on_failure=cancel_graph` combiné avec `when != all_succeeded` à la validation (`graph_invalid`). |
| 2 | Interblocage après `skipped_condition_not_met` (`native_graphs.rs:611-639`) | Relu `native_graphs.rs:611-674` (confirmé) | Oui | Ajouter `skipped_condition_not_met` à `step_is_terminal` et au point fixe de propagation ; documenter hors-périmètre V1 la jonction nominal + repli en `all_succeeded`. |
| 3 | `stop_dependents` saute aussi les étapes de repli ; `cancelled` ne doit pas déclencher de repli | Relu `native_graphs.rs:624-628` (confirmé) | Oui | Filtrer `skipped_dependency_failed` sur `when == all_succeeded` ; exclure `cancelled` des `when_outcomes` et de `any_failed`. |
| 4 | Sécurité `RunDeterministicGate` (posture `discovery`, env, process group, pipe plein) | Vérifié `NFR-01` et contraintes POSIX (confirmé) | Oui | Exiger posture parent `development` pour `gate`, `env_clear()` + PATH minimal sans variables `BRIDGET_*`, `stdin=null`, `setpgid(0,0)` + `kill(-pgid, SIGKILL)` au timeout, lecture continue bornée par threads lecteurs. |
| 5 | Cohérence `HEAD == producer_commit` et arbre propre avant/après `gate` | Vérifié séparation worktree vs `commit_view` (confirmé) | Oui | Vérifier `HEAD == producer_commit` et `git status --porcelain` vide avant `gate`, et `HEAD` inchangé après ; sinon `gate_unavailable`. |
| 6 | Double comptage métriques après rollup si `metrics_recorded` écrit hors transaction | Vérifié `delegation.rs` + `plan.md` (confirmé) | Oui | Écrire `convergence_metrics_raw` et `save_graph` (`metrics_recorded=true`) dans une seule transaction SQLite atomique. |
| 7 | `NULL` dans clés primaires SQLite `daily`/`monthly` casse `ON CONFLICT` | Comportement SQLite standard (confirmé) | Oui | Toutes les colonnes des clés primaires `daily`/`monthly` en `TEXT NOT NULL DEFAULT ''`. |
| 8 | Rollup partiel : `MIN`/`MAX` + ensemble identique agrégé/supprimé | Vérifié SQL rollup (confirmé) | Oui | Agréger/supprimer via table temporaire de clés `(graph_id, step_id)` dans la transaction, avec `ON CONFLICT DO UPDATE` (somme, `MIN`, `MAX`). |
| 9 | Politique incohérente `monthly_retention_months * 31 <= daily_retention_days` | Vérifié bornes `data-model.md` (confirmé) | Oui | Ajouter invariant validé `daily_retention_days >= raw_retention_days` et `monthly_retention_months * 31 > daily_retention_days`. |
| 10 | Purge `native_delegation_graphs` libère `(owner, request_id)` + orphelins `graph_tasks` | Relu `delegation.rs:1450` (confirmé) | Oui | Minimum `graph_retention_days = 7` (défaut 30), purger uniquement graphes terminaux avec `notice_sent = 1`, et supprimer `native_delegation_graph_tasks` dans la même transaction. |
| 11 | Escalade : base Git, `request_id` déterministe par palier, annulation palier 1 | Relu `native_graphs.rs:592-602` (confirmé) | Oui | Clé déterministe incluant le palier `t{tier}`, `prior_producer_task_ids` suivis pour `cancel_graph`/`handle_cancel`, ancrage `expected_base_commit = last_producer_commit`. |
| 12 | Empreinte `gate` normalisée + incohérence `stop_on_stagnation=false` avec `escalation.on_stagnation=true` | Vérifié logique empreinte (confirmé) | Oui | Préfixer `gate:{exit_code}:` + normalisation des nombres/durées volatiles ; refuser `stop_on_stagnation=false` avec `escalation.on_stagnation=true`. |
| 13 | Porte bloquante dans `tick()` et non rejouable après redémarrage | Relu `native_graphs.rs:1573-1850` (confirmé) | Oui | Exécuter la porte hors verrou dans `GraphLaunchJob::ConvergenceGate` (thread détaché) avec `launch_key` par `(graph, step, tier, iter)`. |
| 14 | Flotte mixte (ancien worker ne connaît pas `RunDeterministicGate`) | Vérifié protocole worker (confirmé) | Oui | Convertir tout refus `InvalidEnvelope`/décodage worker en `gate_unavailable` fail-closed explicite. |
| 15 | Deux formes incompatibles pour `when` | Relu `spec.md` vs `data-model.md` (confirmé) | Oui | Forme unique plate : `when: "all_succeeded" | "any_failed" | "on_outcome"` + `when_outcomes: Vec<String>`. |
| 16 | Vocabulaire des issues (`escalated` vs escalade palier, `gate_failed`, `escalate` palier 0) | Relu `native_graphs.rs:1406` (confirmé) | Oui | Renommer le booléen d'escalade de modèle en `tier_escalated` ; `verdict=escalate` court-circuite toujours l'escalade de palier ; liste `when_outcomes` fermée sans `cancelled` ni `gate_failed`, avec `gate_unavailable`. |
| 17 | Budget MCP compact `<= 550 B` à prouver par mesure | Relu `mcp.rs:2947-2958` (confirmé) | Oui | Indice compact court `[{step_id,depends_on?,when?,delegation,convergence?:{verify,gate?,escalation?}}]` vérifié par test strict `<= 550 B`. |
| 18 | État final du graphe quand repli réussit après échec nominal | Relu `native_graphs.rs:640-651` (confirmé) | Oui | État global conservé `partial_failure` (rétrocompatible) + champs explicites `fallback_recovered: true` et `counts.fallback_steps_completed`. |

---

## Passe 2 — Contre-revue adverse du code implémenté (Claude `0d38a586-0fee-4cfa-ae4b-12cb241a1303`)

**Résultat initial** : `0 P0`, `5 P1`, `12 P2`.
**Traitement** :

| Constat | Sévérité | Statut | Action réalisée |
|---|---|---|---|
| `P1-1` : revérification `git_worktree_is_clean` après exécution de la porte | P1 | **Corrigé + testé** | `git_worktree_is_clean(dir)` appelé après `git_head_sha` -> `gate_worktree_mutated` + test dédié dans `deterministic_gate.rs`. |
| `P1-2` : `kill(-pgid, SIGKILL)` après sortie normale + lecture bornée des tubes | P1 | **Corrigé + testé** | `kill_group()` systématique en sortie normale et erreur + `recv_timeout(2s)` sur stdout/stderr + test petit-enfant `sleep 30` héritant de stdout. |
| `P1-3` : annulation tâche orpheline dans `GraphLaunchJob::StepStart` | P1 | **Corrigé** | Ajout de la branche `else` annulant `orphan_tid` via `cancel_task_for_graph` dans `native_graphs.rs`. |
| `P1-4` : perte de mise à jour par relâchement de verrou dans `StepStart` | P1 | **Corrigé** | `apply_admission_response_locked` exécuté intégralement sous le même verrou `DaemonState` que la lecture et la persistance. |
| `P1-5` : contrôle d'accès principal humain local sur `GraphMetrics` / `GraphMetricsPolicy` | P1 | **Corrigé** | `require_local_control_principal(&st, conn)` (même garde que `Grant` : `ConnectionRole::Client` + `ControlStateV1` + hors agent connecté). |
| `P2-3` : `try_wait` `Err(e)` fail-closed | P2 | **Corrigé** | `unavailable_reason = Some("gate_wait_failed:...")`. |
| `P2-4` : fausse stagnation après escalade de palier | P2 | **Corrigé** | Repli `last_producer_commit` restreint aux anciens enregistrements Spec 313 (`!tier_escalated && seen_commits.is_empty()`). |
| `P2-5` : orthogonalité fausse si famille producteur inconnue | P2 | **Corrigé** | `if prod_fam.is_empty() { orthogonal_flag = Some(false); }`. |
| `P2-8` : `timeout_secs` inclus dans la clé de dédup worker | P2 | **Corrigé** | `json_digest(&(execution, expected_commit, argv, timeout_secs))` dans `remote_worker.rs`. |
| `P2-9` : isolation d'environnement sur les appels `git` de la porte | P2 | **Corrigé** | `sanitize_env_for_gate` appliqué à `git_head_sha` et `git_worktree_is_clean` (neutralise `GIT_DIR`/`GIT_WORK_TREE`). |
| `P2-10` : libération RAII de `graph_launching_steps` sur panic | P2 | **Corrigé** | `LaunchKeyGuard` dans `native_graphs.rs`. |
| `P2-1,2,6,7,11,12` | P2 | **Actés** | Limites documentées ou hors périmètre Spec 314 (ex. `P2-12` préexistant Spec 313). |
