# Audit de reutilisation de l'existant — 156-guidage-file

## Decision

Statut: PASS
Date: 2026-10-11 (révisé après contre-revue)
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/156-guidage-file

Conclusion courte: Le plan expose des capacités déjà présentes (intentions,
transports, pont T3, saga de remise, liens d'agents) et ferme des failles par
une garde unique. Il crée un module pur (`guidance.rs`), une table d'audit, un
index et une paire de trames client, chacun justifié. L'édition de file est
retirée du périmètre : aucune trame vers wrapper, aucune colonne, aucune phase.
Aucune duplication évidente. Six vérifications ouvertes sont affectées à T002.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 10 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Intention portée par le message | `MessageIntent::{SteerCurrent, InterruptAndStart}` | `bridget-core/src/message.rs:54-60` | aucun type neuf |
| Injection d'un guidage | `deliver` des transports | `codex_app_server.rs:859-927`, `claude_stream_json.rs:310-340`, `acp.rs:620-660` | aucun code d'injection neuf |
| Chemin d'envoi | `SendIdempotent`, canon, rejeu | `daemon.rs:8584-8654`, test canon `:23255-23270` | l'intention y est déjà dans le canon |
| Refus mémorisé | `reject_idempotent_send` | `daemon.rs:7694-7718` | refus de la garde, comportement des refus d'envoi existants |
| Admission d'un tour d'interruption | `admit_starting_message_for_project` | `daemon.rs:8828-8850` | déjà appelée pour `InterruptAndStart` |
| Garde de pause | `admit_autonomous_effect` + enum | `referent_control.rs:105-133` | une variante, comme `NativeDelegation` (150) |
| Parent d'une cible | `agent_link_for_child` | `idempotency/agent_links.rs:79-94` | pas de nouveau registre de filiation |
| Parent natif | struct `Task` de délégation | `delegation.rs` | lecture seule |
| Pont T3 | `dispatch_command` et `deliveryIntent` | `t3code_contract_v2.rs:255-272`, `t3code.rs:2206` | généralisé de booléen à valeur |
| Entretien horaire | `run_maintenance` | `daemon.rs:7177-7240` | purge de l'audit |
| Style des arguments MCP | schéma `bridget_send` | `mcp.rs:2028-2046` | même style pour `intent` |
| Refus de capacité | `ClientCapability` + `CapabilityNotNegotiated` | `daemon.rs:14541-14610` | `QueueListV1` suit le patron |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Édition de file | `CancelDelivery`, `cancel_delivery` | `protocol.rs:4147-4151`, `codex_app_server.rs:1081-1093` | Non utilisés : sans résultat, et annulent le tour actif. Reportés avec prérequis (R8) |
| File durable de messages | `execution_queue`, `take_next_submission` | `execution_store.rs:456-500` | Non réutilisée : aucun consommateur de production (R11) |
| Annulation d'une demande suivie | `bridget_cancel` | `daemon.rs:18125-18200` | Documenté, non modifié |
| Audit de décisions de contrôle | `control_events` | `referent_control.rs` | Non réutilisée : `kind` fermé par `CHECK`, réservée à la pause |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | R12 |
| `~/.speckit/constitution.md` XIX | minimalisme | périmètre réduit, un outil en lecture |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique par puits | `AgentGuidance` ; pause interne exemptée |
| `docs/decisions/007-socle-idempotence.md` | rejeu explicite | reçu = `idempotency_records` |
| `docs/decisions/044-relai-sous-agent-sous-autorite-parent.md` | preuve enfant limitée à l'annuaire et à l'envoi | refusée pour l'intention et `bridget_queue` |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| mémoire `bridget-sessions-127-132` | tests `t3code`, `mcp::`, transport pour tout texte ou trame | T001, T026 |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 087 (pause) | interruption mémorisée, reprise à la levée | exception interne explicite ; pas de conflit |
| 099/138 (envois et projets) | garde de projet, motif interprojet | appliquée après la garde de droits |
| 111-114, 129 (pont T3) | file 64, lots, remise en cours de tour | inchangés ; guidage hors lot et prioritaire |
| 148-149 | droits hérités, lignée | parent natif lu |
| 150 | `AutonomousEffect`, refus nommés | patron suivi ; acquis supposés livrés |
| 133 | preuve enfant | refusée |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "MessageIntent::"` | `crates/` | transports, daemon, stockage ; aucun appel MCP/CLI |
| `rg take_next_submission` | `crates/` | tests seulement |
| `rg CancelDelivery` | `crates/` | protocole, daemon, pont T3 ; sans résultat |
| `rg ExecutionControlV1` | `crates/` | négociable par tout client (`daemon.rs:16174`) |
| `rg "MessageOrigin::Human"` | `crates/` | pas de producteur hors tests sauf le dépôt de service |
| `rg "deliveryIntent"` | Bridget et T3 | `steer` seul côté Bridget ; `auto/steer/restart` côté T3 |
| `rg "fn deliverable_while_busy"` | `t3code.rs:3126` | claude/cursor, groupables seulement |
| lecture T3 `CommandPolicy.ts` | T3 `origin/main` | `steer` peut devenir `interrupt_restart` |
| `rg tombstone\|rate_limit` | `crates/` | aucun compteur d'interruption existant |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Module de garde | creer nouveau (`guidance.rs`) | fonction pure, table de vérité ; `daemon.rs` > 30 000 lignes | 2026-10-11 |
| Table `guidance_events` | creer nouveau | audit et rythme ; pas un reçu | 2026-10-11 |
| Trames `QueueList` | creer nouveau (client seulement) | lecture ; aucune trame vers wrapper | 2026-10-11 |
| Édition de file | ne pas faire | saga durable requise (contre-revue 5 à 8, 10) | 2026-10-11 |
| Intention `auto` | ne pas faire | dégradation silencieuse | 2026-10-11 |
| Champs `mid_turn_ok`, `interrupt_active` | creer nouveau (optionnels) | le pont ignore la parenté ; `cancel_delivery` mélange retrait et interruption (ronde 2) | 2026-10-11 |
| Méthode `cancel_queued` | creer nouveau (défaut faux, ACP seul) | retrait sans toucher au tour actif | 2026-10-11 |
| Preuve humaine indépendante | ne pas faire | hors modèle (ADR 011) ; rôle attach réutilisé | 2026-10-11 |
| Intention effective | ne pas faire | invisible ou asynchrone (contre-revue 3, 4) | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les règles projet applicables ont été lues
- [x] Les specs existantes proches ont été vérifiées
- [x] Le plan.md a été refactoré ou les divergences sont justifiées
