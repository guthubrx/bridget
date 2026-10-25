# Tests SPEC-162 — registre de preuves

**Date :** 2026-10-13. Les commandes ont été exécutées par ROOT. Le rédacteur documentaire a relu les journaux et capturé les empreintes, sans lancer de test ou de build. Les codes de retour viennent de ROOT ; les compteurs et durées viennent des journaux. Aucun GREEN global.

## Résultats attestés

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-1).

Transport R1 échoue sur `protocol::tests::remote162_dispositions_fermees_sans_faux_succes_quota` : la variante unité d’un enum internally tagged acceptait `{"status":"accepted","ignored":true}`. Le correctif remplace cinq variantes unités par des variantes struct `{}`. R2 rejoue seulement cet oracle avec `--exact`. Les deux warnings de nommage wake158B restent visibles ; aucune correction hors périmètre déduite.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-2).

## Commandes réellement lancées

Les argv ci-dessous sont attestés par ROOT, et non déduits de la note de préparation /Users/moi/.cache/bridget-delivery162-20261013/validation-commands.md. Cette note indiquait elle-même que ses commandes n’étaient pas exécutées.

Transport R1, cwd : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante .

```bash
env CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 BRIDGET_BUILD_CACHE_GIB=40 BRIDGET_BUILD_CACHE_DAYS=7 nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-transport --lib remote162_ -- --test-threads=2
```

Transport R2, même cwd :

```bash
env CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 BRIDGET_BUILD_CACHE_GIB=40 BRIDGET_BUILD_CACHE_DAYS=7 nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-transport --lib protocol::tests::remote162_dispositions_fermees_sans_faux_succes_quota -- --exact --test-threads=2
```

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-3).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-5).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-6).

## Provenance des journaux

Empreintes relues le 2026-10-13T09:04:56.058083+00:00 .

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-7).

## Sources transport avant R2

La capture prétest conservée dans /Users/moi/.cache/bridget-delivery162-20261013/transport162-r2.sources contient :

| Source absolue | SHA-256 prétest R2 |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/protocol.rs | 084a01b339ebeecea671c0f461270615ee33ed806e2829f67bcf7f953f8aa6b0 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/lineage_protocol.rs | 704d26ef95ff54b4b78677f28bf35d55501412442533b1b42e1294a1bc37aec4 |

Aucune empreinte des sources avant R1 n’a été capturée. Les empreintes R2 ou actuelles ne sont pas attribuées rétroactivement à R1.

## Sources adaptateur de bureau capturées après exécution

Capture le 2026-10-13T09:04:56.058083+00:00 . Ce sont des empreintes après test, pas des empreintes prétest. Pour les quatre fichiers T035/T036, l’owner atteste leur absence de modification depuis le run ; ROOT a relu les deux fichiers de production. Pour les autres fichiers, cette capture n’affirme pas une relecture ROOT ni une identité prétest non fournie.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-8).

## Limites et progression courante

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-9).

## Compilation des fondations R1

Journal : /Users/moi/.cache/bridget-delivery162-20261013/daemon162-foundations-r1.log.
Commande dans le worktree Bridget, avec le même environnement Cargo et nice10 que les passages transport :

```bash
python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib remote162_ -- --test-threads=2
```

Les six erreurs sont E0063 dans les fixtures delegation_lineage_tests.rs et daemon/native_lineage_tests.rs, puis E0004 dans quatre matchs de daemon.rs. Le warning unused_mut dans native_delegation.rs reste distinct. Le processus s’est terminé avant toute exécution de test. Les types distants ne sont pas encore activables en production.

## Fondations daemon R2 et replay Lineage R3

ROOT a exécuté R2 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante avec l’environnement Cargo déjà attesté et le filtre 162_, au lieu de remote162_ :

```bash
env CARGO_TARGET_DIR=/Users/moi/Nextcloud/10.Scripts/64.bridget/target CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 BRIDGET_BUILD_CACHE_GIB=40 BRIDGET_BUILD_CACHE_DAYS=7 nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib 162_ -- --test-threads=2
```

Les deux échecs viennent du setup commun qui refuse une socket trop longue. Ils restent dans le journal R2. ROOT a rejoué le même exécutable, sans changement source et sans recompilation, avec TMPDIR court :

```bash
env TMPDIR=/tmp/b162-szysmv9d nice -n 10 /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/bridget_daemon-4f844ffb28f60eee daemon::native_lineage::native_lineage_tests::native162_ --test-threads=2
```

R3 : 2 PASS, 0 FAIL, 0 ignored, 0,12s, RC0. Le même chemin binaire et l’absence de modification/recompilation entre R2/R3 sont attestés par ROOT. Aucune empreinte prétest de ce binaire n’est inventée. L’identité des sources est celle du manifeste prétest R2, pas celle des fichiers modifiés depuis pour le runtime.

| Groupe dans R2 | Résultat | Interprétation |
|---|---:|---|
| Registre | 7 PASS | T003 validée après relecture ROOT. |
| Canon/persistance Task/Fleet | 4 PASS | T004 validée après self-review owner. |
| Permissions transhôtes | 10 PASS | T015/T016/T018 validées après relecture ROOT ; dix cas au total, pas dix par tâche. |
| CLI Lineage | 4 PASS | Unitaire uniquement ; intégration T033 encore ouverte. |
| Worker technique | 9 PASS | Historique pré-correctif de publication ; T005 ouverte. |
| Projection Lineage | 1 PASS | T034 runtime encore ouvert. |
| Setup Lineage maître | 2 FAIL puis replay 2 PASS | R2 conservé ; R3 même binaire et TMPDIR court. |

## Empreintes prétest R2 et nouveaux journaux

Le manifeste /Users/moi/.cache/bridget-delivery162-20261013/daemon162-foundations-r2.sources contient 219 empreintes de fichiers sources, capturées avant R2. SHA-256 : cd658443d32663bd3fca657efd45b9e64df98caa33ba080d363b7c04eef70337 . Ne pas remplacer ce manifeste par des empreintes actuelles.

Extrait pertinent de ce manifeste, pas de nouvelle mesure des sources :

| Source absolue | SHA-256 prétest R2 |
|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/cli_lineage.rs | 7c72539c4e2af8e80ee950ef4903796b55e9001b8c2be719bcd1d02a3c4ea686 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/delegation.rs | 6bf4b67ef0a401016b3f59ec1158c05adf9074a9893a28e44788d5594b15f452 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/desired_state.rs | 047c9442f80c9792b1704473a6993f5e4892a18bec19e9cc874ef3e97481bcd9 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/fleet.rs | 8c7d69e1f8b9ab18bce33a4ef69b0169728418f35cc294d582343b416ca2e367 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/lineage_client.rs | e2d3113b33217fa9a8ebb4ea847ae237915c8baed9ad1394d6d28ad194d3b89e |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/native_permissions.rs | 7830f015609f7de5d5bfe61c4fc917b7f08bf867d077ebf108675ba7ad654b2d |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/native_permissions149_tests.rs | c0a03e0ad8a7d06f84e0fdffbeb4100362cd622c6d7c70bf57c158b2c05a339d |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/registry.rs | 4830b150bb7d6d45040a0ce0e64bfb09950ae986a323d88c7a2aa0d5ac1df77a |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs | 37c9674bd75c64575afede99a896bce4dfc3b731ec11f5c36bf14875cd6a8dc2 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-transport/src/native_permissions.rs | 0d5c3ee52a5cc152fd51cdb047c85b4e54f335744f31d98593fd4ec5b87c8cf3 |

Journaux relus et empreintes capturées le 2026-10-13T09:28:30.160835+00:00 :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-10).

## Qualité TypeScript : checkpoint R1 historique

Le lint R1 produit huit warnings no-inline-schema-compile, dont deux nouveaux corrigés ensuite selon ROOT. La relecture finale reste attendue. Types contrats R1 a RC0 ; son journal vide n’est pas une absence d’exécution, le RC vient de ROOT. Types serveur R1 échoue sur le nouveau test Reader149 (TS2375/TS2488/TS2345) ; l’oracle est corrigé, mais aucun résultat ultérieur n’est attesté dans ce checkpoint. Types web R1 a RC1 avec erreurs hors diff en classification, notamment dans MessagesTimeline ; leur origine n’est pas déclarée préexistante sans preuve supplémentaire.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-11).

## Complément adaptateur de bureau — replays et qualité après corrections

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-12).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-13).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-14).

Le lint R2 des dix fichiers retourne RC0 et cinq warnings no-inline-schema-compile historiques dans Reader. Correspondance actuelle→base attestée par ROOT : 591→574, 607→589, 639→611, 734→706, 781→753. Aucun nouveau warning restant n’est déduit d’un simple RC0 : cette qualification repose sur la comparaison avec la base.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-15).

### Limite web conservée

Le contrôle types web reste RC1 avec dix diagnostics : sept dans MessagesTimeline.logic.test.ts aux lignes 197/201/203/206/211/217/223, et trois dans MessagesTimeline.test.tsx aux lignes 948/996/1031. Aucune erreur dans les dix fichiers 162 contrôlés. L’owner a vérifié l’identité octet pour octet avec la base d833387203e113ec2a24f947ba8b8064a1c2542b des deux tests et des cinq sources/types associés : MessagesTimeline.logic.ts, MessagesTimeline.tsx, session-logic.ts, test-fixtures.ts et orchestrationV2.ts.

