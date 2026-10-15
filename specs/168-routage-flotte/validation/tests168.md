# Tests SPEC168 — preuves ciblées

## État courant T038 — corrections d’audit en cours, sans GREEN

T038 reste ouverte. Les quatre échecs ci-dessous sont conservés séparément :
deux défauts atteints et deux premières préconditions de fixture, qui ne sont pas des RED du produit.
Les 70 noms uniques et 102 PASS antérieurs restent des preuves historiques sur leurs empreintes.
Aucun succès des corrections, finding fixed, audit final, FULL ou livraison n’est annoncé.
Les états suivants décrivent les checkpoints antérieurs.

## État courant — revue Sonnet R1 clôturée après contrôles affectés

ROOT a exécuté les quatre tests exacts affectés : **4 PASS, 0 FAIL**.
Ils rejouent quatre noms existants : **70 noms uniques** à dernière issue PASS, **102 PASS bruts**
et **9 échecs historiques** conservés. Les résultats cumulés portent plusieurs empreintes source.
T037 est cochée : **43 tâches sur 56**, 13 ouvertes. T038 audit et T036 FULL restent ouvertes.
Converge P3 reste CONVERGED ; aucune passe P4 ni nouvelle revue intégrale n’est revendiquée.
GEL final des documents pour initialisation de l’audit V15. Les états suivants sont historiques.

État courant avant P3 : six replays exactsPASS après correctionsP2 ;70noms uniques à dernière issuePASS, sans nouveau nom.
Tasks42/56cochées,14ouvertes (T036–T049). Les captures et comptes plus bas restent historiques ; lire le complémentP2 final.
Aucun verdictP3, revue indépendante, FULL ou livraison anticipé. GEL documentaire P3 après cette publication.

État courant avant P2 :70noms distincts à dernière issuePASS ;40/54tâches cochées.
La campagne affectée aprèsP1 a exécuté33testsPASS avec55entrées source/doc avant/après stables.
Lire le complément final pour les assertions et preuves courantes. Les sections R1–R4 ci-dessous restent historiques.

Date : 2026-10-15. Exécutions ROOT sur Cartae ; ce reçu lit leurs artefacts et ne lance aucune commande.
Source de travail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
Preuves privées : /home/moi/.local/state/bridget-delivery168/tests

**57 noms de tests distincts ont une dernière issue PASS : 56 du lot168 et un oracle de compatibilité163.**
Il s’agit de résultats cumulés sur plusieurs empreintes source, pas d’une suite globale rejouée sur un commit unique.
Les journaux contiennent59exécutions PASS et9échecs intermédiaires. Les répétitions ne sont jamais ajoutées aux57.
La capacité compilation apparaît dans registry5 puis telemetry1 ; le wire128 apparaît dans E2E2 puis wireB1 : deux PASS doublons exclus.
Aucun filtre à zéro test compté. Compilation, listes de tests, format, diffcheck et parseJSON ne sont pas des tests.
Aucune FULL, recette fournisseur réel168, activation ou livraison n’est déduite.

## Issues par sélection et reprises

| Artefact JSON | PASS bruts | FAIL bruts | RC | Durée s | Usage final |
| --- | ---: | ---: | ---: | ---: | --- |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json | 9 | 0 | 0 | 18.288 | dernière preuve pour 9 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json | 2 | 0 | 0 | 11.279 | dernière preuve pour 2 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json | 6 | 0 | 0 | 41.285 | dernière preuve pour 6 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json | 6 | 0 | 0 | 29.191 | dernière preuve pour 6 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json | 2 | 0 | 0 | 28.675 | dernière preuve pour 2 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json | 5 | 0 | 0 | 28.557 | dernière preuve pour 4 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage168-r1.json | 1 | 0 | 0 | 28.492 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json | 12 | 1 | 101 | 1.013 | dernière preuve pour 12 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json | 2 | 0 | 0 | 0.006 | dernière preuve pour 2 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.json | 1 | 0 | 0 | 0.003 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.json | 1 | 0 | 0 | 0.005 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json | 3 | 0 | 0 | 0.032 | dernière preuve pour 3 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.json | 2 | 0 | 0 | 0.489 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r1.json | 0 | 3 | 101 | 0.279 | historique ou doublon ; non ajouté au total |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.json | 1 | 0 | 0 | 0.002 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage-cli168-r1.json | 1 | 0 | 0 | 0.027 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r1.json | 0 | 1 | 101 | 0.147 | historique ou doublon ; non ajouté au total |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.json | 0 | 1 | 101 | 0.166 | historique ou doublon ; non ajouté au total |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json | 2 | 1 | 101 | 1.879 | dernière preuve pour 2 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.json | 1 | 0 | 0 | 0.647 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r3.json | 0 | 1 | 101 | 0.342 | historique ou doublon ; non ajouté au total |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-diagnostic-r3.json | 0 | 1 | 101 | 0.244 | historique ou doublon ; non ajouté au total |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r4.json | 1 | 0 | 0 | 0.340 | dernière preuve pour 1 nom(s) |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.json | 1 | 0 | 0 | 0.242 | dernière preuve pour 1 nom(s) |

## Échecs conservés et corrections affectées

- Check C initial RC101 : six erreurs d’intégration (types et variants non exhaustifs). Check de contrôle RC0 conservé séparément.
- R1 Git : un test natif local, trois worker168 et un worker163 échouent. Le runner trouvait Git2.43 ; le préflight exige Git≥2.44. Aucune garde produit n’est supprimée.
- R2 : ROOT met Git2.56 dans le PATH du runner. Deux worker168 et le compat163 passent. Le natif local atteint maintenant le préflight Worktree ; le horsGit worker atteint son verdict réel.
- R3 local : la fixture ajoute/committe .gitignore contenant .worktrees/, comme la précondition154. Le préflight passe ; l’oracle attendait à tort le SHA complet dans un résumé historique abrégé à7caractères.
- R3 worker diagnostique : MappingUnavailable est observé. Le support avait aussi initialisé un dépôt dans le parent ; retirer seulement project/.git ne rendait pas le cwd horsGit.
- R4 : l’oracle local vérifie le SHA complet dans le JSON et son préfixe7 dans le résumé. La fixture worker retire aussi son dépôt parent synthétique. Exact local1PASS et horsGit worker1PASS ; manifeste25sources avant/après stable.
- Les corrections R3/R4 portent les fixtures/assertions et la visibilité privée de remote_candidate_probe_v2. Les reçus C/B attestent aucun changement du contrôle fonctionnel pour contourner ces échecs. Les erreurs et leurs journaux sont gardés.

## Compilation et statiques — hors compte57

- /home/moi/.local/state/bridget-delivery168/tests/c-check-initial.json : RC101, 26.527s. Check de compilation daemon/tests, aucun test exécuté.
- /home/moi/.local/state/bridget-delivery168/tests/c-check-controle.json : RC0, 14.038s. Check de compilation daemon/tests, aucun test exécuté.
- /home/moi/.local/state/bridget-delivery168/tests/t035-integration-build-r1.json : RC0, 32.404s. Cargo --no-run produit les exécutables ; aucun test compté par cette étape.
- /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r2.json : RC0, 32.279s. Cargo --no-run produit les exécutables ; aucun test compté par cette étape.
- /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r4.json : RC0, 31.801s. Cargo --no-run produit les exécutables ; aucun test compté par cette étape.

- Diffcheck final : RC0,0,043s.
- rustfmt initial : RC1,6,862s. Les lib.rs entraînent un parcours récursif horsdiff, notamment t3code_mcp.rs hérité. Aucun formatage massif.
- Contrôle correctement limité aux24fichiers Rust modifiés : --config skip_children=true, RC0,2,174s, sortie vide. Aucun GREEN de format global.
- ROOT rapporte aussi le parse des JSON modifiés PASS. Le JSON statique lu contient seulement rustfmt/diffcheck : cette attestation de parse est distinguée d’une trace dédiée et ne change aucun compteur.

Commande complète du format ciblé exécuté par ROOT :

```sh
/home/moi/.cargo/bin/rustfmt --edition 2024 --check --config skip_children=true /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli_lineage.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/communication/client.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage_tests.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_mcp.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lib.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lifecycle.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/cli_arguments_integration_test.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/lineage_cli_test.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/native_delegation_e2e.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lib.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs
```

## Commandes exactes de test et de compilation

Les argv ci-dessous proviennent des JSON privés. Le cwd est la source de travail en tête.
Les exécutions d’intégration utilisent directement les exécutables identifiés par Cargo --no-run.
Les PATH spécifiques R2 sont conservés dans leurs JSON ; aucun environnement complet n’est reconstruit.

/home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-transport --lib spec168 -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-transport --lib routing168 -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib delegation::tests::spec168 -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib decision_service::tests::spec168 -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib host_resources::tests:: -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib routing168 -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-lineage168-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib spec168_lineage -- --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e native_delegation::tests::spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e delegation_mcp::tests::spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e provider_limits::tests::spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e routing168_capacite_compilation_est_declarative_et_bornee --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/cli_arguments_integration_test-e2c91ffaae1a01d7 spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca worker168_ --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca spec168_probe_v2_128_ids --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-lineage-cli168-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/lineage_cli_test-2487643096f8e99e spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r1.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca worker168_ --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r3.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-diagnostic-r3.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::worker168_shared_hors_git_general_sha_et_erreur_ambigue --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r4.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::worker168_shared_hors_git_general_sha_et_erreur_ambigue --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/c-check-initial.json

```sh
/home/moi/.cargo/bin/cargo check -p bridget-daemon --tests --offline --locked -j2
```

/home/moi/.local/state/bridget-delivery168/tests/c-check-controle.json

```sh
/home/moi/.cargo/bin/cargo check -p bridget-daemon --tests --offline --locked -j2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-integration-build-r1.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --test cli_arguments_integration_test --test native_delegation_e2e --test lineage_cli_test --no-run --message-format=json
```

/home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r2.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --test native_delegation_e2e --no-run --message-format=json
```

/home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r4.json

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --test native_delegation_e2e --no-run --message-format=json
```

## Dernière issue par nom unique

