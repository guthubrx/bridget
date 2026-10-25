# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/implementation.md](../../../../../../specs/147-panneau-bridget-vivant/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Deux besoins validés : abonnement automatique du panneau et sélection mémorisée par conversation T3. Le principal a lu puis validé spécification, plan, audit de réutilisation16/16 et Analyze16FR/9SC. Il a donné le GO aux lanes Rust/RPC/Web après intégration des corrections de revue locale.

<a id="extrait-2"></a>

## Extrait 2

- Bridget : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant ; branche session-147-panneau-bridget-vivant ; HEAD fa4f9ad936c06ec7ed3f34515c5ae86a06564d09.
- T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant ; branche session-147-panneau-bridget-vivant ; HEAD0bd1e7a52ced2d03f3e71e7dc7569ac16e1927a3.
- Ces HEAD sont des bases Git, pas une empreinte des fichiers modifiés pendant le travail147.
- Lane Rust : crates Bridget, protocole/garde/bus/client/CLI/tests. Lane RPC : contracts/server/client-runtime T3. Lane Web : rightPanelStore et BridgetPanel T3. Chaque lane garde ses fichiers ; une même lane ne lance pas deux writers d'un module.
- Documents : dossier SPEC147 et blocs AGENTS/feature locaux. ADR du canal humain autorisée explicitement au propriétaire documentaire.
- Les racines142/145, données actives et programmes de mission ne sont pas des cibles. Aucun script de cleanup de worktree ou lancement de service réel par le propriétaire documentaire.

<a id="extrait-3"></a>

## Extrait 3

T001 validée et cochée sur contrôle explicite du principal. Son checkpoint réel à23:39 heure locale atteste les PID Bridget 59975/61796 et T3 63000/63083 vivants, aux mêmes chemins, sans restart. Le diff de la racine Bridget préexistante est inchangé : SHA256 85e3093db98db8e369f2a3dd5ee16585777a310b938ed05d88ebb429009519b2. Les travaux préexistants dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.gitignore, /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify/feature.json, /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md et les compétences142 restent intacts ; aucun contenu de ces travaux n'est intégré ici. La racine T3 est propre, hash du diff vide e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855.

<a id="extrait-4"></a>

## Extrait 4

- Cible Cargo isolée présente : /Volumes/SD1TO/bridget-build-147-rust. Utiliser CARGO_TARGET_DIR exact et commandes Cargo directes. Le wrapper de nettoyage de compilation peut retirer des worktrees fusionnés hors périmètre ; ne pas le déclencher.
- Aperçu isolé présent : /Users/moi/.cache/t3-spec147-preview.457DX8. Cette existence ne prouve ni ouverture ni recette.
- Fixtures Rust : réutiliser spec145_human_view_tests dans daemon.rs et son vrai handle_connection/CLI. Aucun nouveau harness098 avec kill de groupe.
- Raccords T3 : tests réels server.test.ts filtre Bridget et state/orchestration.test.ts ; DOM synthétique distingué du service qui doit exécuter un binaire CLI147 réel.
- Environnement : Node26.9.0, manifest racine T3 ^24.13.1. L'installation et les tests ciblés fonctionnent malgré le warning engine. Pas de correction globale de Node nécessaire à ce stade ; la compatibilité de build reste à vérifier séparément.
- Les namespace/socket/identités exacts de fixture et commandes de lanes seront enregistrés après leurs exécutions. Ils ne sont pas inventés dans ce journal.

<a id="extrait-5"></a>

## Extrait 5

T002 préparation validée et cochée par le principal, sans valider les futures recettes. Installation isolée : `pnpm install --offline --frozen-lockfile --ignore-scripts`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant, session terminal34983. Exit0 après6m20.2,1861 packages ajoutés,1846 réutilisés,1861 résolus,0 téléchargés. Pas de fichier log créé ; reçu tool uniquement. Fixtures et temporaires présents ne prouvent pas une recette runtime.

