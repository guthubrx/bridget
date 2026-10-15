# Développement 168 — lot E

Portée : `crates/bridget-daemon/src/delegation.rs` uniquement.

- DDL168 direct pour base neuve ; migration transactionnelle de la forme163 connue, avec copie SQL des BLOB, `metadata_version=1`, CHECK conditionnel 16/128 Kio et recréation des trois index.
- Toute forme de table, trigger ou clé étrangère non reconnue échoue avant commit ; contrôle `quick_check` et `foreign_key_check` après migration.
- Reçu versionné : v1 reste borné à 16 Kio/c01..c08 ; v2 accepte 128 Kio/c001..c128, canon toujours borné à 512 Kio.
- API CAS publiées : `AutoAdmissionDecisionLevel`, `start_auto_decision_level`, `advance_auto_admission_decision_level`, `choose_auto_admission_at` ; elles vérifient époque, révision et niveau. La dernière reçoit désormais `chosen_metadata: Vec<u8>` avant `now`, vérifie v2/128 Kio/niveau inchangé et persiste cette décision opaque dans le même `UPDATE` CAS que `Chosen`.
- Oracles ajoutés dans le même fichier : migration des cinq états et BLOB, formes de schéma/index/FK refusées sans réparation, bornes JSON v1/v2/canon, CAS époque/révision/niveau, métadonnée de décision opaque rouverte, admission Task v2 atomique, rollback d'owner et rejeu tardif.
- Correctif relecture : la contrainte de taille de table est déplacée après la dernière colonne, avant la clé primaire, dans les DDL de migration et de base neuve ; SQLite accepte ainsi les deux formes sans modifier les bornes.
- Durcissement relecture : DDL163/168 et trois index contrôlés sous forme normalisée exacte ; dépendances FK sortantes et entrantes refusées avant `DROP`, anciennes API163 refusées pour v2, et niveau autorisé seulement `decision_service → lightweight_model`.

Validation exécutée : `rustfmt --edition 2024 crates/bridget-daemon/src/delegation.rs` et `git diff --check` (succès). Aucun Cargo, test, build, lint, Git write ou service n'a été exécuté.

Validation différée ROOT : depuis `/home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte`, exécuter exactement `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 cargo test -p bridget-daemon delegation::tests -- --test-threads=2`. Cette commande doit être lancée une seule fois avec le permis Cargo ROOT ; elle couvre les oracles SQL source E, mais ne vaut ni validation globale du workspace ni preuve d'activation.

Raccord restant : les constructeurs externes de `AutoAdmissionReceipt` doivent fournir `metadata_version` et les appelants v2 de `choose_auto_admission_at` doivent fournir `chosen_metadata` opaque. Les faits de routage, dont `metadata.decision` fermé, restent dans `metadata` v2 opaque, sans duplication de politique dans le store.