Chaque ligne renvoie à sa dernière issue connue. Le statut PASS ne transforme pas un ancien hash en hashfinal.
- claude_stream_json::tests::spec168_exit_nonzero_refuse_meme_une_sortie_structuree_valide — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_faux_executable_verifie_flags_env_cwd_et_stderr_neutralise — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_formateur_refuse_outils_capacites_et_reponses_contradictoires — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_objets_json_sont_bornes_avant_leur_decodage_detaille — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_sortie_leader_ne_libere_pas_un_descendant_qui_tient_stdout — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_stdin_non_lu_expire_depuis_le_lancement_et_arrete_enfant — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_stdout_excessif_est_borne_et_arrete_sans_attendre_la_fin_naturelle — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- daemon::native_delegation::tests::spec168_capacite12_secours_natif_deux_connexions_env_distincts_et_effort_impose_refuse — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_catalogue_12_128_meme_uid_sans_agent_gardes_et_zero_effet — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_charge_entiere_objectif_contraintes_et_tache_sans_handoff — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_cinq_axes_effort_impose_et_defaut_natif_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_crash_pending_et_started_reprennent_le_repli_fige_sans_nouvelle_consultation — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r4.json
- daemon::native_delegation::tests::spec168_modele_unique_et_deux_niveaux_off_ne_consultent_pas_et_rejouent_avant_politique — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_niveaux_inaccessibles_zero_essai_et_effort_role_non_qualifie_explicite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_rollback_admission_sql_garde_chosen_puis_reprise_sans_decideur — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_service_post_reel_concurrence_enveloppe_retour_tardif_reprise_et_reouverture — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- daemon::native_delegation::tests::spec168_union_4096_reservations_et_taches_borne_le_puits_natif_sans_effet — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json
- decision_service::tests::spec168_effort_valide_ou_defaut_atteste_sans_niveau_invente — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_laya_default_natif_null_et_transport_legacy163_restent_distincts — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_laya_http_refuse_choix_effort_et_metadonnees_invalides — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_laya_post_choisit_sur_l_ensemble_entier_et_garde_ses_faits — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_laya_refuse_capacite_et_deadline_avant_tout_reseau — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-lineage168-r1.json
- delegation::tests::spec168_cas_niveau_epoch_revision_et_choix_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_migration_faute_copie_rollback_conserve_legacy — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_migre_163_sans_alterer_les_blobs_et_borne_v2 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_refuse_schema_index_fk_sortante_et_preserve_la_base — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_reouverture_refuse_trigger_et_fk_entrante — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_v2_persiste_decision_fermee_et_admission_atomique — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation_mcp::tests::spec168_mcp_auto_delegate_negocie_v2_et_preserve_le_schema_ferme — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json
- delegation_mcp::tests::spec168_wire163_byte_compatible_et_nouveau_client_refuse_ancien_maitre — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json
- host_resources::tests::spec168_cpu_initial_delta_inconnu_apres_erreur_et_reset_epoch — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json
- host_resources::tests::spec168_linux_guest_ram_load_et_bornes_native — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json
- protocol::lineage_protocol::tests::spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- protocol::routing168_protocol_tests::routing168_auto_v2_preserve_le_defaut_general — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json
- protocol::routing168_protocol_tests::routing168_ids_et_probe_v2_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json
- protocol::tests::spec168_prepare_v2_contrat_ferme_et_prepare_v1_inchange — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- provider_limits::tests::spec168_quota_connexion_instance_definition_epoque_ttl_et_credits_100 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.json
- registry::tests::routing168_129_meme_desactive_refuse_sans_catalogue_partiel — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_axes_independants_fixes_et_contradiction_restent_des_filtres — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_capacite_compilation_est_declarative_et_bornee — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.json
- registry::tests::routing168_catalogue_12_preserve_connexions_efforts_et_defaut_natif — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_decideurs_inaccessibles_sont_valides_et_claude_distant_refuse — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- remote_worker_162::spec168_probe_v2_128_ids_reste_distinct_du_wire163 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.json
- remote_worker_162::worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.json
- remote_worker_162::worker168_probe128_connexion_revoquee_prepare_et_base_epinglee — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json
- remote_worker_162::worker168_reconnexion_epoch_et_capacite163_restent_distinctes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json
- remote_worker_162::worker168_shared_hors_git_general_sha_et_erreur_ambigue — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.json
- spec168_catalogue_cli_ancien_maitre_refuse_avant_toute_lecture — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_catalogue_cli_humain_negocie_12_128_et_filtre_machine_sans_identite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_catalogue_daemon_reel_12_128_sans_agent_t3_task_ni_modele — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.json
- spec168_delegate_workload_et_axes_refusent_les_valeurs_invalides_avant_connexion — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_lineage_cli_v4_conserve_version_et_refus_sans_downgrade — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-lineage-cli168-r1.json

## Matrice des tâches T002–T035

Légende : TP=transport spec168(9), TR=transport routing168(2), ST=store(6), DS=décision(6),
HR=ressources(2), REG=registre(5), N=13natifs, Q=quota(1), CLI=CLI168(3),
E2E=catalogue réel isolé+wire(2), Lunit=Lineage store(1), LCLI=Lineage CLI(1),
W128/Wepoch=workerR2(2), WhorsGit=workerR4(1), W163=compatR2(1).
Les suffixes N désignent le scénario natif exact listé plus haut ; NlocalR4 remplace son échec R1.
Les cochages constatent les tâches code et leurs preuves contrôlées. Ils n’attestent ni production, ni tous les modèles réels.

| Tâche | État documentaire | Preuves | Portée et limite |
| --- | --- | --- | --- |
| T002 | cochée | TP/TR, MCP, W128/Wepoch/W163 | Types fermés, négociation168, wire163 et Lineagev4 ; publication B et réemploi confirmés. |
| T003 | cochée | ST | DDL neuf, migration BLOB dans cinq états, index/FK/trigger, faute de copie et rollback. |
| T004 | cochée | ST, Nservice/Nrollback | Reçu versionné et CAS niveau/époque/révision ; décision et Chosen atomiques, lien Task conservé. |
| T005 | cochée | REG, CLI, E2E | Matrice12/128 entière ; limite129 y compris désactivée ; axes et modèles distincts. |
| T006 | cochée | REG et reçu A | Provenance/défaut effort résoluble ; catalogue pur sans découverte/auth. Droit fournisseur demeure inconnu. |
| T007 | cochée | Ncatalogue, E2E, CLI | MêmeUID, rôle Client et capacité dédiée ; lecture réelle sans agent/T3, aucun effet métier. |
| T008 | cochée | CLI, E2E | Lecture humaine --catalogue --json, filtre machine et refus d’ancien maître. |
| T009 | cochée | REG, Ncatalogue, CLI, E2E | Catalogue12/128, registre129, données publiques et zéroTask/reçu/fournisseur dans namespace isolé. |
| T010 | cochée | MCP, CLI, Nunique | FaçadeV2 stable, axes/workload fermés ; wireV1 explicite et refus d’ancien maître. |
| T011 | cochée | Naxes, REG | Axes indépendants, effort imposé fermé, défaut natif null et contraintes contradictoires. |
| T012 | cochée | Naxes/Nunique/Ninaccessible, MCP, CLI | Modèle seul/machine seule, contraintes, rejeu avant politique fraîche ; refus version et enveloppe sans bascule. |
| T013 | cochée | HR, W128, reçu B/C | Collecteur natif borné, premier CPU inconnu, un relevé du lot128 ; réemploi du tick publié sans nouveau daemon. |
| T014 | cochée | Q, Nscore et reçu C | Faits quota liés connexion/instance/définition/époque/TTL ; crédits disponibles avec fenêtre100%, aucun pool inventé. |
| T015 | cochée | Nscore, Q, REG | Score borné, ordre stable, inconnus neutres, saturation exclue avant construction de charge ; compilation déclarée. |
| T016 | cochée | W128/Wepoch/W163, Nscore, E2E | Probe128 sans effets, première mesure, révocation fraîche et ancienne époque/capacité séparées. |
| T017 | cochée | Ncharge, DSpost, N12 | Charge entière/contraintes/objectifs et agrégation conservées ; capacité insuffisante explicite, pas de top8. |
| T018 | cochée | TP, qualification CLI distincte | Transport Claude isolé testé par faux exécutable ; flags/env/cwd, StructuredOutput seul, stderr nul, bornes avant parsing. |
| T019 | cochée | DS, Nservice/N12 | Adaptateur Laya168 entier et fermé, legacy163 séparé, défaut effort, capacité/deadline et permis jusqu’au drain. |
| T020 | cochée | Nservice/N12/Nunique/Ninaccessible/Nservicepost | Chaîne native vers HTTP/transport contrôlés, niveauOFF sans essai, CAS et rejet des réponses tardives. |
| T021 | cochée | TP, DS, Nservicepost | Capacités/outillage/sorties contradictoires, stdout/JSON, timeout dès spawn, groupe enfant/descendant et permis réel. |
| T022 | cochée | Nservice/N12/Nunique, Nscore | Parcours contrôlé complet : service valide, liste12 au secours, OFF→score et ID exclu non réintroduit. Pas recette fournisseur réel. |
| T023 | cochée | N12, W128, Q et reçu C | Connexions résolues via lanceurs propres, environnements isolés et recontrôles enfant ; aucun credential transféré. |
| T024 | cochée | W128/WhorsGit, TP | PrepareV2 recontrôle connexion exacte/moteur/digests/droits avant effets et Start après révocation ; v1 conservé. |
| T025 | ouverte | REG, N12 — PARTIEL | Deux références Codex représentées ; deux environnements réellement observés dans N12 sont Claude, pas Codex. Preuve exacte des deux environnements Codex reste ouverte. |
| T026 | cochée | ST, Nservicepost/Nrollback/Nunion | CAS et ancienne réponse, rollback Task/reçu, états techniques conservés et rétention partagée au puits. |
| T027 | cochée | Npending/Nrollback/Nservicepost et reçu C | Pending/Started/Chosen récupérés depuis snapshot ; même choix/repli et aucune nouvelle consultation ; gardes existantes réemployées. |
| T028 | cochée | ST, Nunion/Nrollback et procédure documentée | SQLite persistante, BLOB non textuels/cinq états, CHECK16/128Kio, rollback/reouverture/atomicité et UNION ; retour arrière reste procédure opérateur. |
| T029 | cochée | TP, Lunit, LCLI | Lineagev4 depuis décision figée, v3null168, placement et163 conservés ; CLI sans downgrade silencieux. |
| T030 | cochée | Nservicepost/Npending/Nrollback, ST | Concurrence/collision, réponse tardive, réouverture et crashPending/Started/Chosen/Admitted ; uneTask et choix durable. |
| T031 | cochée | NlocalR4, CLI/MCP et reçu C | SHA source exact pour Shared/Worktree, horsGit général distinct de métadonnées cassées ; aucune base distante implicite. |
| T032 | cochée | W128/WhorsGit, W163 | Base au Prepare, base_ref antérieure àHEAD, SHA manquant/inexact, horsGit et révocation avant fournisseur. |
| T033 | ouverte | NlocalR4, W128/WhorsGit — PARTIEL | SHA/base_ref/horsGit et Git cassé exercés. Aucun scénario I/O/timeout Git168 distinct identifié parmi les57 ; timeout HTTP n’est pas assimilé à cette preuve. |
| T034 | ouverte | NlocalR4, Lunit/LCLI — PARTIEL | JSON garde SHA complets/dirty, résumé abrégé7, branche/path/Kept et source inchangée. Pas de preuve de deux worktrees sur deux machines ni de récupération distante. |
| T035 | ouverte | Tous les groupes et statiques — PARTIEL | 57noms à dernière issuePASS, compilation et format ciblé valides ; dépendances/preuvesT025/T033/T034 encore ouvertes. |