La cause lue est une union de fixtures où providerThreadId peut être undefined alors que le type attend null. Cette comparaison rend la préexistence fortement plausible ; aucune compilation de la baseline n’a été exécutée. Ne pas déclarer le web GREEN ni sa baseline compilée. Les argv complets lint/types ne figurent pas dans les journaux disponibles et ne sont pas inventés.

### Empreintes post-corrections

Capture actuelle le 2026-10-13T09:37:36.660829+00:00 ; seuls les trois fichiers corrigés sont réempreintés. Les captures antérieures du tableau T035/T036 restent historiques.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-16).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-17).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-18).

## Runtime 162 — checkpoint après transport R1 et daemon R3

Les résultats ci-dessous portent sur les manifestes capturés avant chaque passage, pas sur les sources modifiées ensuite. Les reçus JSON et les journaux ont été lus pour ce checkpoint. Aucun test n’a été lancé par le rédacteur.

| Groupe | Compilation | Tests | RC et durée totale |
|---|---|---|---|
| transport162-runtime-r1 | 15,34 s | 8 PASS, 0 FAIL ; 0,11 s | RC0 ; 17,52 s |
| daemon162-runtime-r1, historique | Échec : chrono absent et deux comparaisons agent_id incorrectes | Aucun test exécuté | RC101 ; 32,24 s, transmis par ROOT |
| daemon162-runtime-r2 | Échec E0308 dans la fixture native_lineage_tests : HumanLineageRequest attendu, WrapperToDaemon fourni | Aucun test exécuté ; fixture corrigée par ROOT avant R3 | RC101 ; 15,96 s |
| daemon162-runtime-r3 | 40,59 s ; 11 warnings conservés dans le journal | 53 PASS, 2 FAIL, 0 ignored ; 11,10 s | RC101 ; 51,76 s |

Les argv des reçus, exécutés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante :

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-transport --lib 162_ -- --test-threads=2
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib 162_ -- --test-threads=2
```

Les reçus JSON ne sérialisent pas l’environnement complet ; aucun environnement supplémentaire n’est reconstruit ici. Ils enregistrent les argv, le code de retour et la durée totale. Les manifestes associés conservent les empreintes sources prétest.

### Deux échecs R3 encore ouverts à ce checkpoint

- attach::tests::native162_lineage_live_rejoue_overrun_et_conserve_gap_irrecuperable : assertion dans attach.rs:6175 ; obtenu Err(JournalUnavailable), attendu Err(BindingUnavailable).
- remote_worker::tests::worker162_maitre_ancien_refuse_avant_hello_et_credential : assertion dans remote_worker.rs:3296 ; probe.is_ok() obtenu false, attendu true pour le maître compatible.

Les propriétaires diagnostiquent ces écarts. Aucun des deux n’est classé comme corrigé ou rejoué dans ce checkpoint. Le résultat 53/2 ne devient pas un succès global et ne prouve ni la recette distante réelle ni la livraison.

### Couverture réussie, limitée aux sources R3

Le journal contient notamment remote162_capture_commit_rejeu_parent_et_wrapper_absents, remote162_capture_differe_quota_puis_settlement_et_accepte, remote162_capture_refus_identites_et_conflits_premier_preserve et remote162_capture_dispositions_terminales_prouvees. ROOT confirme dans cette couverture la récupération du vrai blob depuis une exécution starting dont l’événement terminal s’est perdu. Le passage ne permet aucune réouverture terminale ou dérogation au quota et à l’annulation.

Les dix oracles remote162 de permissions passent. ROOT confirme la couverture du sous-dossier dans ce groupe. native162_journal_distant_froid_gap_perte_et_gardes_sans_repli_local passe, tout comme worker162_runtime_epoch_fence_et_journal_refusent_mauvaise_identite. Le test de suivi vivant en échec reste séparé de cette réussite de lecture froide et des gardes.

Aucune case supplémentaire n’est cochée. T002 reste ouverte dans ce complément : les huit oracles transport passent sur leur manifeste, mais ROOT corrige encore le raccordement RejectedPrepare. La stabilité du contrat après ce correctif doit être confirmée avant son cochage.

### Essai réel de permutation atomique adaptateur de bureau, application intacte

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-19).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-20).

### Fichiers de preuve et empreintes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-21).

## Runtime R4/R5 — groupe ciblé réussi, E2E encore ouverts

R4 : RC101, 19,74 s. Une seule erreur E0277 empêche la compilation : DelegationControlRefusal ne se convertit pas en String dans delegation_halt. Aucun test exécuté. ROOT corrige par map_err vers control_state_unavailable.

R5 : RC0, 52,13 s au total. Compilation 46,66 s ; 59 tests unitaires PASS en 2,06 s ; trois intégrations CLI PASS en 0,12 s. Aucun échec ou test ignoré dans ce groupe. Il ne s’agit pas de la suite globale ni d’une recette avec tous les processus distants réels.

Commande attestée par les deux reçus, depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante :

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib --test lineage_cli_test 162_ -- --test-threads=2
```

Le reçu R5 exclut explicitement tests/native_delegation_e2e.rs et tests/fixtures/remote_worker_162.rs du manifeste, car cette cible n’est pas compilée et ses sources sont en cours de préparation. Leur absence d’empreinte n’est pas une preuve de stabilité ou de réussite. Le manifeste R5 borne seulement les sources de son groupe.

Les deux échecs R3 sont résolus dans R5. ROOT attribue leurs causes aux fixtures : réponse JSON worker sans saut de ligne final ; faux serveur Attach fermé trop tôt. Les noms exacts native162_lineage_live_rejoue_overrun_et_conserve_gap_irrecuperable et worker162_maitre_ancien_refuse_avant_hello_et_credential figurent maintenant PASS. L’historique R3 reste conservé.

### Couverture et décisions de cochage

- T005 validée : worker162_blob_immutable_et_relais_apres_reouverture vérifie métadonnées inchangées, rejeu égal et divergence sans écrasement ; worker162_budget_reserve_sept_slots_et_compte_orphelins vérifie le plafond et les temporaires ; worker162_publication_compte_json_et_refuse_symlink_et_temporaire exerce un corps de 262 144 octets échappés, le symlink et le temporaire ambigu. Les oracles verrou/enrollment, corruption et modes privés passent aussi. La source remote_worker.rs est toujours identique au manifeste R5 au moment du contrôle documentaire.
- T033 validée : les quatre tests unitaires CLI passent avec les trois intégrations native162_lineage_cli_accepts_requested_v2_success, native162_lineage_cli_preserves_v1_error_for_v2_request et native162_lineage_cli_rejects_v1_success_for_v2_request. Ils complètent les preuves transport déjà consignées. cli_lineage.rs et lineage_client.rs sont identiques au manifeste R5.
- T022 reste ouverte : les quatre tests wrapper passent, dont remote162_wrapper_blob_sans_send_apres_arret_worker. La lecture montre un WorkerStore déjà fermé et une publication sans Send/SendIdempotent, puis réouverture et lecture du résultat. Cela ne prouve pas encore l’interruption d’un processus worker pendant la publication réelle prévue par le plan. Les E2E en préparation doivent couvrir cette frontière.
- T002 reste ouverte : le contrôle transport remote162_observation_recontrole_borne_et_refus_prepare est en cours, sans résultat repris ici. T026 reste ouverte : raccordement schedule_execution_recovery avec le slot de pause152 en cours. T014/T028/T031 attendent encore les essais avec vrais processus.

Les captures de blob starting, notification canonique du parent et refus exact des deux appelants Send passent : remote162_capture_commit_rejeu_parent_et_wrapper_absents, remote162_resultat_canonique_notifie_parent_sans_recapture_legacy et remote162_callers_send_refus_legacy_exact_sans_ack. remote162_reprise_managed_conserve_slots_sans_preflight_local et remote162_reprise_sans_desired_conserve_nom_quota_et_identite couvrent la réouverture et les identités Fleet. Ces réussites ne soldent pas à elles seules T026 ou les gates E2E.

Précision T029 : après live_feed_overrun, AttachRelay::restart_from_seq réemploie le lecteur froid sur les fichiers de destination, sous l’autorisation de l’abonnement et avec la garde du snapshot à chaque émission. Il n’appelle pas WorkerJournalPage ; ce RPC sert la lecture froide explicite. Les gaps de rétention irrécupérables restent signalés. Le replay et les gardes sont PASS dans le groupe R5.

### Empreintes des preuves R4/R5

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r4.log | 71b1f3ceba21cdacb3cb38941a48323241561a35625d525942af3fee079d1246 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r4.json | 47552f205b6c5dc6024b0957826bc88fc0a488d9fdcb016d284870a75515883a |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r4.sources | a078d728beeb7b14c792a3e0a79fd4545b5337c1b3921287752fd9dc43743dd9 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r5.log | 5212fd05cd21f49c08a713b870c61c982bbbc5704f85093e17f53751382703ae |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r5.json | a5719002a8573ff6c0e895b00c2240e85898a24cbd0966dfad59efc4c4fa8d13 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r5.sources | 4c023aba91f4153276294f3ff0aea6d2b7a0fd039c233f8582c76b855e105d86 |

## Transport final ciblé et reprise152/160 — avant résultat E2E R1

