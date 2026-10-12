

## Matrice de clôture158B — préparation pour vérification ROOT

Date 2026-10-12T12:57:31.081911+00:00. Cette matrice ne coche aucune tâche. P3 est CONVERGED code/contrat (7FR/3US), sans verdict de livraison. Contre-revueR2/clarificationdoc, agrégation/scoring/validateur et livraison restent des gates distincts. Les preuves ci-dessous ne transforment aucun RC101 en GREEN.

| Tâche | Preuve exacte acquise | Code/doc nécessaire | Limite ou gate restant |
|---|---|---|---|
| T101 | REF/REFJSON : 30 PASS historiques, sourcesA relevées ; FULL-SUMMARY :30/30 noms+cibles PASS | RéférenceA distincte des exécutionsB | Déjà cochée ; aucune baseline rejouée séparément. |
| T102 | SURFACE-RED0P2F ; TYPE-COMPILE8,58s/0assertion ; GROUP-COMPILE0assertion ; TYPE-GREEN2P ; GROUP-R1 G1domaine+SQL PASS ; FULL idem | Types/domaine/Store fermés | RED de surfaces réel ; RED des APIs absentes compile-only explicitement permis par la tâche. Aucun REDSQL16/128 inventé. |
| T103 | TYPE-GREEN2P19,40s ; GROUP-R1 G1domaine PASS ; FULL typesPASS | Protocole/FrozenWake/ déclarationmodule existante | Sous-ensemble serde distinct des bornes SQL. |
| T104 | GROUP-COMPILE0assertion ; DATE-RED G3 réel0P1F6,34s ; GROUP-R3 G3PASS ; GROUP-R1 G2PASS ; FULL | Calendrier/gel/restart | Filtre incorrect0test exclu ; aucun RED métierG2 distinct constaté, seulement compilation initiale. |
| T105 | GROUP-R1 G2PASS + variantes UTC/gap/fold/non-DST ; FULL ; IMPL bornes2881×8 | libc existant, calcul horsverrou/CAS | Mesures debug locales1,105791/1,747084/1,462834/1,94925ms ; pas latence service ni benchmark fournisseur. |
| T106 | GROUP-R1 G1SQL+G3PASS ; GROUP-R3 G3PASS ; FULL ; IMPL | Storewake additive, Immediate/CAS, bornes physiques | Reçus099/102 conservés ; pas preuve de crashprocessus réel. |
| T107 | GROUP-COMPILE0assertion ; GROUP-R1 G4/G5+G6FAIL ; WRITER-RED G5FAIL réel ; POST-RED G6FAIL ; FINAL-4 mêmesG4/G5/G6PASS | Même deux oracles regroupantG4–6 | Échecs FK/cléASCII de fixture gardés historiques ; pas crédités comme défaut métier. |
| T108 | GROUP-R2 G4/G5PASS ; FULL ; FINAL-4 G4/G5PASS ; P3 | tick1s+gardeScheduledWake/lineage/pauses | Pas autostart modèle ; garde native via preuves existantes ; replays finaux ciblés uniquement. |
| T109 | WRITER-RED1P1F37,52s ; FINAL-4 4P0F28,82s ; POST-GREENG6 ; P3 | 099 avantwrite,102commit/reprise et horloge fraîche | No099/noFrame après grâce ; outcome_unknown ne prouve pas réception. |
| T110 | SURFACE-REDwakeFAIL ; GROUP-R1 surfacewakePASS ; FULL ; P3 | Uniqueoutilwake + tableauScopes/daemon | MCP sans flags humains ; Code/contrat convergés, pas recette fournisseur. |
| T111 | GROUP-COMPILE0assertion ; GROUP-R1 quotaFAIL InvalidIssuerScope avant quota ; GROUP-R2 quotaPASS ; FULL ; DND-CLI G7PASS ; FINAL-4 G7/G6PASS | Quota durable + CLIheadless réelle/credential149 | RED quota60 comportemental distinct non constaté ; échec raccordScope réel nommé comme tel. CLIpositiveprouve réserve/canon ; G6commit/guards séparés. |
| T112 | GROUP-R2 oraclequota/rejeu/collision/restartPASS ; FULL ; POST-GREEN ; FINAL-4 G6aux commit102/099PASS | Réemploi registre099/horizon7jours/UUID102avant réserve | Pas nouveau registrequota ; tests de crashfixture ne prétendent pascrash réel. |
| T113 | DND-CLI G7vraieCLI PASS44,08s ; FINAL-4 G7/G6PASS28,82s | cmd_wake/client/daemon, credentialsRegistered réelles | Headless membre/projet : G6 handler réel ; CLIenfant privé admission/canon et absent/invalide0réserve/0post. Aucun faux token accepté. |
| T114 | STORE-RED0P3F57,83s ; SURFACE-REDmissedFAIL ; GROUP-R1 StoreG8/G9/G10PASS ; GROUP-R2 pipelinePASS ; FINAL-4 pipelinePASS | Troisgroupes8–10, aucun11 | T114-store-RED-runtime s’arrêtaitE0596 :0assertion. Premier REDmétier est STORE-RED. |
| T115 | STORE-RED G8/G9FAIL ; GROUP-R1 StoreG8/G9PASS ; FULL ; IMPL/P3 | Migrationsfaits/owners/gapglobal, bornes/CHECKBLOB | FR106docsalignées ; dernière gardeemptyBatch est relue, pas nouveau FULLpostfix revendiqué. |
| T116 | STORE-REDprovenanceFAIL ; GROUP-R1 StoreG8PASS ; GROUP-R2 pipelinePASS ; DND-CLI deuxanciensdiagPASS ; FINAL-4 pipelinePASS | Workerunique64/IObudget1s/deadline5s/once | Garde stockage ne recrée pas once ; Written ne prouve paslecture modèle. |
| T117 | STORE-REDG10FAIL ; GROUP-R1 StoreG10PASS ; GROUP-R2 pipelinePASS ; FINAL-4 pipelinePASS ; P3 | ÉpisodesRegister+DND/ACK099CAS/cutoffs | ACKOk mêmeanswered_requestNone, ancienowner/génération refusé ; pas touscrashesruntimevalidés. |
| T118 | SURFACE-REDmissedFAIL ; TYPE-GREEN ; GROUP-R1 surface+StoreG9/G10PASS ; FULL/P3 | ObservationRequestmissed/MCPReads/pagination | Reads ne supprime pas lesfaits ni n’avancecutoff ; pas reconstructeur ni ownerlibre. |
| T119 | Documents B présents et relus par P3/IMPL ; ledger testsappendfinal | Documentationnative, skill, commandes, quickstart | R2clarificationdoc annoncée prochaine : relecture/appliqueROOT requis avantcochagefinal. |
| T120 | ADRexistante B présente ; P3/IMPL alternatives/frontières | MêmeADR158, décisionsA conservées | Accepté = décisionarchitecturale, pas livraison ; R2doc/gates séparés. |
| T121 | FULL1356,44sRC10195cibles2456P12F74I ; FULL-SUMMARY30/30PASS,7enfantsexclus ; FINAL-4 et DND-CLI ; REPRISE-0991P1F15,85s | UNE FULL réellement exécutée puis replays affectés | Exigence exécution/comparaison acquise, pas GLOBALGREEN. Essai14,37scompile0test exclu ; aucun secondFULL ni30rerun. |
| T122 | IMPL SHA5ad3c129a834937cee0b124b1bdebb3ddc7f5733011777d6cec495cb36905fa0 ; lectureinitiale+deltafresh/T122N-P149 ; P3 | Rapport entier/delta normalisé/testssources et limites | GAPmatériel : validation/diff-158B.patch absent au contrôle. Le diffcache n’est pas son chemin demandé. ROOT doit produire/copy/check avantcochage. |
| T123 | POST-RED0P1F36,42s ; POST-GREEN1P0F43,11s ; G6ASCII PASSdansWRITER-RED ; FINAL-4 G6PASS | Recherchecanon/UUID102 avant grâce+gardes, clôture099 | Conserver canon/issued_at et countposts ; simulecrashfrontière par SQLfixture, pascrashprocessus observé. |