T001 reste la preuve Analyze. Total après consolidation :31/49cochées,18ouvertes.
T025/T033/T034/T035 et T036–T049 restent ouvertes. Les trois niveaux réels, le déploiement,
la synchronisation de deux machines et le nettoyage exigent leurs preuves opérationnelles prévues.
La préqualification CLI Haiku réelle antérieure est hors compteur57 et ne vaut pas recette native168.

## Provenance des sources

Les manifestes identifient les états testés ; ils ne sont ni fusionnés ni remplacés par le hash actuel.
- /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json : 4 entrées avant/après identiques pendant cette exécution.
- /home/moi/.local/state/bridget-delivery168/tests/t035-unit-batch-r1.json : 23 entrées avant/après identiques pendant cette exécution.
- /home/moi/.local/state/bridget-delivery168/tests/t035-integration-batch-r1.json : 25 entrées avant/après identiques pendant cette exécution.
- /home/moi/.local/state/bridget-delivery168/tests/t035-affected-r3-hashes.json : 25 entrées avant/après identiques pendant cette exécution.
- /home/moi/.local/state/bridget-delivery168/tests/t035-affected-r4-hashes.json : 25 entrées avant/après identiques pendant cette exécution.

Le transport9 initial porte protocol.rs dd60dc83… ; le lot unitaire et les suivants portent ac7f9d1b….
Le reçu B décrit le retrait d’une variable de fixture inutilisée entre ces deux états ; pas de rejeu global sous-entendu.
Le manifeste transport initial couvre managed_session.rs ; celui-ci n’est pas dans le manifeste final25.
Les huit différences entre unitaire et intégration sont conservées dans leurs manifests ; les tests unitaire restent des preuves de leur état.
Entre build d’intégration et R4, les changements concernent daemon.rs, native_delegation.rs et la fixture worker162.
Leurs empreintes précises sont gardées dans les manifestes ; les12natifs déjà PASS n’ont pas été rejoués pour le seul ajustement de fixture local.
Les deux replaysR4 affectés ont une source stable avant/après. Aucun «57PASS tous sur hashfinal unique» n’est revendiqué.

## Artefacts lus et sommes SHA-256

| Artefact absolu | SHA-256 |
| --- | --- |
| /home/moi/.local/state/bridget-delivery168/tests/c-check-controle.json | 79ce1ce15141ea500284dcdb837bfe07914622d5aa1e64edbd94073122891fa1 |
| /home/moi/.local/state/bridget-delivery168/tests/c-check-controle.log | 1e7298f556fa6a866fb2fc23fc64574cf808c342afb765ba3809a45fed742e50 |
| /home/moi/.local/state/bridget-delivery168/tests/c-check-initial.json | 4b814d9eb40cf9373db993c4bba2cd14723c187334fd7cb1ca89960c76bedf2c |
| /home/moi/.local/state/bridget-delivery168/tests/c-check-initial.log | a1a962f5f71e8d6ea495c21cec09128203aed566fae188f4c9860c9704037df8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r2.json | e88e0461792b675e527f14e1361d38aa0b2a1a75856d184a2c2d75368ac4c86d |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r2.stderr.log | 55cce49e2b3878d2ac223ac024897c4b4e24db88685a0fd6663c99d9b6542f31 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r2.stdout.jsonl | 48e1485115283afb69cd4acf2fe61d24b8c00fbbf1804a8bfb821276b2834585 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r4.json | 1416b97fbd6cb697fd64a881b154a4f129877a54869ae5a86d52da5c13b026f5 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r4.stderr.log | 39b18c252fea98187b56e2b670012b1eacb1b1d00fd967adb7e9c739acf416a8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-build-r4.stdout.jsonl | b269b88719528f4d48503675eeb0bba3b32fa08767a66b1103febd93a6a16e57 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-r3-hashes.json | 26c2f9171860332e3d1650346872ab01be9b099f530503b9443d45af7553f75f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-affected-r4-hashes.json | 373f3ef1f4aea23df0724d855fef9c4bb3dbdbcd6078002062c510c74e82056c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json | 6918edf78828bca8161ad23504213e327f3bd4a40a9f3714f2f1514bfee4ad4f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.log | d583dc720337017c8b08fa6f4aada3d5a4e2b9abb8c62bdd14e2f1644c1446b8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json | c3561e8605cbf7801a7ac56519b91932d11060723db8ee1d75d4d87bb0828538 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.log | 0246b476dd94649eab486c9df51c37a27fdfd1f2968402a0935101e100fc3c3c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.json | a030d124fb153e2b0847872dbbf49810f15cc7f2b363e6de2913c13d7f313d4d |
| /home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.log | 308811455fc604be2f17ed2f52e9fc784d953fd8132c5bdbe53eed6d638328a0 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-final-diff-check.log | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-final-rustfmt-scoped.json | e23c901a87493b9cf97cd3c10572af7f6756a9d07df8bd069400bff0fa944e6e |
| /home/moi/.local/state/bridget-delivery168/tests/t035-final-rustfmt-scoped.log | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-final-rustfmt.log | 030f06ed973d74d2caf24fd9925babaf61040a6e1cf51f5e3f6c3100a9b0edac |
| /home/moi/.local/state/bridget-delivery168/tests/t035-final-static.json | ce06917d6907de232a0bdb40ef052afce2e8ba8a5c5f423002f316371ad12fdf |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.json | d6aed3207f6141ca50fafc6464b68c46b206ff73b729f65a10904462e10e17a4 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r2.log | 261ff292b2cd7803bbb2fce00df6b4975335eca48ada24bf4f9603eaba023ce8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r3.json | 8dd3b9989b88a3f27609d1e8a140a0d322c816e3765074fc3565d89aaaf046e7 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r3.log | 27f70a3d75dae9b1dc46d36252417afba02ed87e1607d42573b4be137861022f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r4.json | ee0e821589b2647117a9900a2d57b1a6fdd4e96bb206ef965d286df673157894 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-git-local168-r4.log | b1e7048d12e6641eaaa177072d66fd41ecba7c15f08fb2507078fa08141838a8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-integration-batch-r1.json | 541103c0f43cd83387c675266b9a7a4be7a12496422ea3d65c8d1674b60e5018 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-integration-build-r1.json | 0ca884a321622f2aa1d139606ee172d8c5c1b045aae7c026c09e1e6371ae7226 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-integration-build-r1.stderr.log | 2f57b95aae94f63b020645ae37537e541f7676a92d4c2b16900e5e5af4ef820e |
| /home/moi/.local/state/bridget-delivery168/tests/t035-integration-build-r1.stdout.jsonl | 04a50c2fbfcddf84af71ec58b83ebe078e6915ada5e618b6d3bd00e939db680c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage-cli168-r1.json | f2b7871e2ac9d8dcf0d373acc933aa76f5aecf93979cc0cfda8d910b7a80398c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage-cli168-r1.log | 6109c9d70b7a19c16ab65b003c1243ba75944084533b1a25af84ed548154f4ae |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage168-r1.json | 5d72455feed8e2236078dafd1940aeef30eb4b6740d991a80639780bd8cdc5ca |
| /home/moi/.local/state/bridget-delivery168/tests/t035-lineage168-r1.log | 751a46af1afd23f844b693389d85cced6182184886b62eddcd2e2f18a0f1a971 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json | d731a026a33903a170d0c00a949cd6344ed23c2788b96c24297c5d3e343e67e9 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.log | 9ce92ec790af25167498109ce2b745c9d2ff4d615c61179235d368a034824705 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.json | 3b2fa2bc4fc260eedb7f80e5c9fa657229cd3cde9d0f37429a6a077833978824 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-native168-r1.log | cc17b134a0a0fd2d5ebe961752d5707cfd6e1923b8ad90820a577d7c64780bde |
| /home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.json | 164e52cc23a1621aaa440b8f283b68d01207f26bccc33bdc9acacc4d1d5c69d8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.log | f94d82b373f90a21a52b70c48c52341d42338ad412a78b358659b54cfa4864a9 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json | 39064e959ddac3b7556c4bd0a6845dc1234f39d7ccb6aa063ad3f69db20976cb |
| /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.log | 2e082d5a131085192c0f68e27a0a65bd3f512d93884006befebaa7e6ac447fc3 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json | 2ca2202c1b96a8ab7f23f8086bf68269101fa10541d8e6d65aa63294de970f49 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.log | 8040b4da46a67e4ef1940e7d867992eec918ae5a98aa0858f5fbf5269c1ab5fb |
| /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json | 89a93285766391bc0f079ae0f00a1da12997eb933fa80073de1a014da53d2ad0 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.log | a29b02533759c348ae3dfd8906c9a356553ec4c86731508fbcee5807ca983136 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.json | 52e80dc24c481c068f470a46c0ad57936917c5ad2969affb56d5930cdbd51601 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.log | 963b135dbbd23a285bbae39fbbf402386762e82bc65f327556dcafec4ddbf93a |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json | f93cc05731b1608ab5121e9e2e271874cdb02302bf3e32a267d07f244ca97af3 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.log | d88a523414ef6c09ca14bb30d6e8fdbedda27c835d795bd9cb8a097634cbcb9a |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json | b273b1eeae313ad5f93e591665c03d8d6ca7317533f7534924f4119fc8366a9c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.log | 2d66ef3ec7fe10a14120736f6aa2a8155111199b14916428e493311821332fec |
| /home/moi/.local/state/bridget-delivery168/tests/t035-unit-batch-r1.json | 40f07aa2a2d44a5f2f59731898c660ca20ad0ced623cd549157cde9777d98ce8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-diagnostic-r3.json | f0686073fd53b93d9e97e1ab26a9e76fc120035d91f569cf30d137dadfcbc321 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-diagnostic-r3.log | 66291edea9374f15e98b17cf3f8351f224a3be7d6ad99a1227e8aa0bf4a836a0 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.json | 4d2b3023ac9a3d00110beb0637af12dca49188b10d119390f5cffd6b59c6028f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.log | 92728130797e53c8b04efe8521841ca78a49e75114ad115e4b35653280aff8d1 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.json | ce577d6a59e29705cc86ee99af337967811be0b1003cd0a15bbad4f440b8a309 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.log | cf9ea2d395ab19172801207c41d9f5033a428dad6b92136434bc82a2374fa17f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r1.json | 5f00f698a73616826bd927cdd7f2e68c801818c908eea779ed72885a049e05d4 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r1.log | fd0c04aba3ac5f9eddd839f67a97ab90302c351a6eff1fe7fcb5fb70185bd8f8 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.json | 9e216ddf047c725e445e5e8956aa5790fae12cf81b560107a2d7f83dccbbf17a |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.log | 1abe6fe8aa7d84af7e7207c9956a38f7852a4a2d2d47f65cce229b4dc3bec980 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r1.json | 6397e6535d3569973f7cae6c2d0eb91a227d4ecd6502ba754ca04d1d0051bfc5 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r1.log | 195cd2c35ecc285ef477c6bcbd99e6f94cfd76d20066e8cd082f07326229e045 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json | c0df0264491d0daf42b83e859c9bcdc51b9d96265097f0f3749d73d1b10be8c6 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.log | 3571a37c90d1e856d92db9f88ecb6e040c203bbe950008ec42174ee4bacb0459 |

