# Plan - Session158a vue globale, arrêt global, portées MCP

**Spec** : `specs/158-reveils-controle/spec.md`
**Branche** : branche/worktree158A isolés à renseigner par ROOT après vérification de la livraison157. Les anciens chemins150 sont historiques.
**Date** : 2026-10-11 (révisé après contre-revue Codex, verdict BLOCKED corrigé)
**Portée** : 158a seulement. 158b reste en spec. Budget de tours retiré.

## Résumé

Trois ajouts, en conservant le contrat d'appel de la base157 livrée, y compris les ajouts151–157 :

1. vue globale par `bridget control delegations`, réservée à l'humain attesté (R1) ;
2. arrêt global distinct de la pause, composé avec elle dans une seule décision d'admission (R2, R3) ;
3. table des portées MCP par action, test exhaustif et plafond lecture seule (R6, R7).

## Contexte technique

- Rust 2024, workspace à 3 crates. SQLite, base `bridget.db`, connexion du magasin de délégation.
- Aucune nouvelle dépendance. Un module nouveau : `mcp_scopes.rs`.
- Tests : recette `umask 077` (quickstart).

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | variantes de contrôle (`DelegationOverviewRead`, `DelegationOverview`, `DelegationHaltSet`, `DelegationHaltState`, `DelegationCancel`, `DelegationControlRejected`) |
| `crates/bridget-daemon/src/delegation.rs` | tables `native_delegation_halt` et `native_delegation_halt_commands`, `overview_page` |
| `crates/bridget-daemon/src/referent_control.rs` | `admit_native_delegation(control, halt)` : garde commune pause et arrêt |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | admission par la garde commune, clé `delegations_halted` du catalogue |
| `crates/bridget-daemon/src/daemon.rs` | traitement des requêtes de contrôle (négociation et `control_client_actor`) |
| `crates/bridget-daemon/src/cli.rs` | `control delegations [cancel]`, `halt-delegations`, `resume-delegations`, `control status` |
| `crates/bridget-daemon/src/mcp_scopes.rs` (nouveau) | table des portées, `scope_of`, tests exhaustifs |
| `crates/bridget-daemon/src/mcp.rs` | plafond avant le `match` d'exécution |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module |
| docs et skill | CLI, refus, portées, rupture voulue pour `post` |
| `docs/decisions/158-controle-global-portees.md` | ADR |

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 158 (T020), amende la lecture de l'ADR 027 |
| VIII Forensic | OK | défauts prouvés par lecture ; contre-revue vérifiée ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique en pipeline |
| XVI Worktree | À vérifier au GO | worktree158A isolé créé par ROOT après livraison157 |
| XVIII Complexité | OK | vue O(n log n), n ≤ 4096 ; reçus O(1) ; `scope_of` O(1) ; plafond : une recherche par appel MCP, à mesurer (T017) |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Minimalisme (Article XIX)

- Pas de capacité client, pas de genre d'événement de contrôle, pas de champ de tâche.
- Budget de tours retiré : une borne qui ne vaut que pour un fournisseur et se remet à zéro à chaque ronde est pire que pas de borne.
- Réutilisés : `human_cancel`, négociation `ControlStateV1` plus `control_client_actor`, garde `referent_control.rs`.
- Un module nouveau, justifié : la table de portées est un contrat testable ; `mcp.rs` fait 5415 lignes.

Potentiel minimalisme : environ 120 lignes non écrites par le retrait du budget
de tours (champ, requête récursive, dépôt boîte, schéma, tests).

### Responsabilité future (Article XX)

- Chaque décision a sa preuve dans `research.md` et `adversarial-review-codex.md`.
- Limites écrites : garantie des portées par test de CI, pas par le compilateur ; attestation humaine bornée par l'ADR 011 ; reçus de commande limités à 256.
- Non vérifié : coût de la recherche de la tâche d'un appelant (T017) ; liste exacte des outils de 151 à 157.

## Décisions d'architecture

| Décision | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Vue globale | requête de contrôle, acteur humain exigé | drapeau sur `lineage` ; `ControlStateV1` seule | autorité humaine ; la capacité s'accorde à tout client |
| Arrêt et pause | deux états, une garde commune | règle séparée dans l'admission | ADR 027 : garde unique |
| Idempotence de l'arrêt | reçus bornés portant la réponse, annulation une seule fois | `last_command_id` ; rejeu qui réannule | A, B, rejeu de A remettrait l'arrêt ou annulerait une tâche C neuve |
| Stockage de l'arrêt | ligne unique | genre dans `control_events` | reconstruction de table à `CHECK` en production |
| Portées | table par action + test | types à la T3 | ~30 déclarations à réécrire |
| Plafond | `effective_posture`, durée de vie de l'enfant | posture demandée, tâche non terminale | une posture omise ; un enfant garde la vie entre rondes |
| Fils pour `discovery` | `read` et `ack` permis, `post` refusé | tout fil interdit | lecture sans `ack` rejoue la même page |
| Budget de tours | retiré | compteur d'usage actuel | non dédupliqué, Claude seul, remis à zéro par ronde |

