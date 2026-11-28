# Spécification 317 : Map-Reduce borné sur workers chauds et Best-of-N sur escalade (Lot B)

- Statut : En cours d'implémentation
- Date : 2026-11-28
- Prérequis livré : Spec 316 Lot A (`dacd18ef`, budgets durs, prompt caching, tableau noir, `model_tier`, mesure comparative par `bridget_build_id`).

## 1. Contexte et problème

Deux besoins complémentaires au Lot A restent ouverts pour tirer parti du parallélisme sans tomber dans les dérives d'un essaim libre :

1. **Traitement de lots d'items indépendants (Map-Reduce)** : aujourd'hui, auditer ou migrer 8 à 16 fichiers/modules indépendants exige de déclarer 8 à 16 étapes statiques séparées, chacune démarrant une session neuve à froid (sans réutiliser le cache KV).
2. **Déblocage d'une étape en stagnation lors de l'escalade (`Best-of-N` réservé à l'escalade)** : dans 80-90 % des cas, le palier 0 suffit (0 surcoût). Lorsqu'une étape de convergence stagne ou atteint son plafond, parier sur un seul modèle de remplacement peut retomber dans un angle mort ; lancer 2 (ou 3 avec porte déterministe) candidats orthogonaux en parallèle avec annulation immédiate des perdants dès le premier `approve` maximise le taux de déblocage tout en bornant la dépense.

## 2. Exigences fonctionnelles

### FR-001 : Étape Map-Reduce bornée sur pool de workers chauds (`map`)
- `DelegateGraphStep` accepte un champ optionnel `map`:
  - `items`: `Vec<String>` (`2..=16` éléments non vides, max `512` octets chacun).
  - `max_concurrency`: `u8` (`1..=4`, défaut `2`).
  - `reuse_workers`: `bool` (défaut `true`) — maintient `keep_alive_secs >= 900` sur chaque slot worker et enchaîne les items suivants via `Continue` sur la même session chaude (repli transparent sur une nouvelle admission avec préfixe invariant en tête si le worker a expiré).
  - `fail_fast`: `bool` (défaut `false`) — si vrai, saute les items restants dès le premier échec.
- `map` et `convergence` sont mutuellement exclusifs sur une même étape (pré-validé localement avant connexion).
- Chaque admission / continuation d'item respecte `budget` (`max_total_rounds`, `max_frontier_rounds`, `max_duration_secs`) et le `model_tier` de l'étape.
- Lorsque tous les items sont terminés, Bridget produit un bilan agrégé structuré dans `step.result_excerpt` (transmis automatiquement à l'étape aval `reduce` via `depends_on`), applique les éventuelles `blackboard_writes`, et expose `map_state` (y compris `reused_worker_hits`) dans `bridget_graph_status`.

### FR-002 : Escalade `Best-of-N` réservée à `convergence.escalation` (`candidates`)
- `GraphConvergenceEscalation` accepte un champ optionnel `candidates`: `u8` (`1..=3`, défaut `1` = comportement actuel inchangé).
- Sans porte déterministe (`convergence.gate` absente), `candidates` est strictement plafonné à `2` (pré-validé localement avant connexion).
- `candidates > 1` constitue un motif d'escalade valide même sans surcharge manuelle de `model`/`engine` : Bridget diversifie automatiquement les moteurs sains disponibles entre les candidats.
- Lorsque l'escalade se déclenche (`stagnated` ou `max_iterations_reached`, jamais sur `verdict: "escalate"`), Bridget lance jusqu'à `candidates` producteurs indépendants en worktrees isolés, chacun soumis à `budget` (`is_frontier = true`).
- Chaque candidat passe d'abord `gate` (0 jeton) s'il est configuré ; un échec de `gate` rejette ce candidat sans appeler de vérificateur LLM (`gate_saved_verifier_calls += 1`).
- **Arrêt anticipé au premier gagnant** : dès qu'un candidat obtient `verdict: "approve"`, son `task_id` et son `head_commit` deviennent le résultat officiel de l'étape, et tous les autres candidats encore actifs (producteurs et vérificateurs) sont immédiatement annulés (`candidates_cancelled`).

### FR-003 : Annulation, télémétrie et plafond MCP compact
- `bridget_graph_cancel` et `on_failure = cancel_graph` annulent tous les `task_id` actifs issus de `map_state` et de `escalation_candidates`.
- `collect_step_usage_agents` inclut tous les `task_id` de `map_state` et `escalation_candidates` afin que `graph_build_metrics` et `bridget graphs metrics --compare` reflètent 100 % des tokens réels et des gains de cache.
- Les 20 outils MCP compacts (`BRIDGET_MCP_COMPACT_SCHEMAS=1`) restent strictement `< 600` octets.