Les espaces temporaires et caches inscrits dans ces reçus sont des traces historiques,
pas une autorisation de suppression. Les JSON/logs/manifests privés sont conservés intacts.
Aucun Cargo, test, build, Gitwrite, service ou configuration de production exécuté par cette consolidation.

## Gel documentaire

Après publication de cette consolidation, aucune écriture documentaire pendant Converge.
Seul un appendTasks explicitement ordonné par ROOT est autorisé pendant la passe.
Tout complément de preuve ou correction attend la sortie de cette passe et un mandat ROOT.


## Complément hors passe après corrections P1 — campagne affectée avant P2

Cette section donne l’état courant. Les captures57/31 et32/54 précédentes restent historiques.
ROOT a exécuté33tests :13natifs +7décision +1Lineage store +10Git communs +1Lineage transport +1LineageCLI.
Tous passent, aucune sélection vide, aucun échec ni ignoré dans ce lot.
Namespace : /tmp/b168p1-m6y_5jyp.
Manifestes55entrées avant/après byte-identiques, y compris sources et documents pendant la campagne :
/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-hashes.json.
Tous les JSON des sélections/builds reprennent le même source_hashes.

Recomptage depuis les JSON/logs privés :70noms distincts à dernière issuePASS.
Par rapport aux57précédents, seuls trois noms décision et dix noms Git communs sont nouveaux.
Les13natifs étendent leurs oracles existants ; T054 ne crée pas de nom supplémentaire.
Au total brut conservé :92exécutionsPASS et9échecs historiques, sans assimilation des replays à de nouveaux tests.
Ces70issues portent plusieurs empreintes successives ; aucune suite globale70sur hashunique revendiquée.

| Sélection | PASS | FAIL | Ignorés | RC | Durée s |
| --- | ---: | ---: | ---: | ---: | ---: |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json | 13 | 0 | 0 | 0 | 1.337 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json | 7 | 0 | 0 | 0 | 2.923 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage.json | 1 | 0 | 0 | 0 | 0.018 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json | 10 | 0 | 0 | 0 | 1.217 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-transport.json | 1 | 0 | 0 | 0 | 0.002 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-cli.json | 1 | 0 | 0 | 0 | 0.027 |

Commandes ROOT exactes, déjà exécutées :

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168 --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e decision_service::tests::spec168_laya_ --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e workspace_git::tests:: --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-transport.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_transport-5726269dfab634e9 protocol::lineage_protocol::tests::spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3 --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-cli.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/lineage_cli_test-2487643096f8e99e spec168_lineage_cli_v4_conserve_version_et_refus_sans_downgrade --exact --test-threads=2
```

Build transport : RC0, 11.609s, --no-run ; aucun test compté.

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-transport --lib --no-run --message-format=json
```

