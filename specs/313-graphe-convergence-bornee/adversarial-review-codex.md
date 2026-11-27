# Contre-revue adverse — Spec 313 (Graphes de convergence bornée et découvrabilité MCP)

## 1. Sollicitation pair externe (Codex)

- **Destinataire** : `a4c7d3b7-34cf-4603-9f6c-c6ff38558cc7` (Codex, même projet `64.bridget`, présence `connected`)
- **Identifiant message** : `mcp-97309-6b0929ed-1` (`reply_timeout=60`, lecture seule, 0 modification)
- **Horodatage** : 2026-11-27 07:57 CET
- **Statut** : `pas de réponse dans le délai` — journal vérifié (`seq=36 turn_start` à `06:57:52Z`, `seq=37 turn_end stop_reason="error"` à `06:57:57Z`). Aucune relance non bornée effectuée (conformité `SPECKIT-COORDINATION` / `my-specify-all`).

## 2. Contre-revue adverse systématique interne (concurrence, reprise, annulation, schéma compact)

| # | Risque identifié | Sévérité | Résolution appliquée |
|---|---|---|---|
| 1 | **Reprise après redémarrage daemon pendant `launching_verify` / `launching_produce`** : `st.graph_launching_steps` est en mémoire ; si le daemon redémarre pendant une admission de vérificateur ou une continuation producteur, `cs.phase` resterait bloqué en `launching_*`. | Critique | Dans `tick()`, traiter `launching_verify` / `launching_produce` comme `pending_verify` / `pending_produce` dès que `launch_key` n'est pas actif dans `graph_launching_steps`, en réutilisant la même clé déterministe `convergence_step_request_id` (reprise idempotente). |
| 2 | **Fuite de tâche orpheline en cas d'annulation concurrente pendant l'admission** : si `bridget_graph_cancel` survient pendant qu'un thread `ConvergenceVerify` ou `ConvergenceNextProduce` attend l'admission, la nouvelle tâche admise ne serait pas annulée. | Majeure | Après reprise du verrou dans les threads `ConvergenceVerify` et `ConvergenceNextProduce`, si le graphe/étape n'est plus en phase attendue (`running` + `launching_*`), annuler immédiatement le `task_id` nouvellement créé hors verrou via `cancel_task_for_graph`. |
| 3 | **Double tâche à annuler sur `on_failure: cancel_graph`** : en phase `verifying`, `producer_task_id` reste gardé (`keep_alive_secs >= 900`) tandis que `step.task_id` pointe sur `verifier_task_id`. | Majeure | Dans `propagate_graph_state`, lorsque `CancelGraph` traite une étape `running` dotée d'un `convergence_state`, ajouter à `to_cancel_tasks` tous les `task_id` distincts (`step.task_id`, `producer_task_id`, `verifier_task_id`). |
| 4 | **Compatibilité Antigravity MCP compact (`fd0f3322`)** : tout `type: "object"` imbriqué avec `properties` fait rejeter silencieusement l'outil par Antigravity. | Critique | `bridget_delegate_graph` garde `steps.items = {"type":"object"}` sans `properties` imbriquées ; le contrat `steps` + `convergence` + verdicts est porté par `steps.description` et `tool.description`, vérifié par `compact318_tests` et `spec309_compact_discovery_tools` (`<= 1800` octets/outil, `<= 10 000` octets total). |
| 5 | **Consommation indue de `request_id` sur erreur de forme** : un cycle DAG ou un `max_iterations` invalide envoyé au daemon pouvait consommer ou polluer l'admission. | Majeure | Pré-validation complète côté client MCP (`delegation_mcp.rs`) avant ouverture de socket (Kahn + bornes `2..=5` + `parse_auto_delegate_object` sur `delegation` et `convergence.verify` + `parse_brief` + `parse_workspace`). |

## Verdict

**APPROVE_WITH_CHANGES -> TOUS POINTS APPLIQUÉS ET TESTÉS.**
