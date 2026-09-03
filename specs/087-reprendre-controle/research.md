# Research: Reprendre le contrôle

Date : 2026-09-02. Sources : code d'origin/main `bf89d4c`, ADR 003, 011, 014, 015, cartes d'exploration du 02/09, métriques d'efficience (fenêtre D).

## R1. Qui possède l'état de contrôle (pause, budget) ?

**Décision** : le daemon Bridget, table `control_state` à une ligne dans `bridget.db`, module `crates/bridget-daemon/src/control_state.rs`.

**Rationale** : les producteurs de travail autonome sont répartis : rondes et relances dans le daemon, routines et réassignations dans Maicie. Maicie ouvre déjà une connexion au daemon à chaque commande (`main.rs:514`, `open_store_with_reconciliation`) ; lire une trame de plus est gratuit. L'inverse, le daemon lisant la base Maicie, est interdit par l'ADR-003 et déjà violé une fois (`identity_migration.rs:140`) ; on ne l'aggrave pas.

**Alternatives écartées** : un fichier JSON partagé (deux écrivains, pas d'atomicité entre processus) ; la config Maicie (`config.json`, non modifiable par l'interface, `deny_unknown_fields`).

## R2. Qui est « le référent » pour le daemon ?

**Décision** : le principal humain est reconnu par l'attribution d'émetteur existante (`resolve_sender_attribution`, `daemon.rs:1481`) : une connexion du relais UI émettant sous `UI_HUMAN_AGENT_ID`, ou une commande CLI qui exige un terminal interactif comme `confirm_local_profile_approval` (`plugins/maicie/src/main.rs:1313`). Les mutations d'état de contrôle exigent la capacité `ControlStateV1` dans `ClientHello`, que la façade MCP (`mcp.rs:789-810`) ne demande jamais.

**Rationale** : c'est la même borne que l'ADR-011, écrite et acceptée : un agent local avec un shell peut tout faire, y compris tuer le daemon ; la garde vise l'agent qui agit par ses outils déclarés, pas l'agent hostile. Elle est consignée comme telle dans l'ADR 027.

**Résidu confirmé par la contre-revue (Jim, 02/09)** : `replace_stale_ui_human_route` (`daemon.rs:3169-3200`) laisse toute connexion `type=ui` sous `UI_HUMAN_AGENT_ID` évincer la route UI vivante, donc obtenir l'attribution humaine. Aucune preuve structurelle n'est disponible dans le même compte. Parade retenue : quand la route remplacée est encore vivante (écrivain joignable, présence fraîche), le daemon dépose un item `human_route_replaced` dans la boîte et le pousse sur le canal externe, que l'agent ne lit pas ; le référent voit l'usurpation en moins d'une minute. FR-011 et SC-005 sont bornés en conséquence.

**Alternatives écartées** : vérifier le TTY du pair par `/proc/<pid>/fd/0` (Linux seulement, fragile sous tmux) ; un jeton secret (déplace le problème, un agent lit les fichiers).

## R3. Comment la voie « demandé par l'humain » de l'ADR-014 se ferme-t-elle ?

**Décision** : le daemon fabrique `DelegateOrigin::Human { message_id, observed, attestation }` quand un dépôt `ServiceRequestPayload::Delegate` arrive du principal humain, et refuse (`ServiceRefusal::HumanOriginForbidden`) tout `origin` fourni par un autre émetteur. Il enregistre d'abord le message humain au ledger (`record_message_in_transaction`, cible `maicie`), puis calcule `signature = human_message_content_seal(observed)` et `canonical_request_sha256` sur les octets canoniques du dépôt. Maicie construit le permit par `ObjectiveOpeningPermit::human_request(...)` (`domain.rs:240`), qui rejoue les cinq vérifications, et consomme le `message_id` dans `human_origin_consumptions` (usage unique).