Build daemon : RC0, 38.675s, --no-run ; aucun test compté.

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --test lineage_cli_test --no-run --message-format=json
```

### Assertions retrouvées et clôtures documentaires

| Tâches | Oracle(s) réellement exécuté(s) | Assertions lues dans les sources de cette campagne |
| --- | --- | --- |
| T033 | workspace_git::tests::native154_git_sorties_cumulees_bornees ; workspace_git::tests::native154_git_delai_termine_son_enfant ; huit autres tests communs | Sorties tronquées/borne et délai avec arrêt enfant, sur workspace_git.rs actuel. Les cas168 SHA/base/horsGit restent les preuves natives/worker déjà conservées. |
| T034,T053 | daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git | Deux clés et Task distinctes, chemins/branches distincts, deux worktrees simultanés au même SHA complet ; un commit enfant puis rapports base/head/branch_commit/commits/dirty/Kept pour chacun ; JSON durable exact ; HEAD/contenu source inchangés. |
| T050 | daemon::native_delegation::tests::spec168_charge_entiere_objectif_contraintes_et_tache_sans_handoff ; spec168_service_post_reel_concurrence_enveloppe_retour_tardif_reprise_et_reouverture ; spec168_service_valide_choisit_la_combinaison_complete | Charge/POST conserve continuité locale, cacheunknown, compilation déclarée3 même sans mesure fraîche, termes du score et quotas ; reconstruction depuis snapshot identique sans horloge fraîche. |
| T051 | delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null ; oracles Lineage transport et CLI | Déterministe sans reason avec facts fermés ; ressources/quota connus puis inconnus, âges historiques invariants malgré contact plus récent ; BLOB inchangé à la lecture ; clécredential inconnue refusée ; v1/v2/v3 et placement préservés. |
| T052 | Trois nouveaux oracles DS listés ci-dessous ; scénario natif service valide étendu | Santé négative en cache : un GET, aucun POST et motif conservé pendant cooldown, puis GET/POST après expiration ; cache lié au service_id ; GET/POST même deadline et permis réel jusqu’au drain ; secours natif effectivement choisi. |
| T054 | daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete, nom existant étendu | Probe corrélé : cible distante95%CPU avec continuité pinnée retirée devant alternative10%CPU fraîche ; POST n’expose que les deux candidats locaux ; réponsec003 exclue refusée ; admission finale locale unique ; rejeu sans nouveau probe/HTTP ; machine imposée saturée refuse sans nouvelleTask. |
| T035 | Six sélections ci-dessus, builds et statiques ciblés antérieurs | Campagne ciblée effectuée, dépendances contrôlées désormais soldées. Aucun succès de FULL, audit/revue finale ou production déduit. |

Trois nouveaux noms D :
- decision_service::tests::spec168_laya_reutilise_sante_cachee_par_identifiant_service
- decision_service::tests::spec168_laya_sante_negative_et_cooldown_interdisent_post_et_gardent_motif
- decision_service::tests::spec168_laya_sante_post_partagent_deadline_et_permis_reseau_jusqu_au_drain

### Sources et artefacts de la campagne

Les lectures de code ont confirmé les empreintes annoncées ; aucune source n’est modifiée par cette consolidation.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs : f981c72e7dfd9f68c9cfa48967c7a01cfa4d72a39f185b1416b8852436d4b162.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs : 56ef912ddf54c3c531ce28bdf33cc60182028de6c6e5d55faae9107c3e1edb5f.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs : 77fd7368fd7f9372614237efd0edfcbc2593ea084175dd5b691ae6878c814049.

| Artefact absolu | SHA-256 |
| --- | --- |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-daemon.json | 05b73e89d3a15bcc77e44877bdbeefd4a2e623fe73845f620711bf7701711404 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-daemon.stderr.log | 6a05548013a9307182b43f4642d28e19c8d12a40aa2a4100f4ffa67cdedd4594 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-daemon.stdout.jsonl | 7cc6abdcb198e112a6d15a3484c6ab9a5849f5c7a9a57f2feb46e266f7b712d1 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-transport.json | a432a6a87fa385133fba1278e0e47bd84ec68641c486e7449aebf1ca4af05e84 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-transport.stderr.log | 43d901b774b34c106ff67c738cdda1b1b63b0f88c472e86f58319c4274dc3cf0 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-build-transport.stdout.jsonl | 81448e24ef989807e2b11e9bd4ae891b20c14974178da98fb8e5df1b1d8ae747 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json | 4a29101890382a7399528a490cdad72afc6637b92401809259c6d92df059828c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.log | 57290e7875510542d0a0ee4d8aad74fa0dae7d805c26543b060d1049f94ea871 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-hashes.json | 67bd7e3f4e0fa289282c8c28c57b7f490c1dab9693dc1fd79bd873f85a56bfde |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-cli.json | ad66ba1d8377605da7fc13cd46075b7a08f61d4fbcdf2c06eaf021e5d1f6e713 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-cli.log | 6109c9d70b7a19c16ab65b003c1243ba75944084533b1a25af84ed548154f4ae |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-transport.json | ea750258db42b64b95974cdda69db3e90887dded3218e1c676ceaf1f5a0f885b |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-transport.log | bd492a5ad051af3ddaafb54e553ee41abe31dc34385e5d2bfb64bbbe3643641c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage.json | e564d66de71c0d8fa27d2723a3ff51708f1cdcc382d6a15d036aacc5c87a75e9 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage.log | 02d09f7680180711340486295d7fdeaf9322f64a6dffe975fc6b2472556d993e |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json | 0052fa7738bf8101fe13f19dcdc844aa88854f8b6c1fbb27452dd0ed843ba27f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.log | 1d2a9b4507964ab4f7971b7ee7b9e341169f5c07925ff1053f8a6493e75382da |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-static.json | 7214efb90c9288b79c78fd81808327c68ebbcf3bd74ff3a894045edd4d0de3c3 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json | b3494c5620d6a16043bb9f967d625e52ff105ce351fc1d0058f0488980026fe1 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.log | 94bf26755f4600bb6d6321c6aec423cc4a5b591b285010310ffb3fa80609a490 |

### Dernière issue nominative courante —70 noms

- claude_stream_json::tests::spec168_exit_nonzero_refuse_meme_une_sortie_structuree_valide — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_faux_executable_verifie_flags_env_cwd_et_stderr_neutralise — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_formateur_refuse_outils_capacites_et_reponses_contradictoires — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_objets_json_sont_bornes_avant_leur_decodage_detaille — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_sortie_leader_ne_libere_pas_un_descendant_qui_tient_stdout — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_stdin_non_lu_expire_depuis_le_lancement_et_arrete_enfant — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- claude_stream_json::tests::spec168_stdout_excessif_est_borne_et_arrete_sans_attendre_la_fin_naturelle — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- daemon::native_delegation::tests::spec168_capacite12_secours_natif_deux_connexions_env_distincts_et_effort_impose_refuse — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_catalogue_12_128_meme_uid_sans_agent_gardes_et_zero_effet — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_charge_entiere_objectif_contraintes_et_tache_sans_handoff — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_cinq_axes_effort_impose_et_defaut_natif_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_crash_pending_et_started_reprennent_le_repli_fige_sans_nouvelle_consultation — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_modele_unique_et_deux_niveaux_off_ne_consultent_pas_et_rejouent_avant_politique — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_niveaux_inaccessibles_zero_essai_et_effort_role_non_qualifie_explicite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_rollback_admission_sql_garde_chosen_puis_reprise_sans_decideur — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_service_post_reel_concurrence_enveloppe_retour_tardif_reprise_et_reouverture — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- daemon::native_delegation::tests::spec168_union_4096_reservations_et_taches_borne_le_puits_natif_sans_effet — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-native.json
- decision_service::tests::spec168_effort_valide_ou_defaut_atteste_sans_niveau_invente — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- decision_service::tests::spec168_laya_default_natif_null_et_transport_legacy163_restent_distincts — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_http_refuse_choix_effort_et_metadonnees_invalides — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_post_choisit_sur_l_ensemble_entier_et_garde_ses_faits — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_refuse_capacite_et_deadline_avant_tout_reseau — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_reutilise_sante_cachee_par_identifiant_service — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_sante_negative_et_cooldown_interdisent_post_et_gardent_motif — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_laya_sante_post_partagent_deadline_et_permis_reseau_jusqu_au_drain — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-decision.json
- decision_service::tests::spec168_permis_reste_detenu_apres_timeout_demandeur_jusqu_au_drain_reel — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-decision168-r1.json
- delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage.json
- delegation::tests::spec168_cas_niveau_epoch_revision_et_choix_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_migration_faute_copie_rollback_conserve_legacy — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_migre_163_sans_alterer_les_blobs_et_borne_v2 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_refuse_schema_index_fk_sortante_et_preserve_la_base — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_reouverture_refuse_trigger_et_fk_entrante — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation::tests::spec168_v2_persiste_decision_fermee_et_admission_atomique — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-store168-r1.json
- delegation_mcp::tests::spec168_mcp_auto_delegate_negocie_v2_et_preserve_le_schema_ferme — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json
- delegation_mcp::tests::spec168_wire163_byte_compatible_et_nouveau_client_refuse_ancien_maitre — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-mcp168-r1.json
- host_resources::tests::spec168_cpu_initial_delta_inconnu_apres_erreur_et_reset_epoch — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json
- host_resources::tests::spec168_linux_guest_ram_load_et_bornes_native — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-resources168-r1.json
- protocol::lineage_protocol::tests::spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-transport.json
- protocol::routing168_protocol_tests::routing168_auto_v2_preserve_le_defaut_general — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json
- protocol::routing168_protocol_tests::routing168_ids_et_probe_v2_sont_fermes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-routing168-r1.json
- protocol::tests::spec168_prepare_v2_contrat_ferme_et_prepare_v1_inchange — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-transport-spec168-r1.json
- provider_limits::tests::spec168_quota_connexion_instance_definition_epoque_ttl_et_credits_100 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-quota168-r1.json
- registry::tests::routing168_129_meme_desactive_refuse_sans_catalogue_partiel — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_axes_independants_fixes_et_contradiction_restent_des_filtres — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_capacite_compilation_est_declarative_et_bornee — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-telemetry168-r1.json
- registry::tests::routing168_catalogue_12_preserve_connexions_efforts_et_defaut_natif — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- registry::tests::routing168_decideurs_inaccessibles_sont_valides_et_claude_distant_refuse — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-registry168-r1.json
- remote_worker_162::spec168_probe_v2_128_ids_reste_distinct_du_wire163 — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker-wire168-r1.json
- remote_worker_162::worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker163-compat-r2.json
- remote_worker_162::worker168_probe128_connexion_revoquee_prepare_et_base_epinglee — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json
- remote_worker_162::worker168_reconnexion_epoch_et_capacite163_restent_distinctes — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker168-r2.json
- remote_worker_162::worker168_shared_hors_git_general_sha_et_erreur_ambigue — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-worker-horsgit168-r4.json
- spec168_catalogue_cli_ancien_maitre_refuse_avant_toute_lecture — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_catalogue_cli_humain_negocie_12_128_et_filtre_machine_sans_identite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_catalogue_daemon_reel_12_128_sans_agent_t3_task_ni_modele — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-e2e-catalogue168-r1.json
- spec168_delegate_workload_et_axes_refusent_les_valeurs_invalides_avant_connexion — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-cli168-r1.json
- spec168_lineage_cli_v4_conserve_version_et_refus_sans_downgrade — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-lineage-cli.json
- workspace_git::tests::native154_git_creation_depuis_sous_dossier_et_worktree_emboite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_delai_termine_son_enfant — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_environnement_et_options_effectives — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_formes_interdites_ne_lancent_rien — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_hook_post_checkout_et_gpg_ne_sont_jamais_lances — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_noms_canoniques_sans_option — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_rapport_ignore_gitfile_enfant_et_parent_emboite — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_rapport_zero_deux_soixante_commits_et_tete_detachee — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_sorties_cumulees_bornees — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json
- workspace_git::tests::native154_git_version_ancienne_refusee_a_l_avant_vol — PASS ; /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-workspace-git.json

### Statut documentaire avant P2

40/54tâches cochées. Les14tâches T036–T049 restent ouvertes.
T025 conserve son rattachement aux observations opérateur ; la précédente matrice qui la disait ouverte est historique.
T033/T034/T035/T050–T054 sont cochées sur les preuves précises ci-dessus, sans équivalence vague.
P1 demeure un verdict historique NONCONVERGED ; le futur verdict P2 n’est pas annoncé par ce reçu.
Nouveau GEL documentaire à la publication. Aucune écriture pendantP2 hors appendTasks explicitement ordonné par ROOT.


## Statique final P1 — hors compteur de tests

Reçu : /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-static.json
SHA-256 7214efb90c9288b79c78fd81808327c68ebbcf3bd74ff3a894045edd4d0de3c3.
Période :2026-10-15T09:58:48.621203Z à09:58:50.754329Z.
Rust24fichiers avant/après inchangés. Aucun Cargo/test/build ni modification de format.
- rustfmt --check --config skip_children=true : RC0,2,081s, stdout/stderr vides.
- git diff --check : RC0,0,046s, stdout/stderr vides.
- Nouveau host_resources.rs, contrôle no-index --check : RC1 normal pour une différence avec /dev/null,0,002s, sorties vides et aucun diagnostic d’espacement.
- Aucun parseJSON exécuté dans ce statique. Aucun succès de format global déduit.
Commandes exactes déjà exécutées, cwd commun /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte :

```sh
/home/moi/.cargo/bin/rustfmt --edition 2024 --check --config skip_children=true crates/bridget-daemon/src/cli.rs crates/bridget-daemon/src/cli_lineage.rs crates/bridget-daemon/src/communication/client.rs crates/bridget-daemon/src/daemon.rs crates/bridget-daemon/src/daemon/native_delegation.rs crates/bridget-daemon/src/decision_service.rs crates/bridget-daemon/src/delegation.rs crates/bridget-daemon/src/delegation_lineage.rs crates/bridget-daemon/src/delegation_lineage_tests.rs crates/bridget-daemon/src/delegation_mcp.rs crates/bridget-daemon/src/host_resources.rs crates/bridget-daemon/src/lib.rs crates/bridget-daemon/src/lifecycle.rs crates/bridget-daemon/src/provider_limits.rs crates/bridget-daemon/src/registry.rs crates/bridget-daemon/src/remote_worker.rs crates/bridget-daemon/tests/cli_arguments_integration_test.rs crates/bridget-daemon/tests/fixtures/remote_worker_162.rs crates/bridget-daemon/tests/lineage_cli_test.rs crates/bridget-daemon/tests/native_delegation_e2e.rs crates/bridget-transport/src/claude_stream_json.rs crates/bridget-transport/src/lib.rs crates/bridget-transport/src/lineage_protocol.rs crates/bridget-transport/src/protocol.rs
```

```sh
git diff --check
```

```sh
git diff --no-index --check /dev/null crates/bridget-daemon/src/host_resources.rs
```

Aucun test ajouté au compteur70 ; Tasks reste40/54. Consolidation close, GEL documentaire P2 effectif.


## Complément hors passe après corrections P2 — gel avant P3

État courant :42/56tâches cochées ; T036–T049 ouvertes, T037 non cochée.
ROOT a exécuté six filtres exacts affectés, chacun RC0/1PASS, sans échec ni ignoré.
Namespace : /tmp/b168p2-h4hznyr5. Manifestes55entrées avant/après identiques, source_unchanged=true/all_selections_ok=true :
/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-hashes.json.
Les six JSON portent les mêmes source_hashes. Les builds --no-run sont des compilations, pas des tests.

Recomptage depuis tous les JSON/logs disponibles :70noms uniques à dernière issuePASS, inchangé.
Les six PASS sont des replays de noms existants. Brut historique conservé :98PASS et9FAIL,
sans ajouter un doublon au nombre de tests. Aucun nouveau nom T055/T056 n’a été créé.

| Sélection exacte | Nom exécuté | PASS | RC | Durée s |
| --- | --- | ---: | ---: | ---: |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-score.json | daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation | 1 | 0 | 0.063 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-payload.json | daemon::native_delegation::tests::spec168_charge_entiere_objectif_contraintes_et_tache_sans_handoff | 1 | 0 | 0.094 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-service.json | daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete | 1 | 0 | 0.323 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-connections.json | daemon::native_delegation::tests::spec168_capacite12_secours_natif_deux_connexions_env_distincts_et_effort_impose_refuse | 1 | 0 | 0.168 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage.json | delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null | 1 | 0 | 0.025 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage-transport.json | protocol::lineage_protocol::tests::spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3 | 1 | 0 | 0.003 |

### Concordance des assertions et clôtures

- T055 : l’oracle payload lit réellement ProviderLimits pour trois connexions/instances du même type.
  main porte quota100% et credits=usable ; other=unusable ; unknown=unknown. Les états sont présents
  séparément dans charge/snapshot/facts. Absence, contrôle de dépense atteint, fait périmé,
  digest/époque invalide restent inconnus ou inutilisables selon le fait attesté, sans propagation entre connexions.
- Les oracles service et connections constatent credits=usable dans le POST HTTP contrôlé et dans
  la charge du secours isolé, puis la projection choisie. Un changement ultérieur de l’observation
  ne réécrit ni le reçu ni sa charge historique ; aucun solde opaque n’est transmis.
- Lineage vérifie usable/unusable/unknown pour la seule connexion élue avec quota100% ; les crédits opposés
  d’une autre connexion ne comblent jamais ses inconnus. Le DTO fermé refuse un objet balance opaque.
- T056 : l’oracle score/fraîcheur constate CPU/RAM/charge null dans le snapshot, unknown dans la charge
  et absence des anciens nombres périmés. Date/âge/fresh=false restent conservés ; la capacité compilation
  déclarée reste disponible. La reconstruction depuis snapshot reste byte-identique.
- L’oracle transport Lineage refuse compilation_slots=11 à la désérialisation et à la sérialisation.
  Les valeurs0..=10 restent le contrat. La projection conserve les mesures inconnues et l’âge historique,
  sans relire une horloge fraîche ni perdre compilation_capacity déclarée.
- T055/T056 sont cochées sur ces assertions lues dans les sources exactes exécutées et les six reçusPASS.
  La revue P3 reste à venir ; P2 NONCONVERGED conserve son statut historique.
- Format/diffcheck ciblés propriétaires C/E rapportés RC0. Aucun nouveau statique global, parseJSON,
  FULL, appel fournisseur réel, activation ou release n’est revendiqué.

### Commandes ROOT exactes


/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-score.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-payload.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_charge_entiere_objectif_contraintes_et_tache_sans_handoff --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-service.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-connections.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_capacite12_secours_natif_deux_connexions_env_distincts_et_effort_impose_refuse --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null --exact --test-threads=2
```

