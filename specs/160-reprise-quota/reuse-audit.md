# Audit de réutilisation de l'existant - 160-reprise-quota

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/160-reprise-quota

Conclusion courte: Le plan réutilise les primitives de 152 (chaîne d'exécutions,
garde de pause, journal des pauses, échéance effective, décision de reprise) et de
157 (mémoire de limites, épuisement attesté). Il extrait deux fonctions à deux
usages. Il crée une table et deux champs, tous justifiés par une exigence
d'entrée. Aucun module nouveau. Aucune duplication évidente. Les primitives de
152 et 157 sont planifiées à la date de cet audit : la porte T001 les prouve sur
le code livré.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 8 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Dernière exécution de la chaîne | `latest_in_chain` | 152 `data-model.md`, T010 | feuille de plus grande génération, aussi utilisée pour la feuille d'un `failed` |
| Garde de pause pour la relance | `AutonomousEffect::NativeMissionDispatch` | 152 T009, T019 | aucune variante nouvelle |
| Sortie du gestionnaire `DeliveryRejected` | `rejected()` qui qualifie la pause | 152 T013, T044 ; `daemon.rs:18078-18125` | généralisé à trois valeurs |
| Mesure du gel | `pause_overlap` et journal `control_events` | 152 T050 ; `referent_control.rs` | `pause_intervals` en est extrait |
| Échéance effective, suivi de demande | `effective_deadline`, `frozen_secs` | 152 T017, T018 | seul l'appel à la mesure change |
| Décision de reprise après panne | `prepare_restart`, `restart_resumes` | 152 T008, T023, T025 | exception pour l'attente, compteur intact |
| Mémoire de limites, épuisement par fenêtre | `provider_limits`, `Quota` | 157 `data-model.md`, T004, T014 | une fonction de lecture ajoutée |
| Remise du message repris | queue de `schedule_execution_recovery` | `daemon.rs:4361-4480` | extraite en `deliver_reconstructed` (2 usages), date d'émission en paramètre, `Replayed` repris |
| Enveloppe de rejeu | `canonical` stocké à part de `request` | `delegation.rs:114-130` | `resume_on_limit` entre dans l'enveloppe |
| Définition épinglée | `scoped.resolved_definition` de la tâche | `native_delegation.rs` (admission) | garantit l'absence de substitution |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Reconstruction d'un `failed` | `reconstruct_active_for_agent` | `execution_store.rs:1296-1450` | Non réutilisable : exclut `failed` (`:1327`, `:1378`, `:1395`), raison de continuation en dur (`:1427`), sélection par agent avec `LIMIT 2`. Fonction ciblée justifiée (research R4) |
| Identité de relance | `execution_continuations` | `execution_store.rs:1588` | Pas de champ d'identité (clé primaire sur l'enfant, `reason` libre). Table dédiée justifiée (research R5) |
| Autorisation d'une continuation | `reserve_governed_continuation` | `managed_supervisor.rs:64` | Conçue pour parents actifs ou inactifs. Non utilisée (D-H), relue en T002 |
| Avis d'échec durable | `native-failure-<task_id>`, `failure_sent` | `native_delegation.rs:1048-1068` | réutilisé à la place d'un rejeu du message direct (objection 5) ; aucune extraction `notify_delivery_failure` |
| Trame `DeliveryRejected` et liaison d'exécution du wrapper | `ManagedExecutionBinding` (`execution_id`, `generation`) | `wrapper.rs:5519-5527`, `protocol.rs:4164` | deux champs optionnels ajoutés à la trame, remplis depuis la liaison existante ; pas de nouvelle corrélation |
| Quarantaine des remises natives | `schedule_idempotent_delivery_recovery` | `daemon.rs:4195-4215`, `send_delivery.rs:294` | non contournée : la relance échoue explicitement si sa remise est mise en quarantaine |
| Abandon d'une remise | `mark_delivery_indeterminate` | `send_delivery.rs:529` | utilisé par l'annulation pendant `relaunching` |
| Reçus de refus de 152 | `native_delegation_refusals` | 152 `data-model.md` | Sans rapport : la relance n'est pas un refus |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | O(1), O(c), O(P + Q log Q) annotés |
| `~/.speckit/constitution.md` XIX | minimalisme | pas de module, extractions à deux usages |
| `~/.speckit/constitution.md` VIII | forensic avant correction | T002, tests rouges T003 à T008 |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |
| contrat 148 | pas de substitution implicite | FR-015, T026 |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | rejeu par `request_id`, enveloppe | `resume_on_limit` dans l'enveloppe |
| 149-sous-agents-lineage | Lineage à états fermés | aucun état nouveau, tâche `working` |
| 150-socle-delegation | `reply_timeout`, ordre des refus | échéance effective repose sur `reply_timeout` |
| 152-reprise-missions | reprise, gel de pause, `restart_resumes` | primitives réutilisées, compteur de pannes intact |
| 157-catalogue-quotas | mémoire de limites, `fallback` | qualification, absence de substitution |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `sed -n 1296,1450p execution_store.rs` | reconstruction | exclusion de `failed`, raison en dur |
| `grep "'waiting_user_input', 'interrupting'"` | `execution_store.rs` | `:531`, `:1327`, `:1378`, `:1395`, `:1467`, `:1481` |
| `grep CREATE TABLE execution_continuations` | `execution_store.rs` | `:1588`, `mode` fermé, `reason` libre |
| `grep execution_schema_migrations` | `execution_store.rs` | versions 1 à 11, prochaine 12 |
| `sed -n 18060,18135p daemon.rs` | gestionnaire `DeliveryRejected` | suppression du suivi et notification du parent |
| `sed -n 6215,6262p wrapper.rs` | publication de l'échec | transition `failed` puis `DeliveryRejected` |
| `sed -n 4228,4520p daemon.rs` | `schedule_execution_recovery` | queue extractible, garde native, gouvernance |
| `grep reserve_governed_continuation` | `managed_supervisor.rs` | `:64`, sources `InactiveParent`, `RecoveryAfterIdleWrapper` |
| `git show origin/main:.../UsageLimitRecoveryWorker.ts` | T3 | identité, garde de fenêtre passée, armement puis relance |
| lecture `specs/152-reprise-missions/*` | primitives | noms et tâches citées |
| lecture `specs/157-catalogue-quotas/*` | mémoire de limites | `Seen`, `Quota`, FR-008 |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Reconstruction d'un `failed` | fonction ciblée | la reprise ordinaire exclut `failed` et sa raison est fixe | 2026-10-11 |
| Où garder l'identité | table dédiée | `execution_continuations` n'a pas de champ d'identité | 2026-10-11 |
| Compteur de rondes | longueur d'une liste sans retrait | un seul fait, porte aussi les intervalles de gel | 2026-10-11 |
| Gel | union d'intervalles dérivée | 152 dérive déjà le gel, aucune écriture à perdre | 2026-10-11 |
| Gouvernance de continuation | non utilisée | qualification plus entrée de tâche, plus stricte | 2026-10-11 |
| Avis informatif au parent | non | question 3, défaut | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
