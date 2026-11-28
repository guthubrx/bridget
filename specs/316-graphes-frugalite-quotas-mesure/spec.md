# Spécification 316 : Graphes frugaux en quotas et mesure comparative par version (Lot A)

- Statut : En cours d'implémentation
- Date : 2026-11-28
- Suite prévue : Lot B (Spec 317 : Map-Reduce borné sur workers chauds + escalade `Best-of-N` avec annulation anticipée).

## 1. Contexte et problème

Bridget dispose déjà de graphes DAG (`313`), de boucles de convergence avec portes déterministes à 0 jeton, vérificateurs orthogonaux, escalades bornées, branches de repli et métriques Time-Machine SQLite (`314`). Il collecte également les compteurs de tokens attestés par exécution dans `execution_usage_totals` et `usage_samples`.

Cinq manques subsistent pour économiser les quotas et mesurer objectivement les gains ou régressions entre deux versions de Bridget :

1. **Invalidation du cache de préfixe (KV-cache)** : les constructeurs de prompts de graphe (`build_verifier_task`, `build_enriched_step_task`, `build_producer_revision_task`) placent aujourd'hui les métadonnées variables (numéro d'itération, `task_id`, commit courant, extraits) en tête du prompt avant la consigne fixe.
2. **Absence de plafond budgétaire au niveau du graphe** : rien ne borne globalement le nombre total d'admissions/tours, les tours frontière ou la durée totale d'un graphe entier.
3. **Absence de tableau noir structuré (`blackboard`)** : les étapes aval reçoivent de longs extraits texte ou relisent les mêmes fichiers au lieu de lire quelques clés JSON compactes produites par les étapes amont.
4. **Absence de routage par palier de coût (`model_tier`)** : une étape simple ne peut pas demander explicitement un modèle léger (`light`) sans figer un nom de modèle précis, ni réserver `frontier` aux étapes complexes.
5. **Comparaison inter-versions absente** : `bridget_build_id` et les tokens réels (`execution_usage_totals`) ne sont pas rattachés aux métriques de graphe, ce qui empêche de mesurer l'écart fonctionnel, tokens, cache et durée entre une version Bridget `A` et une version `B`.

## 2. Exigences fonctionnelles

### FR-001 : Préfixe stable pour le Prompt Caching
- `build_verifier_task` place les blocs invariants (`## Contrat de verdict fermé` et `## Consigne de vérification`) en tête, et le contexte variable de l'itération courante (`task_id`, commit, fichiers, extrait) en pied.
- `build_enriched_step_task` place la consigne métier fixe (et le contrat d'écriture tableau noir éventuel) en tête, et les lectures du tableau noir puis les résultats variables des dépendances en pied. En cas de troncature à `MAX_TASK_BYTES`, seule la partie variable en pied est raccourcie.
- `build_producer_revision_task` avec `for_fallback_new_task = true` place la consigne initiale en tête afin de conserver un préfixe identique à l'itération 1.

### FR-002 : Budgets durs de graphe (`budget`)
- `DelegateGraphRequest` accepte un objet optionnel `budget` :
  - `max_total_rounds` (`1..=128`, optionnel) : plafond total d'admissions et de reprises sur le graphe.
  - `max_frontier_rounds` (`0..=64`, optionnel) : plafond d'admissions/reprises de palier frontière (`model_tier = frontier`, palier d'escalade `> 0`, ou modèle frontière explicite).
  - `max_duration_secs` (`30..=86400`, optionnel) : durée murale maximale depuis `created_at`.
- Avant chaque admission (étape racine, étape aval, vérificateur, reprise producteur), le budget est contrôlé.
- Un dépassement clôture proprement l'étape avec `outcome = "budget_exhausted"` et `error = "budget_exhausted:<cause>"`.
- `budget_exhausted` est autorisé dans `when_outcomes`. Lorsque seul `max_frontier_rounds` est épuisé, une étape non frontière (`light` ou `standard`) reste admise tant que `max_total_rounds` et `max_duration_secs` ne sont pas atteints.
- `bridget_graph_status` expose `budget` et `budget_usage` (`total_rounds_used`, `frontier_rounds_used`, `exhausted_reason`).