| Passage | Résultat exact | Compilation / tests / durée totale |
|---|---|---|
| transport162-runtime-r2 | RC101 ; un oracle échoue : ResourceLimit obtenu, InvalidEnvelope attendu | 13,94 s / 0,01 s / 14,02 s |
| transport162-runtime-r3 | RC0 ; même filtre, 1 PASS après correction de l’oracle | 5,22 s / 0,00 s / 7,61 s |
| transport162-final-targeted | RC0 ; 9 PASS, binaire R3 réutilisé sans rebuild | aucune / 0,13 s / 0,14 s |
| daemon162-recovery-r1 | RC0 ; 5 PASS ciblés reprise152/160 | 23,15 s / 20,40 s / 46,38 s |

Le FAIL transport R2 est conservé. Il concerne protocol::tests::remote162_observation_recontrole_borne_et_refus_prepare, assertion protocol.rs:6477. Le correctif ajuste l’oracle à ResourceLimit ; il ne transforme pas un dépassement de borne en acceptation.

Les manifestes transport R3 et final-targeted ont été comparés : ils sont identiques octet pour octet. Le reçu final confirme binary_from=transport162-runtime-r3 et rebuild=false. Les deux sources de T002 restent identiques aux empreintes prétest lors de ce contrôle : protocol.rs SHA-256 f4fe42a4aba7506da649b33aa5d5e84428bdc29053ff29adb09ef7b3a560a84b ; lineage_protocol.rs SHA-256 ba77269ec8991b8311b8db33612442d27ad18c774a6c019695c80b31227aa0b8.

Les neuf oracles couvrent : version de projection list/show ; wire v1 inchangé et placement v2 fermé ; contrat fermé et corrélations de slot ; dispositions fermées ; handshake, capacité et secret masqué ; borne Observation et refus Prepare ; chemins/révisions/permis ; journal/réponse bornés sans chemin fourni ; digest du résultat et identité de reprise quota. Ils complètent la relecture du contrat et permettent de cocher T002. Progression : 11/42. Cela ne valide pas l’exécution distante complète.

### Reprise : cinq réussites et limite de l’assertion ajoutée après

Les cinq tests exacts de daemon162-recovery-r1 sont :

- daemon::native_delegation::tests::remote162_pause_reconstruction_unique_attend_slot_et_refuse_reconnexion_active ;
- daemon::native_delegation::tests::native152_rouge_pause_reprise_attend_interrupted ;
- daemon::native_delegation::tests::native152_pause_chaine_suivie_apres_reprise ;
- daemon::native_delegation::tests::native160_rouge_relance_frontieres ;
- daemon::native_delegation::tests::native160_rouge_concurrence_reprise_ordinaire.

L’assertion AgentInfo ajoutée ensuite n’est pas couverte par ce passage. Elle doit être exécutée dans le prochain groupe E2E R1 selon ROOT. T026 demeure ouverte jusqu’à la consolidation de ses preuves ; les cinq réussites ne sont pas attribuées aux octets modifiés après le manifeste. Le reçu exclut les deux sources E2E, non compilées par ce groupe.

Quatre E2E sont maintenant écrits dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs : release/rejeu/restart sans second fournisseur ; expiration et coupure avant RELEASE ; cancel avec naissance divergente puis arrêt attesté ; délégation avec maître réel, résultat unique, cwd et modèle. La présence des quatre fonctions a été vérifiée. Leur build R1 est actif chez ROOT ; aucun résultat E2E n’est disponible dans ce checkpoint. T022, T014/T028/T031 restent ouvertes.

### Commandes attestées

Les argv proviennent des reçus JSON, depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante. Aucun environnement absent des reçus n’est reconstruit.

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-transport --lib remote162_observation_recontrole_borne_et_refus_prepare -- --test-threads=2
```

```bash
nice -n 10 /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/bridget_transport-fde7a29f8bcbdaca 162_ --test-threads=2
```

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib -- --test-threads=2 remote162_pause_reconstruction_unique_attend_slot_et_refuse_reconnexion_active native152_rouge_pause_reprise_attend_interrupted native152_pause_chaine_suivie_apres_reprise native160_rouge_relance_frontieres native160_rouge_concurrence_reprise_ordinaire
```

### Empreintes des preuves

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r2.json | f7f233f3fc0493672c12833020fb0815c50c093f0dc6f8e0ff2fb55f6f79178a |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r2.log | 30247791568f58dfdf7e6c48876f87d89a8ea9ba923f177b10debdbe6a1fb415 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r2.sources | 683077ce804747a0fd5d0e42106f2b5feaa97f2d7e7d8908aad7104f3d0209d7 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r3.json | dddde2f49885cb975e9ff3de4ec9e29511e0ab55345ed2803f7439700ecb9576 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r3.log | e42e3a375509fb1981b8d4dd3ff379f3deeeeca35a226f404ed97cce9ab01481 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r3.sources | 4975733cfdd6767804f305b99115d314ea2b7fc359a189c3b4c02ca8f8b6d751 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-final-targeted.json | 24a63ebd998be5b8ef89688443f868bb1da84f35eb01745a533d4f1fd3303f29 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-final-targeted.log | 03c36b3becbfcf8b7bb9f3f82cac1159db8223856ade21904a6526790f6b7015 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-final-targeted.sources | 4975733cfdd6767804f305b99115d314ea2b7fc359a189c3b4c02ca8f8b6d751 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-recovery-r1.json | 6c16fe5c7bcade19b905c73611325b19227130220bdaada383c2ce14070a6174 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-recovery-r1.log | fdd0078290e5e68eb2cba20794d5a95a16d2bfbcab2320100e4ddde76ef7de80 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-recovery-r1.sources | 8ff3fc1558fbeba13233f6e1e5dfc3a95e9429376430738cc3353006aafd3a81 |

## E2E R1 — AgentInfo réussi, quatre E2E échoués

Le reçu daemon162-e2e-r1 donne RC101, 51,80 s. Compilation 26,86 s. Le test unitaire remote162_reprise_managed_conserve_slots_sans_preflight_local passe en 0,09 s ; l’assertion AgentInfo ajoutée après recovery-r1 est désormais couverte dans ce passage, selon ROOT. Les quatre E2E échouent en 24,74 s ; aucun E2E n’est déclaré réussi.

- remote162_e2e_cancel_naissance_divergente_preserve_processus_puis_arret_atteste ; remote162_e2e_permis_expire_et_coupure_avant_release_sans_fournisseur ; remote162_e2e_release_rejeu_restart_worker_sans_second_fournisseur : WorkerHello absent, délai dépassé. ROOT a identifié la racine de socket de fixture en 0755, refusée au démarrage. Correction de fixture prévue, pas repli de la garde de production.
- remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele : comparaison du groupe du fournisseur avec celui du wrapper, valeurs distinctes 87253 et 87198 à remote_worker_162.rs:940. ROOT attribue l’échec à l’oracle de fixture. Le mode worker a été compilé et réellement lancé dans ce scénario ; son succès de bout en bout n’est pas acquis.

Les corrections de fixtures sont connues ; les compléments T022/T025 restent en édition. Aucun replay ultérieur n’est inclus dans ce checkpoint et aucune tâche supplémentaire n’est cochée.

ROOT atteste rustfmt avec édition2024 RC0 sur treize fichiers Rust stables. Une première invocation avec édition2021 a échoué par mauvais paramètre d’outil, puis a été corrigée. Il s’agit d’un pur formatage ; ce contrôle n’est pas un test de comportement ni un succès E2E.

