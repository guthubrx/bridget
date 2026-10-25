# Recherche - Session158 contrôle humain, portées MCP, réveils

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-1).

## R1 - Vue globale par le plan de contrôle, réservée à l'humain attesté

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-2).

## R2 - Arrêt global composé avec la pause dans la garde unique

- **Constat** : `admit_autonomous_effect` ne lit que `ControlStateFrame.paused` (`referent_control.rs:105-133`). ADR 027 : « un puits qui ne l'appelle pas est un défaut ».
- **Décision** : l'arrêt reste un état séparé, mais `admit_native_delegation(control, halt)` vit dans `referent_control.rs` et rend une seule décision. Table à 4 combinaisons. Arrêt prioritaire sur pause. Rejeu avant admission (ordre de 150).
- **Annulation** : seulement avec `--cancel-active`, par `human_cancel` (`delegation_lineage.rs:225`) une fois par arbre.
- **Alternative rejetée** : réutiliser `paused` : la pause interrompt aussi rondes et relances.

## R3 - Stockage et idempotence de l'arrêt

- **Constat** : `control_events` a un `CHECK` dérivé de `ControlEventKind::ALL` ; ajouter un genre exige de reconstruire la table (`migrate_rights_columns`). `command_id UNIQUE` y sert de rejeu.
- **Décision** : ligne d'état unique + table de reçus `native_delegation_halt_commands` (256 au plus). Un `command_id` connu rend la réponse ENREGISTRÉE, marquée rejouée, sans réécriture ni parcours d'annulation ; l'état courant reste inchangé. Séquence A, B, rejeu de A : l'état reste « repris ». Les reçus sont persistants.
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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-3).

## R7 - Plafond lecture seule

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-4).

## R8 - Réveils (158b) : ce qui existe

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-5).

## R9 - Rejeu des faits et secret (158b)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/research.md#extrait-6).

## R10 - Webhook

Hors périmètre (voir `spec.md`). `bridget wake` couvre le besoin par un relais local.

## Préparation de réemploi sur source157 gelée (non livrée)

Lecture ciblée du2026-10-12 dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas`.
Ce chemin est une préparation, pas le reçu de livraison157. ROOT doit comparer
les fonctions et empreintes ci-dessous à la base livrée avant le GO158A.
Aucun test158, coût mesuré ou verdict de livraison n'est revendiqué ici.

| Point | Réemploi confirmé par lecture | Limite / tâche158A |
|---|---|---|
| Autorité | daemon::control_client_actor délègue à referent_control::human_principal_actor ; issuer_scope négocié, pas fromalias | BorneADR011 conservée ; T008/T012, après négociation/capacité. |
| Négociation | daemon refuse NegotiationRequired puis CapabilityNotNegotiated ; ControlStateV1 figure parmi les capacités accordées | Capacité différente de l'acteur ; mêmes refus fermés dans le contrat158. |
| CLI | cli::require_interactive_terminal et cmd_control existent | Réemployer pour T009/T013 ; aucun nouveau transport. |
| Annulation | DelegationStore::human_cancel conserve un reçu par root/request_id, sans refaire l'effet au rejeu ; tick existant | T008/T012 ; reçus256 de halt distincts, ne pas étendre les reçus d'annulation par confort. |
| Admission157 | by_agent_request précède la gardeNativeDelegation, puis parent_fact ; Reserve/Commit réemploient request+définition brute+substitution épinglés par Probe ; Commit revalide dans le dossier enfant réel | T004/T011 doivent composer halt/pause à ce même puits, sans sélectionner une autre cible, perdre les ajouts157 ou changer les gardes154/155. |
| Plafond | DelegationStore::for_child existe ; effective_posture et cleanup_done existent ; lookupSQL par json_extract(child) | T017 mesure le coût avant choix d'index/cache. Aucun résultat de mesure n'est disponible ici. |
| SurfaceMCP | mcp::tools étend le catalogue avec delegation_mcp::tools ; le dispatch contient wait/continue/ack/respond/cancel et queue156 ; journal avecto appelle execute_send | T015/T016 classent le catalogue réellement livré157 et vérifient readOnlyHint/base. Pas de classification prétendue complète ici. |

Empreintes de la source candidate relue (pas un SHA de livraison) :

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon.rs — `0995839a6118274b00c3747253980acd362f08b23ea67273ff37dd42e385568f` ; plages 10713–10717,15896–15934,17578–17598

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/referent_control.rs — `5a1720d9c79a102c3ccca0aa92f08e0b23402e8026b957f7a4cde4febe96f9f0` ; plages 31–38,131–142

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/daemon/native_delegation.rs — `038ce196f2c7f4ed0ef0cb2f0e8882a5d8fd41a90a1032ccfcd333a2079dd04d` ; plages 1018–1094,1130–1145,1194–1206,1216–1228

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/delegation_lineage.rs — `de49575af4fdbd68d104579c5b824210c92b747f72a357526cf5b062f910dda3` ; plages 146–149,470–499

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/cli.rs — `2a8e7c30daad48d595191770b05a1ee9c432627dcceed23a6322b27cb68248bd` ; plages 5961–5968,6096–6110

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/mcp.rs — `04978c36d8955070004d01800f9b6b0c9160718fd377a101372b7fd53ccbdf3b` ; plages 618–646,708–724,2247–2258,2593–2599

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/delegation_mcp.rs — `1b4a85f2d251a67c60001b7fdc745f4dd4d02d60261711b15cde1a91aa768b77` ; plages 67–80

/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/157-catalogue-quotas/crates/bridget-daemon/src/delegation.rs — `13e41d83a0f4367f4f6eb3437486c4fd822f6a1231568f3d379f2bfbd5706713` ; plages 192–217


## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.

## Réemploi/contre-revue fraîche158A après livraison157

Index UNIQUE native_delegations_child confirmé dans delegation_lineage.rs126 ; for_child146 le réutilise. Le MCP séparé reçoit identité/socket, donc CallerScope interne fournit le booléen via l'identité inscrite du daemon ciblé, sans DB cliente ni panne convertie en standard. L'arrêt compose la garde de Delegate/Continue ; enfant/open/approve restent pause seule. Le count précommit est celui des plus hautes racines actives visées, calculé seulement sur nouvelle commande via FnOnce ; réception historique au rejeu, jamais un nombre physique confirmé. Ces décisions n'ajoutent ni Saga, table, outil, scénario, tâche ou fonctions158B. La baseline fraîche est exécutée et reçue par ROOT ; l'auditeur n'exécute aucun test.

## Extension158B après livraison158A

La partie B complète US5–US7. Son artefact de référence est /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/research-158B.md. Les preuves et choix158A ci-dessus restent historiques et conservés.