**Rationale** : `human_request` existe déjà, avec ses refus fermés ; il ne manque que l'émetteur de l'attestation. Ce que l'attestation prouve reste ce que l'ADR-014 dit : causée par un message précis, non altéré, non rejoué ; l'identité de l'auteur vaut ce que vaut l'attribution d'émetteur.

**Alternatives écartées** : laisser le client Maicie ou MCP déclarer l'origine (forgeable, refusé par l'ADR-014) ; une trame séparée « attestation » (course entre dépôt et preuve).

## R4. Un focus, ou plusieurs ?

**Décision** : un seul focus actif, une file FIFO derrière, table Maicie `focus_queue (objective_id PK, position, opened_at)`. Une seconde demande exige une confirmation explicite « remplacer » ou « mettre en file » portée par le champ `on_conflict` de la requête.

**Rationale** : la spec fixe un référent unique ; deux priorités simultanées reproduiraient l'ambiguïté qu'on corrige.

## R5. Comment le focus gèle-t-il la base et transmet-il les identifiants ?

**Décision** : Maicie réutilise `review_git.rs` pour mesurer la tête de `origin/<branche par défaut>` du projet (`git ls-remote`, cf. mémoire « un lot peut n'exister que sur origin ») et construit `ReviewTarget`. L'instruction de délégation (`app.rs:1729`, `delegation_instruction`) gagne un bloc `IDENTIFIANTS DE DÉPÔT` avec `objective_id`, `delegation_id`, `message_id`, aujourd'hui recopiés à la main par le référent (`docs/regles-chantier.md:421-427`).

**Rationale** : tout ce que le référent recopie est calculable ; `review_git` mesure déjà, on ne crée pas de second mécanisme.

## R6. Où vit la boîte de réception, et comment le canal externe est-il appelé ?

**Décision** : table `human_inbox` dans `bridget.db`, module `human_inbox.rs`. Dépôt idempotent par `dedup_key`. Producteurs : le daemon (relances au palier 3, focus en attente d'agent, budget atteint relayé) et Maicie par la trame `HumanInboxDeposit` sous `ServiceCapability::HumanInboxV1`, alimentée par `notification_outbox` (`store.rs:8635`) avec un destinataire réservé `human`. Décisions relevées par Maicie en pull (`HumanInboxDecisions`), comme le guichet ; la relève ne marque rien côté daemon. Maicie applique la décision dans sa transaction, enregistre `decision_id` dans `human_decisions_applied`, puis envoie `HumanInboxAck { decision_id }` idempotent ; le daemon pose alors `acked_at`. Un plantage entre relève et application relit la même décision à la relève suivante (contre-revue Jim, objection 3b retenue). Le précédent est le guichet : `guichet_requests` et `guichet_lifecycle_events` vivent dans le daemon et portent charges et réponses Maicie sans que l'ADR-003 soit violée. Canal externe : le daemon lit `~/.config/bridget/human-channel.json` (`{"command": ["/chemin/absolu/…", "…"]}`) et lance la commande avec un résumé JSON borné sur stdin ; échec consigné dans `notify_attempts_json`, jamais bloquant. Le rappel des items anciens est fait par le thread horaire existant du daemon (`daemon.rs:4537`), qui devient la seule décision « de ronde » autorisée par la spec.

**Rationale** : la boîte est un fait du plan de contrôle ; Maicie n'a pas de processus résident pour pousser vers l'extérieur. Le canal Telegram vit hors dépôt (skill `telegram-founder`) : une commande configurée le réutilise sans dépendance.

**Alternatives écartées** : boîte dans Maicie (pas de canal sortant, pas d'interface directe) ; client HTTP Telegram dans le daemon (dépendance et secret dans le dépôt).

## R7. Où le budget est-il compté et appliqué ?

**Décision** : le plafond est dans `control_state` (défaut 5). Maicie l'applique à l'ouverture : `delegate` avec permit `AutoGenerated` compte les objectifs ouverts d'origine `AutoGenerated` (O(n), n borné) et refuse par `DelegateError::BudgetReached`. Les routines consignent `RoutineOccurrence { state: Differee, reason: "budget" }` (mécanisme existant, `routines.rs:445`). Un item `budget_reached` est déposé avec `dedup_key = "budget-reached"` : tant qu'un item ouvert existe, aucun doublon.

**Rationale** : le producteur qui ouvre est Maicie ; compter ailleurs créerait une course entre lecture et ouverture.

## R8. Comment la pause atteint-elle chaque producteur sans oubli ?

**Décision** : inventaire fermé des producteurs, chacun avec sa garde et son test d'effet :

| Producteur | Lieu | Garde | Preuve d'effet |
|---|---|---|---|
| Ronde par projet | `daemon.rs:9887-9910` | `ProjectRoundRefusal::ControlPaused` avant `PolicyDisabled` | `record_project_round_dispatch` en `Refused` avec ce motif |
| Relances des demandes suivies | `collect_reminder_actions`, `daemon.rs:6914` | paliers 1 et 2 non émis en pause ; palier 3 conservé (information) | `ReminderAction` absent, événement `deferred` consigné |
| Routines | `routines.rs:440-500` | occurrence `Differee` motif `pause` avant `delegate` | ligne `routine_occurrences` |
| Réassignation | `store.rs:6492`, `reduire_reassignation` | réduction `Differee { motif: pause }` sans génération | `reassignment_reductions` |
| Dispatch différé de délégations | `deferred_delegation_dispatch` | non relevé en pause | compteur `status --json` |
| Continuations automatiques d'exécution (ajouté par l'audit) | `reserve_governed_continuation`, `managed_supervisor.rs:36` | `AutonomyRuntimeState::Paused` | `GovernedContinuation::Budget(Paused)` |
| Tours fournisseur déjà en cours (amendement 2026-09-03) | `execution_store.rs`, `daemon.rs` | interruption interne à la pose, reprise contrôlée à la levée | `control_pause_interruptions`, commande `control-pause-<gen>-<execution>` |
| Rejeu des outboxes de délégation à chaque ouverture Maicie (ajouté par la contre-revue) | `reconcile_pending`, `reconcile.rs:353-368` | outboxes d'origine `AutoGenerated` non poussées | `status --json`, `delegation_outbox.state` inchangé |
| Déblocage d'une dépendance qui matérialise une outbox (ajouté par la contre-revue) | `store.rs:7724-7886`, `deferred_delegation_dispatch` → `delegation_outbox` | matérialisation différée, motif `pause` | ligne `deferred_delegation_dispatch` conservée avec motif |

**Décision révisée le 2026-09-03 par le référent** : un tour fournisseur déjà en cours n'est plus admis pendant la pause. La pose mémorise l'exécution dans `control_pause_interruptions` avant d'émettre l'interruption interne ; la levée reconstruit seulement ces parents, immédiatement si l'agent est connecté et libre, sinon au prochain enregistrement. La réconciliation de flotte reste admise : elle relance des agents, pas du travail.

**Conception retenue** : une garde unique par plan, `admit_autonomous_effect(effect, snapshot) -> Admitted | Deferred { motif }`, dans `plugins/maicie/src/control.rs` pour les puits Maicie (routines, réassignation, rejeu d'outbox, déblocage de dépendance, dispatch différé) et dans `crates/bridget-daemon/src/referent_control.rs` pour les puits daemon (ronde, relances, continuations). Chaque puits appelle la garde, chaque appel a son test d'effet et son mutant.

**Rationale** : mémoire projet « chercher ce que le compilateur ne contraint pas » : un producteur oublié est silencieux et vert. La liste est un artefact de tasks, chaque ligne une tâche avec son mutant.

## Baseline research (Article IX)

Consulté `~/.speckit/research/01-ai-agents-agentic-ai.md` : le red flag « agent washing » et la recommandation de garde-fous humains explicites (Guardian Agents) vont dans le sens de cette spec : rendre observable et bornable l'autonomie plutôt que l'augmenter. Pas de validation live nécessaire : la feature ne choisit aucun framework ni fournisseur.
