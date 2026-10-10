# Session149 - tests natifs r4 (Sonnet 5.5 high, claudeAgent)

Date : 2026-10-10. Aucun modèle réel, aucun service, aucune DB de production, aucun Git, aucun cochage, aucune release, aucune installation.
Aucune ligne de production modifiée. Le rapport r3 reste intact : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-tests-sonnet-r3.md`.

## Résultat

| Mesure | Valeur |
|---|---|
| bridget-daemon (`cargo test -p bridget-daemon --offline --no-fail-fast`, 78 exécutables) | **1471 PASS / 0 FAIL / 61 ignored** |
| bridget-transport (10 exécutables) | **325 PASS / 0 FAIL / 2 ignored** |
| Total | **1796 PASS / 0 FAIL / 63 ignored** |
| Observer 149 en série (`--test-threads=1`) | 10 PASS / 0 FAIL |
| Les 6 FAIL de r3 | 6/6 résolus (3 prod corrigés par Sol, 3 fixtures obsolètes) |

Compilation n'est pas PASS : ces chiffres sont des tests exécutés.
Correction du rapport r3 : « transport 326 » était une erreur de somme. Les logs r3 donnent 325/0/2, identiques par exécutable à r4.

Environnement : `umask 077`, `TMPDIR=/tmp/b149t-r4` (APFS, 0700, hors `/Users/moi`), `CARGO_TARGET_DIR` externe, jobs 2, incrémental 0, offline.
Piège observé : sans `umask 077`, `config-149` est créé en 0755 et refusé (`EnvUnfit ... attendu 0700`). Trois tests 149 échouent alors (`catalogue_*`, `spawn_refuse_les_entrees_gelees`). C'est un défaut d'environnement, pas de production. Prouvé par sonde temporaire, retirée.

## 1. Correction Sol (native_delegation.rs lignes 40-42)

Lecture ciblée : `revoked_agent(owner) || revoked(instance)` rend `delegation_grant_required` (ligne 40). Un descendant dont la racine (`root_owner_agent_id`) est révoquée rend `permission_not_inherited` (ligne 42), avant attestation et snapshot.

Contrat : `contracts/permissions.md` lignes 32-33 et 408-409 : les refus de révocation 148 gardent la priorité.
Sûreté vérifiée :
- Rejeu : `by_agent_request` (ligne ~415) précède `parent_fact` (ligne ~420) : inchangé.
- Catalogue : `parent_fact` propage le refus par `?` : un propriétaire révoqué n'obtient pas de catalogue.
- Après admission : `spawn_task` (ligne ~535) refuse toujours `permission_not_inherited` si propriétaire, racine ou instance est révoqué.
- Aucun droit élargi : les deux codes sont des refus.

Tests : 59 PASS / 0 FAIL (`native_delegation::`, `native149`, `native_permission`). Les 3 tests 148 (`native148_agent_revocation_survives_a_new_instance_until_explicit_grant`, `native148_attested_git_discovery_accepts_repo_and_external_worktree_only`, `native148_replay_precedes_grant_revocation_and_definition_change`) et `native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant` passent. Journaux : `native-r4-step1-native-delegation.log`, `native-r4-step1-rerun.log`.

## 2. Audit des 5 oracles 148 modifiés en r3

Méthode : diff `git diff HEAD` du bloc `#[cfg(test)]` de `native_delegation.rs` + contrat 149. Verdict : **les 5 sont acceptés, aucune régression masquée**.

