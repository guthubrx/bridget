# Contre-revue adverse PLAN158A

**Verdict : APPROVE_WITH_CHANGES.** Je n'ai trouvé aucune contradiction bloquante, aucun gain de droits et aucun doublon. Sept corrections sont nécessaires avant le code. Les deux premières sont les plus importantes.

Je n'ai modifié aucun fichier et n'ai rien lancé (ni Cargo, ni test, ni build). Tout vient de la lecture du WT à `bd37e927`.

## Vérifié et conforme

- **Négociation et acteur humain.** Les gardes de `daemon.rs` (~15890-15935) refusent d'abord `NegotiationRequired`, puis `CapabilityNotNegotiated`. `ControlStateSet` et `HumanInbox*` exigent `ControlStateV1` seule. L'acteur vient de `control_client_actor`, qui lit `issuer_scope` (`daemon.rs:10713`), jamais `from`. Les nouvelles requêtes sont donc atteignables par le bon chemin. Il ne faut pas copier le bras `ControlStateRead`, qui accepte aussi `|| Lookup` (`daemon.rs:~15912`).
- **Matrice exhaustive sans `_`** (`daemon.rs:~16000`). Une nouvelle variante ne compile pas tant qu'elle n'est pas classée. T002/T008 doivent donc dire dans quel bras ranger les 4 variantes `Delegation*` : le bras strict, comme `ControlStateSet`.
- **Rejeu avant garde.** À `native_delegation.rs:1067`, `by_agent_request` précède la garde de pause. Placer l'arrêt au même endroit respecte FR-008.
- **Clé libre après refus.** `record_refusal` (`delegation.rs:657`) fonctionne par liste blanche de 5 codes. `delegations_halted` n'y figure pas, donc le refus n'est pas retenu et la clé reste libre. Ajoutez seulement un test miroir de `native152_refus_transitoire_jamais_garde_referent_paused`.
- **Contrat157.** Le plan ne touche ni `fallback`, ni `substitution`, ni Probe/Reserve/Commit. L'arrêt est relu à chaque passe dans la même base, donc cohérent.

## Corrections nécessaires

**1. Le plafond MCP n'a pas de mécanisme d'accès à la tâche (T017c).**
- Seul le daemon ouvre `DelegationStore` (`daemon.rs:3438`). Le processus MCP n'a qu'une socket (`execute_tool_at_with_identity(identity, name, args, socket)`).
- `DelegationStore::open` fait un `Immediate` et des `CREATE TABLE/INDEX` (`delegation.rs:296-352`). Il ne convient pas à un second processus.
- `for_child` (`delegation_lineage.rs:146`) n'a aucun appelant aujourd'hui. Il ne rend que la première ligne, et `json_extract(child)` n'a aucun index. Le coût est un parcours de ≤4096 lignes par appel MCP, ce que T017b doit mesurer.
- Correction : écrire dans le plan le raccord en lecture seule sans DDL (`SQLITE_OPEN_READ_ONLY`, requête SQL isolée, sans `open`). Écrire aussi la règle d'échec. Je recommande de refuser fermé, avec un code réessayable, tout ce qui dépasse `writes_own_state` si la lecture échoue. Les lectures restent permises.
- Le nom de l'enfant est un `Uuid::new_v4()` (`native_delegation.rs:1324`), donc unique. Je n'ai pas vérifié qu'il égale `identity.name` en production. T017 doit le prouver par un test avec un vrai enfant.

