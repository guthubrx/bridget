# Clarification documentaire G-P-07(b) - Haiku r5

Date : 2026-10-10. Auteur : sous-agent documentaire (Haiku 5.5).
Périmètre : documentation seule. Aucun code, test, build, Cargo, Git, configuration, base ou service n'a été touché.

## 1. Résumé

Le passage « lecture et rejeu inchangés » était trop large. Il laissait croire qu'un credential retiré ou tourné garde ses lectures et son rejeu. Le comportement réel est différent : le montage MCP privé de ce credential est fermé en bloc (preuve R8.4).

J'ai aligné les deux documents sur le wording exact de la section 5 de la revue source `validation/native-f2-o4-source-review-sonnet-r1.md`. Aucun comportement n'est ajouté. Aucun statut global n'est changé.

## 2. Contradictions corrigées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/permissions-doc-clarification-haiku-r5.md#extrait-1).

Les lignes (c) et (a) de G-P-07 restent inchangées sur le fond. (c) précise déjà qu'une admission 149 avec la seule v1 est refusée.

## 3. Ce qui n'a pas été modifié

- Le statut « proposé pour gate GLM » (`contracts/permissions.md`, ligne 3) reste tel quel. Aucun passage à « validé ».
- Le fichier `validation/permissions-contract-deltas-r4.md` n'est pas touché. L'historique r3 REQUEST_CHANGES puis r4 APPROVE est conservé.
- Les findings F2 et O4 ne sont pas clôturés.
- Aucun test n'a été ajouté. Aucun scénario n'a été marqué comme exécuté.
- Aucun nouveau grant, aucune permission promise.
- Les preuves anciennes (`recovery149.md`, `interop149.md`, `native-network-proofs-r2.md`) ne sont pas modifiées.
- `quickstart`, `implementation`, `tasks`, `plan` et les ADR ne sont pas touchés.

## 4. Passages voisins laissés en l'état

Ces passages ne sont pas dans le périmètre demandé. Le principal doit les arbitrer.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/permissions-doc-clarification-haiku-r5.md#extrait-2).

## 5. Limites

- Validation source seule. Ce document a été relu et écrit, mais rien n'a été exécuté. La revue Sonnet r1 n'exécute rien non plus.
- Les preuves R8.4 et S5.1/S5.1b viennent du binaire r5 (recette). Le delta F2/O4 n'est pas dans ce binaire, selon la section 1 de la revue. Ces preuves établissent le comportement du credential. Elles ne prouvent pas la correction F2/O4.
- `validation/native-network-proofs-r2.md` ne contient aucune ligne R8.4. Il ne sert ici que de contexte F2/O4, partiel. Il ne doit pas être cité comme preuve de ce cas.
- Une ronde r8 runtime reste obligatoire avant toute clôture de F2/O4 (section 6 de la revue).

## 6. Références

- Revue source : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-f2-o4-source-review-sonnet-r1.md`, section 5 (SOURCE_ONLY_APPROVE, 2026-10-10).
- Preuve R8.4 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149.md`, lignes 129 et section O1.
- Preuves S5.1 et S5.1b : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/interop149.md`, lignes 82-83.
- Historique G-P : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/permissions-contract-deltas-r4.md` (inchangé).
- Documents modifiés : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/test-strategy.md`.
