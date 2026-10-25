# Plan - Session156 guidage, interruption et lecture de file

**Spec** : `specs/156-guidage-file/spec.md`
**Branche** : `session-156-guidage-file`
**Worktree** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file` (créé par root après livraison155 ; base livrée appliquée, aucune création par cet owner)
**Date** : 2026-10-11 (révisé après contre-revue Codex : ronde 1 BLOCKED, 11 points ; ronde 2, 4 points)
**Statut de clôture** : Completed,32/32 ; CONVERGED passage2 ; livraison Git, binaire signé, activation Bridget et nettoyage prouvés dans validation/livraison156.md. Aucun GREEN global.

## Résumé

Trois apports, sans nouveau mécanisme d'injection :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/plan.md#extrait-1).

**Hors périmètre, décidé après contre-revue** : `cancel`, `reorder`, `promote`
(saga durable, R8). Effet sur la note : Guidage vise 4, pas 5.

**Cadre de menace** (ADR 011, spec « Cadre de menace ») : la garde protège contre
un agent qui utilise ses outils Bridget. Un processus hostile du même compte
Unix qui forge des trames brutes reste hors modèle ; on ne le promet pas.

## Contexte technique

- Rust 2024, workspace à 3 crates. Aucune nouvelle dépendance.
- SQLite (`bridget.db`), même base que l'idempotence.
- Tests : recette `umask 077` de la session150 (`quickstart.md`).

## Décisions architecturales

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/plan.md#extrait-2).

## Fichiers touchés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/plan.md#extrait-3).

Aucun changement du schéma de `send_deliveries` ni du `CHECK` de phase. Claude conserve son comportement. Codex reçoit la seule correction ciblée de provenance du repli sur échéance F1.

### Admission et réutilisation du montage T013

Pour ControlExecution, reserve_control_command conserve canon et rejeu AVANT la garde. La garde d’une commande neuve précède mark_control_dispatched, le comptage et l’émission ; le rejeu conserve sa réponse existante.

La remise immédiate utilise la connexion source réelle. À la récupération, from ne permet jamais d’emprunter une route UI ou humaine ; sans preuve UI persistée, la remise attend normalement. Pour les agents UUID, recalculer les liens depuis les faits actuels.

SendIdempotent inscrit l’admission guidance_events dans la même transaction begin_send_delivery_with_guidance que remise/suivi/lien/journal. Sur les chemins historique/control/cancel, l’audit accepté est durable avant émission physique. Un INSERT échoué interdit le contrôle. Un crash après admission mais avant émission peut consommer le rythme sans effet physique ; ce choix est conservateur.

T013 reprend une copie légère de Fixture et compile_provider152 de
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/crates/bridget-daemon/tests/native_delegation_e2e.rs
(ancres125 et462). Les trois usages prévus sont Codex actif, Claude actif et
Codex inactif ; leurs extensions de délais/FIFO restent dans les mêmes scénarios.
Le montage appartient au seul guidance156_e2e.rs prévu. Aucun framework,
nouvelle API ou scénario de confort n’est ajouté. Ces usages ne sont pas encore
une preuve d’exécution. Attach conserve son rôle et peut envoyer ClientHello uniquement avec la liste exacte [QueueListV1], sans identité agent/conn_names ni autre capacité. L’UI Client attestée garde son rôle et peut lire toute la projection.

## Étapes d'implémentation

1. Référence de non-régression (T001) et six vérifications (T002 a à f).
2. Fondations en parallèle : protocole (T003), pause (T004), garde pure (T005), stockage (T006).
3. US1 : intention (T007 à T013).
4. US2 : garde sur tous les chemins atteignables par les outils (envoi, contrôle, guidage implicite, annulation), autorité humaine, origine, anti-boucle, pause, mission, tests combinés (T014 à T023).
5. US3 : lecture de file (T024 à T026).
6. Documentation, non-régression, suite complète, relecture (T027 à T031).

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 156 prévue (T028) |
| VIII Forensic | OK | failles prouvées par lecture ; tests rouges d'abord |
| XV Test-before-next | Adapté | pas de commit automatique ; tâche cochée après test vert |
| XVI Worktree | Prévu | worktree156 isolé après livraison155 |
| XVIII Complexité | OK | R12 |
| XIX Minimalisme | OK | périmètre réduit après contre-revue |
| XX Vertus LLM | OK | chaque décision a sa preuve `fichier:ligne` ; contre-revue archivée |

### Minimalisme (Article XIX)

- Les intentions, transports et le pont existent. On expose et on protège.
- Un outil en lecture seule. Ni `cancel`, ni `reorder`, ni `promote`, ni `auto`.
- Un module pur, une table d'audit, une capacité, une paire de trames client.
- Aucun nouveau mécanisme d'injection ni schéma de send_deliveries. Raccord wrapper des champs prévus et correction ciblée du repli Codex F1.

Potentiel minimalisme : non mesuré avant code ; T031 compte les lignes réellement suppressibles. L’index est conservé pour la lecture et n’est pas une ligne suppressible.

### Responsabilité future (Article XX)

- Non vérifiable sans exécution : le comportement réel de l'émetteur quand un transport rejette un `steer` (T002(a)).
- Prérequis de la session d'édition de file notés en R8 et dans l'ADR.

## Stratégie de test

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/plan.md#extrait-4).

## Risques

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/156-guidage-file/plan.md#extrait-5).

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de service.
Diff enregistré dans `validation/diff-156.patch`.

## Revalidation de la capacité de guidage adaptateur de bureau

La lecture fraîche du bundle installé149 remplace l’hypothèse Claude/Cursor :
Claude seul a activeSteering natif. Cursor se replie sur interrupt_restart,
donc steer_unavailable et attente implicite en busy même avec mid_turn_ok=true.
T012, T021 et T029 étendent leurs oracles existants, sans nouveau nom ou tâche.
Cette règle ne change pas le transport Codex natif. Le filtrage de provenance
et la liste de capacités sont deux gardes distinctes, toutes deux requises.

## Limite de compatibilité REST/V1 du pont adaptateur de bureau

La voie REST/V1 Client.dispatch ignore l’ancien booléen et ne sait pas exprimer
deliveryIntent. Elle refuse donc localement toute intention explicite avant
émission : steer_unavailable pour steer, motif de capacité explicite pour
interrupt, avec les erreurs/report_discard existants. L’envoi ordinaire None
reste inchangé. Le paquet installé149 est V2 : le trajet de production visé
n’est pas changé par cette fermeture V1. Aucun protocole, API, type ou nom
d’oracle supplémentaire ; T012 étend seulement les cas déjà prévus.

## Coût de la lecture QueueList revalidé

Le parcours indexé coûte O(log n + s), s candidats dispatching parcourus, expirés
inclus ; les jointures indexées s’ajoutent. LIMIT200 borne le résultat, pas le
travail total. Aucun EXPLAIN, benchmark ou nouvel index d’optimisation n’est
introduit. T025 doit annoter la requête réalisée, sans prétendre s≤200.
