# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/tasks.md](../../../../../../specs/163-decision-admission/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

ROOT a transféré au second développeur Cartae, nommé DEV_WORKER_CARTAE163 dans ces documents, seulement /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs, /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs et /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs. DEV_CARTAE163 principal conserve daemon, protocole, store, registre, HTTP, admission, CLI/MCP, Lineage et intégration maître. Le second prépare les gardes/probe/oracles existants de T012/T013/T015 ; il transmet les signatures ou branches nécessaires au principal et n’édite jamais ses fichiers. Les deux partagent le même worktree distant isolé, avec frontières de fichiers exclusives. ROOT crée et atteste son checkout /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission. Le miroir documentaire est /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission. DEV_T3 consomme le wire figé ; ROOT doit créer et confirmer le binding du worktree prévu /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission avant tout code T3. ROOT rapporte le checkout Cartae de session créé/propre sur base DCFD et le rattachement rétabli ; le rédacteur ne les a pas observés par SSH. ROOT rapporte également le worktree T3 créé et vérifié sur source 174a9d04554758aacffb4aade588e61c810faf5a ; archiver son binding avant code. Les admissions des deux développeurs sont rapportées dans le journal ; elles ne prouvent aucune validation future.

<a id="extrait-2"></a>

## Extrait 2

- [x] T001 **ROOT** — Confirmer et archiver l’attestation ROOT du checkout Cartae de session sur base dcfd9bc29140457b1fa016c276f5a48933415a5c, namespace privé umask 077/TMPDIR court et empreintes des documents. Archiver le rattachement Bridget rétabli avant de lancer DEV_CARTAE163 ; aucun lancement présumé. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-022, SC-009, O6. Dépendance : GO plan ROOT.
- [x] T002 **ROOT** — Confirmer et archiver l’attestation du worktree T3 de session et son binding avant DEV_T3 ; préserver main, les travaux étrangers et T3 chargé. ROOT rapporte cette cible créée et vérifiée ; cette tâche documentaire ne coche pas son contrôle futur de binding. Chemins : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-019, FR-022, SC-007, SC-009. Dépendance : T001.
- [x] T003 **ROOT** — Figer responsabilités et campagne de commandes adaptées dans le journal : une compilation dans toute la flotte, jobs 2/nice 10/tests 2 ; aucun worker/socket/DB de production touché. Les tests seront groupés et aucune tâche cochée avant sa preuve. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-022, SC-009, O6. Dépendance : T001–T002.
## Phase 2 — Fondations bloquantes

<a id="extrait-3"></a>

## Extrait 3

- [x] T011 [US2] **DEV_CARTAE163** — Étendre le filtre local via target_resolver et les profils bornés : zéro candidat refuse, candidat unique sans décideur ; droits/mapping/capacité inconnus excluent, aucun quota ou authentification inventé. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-005, FR-006, FR-008, SC-002, SC-003. Dépendance : T010.
- [x] T012 [US2] **DEV_WORKER_CARTAE163** — Extraire les gardes de lecture partagées avec Prepare : binaire/modèle/effort, mapping/racines/droits/dépôt/base. Worktree futur attesté par prérequis seulement ; aucun prepare_workspace, fournisseur/bootstrap, tokenPrepared, canon Created ou slot. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs. Couverture : FR-005, FR-006, FR-007, SC-003. Dépendance : T011.
- [x] T013 [US2] **DEV_WORKER_CARTAE163 côté worker / DEV_CARTAE163 côté protocole** — Implémenter négociation additive par époque et probe groupé par worker sur canal existant. Ancien worker conserve Hello/Accepted v1 ; nouvelle variante envoyée uniquement sur capacité négociée valide, reconnexion invalide les faits. Le second édite uniquement le côté remote_worker et rapporte ses besoins de signatures au principal, propriétaire exclusif de protocol.rs. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs. Couverture : FR-006, FR-007, FR-019, SC-003, SC-007. Dépendance : T012.
- [x] T014 [US2] **DEV_CARTAE163** — Construire snapshot de candidats et IDs opaques depuis faits groupés ; séparer quota Fleet et limite fournisseur, revalider seul élu au puits162 sans fallback distant injecté ni reclassement après refus. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-006, FR-008, FR-016, FR-017, SC-003, SC-006. Dépendance : T013.
- [x] T015 [US2] **DEV_WORKER_CARTAE163 pour oracles worker / DEV_CARTAE163 pour raccord et exécution groupée** — Étendre la fixture worker existante et les tests de permissions pour zéro effet du probe, ancien pair, reconnection, huit profils, cwd futur et révocation avant admission. Observer compteurs processus/worktrees/slots et faits réels ; exécuter la campagne. Le second prépare seulement les oracles des fichiers worker/fixture attribués ; le principal possède native_delegation.rs et lance les contrôles groupés après stabilisation des interfaces. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs. Couverture : FR-005, FR-006, FR-007, FR-008, FR-016, FR-019, SC-002, SC-003, SC-006. Dépendance : T014.
- [x] T016 [US2] **ROOT** — Vérifier que les trois routes autorisées restent Bridget/Roadie/Opus2D, et que la classification n’utilise jamais Prepare ni n’élève les droits. Transmettre le wire figé et preuves de compatibilité au propriétaire T3. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/contracts/auto-admission.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-005, FR-007, FR-019, SC-003, SC-007. Dépendance : T015.
## US3 — Continuer sans décideur disponible (P1)

