# Session149 - tests natifs r3 (Sonnet 5.5 high, claudeAgent)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 high. Remplace le testeur GLM arrêté.
Aucun modèle réel, aucun service, aucune DB de production, aucun Git, aucun cochage, aucune release.
Reçu machine : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json`

## Résultat

| Mesure | Valeur |
|---|---|
| Commande finale | `cargo test -p bridget-transport -p bridget-daemon --offline --no-fail-fast` |
| Total | **1791 PASS / 6 FAIL / 63 ignored** |
| bridget-transport | 326 PASS / 0 FAIL / 2 ignored |
| bridget-daemon | 1465 PASS / 6 FAIL / 61 ignored |
| Compilation tous tests | OK (83 exécutables, 0 erreur) |
| Binaire debug | **disponible** (voir plus bas) |

Compilation n'est pas PASS : les chiffres ci-dessus sont des tests exécutés.

### Tests 149 (propriétaire : ce testeur)

| Groupe | Résultat |
|---|---|
| Transport P `native_permissions149_test` | 8/8 |
| Transport P `native_permissions149_e2e` | 5/5 |
| Transport L `lineage_contract_test` | 6/6 |
| Daemon P `native_permissions149_tests` | 13/13 |
| Daemon P `native_permission_observer149_tests` | 10/10 |
| Daemon P `native_delegation_permissions149_tests` | 8/8 |
| Daemon L `delegation_lineage_tests` | 10/10 |
| Daemon L `native_lineage_tests` | 9/9 |
| Daemon L CLI `lineage_cli_test` | 3/3 (1 ignored : `support::performance_daemon_worker`, sous-processus des bancs) |
| Anciens `native_delegation::tests::native148_*` | 16 PASS / **3 FAIL** (voir F-PROD) |

Le test CLI `lineage_cli_149_*` a été rejoué 3 fois de suite après correction : 3/3 à chaque passe.

## Les 6 échecs restants

### F-PROD : 3 tests 148, candidat bug de production (pour Sol native149)

- `daemon::native_delegation::tests::native148_agent_revocation_survives_a_new_instance_until_explicit_grant` (ligne 1579)
- `daemon::native_delegation::tests::native148_attested_git_discovery_accepts_repo_and_external_worktree_only` (ligne 1420)
- `daemon::native_delegation::tests::native148_replay_precedes_grant_revocation_and_definition_change` (ligne 1898)

Constat : après `grant_agent(OWNER, ..., revoke=true)`, une demande Discovery sans fait obtient `permission_not_inherited`. Les tests 148 attendent `delegation_grant_required`.

Cause lue dans la source : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/daemon/native_delegation.rs:39`, première ligne de `parent_fact` :
`if revoked_agent(owner)? || revoked(instance)? { return Err("permission_not_inherited") }`.
`parent_fact` s'exécute avant `authorize_cwd` dans la branche Delegate. Le refus 148 `delegation_grant_required` n'est plus jamais atteint.

Contrat : `contracts/permissions.md` ligne 32 : « Les révocations stables148 [...] restent prioritaires ». Ligne 409 : « Les refus d'identité, révocation, binding et opt-out148 restent ceux du contrat148. Ils ont priorité ». L'oracle 148 est donc cohérent avec le contrat 149. La production s'écarte du contrat.

Correctif minimal proposé (décision Sol) : à la ligne 39, rendre `delegation_grant_required` pour une révocation directe du propriétaire ou de l'instance. Garder `permission_not_inherited` pour le descendant d'une racine révoquée (ligne 41, couvert par `native149_revocation_de_la_racine_coupe_enfant_et_petit_enfant`, qui passe). Je n'ai pas modifié ces trois tests ni la production.

### Échecs de base, hors périmètre 149 (non rejoués sur HEAD, attribution par lecture)

| Test | Cause | Preuve |
|---|---|---|
| `spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises` | `--allowedTools` contient 4 outils natifs 148 en plus (`bridget_capabilities`, `bridget_delegate`, `bridget_task_status`, `bridget_task_cancel`). Le test attend 16 outils. | `BRIDGET_SAFE_MCP_TOOLS` dans `wrapper.rs` vient du commit 347d7885 (148) ; `wrapper.rs` ne contient pas ces noms dans le diff 149 ; `tests/claude_native_permissions_test.rs` non modifié. |
| `spec102_v33_parite_cli_mcp` | `bridget_thread` expose 9 actions, le test en attend 8. | Commit 82f4abeb (session147, membres) ; `store/threads.rs` non modifié par 149. |
| `spec102_v34_migration_base_pre102_et_contraintes_effectives` | `MAX(version)` de `thread_schema_migrations` vaut 3, le test attend 2. | Même commit 82f4abeb. |

