# Spécification 314 — Convergence adaptative : portes déterministes, escalade en cascade, branches de repli DAG et métriques SQLite à rétention multi-paliers

**Date** : 2026-11-27
**Statut** : En cours (`my-specify-all`, branche unique `main`)
**Dépendances** : Spec 313 (`bridget_delegate_graph` + convergence bornée dans `native_graphs.rs`), `WorkerRequestOperation` (`remote_worker.rs`), `maintain_retention` (`maintenance.rs` / `native_delegation.rs`), schémas MCP compacts ≤ 550 octets (`mcp.rs`, commit `f6758227`).

---

## 1. Demande initiale, cadrage `$be-ambitious` et exigences validées

### Demande utilisateur initiale
- Doter Bridget du cran au-dessus de la Spec 313 :
  1. **Portes déterministes** (contrôle à 0 token avant le vérificateur LLM), exécutées par `argv` strict sans shell libre dans le worktree isolé du producteur (option 1 validée par l'utilisateur).
  2. **Cascade d'escalade de modèle intra-étape** (changement potentiel de modèle/moteur/effort si une étape stagne ou épuise son palier initial, avec boucle bornée sur le palier d'escalade puis abandon propre) + **anti-oscillation (`A → B → A`)** + option **vérificateur orthogonal (`different_provider`)**.
  3. **Branches de repli conditionnelles dans le DAG** (étapes aval déclenchées sur échec/stagnation/escalade/plafond, pouvant elles-mêmes porter leur propre convergence bornée tout en gardant le DAG global acyclique).
  4. **Métriques durables en SQLite avec auto-agrégation et purge multi-paliers paramétrables façon Time Machine** (détail brut → agrégats journaliers → agrégats mensuels → purge finale, seuils configurables dans les paramètres de Bridget, plus purge des graphes terminés anciens).

### Faits établis
- `bridget_delegate_graph` et la boucle de convergence bornée Spec 313 sont actifs sur la flotte (`native_graphs.rs`, `delegation.rs`, `delegation_mcp.rs`, `protocol.rs`).
- `WorkerRequestOperation::ReadCommitViewFile` (`protocol.rs:3922`, `remote_worker.rs:6587`, `native_delegation.rs:8434`) fournit déjà le modèle exact d'opération synchrone bornée maître → worker sur un worktree attesté (`WorkerResultIdentity`).
- `maintenance::maintain_stores` (`maintenance.rs:124`) appelle déjà `native_delegation::maintain_retention` toutes les heures ; `bridget.db` stocke `native_delegation_graphs` sans purge d'âge à ce jour.
- Les schémas MCP compacts en mode Antigravity ACP doivent rester `<= 550` octets par outil (`mcp.rs`, test `spec309` / `366`).

### Exclusions explicites
- Aucune modification du routage automatique par apprentissage autonome non supervisé en V1 : les métriques sont collectées, agrégées et consultables de façon fiable ; un futur branchement dans `decision_service` restera une option distincte.
- Aucune exécution de shell libre (`sh -c`) dans les portes déterministes : uniquement un `argv` explicite exécuté dans le worktree attesté du producteur.
- Aucun redémarrage automatique de `com.bridget.t3` ou de T3 Code (règle permanente `AGENTS.md`).

---

## 2. Scénarios utilisateur (User Stories & Acceptation)

### US-1 (P1) — Porte déterministe à 0 token avant le vérificateur LLM (`convergence.gate`)
En tant qu'orchestrateur, je peux déclarer une porte déterministe `convergence.gate` (`argv`, `timeout_secs`) sur une étape convergée afin que Bridget exécute ce contrôle objectif dans le worktree du producteur avant tout appel au vérificateur LLM.

**Critères d'acceptation :**
1. **Échec déterministe = 0 appel LLM vérificateur** : si `gate` retourne `exit_code != 0` ou expire (`timed_out`), le vérificateur LLM n'est **pas** appelé ; le diagnostic borné (`exit_code`, durée, extrait stdout+stderr ≤ 8 Kio) est renvoyé directement au producteur au tour suivant, et `gate_saved_verifier_calls` est incrémenté.
2. **Porte verte → vérificateur LLM** : si `gate` retourne `exit_code == 0`, le vérificateur LLM démarre normalement sur `commit_view` et reçoit la preuve que la porte déterministe est passée (`exit_code=0`, durée).
3. **Isolation et sécurité strictes** : `gate` est refusé à l'admission si la posture attestée du parent n'est pas `development` (zéro élévation en `discovery`). `argv` est exécuté sans shell (`Command::new(&argv[0]).args(&argv[1..])`) avec environnement nettoyé (`env_clear()`, `PATH` minimal sans variables `BRIDGET_*` ni jetons, `stdin` nul), uniquement dans le worktree attesté du producteur après vérification que `HEAD == producer_commit` et que l'arbre Git est propre avant exécution (et `HEAD` inchangé après), avec `timeout_secs` borné `5..=300` (défaut `120`), arrêt du groupe de processus entier (`SIGKILL` sur `-pgid`) au timeout et lecture continue bornée à `8 192` octets UTF-8.
4. **Fail-closed sur panne d'infrastructure ou arbre sale** : si le worker est ancien/indisponible, si `producer_commit` est absent, ou si le worktree ne correspond pas exactement au commit attesté, l'étape échoue explicitement (`gate_unavailable`) et n'approuve jamais silencieusement.