### Références des preuves

- REF : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/baseline-158B.md
  SHA256 : 7b132e7fff91152375fe46a65ffacb4479d3cf12a654a43f5ab3f07fc475e06f
- REFJSON : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/baseline-158B.json
  SHA256 : ea8df12809c71320787afa2f12be2872624aa1cb0ae18686c5edb6fd4417a970
- SURFACE-RED : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-T114-surfaces-RED.json
  SHA256 : 6a683fee252fb4b3f3b85718c93097139094f92191d2abd9f14c383742c3d9c5
- TYPE-COMPILE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-protocol-RED-compile.json
  SHA256 : 60430d057fa78465f23b8de2902106839f970709d650e6ec7218d4680b570f5d
- TYPE-GREEN : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T103-protocol-GREEN.json
  SHA256 : 95a13ee9177cdd217ae3027632f4d9ce6ab13fac5033deab6f486597ec1e5cbd
- GROUP-COMPILE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-T104-T107-T111-grouped-RED-compile.json
  SHA256 : 24dfc316066afe000432ed895a806fe13151cf51f5b2e0e9dd9d9353ae652e8f
- STORE-RED : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T114-store-RED-assertions.json
  SHA256 : 03ca3bde7d9d2c4ad7ead9b486e3ce1596362e5a21d8a38b08d4b98fcc97f5e8
