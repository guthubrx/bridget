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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-tests-sonnet-r3.md#extrait-1).

### Oracles 148 remplacés par le contrat 149 (à valider ou rejeter par le parent)

Ces changements touchent d'anciens oracles. Chacun est justifié par une phrase du contrat. Aucun n'a été fait en silence.

1. Builder `request()` : `posture: Some(Discovery)` au lieu de `Some(Development)`. Sans fait, `Development` donne `permission_attestation_unavailable` (`permissions.md` lignes 38-41 : « version1 donne permission_attestation_unavailable sans grant ni fallback discovery »). Le comportement legacy 148 sans fait reste Discovery.
2. `native148_cwd_and_development_grants_are_closed` : demande `Development` explicite, attend `permission_attestation_unavailable` au lieu de `development_grant_required`.
3. `native148_glm_catalogue_uses_native_registry_and_discovery_permissions` : `development_refusal` = `permission_attestation_unavailable` au lieu de `development_protocol_unavailable` (le catalogue 149 calcule `development` par `child_policy` sur le fait).
4. `native148_mission_replay_and_result_wait_for_children_without_t3` : `root_permission(...).maximum()` = `Discovery` au lieu de `Development` (l'enfant admis sans fait est en Discovery).
5. `native148_cancel_stops_native_descendants_before_parent` : le descendant reçoit `parent_task_id = Some(root.task_id)` (nouveau champ ; les descendants sont trouvés par `parent_task_id`). Adaptation mécanique.

## Risques et limites

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/native-tests-sonnet-r3.md#extrait-2).

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
