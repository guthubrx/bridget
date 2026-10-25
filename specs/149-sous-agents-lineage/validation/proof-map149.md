# Matrice de preuves 149 (version relue par la revue T041)

## Synchronisation finale du 2026-10-10 (prévaut sur les sections ci-dessous)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-1).


## Actualisation du principal — après r9 et réseau r4

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-2).

`native-network-proofs-r4.md` ferme O4.6, E2 et F3 sur le candidat release `abc850858975` : 16 contrôles réussis, dont l'état initial E2 simulé et une observation O4. Aucun crash naturel de cette fenêtre E2 n'est prétendu. Le test de performance r9 reste non conclu. La dernière recette avec parent Codex vivant r7 et les tâches de livraison T043–T045 restent en attente. Les 45 cases relèvent encore du principal.

Date : 2026-10-10. Cette version remplace la matrice intermédiaire du rédacteur documentaire. Elle a été corrigée après lecture des tests exécutés, de leurs assertions et des logs. Elle n'exécute rien. Aucune case de `tasks.md` n'est cochée. Le cochage revient au principal.

Rapport de revue lié : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` (verdict SOURCES : SOURCE_ONLY_APPROVE ; session : PARTIEL).

## 1. Règles de classement

Un état se lit avec un **niveau cible**. Une tâche est « Prouvée » quand son propre critère dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md` est rempli à son niveau. Elle n'a pas besoin d'une recette réelle quand son critère ne la demande pas.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-3).

États : **Prouvée** (niveau cible atteint) | **Partielle** (une part du critère manque ; le manque est nommé) | **Non prouvée** (aucune exécution) | **Pending** (dépend d'un autre propriétaire).

Un plan, une revue ou un diff ne prouvent pas une implémentation. Un test nommé et exécuté la prouve à son niveau. Les copies de `/bin/echo` ne prouvent aucun comportement de modèle.

## 2. Contradictions entre rapports (le plus récent précis prévaut)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-5).

## 3. Compteurs retenus (dernière ronde par suite)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-6).

Les rondes ne s'additionnent pas.

## 4. Matrice par tâche (T001 à T045)

Les chemins de tests sont absolus. Les noms de tests sont ceux du code.

### Phase 0 : contrats et gate documentaire

| Tâche | État | Niveau | Preuve | Manque |
|---|---|---|---|---|
| T001 | Prouvée | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/lineage.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/plan-lineage-r3.md` (G-L APPROVE), `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md` (G-P APPROVE) | Rien. |
| T002 | Prouvée | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/test-strategy.md` (33 scénarios), deux décisions distinctes G-L et G-P toutes deux APPROVE | Le critère est la stratégie et les deux gates. L'exécution des 33 scénarios n'en fait pas partie. |
| T003 | Pending | DOC | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-real-recipes-prepare-r1.md` : empreintes de préparation du lanceur et du CLI | Registre de recette réel et provenance standalone : attendent T038 (autre propriétaire). |

### Phase 1 : attestation effective et héritage natif

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-7).

### Phase 2 : état natif, journal et suivi

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-8).

### Phase 3 : projection et surfaces adaptateur de bureau

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-9).

### Phase 4 : recettes complètes et revue finale

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-10).

## 5. Scénarios S149-01 à S149-33

Un scénario réel (M) reste pending tant qu'aucun reçu n'existe. Les cases « U » sont des preuves de niveau test, pas de comportement de modèle.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-11).

## 6. Gaps de preuve réels et minimaux

Détail dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/review-final149.md` section 5. Aucun ne bloque SOURCE_ONLY_APPROVE.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/proof-map149.md#extrait-12).

## 7. Ce qui reste avant le cochage final

1. T036 à T039 : reçus réels des autres propriétaires. Sans eux, SC001, SC003 et SC004 restent non prouvés.
2. S149-21 : nommer le test d'opt-out.
3. Décider du sort de G1 à G9.
4. T042 : `quickstart.md` et `implementation.md`.
5. T043 à T045 : Git, paquets, reçu final.

## 8. Coquilles relevées, non corrigées

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md`, en-tête : « G-P REQUEST_CHANGES ciblé r3 » est périmé.
- Même fichier : « àS149-33 », « Phase0 » et suivants, « profondeur8 », « liste100/128KiB », « pages16KiB/100 ».
