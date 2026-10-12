# Baseline160 T001 — reçu sans rerun

Une invocation Cargo a exécuté les sept filtres en OR : **276 PASS, 0 FAIL et 2 ignored**, RC0, 208,39 s. Les 223 sources sont identiques avant et après ce run. Les noms exacts correspondent au dernier résumé de chaque cible Cargo.

84 cibles ont été parcourues. Cinq contiennent des tests sélectionnés. Aucun résumé de processus enfant n’est présent. Les filtres ne se chevauchent pas dans ce run : 276 appartenances PASS correspondent à 276 noms qualifiés distincts.

| Filtre | PASS | FAIL | Ignorés |
|---|---:|---:|---:|
| native148 | 27 | 0 | 1 |
| native149 | 78 | 0 | 0 |
| native150 | 51 | 0 | 0 |
| native152 | 56 | 0 | 0 |
| native157 | 43 | 0 | 1 |
| delegation149 | 10 | 0 | 0 |
| spec_087 | 11 | 0 | 0 |

| Cible sélectionnée | PASS | FAIL | Ignorés | Ligne du résumé |
|---|---:|---:|---:|---:|
| lib | 261 | 0 | 1 | 343 |
| native_delegation_e2e.rs | 6 | 0 | 0 | 721 |
| native_delegation_real_glm.rs | 3 | 0 | 1 | 731 |
| native_provider_death149_test.rs | 2 | 0 | 0 | 739 |
| native_wrapper_stop149_test.rs | 4 | 0 | 0 | 749 |

Tests ignorés exacts :

- bridget-daemon / lib / daemon::native_delegation::native_delegation_permissions149_tests::native157_catalogue_latence ; ligne 118
- bridget-daemon / native_delegation_real_glm.rs / native148_parent_codex_fixture_delegates_once_to_real_glm ; ligne 726

Commande réelle, non relancée :

```text
/usr/bin/nice -n 10 python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support -- native148 native149 native150 native152 native157 delegation149 spec_087 --test-threads=2
```

Cwd déclaré par ROOT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota

Reçu : /Users/moi/.cache/bridget-tests160-20261012/T001-baseline-seven-prescribed-filters.json
SHA : 006c0e59772704a69a745324b17c4312ab3c27c075e5a63a363b14a43bd861e3

Log : /Users/moi/.cache/bridget-tests160-20261012/T001-baseline-seven-prescribed-filters.log
SHA : 052fbb0b33cfad9a46301bc44ad9425b9a281bb9795fc5454c2a2091911759f7

Carte exhaustive des packages, cibles, noms, lignes et sources : /Users/moi/.cache/bridget-tests160-preparation-20261012/baseline160.json

Le reçu atteste jobs2, threads2 et nice10. Le namespace privé est /private/tmp/b._lv9pscd. Il ne contient pas les valeurs complètes de HOME et PATH ; elles ne sont pas inventées ici.

Cette baseline porte sur les sept filtres daemon. Elle ne constitue ni une suite workspace complète ni une baseline transport entière. Les deux tests ignorés restent ignorés. Les warnings sont conservés dans le log. La stabilité des sources concerne le run terminé, pas les modifications après le dégel.

Aucun test, source, WT, tâche, service ou Git n’a été modifié par cet agent.