<a id="extrait-4"></a>

## Extrait 4

- [x] T030 [US5] **DEV_CARTAE163** — Publier Lineage list/show v3 avec résumé de décision nullable sous gardes existantes, sans endpoint/prompt/politiquedroits. Réemployer CLI/lecteur, conserver v1/v2 bytecompat et versionspages homogènes. Rapprocher la projection du mode durable défini à T023 ; Delegate162 historique garde decision:null si le profil est inconnu. Correction R3 mise en œuvre et contrôles ciblés attestés ; rapprochement final ROOT et gates de livraison ouverts. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/lineage_protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli_lineage.rs. Couverture : FR-019, FR-020, SC-007, O4. Dépendance : T029.
- [x] T031 [US5] **DEV_T3** — Consommer le wire figé dans schémas et BridgetReader existants : v3→v2 seulement unsupported_version structuré, v2→v1 unsupported_version ou invalid_request historiques. Aucunrepli timeout/auth/invalidv3 ; staginghomogène, journal/cancel inchangés. Chemins : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/server/src/bridget/BridgetReader.ts. Couverture : FR-019, FR-020, SC-007, O4. Préparation/codage et tests unitaires préparatoires sur fixtures autorisés après T004 figé, binding T002, copie/cache contrôlés et permis ROOT. La validation finale d’interopérabilité/runtime et le cochage attendent T030 ; ces essais autonomes ne soldent pas T031–T035.
- [x] T032 [US5] **DEV_T3** — Afficher choix/mode/raisontexte bornée dans infobulle existante ; aucun HTML/consigne ni écran/conversationglobal. Ancienreader sans résumé reste sans information de décision, pas explicite inventé. Chemins : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/SubagentTooltipContent.tsx. Couverture : FR-020, SC-007. Dépendance : T031.
- [x] T033 [US5] **DEV_CARTAE163** — Étendre/exécuter contrats Rust et CLI Lineage existants : v1/v2 stricts, v3 nullable, droits humains, résumé sanssecret, pagination homogène et aucun effet journal/cancel. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/lineage_protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/lineage_cli_test.rs. Couverture : FR-019, FR-020, SC-007, O4. Dépendance : T030.
- [x] T034 [US5] **DEV_T3** — Étendre/exécuter tests Reader/contrats/infobulle existants : codes de repli exacts, pages homogènes, absent/explicit/automatic/fallback selon la règle T023/T030, texte hostile et persistance via oracle Orchestrator existant autorisé. Contrôles TypeScript/lint ciblés adaptés, sans relancerT3 et sans copier un banc autonome. Chemins : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/server/src/bridget/BridgetReader149.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage149.test.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts. Couverture : FR-019, FR-020, SC-007, O4. Dépendance : T031–T032 pour code. Essais unitaires préparatoires autonomes sur wire T004 figé autorisés après copie/cache contrôlés et permis ROOT ; validation finale d’interopérabilité/runtime et cochage après T030. Groupe unique des cinq bancs existants, maxWorkers=2 et nice=10, puis format/lint des sept fichiers et typechecks ciblés ; correctifs et replays affectés seulement.
- [x] T035 [US5] **ROOT** — Comparer preuves Rust/T3 aux interfaces figées et aux22FR/9SC ; relire diff et simplifications, corriger uniquement contrôles affectés. Rapprocher mode/choix/raison de l’observationnative sansprésumer runtime T3 ancien. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/analysis.md. Couverture : FR-019, FR-020, FR-022, SC-007, SC-009. Dépendance : T033–T034.
## Phase finale — Validation, livraison et nettoyage

