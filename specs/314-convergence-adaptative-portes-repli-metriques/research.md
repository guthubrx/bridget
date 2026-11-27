# Recherche technique — Spec 314

## 1. Synthèse état de l'art (2025–2026)
- **Portes déterministes avant vérificateur LLM** : les boucles d'auto-évaluation purement LLM échouent par rationalisation du code erroné ; la discipline industrielle sépare génération, contrôle déterministe (compilation/tests/lint à 0 token faisant foi) et vérification sémantique indépendante (conseil sur intention/architecture, incapable de contourner une porte rouge).
- **Sessions collantes + escalade sur couture** : changer de modèle à chaque tour détruit le *prompt caching* et augmente le coût ; la stratégie optimale garde la session active (`Continue`) dans le palier initial et n'escalade vers un modèle supérieur qu'au point de rupture (stagnation, oscillation ou plafond du palier 1), avec un contexte compressé (dernier commit + blocages).
- **Détection d'oscillation (`A → B → A`)** : comparer uniquement le tour `n` au tour `n-1` laisse passer les allers-retours entre deux états déjà rejetés ; mémoriser l'ensemble des empreintes `(commit, blocking_fingerprint)` du palier stoppe immédiatement le *churn*.
- **Orthogonalité cross-fournisseurs** : un vérificateur d'une autre famille que le producteur réduit les angles morts corrélés (*agreeableness bias*).
- **Rétention multi-paliers façon Time Machine (RRD / downsampling)** : conserver le détail brut à court terme (`raw`), compacter en agrégats journaliers (`daily`) puis mensuels (`monthly`) avant purge finale garantit une empreinte SQLite bornée dans le temps sans perdre les tendances de long terme.

## 2. Hypothèses techniques vérifiées sur le code réel de Bridget

### H1 — Exécution de la porte déterministe sur worker distant sans ouvrir de brèche sécurité
- **Preuve code** : `crates/bridget-daemon/src/remote_worker.rs:6586-6675` (`defer_view_file` / `poll_view_files`) et `crates/bridget-daemon/src/daemon/native_delegation.rs:8434-8480` (`remote_worker_roundtrip_with_send_deadline`).
- **Résolution** : Le worker identifie déjà le worktree attesté d'une exécution via `WorkerResultIdentity`. `RunDeterministicGate` réutilise exactement cette preuve d'exécution, lance `Command::new(&argv[0]).args(&argv[1..])` (aucun shell) dans ce worktree, dans un thread détaché surveillé avec timeout dur (`5..=300 s`) et capture bornée à `8 192` octets.

### H2 — Budget schéma MCP compact Antigravity ACP (`<= 550 B`)
- **Preuve code** : `crates/bridget-daemon/src/mcp.rs:2947-2958` et test `spec309` (`<= 550 B` par outil suite au commit `f6758227`).
- **Résolution** : En mode compact, `bridget_delegate_graph` a actuellement ~435 octets. En portant `steps` hint à `[{step_id,depends_on?,when?,delegation,convergence?:{verify,gate?,escalation?,max_iterations?:2..5}}]` (+30 octets), l'outil reste sous ~475 octets (`< 550 B`).

### H3 — Entretien SQLite horaire déjà présent
- **Preuve code** : `crates/bridget-daemon/src/daemon/maintenance.rs:157` appelle `native_delegation::maintain_retention(st, now)` toutes les heures (`MAINTENANCE_INTERVAL = 3600s`).
- **Résolution** : Brancher `rollup_and_purge_convergence_metrics` et `purge_expired_graphs` directement dans `maintain_retention` évite de créer un nouveau thread ou timer (Article XIX).

### H4 — Sélection d'un moteur orthogonal (`different_provider: true`)
- **Preuve code** : `crates/bridget-daemon/src/daemon/native_delegation.rs` observe les candidats disponibles par projet via le catalogue/flotte (`observe_routing_candidates_for_project` / `engine`).
- **Résolution** : Lors du lancement du vérificateur, si `different_provider == true` et `verify.engine.is_none()`, lire l'`engine` effectif de la tâche productrice et choisir un autre `engine` présent parmi les candidats ouverts (avec quota non épuisé) ; si aucun autre moteur n'est ouvert, laisser le routage normal choisir et consigner `verifier_orthogonal = false` dans l'itération et les métriques.
