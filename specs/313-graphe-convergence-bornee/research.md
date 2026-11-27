# Recherche et décisions techniques — 313

## 1. Hypothèse critique vérifiée avant codage : continuité du producteur via `Continue` vs nouvelle tâche

- **Hypothèse** : Pour éviter que le tour 2 d'une boucle `produce ↔ verify` ne reparte de zéro (ou de l'ancien `expected_base_commit`), Bridget peut réutiliser `NativeDelegationRequest::Continue` sur la tâche productrice gardée (`keep_alive_secs`).
- **Preuve dans le code existant** :
  - `NativeDelegationRequest::Continue { task_id, request_id, task, brief }` existe déjà dans `crates/bridget-daemon/src/daemon/native_delegation.rs:13060+`.
  - Une tâche admise avec `keep_alive_secs: Some(900)` garde son enfant et son worktree vivants jusqu'à 15 minutes après `result_available` (plafond 7 200 s, max 8 rondes — largement suffisant pour `max_iterations <= 5`).
  - Lors d'un `Continue` accepté, `task.round` passe de `r` à `r + 1` et `task.state` revient à `working` jusqu'à la remise de la ronde `r + 1` (`result_available`).
- **Décision** :
  - Pour toute étape ayant `convergence.is_some()`, le lancement du tour 1 force `keep_alive_secs = Some(existing.unwrap_or(900).clamp(900, 7200))` afin de préserver le worktree et la session productrice entre `produce` et `verify`.
  - Au tour `k >= 2`, Bridget tente d'abord `NativeDelegationRequest::Continue` sur `producer_task_id` en stockant `producer_expected_round = k`. Si `Continue` échoue (garde expirée, redémarrage, tâche non continuable), repli transparent et sûr sur une nouvelle admission `AutoDelegateV2` avec `expected_base_commit` / `workspace.base_ref` mis à jour vers le dernier `producer_commit` connu.

## 2. Hypothèse vérifiée : synchronisation des rondes `Continue` dans `sync_steps_from_tasks`

- **Risque** : Juste après `Continue`, tant que l'admission de la ronde 2 est en cours, `task.state` peut encore être `result_available` de la ronde 1.
- **Solution** : Stocker `producer_expected_round: u32` dans `NativeGraphConvergenceState` et ne considérer la phase `producing` terminée que lorsque `task.state == "result_available" && task.round >= producer_expected_round` (ou que `task_id` est une nouvelle tâche de repli avec `round >= 1`).

## 3. Hypothèse vérifiée : compatibilité Antigravity en mode MCP compact

- **Preuve** : Commit `fd0f3322` (`spec318`/`spec322`) et vérification en direct sur `bridget mcp` (`BRIDGET_MCP_COMPACT_SCHEMAS=1`). Antigravity supprime silencieusement un outil si un champ `type: "object"` imbriqué déclare `"properties"`, mais accepte les `"description"` sur les propriétés et les `"enum"` sur les chaînes.
- **Décision** : Conserver `steps: {"type":"array","items":{"type":"object"},"description":"..."}` en mode compact, sans `properties` imbriquées, et rester sous le budget compact ACP (`<= 1 800` octets/outil, `<= 11 000` octets total).

## 4. Hypothèse vérifiée : propagation du commit produit vers `verify`

- **Constat** : Lorsque `produce` crée un commit dans un worktree, `factual_summary["commit"]` porte le SHA 40 hex du commit produit.
- **Décision** : Lors du lancement de `verify` au tour `k`, si `producer_commit` est un SHA 40 hex valide :
  - si `verify.expected_base_commit` était absent ou égal à l'ancien SHA initial, mettre à jour `verify.expected_base_commit = Some(producer_commit)` lorsque `verify.workspace` est `CommitView` ou `Worktree` (et `base_ref = Some(producer_commit)` si `Worktree` sans `base_ref` explicite ou égal à l'ancien SHA) ;
  - toujours inclure `commit=<sha>` dans le préambule factuel de `verify`.
  - Si `verify` utilise `shared` ou n'est pas en dépôt Git, ne pas injecter d'`expected_base_commit` non demandé.