<a id="extrait-5"></a>

## Extrait 5

- [x] T036 **DEV_CARTAE163 + ROOT** — Exécuter Converge et la validation finale Linux regroupée prévue après succès des campagnes Rust/T3, format/lint/build/tests appropriés et replays affectés seulement. FULL unique seulement si gate applicable la prévoit ; conserver les échecs hérités et provenances sans GREEN global inventé. ROOT arbitre le permis compilation de flotte. Chemins : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/Cargo.toml ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Couverture : FR-007, FR-011, FR-012, FR-015, FR-018, FR-019, FR-021, SC-001, SC-002, SC-003, SC-004, SC-005, SC-006, SC-007. Dépendance : T035 ; avant clôture de Converge, rapprocher aussi T046–T057 et leurs preuves affectées, sans suite supplémentaire.
- [x] T037 **ROOT** — Faire contre-revue code et audit prévu des frontières droit/SQL/HTTP/crash/complexité/minimalisme/compatibilité, corriger findings bloquants puis replays affectés. Conserver couverture et limites honnêtes ; aucun outillage ou scénario confort ajouté. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/adversarial-review-claude.md. Couverture : FR-005, FR-007, FR-015, FR-019, FR-021, FR-022, SC-003, SC-005, SC-007, SC-009. Dépendance : T036 et rapprochement de T046–T057 avec leurs preuves affectées ; audit readonly frais du diff stable avant commit, phase fix non exécutée et prérequis dépôt propre/HEAD vert non attestés.
- [x] T038 **ROOT** — Après les gates de convergence/contre-revue/audit, ROOT crée un checkpoint source validé puis compile/vérifie Mac et ses tests OS spécifiques depuis le même commit source Rust attesté que Linux ; vérifier package T3 et signatures applicables. Publier provenance distincte, sans confondre essais Linux/Mac ; aucun processus T3 relancé. Si le build Mac révèle une correction, exécuter les contrôles affectés puis delta de revue/audit avant le commit source final ; aucune FULL répétée. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/target/release/bridget ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/desktop ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Couverture : FR-019, FR-022, SC-007, SC-009, O6. Dépendance : T037 ; permis de flotte libre.
- [x] T039 **ROOT** — Attester service/protocole/endpoint/hosting/accès depuis maître Mac et checkpoint exact ; retenir explicitement fixed/fallback/profils/préférences sans secret public. VérifierMINREQ_TIMEOUT absent. Préparer sauvegarde cohérente, garde de downgrade et quiescence ; laisser off si gate réelle manque. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/contracts/decision-service.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md. Couverture : FR-001, FR-002, FR-003, FR-005, FR-022, SC-001, SC-002, SC-008, SC-009. Dépendance : T038.
- [x] T040 **ROOT** — Exécuter recette réelle depuis maître Mac sur service/checkpoint retenus : huit descriptions maximales/template/bornes, choixautorisé puis admissionrésultatautorisée dans un maître/worker de recette isolés avant installation de production, indisponibilité/repli et annulabilité héritée. Vérifier troncaturepossible indépendammentusage.input_tokens. Conserver preuves synthétiques et limites ; aucun Jev attesté par son nom. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Couverture : FR-008, FR-010, FR-016, FR-017, FR-020, FR-022, SC-002, SC-003, SC-006, SC-007, SC-008. Dépendance : T039.
- [x] T041 **ROOT** — Intégrer/pousser les checkpoints source validés Bridget etforkT3, puis commettre les correctifs finaux prouvés si nécessaires, avec hooks habituels/identité inchangée ; vérifier SHA distants et provenancebuild. Aucun force ni signature interdite. Documenter source et documents et rollback compatible avec reçus163. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md. Couverture : FR-022, SC-009. Dépendance : T040.
- [x] T042 **ROOT** — Installer atomiquement binaires Mac/Linux et paquet T3 applicables signés depuis sources attestées. Activer daemon/pont/worker sous quiescence et sauvegarde si nécessaire, natifBridgetrestart ; ne pas remplacer états par une sauvegarde perdant de nouveaux reçus. Distinguer fichier installé/processus changé. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/target/release/bridget ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/target/release/bridget ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md. Couverture : FR-018, FR-019, FR-022, SC-006, SC-007, SC-009. Dépendance : T041.
- [x] T043 **ROOT** — Valider G8 opérationnel après installation : processus/provenance, accèsdécideurdepuismaître, configuration effective, reçu/décision/résultat durables et axes désactivables ; pas de GREEN dérivé de startup. Conserver rollback/preuves et statut exact T3 chargé. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md. Couverture : FR-001, FR-003, FR-018, FR-020, FR-022, SC-001, SC-002, SC-006, SC-007, SC-008, SC-009. Dépendance : T042.
- [x] T044 **ROOT** — Valider G9 nettoyage après merge/push : inventaire TMP/branches/worktrees/stages Mac et Cartae, preuves archivées et SHA et contrôle d’usage avant retrait sansforce. Retirer seulementlot inutile ; préserver DB/rollback/prérequis/processus et travaux étrangers, justifier les caches conservés. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission. Couverture : FR-022, SC-009. Dépendance : T043.
- [x] T045 **ROOT** — Clore tâches uniquement sur preuves préexistantes, journal/spec/gates etcheckpoint final ; commit/push des métadonnées puis vérifier HEAD distants/arbres propres/compteur 58. Livré seulement si G8/G9 prouvés ; reçu finale distingue limites et conservations. Chemins : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/spec.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/implementation.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/livraison163.md. Couverture : FR-022, SC-009. Dépendance : T044.