## Étapes d'implémentation

1. Baseline (T001). 2. Fondations en parallèle (T002 à T005). 3. Vue globale (T006 à T009). 4. Arrêt (T010 à T013). 5. Portées et plafond (T014 à T018). 6. Docs, ADR, suite complète, relecture (T019 à T022).

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | délégation admise pendant l'arrêt ; `tools()` sans table ; enfant `discovery` qui écrit |
| Unitaire | 4 combinaisons pause/arrêt ; A, B, rejeu de A (aussi après redémarrage) ; client agent négocié refusé ; page globale ; table dans les deux sens ; `readOnlyHint` ; plafond (posture omise, terminal avant nettoyage, entre rondes) ; fil lecture, `ack`, page suivante ; journal avec et sans `to` |
| Surface | aucun outil ne pose l'arrêt ; aucune portée « contrôle humain » |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087`, tests MCP |
| Suite complète | `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Plafond `discovery` bloque un usage existant (post, partage de journal, publication d'artefact) | politique explicite FR-029, ruptures documentées, tests de la réponse automatique et des artefacts (T018), question 2 |
| Rejeu d'un arrêt avec annulation | reçu portant la réponse, annulation à la première application seulement, test A/B/C/rejeu (T012) |
| Recherche de la tâche d'un appelant | réutiliser l'index UNIQUE child existant via CallerScope interne ; mesure T017 ; aucun cache/index supplémentaire par défaut |
| Outils de 151 à 157 non classés | test exhaustif échoue ; T016 |
| Annulation en cascade partielle | `human_cancel` idempotent par `request_id` ; relance possible |
| Reçus de commande purgés (au-delà de 256) | limite documentée ; identifiants neufs par geste de CLI |
| Lignes décalées après 150 à 157 | l'implémenteur relit chaque fonction ; le plan cite aussi des noms |

## Esquisse 158b (non planifiée)

Planificateur greffé sur la boucle de 1 s, table `wake_schedules`, variante
`AutonomousEffect::ScheduledWake` dans la garde commune, outil `bridget_wake` à
portée dépendante de la cible, CLI `bridget wake`, table `observation_missed`.

## Livraison

Code dans le worktree158A isolé. ROOT effectue la livraison Git/disque et le
nettoyage après les gates du pipeline utilisateur. Activation Bridget seulement
sous la condition déjà autorisée de zéro mission non terminale ; aucune relanceT3.
Pas de commit automatique par les sous-agents. Diff enregistré dans
`validation/diff-158.patch`. Les chemins et empreintes du reçu157 livré sont à
renseigner avant le GO158A, puis T001 établit la baseline fraîche.

## Base158A vérifiée par le principal

Date : 2026-10-12. Branche : session-158-reveils-controle. Racine : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158-reveils-controle. Base Git : bd37e927eee0b28f058df9d6a6992e38665a6ad0. Sources157 : 8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b. Reçu canonique157 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md. SHA256 : 6ac3d08948f39d6999b0a5c425235a70a1ad7d99d433866887abbdf52a86b705. Intégration, push, activation et nettoyage157 vérifiés. Les déclarations de préparation ci-dessus restent historiques ; cette vérification lève leur prérequis de livraison.

## Décisions ROOT de contre-revue fraîche158A

CallerScope est une requête interne dans l'enveloppe native existante, sans outil MCP : identité inscrite, lookup indexé côté daemon, résultat booléen obligatoire et panne fermée avant tout effet. Aucune DB cliente. Halt compose Delegate et Continue seulement ; demandes ouvertes/réponses et missions déjà queued/en cours gardent leur contrat de pause. Le snapshot des plus hautes racines actives et leur count est évalué une seule fois via FnOnce avant COMMIT ; verrou daemon jusqu'à human_cancel ; cancelled compte des arbres visés. Replayhalted est historique. Overview transporte les métadonnées courantes since/by/reason Option pour status. Pas d'autre variante, tâche ou test autonome. Journal to absent/null est Reads, autre valeur Sends ; annotations dérivées des bases de la table existante. CLI hors plafond MCP, borne ADR011 conservée.

## Extension158B après livraison158A

La partie B complète US5–US7. Son artefact de référence est /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/plan-158B.md. Les preuves et choix158A ci-dessus restent historiques et conservés.