Commande provenant du reçu :

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib --test native_delegation_e2e -- --test-threads=2 remote162_e2e_ remote162_reprise_managed_conserve_slots_sans_preflight_local
```

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r1.json | 2c585f701115167f87856d5d73597d3d753179eb4bc8220b4f715cbfe3d1c69c |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r1.log | 3d23a2bae2b212e9225a05bc9acfe7dce5c1b44026f33495e1dbb46232cf1ea1 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r1.sources | a4e84c644fdb16664155af0cdd233b82b336bdc39c8803442669a41dfdf0aa4a |

## Groupe R2 et régressions155 — résultats distincts, E2E non validés

| Passage | Résultat | Durée |
|---|---|---|
| daemon162-e2e-r2 | Compilation réussie ; 66 unitaires PASS, 1 FAIL ; Cargo s’arrête avant les intégrations | Compilation 1 min 09 s ; tests 6,13 s ; total 75,99 s, RC101 |
| daemon162-e2e-r2-processes | Binaire E2E déjà compilé, lancé directement sans rebuild ; 0 PASS, 4 FAIL | Tests 37,61 s ; total 37,67 s, RC101 |
| daemon162-child155-regression-r1 | Même build R2 ; 56 PASS | Tests 24,10 s ; total 24,23 s, RC0 |

L’unitaire échoué est worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications : StoreUnavailable à remote_worker.rs:3555. ROOT et l’owner ont identifié un task_id non UUID dans la fixture et corrigé seulement le test après la capture R2. Aucun replay de ce correctif n’est attribué à R2.

Les trois E2E par relais échouent encore sur WorkerHello absent, avec Resource temporarily unavailable (os error35) observé dans le relais. Le nominal atteint réellement result_available avec le résultat fixture162-remote-answer, puis échoue dans l’oracle de vue à native_delegation_e2e.rs:430. Ce constat métier ne transforme pas le scénario en PASS. L’owner poursuit le diagnostic sur données réelles ; aucune correction ultérieure n’est déclarée testée ici.

Les 66 réussites incluent les trois primitives historiques managed_process : crash_reel_aux_trois_frontieres_preserve_la_preuve_attendue, eof_abandonne_tandis_que_release_exec_la_sonde et annulation_avant_marqueur_termine_le_bootstrap_sans_exec. Elles complètent les preuves de frontière sans remplacer les E2E162 échoués.

La régression155 contient 56 réussites. Deux oracles sont déjà inclus dans les 66 : remote162_demande155_recontrole_worker_hors_verrou_et_epoch_avant_reponse et remote162_quota157_partagee_par_agent_type_avant_reprise_distante. Ne pas additionner ces passages comme 122 tests distincts. La nouvelle réponse155 revalide le worker hors verrou puis époque/contact/identité/droits avant émission, selon la relecture ROOT ; les chemins distants ne sont plus lus sur le maître. Le parent ne peut toujours pas approuver la commande humaine.

T030 est cochée après relecture ROOT, preuves antérieures de résultat canonique/notification et cette régression155. T014/T028/T031/T032 restent ouvertes. Aucun autre cochage ni validation E2E globale. AgentInfo était déjà PASS dans E2E R1 ; il n’est pas présenté comme une nouvelle réussite de ce checkpoint.

### Provenance des exécutions directes

Les deux reçus directs pointent build_source_manifest=daemon162-e2e-r2.sources. Binaire E2E SHA-256 a6386b3713d184cce5fecbdbbca89f152e032e9b6576f6d8afc0f3240abae82d ; binaire unitaire SHA-256 e7303f0ac191ac87e0118af5f53cf934fff96ba26688ecb072e88cda82493cba. Les sources corrigées après ce manifeste ne sont pas couvertes automatiquement.

Le reçu R2 consigne cache_retention_gib=45. Mesure de départ transmise par ROOT : 37,90 Gio logiques de cache, 30,88 Gio libres sur le volume. Rétention temporaire utile 162/163, aucune cible de consommation ni purge forcée après compilation. T040 et G9 doivent conserver cette limite et décider le nettoyage final sur preuves.

### Commandes et empreintes

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --lib --test native_delegation_e2e --test lineage_cli_test -- --test-threads=2 162_ native_child_requests::tests:: crash_reel_aux_trois_frontieres_preserve_la_preuve_attendue eof_abandonne_tandis_que_release_exec_la_sonde annulation_avant_marqueur_termine_le_bootstrap_sans_exec
```

```bash
nice -n 10 /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/native_delegation_e2e-857f386f1a63dfc7 --test-threads=2 remote162_e2e_
```

```bash
nice -n 10 /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/deps/bridget_daemon-4f844ffb28f60eee --test-threads=2 native_child_requests155_tests::
```

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r2.json | 6d3ce171786684521b6671c425b74c56ca01dee3a52b0a65e6bc6b537080cfbe |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r2.log | 8a991de56eabacf0e67f2667d8f59583b3343188f02d4af4cf8420bec5f925da |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r2.sources | 5a435e83b8d2a4bf332de93fc6207ce50f943f1fa601392abf616a31b66f353c |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r2-processes.json | afc46a5062ab28a1f6561bf3d6e71a21906bffcde1a659cbce3fd8e97e3dde26 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r2-processes.log | fd32edbac42d51b8a858f91ee6df62975071afef9c6b7822c1b064755980349d |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-child155-regression-r1.json | d79a893e4bd7d3aab75e3812946eb2af939f6524792e25946ef055f60f7d3b29 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-child155-regression-r1.log | 6742e8f7bd28b90de5b28d8050a11e12d6b6632f4507e940c63a8c3282a2e1cb |

## E2E R3 — cinq réussites ciblées et couverture consolidée

Le passage daemon162-e2e-r3 retourne RC0 en 52,41 s : compilation 27,94 s ; unitaire worktree 1 PASS en 4,22 s ; quatre E2E PASS en 15,45 s. Le test worktree corrigé préserve base, branche, modifications et rejeu. Les quatre scénarios passent sur le manifeste R3 :

- remote162_e2e_cancel_naissance_divergente_preserve_processus_puis_arret_atteste : identité PID/naissance divergente refusée, processus préservé, puis annulation attestée ;
- remote162_e2e_permis_expire_et_coupure_avant_release_sans_fournisseur : expiration et coupure avant RELEASE sans fournisseur lancé ;
- remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele : admission par maître réel, cwd/modèle explicites et notification unique ;
- remote162_e2e_release_rejeu_restart_worker_sans_second_fournisseur : arrêt du worker pendant production, publication du blob et mort du wrapper, reprise, ACK perdu et refus d’un second fournisseur.

Ces preuves complètent les unitaires et régressions déjà conservés. Les anciennes erreurs R1/R2 restent historiques, sans addition de replays comme nouveaux tests distincts. Le cache après R3 vaut 38,74 Gio logiques ; aucune purge ni économie physique déduite.

### Mapping de validation T006–T039

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-22).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-23).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-24).

La FULL Rust unique FULL162-once-final-source est lancée par ROOT sur les sources gelées. Aucun résultat disponible à ce checkpoint. T040/T041/T042 restent ouverts ; le statut Implemented ne peut pas être attribué avant 42/42. Les gates de production et nettoyage restent distincts.

Commande R3 :

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 --no-fail-fast -p bridget-daemon --lib --test native_delegation_e2e -- --test-threads=2 worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications remote162_e2e_
```

| Preuve absolue | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r3.json | 25a58b8fdce8c86e435902c97474088a931abffac5d1130cfc39a20137ee56de |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r3.log | a97366a77a724544be6a7fcf0f9602d7f25e000e370a371c9fd1946182acbdcb |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-e2e-r3.sources | 26cf1a32537fffbc45854d1b928249add5ccf1f1b98ac8ea6aa33c3d13d82958 |

## FULL unique en cours — résultats partiels, aucun verdict global

ROOT poursuit FULL162-once-final-source, session 41184. Le dernier bilan partiel du seul libdaemon est 1 918 PASS, 68 FAIL et 15 ignored ; les intégrations continuent. Ce compteur n’est pas le résultat final du workspace et ne remplace aucune preuve ciblée antérieure.

ROOT a identifié des causes de runner pour une partie des échecs : umask non 077 (DB créée en 0644 et profil privé en 0755), TMPDIR /tmp/b162-szysmv9d non canonique ou trop long, snapshot158B non fourni. Cette attribution reste partielle. Aucune baseline n’a été exécutée pour prouver que tous les échecs sont antérieurs ; aucun GREEN global ni succès par soustraction n’est autorisé.

La revue du code par ROOT a aussi trouvé des correctifs de production à préparer après la FULL : startup terminal, validation du tuple avant mutation et borne observed. Les oracles dédiés ParentProof/révocation sont en préparation. Les cases déjà cochées renvoient aux preuves versionnées précédentes ; elles ne valident pas automatiquement ces octets futurs. Converge reste à faire. Aucun cochage anticipé dans ce checkpoint et aucune seconde FULL prévue.

## T042 — quatre défauts retenus avant Converge

La relecture indépendante de T042 a produit quatre constats, dont les ancrages ont été relus par ROOT. Cette étape n’est pas un passage Converge ; T042 reste ouverte.

| Priorité | Défaut retenu | Correction en préparation et limite |
|---|---|---|
| P1 | La deadline de bootstrap de 300 secondes continue à s’appliquer à Released : elle peut annuler une mission ou refuser ses slots après cinq minutes. | Séparer la fenêtre de bootstrap de la durée métier et des droits de reprise. Aucun replay du correctif n’est encore repris ici. |
| P1 | Désactiver une machine empêche son authentification de reprise, malgré la branche maître prévue pour autoriser la récupération. | Préserver la récupération attestée des exécutions existantes tout en refusant les nouvelles admissions. Pas d’activation de route impliquée. |
| P1 | Le parent courant n’est pas relu juste avant RELEASE : le parent capturé est comparé à lui-même. | Relire l’autorité courante puis vérifier la preuve avant libération. Les anciens tests de primitives ne prouvent pas ce raccord corrigé. |
| P2 | Une posture omise est forcée à Development au lieu d’hériter de celle du parent. | Préserver l’héritage lorsque la demande ne fournit pas de posture. |

Code161/reuse préparent ces corrections sous coordination ROOT, encore en gel pendant la validation 86336 au moment de ce checkpoint. Aucun résultat futur, mutation de service ou cochage T042 n’est déduit. Les cases antérieures restent des preuves versionnées ; elles ne constituent pas une validation automatique des corrections en cours.

### FULL conservée et replays partiels

La FULL unique termine en 1 212,60 s avec RC101. Son parser conserve 2 613 PASS, 122 FAIL, 68 ignorés et 97 cibles. Le comptage utilise le dernier résultat de chaque bloc Cargo et exclut les helpers ; il ne somme pas les résultats internes d’un même binaire comme des cibles supplémentaires.

ROOT atteste le replay unitaire R1 : 125 PASS, dont les 68 échecs précédents du libdaemon. Les premières intégrations corrigées réussissent. Dix échecs des tests159 restent ; la majorité est reliée de façon prouvée à la feature test-support absente. La prochaine compilation dédiée doit vérifier ces cas ; elle ne constitue pas une seconde FULL. Les journaux détaillés de ces replays ne sont pas réinventés ici : ces nombres proviennent du checkpoint ROOT et devront rester associés à leurs reçus à la consolidation finale.

L’environnement de replay est attesté par /Users/moi/.cache/bridget-delivery162-20261013/replays162-environment.json : TMPDIR court et canonique /private/tmp/bq0davs1t, mode 0700, umask 0077, snapshot de migration explicite /Users/moi/.cache/bridget-restoration159A-reference-20261012/bridget158B-real-snapshot.db. Cela corrige des prérequis du runner ; ce n’est pas une baseline du code antérieur. Les autres échecs ne sont pas supposés préexistants.

Le diagnostic des 54 échecs d’intégration est rapporté par ROOT/cache161 : 32 liés à umask prouvé, 7 probables, 14 prérequis et 1 mesure de performance SC005 à 6 166 ns pour un seuil de 5 000 ns. Ces catégories décrivent un diagnostic, pas des tests déjà rejoués avec succès. Aucun GREEN global ou score recalculé après retrait arbitraire des échecs.

| Preuve absolue | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/FULL162-once-final-source.counts.json | 6686164c039acb143166e5edda2c62fbc498e5b21c5baddf3dc3dfee9163773a |
| /Users/moi/.cache/bridget-delivery162-20261013/FULL162-once-final-source.json | af6fdc51101806a8ba06640971c874e99316f864bd4b00165ae71d0fee2da801 |
| /Users/moi/.cache/bridget-delivery162-20261013/FULL162-once-final-source.sources | 26cf1a32537fffbc45854d1b928249add5ccf1f1b98ac8ea6aa33c3d13d82958 |
| /Users/moi/.cache/bridget-delivery162-20261013/replays162-environment.json | da20443cf7153e090a2fee0992ca0947ee22598d1820cbfd2810b1e19ecea50b |

## Clôture développement — replay R2 réussi, FULL historique conservée

REPLAY162-selfreview-et-hooks159-r2 retourne RC0 en 206,675 s. Il compile une seule fois les cibles ciblées avec bridget-daemon/test-support, puis lance les binaires identifiés dans le reçu. Résultat : 60 unitaires daemon, 9 transport, 4 E2E162, 3 edit159 et 7 held159, soit 83 PASS/0 FAIL. Environnement attesté : umask 0077 et TMPDIR /private/tmp/bq0davs1t. Aucun second workspace FULL n’a été exécuté.

Le replay R1 complet, dont le compteur est maintenant lu, avait 173 PASS/10 FAIL, RC101 en423, 438 s. Son seul groupe libdaemon avait 125 PASS ; ce n’était pas le total du replay. Parmi les 122 échecs de la FULL historique, 112 ont un rejeu PASS dans R1 et les dix restants dans R2, selon le rapprochement vérifié par ROOT. Les autres réussites ciblées incluent régressions et nouveaux oracles ; ne pas les additionner comme des tests uniques ni réécrire la FULL en succès. Celle-ci reste 2 613 PASS/122 FAIL/68 ignorés, RC101. Aucune baseline ancienne inventée.

### Dernières tâches soldées sur preuves

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-25).

### Résolution des quatre défauts T042

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-26).

Les ancrages ci-dessus sont ceux transmis par la lecture indépendante des sources R2. Leurs chemins complets appartiennent à /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src. La validation reste composée de tests d’état/transaction, processus réels et lecture des primitives fsync. Elle ne prétend pas injecter une panne à chaque instruction CPU ou reproduire une panne électrique.

### Limites conservées après 42/42

Développement et préparation sont Implemented, 42/42. Converge, contre-revue/audit finaux et G8/G9 restent à effectuer. Les recettes distantes réelles et les configurations installées ne sont pas attestées par les fixtures locales. Les trois projets sont autorisés et provisionnés ; aucune route n’est activée par ce checkpoint.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-27).

### Commande de compilation ciblée R2 et provenance

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon -p bridget-transport --features bridget-daemon/test-support --lib --test native_delegation_e2e --test edit159_e2e --test held159_e2e --no-run --message-format=json
```

