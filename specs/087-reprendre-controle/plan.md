# Implementation Plan: Reprendre le contrôle

**Branch**: `087-reprendre-controle` | **Date**: 2026-09-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/087-reprendre-controle/spec.md`

## Summary

Donner au référent quatre leviers durables sur le coordinateur : une pause de l'autonomie, un objectif prioritaire d'origine humaine attestée (focus), une boîte de réception des décisions qui l'attendent, et un plafond d'objectifs auto-générés. L'approche suit la frontière existante entre les plans : l'état de contrôle et la boîte de réception vivent dans le daemon Bridget, le focus et le budget s'appliquent dans Maicie qui lit l'état de contrôle à chaque relève. La voie « demandé par l'humain » de l'ADR-014 est fermée en faisant produire l'attestation par le daemon au moment où le principal humain dépose une délégation au guichet. Aucune nouvelle dépendance, aucun nouveau protocole : trois trames ajoutées au contrat existant, une table par base, une migration Maicie.

## Technical Context

**Language/Version**: Rust 1.92 (workspace existant), JavaScript vanilla pour l'interface embarquée
**Primary Dependencies**: `rusqlite`, `serde`, `sha2` déjà présents ; aucun ajout
**Storage**: `~/.cache/bridget/bridget.db` (table `control_state`, table `human_inbox`), base Maicie (migration v24 : table `focus_queue`, table `human_origin_consumptions`)
**Testing**: `cargo test` par crate et par périmètre déclaré, `node --test` pour `app.js`, scénarios Gherkin dans `tests/features/`
**Target Platform**: Linux serveur (cartae.app) et macOS (poste), même binaire
**Project Type**: daemon + CLI + plugin CLI + relais HTTP loopback + front embarqué
**Performance Goals**: aucun objectif chiffré nouveau ; la lecture de l'état de contrôle par Maicie est une trame par relève, soit une toutes les deux minutes
**Constraints**: additif sur le fil ET à la source (tout initialiseur de `ServiceRequestPayload::Delegate` mis à jour) ; aucune lecture directe de `bridget.db` par l'interface ; l'attestation d'origine humaine n'est jamais acceptée d'un client, seulement fabriquée par le daemon
**Scale/Scope**: une installation, un référent, quelques dizaines d'objectifs ouverts au plus

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Article | Vérification | Verdict |
|---|---|---|
| I Langue | Tous les artefacts en français | OK |
| III Processus SpecKit | spec → plan → audit-existing → tasks → implement | OK, ce plan |
| VII ADR | Décision structurante : propriétaire de l'état de contrôle et voie humaine attestée | ADR 027 à créer en phase implement |
| XV Test-before-next | Gates par périmètre déclaré ; la suite lib du daemon est rouge sur main (147) indépendamment de ce lot, cf. mémoire projet | Périmètres : `bridget-transport` complet, `bridget-daemon --lib control:: ui:: project_`, `maicie` ciblé, node UI |
| XVI Worktree | Branche `087-reprendre-controle` ; implémentation dans un worktree serveur `/home/moi/bridget-referent/.worktrees/087-reprendre-controle` | OK |
| XVIII Complexité | Comptage du budget : O(n) sur les objectifs ouverts, n borné par le plafond et le focus ; annoté | OK |
| XIX Minimalisme | Pas de nouveau crate, pas de nouvelle dépendance ; réutilisation de `RoutineOccurrence::Differee`, `review_git`, guichet, `notification_outbox`, `resolve_sender_attribution`, `ObjectiveOpeningPermit::human_request` | OK |
| XX Responsabilité | Chaque levier a un test d'effet (mutant : retirer la garde doit rendre un test rouge), pas seulement de présence | OK, exigé dans tasks |
| Mémoire projet : « un correctif peut désarmer son propre oracle » | Les gardes de pause sont testées par effet observable (occurrence différée consignée), pas par absence | OK |

Violation à justifier : ajout de logique dans `daemon.rs` (25 000 lignes). Justification : les bras de `handle_wrapper_message` restent des délégations d'une ligne vers un nouveau module `control_state.rs` ; aucune logique métier nouvelle n'est écrite dans `daemon.rs`.

## Project Structure

### Documentation (this feature)

```text
specs/087-reprendre-controle/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── control-state-v1.md
│   ├── human-inbox-v1.md
│   ├── delegate-origin-v1.md
│   └── ui-routes-v1.md
├── checklists/requirements.md
└── tasks.md                     # phase suivante
```

### Source Code (repository root)

```text
crates/bridget-transport/src/
├── protocol.rs                  # + ClientCapability::ControlStateV1, ServiceCapability::HumanInboxV1
│                                # + WrapperToDaemon::{ControlStateRead, ControlStateSet, HumanInboxList, HumanInboxResolve, HumanInboxDeposit, HumanInboxDecisions}
│                                # + DaemonToWrapper::{ControlState, HumanInbox, HumanInboxDeposited, HumanInboxDecisionsBatch}
│                                # + ServiceRequestPayload::Delegate.origin : Option<DelegateOrigin>
│                                # + ProjectRoundRefusal::ControlPaused
crates/bridget-daemon/src/
├── referent_control.rs          # NOUVEAU : table control_state + control_events, lecture/écriture, garde du principal humain
├── managed_supervisor.rs        # runtime = AutonomyRuntimeState::Paused quand la pause est active (producteur n° 6)
├── human_inbox.rs               # NOUVEAU : table human_inbox, dépôt idempotent, résolution, poussée externe, rappel
├── daemon.rs                    # bras de dispatch (délégation vers les modules), garde ronde ControlPaused, relances en pause,
│                                # fabrication de DelegateOrigin pour le principal humain, alerte human_route_replaced
├── execution_store.rs           # ordre de file : priority_class "focus" avant "normal"
├── store.rs                     # DDL des deux tables au démarrage (même discipline que l'existant)
├── cli.rs                       # bridget control pause|resume|budget|status, bridget inbox list|resolve, pied de `who`
├── ui.rs                        # routes /v1/control/state, /v1/control/focus, /v1/inbox, /v1/inbox/<id>/resolve — via socket uniquement
└── assets/ui/app.js, theme.css  # bandeau d'état de contrôle, saisie « Travaille sur… », panneau boîte de réception, réglage du plafond
plugins/maicie/src/
├── control.rs                   # NOUVEAU : lecture de l'état de contrôle par relève, décisions de différé (pause / focus / budget)
├── bridget_client.rs            # read_control_state, deposit_human_inbox, fetch_human_decisions
├── routines.rs                  # occurrence Differee avec motif pause / focus / budget
├── store.rs                     # migration v24 : focus_queue, human_origin_consumptions ; comptage AutoGenerated ouverts
├── domain.rs                    # focus dans le payload d'objectif ; réduction de réassignation « différée »
├── greffe_service.rs, guichet.rs# permit HumanRequest depuis DelegateOrigin ; refus si absent ; focus à l'ouverture
├── app.rs                       # delegate : budget, focus, gel de base par review_git, identifiants dans l'instruction
├── reconcile.rs                 # ordre : focus d'abord ; relève et acquittement des décisions de boîte ; dépôt des items humains ; focus en attente d'agent
└── main.rs                      # maicie status --json : bloc control ; maicie focus close|queue
tests/features/087-reprendre-controle.feature
docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md
```

**Structure Decision**: aucun nouveau crate. Deux modules neufs dans le daemon, un dans Maicie. Les fichiers existants ne reçoivent que des points d'accroche.

## Réutilisation de l'existant

Ajouté après l'audit de réutilisation ([reuse-audit.md](reuse-audit.md)), le 2026-09-02.

| Besoin | Existant réutilisé | Preuve |
|---|---|---|
| Pause des continuations automatiques d'exécution, **producteur n° 6** absent de R8 | `reserve_governed_continuation(policy, runtime: AutonomyRuntimeState, …)` ; la pause globale se traduit par `AutonomyRuntimeState::Paused`, issue `ExecutionBudgetOutcome::Paused` déjà définie | `crates/bridget-daemon/src/managed_supervisor.rs:36-51`, `crates/bridget-daemon/src/fleet.rs:91-106` |
| Agent libre proposé au focus d'abord (FR-015) | `WorkSubmission.priority_class`, valeur `focus` ajoutée à côté de `normal` | `crates/bridget-core/src/execution.rs:99,125` |
| Signalement local d'un item de boîte | notification native ADR-016 dans `app.js` | `docs/decisions/016-notifications-ui-preuves-agent.md` |
| Exposition du plafond dans Paramètres du serveur | `server_setting_descriptors` et le couple prévisualisation puis application de SPEC-080 | `crates/bridget-daemon/src/control_settings.rs:137` |
| Attestation humaine, garde TTY, relève pull, occurrences différées, réducteur de réassignation, gel par `review_git`, thread horaire | voir R3, R5, R6, R8 de research.md | `plugins/maicie/src/domain.rs:240`, `main.rs:1313`, `routines.rs:445`, `store.rs:6492`, `daemon.rs:4537` |

Renommages issus de l'audit : le module daemon s'appelle `referent_control.rs` (la table reste `control_state`) pour ne pas se confondre avec `AttachRelayControlState` ; le plafond s'appelle `auto_objectives_cap` pour ne pas se confondre avec `status_capture_budget_ms`.

## Divergences volontaires

| Choix | Existant proche non retenu | Raison |
|---|---|---|
| Plafond compté en objectifs | `AutonomyBudgetPolicy` (secondes, tokens, descendants), `FleetConfig.quota` (agents) | Grandeurs différentes ; les fusionner créerait une constante à deux sens |
| Boîte de réception distincte des `attention_events` | `attention_events` par agent, sans options ni résolution | Un item de boîte porte une décision à prendre et un état résolu ; un événement d'attention est un fait à voir |
| Pause globale distincte du DND | statut « ne pas déranger » par agent | Le DND coupe la réception d'un agent ; la pause coupe la production autonome du système |

## Phase 0 : recherche

Voir [research.md](research.md). Les huit inconnues sont tranchées ; aucune ne reste ouverte.

## Phase 1 : conception

- [data-model.md](data-model.md) : entités, tables, états, invariants.
- [contracts/](contracts/) : trames de protocole et routes de l'interface, versionnées.
- [quickstart.md](quickstart.md) : parcours opérateur réel, sans ligne de commande.

Mise à jour du contexte agent : le script `update-agent-context.sh` n'existe pas dans `.specify/scripts/bash/` de ce projet ; `AGENTS.md` est régénéré par `sync-project.py`, rien à faire ici.

## Corrections issues de la contre-revue adverse

Voir [adversarial-review-jim.md](adversarial-review-jim.md), 2026-09-02. Retenu : garde unique `admit_autonomous_effect` par plan, deux puits ajoutés (rejeu d'outbox, déblocage de dépendance), acquittement des décisions après commit Maicie avec `decision_id`, alerte `human_route_replaced` quand une route humaine vivante est remplacée, FR-011 et SC-005 bornés au modèle coopératif. Rejeté avec preuve : la violation d'ADR-003 par la boîte (précédent du guichet), la reprise des remises et la réconciliation de flotte comme producteurs de travail.

## Constitution Check après conception

Inchangé. Un point ajouté : la garde « seul le référent lève la pause » repose sur l'attribution d'émetteur existante et sur un terminal interactif côté CLI, ce qui est la même borne que l'ADR-011 ; elle est écrite dans l'ADR 027 avec sa limite, pas présentée comme une authentification.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| Deux tables neuves dans `bridget.db` sans propriétaire de schéma unique | La consolidation des bases est hors périmètre (spec distincte) | Attendre la consolidation retarderait le frein d'urgence ; les tables suivent la discipline actuelle de `store.rs` et seront reprises avec les autres |
| Un champ ajouté à `ServiceRequestPayload::Delegate` | La provenance doit voyager avec le dépôt, dans la même enveloppe idempotente | Une trame séparée créerait une course entre le dépôt et sa provenance |
