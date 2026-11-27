# Spécification 313 — Graphes de missions avec étapes de convergence bornée et découvrabilité MCP

**Date** : 2026-11-27
**Statut** : Implémentée (`my-specify-all`, branche unique `main`)
**Dépendances** : `bridget_delegate_graph` (`native_graphs.rs`), `bridget_auto_delegate` (contrats 150/152/154/157/160/168), `bridget_task_continue`, schémas MCP compacts (309/318/321/322/339).

---

## 1. Demande initiale et cadrage validé (Phase 0 — `$be-ambitious`)

### Demande utilisateur initiale
- Pouvoir associer dans `bridget_delegate_graph` la force d'un DAG et la force d'une boucle bornée avec critère de sortie et arrêt sur stagnation.
- Rendre la capacité réellement découvrable et appelable sans erreur dès le premier essai par les agents (MCP complet et compact, pré-validation locale, skills, documentation).

### Cadrage reformulé et validé
- **Objectif** : Étendre `bridget_delegate_graph` avec une primitive unique de **convergence bornée** (`produce ↔ verify`, 2 à 5 itérations max) encapsulée dans un nœud du DAG acyclique, tout en rendant les graphes Bridget (DAG simples et convergences) immédiatement découvrables et pré-validés sans consommer `request_id`.
- **Faits établis** :
  - `bridget_delegate_graph` existe (`crates/bridget-daemon/src/daemon/native_graphs.rs`), impose un DAG strict (1–16 étapes) via tri topologique de Kahn et persiste l'état en SQLite.
  - Il était absent des skills `/Users/moi/.agents/skills/bridget/SKILL.md` et `/Users/moi/.agents/skills/graph_it/SKILL.md`, et son champ `steps` en mode MCP compact n'exposait aucune description de structure.
- **Exclusions explicites de la V1** (options futures distinctes) :
  - Option A : vérificateur shell déterministe sans LLM dans `verify`.
  - Option B : visualisation dédiée des itérations de boucle dans la console web (`bridget ui`).
  - Boucles imbriquées dans des boucles (interdit par construction en V1).

---

## 2. Scénarios utilisateur (User Stories & Acceptation)

### US-1 (P1) — Étape de convergence bornée `produce ↔ verify` dans un DAG
En tant qu'agent orchestrateur, je peux déclarer dans `bridget_delegate_graph` un DAG `A → B (convergence max 3 tours) → D` afin que Bridget enchaîne automatiquement le producteur et le vérificateur indépendant de `B` sans me réveiller entre les tours, puis déclenche `D` dès que le vérificateur approuve.

**Critères d'acceptation :**
1. **Convergence nominale** : au tour 1, le producteur rend un livrable ; le vérificateur reçoit le résumé factuel (dont `commit`, `files_changed`, extrait) et rend `{"verdict":"revise","reason":"...","blocking_points":["..."]}` ; Bridget relance automatiquement le tour 2 du producteur avec ces points bloquants, puis le vérificateur du tour 2 rend `{"verdict":"approve","reason":"..."}` → l'étape passe `completed`, les étapes aval (`D`) démarrent, et le parent n'est réveillé qu'en fin de graphe (avec `notify_policy=on_graph_done_or_failure`).
2. **Continuité d'état entre tours** : au tour $k+1$, le producteur reprend son travail du tour $k$ (via `Continue` sur la même tâche/worktree gardée si elle est toujours active, ou sinon via une nouvelle tâche ancrée sur le dernier commit produit par le tour $k$ et informée des points bloquants). Le vérificateur du tour $k$ reçoit le dernier commit produit au tour $k$ lorsque sa délégation utilise `expected_base_commit` / `worktree` / `commit_view`.
3. **Compatibilité ascendante** : tout graphe DAG existant sans champ `convergence` conserve un comportement strictement identique.