Les cinq argv de lancement, chemins des binaires, options et durées sont conservés dans le reçu JSON R2 ; ils ne sont pas reconstruits. Le manifeste R2 répertorie 42 fichiers du périmètre capturé, pas tous les fichiers historiques du dépôt.

| Preuve absolue | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-selfreview-et-hooks159-r2.json | f025b2ff50d526109282b46c0c9102ba24362d5f047cd872ee9e1ccf4df40cd7 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-selfreview-et-hooks159-r2.log | c5adf13403fdfcd7876fa06f353ee68cd9b5dacf02c2988143f8793cd7ec2be4 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-selfreview-et-hooks159-r2.sources | 99361bc92e0e76158b91d0b57cc39c795dc68316501737b1e84f0633ff87e027 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-correctifs-et-environnement-r1.json | 0f8ddc193f9d94d371f91a8288c9e49a61dea266ead6a5906b609a1a1359454f |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-correctifs-et-environnement-r1.log | 139aad24f69c8731f3cacd2444a0d8160b896a01b3832bfcc9b8e37c7342673a |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-correctifs-et-environnement-r1.sources | 4a1b9dd06df4a5320378e34d9e7e23e07fcf1c4a1b8a2471eee712e6823111a6 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-correctifs-et-environnement-r1.counts.json | da9e579eccda06568b9c489e8956bd980063d43fc99377b67e5526951ffb5b48 |

## Converge1 clos — non convergé, T043 ouverte

ROOT atteste un seul manque contractuel : T008 exige une trace du remplacement de connexion worker sans secret. Ce n’est pas un défaut de permission ni de lancement. Converge1 est clos sans convergence. ROOT a ajouté T043 après ce passage ; code161 prépare un correctif ciblé dans daemon.rs, hors phase de convergence. Aucun résultat de test du correctif n’est disponible à ce checkpoint et T043 n’est pas cochée.

État courant : In Progress, 42/43. L’ancien 42/42 décrit le gel avant Converge1. La phase n’a pas été consommée ou renommée en réussite ; la prochaine clôture attend le GO de ROOT. G8/G9 restent ouverts.

Le rapprochement exact de la FULL est désormais prouvé par /Users/moi/.cache/bridget-delivery162-20261013/replays162-coverage-full-failures.json, SHA-256 5291b6f3f245a301ae157d100e13139e35b290cda8a6d4f28f9804e2449a2a20. Il associe chacun des 122 noms de tests échoués à une ligne PASS de R1 ou R2, sans manquant. Les 42 empreintes sources R2 sont réattestées identiques à l’instant de ce reçu, avant le correctif T043 ; cette preuve ne s’étend pas automatiquement aux octets futurs. Aucun nouveau test, aucune baseline et aucun GREEN global ne sont déduits.

## T043 validée — 43/43, gel avant Converge2 ciblé

REPLAY162-T043-auth-epoch-r3 retourne RC0 en 48,9505 s : compilation 38,07 s puis un oracle PASS en 0,09 s. Le test existant daemon::presence_tests::remote162_worker_auth_epoch_et_contact_sans_presence_fournisseur est rejoué, sans nouvelle campagne globale. Le reçu porte daemon.rs SHA-256 71c5b7065eb8e1133edbbc6a56a14165a0768fd7bd89a1ae6317e3b9c389abe4.

ROOT a relu le bloc de production : previous_epoch est capturé avant insertion ; info! est émis après drop(st), seulement lors du remplacement. Le message porte machine_id, worker_id et ancienne/nouvelle époque, aucun credential. La présence et le contenu de la trace sont attestés par cette lecture ; le replay vérifie le comportement existant d’authentification/époque, sans prétendre qu’un nouvel oracle capture le texte du logger.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-28).

```bash
nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --features test-support --lib remote162_worker_auth_epoch_et_contact_sans_presence_fournisseur -- --test-threads=2
```

| Preuve absolue | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T043-auth-epoch-r3.json | b17a68832c907dc14bcb200c51e4638ca3dfbd983b525aae1e913f3cdceb993a |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T043-auth-epoch-r3.log | b33e675a665a502c52ce52d27b58e2bb134d2aeee5cbe2b865e2af7f383e9990 |

Le journal conserve 43,27 Gio logiques de cache utile après ce passage ; aucune économie physique ni purge forcée déduite.

## Converge2 — CONVERGED, développement/préparation 43/43

Le passage 2 ciblé est terminé : CONVERGED, aucune tâche ajoutée. L’agent de lecture a capturé la même empreinte tasks.md avant et après : SHA-256 71115d7c1ebffd0aa5d29368b52a6bc6874ceda66f2a5f7006ba8e43ecfd84ae. Le rédacteur l’a réattestée sans modifier ce fichier. Les 43 tâches restent validées ; ce verdict ne clôt pas la livraison.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-29).

La contre-revue Claude code R1 est lancée, verdict en attente, borne 15 minutes. Identifiant de délégation : node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r1. L’audit est préparé en parallèle sur les sources gelées ; aucun verdict favorable n’est anticipé. G8/G9 restent ouverts. Aucun GREEN global, installation, activation ou nettoyage final déduit de CONVERGED.

## Contre-revue code R1 BLOCKED — T044, 43/44

ROOT a reçu/acquitté BLOCKED et confirmé un manque de réconciliation starting, une observation Refused ignorée et un reçu Reserved après erreur de création. Ces trois axes d’un même défaut de raccordement sont détaillés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/adversarial-review-claude.md. T044 ajoute réconciliation bornée hors mutex, refus durable avant naissance prouvée ou arrêt attesté et deux E2E sans DesiredStopped injecté.

