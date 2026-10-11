# Audit de réutilisation de l'existant - 157-catalogue-quotas

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/157-catalogue-quotas

Conclusion courte: Le plan réutilise le préflight de `submit_spawn`, les codes de
`refusal_record`, `child_policy`, les sources de `rate_limits`, l'enveloppe
`canonical`. La reprise sur quota est sortie (160). Un seul module nouveau (`provider_limits.rs`),
justifié par 4 usages. Aucune duplication évidente.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 11 |
| Items audites | 11 |
| Reutilisations deja prevues | 7 |
| Existants potentiellement pertinents | 3 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 5 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Préflight de disponibilité | contrôles de `submit_spawn` (capacités, binaire) ET de `prepare_spawn_parts` (facturation, protocole, environnement) | `crates/bridget-daemon/src/lifecycle.rs:209-226`, `:342-380`, `command_exists` `:542` | extraction des deux, pas de nouvelle vérification |
| Codes de refus de disponibilité | `refusal_record` | `lifecycle.rs:561` | mêmes noms (`command_missing`, `billing_guard`, `env_unfit`, `unsupported_capability`) |
| Politique de l'enfant | `child_policy` | `crates/bridget-daemon/src/native_permissions.rs:206` | appelée par le verdict, aucune règle de droits neuve |
| Sources de quota | `handle_rate_limit` et ses 3 sources | `daemon.rs:6600-6660`, `claude_stream_json.rs:1355`, `codex_app_server.rs:2643`, `cli.rs:4999` | une écriture ajoutée au point unique |
| Faits de limite | `RateLimitFact` | `bridget-transport/src/protocol.rs:4843` | structure réutilisée telle quelle |
| Enveloppe de rejeu | `canonical` stocké à part de `request` | `crates/bridget-daemon/src/delegation.rs:114-130` | permet de réécrire la cible effective |
| Ordre des refus | contrat 150 | `specs/150-socle-delegation/contracts/delegation150.md` | insertion de la sélection de cible |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Mémoire de limites | `Presence.rate_limits` | `daemon.rs:566` | Non réutilisable seul : disparaît avec la présence, sans instant d'observation. Module neuf justifié (research R4) |
| Détail dans un refus | `failure(code)` | `native_delegation.rs:34` | Étendu par `failure_with_detail`, pas remplacé |
| Sonde d'authentification | aucune | `rg` credential, login, unauthenticated : rien de pertinent | Non créée (question 3) |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | O(1) candidats, O(log n) mémoire, mesure du catalogue |
| `~/.speckit/constitution.md` XIX | minimalisme | un module, pas de table, pas de CLI |
| `~/.speckit/constitution.md` XXII | mesurer avant d'optimiser | T001 et T012 |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` (checkpoints 149) | livraison sans relance | aucune relance de service |
| contrat 148 | `model_unavailable` sans substitution | `model_unavailable` jamais remplacé |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id` | enveloppe étendue sans changer les octets sans nouveau champ |
| 149-sous-agents-lineage | droits hérités, Lineage à états fermés | aucun nouvel état Lineage |
| 150-socle-delegation | refus nommés, ordre des refus, `reply_timeout` | ordre étendu |
| 116 (entretien) | faits republiés par les sources | justifie une mémoire volatile |
| 089 (tests) | recette `umask 077` | gates de test |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg rate_limits` | `crates/` | `daemon.rs:566`, `:6641`, `protocol.rs:4686`, `codex_app_server.rs:2643` |
| `rg claude-statusline` | `crates/` | `cli.rs:4859-5126`, statut fixe `unknown` |
| `rg permission_source_unavailable` | `crates/` | `native_permissions.rs:228`, `:254` : cause du constat |
| `rg launch_context` | `crates/` | optionnel côté Claude (`native_permissions.rs:75` transport) |
| `rg command_exists` | `lifecycle.rs` | `:218-226`, `:400`, `:542` |
| `rg credential\|unauthenticated` | `crates/bridget-daemon/src` | aucune sonde d'authentification fournisseur |
| `rg "fn which"` | `cli.rs` | `cli.rs:642`, usage hors délégation, non réutilisé |
| `git show origin/main:.../OrchestratorMcpService.ts` | T3 | `providerConstraints` partagé (`:264-280`, `:1146`) |
| `git show origin/main:.../UsageLimitRecoveryWorker.ts` | T3 | gardé pour la 160 (identité `thread:run:reset`) |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Authentification | non vérifiée, dite dans `checks` | aucune sonde sans appel fournisseur | 2026-10-11 |
| Où mémoriser les limites | module `provider_limits.rs` | 4 usages, `daemon.rs` trop gros | 2026-10-11 |
| Persistance des limites | volatile | les sources republient | 2026-10-11 |
| Cible effective | écrite dans `record.request` | `canonical` garde l'enveloppe | 2026-10-11 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Reporté en 160

Le réemploi des primitives de reprise de 152 (`latest_in_chain`, `may_resume`,
`schedule_execution_recovery`, `native_child_env`) est audité en 160, quand 152
est livrée et ses primitives prouvées (`spec.md`, section « Session future 160 »).
