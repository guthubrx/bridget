# Guides natifs 149 - ronde Haiku r1

Date : 2026-10-10. Auteur : agent documentaire (claude-haiku-5-5).

**Statut : modifications documentaires faites. Aucun test exécuté. Aucune case cochée.
Aucun commit, aucun redémarrage, aucune installation, aucune modification de code,
de configuration, de base ou de service.**

## 1. Fichiers modifiés

Tous les chemins sont absolus et se trouvent dans le worktree 149.

| # | Fichier | Motif |
|---|---|---|
| 1 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/docs/delegation-native.md` | Guide principal : héritage, posture, mappage, grant, identité, suivi humain. |
| 2 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/skills/bridget/SKILL.md` | Parcours de délégation : posture facultative, droits, `t3_session_unavailable`, activation. |
| 3 | `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/skills/bridget/references/commandes.md` | Modifié seulement parce que le contrat de `bridget_delegate` et du catalogue a changé (4 lignes). |

Aucun autre fichier n'a été modifié par cette ronde. Le worktree contient d'autres
modifications, qui ne viennent pas de cette ronde. Aucun lien `~/.codex`, `~/.agents`
ou `~/.claude` n'a été touché.

## 2. Sources lues (lecture seule)

- Les trois guides, en entier. Pour `commandes.md`, les lignes 1 à 728 ont été lues,
  puis la fin (729 à 857) a été vérifiée par recherche de termes.
- `crates/bridget-daemon/src/daemon/native_delegation.rs` : lignes 1 à 66 (garde de
  révocation, `parent_fact`), 88 à 134 (`root_permission`, `same_project`), 136 à 190
  (reprise par le parent), 255 à 505 (catalogue et admission `Delegate`).
- `crates/bridget-daemon/src/delegation_mcp.rs` : lignes 1 à 80 (schémas MCP et
  arguments acceptés).
- `crates/bridget-daemon/src/native_permissions.rs` : lignes 206 à 330 (table de mappage
  `child_policy`, arguments enfant, snapshot).
- `crates/bridget-daemon/src/mcp_identity.rs` : lignes 320 à 340 et 878 à 890
  (`t3_session_unavailable`).
- `crates/bridget-daemon/src/cli_lineage.rs` : grammaire des options (recherche ciblée).
- `crates/bridget-daemon/src/registry.rs` : lignes 205 à 225 (message de posture).
- `specs/149-sous-agents-lineage/contracts/permissions.md` : G-P-07, G-P-08, tableau de
  mappage et inventaire des refus.
- `specs/149-sous-agents-lineage/contracts/lineage.md` : statuts natifs et CLI.
- `specs/149-sous-agents-lineage/quickstart.md` : lu comme guide courant, puis recoupé
  avec le code.
- `specs/149-sous-agents-lineage/validation/ui149.md` : portée de la preuve UI.

## 3. Ce qui change, point par point

### delegation-native.md

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md#extrait-1).

### SKILL.md

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md#extrait-2).

### commandes.md

- `bridget_capabilities` : mention de `inherit`, `development` et de leurs motifs.
- `bridget_delegate` : `request_id`, `agent_type`, `model`, `task`, `cwd` obligatoires ;
  `posture` et `effort` facultatifs.
- Limites de posture : l'héritage 149 ne demande aucun grant.

## 4. Contrôles de cohérence (lecture seule, aucun test)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md#extrait-3).

## 5. Écarts et points ouverts (à trancher par le principal)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md#extrait-4).

## 6. Limites de cette ronde

- Aucun test n'a été lancé, conformément au brief. Les codes et statuts ont été vérifiés
  par lecture et recherche, pas par exécution.
- Aucune relance, aucun binaire, aucune installation : l'activation des comportements
  149 reste non prouvée. Les guides le disent.
- Les preuves réelles r5 et r2 mentionnées dans `quickstart.md` n'ont pas été
  revérifiées. Elles ne sont pas citées dans les trois guides modifiés.