Après création, EOF/StartupFailed ne suffit pas à prouver la mort : conserver Unknown si l’arrêt n’est pas attesté. Aucun résultat futur ni correctif validé n’est annoncé. Les propriétaires corrigent ; état In Progress, 43/44. Converge2 reste historique sur la source précédente, aucun passage 3 encore. Préparation de l’audit pré-R1 suspendue ; G8/G9 ouverts. tasks.md SHA-256 après ajout : 4b9f45140710afd3ff7a7d4aea14d58c4258532c7cc18870a571ebcef43e189a.

## R1 — nuance de borne et provenance des prochains essais

ROOT confirme prepared.expires de 30 secondes dans le prédicat wait_ready_or_cancel, en plus de deadline_at 300. L’accusation d’une attente 300 sans borne 30 est corrigée comme inexacte ; le manque de réconciliation reste retenu. T044 précise ses deux E2E : StartupFailed après RELEASE et vrai ReadyToRelease retardé six secondes face au permis cinq secondes. Arrêt réel, remote_start_failed et quota 1 réutilisable sont attendus sans DesiredStopped injecté. EOF/StartupFailed avant Ready relève des tests de primitives séparés. Tous ces nouveaux résultats sont encore en attente ; aucun cochage ni verdict favorable anticipé.

## R1/T044 — clôture du slot sans résultat encore à valider

ROOT et l’owner ont confirmé un reliquat du même défaut : finish_command ne fermait pas les slots sans blob. Après remote_start_failed, la réservation durable de 2 Mio restait comptée ; le nettoyage maître ne transmettait pas Cancel ResultSlot. La correction worker réemploie close_result_slot après arrêt attesté et réessaie la clôture sur Terminal/Refused si le stockage a échoué. Elle ne collecte pas les blobs Deferred et ne libère aucune preuve ambiguë.

T044 inclut désormais la libération de cette réservation sans résultat et sa preuve ciblée, encore en attente. Aucun cochage : 43/44. La sonde MASTER se limite maintenant à un seul candidat coûteux par cadence ; le curseur avance aussi si le candidat est invalide. Aucun parcours coûteux N+1 ajouté. Ces points décrivent les corrections en cours, pas des tests réussis.

## T044 — passage R4 réussi, reliquat de parcours worker ouvert

ROOT a exécuté R4 avec test-support, umask 0077, TMP canonique /private/tmp/bq0davs1t, nice 10 et deux threads. Résultat : **73 tests lib PASS, 4 helpers de fixture ignorés ; 4 E2E PASS ; zéro échec, RC0**. Durée totale : 73,133 s ; compilation 32,89 s selon le reçu de ROOT. Les journaux donnent 4,27 s pour la lib et 32,12 s pour les E2E. Le reçu atteste sources_unchanged=true. Les quatre E2E sont des fonctions existantes enrichies : ne pas compter leurs nouveaux scénarios comme des fonctions de test supplémentaires.

Les primitives worker162_bootstrap_erreur_garde_handle_jusqua_arret et worker162_marqueur_refuse_conserve_bootstrap_abandonne passent. worker162_collecte_exige_disposition_producteur_et_cloture passe aussi. Les E2E passent par le vrai maître et worker : StartupFailed après RELEASE et vrai ReadyToRelease retardé de six secondes face au permis de cinq secondes. Ils observent arrêt, remote_start_failed et quota de 1 réutilisable sans injection de DesiredStopped. Ce passage ne teste pas une expiration de préparation de 30 secondes. Les erreurs avant Ready relèvent des tests de primitives distincts.

Preuves privées et SHA-256 :

| Fichier | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-demarrage-r4.json | 6b58ae512366f963e309354de6c648f6ae0c89464b60873db5448e4c86699863 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-demarrage-r4.log | 33e07816e52c30f185448df0b65c4c956fe950d3c7b27925d371dbef09d05afd |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-demarrage-r4.sources | 11eea09dd5389e5f4a6565e7e5387875ec47fb5ed4496e8a35a2ede118f1e0f5 |

Compilation : `nice -n 10 python3 scripts/build.py cargo test --locked --offline --jobs 2 -p bridget-daemon --features test-support --lib --test native_delegation_e2e --no-run --message-format=json`, depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante. Les deux commandes de lancement exactes, leurs binaires et les filtres sont conservés dans le JSON ci-dessus.

**Limite actuelle :** après ce passage, ROOT a identifié dans reconcile_all un parcours des commandes terminales retenues sept jours qui relance un inventaire complet des slots pour chaque commande. Ce coût répété n’est pas une défaillance mesurée par R4. L’owner corrige par une lecture groupée par passage, sans nouveau store ni indicateur persistant. Aucune latence ni amélioration de performance n’est mesurée. Le replay de cette correction reste attendu. T044 demeure ouverte : **43/44, In Progress**. Converge3 n’a pas commencé. Les preuves R4 restent historiques pour leurs sources ; aucune seconde FULL ni succès global n’est déduit. G8/G9 restent ouverts.

## Checkpoint final T044 — développement 44/44, avant Converge3

R5 valide la dernière correction worker : **17 tests worker PASS et 4 E2E PASS, zéro échec/ignoré, RC0**, en 58,317 s au total. Compilation : 21,83 s selon ROOT. Le reçu atteste sources_unchanged=true. R4 conserve ses 73 lib PASS et 4 E2E PASS sur la source précédente ; les primitives et le maître sont inchangés selon ROOT. Ces replays ne sont pas additionnés comme des tests distincts et ne remplacent pas la FULL historique en échec.

La lecture du worker final confirme un inventaire groupé des commandes et slots dans reconcile_all, puis un index temporaire des commandes Terminal/Refused. La jointure vérifie l’identité complète et réemploie finish_result_slot. Les commandes actives gardent leur réconciliation propre. Aucun store, flag persistant ni protocole nouveau. Aucun gain de latence mesuré n’est revendiqué.

T044 est cochée après lecture des reçus R4/R5 et de cette correction. État courant **Implemented, 44/44 pour le développement et la préparation**. G8/G9 restent ouverts. Converge3 est à exécuter ; aucun verdict favorable de ce passage, d’audit ou de contre-revue n’est anticipé. Le BLOCKED R1 et Converge2 restent historiques. SHA-256 de tasks.md gelé : 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea.

### Provenance du passage final R5

| Fichier | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-maintenance-r5.json | 54ab6561ada60880fde8fb6a4523902fba2f63740620aeba2b4f3ce2e5210eeb |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-maintenance-r5.log | 98a7df2176335b1cfba1988c963da7779fd13076ad8fbde846b43b8449921041 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-T044-maintenance-r5.sources | b161f88b13342f0e32acf66ebdf548318b1ccbae504cd46d5dce65861e3d19fc |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs | 7b749f35859b85487792afd82af409db672687abf347146e26770fb5b898229e |

Le JSON conserve la commande de compilation identique à R4 avec test-support, umask 0077 et TMP canonique. Il lance ensuite le binaire lib avec worker162_, puis les quatre E2E avec remote162_, nice 10 et deux threads. Le journal donne 2,23 s et 32,68 s de tests. Le SHA du worker a été relu après exécution. La provenance des scénarios reste celle de R4 : retard Ready six secondes face au permis cinq secondes, sans test revendiqué du timeout 30 secondes.

## Converge3 — CONVERGED, 44/44, source inchangée

Le troisième passage est terminé : **CONVERGED, zéro tâche ajoutée, 44/44**. SHA-256 de tasks.md identique avant et après : 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea. Le worker final est vérifié à 7b749f35859b85487792afd82af409db672687abf347146e26770fb5b898229e. Aucun fichier de tâches ou code n’a été modifié pendant ce passage.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-30).

Précision de rétention validée par ROOT : une disposition Deferred conserve le canon, le reçu, le blob et le décompte des octets réels. La réserve de publication de 2 Mio peut être libérée après preuve de mort du producteur exact et fermeture irrévocable du slot. Cette libération n’est ni une purge du résultat différé ni l’autorisation d’un nouvel écrivain. Les formulations antérieures « réservation Deferred conservée » désignent la conservation de sa preuve et de ses octets ; elles n’imposent pas de conserver une réserve de publication devenue inutilisable après cette double preuve.

R4 atteste 73 lib PASS et 4 E2E PASS, avec quatre helpers ignorés ; R5 atteste 17 worker PASS et 4 E2E PASS. Les scénarios de panne après RELEASE et de Ready retardé six secondes couvrent le permis cinq secondes et le quota de 1 rendu. Ils ne constituent pas un essai réel du timeout 30 secondes. Les replays ne sont pas additionnés comme tests distincts. Aucun GREEN global n’est déduit.

La contre-revue code Claude R2 est lancée sous taskId node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r2, avec borne de quinze minutes ; verdict en attente. ROOT annonce les modules d’audit terminés avec 0 CRITICAL, 0 HIGH, 3 MEDIUM et 1 LOW. Le scoring et le validateur sont en cours : aucun grade ni succès du validateur anticipé. G8/G9 restent ouverts ; CONVERGED ne vaut ni livraison ni activation.

## Contre-revue code R2 — APPROVE_WITH_CHANGES, D1 requis avant G8

ROOT a reçu et acquitté le verdict **APPROVE_WITH_CHANGES** de la contre-revue R2. Les objections R1 sont résolues. Le taskId reste node:delegated-task:command%3Amcp%3A3fa5c3d9-0ca4-4322-8649-ec020a6fcfe7%3Adelegate-task%3A162-code-review-20261013-r2.