### US-2 (P1) — Garde-fous d'arrêt (plafond, stagnation, escalade, verdict illisible)
En tant qu'utilisateur et opérateur de la flotte, je veux la garantie qu'aucune étape de convergence ne peut tourner en boucle infinie, s'auto-valider en prose libre ou gaspiller des tours sans progrès.

**Critères d'acceptation :**
1. **Plafond dur (`max_iterations`)** : valeur entière bornée `2..=5` (défaut `3`). Si le dernier tour retourne encore `revise`, l'étape se termine en échec explicite (`convergence_max_iterations_reached`) et propage `on_failure` normalement.
2. **Arrêt sur stagnation (`stop_on_stagnation`, défaut `true`)** : à partir du tour 2, si le producteur rend exactement le même commit SHA que le tour précédent (sans nouveau commit) OU si le vérificateur rend exactement la même empreinte normalisée de motifs bloquants deux tours de suite, la boucle s'arrête immédiatement en `convergence_stagnated` avant d'atteindre `max_iterations`.
3. **Escalade immédiate** : si le vérificateur rend `{"verdict":"escalate","reason":"..."}`, la boucle s'arrête immédiatement en `convergence_escalated` et remonte la raison au parent sans lancer d'autre tour.
4. **Fail-closed sur verdict absent ou invalide** : si la réponse du vérificateur ne contient aucun objet JSON fermé avec `verdict` dans `approve | revise | escalate`, l'étape ne s'approuve jamais silencieusement et s'arrête en `convergence_verdict_unparseable`.
5. **Annulation propre** : `bridget_graph_cancel` annule la tâche active (producteur ou vérificateur en cours) et toute garde associée de l'étape de convergence.

### US-3 (P1) — Traçabilité durable par itération
En tant qu'orchestrateur ou humain, je peux inspecter `bridget_graph_status(graph_id)` et le bilan consolidé final pour voir l'itération courante, la phase (`producing` / `verifying` / `done`), l'issue (`approved`, `escalated`, `stagnated`, `max_iterations_reached`, `verdict_unparseable`, `failed`) et l'historique numéroté de chaque tour (`iteration`, `producer_task_id`, `producer_commit`, `verifier_task_id`, `verdict`, `reason`, `blocking_points`).

### US-4 (P1) — Pré-validation locale MCP sans brûler `request_id` et schémas complets/compacts
En tant qu'agent appelant (Antigravity compact, Codex ou Claude), je vois clairement dans le schéma MCP comment structurer `steps` et `convergence`, et toute erreur de forme (champ inconnu, `max_iterations` hors `2..=5`, cycle dans `depends_on`, `brief` ou `workspace` invalide dans `delegation` ou `convergence.verify`) est rejetée localement **avant** connexion au daemon sans consommer `request_id`, avec un exemple minimal exact dans le message d'erreur.

### US-5 (P2) — Découvrabilité dans les skills et la documentation
En tant qu'agent consultant `bridget/SKILL.md`, `graph_it/SKILL.md` ou `docs/delegation-native*.md`, je trouve en quelques secondes quand utiliser `bridget_auto_delegate` vs `bridget_delegate_graph` (DAG) vs `bridget_delegate_graph` avec `convergence`, ainsi qu'un exemple JSON directement réutilisable.

---

## 3. Exigences fonctionnelles et non fonctionnelles

