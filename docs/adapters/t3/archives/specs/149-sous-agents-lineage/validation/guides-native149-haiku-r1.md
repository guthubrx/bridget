# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/guides-native149-haiku-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

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

<a id="extrait-2"></a>

## Extrait 2

- Titre : « contrat 148 » devient « contrat 149 ».
- Paramètres : `posture` n'est plus listée comme obligatoire.
- Droits : héritage sans grant, `development` soumis au mappage, refus `workspaceWrite`
  vers Claude/GLM, reprise par une nouvelle instance encadrée.
- T3 : fermeture de la session par `t3_session_unavailable` après retrait du credential.
- Activation 149 et renvoi vers `bridget lineage` (terminal, pas MCP).

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

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
