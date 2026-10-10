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

- Ajout : ce moteur n'est pas `delegate_task` de T3 ; l'exécution ne dépend pas de T3.
- Exemple : `posture` retirée. Une section dit qu'elle est facultative.
- Rejeu : mauvaise enveloppe produit `envelope_mismatch` sans nouvelle mission ; une
  clé neuve crée une mission ; ne pas changer de clé après une issue inconnue.
- Arrêt du daemon pendant la mission : `failed` avec `unreachable`, sans relance ni
  nouvel enfant, rejeu renvoie la même tâche.
- Périmètre du dossier : `cwd_scope` vaut `project` (preuve de droits, racine =
  `cwd_root`, refus `cwd_outside_parent_project`), `root` (grant) ou `same_project`
  (parent externe sans grant, `cwd_root` = `null`).
- Posture : absente = héritage ; `discovery` sans preuve = chemin 148 avec grant ;
  `development` sans preuve = `permission_attestation_unavailable`.
- Mappage 149 en quatre familles : Codex vers Codex ; Codex vers Claude/GLM (seulement
  `dangerFullAccess` + `never`, bypass limité à la mission, `workspaceWrite` refusé) ;
  Claude/GLM vers Claude/GLM (mêmes entrées, dossier identique, règles `allow`/`deny`
  gardées, aucun bypass universel) ; Claude/GLM vers Codex (bypass complet ou lecture
  seule). Tous les refus arrivent avant la création de la tâche.
- Grant : le MCP ne le crée jamais. Il n'est requis que pour la voie 148. La révocation
  reste prioritaire, héritage compris. Seul un nouveau grant humain la lève.
- Identité : credential retiré, révoqué ou tourné = `t3_session_unavailable` pour les
  outils Bridget de la session. Admission héritée ou `development` exige la version 2
  du fait de droits ; sinon `permission_attestation_unavailable`.
- Activation : les comportements 149 exigent le daemon et le binaire 149 chargés.
- Nouvelle section : suivi humain (`bridget lineage inspect|watch|cancel` avec
  `--t3-thread` et `--project-root`, panneau Lineage, statuts natifs, portée de preuve).
- Nouvelle section : ce qui ne change pas (`bridget spawn`, postures, protections du
  terminal, `delegate-grant` humain).
- Supprimé : l'affirmation 148 « Claude/GLM n'ont pas de protocole natif de développement
  confiné », avec `development_protocol_unavailable`. Ce code n'existe plus dans le code
  de production ; il n'apparaît que dans un commentaire de test.
- Supprimé : la phrase sur le message « posture développement réservée à Codex
  app-server ». Ce message vient du lancement ordinaire (`registry.rs:218`), pas de
  `bridget_delegate`.

### SKILL.md

- Titre : « contrat 148 » devient « contrat 149 ».
- Paramètres : `posture` n'est plus listée comme obligatoire.
- Droits : héritage sans grant, `development` soumis au mappage, refus `workspaceWrite`
  vers Claude/GLM, reprise par une nouvelle instance encadrée.
- T3 : fermeture de la session par `t3_session_unavailable` après retrait du credential.
- Activation 149 et renvoi vers `bridget lineage` (terminal, pas MCP).

### commandes.md

- `bridget_capabilities` : mention de `inherit`, `development` et de leurs motifs.
- `bridget_delegate` : `request_id`, `agent_type`, `model`, `task`, `cwd` obligatoires ;
  `posture` et `effort` facultatifs.
- Limites de posture : l'héritage 149 ne demande aucun grant.

## 4. Contrôles de cohérence (lecture seule, aucun test)

- Phrases 148 obsolètes (`development_protocol_unavailable`, « restent limités à »,
  « ne peut pas la reprendre », « contrat 148 » en titre, « posture `development` nécessite ») :
  **0 occurrence** dans les trois fichiers.
- Codes cités dans les guides : chacun a été trouvé dans le code de production hors
  tests (`envelope_mismatch`, `cwd_outside_parent_project`, `unreachable`,
  `inherit_refusal`, `development_refusal`, `cwd_root`, `same_project`,
  `permission_not_inherited`, `provider_confinement_unavailable`,
  `permission_mapping_unavailable`, `settings_revision_changed`,
  `permission_source_unavailable`, `delegation_grant_required`,
  `permission_attestation_unavailable`, `t3_session_unavailable`, `mission_reply_timeout`).
- Statuts Lineage cités (`queued` à `cancelled`) : présents dans les sources.
- Options CLI `--t3-thread` et `--project-root` : présentes dans `cli_lineage.rs`.
- Tirets cadratins : `docs/delegation-native.md` = 0. `SKILL.md` = 1, préexistant
  (ligne 88, section « Privilégier le même projet », non modifiée). `commandes.md` = 21,
  tous préexistants ; aucun n'a été ajouté. Les lignes ajoutées utilisent le tiret simple.
- Accents : les textes ajoutés sont rédigés avec les accents français.

## 5. Écarts et points ouverts (à trancher par le principal)

1. **« Validation humaine pour tâches héritées » (brief).** Le brief dit que l'utilisateur
   149 « n'exige nouvelle validation humaine que pour admitted tasks inherited ». Le
   code n'ajoute aucune validation humaine pour une tâche héritée. Seule la révocation
   explicite refuse. Aucune règle de validation humaine n'a donc été écrite dans les
   guides. À confirmer avant livraison.
2. **Valeur `cwd_scope = "project"`.** Le code l'émet quand une preuve de droits existe.
   Le contrat `permissions.md` ne liste pas les valeurs de `cwd_scope`. Le guide suit
   donc le code. Le contrat devrait nommer cette valeur.
3. **Fermeture par `t3_session_unavailable`.** G-P-07 (b) énumère quatre outils
   (`bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`, `bridget_who`) et
   parle de « tout appel de ce credential ». Les guides disent « tous les outils Bridget
   de la session ». Si la liste est exhaustive, cette phrase doit être resserrée.
4. **Preuve UI.** `ui149.md` décrit une recette web sur daemon simulé, sans numéro de
   ronde dans son titre. Le quickstart cite une ronde r5 pour T040. Les guides ne citent
   aucun numéro de ronde et ne revendiquent aucune preuve avec daemon réel.
5. **Source T3 non lue.** Le panneau Lineage est décrit d'après `quickstart.md`,
   `contracts/lineage.md` et `ui149.md`. Les sources du client T3 n'ont pas été lues.
6. **Claude vers Claude : dossier identique.** Le code exige que le dossier de l'enfant
   soit celui du parent (`permission_mapping_unavailable` sinon). Le contrat le dit aussi.
   Le guide le reprend.

## 6. Limites de cette ronde

- Aucun test n'a été lancé, conformément au brief. Les codes et statuts ont été vérifiés
  par lecture et recherche, pas par exécution.
- Aucune relance, aucun binaire, aucune installation : l'activation des comportements
  149 reste non prouvée. Les guides le disent.
- Les preuves réelles r5 et r2 mentionnées dans `quickstart.md` n'ont pas été
  revérifiées. Elles ne sont pas citées dans les trois guides modifiés.