- **FR-01** : `DelegateGraphStep` accepte un champ optionnel `convergence: Option<GraphConvergenceLoop>` (`#[serde(deny_unknown_fields)]`) contenant `verify: AutoDelegateV2`, `max_iterations: u8` (`2..=5`, défaut `3`), `stop_on_stagnation: bool` (défaut `true`).
- **FR-02** : Validation stricte côté client (`delegation_mcp.rs`) **et** côté daemon (`native_graphs.rs`) : mêmes règles de forme, mêmes bornes (`1..=16` étapes, `2..=5` itérations, absence de cycle DAG par Kahn, champs `AutoDelegateV2` et `brief`/`workspace` validés pour `delegation` et `convergence.verify`).
- **FR-03** : Le préambule du vérificateur injecte automatiquement les faits du tour producteur (`iteration/max_iterations`, `task_id`, `commit`, `files_changed`, extrait) **et** le contrat de verdict JSON fermé (`approve` / `revise` / `escalate`).
- **FR-04** : Le parseur de verdict lit le dernier objet JSON valide contenant `"verdict"` (en tolérant les blocs Markdown ```` ```json ````) et refuse en fail-closed toute sortie sans verdict fermé valide.
- **FR-05** : Continuité entre tours : le producteur d'une étape `convergence` est lancé avec une garde minimale (`keep_alive_secs >= 900`, bornée à `7200`) afin que le tour `k >= 2` puisse reprendre le même worktree/session via `NativeDelegationRequest::Continue` (`round = k`) ; si la reprise `Continue` est refusée (ex. tâche non gardée ou expirée), repli automatique sur une nouvelle admission `AutoDelegateV2` avec le dernier `commit` connu et le retour du vérificateur.
- **FR-06** : Chaque itération est identifiée de manière déterministe et idempotente (`{graph_req}:{step_id}:i{k}:produce` et `{graph_req}:{step_id}:i{k}:verify`), persistée dans `NativeGraphStepRecord.convergence_state` en SQLite.
- **FR-07** : Détection de stagnation sur deux signaux indépendants (`k >= 2`) : même `producer_commit` non vide que le tour `k-1`, ou même `blocking_fingerprint` SHA-256 normalisé du vérificateur que le tour `k-1`.
- **FR-08** : En mode MCP compact (`BRIDGET_MCP_COMPACT_SCHEMAS=1`), `bridget_delegate_graph` garde `steps.items = {"type":"object"}` **sans `properties` imbriquées** (contrainte Antigravity `fd0f3322`), ajoute une `description` claire sur `steps` et reste sous le budget compact ACP (`<= 1 800` octets/outil, `<= 11 000` octets total).
- **FR-09** : Mise à jour de `/Users/moi/.agents/skills/bridget/SKILL.md`, `/Users/moi/.agents/skills/graph_it/SKILL.md`, `docs/delegation-native.fr.md` et `docs/delegation-native.md`.
- **NFR-01 (Sécurité / Permissions)** : aucune élévation de droits ; producteur et vérificateur restent soumis aux permissions attestées du parent et à leur posture respective.
- **NFR-02 (Complexité)** : validation $O(V + E)$ (Kahn, $V \le 16$), parsing verdict $O(L)$ en une passe sur le résultat borné, `tick()` $O(G \cdot S)$ quasi nul sans graphe actif.
- **NFR-03 (Compatibilité SQLite)** : les colonnes/JSON existants de `native_delegation_graphs` restent désérialisables sans migration destructive (`#[serde(default)]`).

---

## 4. Critères de réussite mesurables

1. Tests unitaires et d'état prouvant : (a) convergence en 2 tours (`revise` → `approve`) débloquant l'étape aval, (b) arrêt sur stagnation (commit identique et `blocking_points` identiques), (c) arrêt immédiat sur `escalate`, (d) arrêt sur `max_iterations`, (e) fail-closed sur verdict non structuré, (f) annulation en phase `producing` et `verifying`, (g) rétrocompatibilité des graphes DAG existants.
2. Tests MCP prouvant : pré-validation locale de `steps`, `depends_on`, cycles et `convergence` avant connexion (zéro consommation de `request_id`), schéma compact Antigravity valide (zéro `properties` imbriquées sur `object`, zéro description tronquée, budget respecté).
3. Vérification en direct sur le binaire installé (`bridget mcp` en mode compact) + déploiement vérifié sur les 5 machines (`cartae`, `sysle2`, `ksstore`, `macbook-m1`, `monordinateur`).