### US-2 (P1) — Anti-oscillation (`A → B → A`), cascade d'escalade de modèle et vérificateur orthogonal
En tant qu'orchestrateur, je peux configurer une escalade intra-étape (`convergence.escalation`) et un vérificateur orthogonal (`different_provider: true`) afin qu'une étape qui stagne, oscille ou épuise son palier rapide bascule automatiquement vers un modèle/moteur/effort supérieur pour un nombre borné de tours supplémentaires, puis s'arrête proprement.

**Critères d'acceptation :**
1. **Anti-oscillation globale sur l'étape** : à partir du tour 2 d'un même palier, si le producteur revient à un `commit` déjà vu dans ce palier (`A → B → A` ou `A → A`) OU si le vérificateur/la porte reproduit une empreinte de blocage normalisée (`gate:{exit_code}:...` sans durées volatiles pour la porte) déjà vue dans ce palier, la stagnation/oscillation est détectée immédiatement.
2. **Sessions collantes + escalade sur couture** : pendant le palier 1 (`tier = 0`), le producteur réutilise sa session (`Continue`) ; si le palier 1 échoue par stagnation/oscillation (si `on_stagnation = true`) ou plafond `max_iterations` (si `on_max_iterations = true`), Bridget ouvre **une seule** escalade vers le palier 2 (`tier = 1`, `tier_escalated = true`) via une nouvelle admission `AutoDelegateV2` à clé déterministe par palier (`:t1:i{k}:produce`) avec les surcharges `(engine, model, effort, machine_id)`, ancrée sur `last_producer_commit` + synthèse compacte des échecs du palier 1, pour au plus `extra_iterations` (`1..=3`, plafond global dur `max_iterations + extra_iterations <= 8`). Les tâches des paliers précédents restent suivies pour annulation propre.
3. **Verdict `escalate` court-circuite l'escalade de palier** : si le vérificateur rend `verdict: "escalate"` au palier 0 (arbitrage humain / hors périmètre), l'étape s'arrête immédiatement en `outcome = "escalated"` sans tenter le palier 2.
4. **Fin garantie sur le palier 2** : si le palier 2 stagne, oscille, reçoit `escalate` ou atteint le plafond global, l'étape échoue définitivement sans nouvelle escalade.
5. **Vérificateur orthogonal (`different_provider: true`)** : lorsque le vérificateur n'impose pas d'`engine` explicite, Bridget préfère un moteur différent de celui du producteur parmi les moteurs disponibles ; si aucun autre moteur n'est joignable, il conserve le moteur disponible et trace `verifier_orthogonal = false` sans bloquer la mission.

### US-3 (P1) — Branches de repli conditionnelles dans le DAG (`steps[j].when` + `when_outcomes`)
En tant qu'orchestrateur, je peux déclarer dans `bridget_delegate_graph` une étape aval conditionnelle (`when: "all_succeeded" | "any_failed" | "on_outcome"`, avec `when_outcomes: [...]` requis uniquement pour `on_outcome`) qui ne s'exécute que si l'étape amont échoue ou sort sur un motif donné, et qui peut elle-même porter sa propre convergence bornée.

