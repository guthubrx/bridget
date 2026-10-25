# Preuves natives r2 - T036 et T039 (récapitulatif)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde r2 après le correctif F1 (reçu natif r5). Aucun commit, aucune case cochée, aucune écriture de production, aucun redémarrage, aucun modèle.
Archives r1 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149-r1.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r1/`.

## Verdict global : PARTIAL

| Tâche | Contrôles | Verdict | Rapport |
|---|---|---|---|
| T036 interop réseau | 43 + 15 + 7 = 65/65 PASS | **APPROVE** sur le périmètre exécuté | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md` |
| T039 scénarios dégradés | 35/37 PASS, 2 FAIL (diagnostic de F2) | **PARTIAL** : F1 corrigé, nouvel écart F2 à trancher | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md` |

## Binaire testé et sources

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md#extrait-1).

## Ce qui a changé depuis r1

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md#extrait-2).

## Ce qui est réellement exécuté

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md#extrait-3).

## Simulé, nommé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md#extrait-4).

## Lacunes concrètes avant de cocher

1. Décision du principal sur F2 (option A recommandée), puis rejeu de `recovery149.ts` sur le binaire qui la contient. Aucun code de production n'a été touché par moi.
2. Revue finale du texte G-P-07(b) (clarification proposée, O1).
3. `queued` non engagé après SIGTERM : seulement couvert par le test unitaire (O6).
4. Orphelin après arrêt du daemon (O4) : à évaluer sous launchd (non vérifié ici).
5. Rejeu éventuel sur le binaire r6 (non fait : consigne r5 immuable).
6. Hors périmètre ici : modèle réel (T037), voie standalone Claude/GLM et observer PTY (T038), `can_use_tool` (G-P-01), rendu navigateur et coque desktop.

## Fichiers

- Recettes et résultats : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (voir `README.md`) ; résultats bruts `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/*.json`.
- Rapports : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-proofs-r2.md`.
