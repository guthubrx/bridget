# Audit de reutilisation de l'existant - 158-reveils-controle

## Decision

Statut: PASS historique (2026-10-11) ; revalidation finale sur livraison157 requise avant GO158A
Date: 2026-10-11 (révisé après contre-revue)
Feature dir historique: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/158-reveils-controle
Feature dir158A: à renseigner par ROOT dans son worktree isolé après livraison157 vérifiée. Reçu canonique157 et empreintes également à renseigner ; aucun SHA livré déduit de la préparation.

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
| Instruction utilisateur courante152–160 | livraison Git/disque et nettoyage par ROOT après gates | activation Bridget sous condition autorisée zéro mission non terminale ; aucune relanceT3 ; anciens checkpoints149 historiques |

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/158-reveils-controle/reuse-audit.md#extrait-1).

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

## Actualisation du 12 octobre 2026

PASS : 14/14 points de réemploi, aucun doublon restant. Preuve fraîche : validation/reuse-audit-fresh.md. Les notes précédentes restent historiques.

## Extension158B — gate actuel

Réemploi158B PASS,40/40 items. Rapport complet : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/reuse-audit-158B.md. Le rapport158A ci-dessus reste historique.


Clôture ROOT après revueplanR1 :15hypothèses tranchées, contrats de cycle de vie/crash/worker/cutoff précisés. Les40items audités restent inchangés ; aucun service/table/module supplémentaire. GatePASS5/5 maintenu. Empreintes actualisées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/root-plan-tasks158B.json. Pasde validationB/test/Analyze déduite.