<a id="extrait-6"></a>

## Extrait 6

- Rust protocole : `CARGO_TARGET_DIR=/Volumes/SD1TO/bridget-build-147-rust CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 /Users/moi/.cargo/bin/cargo test -p bridget-transport spec147 -- --nocapture`, cwd Bridget147. Exit101,0 passed/1 failed/297 filtrés ; capacité human_thread_watch_v1 inconnue. Compilation1m22, test0.00s. Tool session96778/chunk f713c5 ; pas de logfile. L'heure exacte de fin n'a pas été lue ;21:40:29 UTC est l'heure de transmission ultérieure, pas celle du RED.
- RPC : `./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts apps/server/src/auth/RpcAuthorization.test.ts apps/server/src/bridget/BridgetReader.test.ts`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant. Trois fichiers failed,7 failed/78 passed,85 total ; runner Start23:38:15, durée980ms. Tool session44257 ; pas de logfile.
- Web : `../../node_modules/.bin/vp test run --project unit src/rightPanelStore.test.ts src/components/BridgetPanel.test.tsx`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web. Runner23:38:50, durée1.67s ;5 failed/95 passed,100 total. Store71=68 pass/3 failed ; panel29=27 pass/2 failed. Défaillances : sélection/store/migration absents, choix hors page1 perdu et100 signaux ignorés. Tool session40308/final chunk474c8f ; pas de logfile. La première tentative de démarrage unit depuis la racine n'est pas un RED métier.

<a id="extrait-7"></a>

## Extrait 7

Complément Web comparatif : six tests147 lancés contre BridgetPanel HEAD146. Commande `../../node_modules/.bin/vp test run --project unit src/components/BridgetPanel.test.tsx -t 'revalidates replacement|keeps the choice after|isolates remembered|cancels reads and masks|masks content during disconnect|releases its watch'`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web. Runner23:47:26, durée1.72s ;6 failed/29 skipped,35 total. Tool session73501/chunk494969, sans logfile. Le propriétaire atteste la restauration immédiate du code147 par apply_patch. Ces tests ont été ajoutés pendant un GREEN partiel, puis comparés à la baseline : ce n'est pas une preuve RED antérieure à toute implémentation. Ils montrent notamment137 lignes incohérentes au lieu100 pendant la course page2/page3, l'effacement du choix au refus de binding, et le corps maintenu lors d'une déconnexion.

<a id="extrait-8"></a>

## Extrait 8

RPC : `./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts apps/server/src/bridget/BridgetReader.test.ts apps/server/src/auth/RpcAuthorization.test.ts packages/client-runtime/src/state/runtime.test.ts packages/client-runtime/src/state/orchestration.test.ts --testTimeout=10000`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant. Cinq fichiers et122 tests passent. Runner23:48:04, durée1.40s, session60055, aucun logfile. Les tests couvrent dix lectures RPC, dix libérations, nouvelle génération, reprise transport et refus métier terminal. Un enfant OS réel, exécutant un script synthétique privé, est annulé et devient isRunning=false. Cette preuve est distincte du raccord réel au binaire CLI147 et au daemon, encore attendu.

<a id="extrait-9"></a>

## Extrait 9

- Le ready premier sur le wire ne garantit pas un rendu DOM distinct. Le runtime garde event/readyGeneration/subscriptionId dans un mémo volatil. visitId local unique isole la subscription d'une visite ; RPC garde ses deux champs threadId/projectId.
- Le WebSocket T3 peut rester sain quand le stream CLI/daemon tombe. Le supervisor reste chargé du WebSocket. Le seul stream CLI utilise Schedule existant, au plus trois reprises après l'initiale, uniquement unavailable/command_failed/timeout. Corps masqués et UUID gardé pendant l'attente ; ready neuf revalide. Après plafond : message et refresh manuel. Aucun retry business invalid_output/version/projet/binding/demande invalide.
- Plus de50 nouveautés peuvent séparer la tête du segment ancien. Relire les pages intermédiaires jusqu'à l'ancre puis le segment consulté sous S commun, publication atomique. Coût O(nouveautés + pages consultées), pas une lecture de toute la base.