/home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage-transport.json

```sh
/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_transport-5726269dfab634e9 protocol::lineage_protocol::tests::spec168_lineage_v4_est_ferme_et_ne_retouche_pas_v3 --exact --test-threads=2
```

Build transport : RC0, 11.560s, --no-run, non compté comme test.

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-transport --lib --no-run --message-format=json
```

Build daemon : RC0, 36.163s, --no-run, non compté comme test.

```sh
nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --no-run --message-format=json
```

### Sources et artefacts

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs : c803ea677c8211014a12a1cb3f8757ea3ad719d269a4821d1f6f9a262f32abf6.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs : de16154768fbdb15ebf523ba22b9094fa137e7c41cfc8fb61ad8c747258b8124.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage_tests.rs : b758283bee9516e41807110e1ee3179b8956cf8889e6403a655d4c9ba021526d.

| Artefact absolu | SHA-256 |
| --- | --- |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-daemon.json | a96440831e62f40a84946abe8819e634c6abd6407f35eeaf9c8e7ede215f96ec |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-daemon.stderr.log | 0aaf9321741e17b9ace03fe71891f0ddecad0d959526b11cca54d4298b4c39d6 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-daemon.stdout.jsonl | aee3a10bb283edcd7fb0e004fb93eb5c702d77ddb4294a866429c3d349bc8c01 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-transport.json | 254a1c865eeb27fca2ace9a50141ce8903372295ef70793bf3a02e43557ab042 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-transport.stderr.log | a7e573a35e42badae05afe9ca7c42a1c1559158147537153bde333337a5e2dac |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-build-transport.stdout.jsonl | 9aab5ab1e343a49597ac03cb1542f5b4c11dff057109ed35ed2d18ec853efb54 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-connections.json | 7d91eac19e052780ec2c827648a026ca5c4a27373330e8f3c38a98416672f466 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-connections.log | 1f37ff7d04a18d4cd7c482651b1a3a4da32dc7b3c441d52c2f14f7e270ee315f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-hashes.json | 1a5020cb1db0f11d68fe1b8cb39fe3d834afc1ec60881973cbec13dc7ec3fd75 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage-transport.json | 97f8a108d6469a5b74ecba10a964373a36d4703778b0c52521eeb6bac21852d4 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage-transport.log | bd492a5ad051af3ddaafb54e553ee41abe31dc34385e5d2bfb64bbbe3643641c |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage.json | 90f472eadab2f8dda171c51e8e58b1be1682318793a730f4694961090b804e2f |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-lineage.log | 83489f2d20301864285b33b2300a7642e5d1b37b16492e381eaf055515906271 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-payload.json | 365f851ec412fec2b1afdadfba709fd6d63eb18832dea036af329838a806479b |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-payload.log | 490444a13ccc64f717fcd353232d781539c9ab7362785dc00c2002a9685f4b70 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-score.json | 611544682e0ae6c187bbe189f5cfb640a964b4d9efc59a8cb685582bae776e33 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-score.log | e8959dcbd007637d9d9bb9eaa5f2fad8dc6e267cff405ecca45c83382fde42e7 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-service.json | 556878058bd4a15225e1b7c71c7c8ff34fabeb6fafc53f9d05d6c5ab3e3e7002 |
| /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-service.log | 49f142436ae3fa46e9346057b8316e0e0dacf318f4a12d332bd42709a1f5b257 |

Les preuves privées restent intactes. Aucun code/test/Cargo/service/Git modifié ou exécuté par cette consolidation. GEL P3 effectif après publication.

## Complément T037 — quatre contrôles affectés après Sonnet R1

Exécutions ROOT sur Cartae ; lecture documentaire des reçus et assertions, aucun test relancé ici.
Namespace : /tmp/b168r1-v546hg07.
Compilation daemon bibliothèque --no-run : RC 0, 29,401 s ; aucun test compté à cette étape.
Les quatre sélections suivantes emploient --exact et --test-threads=2 :

| Sélection | Nom exact | RC | PASS / FAIL | Durée s |
|---|---|---:|---:|---:|
| registry | registry::tests::spec168_telemetrie_cpu_charge_seuils_1_100_fermes | 0 | 1 / 0 | 0.014 |
| score | daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation | 0 | 1 / 0 | 0.061 |
| service | daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete | 0 | 1 / 0 | 0.314 |
| busy | daemon::native_delegation::tests::spec168_niveaux_inaccessibles_zero_essai_et_effort_role_non_qualifie_explicite | 0 | 1 / 0 | 0.143 |

Assertions utiles lues sur la source finale :
- Registre : sept poids fermés et bornés, valeurs partielles, défauts omis, roundtrip historique
  byte-identique et digest préservé ; poids non défaut sérialisés et digest différent.
- Score : TTL reçu au maître, réception future/expirée refusée, mesures source future à réception
  inconnues même après délai ; arrondis et coefficients de la coordinatrice uniformes et figés.
- Service valide : verdict distant corrélé conservé malgré horloge worker en avance ; ses mesures
  futures deviennent inconnues. Le contrôle ne devient pas une recette Laya ou fournisseur réel.
- Décideur occupé : repli déterministe et note durable decision_capacity_occupied, permis physique
  non libéré par un autre demandeur, rejeu identique, aucun essai HTTP ni lancement enfant.
- L’aide CLI est relue directement : moteur, connexion, workload et base attendue présents.
  Cette lecture est distincte des quatre tests ; aucun test CLI supplémentaire n’est compté.

Commandes exactes conservées par les reçus :

- nice -n 10 /home/moi/.cargo/bin/cargo test --locked --offline -j 2 -p bridget-daemon --lib --no-run --message-format=json
- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e registry::tests::spec168_telemetrie_cpu_charge_seuils_1_100_fermes --exact --test-threads=2
- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation --exact --test-threads=2
- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete --exact --test-threads=2
- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/bridget_daemon-b50bfd4b4880ca3e daemon::native_delegation::tests::spec168_niveaux_inaccessibles_zero_essai_et_effort_role_non_qualifie_explicite --exact --test-threads=2

Reçus JSON et empreintes (les quatre sélections ont aussi leur .log homonyme) :

- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-registry.json ; SHA-256 5831c453a884e546bcc6dfc7f3c61249a7c146d4f153d19ff3cb83e64e19a92d.
  Journal : /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-registry.log ; SHA-256 d134ce91eee9ef444712f06556304facd322d2b5539d8f71daa4fc65edd24267.
- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-score.json ; SHA-256 b17194ccaa45f2423bcf5ba737f1586941b8bcc05bb55d9d57b6e9dc47265058.
  Journal : /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-score.log ; SHA-256 e8959dcbd007637d9d9bb9eaa5f2fad8dc6e267cff405ecca45c83382fde42e7.
- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-service.json ; SHA-256 a7be2eebf8c53ed4e359bbc5e4ab51c7eca0e0e49840db3b0af9fc791e4a743f.
  Journal : /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-service.log ; SHA-256 5f4b8c7e35e1474b85f82fba38babbec450690e3685683050b661f152fb14e17.
- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-busy.json ; SHA-256 c4b2313e713f90ed639b6c856e09a57d29d4612f4696454dce495d0e73b18063.
  Journal : /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-busy.log ; SHA-256 74fb92feb2a489fbc4c7019180f101f0d4bd55eb4c1e481f7cf42a56b50a2293.
- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-build-daemon.json ; SHA-256 544cc4a8c3583b321b52244e22022693e830b0ff9d637eb6d98f692ed44048c6.
- /home/moi/.local/state/bridget-delivery168/tests/t037-sonnet-r1-hashes.json ; SHA-256 b955350125d9ce1a5ce4c7149683875dc86012fdbf8e6ae701355b74a9bc6ae8.

Le manifeste hashes contient 56 entrées avant/après identiques et all_selections_ok=true.
Cette stabilité couvre la campagne avant cette mise à jour documentaire, pas un gel rétroactif des documents.
Sources finales :
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; SHA-256 dabe6fc79b6ae49a23f8556b834cc09a5f3f6e4524ad24e3c2fe3129169ac948.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs ; SHA-256 f0fdf419c21623cfc423d02a3fb92311405db5b1ade93dcbfa6ce759408ca817.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs ; SHA-256 541f9b337efe9a99b388a553544c87e8aa6a17f9db8783af98742965148df253.

ROOT atteste aucun Cargo actif après campagne. Aucune FULL, recette réelle 168, activation,
release, audit achevé ou GREEN global n’est déduit. Les neuf échecs historiques restent conservés.


## T038 — RED d’audit et échecs de précondition conservés

ROOT a exécuté ces contrôles ; le rédacteur lit leurs JSON/journaux sans relance.
Les deux premiers essais s’arrêtent sur auto_admission_invalid puis routing_machine_policy_unknown :
ils ne prouvent pas le défaut Worktree. Les diagnostics de fixture sont conservés ; aucune baseline
n’est effacée. Le RED final atteint remote_prepare:PlacementMismatch avec S disponible et HEADworker=T≠S.

| Artefact | RC | PASS / FAIL | Durée s | Qualification |
|---|---:|---:|---:|---|
| t038-audit-r1-red-remote-worktree.json | 101 | 0 / 1 | 0.416 | Précondition fixture ; non RED produit |
| t038-audit-r1-red-replay-remote-worktree.json | 101 | 0 / 1 | 0.414 | Précondition politique fixture ; non RED produit |
| t038-audit-r1-red-final-remote-worktree.json | 101 | 0 / 1 | 0.566 | RED Worktree atteint : remote_prepare:PlacementMismatch |
| t038-audit-r1-nplus1-red-remote-worktree.json | 101 | 0 / 1 | 6.110 | RED N+1 atteint : 521 appels Git, 2 tuples, 128 options |


Le compteur 521 vient des appels réels capturés par le banc ; il n’est pas un débit ni un benchmark
de production. Le seuil de l’oracle est 40 pour 2 tuples. Les décomptes statiques r0 par variante Git
restent une analyse distincte, pas une reconstruction de cette trace.

Corrections d’interface arbitrées, encore à valider :
- PrepareV2 Worktree inspect_only=true garde workspace/base_ref/SHA ; token inspect-only-v1,
  aucun état Prepared/worktree/slot, Start refuse ; second Prepare normal après Task durable.
- Faits Git mémoïsés par tuple distinct dans le lot ≤128, refus et succès compris ; droits,
  définition, connexion, modèle et effort individuels ; Prepare/Start revalidation fraîche.
- Instant absolu depuis admission pour la chaîne et le processus ; aucune remise à zéro au spawn.
- Seconde sonde effort : définition brute, époque et mapping comparés à la première observation fraîche.

Commandes et preuves conservées :

- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele --exact --test-threads=2
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-remote-worktree.json ; SHA-256 f5577a6e982d8f81ca03ec11a32c1938ba57896eed4a38c112bccefeeff4a419.
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-remote-worktree.log ; SHA-256 1a1d9186c31c6fe3e9b8060ebadeb19f1ba6949b36db298d38b3916e8b8c5b0e.

- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele --exact --test-threads=2
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-replay-remote-worktree.json ; SHA-256 0657cdd1e1e1b3d1dcbebcf1e0e88e125ed680094b7150e2f7618194580f1cad.
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-replay-remote-worktree.log ; SHA-256 34bf000c9adb73640c618f74ab5e5cea2e9a31dc893b1ae56b4abf323fe38403.

- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele --exact --test-threads=2
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-final-remote-worktree.json ; SHA-256 dba784dc10557a77feca254b50cc15f2330e5cc1ef2b46ff90d8e1feacf6f402.
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-red-final-remote-worktree.log ; SHA-256 7a668e23d618186f3822d948adb2095332c1eea9c5a16fe44450b17099bea93c.

- /home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target/debug/deps/native_delegation_e2e-b1a6db4100b356ca remote_worker_162::worker168_probe128_connexion_revoquee_prepare_et_base_epinglee --exact --test-threads=2
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-nplus1-red-remote-worktree.json ; SHA-256 3d65b68c69ed60414b9ea92314070728765890dc54805aa29c34e07f730806a7.
  /home/moi/.local/state/bridget-delivery168/tests/t038-audit-r1-nplus1-red-remote-worktree.log ; SHA-256 32dedf701330f013ade6be1580eec34b6464283d43d807c01b583b458c804001.


Diagnostics initiaux distincts :
- /home/moi/.local/state/bridget-delivery168/tests/t038-red-fixture-diagnostic.json ; SHA-256 7f4bbca5e4fa292e7459dd061afc983e123195f4a1788dc791ca7bff0ece59c4.
- /home/moi/.local/state/bridget-delivery168/tests/t038-red-fixture-policy-diagnostic.json ; SHA-256 f307e13b717d45bfe7a8a984b7ac61bacb220fb6ccd03b86f8cf2bba360a99c2.


Les journaux gardent les deux premiers FAIL de fixture, les deux RED atteints et leurs empreintes.
Ces quatre FAIL sont un lot d’audit nouveau, distinct des 9 échecs historiques déjà conservés.
Aucun test GREEN de la correction n’est ajouté ; ROOT publiera les résultats d’exécution suivants.


## T036 — suite complète unique et reprises ciblées finales (15 octobre 2026)

La suite complète unique termine **RC101 en 775,777964 s : 2 820 PASS, 32 FAIL, 71 ignorés**. Les 97 cibles Cargo produisent 105 sommaires ; huit sommaires de harnais enfants sont exclus. Leurs huit PASS ne s’ajoutent pas aux 2 820. Les sorties ne sont pas tronquées et le parsing est complet. Les sources restent identiques pendant la FULL : `source_consistent=true`, SHA-256 `d2ae99201d3259718246b039cddf08a6c689538bf5c0088496d4be4c003f698e`. Les sommaires ne sont pas attribués artificiellement aux cibles par leur ordre de sortie.

### Chronologie conservée

1. **Migrations 159 : 9 PASS.** Les neuf filtres exacts sont repris sur le même binaire de bibliothèque que la FULL, sans nouvelle compilation. Seule la sauvegarde réelle requise mais absente est fournie. Binaire SHA-256 : `d4b900dd65f760d6c5b71e10c88d3c15ffbab4f4471909eb3cdc14c4dfbf36d6`.
2. **Reprise r1 : 12 PASS, 3 FAIL, 8 différés.** Quinze oracles sont réellement exécutés sur les 23 autres échecs. Les deux builds ciblés sont RC0, en 34,261 s et 7,685 s. `source_consistent=true`, manifeste avant/après `f450f3e7ac327d56a7dd64eaa35e374d5ee25ee73fb1c94a9e72220af40d4940`. Les trois FAIL et huit cas différés restent dans ce reçu historique.
3. **Reprise r2 : 8 PASS sur exactement 8 tests, zéro FAIL.** Compilation RC0 en 32,421 s avec la feature existante `test-support` et umask `0077`. `source_consistent=true`. Aucun nouveau patch produit n’est requis pour ces préconditions. Les chemins et SHA des trois binaires figurent dans le reçu r2.

Bilan des **32 échecs initiaux : 29 oracles distincts PASS après reprise ; trois annulations 159 non validables sur Linux**. Ces trois échecs FULL restent conservés ; aucun `ignored` n’est ajouté. La suite complète initiale ne devient pas verte. Les replays ne constituent pas une seconde FULL.

Les premières erreurs établies concernent les fixtures ou leurs préconditions : cwd inexistant ; version de protocole devenue supportée ; catalogue de 32 outils encore attendu à 31 ; faux maîtres encore en auto_delegate_v1 ; réouverture d’une autre DB ; PATH sans Git qualifié ; chemin lsof propre à macOS ; empreinte non canonique de /bin/sh ; précondition globale « Docker absent ». Les dix E2E 159 exigeaient aussi le binaire daemon de test et les deux chemins de rapports de latence. Les neuf fichiers modifiés après FULL ne changent que les tests et leur diagnostic conditionnel ; le code produit reste inchangé. Les contre-lectures ci-dessous approuvent ces deltas statiquement dans leur périmètre, sans transformer cette lecture en exécution.

Le lot r1 a ensuite apporté des données nouvelles : le daemon rejetait une DB créée en mode 0640 sous umask 0007 ; l’autre wrapper employait le binaire libtest sans la feature `test-support`, avec stderr `Unrecognized option: 'instance-id'`. Le lot r2 conserve les gardes de confidentialité et emploie umask 0077 et cette feature existante. Il ne change ni les droits de production ni le mécanisme produit.

Le test `t3code_identity::spec101_identity::spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` passe au premier replay instrumenté. Le diagnostic conditionnel ajouté n’est pas atteint. La cause initiale de `process_arguments=None` demeure inconnue ; ce PASS ne prouve pas sa résolution causale.

### Trois scénarios non validables sur Cartae Linux

- `edit159_e2e_cancel_arret_apres_commit`.
- `edit159_e2e_cancel_arret_avant_transaction`.
- `edit159_e2e_cancel_echec_interne_transaction`.

Leur `watch_marker` repose sur kqueue Darwin/BSD ; le stub Linux panique explicitement. Aucun observateur Linux n’est inventé, aucune assertion n’est retirée. Diagnostic : /home/moi/.local/state/bridget-delivery168/full-triage/e2e159-wrapper-r1-diagnostic.md, SHA-256 `18bdecf94cea3e16a07fbce7ea35eb97bf8efd158c35c803278a3a7313d1b13e`. Ces frontières restent à exécuter sur une plateforme prise en charge ; elles ne sont ni PASS ni nouveaux ignorés.

### Reçus, contre-lectures et intégrité

- /home/moi/.local/state/bridget-delivery168/tests/t036-full168-1ru8z5rf/result.json — SHA-256 `149bc0a5470df503d706b30cc50ee11bc3aa1fe302785fdc42c091a64dd1eaa0`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/result.json — SHA-256 `17378e035ca07e49080373eaecde8f1e68fddb43eb9f1c9ca66d65ca0c480586`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/result.json — SHA-256 `78a6de7c1f9e0291f901112b1f5b6faedef8fa65e96827ba4cb67627926e1d3a`.
- /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/result.json — SHA-256 `2b359ce75ecc546c76dba1e4a3122c467566854df1739c160924d42fea3efe65`.
- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-post-full168.md — SHA-256 `c8240ff0386b00d27214c776028c8237b300f79dc1173b9983336bd3369737bd`.
- /home/moi/.local/state/bridget-delivery168/full-triage/review-fixtures-delta-core-t3.md — SHA-256 `42ea238ebad7c0a9e89c431bdba77f12ba3ce70129a34e3bf334d373a34068e2`.

Chaque entrée ci-dessous correspond à un filtre exécuté avec `--exact --test-threads=2`. Les arguments complets, binaires, durées et SHA des logs sont dans les reçus. Les chemins absolus et empreintes permettent de relire les preuves sans réexécuter le test.

#### Migrations au binaire FULL original

- `idempotency::tests::held159_migration_fantomes_ecartes_avec_warn` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/00.log ; SHA-256 `bb7cfa3f46ced7fe257216b2e497122da6960deb50136d6122de356f0c3394f4`.
- `idempotency::tests::held159_migration_liens_execution_conserves` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/01.log ; SHA-256 `4660981edc6f0edc4bc51010463c3a339ededa3435b5af4489c36ba9509e1a51`.
- `idempotency::tests::held159_migration_ordre_preserve` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/02.log ; SHA-256 `8142e62b0885713639d96a5efa296d3f9b1de59542c0d51620de7e9e4e15b5e8`.
- `idempotency::tests::held159_migration_phases_held_et_cancelled_admises` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/03.log ; SHA-256 `81f867722d75a379543f4263749a774233dd5c3dd6d3b7ddab3eb7ff40b8defb`.
- `idempotency::tests::held159_migration_recipient_agent_rempli` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/04.log ; SHA-256 `5895ab1a59350def0e213a94c6a991713ad5d98dd15dec86d2cffc4503335e32`.
- `idempotency::tests::held159_migration_rejeu_sans_effet` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/05.log ; SHA-256 `5fce0b274c6e94a79b54bbc0d16ad99cfe109c30cabb58380399bd93fae70e0b`.
- `idempotency::tests::held159_migration_rollback_si_echec` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/06.log ; SHA-256 `b01c83b409dfe6e320f15ac176914549d695b5b4fe707b4ae5a37aa8f938079b`.
- `idempotency::tests::held159_migration_sans_perte` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/07.log ; SHA-256 `b4c23afc9ba3cd107cf9ba72f207ba066a7165c773f56378d4ba3ac93dfcbfb5`.
- `idempotency::tests::held159_migration_turn_gate_defaut_zero` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-migration159-original-lib/08.log ; SHA-256 `3c4f6b85765dd13b3e984447e52ee6c21438bbadd7b46b4233a8960f5ce3ca0c`.

#### PASS du lot r1

- `t3code_identity::spec101_identity::spec147_native_t3_refresh_publishes_and_revokes_real_os_processes` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-00.log ; SHA-256 `fe4a91071ea652b277c0c32b22f727ec676f648c400be3be1b98a0007c28d646`.
- `projet_refuse_et_session_hote_spawn_send_attach_stop_sans_docker` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-11.log ; SHA-256 `22c73226c3c908eca9f98bf02a8cf52643c95da4559660d10fb7ac4af87178a2`.
- `spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-12.log ; SHA-256 `0fd23a5f2e6400a9b726f28e2a119b1006961313e65e1cfad9718ccd693d70bf`.
- `daemon_started_at_256_raises_soft_to_4096_and_holds_over_256_descriptors` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-13.log ; SHA-256 `2f6ce457522c1a3366f8c33e3d581d55d5bcd893be716a80e67b612d801c1102`.
- `daemon_with_hard_512_is_clamped_to_hard_and_warns` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-14.log ; SHA-256 `b1300616815ca8bcb4295eddaf95adee7560650a1ca9a19f0f8b26f6f142b283`.
- `cli::hook_tests::spec167_delegate_conserve_les_choix_explicites_sans_defaults_caches` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-15.log ; SHA-256 `c323a849dad6da930b74be5f96a881db4f67c83eb30c5c4fe6e6b803735d56b4`.
- `cli::hook_tests::spec167_delegate_reduit_le_parcours_nominal_a_la_mission` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-16.log ; SHA-256 `a37d95429c047b2f9f9c8e9990063b7cc52f3060d9222b6a731354ed32e10974`.
- `daemon::native_lineage::native_lineage_tests::native149_validate_ferme_les_formes_ouvertes` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-17.log ; SHA-256 `ab52accc5743d6e0c3a4390230976d39e9322733d9c714e9cf2cf89e4fddf06f`.
- `daemon::presence_tests::guidance156_rate_survit_au_redemarrage` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-18.log ; SHA-256 `0a21e98956f53664a828b4f9407543dfd4815be064b844b0f5e2b9d20f6ba847`.
- `mcp::tests::matrice_fr009_couvre_les_quinze_cas` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-20.log ; SHA-256 `b136b2d78c89b50c6cb239de7d5fe3643c83a4803d89a755bd2da8c82e65561e`.
- `mcp::tests::mcp158_plafond_parent_garde_tous_les_outils` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-21.log ; SHA-256 `db0a34de4ea74a5b0ddf85946205468e01931c64c45cdfeb74207d01cef082f7`.
- `native149_recheck_gele_resout_le_launcher_par_path_et_detecte_le_changement` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-22.log ; SHA-256 `db88f59c940693af710d5adb2ad934c1335c67f8cf3c6f6432ec73c9b60b2796`.