<a id="extrait-6"></a>

## Extrait 6

T001–T006 bloquent la chaîne Rust métier : US1 → US2 → US3 → US4 → US5 Rust → validation/livraison. Le codage T3 peut commencer après T004 figé et binding T002 confirmé, en parallèle du Rust sur fichiers distincts. Les essais unitaires préparatoires Reader/contrats/infobulle/store sur fixtures du wire T004 sont permis avant T030 après copie/cache contrôlés et attribution du slot ROOT. La validation finale d’interopérabilité/runtime et le cochage de T031–T035 attendent T030 ; ROOT sérialise les commandes de compilation. L’attribution initiale à un propriétaire Rust unique est historique. La préparation worker T012/T013 et ses oracles T015 peut avancer en parallèle des surfaces maître sur fichiers exclusifs ; le raccord et les contrôles groupés attendent les interfaces stables et les dépendances métier indiquées. Aucun marqueur [P] ne supprime ces dépendances. Le principal possède Cargo ; le second ne compile, ne teste et ne lance aucun lint sans permis ROOT. Aucun contrôle séparé par agent. Les essais préparatoires T3 T034 suivent leur code et le permis ROOT ; leur succès ne remplace pas le gate final T030.

<a id="extrait-7"></a>

## Extrait 7

Exemples par histoire : US1 registre puis admission ; US2 préparation extraction/worker par le second sur fichiers exclusifs, signatures transmises au principal puis raccord groupé ; US3 réseau puis santé puis adaptateur ; US4 reçus puis admission puis entretien/collecte ; US5 préparation/codage et essais unitaires T3 sur fixtures après T004/binding/copie/cache et permis ROOT ; Rust continue son code, Cargo reste suspendu pendant le slot T3. Validation finale d’interopérabilité/runtime et cochage après T030. Les lectures ROOT peuvent accompagner ces phases, sans mutation concurrente du même fichier.

<a id="extrait-8"></a>

## Extrait 8

Reste recommandé : 360–600 minutes, revue/corrections et delivery compris, hors indisponibilité de modèle ou quota et hors rétablissement du connecteur. Rust et Linux : 180–300 min ; consommateur T3 : 45–75 min partiellement parallélisable ; vérification/audit/recette : 60–105 min ; livraison/nettoyage : 60–90 min. Cette fourchette remplace une précision artificielle, sans multiplier le temps par nombre de tâches. ROOT recalera à partir des premières preuves et délais de compilation.

<a id="extrait-9"></a>

## Extrait 9

T001 : worktree distant /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission créé propre sur baseDCFD par Git/SSH ROOT ; rattachement vérifié par bridget_capabilities réussi, dev=true, Codex gpt-5.6-terra. Son cochage exige l’attestation avant lancement, pas le lancement du développeur. T002 : worktree /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission créé sur source174a, branche session163 attestée par ROOT. T003 : propriétaires et budget de flotte figés, détails dans le journal.

<a id="extrait-10"></a>

## Extrait 10

