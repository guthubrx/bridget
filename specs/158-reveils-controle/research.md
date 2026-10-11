# Recherche - Session158 contrôle humain, portées MCP, réveils

Date : 2026-10-11, révisée après contre-revue. Source : checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`) et T3
`origin/main` par `git show`. Les lignes changent après 150 à 157.

## R1 - Vue globale par le plan de contrôle, réservée à l'humain attesté

- **Constat** : `bridget lineage` exige `--t3-thread` (`cli_lineage.rs:19-20`) ; le daemon rattache la requête à un fil T3 (`daemon/native_lineage.rs`, `authorize`).
- **Constat sécurité** : `ControlStateV1` est une capacité négociable. Le daemon l'accorde à tout client qui la demande (`daemon.rs:16168-16180`). Les mutations et la boîte humaine ajoutent `control_client_actor` (`daemon.rs:9573`, `9772`, `9802`). `ControlStateRead` seul ne le fait pas.
- **Décision** : `DelegationOverviewRead` exige la négociation ET `control_client_actor`. Elle ouvre les métadonnées de tous les parents : c'est une lecture sensible.
- **Borne** : `issuer_scope` est déclaré par le client (ADR 011). Un processus hostile du même compte n'est pas arrêté.
- **Complexité** : O(n log n), n ≤ 4096 tâches conservées (borne 150).

## R2 - Arrêt global composé avec la pause dans la garde unique

- **Constat** : `admit_autonomous_effect` ne lit que `ControlStateFrame.paused` (`referent_control.rs:105-133`). ADR 027 : « un puits qui ne l'appelle pas est un défaut ».
- **Décision** : l'arrêt reste un état séparé, mais `admit_native_delegation(control, halt)` vit dans `referent_control.rs` et rend une seule décision. Table à 4 combinaisons. Arrêt prioritaire sur pause. Rejeu avant admission (ordre de 150).
- **Annulation** : seulement avec `--cancel-active`, par `human_cancel` (`delegation_lineage.rs:225`) une fois par arbre.
- **Alternative rejetée** : réutiliser `paused` : la pause interrompt aussi rondes et relances.

## R3 - Stockage et idempotence de l'arrêt

- **Constat** : `control_events` a un `CHECK` dérivé de `ControlEventKind::ALL` ; ajouter un genre exige de reconstruire la table (`migrate_rights_columns`). `command_id UNIQUE` y sert de rejeu.
- **Décision** : ligne d'état unique + table de reçus `native_delegation_halt_commands` (256 au plus). Un `command_id` connu rend l'état courant sans réécriture. Séquence A, B, rejeu de A : l'état reste « repris ». Les reçus sont persistants.
- **Limite** : un `command_id` plus ancien que 256 commandes serait ré-appliqué. Chaque geste de CLI crée un identifiant neuf.
- **Historique** : journal du daemon (`info!`) et ADR. Pas d'audit en base.

## R4 - Budget de tours : retiré (raisons vérifiées)

- `execution_store.rs:793` : l'agrégat fait `turns = turns + 1` par échantillon (`ON CONFLICT(execution_id, generation, source)`). Il compte les échantillons reçus, sans identifiant de tour ni déduplication.
- `claude_stream_json.rs:1379` : un échantillon par événement `result` de Claude (le code évite le doublon `assistant`/`result`).
- `wrapper.rs:6372-6378` : les sources ACP et Codex sont ignorées (« source ACP/Codex non autorisée pour L4 »). Aucun tour Codex n'est compté.
- Spec 153 FR-016/017 : chaque ronde a une mission distincte (`execution-<mission>`, `native_delegation.rs:901`). Le cumul par tâche n'existe pas.
- `execution_store.rs:818` : le calcul agrège par CTE récursive sur `execution_continuations`, sans borne de profondeur.
- **Décision** : pas de budget de tours en 158a. Le temps reste borné par `reply_timeout` (150).
- **Conditions de reprise (session future)** : identifiant de tour provider-neutre ; adaptateur Codex ; agrégation de toutes les exécutions de la tâche, rondes et reprises comprises ; réussite au tour N acceptée et refus d'ouvrir le tour N+1 (pas d'échec dès `turns >= N`) ; évaluation pilotée par l'événement d'usage (compteur incrémenté à l'écriture), pas par un balayage récursif périodique ; ou, à défaut, CTE avec `LIMIT` et test de charge qui prouve la borne.

## R5 - Boîte humaine

Aucun dépôt de 158a. Le genre `budget_reached` (`protocol.rs:1272`) reste libre pour la future borne.

## R6 - Portées MCP : table par action et test exhaustif

- **Constat T3** : `McpToolAccess.ts` rend la déclaration obligatoire par les types.
- **Constat Bridget** : `tools()` est un `Vec<Value>` (`mcp.rs:1891`), l'exécution un `match` sur le nom (`mcp.rs:470-598`). Une garantie de compilation obligerait à réécrire ~30 déclarations.
- **Décision** : `TOOL_SCOPES` + test exhaustif dans les deux sens + cohérence `readOnlyHint`. Garantie de CI, déclarée comme telle.
- **Fils** : `bridget_thread` `read` crée un reçu qui rejoue la même page tant que l'ACK manque (`store/threads.rs:1549-1565`). `read` et `ack` sont donc de l'état propre. `post`, `create`, `add_members`, `close` sont des écritures de fil.
- **Surface** : le test existant refuse toute mutation par `bridget_control_status` (`mcp.rs:2707-2708`). La règle « aucun outil ne pose l'arrêt » le généralise.

## R7 - Plafond lecture seule

- **Constat T3** : plafond `read-only` par défaut pour un client externe.
- **Constat Bridget** : la tâche garde `effective_posture` (`delegation.rs:40`) ; la sortie d'admission utilise `task.effective_posture.or(posture)` (`native_delegation.rs:83`). Une posture omise est résolue à l'admission. Spec 153 FR-013/014 : un enfant inactif reste vivant entre rondes.
- **Décision** : plafond `read_only` s'il existe une tâche dont l'enfant est l'appelant, sans `cleanup_done`, de posture effective `discovery`. Il ne dépend pas de `terminal()`.
- **Non vérifié** : coût de la recherche par nom d'enfant. T017 mesure avant le code. Repli : cache par nom d'enfant.
- **Borne** : ADR 011 inchangée.

## R8 - Réveils (158b) : ce qui existe

- Boucle de 1 s du daemon (`daemon.rs:4697-4710`) : `native_delegation::tick`, relances, `dispatch_thread_wakes` (`daemon.rs:7812`). Un planificateur s'y greffe.
- `WakeRow` (`store/threads.rs:102`) gère hors ligne, DND, débit, reprise : un réveil lié à un fil peut l'utiliser.
- `scripts/bridget-ronde.py` : patron d'un planificateur externe qui appelle la CLI.
- Règles T3 (`scheduledTasks/Schedule.ts`) : minimum 1 minute, grâce de 10 minutes, rattrapage unique d'un intervalle.
- Un réveil est un effet autonome : nouvelle variante dans la garde commune de `referent_control.rs`.
- Programmer un réveil vers un autre agent ou un fil est un effet extérieur (portée `sends`), avec droits de projet et de fil vérifiés avant admission.

## R9 - Rejeu des faits et secret (158b)

- `queue_observation_notifications` abandonne la notification si l'abonné n'a pas de route, est en DND, ou si la file de 64 est pleine, et incrémente `observation_lost` (`daemon.rs:1427-1495`). Seul l'abonnement est persisté.
- Décision visée : garder les seuls faits non remis dans une table bornée. Un fait jamais vu par le daemon n'est pas récupérable ; la lacune est datée.
- Secret (T3) : référence `secret-ref:<hex32>`, 24 h, consommée par l'outil qui la demande (`SecretRequests.ts`). Bridget n'a aucun outil consommateur. Reporté.

## R10 - Webhook

Hors périmètre (voir `spec.md`). `bridget wake` couvre le besoin par un relais local.