| Point | Vérification et disposition |
|---|---|
| D1 — perte du Child lors d’une erreur de libération | Constat moyen retenu et requis avant G8. ROOT a confirmé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs:2183 et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/managed_process.rs:356 que release().map_err()? consomme le Child et perd ce handle si l’écriture RELEASE échoue. Même défaut si set_status_nonblocking échoue après libération. L’owner corrige ces deux fichiers ; oracle de primitive et replay R6 ciblé attendus. Aucun résultat R6 anticipé. |
| L1 — crash entre Created et marqueur | Limite conservée : un crash worker dans cette fenêtre peut laisser Unknown et sa réservation de quota. La fermeture du bootstrap ne justifie pas un redispatch. Cette limite doit figurer dans le reçu de livraison. Aucun scénario de zombie Linux n’a été exécuté ; l’analyse du reviewer est une inférence, pas une preuve de recette. |

Converge3 reste un verdict historique correct sur la source examinée avant D1. Le correctif D1 appartient à la phase de revue/audit, avec réattestation ciblée après correction. Aucun Converge4 ni nouvelle ronde complète n’est demandé. Les **44/44 tâches planifiées restent figées**, SHA-256 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea ; ce compteur ne vaut pas gate final. **Pipeline In Progress : D1 en cours, G8/G9 ouverts.**

## D1 — R6 réussi, transition corrigée après relecture, R7 attendu

R6 donne **29 lib PASS et 4 E2E PASS, zéro échec, quatre helpers ignorés, RC0**, en 59,578 s au total. Les journaux donnent 2,38 s et 30,08 s de tests. Le reçu atteste sources_unchanged=true, test-support, umask 0077 et TMP canonique /private/tmp/bq0davs1t. Commandes exactes, binaires et manifest sont conservés dans les preuves suivantes :

| Fichier | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.json | 65c686b7bd5b095552889a99e991e0b2aae9db8479572b78a77a31dd93e98358 |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.log | 84e3577dc29263bdb20eaae36fbc466cc5db9efb18e4c700bff0fa2bd8d2a2dd |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-release-r6.sources | fcae9dbbe088fd0b05c09836d33e185ff9a12c1248d1d9817b6206b978b57197 |

Le correctif réemploie release_retained : RunningManagedChild et marqueur restent disponibles en cas d’erreur EPIPE, write ou flush, et l’API release historique reste compatible. Le worker conserve le handle avant set_status_nonblocking et avant stockage. L’état transitoire du canal en erreur empêche une lecture bloquante ou un faux NativeReady en cas d’échec de stockage ; l’arrêt attesté est demandé avant persistance. Aucun nouveau store ni framework.

La lecture indépendante a confirmé la conservation du handle, puis trouvé que native_status refusait failed=true lorsque la phase était ReleaseAuthorized. ROOT a vérifié et fait corriger cette garde et l’oracle existant. **R6 reste historique sur ses sources ; le replay exact R7 worker162_marqueur_et_permis_precedent_release est en cours, sans résultat anticipé.** D1 reste ouvert avant G8, en attente de cette preuve et de la relecture finale ciblée. Les 44 tâches restent figées ; aucun Converge4, test de confort ou nouvelle FULL.

Le budget temporaire de rétention du cache passe de 45 à 46 Gio pour R7, afin de conserver le profil debug observé à 44,69 Gio et encore utile à 163. Il s’agit d’une règle de rétention, pas d’un plafond strict ni d’un objectif de consommation. Aucun gain physique n’est revendiqué. G8/G9 restent ouverts.

## D1 clos — replay R7 et relecture ciblée conformes

ROOT autorise la clôture de D1 après **R7 : 1 PASS, zéro échec/ignoré, RC0, 9,804 s au total**. Compilation 7,95 s selon ROOT ; le journal donne 0,15 s de test. Le reçu atteste sources_unchanged=true, umask 0077, TMP canonique et rétention temporaire 46 Gio.

L’oracle exact est remote_worker::tests::worker162_marqueur_et_permis_precedent_release. La commande conservée dans le JSON utilise nice 10, scripts/build.py, cargo test --locked --offline --jobs 2, bridget-daemon, test-support, --lib, --exact et deux threads. Le rédacteur a relu les preuves et vérifié les deux empreintes de source, sans exécuter de test.

| Preuve ou source | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-transition-r7.json | efadfd309186e1c99fad5c57f9ac1d8b8cb509eebf83ae9d7c2b5b4a6ff183ba |
| /Users/moi/.cache/bridget-delivery162-20261013/REPLAY162-D1-transition-r7.log | a037aaeed1ef85ae70024782960df75632b538baa4fa66f1f23eb107e46ed48d |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/remote_worker.rs | 3ee35d1a54376f40891c2b8d9f4eb763d1d1a2726076ca936471d584721fb2d6 |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/crates/bridget-daemon/src/managed_process.rs | 1d4937145011edc55ff9875a83f25998dd4a000d7a962b526237d1116d040c83 |

La relecture indépendante ciblée confirme D1 résolu : le handle est conservé ; failed=true est admis en ReleaseAuthorized avec marqueur, sans promouvoir la phase. failed=false reste interdit dans cette situation. ROOT rapporte également diffcheck RC0. R6 reste la preuve des primitives et des quatre E2E sur les sources avant la garde finale ; R7 complète cette preuve pour la transition corrigée. Aucun nouveau passage complet ni Converge4 n’a eu lieu.

Les gates de test et de revue propres à D1 sont clos. Le verdict R2 APPROVE_WITH_CHANGES est conservé avec cette réponse ciblée, sans le renommer APPROVE. **44/44 tâches planifiées, SHA tasks inchangé 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea.** Le scoring et le validateur d’audit sont encore en finalisation : aucun grade anticipé. G8/G9 restent ouverts, aucun succès global ni livraison déduit. La limite L1 reste applicable.

## Gate d’audit final conforme — 13 octobre 2026

L’audit final après correction D1 reçoit **A, 99,095238/100**, avec **0 CRITICAL, 0 HIGH, 3 MEDIUM et 1 LOW**. Le validateur termine avec **RC0, zéro erreur et zéro warning**. D1 est corrigé et réattesté par R6/R7 ; les trois MEDIUM et le LOW du rapport restent ouverts.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-31).

Reçu relu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/audits/2026-10-13/session-2026-10-13-spec-162-01/validation-final.json. SHA-256 : 85884560f21e2c4029d98ad6bfeab479df676ca843b9d7ec8432ae51e6f00170. Le reçu date de 2026-10-13T12:56:07.662381+00:00. Les empreintes de grade.json et coverage.json ont été vérifiées conformes à ce reçu : e68398193480b829418ab9349adb8192ec8ef7bc5875c50c9156afd87e851bad et df5fa5b02f4822f5ab449f2f4769004dccbf7ce17878cb487ed7273bcb570694.

**Le gate d’audit est clos ; G8/G9 restent ouverts.** Le reçu indique explicitement runtime_delivery_approved=false. Git, installation, activation et nettoyage relèvent encore de ROOT. Les 44/44 tâches planifiées restent figées ; aucune livraison n’est déduite du score ni du validateur.

## Réouverture après recette G8 — arrêt incomplet, 44/46

La recette d’annulation réelle a révélé un défaut sur la tâche **9fb66659-3ae0-481d-9b0a-335261a000f4** : cancelled/cleanup_done et reçu worker Terminal alors que codex PID355934 et sleep PID355935 restaient vivants, naissance141506025 et PGID355934. Le wrapper PID354553 avait disparu. Les descendants étaient encore observés après environ158 secondes puis ont fini naturellement ; aucun signal correctif n’a été envoyé par l’observateur. UID/SID/namespace de ces processus n’ont pas été capturés. Le statut métier n’atteste donc pas ici l’arrêt complet.

Preuve : /Users/moi/.cache/bridget-delivery162-20261013/recipe162-cancel-observer-failure.json, SHA d77ad9ca5de6111cb462cb019757dee720291e701bedcfb415d5378e4921edea. Cause retenue : le suivi du groupe du wrapper ne couvre pas les groupes distincts créés par la sandbox. Le correctif garde le wrapper Linux comme subreaper jusqu’à récolte de ses descendants et publication d’une preuve durable de drainage ; le worker exige cette preuve avant Terminal après RELEASE possible.

Le plan /Users/moi/.cache/bridget-delivery162-20261013/cancel162-fix-plan.md est relu, SHA c5f9e91687bf84713807523e984789ad63826a836502727a78de53aee91e9d3b. La revue /Users/moi/.cache/bridget-delivery162-20261013/cancel162-plan-review-r1.md porte APPROVE_WITH_CHANGES. Les quatre points ont une réponse dans le plan final : pas de récolte concurrente des Child ; inventaire de tous les TID et prérequis Linux ; garde des trois branches Terminal ; preuve corrélée au marqueur exact (bootstrap exec du wrapper). ROOT a arbitré un unique thread temporaire de signalisation sans récolte pendant stop/Drop, puis récolte après retour des propriétaires de Child. Ces réponses sont des décisions de plan autorisées, pas une preuve de correction ou de test réussi.

Prérequis réels : Linux6.8.0-142-generic, /usr/bin/bwrap0755 uid0 sans setuid/setgid, workerUID1002. Preuve /Users/moi/.cache/bridget-delivery162-20261013/cancel162-host-prerequisites.json, SHA ca34e067dd50a2d43c0fb935385d23ef62a65b7ae09f7ef1518c57e4fed45597. Le correctif doit refuser avant fournisseur si les primitives requises manquent.

