# Audit de reutilisation de l'existant — 087 Reprendre le contrôle

## Decision

Statut: PASS
Date: 2026-09-02
Feature dir: /Users/moi/Nextcloud/10.Scripts/bridget/specs/087-reprendre-controle

Conclusion courte: aucune duplication évidente. Deux réutilisations manquaient au plan et ont été ajoutées : la garde de continuation `reserve_governed_continuation` avec son état `AutonomyRuntimeState::Paused`, qui est le sixième producteur de travail autonome, et la classe de priorité des soumissions d'exécution pour le focus. Deux collisions de noms ont été levées par renommage (`control_state` face à `AttachRelayControlState`, `budget_cap` face au budget de capture Maicie). Le plan a été refactoré, sections « Réutilisation de l'existant » et « Divergences volontaires ».

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 22 |
| Items audites | 22 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 7 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 9 |
| Specs existantes applicables | 11 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Attestation d'origine humaine, cinq vérifications | `ObjectiveOpeningPermit::human_request`, `human_message_content_seal`, `AttestationConsumption` | `plugins/maicie/src/domain.rs:240`, `:356`, `:313` | Il ne manque que l'émetteur daemon ; fixtures réutilisables dans `plugins/maicie/tests/human_origin_attestation.rs:15-25` |
| Occurrences différées des routines | `RoutineOccurrence { state: Differee, reason }` | `plugins/maicie/src/routines.rs:445-455` | Trois motifs ajoutés au vocabulaire existant |
| Réduction de réassignation | `reduire_reassignation`, appelé par le store | `plugins/maicie/src/domain.rs:1351`, `store.rs:6492` | Nouvelle variante de réduction, même réducteur pur |
| Gel de la base pour le focus | `review_git.rs` et `ReviewTarget` | `plugins/maicie/src/review_git.rs`, `crates/bridget-transport/src/protocol.rs:935` | Aucun second mécanisme de mesure de tête |
| Attribution du principal humain | `resolve_sender_attribution`, `UI_HUMAN_AGENT_ID` | `crates/bridget-daemon/src/daemon.rs:1481`, `:105` | Même borne que l'ADR-011 |
| Garde TTY côté CLI | `confirm_local_profile_approval` | `plugins/maicie/src/main.rs:1313-1321` | Motif de refus identique |
| Dépôt idempotent au guichet | `ServiceRequest` → `deposit_guichet`, `GreffeAuthorizationGate` | `crates/bridget-daemon/src/daemon.rs:11167-11321` | La provenance voyage dans l'enveloppe idempotente |
| Relève en pull par Maicie | `GuichetClaimNext` / `GuichetReply`, `open_store_with_reconciliation` | `plugins/maicie/src/main.rs:514-566` | Les décisions de boîte suivent le même schéma |
| Rappel périodique des items | thread horaire du daemon | `crates/bridget-daemon/src/daemon.rs:4537` | Aucun nouveau thread |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Pause de l'autonomie (producteurs) | `reserve_governed_continuation(policy, runtime: AutonomyRuntimeState, …)` et `evaluate_autonomy_budget` : `Paused` existe, personne ne le pose | `crates/bridget-daemon/src/managed_supervisor.rs:36-51`, `fleet.rs:91-106` | **Réutiliser** : la pause globale devient `AutonomyRuntimeState::Paused` pour toute continuation gouvernée. Ajouté au plan comme producteur n° 6 |
| Plafond d'objectifs auto-générés | `AutonomyBudgetPolicy { max_duration_secs, max_facturable_tokens, max_descendants }` (par exécution) ; `FleetConfig.quota` (taille de flotte) | `fleet.rs:60-88` | Distinct : ces bornes comptent des secondes, des tokens, des descendants ou des agents, pas des objectifs. Le nom `auto_objectives_cap` évite la confusion ; `ExecutionBudgetOutcome::BudgetLimit` n'est pas réemployé pour ne pas mélanger deux grandeurs (mémoire projet : une constante partagée fusionne deux grandeurs) |
| Focus, agent libre proposé en premier | `WorkSubmission.priority_class` (valeur unique `"normal"`) | `crates/bridget-core/src/execution.rs:99, 125` | **Réutiliser** : les soumissions issues d'une délégation focus portent `priority_class = "focus"` ; aucun nouveau champ |
| Boîte de réception humaine | `attention_events` (`human_input_needed`, `task_completed`, `terminal_failure`), `client_notification_preferences`, notification native ADR-016 | `crates/bridget-daemon/src/agent_profile.rs:205-231`, `docs/decisions/016-notifications-ui-preuves-agent.md` | Distinct : les événements d'attention sont des faits par agent, non résolubles, sans options. La boîte les complète. **Réutiliser** la notification native ADR-016 dans `app.js` pour signaler un item, pas un second mécanisme de notification |
| Boîte : dépôt par Maicie | `notification_outbox` et `reconcile_notification_one` | `plugins/maicie/src/store.rs:8635`, `reconcile.rs:456` | Réutiliser la file avec un destinataire réservé `human` traité par une branche dédiée de `reconcile_notification_one`, à confirmer à l'implémentation (le destinataire n'est pas routé, donc pas soumis à `validate_agent_id`) |
| Module `control_state.rs` | `AttachRelayControlState` dans `wrapper.rs` | `crates/bridget-daemon/src/wrapper.rs:2664` | Collision de nom, concept différent. Module renommé `referent_control.rs` ; la table garde `control_state` |
| Pause globale et DND | statut « ne pas déranger » par agent, `presence.is_dnd()`, `undisturbed` dans `collect_reminder_actions` | `specs/005-domaines-dnd/spec.md`, `daemon.rs:6921-6925` | Distinct : DND est par agent et coupe la réception ; la pause est globale et coupe la production autonome. La garde de pause se place à côté de `undisturbed`, même style |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucune | | | |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XIX | Pas d'abstraction sous trois usages, pas de wrapper passthrough | Deux modules neufs seulement, chacun avec sa table et ses appelants multiples (daemon, CLI, UI, Maicie) |
| `~/.speckit/constitution.md` XVIII | Complexité annotée | Comptage du budget O(n) annoté ; n borné |
| Mémoire projet « additif sur le fil n'est pas additif à la source » | Un champ `serde(default)` casse les initialiseurs exhaustifs | Tâche dédiée : tous les `ServiceRequestPayload::Delegate { … }` mis à jour, gate `cargo test --workspace --no-run` |
| Mémoire projet « un correctif peut désarmer son propre oracle » | Tester l'effet, pas la présence | Chaque garde de pause a un test d'effet observable et un mutant |
| Mémoire projet « chercher ce que le compilateur ne contraint pas » | Un producteur oublié est vert et muet | Inventaire fermé de six producteurs, un test par ligne |
| Mémoire projet « jamais une enum du domaine recopiée en SQL » | `CHECK (kind IN (…))` diverge | Le `kind` de `human_inbox` dérive son `IN (…)` d'une fonction `as_sql` sur l'enum, comme `define_etat_delegation` |
| `docs/regles-chantier.md` règle 17 | Le numéro de migration de schéma est une ressource globale sérialisée à la main | Avant d'écrire la migration v24 de Maicie : `git grep "SCHEMA_VERSION: i64 = 24"` sur toutes les branches distantes |
| `docs/regles-chantier.md:340` | « Le guichet est sa boîte de dépôt » | Le focus passe par le guichet, pas par un canal parallèle |
| `docs/decisions/011` | Garde d'approbation par exposition acceptée, second facteur différé | La boîte est conçue pour accueillir le second facteur ; l'ADR 027 le dit |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 005 domaines-dnd | Statut par agent, toggle, visible dans `who` | Le pied de `who` gagne la ligne `Contrôle : …`, même rendu |
| 011 maicie-orchestration | Maicie compagnon hors processus, outboxes réconciliées | Lecture de l'état de contrôle à la relève, jamais de copie |
| 012 contrat-client-idempotent | `command_id`, `expected_generation`, rejeu | `ControlStateSet` et `HumanInboxResolve` suivent ce contrat |
| 015 guichet-maicie | Dépôt durable, relève pull, refus fermés | Focus et décisions de boîte |
| 016 (ADR) notifications UI | Notification native sur geste explicite, page ouverte seulement | Réutilisée pour les items de boîte ; le canal externe couvre la page fermée |
| 063 interruption tour humain | Message humain prioritaire dans un tour | La pause n'interrompt pas les tours ; la spec 063 reste le seul chemin d'interruption |
| 064 plan de contrôle | `ExecutionBudgetOutcome`, continuations gouvernées | Producteur n° 6, réutilisé |
| 068 / 070 | Faits d'incident et preuves d'activité | Producteurs d'items `intervention_required` et `reply_debt` |
| 075 cycle de vie agents | Flotte, quotas, `FleetConfig` | Le plafond d'objectifs ne touche pas au quota de flotte |
| 079 / 081 rondes | `ProjectRoundRefusal` fermé, politique par projet | `ControlPaused` ajouté au vocabulaire fermé |
| 080 centre de contrôle | `server_setting_descriptors`, prévisualisation puis application | Le plafond est exposé comme descripteur de réglage serveur |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n control_state` | `crates`, `plugins` | 10 occurrences, toutes `AttachRelayControlState` (`wrapper.rs:2664`) : collision de nom |
| `rg -n -i human_inbox\|inbox` | `crates`, `plugins` | aucun équivalent (5 occurrences hors sujet dans un test de parité) |
| `rg -n -i focus\|focus_queue` | `crates`, `plugins` | 49 occurrences, toutes DOM `focus()` dans `ui.rs` ; aucun concept métier |
| `rg -n budget_cap\|plafond\|BudgetLimit` | `crates`, `plugins` | `status_capture_budget_ms` (Maicie, capture attach) ; `AutonomyBudgetPolicy` (`fleet.rs:80`) ; aucun plafond d'objectifs |
| `rg -n AutonomyRuntimeState\|reserve_governed_continuation` | `crates` | `fleet.rs:91`, `managed_supervisor.rs:36` : `Paused` défini, jamais posé |
| `rg -n priority_class` | `crates` | `execution.rs:99,125` : champ présent, valeur unique `normal` |
| `rg -n attention_events\|client_notification_preferences` | `crates` | `agent_profile.rs:205-231` : notifications par agent, ADR-016 |
| `rg -n notification_outbox\|deferred_delegation_dispatch` | `plugins` | files existantes de Maicie, réutilisables |
| `rg -n telegram\|human-channel` | dépôt | aucune occurrence : le canal vit hors dépôt |
| `rg -n '"control"\|"inbox"\|"focus"' cli.rs` | `crates/bridget-daemon/src/cli.rs` | aucune sous-commande existante ; `dnd` en `cli.rs:154` |
| `ls tests/features` | `tests/` | convention Gherkin en français, `Fonctionnalité` / `Scénario` |
| `ls docs/decisions` | `docs/` | prochain numéro libre 027 (012 et 022 sont dupliqués, à ne pas reproduire) |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Pause des continuations | reutiliser `AutonomyRuntimeState::Paused` via `reserve_governed_continuation` | Vocabulaire et garde déjà écrits pour ce cas exact par SPEC-064 ; un second mécanisme serait un doublon | 2026-09-02 |
| Priorité du focus | reutiliser `priority_class` | Champ existant, valeur `focus` ajoutée ; aucun nouveau champ ni table | 2026-09-02 |
| Nom du module daemon | creer nouveau `referent_control.rs` | Évite la confusion avec `AttachRelayControlState` ; la table reste `control_state` | 2026-09-02 |
| Nom du plafond | creer nouveau `auto_objectives_cap` | Évite `budget_cap`, déjà employé pour le budget de capture, et ne réemploie pas `BudgetLimit` qui mesure une autre grandeur | 2026-09-02 |
| Signalement des items dans l'interface | reutiliser la notification native ADR-016 | Un seul mécanisme de notification locale | 2026-09-02 |
| Dépôt Maicie vers la boîte | approfondir à l'implémentation : `notification_outbox` avec destinataire réservé `human` | Réutilisation probable ; à confirmer contre `validate_agent_id` et les migrations de participants | 2026-09-02 |
| Refactor de `plan.md` | appliqué sans confirmation utilisateur | Pipeline autonome, utilisateur absent ; modifications strictement additives, aucune suppression ; réversibles | 2026-09-02 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