**Critères d'acceptation :**
1. **Rétrocompatibilité et cohérence** : l'absence de `when` vaut `all_succeeded` (comportement Spec 313 inchangé). Toute combinaison `on_failure = "cancel_graph"` avec `when != "all_succeeded"` est refusée à la validation (`graph_invalid`). Une annulation humaine (`bridget_graph_cancel` / `cancelled`) ne déclenche jamais une branche de repli (`cancelled` exclu de `when_outcomes` et de `any_failed`).
2. **Déclenchement ciblé et propagation sans interblocage** : une étape `when: "any_failed"` (ou `when: "on_outcome"` avec `when_outcomes` dans `failed | stagnated | max_iterations_reached | escalated | gate_unavailable | verdict_unparseable`) attend que ses `depends_on` soient toutes terminales ; si la condition est vérifiée, elle démarre en recevant le contexte d'échec de l'amont (erreur, dernier commit, points bloquants) ; sinon elle passe en `skipped_condition_not_met`. L'état `skipped_condition_not_met` est terminal et se propage proprement aux étapes aval d'un repli non pris (aucun interblocage). Une jonction `all_succeeded` dépendant à la fois d'une étape nominale et de son repli est documentée hors périmètre V1.
3. **État global du graphe clair** : si la branche nominale réussit et que seul le repli passe `skipped_condition_not_met`, `graph.state = "completed"`. Si une étape nominale échoue et qu'une branche de repli réussit, `graph.state = "partial_failure"` (rétrocompatible) avec `fallback_recovered: true` et `counts.fallback_steps_completed >= 1`.
4. **DAG strictement acyclique préservé** : la validation de Kahn s'applique sur toutes les étapes (nominales et de repli), limite globale maintenue à `1..=16` étapes.

### US-4 (P1) — Métriques durables SQLite et rétention/agrégation multi-paliers façon Time Machine
En tant qu'opérateur de Bridget, je veux suivre durablement l'efficacité des boucles de convergence sans jamais laisser `bridget.db` grossir sans limite, grâce à une politique de rollup/agrégation et purge multi-paliers paramétrable.

**Critères d'acceptation :**
1. **Enregistrement atomique sans double comptage** : chaque étape convergée qui atteint un état terminal enregistre sa ligne dans `convergence_metrics_raw` ET pose `metrics_recorded = true` sur le graphe dans la **même transaction SQLite atomique**.
2. **Auto-agrégation et purge multi-paliers paramétrables** : la politique `convergence_metrics_policy` (persistée en SQLite, modifiable par CLI) configure avec invariants validés (`daily_retention_days >= raw_retention_days` et `monthly_retention_months * 31 > daily_retention_days`) :
   - `raw_retention_days` (`1..=365`, défaut `30`) → rollup additif (`ON CONFLICT DO UPDATE`, clés `NOT NULL DEFAULT ''`, somme/MIN/MAX) vers `convergence_metrics_daily` + suppression exacte du même ensemble de lignes brutes via table temporaire de clés ;
   - `daily_retention_days` (`7..=1825`, défaut `180`) → rollup additif vers `convergence_metrics_monthly` + suppression exacte du même ensemble de lignes journalières ;
   - `monthly_retention_months` (`1..=120`, défaut `24`) → purge définitive des mensuelles expirées ;
   - `graph_retention_days` (`7..=365`, défaut `30`) → purge des graphes terminaux (`notice_sent = 1`) anciens dans `native_delegation_graphs` et `native_delegation_graph_tasks` ;
   - `max_raw_rows` (`500..=100_000`, défaut `10_000`) → rollup anticipé des brutes les plus anciennes si le volume dépasse le plafond.
3. **Exécution transactionnelle et idempotente** : le rollup + purge s'exécute dans `maintain_retention` (horaire) et à la demande (`bridget graphs metrics-policy --run-now`) dans une transaction SQLite atomique : aucune ligne n'est comptée deux fois ni perdue entre niveaux.
4. **Consultation claire** : `bridget graphs metrics` fusionne `raw` + `daily` + `monthly` selon la fenêtre demandée et restitue taux de convergence, répartition par tour, économies `gate` / anti-stagnation et tableau par couple `(producteur, vérificateur)` (texte et `--json`).

### US-5 (P2) — Découvrabilité MCP (≤ 550 B compact), pré-validation locale, skills et documentation
En tant qu'agent appelant, je dispose des nouveaux champs (`when`, `convergence.gate`, `convergence.escalation`, `convergence.different_provider`) validés localement avant connexion sans consommer `request_id`, avec un schéma compact toujours `<= 550` octets dans `mcp.rs`, ainsi que la documentation à jour dans les skills `bridget` et `graph_it` et `docs/delegation-native*.md`.

---

## 3. Exigences fonctionnelles et non fonctionnelles