T045 corrige ; T046 porte les oracles Linux, la recette réelle et la revue ciblée. **44 tâches validées conservées, deux tâches ouvertes, In Progress 44/46.** Aucun résultat de ces tâches anticipé. Si le wrapper meurt après RELEASE sans preuve, Unknown et la capacité peuvent rester retenus sans limite ; aucun outil force-clear ni arrêt prouvé inventé. Les trois routes nominales réussies restent acquises, mais **G8 demeure ouvert sur l’annulation, G9 reste ouvert**. Les convergences et audits antérieurs restent historiques sur leurs sources.

## Correctif annulation G8 — contre-revue de code R2 APPROVE, essais en attente

Revues relues : /Users/moi/.cache/bridget-delivery162-20261013/cancel162-code-review-r1.md (APPROVE_WITH_CHANGES, SHA52f4a98e0fcf2cd27d9ee8f2dd3b34b36193e828d89f62a224179a2b7ca7f093) et /Users/moi/.cache/bridget-delivery162-20261013/cancel162-code-review-r2.md (**APPROVE**, SHAd76da11c9d179f68e4815d9c2126af7ef82513bfe31242ef019c4ac8b62ee6e4). Le reviewer n’a exécuté aucun test, build ou service. Le verdict porte sur une relecture ciblée des quatre fichiers et du delta après R1, pas sur toute la base de code.

| Objection R1 | Réponse constatée par lecture R2 | Preuve d’exécution |
|---|---|---|
| M1 — descendant sensible à TERM non testé | Fixture enrichie normal/ignore ; normal doit atteindre Terminal sans libération manuelle, ignore conserve naissances et absence de .drained jusqu’à sortie coopérative. Primitive avec double-fork pendant le drain et barrière owner.join. | En attente des essais Linux. |
| L2 — enfant douteux arrête la passe | Erreur conservée, passage aux autres enfants puis retour Err ; inventaire illisible reste un refus conservateur. | En attente. |
| L3 — preuve lue avant sortie | Lecture .drained après sortie dans les trois branches running/abandoned/marqueur ; Unknown peut être réconcilié ensuite sans bypass de preuve après RELEASE. | En attente. |
| L4 — échec d’armement avant fournisseur | Limite conservée, explicitée ci-dessous. Aucun refus propre inventé. | Pas de succès opérationnel déduit. |

**L4 exacte :** l’armement subreaper/pidfd peut échouer avant création du fournisseur alors que RELEASE est déjà possible. Dans ce cas, l’absence de preuve mène à **Unknown avec quota conservé**, pas à un refus propre. Ne pas assimiler « avant fournisseur » à « avant RELEASE ».

Autres limites non bloquantes conservées de R2 :

- Si la synchronisation du répertoire échoue après rename, le retry identique peut retrouver la preuve et retourner Ok sans refaire cette synchronisation. La durabilité du renommage lors d’un crash dans cette fenêtre n’est donc pas garantie. Une preuve perdue laisse une issue conservatrice, sans nouvel écrivain ni succès d’arrêt inventé.
- Un Conflict persistant lors de publication maintient RemoteDrainGuard en boucle, le wrapper vivant et la capacité retenue. La revue relève l’absence de signal dédié. Ce point reste une limite, sans élargissement du correctif.
- Une sortie du processus ou un abort contournant Drop ne publie pas la preuve : Unknown. La recherche ciblée du reviewer ne prouve pas l’absence de toutes les voies de sortie dans l’ensemble du programme.

Le premier runner Linux R1 s’est arrêté avant compilation sur la garde du binaire. Le runner R2 compile actuellement. **Aucun test Linux réussi n’est encore attesté à ce checkpoint.** R2 APPROVE ne coche ni T045 ni T046. État courant In Progress44/46 ; G8/G9 ouverts, aucun verdict de convergence supplémentaire.


## Checkpoint T045 — validation Linux, 13 octobre 2026

T045 est validée ; le compteur passe à 45/46, statut In Progress. T046 reste ouverte pour la recette réelle après installation. Les essais suivants n’ont installé aucun binaire et n’ont modifié aucun service. Le binaire actif est resté identique.

- R2 : deux tests de bibliothèque PASS. La compilation a réussi en 224,200 s. Le premier E2E a ensuite échoué avant le fournisseur, car la fixture sélectionnait Git 2.43. Le RC global 1 reste historique. Reçu : /Users/moi/.cache/bridget-delivery162-20261013/validation162-linux-subreaper-r2/remote/validation-receipt.json ; SHA-256 ad75b4f6450924574a30acd187bd0d043c665deb7a33e178b00f06aa81c09693.
- R3 : fixture Git corrigée, cinq E2E exacts PASS, RC0. Compilation 27,394 s ; somme des essais 45,749 s ; durée totale 73,293 s. Sources inchangées pendant cette exécution. Reçu : /Users/moi/.cache/bridget-delivery162-20261013/validation162-linux-subreaper-r3/remote/validation-receipt.json ; SHA-256 236f61e5905ae777e9207d7d422db8e26470a816d45fd00c32e5a89a7bba4184.

Ces sept succès couvrent la primitive pidfd et la barrière Child::wait, la reprise aux frontières durables, l’arrêt avec descendant setsid et pipe hérité, les permis expirés, le contrôle de naissance avant signal, la reprise sans second fournisseur et l’admission réelle avec résultat unique. Le contrôle UID divergent est attesté par lecture du code ; aucun PASS runtime de ce cas n’est revendiqué. Les manifestes contiennent 3 310 fichiers : R2 1a7b79b98358f05e53588814ad06d19c47dcad64943c1f63ee0d94bd8bd095d0 ; R3 30f6bc27167c4c9c1cecddbcc78585dfdc2e8b2314dedbe017c7798d88f4615d. Le Build ID 611bfa7557b9-dirty identifie le snapshot de validation, pas un commit de livraison.

La contre-revue ciblée R2 est APPROVE. L’annexe d’audit conserve 0 critique, 0 élevé et 1 faible ; elle ne recalcule pas la note globale historique. Son validateur d’artefacts est RC0, sans erreur ni warning. Le complément Git confirme une correction de fixture seulement, sans abaisser la garde produit. Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/audits/2026-10-13/session-2026-10-13-spec-162-01/annexe-cancel-T045-T046/artifact-validation.json et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/audits/2026-10-13/session-2026-10-13-spec-162-01/annexe-cancel-T045-T046/addendum-fixture-git.json.

Les limites conservatrices restent applicables : un échec d’armement après RELEASE possible conserve Unknown et la capacité ; la fenêtre d’échec fsync après rename reste documentée ; un conflit permanent peut maintenir le wrapper vivant. Aucun nouveau test global, succès de recette installée ou nettoyage final n’est déduit.


## Checkpoint final T046 et G8 — 13 octobre 2026

ROOT et l’observateur ont validé l’annulation réelle après installation du correctif 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437. La tâche e2a5d605-063b-4103-b88b-a9c6b247240a est cancelled, résultat null, cleanup_done=true et error=null. Les huit PID observés sont tous absents à 15:58:36,355475 UTC (17:58:36,355475 CEST), avant la fin naturelle prévue du sleep à 15:59:17 UTC. La commande unique avait commencé à 15:56:17 UTC ; la fermeture est intervenue à 15:57:45 UTC. Cette observation solde le défaut réel de descendants survivants.

Les preuves corrélées transmises par ROOT sont conservées avec leurs empreintes :

| Preuve | SHA-256 et résultat |
|---|---|
| .drained exact | f005b5dfae73a0d1da3bb9cc60cc3b75a8d440bc5a7f0266d12ff3fe41e06a99 ; preuve de fin du drainage corrélée à l’exécution. |
| Reçu terminal | c83d05cd282f9cdbd46422979ef10832d19e39011356c1cc4fdb440c8abbae40. |
| Slot résultat | 50f41beea096f645d3606095e9ba555f751080b4fcac2afb84c9ac0664960b5c ; producer_terminal=true, clôturé pour cancelled, disposition=null, finalisé à 15:57:46 UTC. |
| Journal | a7a69bb0b630d75d9a668a3cde0e158a230d5271efdad0aea80bfa67e509a142 ; 19 événements, marqueur de recette non émis. |

Le maître n’a plus de tâche active et les admissions sont ouvertes : /Users/moi/.cache/bridget-delivery162-20261013/cancel162-rerun-active-after.json porte total_active=0, items=[], halted=false. Le parent Opus de recette a été arrêté proprement par la commande e27086a3-3a46-45da-aaca-7862f4e61a66, puis décommissionné par 13c264d9-7bc1-428f-9476-896fcd037d3f ; son historique reste conservé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/162-execution-distante/validation/tests162.md#extrait-32).

T046 est cochée : 46/46, statut Implemented, G8 validé. Les deux tests de bibliothèque et cinq E2E Linux, la contre-revue R2 APPROVE et l’annexe d’audit sans constat critique ou élevé restent les preuves ciblées du correctif. ROOT a relu le rapprochement code/documents/tests sans nouvel écart retenu. Ce checkpoint ne remplace ni la FULL historique ni l’audit global ; aucun GREEN global n’est revendiqué. G9 reste ouvert jusqu’au nettoyage final exécuté par ROOT.
