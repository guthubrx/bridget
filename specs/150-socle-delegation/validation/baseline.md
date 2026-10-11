# Référence de non-régression - Session150 (T001)

Date : 2026-10-11. Code : base `1f2c661d`, worktree non modifié (seuls des
fichiers `specs/` et `docs/` ajoutés). Recette : `umask 077`, racine
`/tmp/b.XXXX`, `BRIDGET_HOME` privé, `cargo test -p bridget-daemon --features
test-support --no-fail-fast <filtre> -- --test-threads=4`.

| Filtre | Réussis | Échecs | Ignorés |
|---|---:|---:|---:|
| native148 | 27 | 0 | 1 |
| native149 | 78 | 0 | 0 |
| delegation149 | 10 | 0 | 0 |
| spec_087 | 11 | 0 | 0 |
| **Total** | **126** | **0** | **1** |

Journal brut : `/tmp/s150-baseline.log` (éphémère).
