# Audit de reutilisation de l'existant - 153-rondes-contexte

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/153-rondes-contexte

Conclusion courte: Le plan étend les modules existants et ne crée ni module, ni
état, ni dépendance. Une seule table d'index est créée (missions passées),
justifiée par l'Art. XVIII. Le brief réutilise tel quel le validateur du
dossier de passation. La fin explicite réutilise l'outil d'annulation. Une seule
extraction de code est prévue (contrôle des droits de `spawn_task`), avec deux
usages. Les éléments de la session 150 ne sont pas relus dans le code (non
livré dans le checkout lu) : ils sont audités par leur spec et marqués « à
reconfirmer en T001 ».

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 17 |
| Items audites | 17 |
| Reutilisations deja prevues | 12 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Validation et rendu du brief | `handoff::validate_and_render` | `crates/bridget-daemon/src/handoff.rs:460` | `pub(crate)`, même crate ; aucune modification |
| Bornes du brief (16 Kio) | `MAX_BODY_BYTES` | `handoff.rs:15` | refus sans troncature déjà codé |
| Schéma MCP du brief | schéma du brouillon de `bridget_handoff` | `crates/bridget-daemon/src/mcp.rs:1977-2005` | même objet `draft` |
| Fin explicite d'un enfant | `Cancel` / `bridget_task_cancel` / `human_cancel` | `native_delegation.rs:518`, `delegation_lineage.rs:225` | comportement ajouté, pas d'outil neuf |
| Arrêt de l'enfant et de sa descendance | `cancel_tree` | `native_delegation.rs:1380-1480` | inchangé, garde levée avant |
| Rejeu et enveloppe | `serde_json::to_vec(&request)` + `envelope_mismatch` | `native_delegation.rs:413-415` | même schéma ; digest SHA-256 via `sha2` |
| Écriture atomique et projection | `increment`, `committed`, patron de `human_cancel` | `delegation_lineage.rs:66`, `:107`, `:225-250` | `begin_round` suit ce patron |
| Garde de pause | `AutonomousEffect::NativeDelegation` (150) | spec 150, T004 et T015 | à reconfirmer en T001 |
| Comptage des places | requêtes indexées de 150 | spec 150, T013 | une clause ajoutée (`holds_slot`) |
| Contrôle des droits avant ronde | contrôle de `spawn_task` | `native_delegation.rs:535-553` | extrait, 2 usages |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Enchaînement de continuations | `AutonomousEffect::Continuation` | `crates/bridget-daemon/src/referent_control.rs:113` | Non réutilisable : continuation d'exécution automatique, pas une ronde de tâche ; on utilise `NativeDelegation` |
| Rondes de projet | `AutonomousEffect::ProjectRound` | `referent_control.rs:109` | Sans rapport (réveil par projet). Nom proche : documenter la différence |
| Carte de reprise de l'enfant | `managed_resume_context`, `bounded_resume_card` | `crates/bridget-daemon/src/wrapper.rs:303-392` | Non réutilisable : faits Git de l'enfant, pas le contexte du parent |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | R13 ; annotations dans T010 et T015 |
| `~/.speckit/constitution.md` XIX | minimalisme | un outil, zéro état, zéro table |
| `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` | garde unique par puits autonome | la ronde passe par la garde de la délégation |
| `docs/decisions/150-retention-et-delai-delegation-native.md` | purge sans perte, pause, délai | prédicats complétés, pas remplacés |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 103-handoff | dossier de passation v1, 16 Kio | brief |
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id` | compatibilité des octets canoniques |
| 149-sous-agents-lineage | neuf états fermés, contrat Lineage | aucun état nouveau |
| 150-socle-delegation | purge, pause, délai de réponse | prédicats, garde, délai de la tâche |
| 151 / 152 (en cours de rédaction) | résultat structuré, attente, reprise | couplage à vérifier en T001 |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "keep_alive\|keepalive\|idle_deadline"` | `crates/` | aucun équivalent (hors keepalive réseau sans rapport) |
| `rg "round" -i` dans `native_delegation.rs`, `delegation*.rs` | daemon | aucun concept de ronde de tâche |
| `rg "validate_and_render"` | `crates/` | `handoff.rs:460` et `mcp.rs:513` (via `parse_request`) |
| `rg "native-result-"` | `crates/` | 3 lieux : `native_delegation.rs:680`, `:1216` (+ tests) |
| `rg "BRIDGET_SAFE_MCP_TOOLS"` | `crates/` | `wrapper.rs:5035`, `:5061` |
| `rg "bridget_task_cancel"` | `crates/` | `delegation_mcp.rs`, `mcp.rs`, `wrapper.rs`, test Claude |
| `rg "capture_reply"` | `daemon.rs` | deux appelants : `daemon.rs:8724` (idempotent), `daemon.rs:17562` (ordinaire) |
| `rg "preserve_order"` | `Cargo.toml` | aucune occurrence directe |
| `git show origin/main:docs/orchestration-v2/thread-lineage-and-context-transfer.md` (T3) | T3 | fork, transfert portable, merge-back : hors périmètre |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Nom du champ de garde | creer `keep_alive_secs` | durée explicite, bornes visibles ; pas de booléen en plus | 2026-10-11 |
| Fin explicite | reutiliser `bridget_task_cancel` | un outil de moins ; sens voisin | 2026-10-11 |
| Brief | reutiliser le brouillon `bridget_handoff` | zéro validateur neuf | 2026-10-11 |
| Rejeu par ronde | creer `rounds[]` dans la tâche | 7 entrées au plus | 2026-10-11 |
| Réponses tardives | creer la table d'index `native_delegation_past_missions` | lecture O(log n) ; `json_each` serait O(n) par message ; patron de l'index unique `$.mission` (`delegation.rs:77`) | 2026-10-11 |
| Preuve d'accusé par ronde | etendre `delivery_acked` de 150 par `unacked_rounds` | pas d'accusé forcé avant la ronde | 2026-10-11 |
| Pose de la garde | code local dans `tick` | un seul écrivain de la remise (151 révisée) ; une fonction à un appelant serait une abstraction inutile (Art. XIX) | 2026-10-11 |
| Contrôle de droits | extraire de `spawn_task` et ajouter la politique actuelle (`parent_fact`, `child_policy`) | 2 usages, sécurité | 2026-10-11 |
| Extrait d'historique | ne pas creer | Bridget ne détient pas l'historique du fournisseur | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