<a id="extrait-10"></a>

## Extrait 10

Web :111 tests passent,72 store et39 panel, runner2026-10-09 00:00:04,4.00s. Commande `../../node_modules/.bin/vp test run --project unit src/rightPanelStore.test.ts src/components/BridgetPanel.test.tsx`, cwd /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web ; session82764/chunkf944a0. Typecheck du même cwd `../../node_modules/.bin/tsc --noEmit --pretty false`, exit0. Ce GREEN précède US5 ; le contrôle visuel et lint finaux restent distincts.

<a id="extrait-11"></a>

## Extrait 11

Interop réelle : `TMPDIR=/tmp BRIDGET_SPEC147_T3_PROGRAM=/opt/homebrew/bin/node BRIDGET_SPEC147_T3_SCRIPT=/Users/moi/.cache/t3-spec147-rpc-interop/reader-interop.mjs /Volumes/SD1TO/bridget-build-147-rust/debug/deps/bridget_daemon-21678d889d786fbb spec147_t3_service_real_cli_interop --ignored --nocapture`, même cwd Bridget :1 pass,0 fail,1084 filtered, exit0,0.94s. Marqueur SPEC147_INTEROP_READER_OK ready changed list_recent show history_recent, stderr vide. Vrai handler/socket/CLI147/Reader typé ; seul ProjectionSnapshotQuery est synthétique. Aucun claim WS ou DOM réel. Le namespace privé exact n'a pas été imprimé à cette première exécution ; il reste à consigner au prochain run. Aucune table active ni notification DeliverT3 ; aucun watch de fixture retenu après la publication de test selon propriétaire.

<a id="extrait-12"></a>

## Extrait 12

- Rust : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/audits/2026-10-08/session-2026-10-08-spec-147-rust-01.
- T3 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/audits/2026-10-08/session-2026-10-08-spec-147-t3-01.

<a id="extrait-13"></a>

## Extrait 13

T3 :136PASS contrats/Reader/auth/runtime, WS1PASS/210skipped, Web116PASS. Typechecks serveur/runtime/Web RC0. Lint ciblé RC0,25 alertes préexistantes server.test. Builds serveur1745ms et Web62.1s RC0 ; Node26/engine24 et chunks>500KiB restent des avertissements explicites.

<a id="extrait-14"></a>

## Extrait 14

Deux audits v14 diff : Rust6 fichiers source, T3 17 fichiers source/test. Cycle1 historique :Rust5 constats corrigés, T3 trois corrigés ; HIGH confirmés conservés. Cycle-scoring readonly :zéro résiduel, A100 limité au diff, pas certification globale. validate_session.py :RC0,0 erreur/0 warning pour chaque session. Aucun scan CVE, benchmark128clients ou profilage production revendiqué. T025/T026/T030 restent ouvertes jusqu’au gate final du principal ; aucun commit/push/install/restart.

<a id="extrait-15"></a>

## Extrait 15

Complément refresh reçu du propriétaire : test spec147_native_t3_refresh_publishes_and_revokes_real_os_processes dans t3code_identity.rs,1PASS/0FAIL1106 filtrés0.11s, compilation17.73s ; régressions t3code_identity22PASS0.41s et mcp_identity14PASS0.36s. Refresh réel, DB privée provider_session_runtime, deux enfants OS neutres/PID/birth/lineage, publication/résolution NativeTree, ambiguïté refusant/révoquant, retour unique republiant et live vide retirant. SQL inchangé/proofcredential préservé. Deux enfants nettoyés par EOF et namespace privé supprimé ; format/diff RC0, aucun nouveau clippy reçu pour ce complément.