Limite : je n'ai pas construit HEAD pour confirmer l'échec. L'attribution repose sur `git blame`, `git diff HEAD` et les fichiers non modifiés.

## Corrections de fixtures (test-only, aucun oracle abaissé sur les tests 149)

Aucune ligne de production modifiée. Fichiers touchés :

- `crates/bridget-daemon/src/native_permissions149_tests.rs`
  - Fixture : vrai Mach-O `/bin/echo` copié en `<fixture>/bin/claude`, command `claude`, PATH `<fixture>/bin:/usr/bin:/bin`, dossier `.git` à la racine (borne la remontée des ancêtres, sinon le vrai `~/.claude/settings.json` avec hooks est lu).
  - `claude_meme_famille` : le fait est capturé à la racine demandée, puis `fact.cwd` est changé. Avant, le contexte était capturé à l'autre racine, donc `settings_revision_changed` masquait `permission_mapping_unavailable`.
  - `snapshot_fige_et_deduplique` : le test « deux contextes » mettait le second contexte dans l'enfant seul, avec un parent Codex sans contexte (donc un seul contexte). Il utilise maintenant un parent Claude et un enfant Claude.
  - `capture_refuse_les_sources_opaques` : l'assertion « toutes les sources sont absentes » ignore les sources `managed` de l'hôte (`/Library/Application Support/ClaudeCode/managed-settings.json` existe sur cette machine). Ajout du cas F5 : lanceur binaire `wrapper-binaire` différent de `claude` donne `permission_source_unavailable`, contrôle `claude` direct accepté.
