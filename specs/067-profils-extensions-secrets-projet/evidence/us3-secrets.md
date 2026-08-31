# US3 - Secrets et rotation

Les secrets restent references et stamps sans contenu. Process-env est lu apres admission seulement, sans argument Docker ni store. OutputRedactionLease masque les fragments avant les sinks durables. Une mutation, rotation ou definition divergente bloque les nouveaux spawns et exige recreation ou nouvelle confirmation, sans arreter les agents deja actifs.

Preuves 2026-08-31: cargo test -p bridget-daemon spec_067_redaction --quiet (2 passes); cargo test -p maicie project_profile --quiet (3 passes); catalogue (6 passes).