Preuves de limite pont avant/après : /Users/moi/.cache/bridget-delivery163-20261013/bridge-limit163-before.json et /Users/moi/.cache/bridget-delivery163-20261013/bridge-limit163-after.json. Pont2525 limite souple8192, limite dure inchangée ; T3 704/969, daemon71469 et LayaSSH81748 protégés. Aucun de ces faits n’est présenté comme observation SSH du rédacteur. Tous gates avant implémentation sont soldés ; les gates endpoint/profils/checkpoint réels restent différés explicitement à G8 et aux tâchesT039/T040/T043.

<a id="extrait-11"></a>

## Extrait 11

Cargo reste unique chez le principal ; le second n’exécute compilation/test/lint qu’avec permis, et les contrôles sont groupés après interface stable. Préparation T3 achevée, mais T030/interopérabilité et cochage final toujours ouverts. Au checkpoint historique du transfert du 13 octobre 2026, G8/G9 et progression 4/45 restent inchangés. Aucun développeur n’édite ces documents ; aucun nouveau scénario, tâche, outil, pipeline ou scope.

<a id="extrait-12"></a>

## Extrait 12

- [x] T046 **DEV_CARTAE163** — Corriger list/show v3 pour conserver aussi placement nullable comme v2, puis exercer les réponses réelles du store dans les oracles Lineage existants. Preuve du manque : crates/bridget-daemon/src/delegation_lineage.rs:845 et :950, placement seulement version==2 ; T3 exige placement en v3. Couverture : T030/T033, FR-019/FR-020, SC-007.
- [x] T047 **DEV_CARTAE163 + DEV_T3** — Rendre cohérente la négociation CLI/Reader réelle : une réponse rétrogradée par la CLI ne doit pas être rejetée comme invalid_output, ni permettre un repli non autorisé ou des pages mélangées. Conserver les bornes v3→v2 unsupported_version et v2→v1 unsupported_version|invalid_request, journal/cancel inchangés. Éprouver dans les bancs existants le raccord réel et les versions/list/show. Preuve : cli_lineage.rs:180–207 masque les refus ; BridgetReader.ts:503 et :606 impose la version demandée. Couverture : T028/T030–T034, FR-019, SC-007.
- [x] T048 **DEV_CARTAE163** — Appliquer les plafonds actifs128/owner16/rétention4096 avant une nouvelle réservation auto, atomiquement avec l'insertion et la purge sûre. Un rejeu précède ces gardes ; le remplacement reçu→Task ne consomme aucune place et reste possible à la borne. Étendre les oracles T026/T027 existants pour la réservation réelle, concurrence/collision, refus chosen et panne SQL/crash selon les scénarios prévus. Preuve : reserve_auto_receipt et insert_auto_admission_receipt_in écrivent sans garde de plafond ; seule admit_delegate compte après le choix, et son >=4096 compte le reçu propre. Couverture : T023–T027, FR-013/FR-014/FR-015/FR-021, SC-005.
- [x] T049 **DEV_CARTAE163** — Revoir les sorties du parcours réseau : un fallback configuré mais filtré ne doit pas laisser decision_started de l'incarnation courante ouvert indéfiniment ; confirmer le refus durable et le rejeu sans HTTP. Vérifier aussi l'ordre de libération du permis health→decide : le résultat est envoyé avant drop(permit), ce qui peut produire un faux capacity_occupied entre les deux échanges. Corriger si confirmé, puis rejouer les seuls oracles existants affectés. Couverture : T019–T021/T023–T027, FR-010/FR-011/FR-012/FR-013/FR-015/FR-017, SC-004/SC-005.

<a id="extrait-13"></a>

## Extrait 13

R7 admise selon ROOT : tâche 94799f7a-d849-43cf-aab4-6be90eb318bc, enfant 336d8ef8-795f-450f-950b-eeaa971aa3b5, native round 7. Permis Cargo ciblé, jobs=2/nice=10/tests=2 ; aucun replay du groupe bibliothèque 28, HTTP neuf, worker, T3 ou FULL. Admission seule, aucune correction ou preuve future cochée.

<a id="extrait-14"></a>

## Extrait 14

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

<a id="extrait-15"></a>

## Extrait 15

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-16"></a>

## Extrait 16

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-17"></a>

## Extrait 17

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-18"></a>

## Extrait 18

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-19"></a>

## Extrait 19

P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 restent conservés, sans P4 formel inventé. Les anciennes FULL avec échecs, l’audit163 partiel 3M/2L, le delta T058 partiel sans finding et le delta T3 R6 sans finding restent distincts ; aucun GREEN global déduit.