| # | Oracle | Verdict | Justification (contrat) |
|---|---|---|---|
| 1 | Builder `request()` : `Some(Discovery)` au lieu de `Development` | Accepté | `permissions.md` l.14-17 : « Development exige les droits d'écriture prouvés [...] Une absence de preuve rend permission_attestation_unavailable ». Sans fait, le code garde le chemin legacy 148 uniquement pour `Some(Discovery)` explicite (grant humain + `authorize_cwd`). `None` = inherit, refusé sans fait. La couverture positive de Development avec fait est dans `native149_catalogue_avec_fait_ouvre_le_developpement_sans_grant_humain` et `native149_posture_omise_resout_le_developpement_effectif_du_fait` (PASS). |
| 2 | `native148_cwd_and_development_grants_are_closed` : Development explicite sans fait rend `permission_attestation_unavailable` | Accepté | Même passage l.14-17 : « il n'ajoute aucun grant humain Bridget ». Le refus `development_grant_required` n'existe plus pour Development. Le test garde l'autre moitié (cwd hors grant : `cwd_outside_parent_grant`) intacte. |
| 3 | Catalogue GLM : `development_refusal` = `permission_attestation_unavailable` | Accepté | l.377-378 : le catalogue présente les mappages supportés et leurs refus. Le code calcule `development` par `child_policy` sur le fait ; sans fait, l'erreur est `permission_attestation_unavailable`. `discovery` reste `true` (assertion conservée). |
| 4 | `root_permission(...).maximum()` = `Discovery` | Accepté (conséquence directe de #1) | La mission du test est maintenant admise en Discovery ; le maximum de la permission racine de l'enfant suit la posture admise. La suite du test (grant Development révoqué puis `None`) est inchangée. |
| 5 | Descendant : `parent_task_id = Some(root.task_id)` | Accepté | `lineage.md` l.190 : le parent d'une tâche imbriquée est `parent_task_id`. `descendants()` (`delegation_lineage.rs:138`) parcourt `$.parent_task_id`. Une ligne clonée sans parent est une racine. L'assertion « annulation des descendants avant le parent » est conservée. |

Aucun oracle abaissé : `delegation_grant_required` reste exigé sur les 3 tests de révocation (étape 1). Nouvelle posture racine Discovery et `parent_task_id` sont réellement contractuels, pas une adaptation de confort.

## 3. Trois anciens échecs (hors périmètre 149)

Preuve **statique** (HEAD `6807c22b` comparé à l'arbre de travail). Je n'ai pas reconstruit HEAD : les fichiers de test et les sources concernées sont identiques entre HEAD et l'arbre, donc le comportement l'est aussi. Marquage honnête : `FBASELINE prouvé statique`, non exécuté sur HEAD.

| Test | Fait de HEAD | Cause | Correction (tests seulement) |
|---|---|---|---|
| `spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises` (`crates/bridget-daemon/tests/claude_native_permissions_test.rs`) | `BRIDGET_SAFE_MCP_TOOLS: [&str; 20]` dans `wrapper.rs` de HEAD contient déjà `bridget_capabilities`, `bridget_delegate`, `bridget_task_status`, `bridget_task_cancel` (ajoutés par 347d7885, session 148). Le diff 149 de `wrapper.rs` ne touche pas ce tableau. Le test n'a pas bougé depuis le commit 1f6acc42 (109). | Oracle périmé : 16 outils attendus. | 4 noms ajoutés à l'ensemble attendu. Égalité d'ensembles exacte conservée (`assert_eq!` BTreeSet), `mcp__bridget__*` et `guichet_delegate` toujours interdits. |
| `spec102_v33_parite_cli_mcp` (`crates/bridget-daemon/tests/spec102_threads_test.rs`) | `mcp.rs` de HEAD : `"action":{"enum":["create","add_members","list","show","post","read","ack","history","close"]}` (9). 82f4abeb (session 147) a ajouté `add_members`. `mcp.rs` non modifié par 149. | Test attendait `len() == 8`. | Égalité d'ensemble exacte sur les 9 actions (plus de `len`, pas de joker). |
| `spec102_v34_migration_base_pre102_et_contraintes_effectives` (même fichier) | `THREAD_SCHEMA_VERSION: i64 = 3` dans `store/threads.rs` de HEAD (82f4abeb : 2 vers 3). `store/` non modifié par 149. | Test attendait `MAX(version) == 2`. | `== 3`, avec message citant la migration session147. La migration réelle est exécutée (base pré-102 migrée), assertion sur la valeur réellement écrite. |

Résultat : les 3 passent (`native-r4-step3-old-tests.log`) puis dans la batterie complète.
Journaux : `native-r4-step3-old-tests.log`, `native-r4-final-daemon.log`.

## 4. Limites et risques

- `native_permission_observer149_tests` écrit le PATH du processus de test une seule fois (`Once`, `set_var`). Aucune production modifiée. Mitigation vérifiée : le module passe 10/10 en série (`--test-threads=1`) en plus de la batterie parallèle. Un risque de concurrence théorique demeure pour d'autres tests qui lisent le PATH pendant l'écriture. Aucun flocon sur 2 batteries r3 et 1 batterie r4.
- Les fixtures `claude` sont des copies de `/bin/echo` : jamais une preuve de comportement du modèle. T037/T038 restent à faire.
- `wrapper::reconnect_tests::le_domaine_derive_nomme_le_depot_courant` exige `TMPDIR` hors de `/Users/moi` (ce dossier contient un `.git`).
- Non vérifié : HEAD exécuté (preuve statique pour les 3 anciens échecs), recette T3 réelle, hooks Claude réels.
- Lignes que j'aurais pu supprimer et que je garde : aucune.

## 5. Binaire debug (aucune installation)

| Champ | Valeur |
|---|---|
| Copie stable (nouvelle, r3 conservée) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-823e8a5fab8a` |
| SHA-256 | `823e8a5fab8abe38fcb439d8fda98d929622dae3751ff8f629d306744379328b` |
| Chemin cible (sera écrasé au prochain build) | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget` |
| Version | `bridget 0.1.3`, Mach-O arm64, 53 993 512 octets |
| Construit | 2026-10-10 14:08:24 +0200, `cargo build -p bridget-daemon --bins --offline` |
| Empreinte sources (tests inclus, 196 fichiers) | `e6f45086b23974604667c93d3b34755d7d73dc5dadbe4dde94c1e09f34285408` |
| Empreinte sources production (111 fichiers) | `e0d86f016f1fbb06d485c13ff863a639a2ed3daef29b8be4247797e1d7a3f215` |
| Fraîcheur | aucun fichier de production plus récent que le binaire ; le correctif Sol (`native_delegation.rs:40-42`) est dans les sources compilées |
| Wrapper | fait initial publié l.4148-4167 avant `NativeTui::start` (l.4184) |
| État Git | arbre non committé : aucune prétention d'état propre. `headCommit` = `6807c22b` plus modifications locales |

L'ancien binaire `bridget-ed5e28bc8ddc` est conservé mais périmé : il précède le correctif Sol. Les recettes doivent utiliser `bridget-823e8a5fab8a`.
Reçu machine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json`. Ancien reçu, copie identique : `.../native149-debug-receipt-r3.json`.

## Fichiers modifiés par cette ronde (tests seulement)

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/claude_native_permissions_test.rs` (+5 lignes)
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/tests/spec102_threads_test.rs` (+33/-7 lignes)

## Journaux

Même répertoire : `native-r4-step1-native-delegation.log`, `native-r4-step1-native149.log` (échec dû à l'umask, conservé comme preuve), `native-r4-step1-rerun.log`, `native-r4-step3-old-tests.log`, `native-r4-final-daemon.log`, `native-r4-final-transport.log`, `native-r4-observer-serial.log`, `native-r4-build-debug.log`.

## Espace disque

Cible externe : 3,6 Go (inchangé). Nouvelle copie du binaire : 54 Mo. `/tmp/b149t-r4` (306 Mo) supprimé en fin de ronde.

## Pour le parent / Sol

Aucun finding actif pour Sol. Aucun bug de production nouveau.