- **FR-01** : Étendre `GraphConvergenceLoop` (`protocol.rs`) de façon 100 % additive (`#[serde(default)]`) avec `gate: Option<GraphDeterministicGate>`, `escalation: Option<GraphConvergenceEscalation>`, `different_provider: bool`.
- **FR-02** : Étendre `DelegateGraphStep` (`protocol.rs`) de façon 100 % additive avec `when: GraphStepCondition` (`all_succeeded` par défaut, `any_failed`, `on_outcome`) et `when_outcomes: Vec<String>` (requis uniquement pour `on_outcome`, excluant `cancelled`).
- **FR-03** : Ajouter `WorkerRequestOperation::RunDeterministicGate` et `WorkerResponseOperation::DeterministicGateResult` dans `protocol.rs`, implémentés dans `remote_worker.rs` (exécution asynchrone non bloquante bornée à 16 portes concurrentes, timeout `5..=300 s`, sortie UTF-8 tronquée à `8 192` octets, sans shell).
- **FR-04** : Étendre `NativeGraphConvergenceState` et `NativeGraphConvergenceIteration` (`delegation.rs`) avec suivi des empreintes vues (`seen_commits`, `seen_blocking_fingerprints`), palier courant (`escalation_tier`, `total_max_iterations`), résultat de porte par tour (`gate_passed`, `gate_exit_code`, `gate_duration_ms`, `gate_output_excerpt`), compteurs d'économies (`gate_saved_verifier_calls`, `stagnation_saved_iterations`), et moteurs/modèles effectifs observés.
- **FR-05** : Implémenter les tables SQLite `convergence_metrics_policy`, `convergence_metrics_raw`, `convergence_metrics_daily`, `convergence_metrics_monthly` dans `DelegationStore`, branchées sur `maintain_retention` (horaire) et exposées via requêtes daemon + CLI `bridget graphs metrics` et `bridget graphs metrics-policy`.
- **FR-06** : Pré-valider localement dans `delegation_mcp.rs` et côté daemon dans `native_graphs.rs` : `gate.argv` non vide (1..=32 args, sans contrôle/NUL), `gate.timeout_secs` dans `5..=300`, `escalation` exigeant au moins un axe `(engine|model|effort)` et `max_iterations + extra_iterations <= 8`, `when` valide et cohérent (`when != all_succeeded` interdit sur une étape racine sans `depends_on`).
- **FR-07** : Maintenir `bridget_delegate_graph` en mode MCP compact strictement `<= 550` octets (sans `properties` imbriquées sur `steps.items`) et mettre à jour les skills `bridget`, `graph_it` et `docs/delegation-native.{fr.,}md`.
- **NFR-01 (Sécurité)** : jamais d'invocation shell dans `gate`, exécution confinée au worktree attesté de la tâche productrice, aucune élévation de permissions.
- **NFR-02 (Complexité)** : détection d'oscillation $O(I)$ avec $I \le 8$, propagation DAG conditionnelle $O(V + E)$ avec $V \le 16$, rollup SQLite indexé par `finished_at` / `day_utc` / `month_utc` avec lot borné par transaction.
- **NFR-03 (Rétrocompatibilité)** : tout graphe Spec 313 ou antérieur persisté en SQLite se désérialise sans erreur (`#[serde(default)]`), et aucun ancien worker ne casse si la porte n'est pas demandée.

---

## 4. Critères de réussite mesurables

1. Tests unitaires & machine à états couvrant : porte déterministe KO (0 appel vérificateur) → correction → porte OK → vérificateur `approve` ; anti-oscillation `A → B → A` ; escalade palier 1 → palier 2 sur stagnation et sur plafond ; plafond global dur après escalade ; `different_provider` (avec alternative et repli propre sans alternative) ; branches DAG `any_failed` et `on_outcome` (déclenchées sur échec, sautées sur succès, graphe `completed` si repli inutile, branche de repli portant sa propre convergence).
2. Tests worker prouvant l'exécution réelle `argv` sans shell dans un dossier temporaire isolé, la troncature UTF-8 ≤ 8 Kio et le timeout dur.
3. Tests SQLite prouvant le cycle complet Time-Machine : insertion `raw` → rollup exact `raw → daily` (conservation stricte des sommes et suppression des brutes) → rollup exact `daily → monthly` → purge finale `monthly` + purge `native_delegation_graphs` + modification et validation des seuils `convergence_metrics_policy` + déclenchement sur `max_raw_rows`.
4. Tests MCP prouvant la pré-validation locale (0 consommation `request_id`) et la taille compacte ACP `<= 550 B` sur tous les 20 outils.
5. Livraison compilée (`linux-x86_64-glibc2.36` + `macos-arm64`), poussée sur `main`, déployée sur la flotte joignable (`deploy-fleet.py`, sans redémarrage non autorisé de `com.bridget.t3`) et vérifiée en direct via `bridget graphs metrics-policy` et `bridget graphs metrics`.