**2. Quatre sites utilisent `AutonomousEffect::NativeDelegation`, T011 n'en nomme qu'un.**
- Les sites sont :
  - `native_delegation.rs:805` (ouverture des demandes enfant) ;
  - `native_delegation.rs:1073` (Delegate) ;
  - `native_delegation.rs:1644` (approbation d'une demande enfant) ;
  - `native_delegation.rs:1781` (`bridget_task_continue`, nouvelle ronde).
- Sans précision, l'arrêt serait plus faible que la pause. Une ronde 153 sur un enfant gardé reste admise pendant l'arrêt alors qu'elle est refusée en pause (`referent_paused`).
- Correction : écrire dans T011 que 1073 et 1781 passent par `admit_native_delegation`. Écrire que 805 et 1644 restent soumis à la pause seule, parce qu'ils achèvent un travail existant. Ajouter un test de `continue` pendant l'arrêt.

**3. `--cancel-active` : définition d'« arbre actif » et courses (FR-009, T012).**
- `human_cancel(root, task_id, …)` (`delegation_lineage.rs:470`) ne passe en `cancelling` que si `!task.terminal() || (guarded && result_available)`. Sur une racine terminale non gardée qui a un descendant actif, il ne fait rien et `tick` ne cascade pas.
- Correction : définir la cible comme les tâches actives les plus hautes dans l'arbre (aucun ancêtre actif). Annuler chacune avec `root = root_owner()`.
- Le plafond global de 4096 reçus d'annulation (`:479`) peut rendre `ResourceLimit`. Le nombre « visé » reste alors supérieur au nombre réellement annulé. Documentez-le.
- Pour la course « A pose, B reprend, C naît, A annule C » : calculer la liste de cibles avant le commit du reçu. Garder le verrou `st` jusqu'à la fin des appels `human_cancel`, comme le fait `native_lineage::handle`. Lancer `tick` après.

**4. Libellé de `countcancelled` à propager.**
- Votre clarification est cohérente avec le code. Mais FR-010 écrit « nombre d'annulées », `data-model.md` « nombre d'annulées », le contrat `cancelled: n`, et SC-003 « 0 tâche active ne reste ».
- Correction : reformuler partout en « nombre d'arbres visés, figé avant le commit du reçu ». Renommer le champ (par exemple `targeted`) pour qu'il ne mente pas. Ajouter la réserve « hors crash et hors échec d'arrêt physique » à SC-003. Le test `control158_halt_commande_rejouee` doit vérifier que le nombre rejoué est le nombre visé figé.

**5. `readOnlyHint` : la plupart des outils n'ont aucune annotation (FR-023, T016).**
- Seuls `bridget_request_wait`, `bridget_queue` et les outils de `delegation_mcp` en portent une (`mcp.rs:2259`, `:2405`). Thread, events, journal, send, who, ledger, status et les autres n'en ont pas.
- « Corriger les annotations divergentes » sous-estime donc le travail. Le test « vrai si et seulement si base = `reads` » impose d'ajouter `readOnlyHint:true` à environ 8 outils de lecture. Cela change le catalogue vu par les hôtes et peut réduire leurs demandes de permission.
- Correction : écrire dans T016 que l'absence d'annotation vaut faux. N'ajouter que `readOnlyHint:true` sur les outils de base `reads`, et le consigner dans l'ADR.

**6. `bridget_journal` : le prédicat « sans `to` » doit être celui de l'exécution.**
- À `mcp.rs:713`, `to` est un `Option<String>` désérialisé. `null` donne `None`, mais `""` donne `Some("")`, qui va vers `execute_send`.
- Correction : T015 doit abaisser à `reads` seulement si `to` est absent ou `null`. Toute autre valeur, `""` comprise, garde la base `sends`. Ajouter ce cas aux tests `mcp158_discovery_journal_avec_et_sans_to`. Même principe pour `action` : valeur inconnue ou absente donne la base, donc le refus.

**7. Deux canaux de refus dans le contrat.**
- Les refus `negotiation_required` et `capability_not_negotiated` sortent aujourd'hui de la garde pré-dispatch sous forme de `ClientRefusal`, avant le gestionnaire. `DelegationControlRejected` ne peut pas les porter.
- Correction : dire dans le contrat quelle trame porte quel motif. Je suggère `ClientRefusal` pour les deux premiers, comme `ControlStateSet`. `DelegationControlRejected` porterait le reste. Aligner les tests de T008 et le CLI.

## Points mineurs

- Le contrat classe en « indicatif » mais omet `bridget_request_wait`, `bridget_queue`, `bridget_task_wait`, `bridget_task_ack`, `bridget_task_continue` et `bridget_task_respond`. Le test exhaustif les attrapera. Ce serait plus propre de les lister.
- `bridget_capabilities` gagne la clé `delegations_halted`. FR-027 ne couvre que `bridget_delegate`. Vérifier qu'aucun test existant ne compare ce résultat octet par octet.
- Pendant l'arrêt sans `--cancel-active`, les tâches déjà en `queued` ou `mission_pending` partent encore (`NativeMissionDispatch` n'est pas touché). C'est cohérent avec « rien d'actif n'est touché ». Écrivez-le dans la doc.
- Le plafond ne s'applique qu'à la façade MCP. Un enfant `discovery` avec accès shell peut encore appeler la CLI `bridget`. Je n'ai pas vérifié l'identité que la CLI porte dans ce cas. Ajoutez cette limite honnête à l'ADR 158, au même titre que la borne ADR 011.
- `data-model.md` donne `standard` à une tâche sans posture. `delegation_lineage.rs:71` affiche `Discovery` par défaut pour une tâche sans posture. L'écart ne touche que les tâches antérieures à 149, qui sont soldées. Notez-le.

## Limites de ma lecture

- Je n'ai lu ni `research.md`, ni `adversarial-review-codex.md`, ni `quickstart.md`, ni le reçu157 complet.
- Je n'ai pas lu l'implémentation du CLI `cmd_control` ni `require_interactive_terminal`.
- Je n'ai exécuté aucun test et n'ai déduit aucun résultat de test ou d'audit global.