- `crates/bridget-daemon/src/native_permission_observer149_tests.rs`
  - `Observer::start` lit le PATH du processus : un `claude` Mach-O est placé en tête du PATH une seule fois (`Once`, `unsafe set_var`, voir risque).
  - Racine de test raccourcie (socket Unix limité à 104 octets, `bind` échouait).
  - Client Python : lit jusqu'au `\n`, retire l'espace, rend `SILENT` si aucun octet. `wait_file` attend un contenu non vide (lecture d'un fichier créé mais pas encore écrit).
  - `pid_mort` : le fournisseur lié est un processus séparé, tué avant la connexion du client. Avant, le client était le fournisseur vivant, donc la demande était traitée.
- `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs`
  - Même fixture `claude` / `.git` / PATH. `AgentRegistry::from_json` fusionne le registre hôte (claude, codex, cursor, gemini avec leurs vrais binaires) : les boucles du catalogue ne portent que sur `native-test` et `claude-child` (`fixture_entries`, 2 entrées exigées).
  - `revocation_de_la_racine` : ajout de la tâche parente (`child = enfant-149`, racine `racine-149`) pour que `for_child` trouve le descendant.
- `crates/bridget-daemon/src/delegation_lineage_tests.rs`
  - `chain_payload` : `origin` du maillon i>0 = `inst-child-{i-1}`.
  - `take_changes()` consommé avant les assertions « aucun changement » (3 tests).
  - Pagination : identifiants UUID (le curseur exige un UUID canonique ; avec `t-a2` toutes les formes étaient refusées, donc le test était vide de sens).
  - `show` : arithmétique UTF-8 corrigée (« héllo wörld » fait 13 octets ; offset 2 est au milieu de `é`, offset 3 est valide).
- `crates/bridget-daemon/src/daemon/native_lineage_tests.rs`
  - Formes `cancel` : les deux champs fournis, un seul invalide.
  - `sessions/<child>` : reliquat supprimé avant et après le test (le dossier est sous le parent du `db_path`, partagé entre exécutions).
  - Fichier journal 0644 : `set_permissions` explicite (l'umask 077 masquait `mode(0o644)`).
- `crates/bridget-daemon/tests/lineage_cli_test.rs`
  - Imports `DirBuilderExt`, `OpenOptionsExt` ; plus de `try_clone` sur `ChildStdout` ; `mut` inutiles retirés.
  - Fixture T3 : aller-retour `CommunicationProjectFact` après le fait de liaison (le fait n'a pas de réponse, la CLI courait avant son traitement : `binding_unavailable` intermittent).
  - Liste : une tâche d'une autre racine est semée, la liste attend les 2 tâches de la racine.
- `crates/bridget-daemon/tests/native_delegation_e2e.rs:280` : `Some(SpawnPosture::Discovery)`.
- `crates/bridget-daemon/src/daemon/native_delegation.rs` (bloc `#[cfg(test)]` seulement), voir ci-dessous.

### Oracles 148 remplacés par le contrat 149 (à valider ou rejeter par le parent)

Ces changements touchent d'anciens oracles. Chacun est justifié par une phrase du contrat. Aucun n'a été fait en silence.

1. Builder `request()` : `posture: Some(Discovery)` au lieu de `Some(Development)`. Sans fait, `Development` donne `permission_attestation_unavailable` (`permissions.md` lignes 38-41 : « version1 donne permission_attestation_unavailable sans grant ni fallback discovery »). Le comportement legacy 148 sans fait reste Discovery.
2. `native148_cwd_and_development_grants_are_closed` : demande `Development` explicite, attend `permission_attestation_unavailable` au lieu de `development_grant_required`.
3. `native148_glm_catalogue_uses_native_registry_and_discovery_permissions` : `development_refusal` = `permission_attestation_unavailable` au lieu de `development_protocol_unavailable` (le catalogue 149 calcule `development` par `child_policy` sur le fait).
4. `native148_mission_replay_and_result_wait_for_children_without_t3` : `root_permission(...).maximum()` = `Discovery` au lieu de `Development` (l'enfant admis sans fait est en Discovery).
5. `native148_cancel_stops_native_descendants_before_parent` : le descendant reçoit `parent_task_id = Some(root.task_id)` (nouveau champ ; les descendants sont trouvés par `parent_task_id`). Adaptation mécanique.

## Risques et limites

- `ensure_claude_on_path` modifie le PATH du processus de test une seule fois, avant tout `Observer::start`. `set_var` reste une écriture d'environnement globale pendant que d'autres tests tournent. Aucun flocon observé sur 2 batteries daemon complètes, mais le risque n'est pas nul.
- Le test `wrapper::reconnect_tests::le_domaine_derive_nomme_le_depot_courant` échoue si `TMPDIR` est sous `/Users/moi` (ce dossier contient un `.git`, le domaine devient `moi`). Avec `TMPDIR=/tmp/b149t` : PASS. La consigne « TMPDIR sous `/Users/moi/.cache` » est donc incompatible avec ce test. Les 4 passes finales utilisent `/tmp/b149t`.
- `Observer` et les fixtures `claude` copient `/bin/echo` : ce sont des fixtures unitaires, jamais une preuve de comportement du modèle ou du CLI Claude réel. T037/T038 restent à faire.
- Non vérifié : comportement des 6 échecs sur HEAD ; recette réelle T3 ; les hooks Claude réels.
- Fichiers que j'aurais pu supprimer et que je garde : aucun.

## Binaire debug (prochaines recettes, aucune installation)

| Champ | Valeur |
|---|---|
| Chemin | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget` |
| Copie stable | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-debug/bridget-ed5e28bc8ddc` |
| SHA-256 | `ed5e28bc8ddc680ca8d35a1dce6784c7f8c5d11c627aa07e3bce62e56a7126fd` |
| Version | `bridget 0.1.3`, Mach-O arm64, 54 027 912 octets |
| Construit | 2026-10-10 13:38:45 +0200 |
| Commande | `cargo build -p bridget-daemon --bins --offline` (CARGO_TARGET_DIR externe, JOBS=2, INCREMENTAL=0) |
| Empreinte des sources | voir `sourceFingerprint.value` du reçu JSON (sha256 des digests triés de 196 fichiers `.rs`/`.toml`) |
| Fraîcheur | aucun `.rs` hors tests plus récent que le binaire |
| Source wrapper | `initialFact` publié avant `NativeTui::start` (`wrapper.rs` 4148-4167) présent dans les sources compilées ; digest dans le reçu |

L'arbre de travail n'est pas committé : le reçu ne prétend aucun état Git propre. `headCommit` = 6807c22b plus modifications locales.
Le binaire de `target/debug` sera écrasé par un prochain `cargo build` : utiliser la copie stable pour les recettes.

## Journaux (tous sous le même répertoire)

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/` :
`native-r3-final-all.log` (batterie finale), `native-r3-build-debug-final.log`, `native-r3-compile-1.log` à `-3.log`, `native-r3-daemon-lib-1.log` à `-7.log` (itérations), `native-r3-daemon-all.log`, `native-r3-cli-lineage*.log`, `native-r3-wrapper-reconnect.log`.

## Espace disque

`CARGO_TARGET_DIR` externe : 3,6 Go (607 Mo au départ, +3 Go dus aux profils test et debug des deux crates). `/tmp/b149t` : ~300 Mo, nettoyé en fin de ronde. Ancien `~/.cache/b149t` (724 Mo) supprimé. Copie du binaire : 54 Mo.

## Pour le parent

1. Réactiver Sol native149 pour F-PROD (3 tests 148, une ligne, `native_delegation.rs:39`).
2. Décider si les 5 oracles 148 remplacés ci-dessus sont acceptés.
3. Décider du sort des 3 échecs de base 147/148 (mise à jour des tests `spec094` et `spec102_v33/v34`, hors périmètre 149).