### FR-003 : Tableau noir structuré par graphe (`blackboard`)
- Chaque graphe possède un magasin clé-valeur borné (`MAX_BLACKBOARD_KEYS = 32`, clé `[A-Za-z0-9_.-]{1..=64}`, valeur JSON `<= 4096` octets).
- Chaque étape peut déclarer `blackboard_reads` (max 16 clés) et `blackboard_writes` (max 8 clés).
- À la clôture réussie d'une étape, Bridget extrait automatiquement un éventuel objet JSON fermé `{"blackboard":{...}}` (en fin de réponse ou sur une ligne autonome, sans appel MCP supplémentaire). Si `blackboard_writes` est déclaré, seules les clés listées sont acceptées.
- Seules les clés demandées par `blackboard_reads` (et présentes dans le tableau noir) sont injectées dans le prompt de l'étape aval.
- `bridget_graph_status` expose l'état courant du tableau noir.

### FR-004 : Routage par palier de coût (`model_tier`)
- Chaque étape peut déclarer `model_tier`: `"light" | "standard" | "frontier"`.
- Lorsque `delegation.model` n'est pas imposé explicitement par le parent, Bridget résout dynamiquement un modèle sain du catalogue disponible sur la flotte correspondant au palier demandé (avec `effort` adapté `low`/`medium`/`high` s'il n'est pas déjà imposé). Si aucun modèle du palier n'est disponible, repli gracieux sur le routeur automatique sans bloquer l'étape.
- Un `delegation.model` explicite du parent n'est jamais écrasé ; `model_tier` sert alors de classification budgétaire et métrique.

### FR-005 : Mesure comparative par version Bridget (`bridget_build_id`) et tokens réels
- Chaque `NativeGraphRecord` enregistre le `bridget_build_id` du daemon qui l'exécute.
- À la clôture de chaque étape terminale de graphe (convergée ou simple), Bridget agrège les compteurs attestés depuis `execution_usage_totals` (`input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`) pour tous les `execution_id` associés à l'étape, et les persiste atomiquement dans `convergence_metrics_raw` et dans une table durable `graph_build_metrics` indexée par `(bridget_build_id, project, model_tier)`.
- `bridget graphs metrics` expose :
  - `by_build` (métriques fonctionnelles + tokens facturables + ratio de cache hit + arrêts budget + lectures/écritures tableau noir par `bridget_build_id`),
  - `by_tier` (comparaison `light` / `standard` / `frontier`),
  - `--compare <buildA>..<buildB>` calculant les écarts (taux d'approbation en points de %, tokens facturables moyens par étape en %, ratio de cache hit en points de %, durée moyenne en %).
- Les 20 schémas MCP compacts restent strictement `< 600` octets.

## 3. Critères d'acceptation (Gherkin)

```gherkin
Feature: Graphes frugaux et comparaison inter-versions (316)

  Scenario: Préfixe stable pour le cache KV sur plusieurs itérations de vérification
    Given une étape de convergence avec verify.task fixe
    When Bridget génère le prompt de vérification pour l'itération 1 puis l'itération 2
    Then le préfixe commun contenant le contrat de verdict et verify.task est strictement identique au début des deux prompts

  Scenario: Budget dur frontière épuisé déclenchant un repli light
    Given un graphe avec budget.max_frontier_rounds = 1 et max_total_rounds = 4
    And une étape frontier suivie d'une étape de repli light sur when_outcomes = ["budget_exhausted"]
    When le budget frontière est épuisé
    Then aucune seconde admission frontière n'est lancée
    And la branche de repli light reste autorisée dans la limite de max_total_rounds

  Scenario: Partage compact via tableau noir sans outil MCP supplémentaire
    Given une étape A avec blackboard_writes = ["api_contract"]
    And une étape B avec depends_on = ["A"] et blackboard_reads = ["api_contract"]
    When A termine avec une ligne JSON {"blackboard":{"api_contract":{"v":2}}}
    Then B reçoit uniquement la clé api_contract dans son prompt et graph.blackboard contient la clé validée

  Scenario: Mesure comparative entre deux builds Bridget
    Given des étapes terminées enregistrées sous buildA et buildB avec leurs compteurs execution_usage_totals
    When on interroge bridget graphs metrics --compare buildA..buildB
    Then le rapport rend les totaux de chaque build et les deltas fonctionnels, tokens facturables, cache hit et durée
```
