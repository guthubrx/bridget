# Audit de reutilisation de l'existant - 158-reveils-controle

## Decision

Statut: PASS
Date: 2026-10-11 (révisé après contre-revue)
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/158-reveils-controle

Conclusion courte: 158a réutilise la négociation de contrôle, l'acteur humain
attesté, l'annulation humaine et la garde de l'ADR 027 (étendue, pas dupliquée).
Un seul module est créé (`mcp_scopes.rs`). Le budget de tours est retiré : le
compteur existant n'est pas réutilisable en l'état. Aucune duplication.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 11 |
| Items audites | 11 |
| Reutilisations deja prevues | 6 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lecture globale et gating | négociation `ControlStateV1` + `control_client_actor` | `cli.rs:5860-5900`, `daemon.rs:9393`, `daemon.rs:16168-16180` | acteur humain exigé en plus de la capacité |
| Acteur humain attesté | `human_principal_actor` | `referent_control.rs:28-43` | même autorité que la boîte humaine |
| Terminal interactif | `require_interactive_terminal` | `cli.rs:5902` | mutations CLI |
| Annulation d'une tâche | `human_cancel` | `delegation_lineage.rs:225` | idempotent par `request_id` |
| Décision d'admission | `admit_autonomous_effect` | `referent_control.rs:105-133` | étendue par `admit_native_delegation` dans le même fichier |
| Rejeu de commande | patron `command_id` du journal de contrôle | `referent_control.rs:107` | reçus bornés, état séparé |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision |
|---|---|---|---|
| Budget de tours | `execution_budget_facts`, `record_execution_usage` | `execution_store.rs:793-870`, `wrapper.rs:6372-6378` | Non réutilisé : échantillons non dédupliqués, Claude seul, remise à zéro par ronde, CTE non bornée. Retiré. |
| Journal de l'arrêt | `control_events` et `ControlEventKind::ALL` | `referent_control.rs:47-70` | Non étendu : `CHECK` à reconstruire en production. |
| Boîte humaine | `HumanInboxKind::BudgetReached` | `protocol.rs:1272` | Non utilisé en 158a (budget retiré). |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | O(n log n) borné ; plafond mesuré en T017 |
| `~/.speckit/constitution.md` XIX | minimalisme | un module ; budget de tours retiré |
| `~/.speckit/constitution.md` XX | honnêteté sur les limites | garantie par CI ; reçus bornés ; borne ADR 011 |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique | admission commune pause et arrêt |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 087 | plan de contrôle, `ControlStateV1` | base de la CLI |
| 148, 149, 150 | contrat de délégation, `human_cancel`, ordre des refus | `delegations_halted` dans l'ordre |
| 151 à 157 | nouveaux outils MCP, rondes (153), posture | classés par le test exhaustif ; plafond suit la vie de l'enfant |
| 145-147 | vue Lineage T3 | aucun changement de forme |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `grep -n "fn tools" mcp.rs` | `crates/bridget-daemon/src` | catalogue `Vec<Value>`, aucune portée |
| lecture de `execution_store.rs:785-830` | compteur d'usage | `turns + 1` par échantillon |
| lecture de `wrapper.rs:6372-6378` | sources d'usage | ACP et Codex ignorés |
| lecture de `daemon.rs:16168-16180` | négociation | `ControlStateV1` accordée à tout client demandeur |
| lecture de `store/threads.rs:1535-1565` | reçu de lecture de fil | même page tant que l'ACK manque |
| lecture de `delegation.rs:30-50` | posture | `effective_posture` présent |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Module de portées | creer `mcp_scopes.rs` | contrat testable ; `mcp.rs` fait 5415 lignes | 2026-10-11 |
| Budget de tours | retirer | aucun tour attesté pour Claude et Codex | 2026-10-11 |
| État d'arrêt | table à une ligne + reçus | évite la migration de `control_events` | 2026-10-11 |
| Garde | étendre `referent_control.rs` | ADR 027 : garde unique | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