<a id="extrait-16"></a>

## Extrait 16

Limites séparées : tuple live T3 et credential de protocole synthétiques dans la fixture refresh, runtimePID égal au processus de test. Aucun HTTP/T3 SDK/modèle réel ni autorité daemon déduit de ce complément. L'autorité aux vraies credentials vient du test MCP distinct. Ne pas joindre ces deux preuves pour annoncer une chaîne produit E2E complète. Relecture locale du complément en cours ; T040 reste ouverte jusqu'à son verdict.

<a id="extrait-17"></a>

## Extrait 17

US9 reçus canoniques Web : RED5 métier/44skipped49total à01:17:06/3.06s puis121PASS72store49panel à01:19:21/1.59s ; tsc/lint/fmt/diffRC0. Build6143modules/30.12sRC0 avec chunks>500kB nonbloquants. Ces reçus deviennent pré-correction FR31 ci-dessous ; aucun cochage T051–T054 ni recette finale anticipé. Audits frais du09/10 Rust/T3 délégués aux reviewers ; initiaux08/10 historiques, pas étendus artificiellement. Mapping31FR/20SC/9US préparé,43/54 cochées. Converge attend nouveaux gels/reçus/audits/GO.

<a id="extrait-18"></a>

## Extrait 18

Audit Rust neuf du09/10 livré puis relu/revalidé par principal : A,0findingactif, validatorRC0/0erreur/0warning. ScopeUS6/US8/T040,8fichiers/hunks seulement, pas tout147 ni grade initial réutilisé. MEDIUM TEST-001 borneCLI corrigé conservé avec fingerprint8b1d5922a379, sans timeout injecté. Aucun test/build/prod exécuté par reviewer. Audit T3 frais et QA native restent attendus ; tasksSHAinchangé,43/54. Contrôle isolation principal vérifie ASAR/release/diffroot identiques (hashesexactsdansresults), aucune installation/commit/restart.

<a id="extrait-19"></a>

## Extrait 19

Audit Rust frais A/zéro actif limité aux extensions US6/US8/T040 ; audit T3 frais A99.86/zéro actif avec MEDIUM UX-001 corrigé conservé. Les deux validateurs passent RC0/0erreur/0warning, relus/revalidés par principal. Ces grades ne réutilisent pas le premier lot et ne certifient pas les dépôts complets.

<a id="extrait-20"></a>

## Extrait 20

Recette US9 finale après fix FR31 : /Users/moi/.cache/t3-spec147-preview.457DX8/us9-final-native.json ; captures /Users/moi/.cache/t3-spec147-preview.457DX8/us9-final-dark360.png et /Users/moi/.cache/t3-spec147-preview.457DX8/us9-final-light360.png. Le principal a lu le reçu et vu les captures. Panneau CSS360px, titre et3 membres sur327px sans débordement, clavierSpace/focus stable, sélection unique/pagination/A→B→A103, normal↔épinglé mêmes nœuds, détails/article/dépliage/copie1224 exacte préservés sur resync/refresh ; panne/refus masquent lignes/détails/messages. Header h2 inférieur mesuré0 dans la nouvelle session. Viewport CSS1843×1152 et captures1280×800 :360px n'est pas un viewport mobile. Composants/CSS/store natifs réels, frontière RPC/données/presse-papiers synthétiques ; aucun E2E transport/provider/modèle complet déduit.

<a id="extrait-21"></a>

## Extrait 21

L'utilisateur autorise désormais commit, fusion, push, installation et redémarrage T3. Le principal exécute cette livraison. Développement54/54 et preuves précédentes conservés ; aucune installation/activation encore revendiquée. Pas de PR T3, pas de push vers son upstream.

<a id="extrait-22"></a>

## Extrait 22