#### PASS du lot r2

- `edit159_e2e_recette_complete_reglage_ouvert` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-00.log ; SHA-256 `72d38a8f585f740deceeb4cd1a25a84d678b8ed508fd0ffc03a9a6b8fc7f9c41`.
- `edit159_e2e_recette_reglage_ferme_not_held` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-01.log ; SHA-256 `f95e6dc29e76b4baaeefa4062f543b6d25a192a6206bc39ab647ccff6bed0764`.
- `held159_latency_baseline_codex_simule` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-02.log ; SHA-256 `29d575e1985618edf60573205bbdecc38e97a73f083dc5cb7b11b49d5c656647`.
- `held159_latency_retenue_codex_simule` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-03.log ; SHA-256 `2470138a998452b88638fabff238f9ff0aa8506d0ecd09df2173127943bf60a9`.
- `held159_porteur_arrete_avec_message_en_cours_preserve_ordre` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-04.log ; SHA-256 `795eeafbc113a6cd11fd524eec3693df69a7e603ecb028650d43c20bc7b4fd46`.
- `held159_daemon_arrete_apres_retention_conserve_la_remise` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-05.log ; SHA-256 `1157a1fd03be4a1bbeabcc37401cb91245b0d12d4c15e6d90c8a6a7557039b1a`.
- `held159_porteur_arrete_sans_message_en_cours` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-06.log ; SHA-256 `524d2a0cb71862462ffdf6b7d49a60805f88b224360ef51e2d15b64afd1edd68`.
- `daemon::presence_tests::sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-required-features-r2/test-07.log ; SHA-256 `d817bfed1aac8b3de6ce308c9cf79937a4bd49d4108ddc597da2c33d2e53e9a2`.

Les trois logs FAIL du lot r1 sont conservés, même après les PASS r2 :

- `edit159_e2e_recette_complete_reglage_ouvert` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-01.log ; SHA-256 `4a69a733a0aea8bfc9709e307cb0ff10018e60f324a29955cfffe1827bad56db`.
- `held159_latency_baseline_codex_simule` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-02.log ; SHA-256 `3f931215375389f6a655a07c355dc7e0f92aca7e19098fe2048aea7e6ccf4b8e`.
- `daemon::presence_tests::sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique` : /home/moi/.local/state/bridget-delivery168/tests/t036-replay-post-fixtures-r1/test-19.log ; SHA-256 `2af7f9b42a1461c6a90738454dc11c3c08ce493c8364d07b98172df69193816e`.

Aucun test, scan ou build supplémentaire n’a été lancé pour rédiger ce reçu. Il n’établit ni recette réelle de modèle, ni activation, ni release. T036 n’est pas cochée par ce reçu ; ROOT conserve cette responsabilité après lecture.


## Complément T045B — délai service configuré, preuve contrôlée

Le 15 octobre 2026, ROOT a fait corriger le plafond effectif fixe de 2 s : le délai du service respecte maintenant sa valeur configurée de 1 à 30 s, dans le budget global absolu restant. Le défaut reste 2 s ; santé et POST ne reçoivent pas deux budgets indépendants.

Source /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs : SHA-256 avant correctif e0e57e569e36593bd3b94c0ba71d1c0a4b6a3f5735eed562c07d7e3dd2f10901 ; après correctif 5668719253fb44f96ccf28a878c952da559c0a3349f73adf0ebf56462c90da7c. Le hash est stable pendant le test.

Une seule sélection exacte est exécutée : daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete. Résultat : RC 0, 1 PASS / 0 FAIL, 6,15 s de test et 45,433 s pour la commande, compilation comprise. Ce nom existant n’ajoute pas un nouveau test distinct.

| Sous-cas contrôlé | Configuration et délais simulés | Issue observée |
|---|---|---|
| Service supérieur à 2 s | Service 4 500 ms, global 5 000 ms, POST 2 200 ms | decision_service |
| Défaut conservé | Service 2 000 ms, global 10 000 ms, POST 2 200 ms | deterministic ; decision_service_caller_deadline |
| Budget global partagé | Service et global 1 000 ms ; santé 600 ms puis POST 650 ms | deterministic ; decision_service_caller_deadline |

Commande exécutée par ROOT, sans rejeu lors de cette rédaction :

nice -n 10 /home/moi/.cargo/bin/cargo test --offline --locked -j 2 -p bridget-daemon --lib daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete -- --exact --test-threads=2

- Reçu : /home/moi/.local/state/bridget-delivery168/tests/t045-budget-service-config-r1.json ; SHA-256 738ac4d040cd13d59d5273821b229f323aa5da2cb406a0de991b0308f6f43149.
- Log : /home/moi/.local/state/bridget-delivery168/tests/t045-budget-service-config-r1.log ; SHA-256 429d6d6d979099e130d4ad4ed9a0610763a901a2dca99fcfd37dbc6a811f024f.
- Diff : /home/moi/.local/state/bridget-delivery168/tests/t045-budget-native-final.patch ; SHA-256 73987a8276b632dc1c14e4a8f9e3f7cbca7d9c2fda751f6fdc7088a1baca0c97.

La contre-lecture statique verify_git_handoff est APPROVE selon la passation ROOT. Aucun autre Cargo, FULL, scanner V15 ou passage P4 n’est déduit. La FULL historique et ses limites restent inchangées.

La première recette réelle source A a rendu un résultat distant sur host cartae au SHA 269de64e172f71f8856373fe8b09cb2d8642da52, mais decision.source=deterministic avec note caller_deadline. Elle ne valide donc pas la voie Laya. Après redémarrage privé, Task, canon, BLOB metadata et résultat restent stables : une Task, un reçu, une exécution. Le payload binaire de Task n’est pas byte-identique ; sa projection métier est stable. Le nombre réel d’appels HTTP n’a pas été observé. Ces faits sont transmis par ROOT, distincts du test contrôlé ci-dessus.

T045 reste ouverte, état 51/56. Nouveau commit source B, paquets, activation et trois recettes réelles restent nécessaires avant release ; aucun SHA B ni succès futur n’est présumé.
