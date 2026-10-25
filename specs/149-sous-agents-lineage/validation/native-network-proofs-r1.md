# Preuves natives r1 - T036 et T039 (récapitulatif)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production.

## Verdict global : PARTIAL

| Tâche | Contrôles | Verdict | Détail |
|---|---|---|---|
| T036 interop réseau | 43/43 + 15/15 PASS | APPROVE sur le périmètre exécuté | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| T039 scénarios dégradés | 23/24 PASS, 1 FAIL | CHANGES_REQUIRED (R9.2) | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` |

Pourquoi PARTIAL : un écart runtime réel (F1 : mission en vol + redémarrage du daemon), et les sources de production ont bougé après le binaire testé. Aucune des deux tâches ne doit être cochée avant la décision sur F1 et un rejeu sur le binaire suivant.

## Ce qui est réellement exécuté (et non déduit d'un prompt)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md#extrait-1).

## Simulé, nommé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md#extrait-2).

## Écarts et observations

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md#extrait-3).

## Lacunes concrètes avant de cocher

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md#extrait-4).

## Fichiers

- Outils et résultats : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (voir `README.md`).
- Rapports : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md`.
- Empreintes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/fingerprints.json`.