- GROUP-R1 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T102-T118-grouped-GREEN-r1.json
  SHA256 : d6e2e187b71267c94dd08efd2646e40f5ef3e92da632d15d4f7372a05794ca8b
- GROUP-R2 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T107-T117-affected-GREEN-r2.json
  SHA256 : dda398768a5300f3943feab27ab7c6a1353cbac7afe9fca6bd0ab616f1090024
- DATE-RED : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T104-calendar-date-RED-exact.json
  SHA256 : 2d3fd942ba1ff1569990b2fe455449c03bded7f328d00f89a4a4f17b8f5b66f8
- GROUP-R3 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T104-T111-T113-G3-G6-CLI-GREEN-r3.json
  SHA256 : be29d9865e01a9a5b0b48bf49c642eb9538b66b72dd34fbc1ad25f52d8eb8d2c
- POST-RED : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T123-G6-postcommit-RED.json
  SHA256 : bea21e38b9855641c0ed02be2cf40002805d4b234030a26d7b82015576cc6ec7
- POST-GREEN : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T123-G6-postcommit-GREEN.json
  SHA256 : d7664c7de2e918b22f73c278f56e505235b60caf11afe47fcf0ba97320d04a4a
- WRITER-RED : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T109-G5-expiration-writer-RED-G6-ascii.json
  SHA256 : 1a322c5147121601c385bfc1af9e68aea9213b09cfe5ee64aa9ce5669c396dd1
- FINAL-4 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T109-departure-clock-GREEN-affected.json
  SHA256 : c82d96f3298b78db6a6f76a1640251a3d982b4ff579a5481c7f2890f1304ec32
- DND-CLI : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T111-T113-G6-G7-attested-T116-diagnostic-GREEN.json
  SHA256 : dd53b24bb8c27b2bc4a094c543ac1e40d0b89f215419221c96d929b603c2d057
- FULL : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T121-FULL158B-compiled-final.json
  SHA256 : ccac449dadc9ee930a863a0e8877bc3a2ae86146960def230c827d569178f491
- FULL-SUMMARY : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T121-FULL158B-summary.json
  SHA256 : 21a2d955cdf9c14f1d3ea2bc52df7ce379d5722b52ddd7c9f3f71844043ad50d
- REPRISE-099 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/T121-099-recovery-timing-replay.json
  SHA256 : ddccaa56cded02ca8d958daea96cf8e55b712ef99e1eb2d9324ac7208df1142c
- IMPL : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/implementation-158B.md
  SHA256 : 5ad3c129a834937cee0b124b1bdebb3ddc7f5733011777d6cec495cb36905fa0
- P3 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/converge158B-p3.md
  SHA256 : 44ee3b1283f1703665323cf18649f45da666373303ffb046c05c1d53acb06541

Sources documentaires inspectées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/docs/delegation-native.md
  SHA256 : 28e8192e00628333c201e799c7fab246ec3c57d84d6756f9f4884c9a27e12495
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/skills/bridget/SKILL.md
  SHA256 : 67e1ea7892a5cfec7d1d0ce387e98af0d0d7fee62aeac9452ea4c7cd731176e4
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/skills/bridget/references/commandes.md
  SHA256 : 01eaad725c9f0f30677ca7e4cf05fb5097c46db5b924edebd6a1fc84ed6cbb00
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/docs/decisions/158-controle-global-portees.md
  SHA256 : 8644f7e82d7de87771b1954f51f536f6216e70459adc1dc27d9f10d16f982307
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/quickstart-158B.md
  SHA256 : f09c6094ad7e24637d9fb43f2bb384a83e1c607b9059ae1bfce3e32a527ef250

Limites transversales : les 8échecs historiquesA restent ; les 2reprises099 byte-identiquesA ne sont pas effacées (replay1PASS/1FAIL). Les 2régressionsDND passent leurs replays. Les dixgroupes ont leurs preuves réparties entre reçus ; ne pas additionner testsrépétés ni enfants. Pasnouvelle recette/modèle/benchmark/Cargo/service/Git par cet owner. Pasnouveau test recommandé. T102/T104/T111 conservent explicitement la nature compile/raccord de leurs premiers RED ; ROOT tranche leur clôture avec ce niveau réel de preuve.

GAPactuelT122 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/validation/diff-158B.patch absent à la préparation ; cette limite doit être levée parROOT, sansinventer une copie déjàfaite. R2/auditvalidator restent àrecevoir.

Garde de préparation : tasksSHA 2dda7d0604735ef29260ed4c19d8c4299b32e8c8a964345c29f1bf53214478b2 ; testsappendSHA 5b109ec8877bffd0d4ec2dbd2ecbf9f66d0edd72ba2edc7fd34310f71c2ee110. Les sources ont été lues seulement ; aucuncochage ni mutationWT.