Repos et destinations vérifiés en lecture seule : Bridget /Users/moi/Nextcloud/10.Scripts/64.bridget, worktree /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant, remote github https://github.com/guthubrx/bridget.git ; T3 /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local, worktree /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant, remote fork https://github.com/guthubrx/t3code.git. Le remote origin https://github.com/pingdotgg/t3code.git n'est pas la destination de push.

<a id="extrait-23"></a>

## Extrait 23

Sauvegardes prévues : application T3, binaire Bridget et petite DB Bridget ciblée, volume contrôlé ; aucune copie de DB T3. Chemins, hashes, commits fusionnés/poussés, installation, redémarrage et smoke production à inscrire seulement après reçus du principal. Aucun cleanup142/145 ni d'autres travaux.

<a id="extrait-24"></a>

## Extrait 24

Bridget installé/actif : releaseSHA25691cdc1e1d6c210c754cb5603011269a202a97474fb3c68351a518cbe6e7e37f5, buildid82f4abebea9a sans dirty ; daemon93353/pont93355, online30agents. Schema3/quickcheckok, human own-inspect list PASS et watchready seq0 puis fermeture sans réveil. Backup146CLI et DB Bridget arrêtée/quickcheckok : /Users/moi/.cache/bridget-install147.VTg5jQ. Aucune sauvegarde DB T3.

<a id="extrait-25"></a>

## Extrait 25

T3 candidat /Users/moi/.cache/t3-spec147-package.LIkn22/staging/T3 Code (Local).app, version0.0.45-local.147/commitf4354fb0d5be ; ASARa1ae5da6e46a3f8dca1f77dc6eefaf9fae2e9695e1b06850770500773808cee2,2413PASS/0FAIL et codesign deep strictRC0. PreflightRC0, rollback146 copieASARa43df7c98b17e1a0465821fff103049e3a7ed33b221f153e2a413b8fe2a624a2/signature stricte vérifiée. Activation T3 pas encore exécutée selon reçu principal ; seul /Users/moi/.cache/bridget-install147.VTg5jQ/t3-activation-result.json fera foi après restart autorisé. Aucun smoke macOS déduit du packaging. Logs et chemins builds préservés dans results.json.

<a id="extrait-26"></a>

## Extrait 26

WT/branche Bridget147 encore présents au checkpoint01:56 : retrait seulement après commit/fusion documentaire du principal. Ce paragraphe conserve l'état antérieur, remplacé par le checkpoint final ci-dessous. Livraison commencée vers01:37CEST, preflight01:49. Le reçu durable /Users/moi/.cache/bridget-install147.VTg5jQ/t3-activation-result.json fera foi pour l'activation ultérieure ; aucun nouveau résultat modèle ni build source ajouté.

<a id="extrait-27"></a>

## Extrait 27

À2026-10-08T23:59:00Z, principal confirme WT Bridget147 retiré par Git normalRC0, comme WT T3 ; total2WT. Suppression de branche distante Bridget ensuite confirméeRC0, rootmain/github identiques e94e01b8. Cleanup terminé :14branches locales/11distantes fusionnées retirées,13anciens builds/7,075,405,824octets alloués supprimés. WT T3 allouait environ76.638GB sur ExFAT, dépendances et références, pas DBT3. Espace libre rapporté :8TB2 de183à255GiB, home de88à91GiB. Ces mesures ne constituent pas une somme exacte de gain récupérable.

<a id="extrait-28"></a>

## Extrait 28

DirtyBridget142/145/146, T3WT145/nonancestor139, unique snapshotDBT3, helpersagent_loop et SHA installéBridget inchangés/préservés. Audits archivés ROOT et tgz avant retrait. Reçu détaillé /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/147-panneau-bridget-vivant/validation/cleanup.json. T3 encore pending à ce checkpoint documentaire seulement ; aucun statut actif déduit tant que son reçu durable n'est pas reçu. Éditions limitées à ces trois fichiers SPEC147 dans ROOT, jamais AGENTS ROOT142, source ou Git.
