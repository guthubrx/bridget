# US1 - Approbation locale

ProjectProfile compose les profils agents existants, epingle la liaison, la politique et les definitions resolues. La confirmation est exclusivement locale, precedee de la resolution complete et de l avertissement de partage intra-projet. Aucune route UI ou MCP ne permet approve, rotate ou revoke.

Preuves 2026-08-31: cargo test -p maicie project_profile --quiet (3 passes); cargo test -p bridget-daemon --test project_profile_surface_test --quiet (1 passe); cargo test -p bridget-transport project_profile --quiet (2 passes).
