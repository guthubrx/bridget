# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/validation/tests162.md](../../../../../../../specs/162-execution-distante/validation/tests162.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Passage | Résultat | Durée attestée | Portée et effet sur tâches |
|---|---|---|---|
| Transport R1 | 7 PASS / 1 FAIL, RC101 | Compilation17,61s ; tests0,11s | Échec du refus de champ inconnu dans une disposition ; historique conservé. T002 ouverte. |
| Transport R2 | Replay exact1 PASS, RC0 | Compilation13,59s ; test0,00s | Corrige l’oracle affecté seulement. Ne remplace pas une régression complète de T002. |
| T3 Lineage R1 | 125 PASS / 3 fichiers, RC0 | 5,76s, départ10:50:15 | Contrats et reader relus conformes par ROOT. T035/T036 validées et cochées. |
| T3 UI R1 | 67 PASS / 3 fichiers, RC0 | 6,30s, départ11:01:32 | Diff des trois composants et des trois fichiers de tests relu par ROOT : conforme. Intégration Rust T034 encore attendue. T037/T038/T039 restent ouvertes. |
| Daemon fondations R1 | Compilation RC101, aucun test exécuté | Pas de durée finale attestée | Deux constructeurs Task sans remote_placement et quatre matchs non exhaustifs des types Worker. Correctifs en cours ; aucun échec métier déduit. |
| Daemon fondations R2 | 35 PASS / 2 FAIL de setup | Compilation 42,49s ; tests 0,59s | Deux chemins de socket trop longs. Sources prétest de 219 fichiers conservées ; pas attribué aux edits ultérieurs. |
| Lineage Rust R3 | Replay2 PASS, RC0 | 0,12s, même binaire sans recompilation | TMPDIR court corrige le setup ; aucun changement source entre R2 et R3. |
| T3 lint R1 | RC0 avec 8 warnings | Durée non attestée ici | Deux warnings nouveaux corrigés ensuite ; relecture et validation finale attendues. |
| Types contrats T3 R1 | RC0 | Durée non attestée ici | Portée contrats seulement. |
| Types serveur T3 R1 | RC1 historique | Durée non attestée ici | Nouveau test Reader mal typé, corrigé par Effect.asVoid ; voir R2 ci-dessous. |
| Types web T3 R1 | RC1 | Durée non attestée ici | Dix diagnostics hors des dix fichiers 162 ; préexistence plausible par comparaison, aucune compilation baseline. |
| Types serveur T3 R2 | RC0 | Durée non attestée ici | Après correction de l’oracle Effect.asVoid. |
| T3 Lineage R2 | 102 PASS / 2 fichiers, RC0 | 1,63s, départ11:26:12 | Replay ciblé contrats+Reader après corrections sémantiques ; format seul effectué ensuite. |
| T3 lint R2 | RC0, cinq warnings historiques | Durée non attestée ici | Dix fichiers vérifiés ; aucun nouveau warning restant selon comparaison ROOT. |
| Format T3 R1 puis recheck ciblé | RC1 puis RC0 | 101ms/10fichiers puis85ms/1fichier | Reader149.test.ts seul corrigé par retour à la ligne ; aucun changement sémantique, pas de replay de tests ajouté. |

<a id="extrait-2"></a>

## Extrait 2

T3 UI affiche un avertissement Node sur SQLite expérimental. Les journaux vp affichent leur répertoire physique /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/162-lineage-distant ; les commandes ROOT ont pour cwd le worktree logique ci-dessous. Aucun rendu dans l’application courante, aucune relance T3 et aucune recette distante ne sont déduits de ces tests.

<a id="extrait-3"></a>

## Extrait 3

T3 Lineage R1, cwd : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant .

<a id="extrait-4"></a>

## Extrait 4

```bash
env PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:/Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/node_modules/.bin:/Users/moi/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin nice -n 10 node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts apps/server/src/bridget/BridgetLineage149.test.ts --maxWorkers=2
```

<a id="extrait-5"></a>

## Extrait 5

T3 UI R1, même cwd, PATH Node24 et nice10 attestés par ROOT :

<a id="extrait-6"></a>

## Extrait 6

```bash
env PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:/Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/node_modules/.bin:/Users/moi/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin nice -n 10 node_modules/.bin/vp test run apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/web/src/components/BridgetTaskJournal149.test.tsx apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx --maxWorkers=2
```

<a id="extrait-7"></a>

## Extrait 7

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-r1.log | 2bca68634be16b39f3dc69542f69f1b52fe7c5db58116fe62e34c0cd7dd890e1 |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-r2.log | 258739dfa099bb9a28eac8c822093ebb678fc4b506a15cf785ee62f832798e2e |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-r2.sources | ec2239bdaed743242adba407910b26b24ebc90e079e2417a0c5d925821112977 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-lineage162-r1.log | 190d8a40da83b35ce26fff1a2859cada83229bc0e2e3536510e7ef73a68f87df |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-ui162-r1.log | ddd44dd56f63dc218940b979627c6eeffc5116d3849b27438a172a25a2c39963 |

<a id="extrait-8"></a>

## Extrait 8

| Groupe | Source absolue | SHA-256 après test |
|---|---|---|
| T035 | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/packages/contracts/src/bridgetLineage.ts | a87772c67ac0cafd5680fa6bc0b7ebe3957d01c3d966255ce1925c8a1de7bb3f |
| T035 | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/packages/contracts/src/bridgetLineage149.test.ts | a3b8fe1a10e4bf1e8542080e830077be77dd680a3b4bfe43c0a07a6ef317124f |
| T036 | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetReader.ts | f92e48e42b4c57841debb62941fea443cdd1579c7b5e863a71d3ce55847b5ab8 |
| T036 | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetReader149.test.ts | 54a9491bf25b91fd3f54e97f6bcb4ef2cd6eec894fa0ecbf8814838377522b9d |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/orchestration-v2/Orchestrator.ts | 6d2aa8f7cd4e46be804122388c0aabc3e78c0ea1aaafdc9182b5064be2eadcd7 |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts | 62f947bfe09f14d09d19bddfd661bd759132ca4ebffcd2b902c5cf2d3413f8c4 |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/BridgetTaskJournal.tsx | 4f0643c87faa193cdfac014cfeaede74fedbe64b5e727827d9a4dd56dd95df65 |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/BridgetTaskJournal149.test.tsx | fcce07fa3acaf8199c7530a6334cbe64fd8943d1d7f151aefcecf157b3109910 |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/chat/ThreadRelationshipsControl.tsx | d37c64eb7ac5b691a3cb42a1db86882c9cccd7230127b84d8286d300ad2fd826 |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx | b88dfbc5227bbdc104ab008d3b1173d64ce15c2dcd98e54392354b84f966a2fd |
| UI à relire | /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/chat/SubagentTooltipContent.tsx | 7b39a74eaa342c0c816635f672b092b4a1ddd66cc7df31fda9d6c68f1a9a7099 |

<a id="extrait-9"></a>

## Extrait 9

- T001, T003, T004, T015, T016, T018, T035 et T036 sont validées : 8/42 tâches de développement/préparation. G8/G9 restent ouverts.
- ROOT a relu le registre de production et les trois APIs de permissions : conformes. Les dix tests de permissions et les sept tests du registre sont ceux du passage R2, sans multiplier ce compte par tâche.
- T004 dispose de la self-review de son owner sur canon/persistance et de quatre oracles PASS dans R2.
- T005 reste ouverte : neuf tests worker PASS à R2, mais le conflit de publication est encore modifié. Ces résultats ne prouvent pas les octets plus récents.
- T033 : les quatre tests unitaires CLI ont maintenant réussi dans R2 ; les trois tests d’intégration restent non exécutés. T033 reste ouverte. T034 n’a pas sa validation runtime.
- T002 reste ouverte. Le replay transport exact ne vaut pas une validation générale du contrat ni le groupe US1 final.
- T3 UI : diff de trois fichiers de production et trois fichiers de tests relu conforme par ROOT. Les cases T037/T038/T039 restent ouvertes : intégration 162 encore en cours. Un correctif Reader est intervenu après les tests T3 R1 ; le replay ciblé R2 contrats+Reader a ensuite réussi. Le formatage seul postérieur n’a pas entraîné de replay supplémentaire. Cela ne clôt pas l’intégration 162 ni le contrôle web en échec.
- Aucun GREEN global, build de livraison, activation, recette fournisseur réelle ou nettoyage final n’est attesté par ce registre.

<a id="extrait-10"></a>

## Extrait 10

| Fichier absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-foundations-r1.log | 92c24032904c7255d576ae573ee8fc433f89adc0efd5f56c61af29ec14b6bc64 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-foundations-r2.log | 227c50cfd466451d97c6f41924c1b887d7b3289cabbb2a1ac3b0640bf78b25d6 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-foundations-r2.sources | cd658443d32663bd3fca657efd45b9e64df98caa33ba080d363b7c04eef70337 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-lineage-r3.log | f2bc9a68c445c1c9302da02a12f38b5c491b72a2d8606410a28a9846e5b7af11 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-lineage-r3.tmpdir | 8b2c07fb07f5b1a3d298fdd83340ec39f2be4e77674f5c2a32cc1be8e2afee49 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-lint162-r1.log | 4524f943ecfacacf6c28e6a4fe1629780d01d6e9c537463ec7c19914f4884124 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-types-contracts162-r1.log | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-types-server162-r1.log | 7c2e20c7a03f3e5da7727174bd00237e8fadc0ce05930ea9b22ad58fb964024e |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-types-web162-r1.log | 09b78c3a62f183902acc04b49f086fac4f3ba5fafefa7cb26290a6c3f9c5845e |

<a id="extrait-11"></a>

## Extrait 11

Les argv exacts de lint/types ne sont pas présents dans ces journaux et ne sont pas reconstruits ici. Les codes de retour et l’état des corrections sont transmis par ROOT. Aucun succès T3 final n’est attribué aux fichiers Reader corrigés après les runs 125/67 PASS.

<a id="extrait-12"></a>

## Extrait 12

ROOT atteste types serveur R2 RC0 après correction de l’oracle avec Effect.asVoid. Le journal vide /Users/moi/.cache/bridget-delivery162-20261013/t3-types-server162-r2.log est conservé ; le RC provient de ROOT, pas d’une inférence sur l’absence de sortie.

<a id="extrait-13"></a>

## Extrait 13

Le replay contrats+Reader R2 donne 102 PASS sur 2 fichiers, RC0, 1,63s, départ11:26:12. Commande attestée dans /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant, même PATH Node24 et nice10 :

<a id="extrait-14"></a>

## Extrait 14

```bash
env PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:/Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/node_modules/.bin:/Users/moi/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin nice -n 10 node_modules/.bin/vp test run packages/contracts/src/bridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts --maxWorkers=2
```

<a id="extrait-15"></a>

## Extrait 15

Format initial sur dix fichiers : RC1, seul Reader149.test.ts signalé. Le journal /Users/moi/.cache/bridget-delivery162-20261013/t3-fmt162-r1.log conserve ensuite le formatage du seul fichier (RC0, 89ms), puis son recheck (RC0, 85ms). Correction de retour à la ligne uniquement ; aucun changement sémantique et aucun replay de tests après ce pur format. Les empreintes actuelles ci-dessous sont donc postérieures au replay R2 ; elles ne sont pas annoncées comme captures prétest.

<a id="extrait-16"></a>

## Extrait 16

| Fichier absolu | SHA-256 actuel, après corrections et format |
|---|---|
| /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/packages/contracts/src/bridgetLineage149.test.ts | 7004c7f12c77d6adc61ee730e37d4594880b7919cfa6aeeb5a7adaa90a810aa5 |
| /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetReader.ts | e012b05abccb7dbb44aa1b8086191d6def79e867d790c231c00997d7b3a92035 |
| /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetReader149.test.ts | 599a850f95eab4f382a769356177976d50b006ea1e1fc25567522185ce0b6ac4 |

<a id="extrait-17"></a>

## Extrait 17

| Journal absolu | SHA-256 |
|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/t3-types-server162-r2.log | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-lineage162-r2.log | 4ec57e87b9e9d100acb9a1ec1a560eef3ffd812d952cb82d91624c3a5b30c6ad |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-lint162-r2.log | e9830cd945b2e7df01a8c249982cd1e9a1615cbc2757673c55efdcde4bc4c0f1 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-fmt162-r1.log | d0afe67dafc509d989b082df87fac9a57712d7a31915b7b2463deef1ebb5adcb |

<a id="extrait-18"></a>

## Extrait 18

Ce complément ne coche aucune tâche et ne valide ni runtime T3, ni recette distante, ni livraison. Aucun test, build, Git ou service lancé par le rédacteur.

<a id="extrait-19"></a>

## Extrait 19

La preuve /Users/moi/.cache/bridget-delivery162-20261013/t3-atomic-swap-probe.json atteste renamex_np avec RENAME_SWAP=2 sur deux dossiers synthétiques privés dans /Applications, même st_dev 16777229. ROOT atteste la permutation des inodes et contenus puis le retour ; le reçu confirme swap_verified=true et rollback_verified=true. installed_application_touched=false et probe_removed=true. La racine temporaire /Applications/.bridget162-swap-probe-yto_71ji a été retirée.

<a id="extrait-20"></a>

## Extrait 20

Cette preuve valide la primitive pour G8 après copie, signature et préparation du rollback. Elle ne prouve ni installation de l’application162, ni activation, ni récupération d’un ancien script149. Aucun bundle T3 n’a été touché par cet essai.

<a id="extrait-21"></a>

## Extrait 21

| Preuve absolue | SHA-256 | Sources répertoriées si manifeste |
|---|---|---|
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r1.json | a235deb09c518f90d0d1528da5e94f502893d442ad1aebffca66d583072c31c3 | — |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r1.log | f5e294135fd1b04430cfa8eed6894de373e6459c32d36ffb0d72ebf8f447b9e7 | — |
| /Users/moi/.cache/bridget-delivery162-20261013/transport162-runtime-r1.sources | 1a9d841cc66f3d7e281b980895c6a298f1364bf42c24c122cb8c8e7a5e2b7fb6 | 21 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r1.json | 722af1823c51b9b07237f8ed60a93b5ade5c6b3c8ecdaa572fd63614ad040497 | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r1.log | cd27c9fbee6540c606190f5c7e7695234da190a9600d8e90eec44f06b1e51829 | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r1.sources | 421d8c809d071679b74b6fb04123cf213d8333e2e33f62593e864aa7ead1826a | 219 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r2.json | 047c1a5dca3a0d5bdc0e974bf58cfcb96c608a92c4ec4a88695a70f19c0054bf | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r2.log | 69feb245b7d84a62c591e2f9e11a4d371228786c1f3e4fa1a63cc8868297ae7a | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r2.sources | 03aef99f723da16a0e91bc541b0ae0650270b33fd9577ab9a86733874784f964 | 219 |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r3.json | a38f0670f04f9b142f71ea2d003a8fd8425ed4d04670f97134a9629f720873fb | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r3.log | 7aaa59bded76e57beeb1c4b2cee2035b1a4a6d8eea67ed14200558655c8f2c5b | — |
| /Users/moi/.cache/bridget-delivery162-20261013/daemon162-runtime-r3.sources | 3344d038a6bcca39cd798c540957959deaf9e0a8602cf3699bf8c421d8236a5f | 219 |
| /Users/moi/.cache/bridget-delivery162-20261013/t3-atomic-swap-probe.json | a35ce9c992184ea6707071a3965096b7e33cf9eb030bc5b8a5f6ff79253799d4 | — |

<a id="extrait-22"></a>

## Extrait 22

Les références R5, R2, recovery et T3 ci-dessous renvoient aux passages et empreintes consignés plus haut. Les relectures de production sont distinctes des assertions exécutées.

<a id="extrait-23"></a>

## Extrait 23

| Tâche(s) | Preuves déterminantes | Décision et limites |
|---|---|---|
| T006 | remote162_worker_cli_options_explicites_sans_namespace R5/R2 ; lancement réel worker dans R3 | Validée ; mode technique et options explicites, pas second daemon métier. |
| T007–T008 | Contrats transport9 PASS ; registre/worker_auth_epoch R2 ; admission nominale R3 et sonde ancien maître | Validées ; choix machine/moteur/modèle et absence de repli, handshake/époques. |
| T009–T013 | worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications R3 ; marqueur/permis/frontières durables R2 ; R3 expiration/rejeu ; quota partagé155 et Fleet R2 | Validées ; réemploi du lancement et de154, bases puis modifications, limites globales. Réseau hors mutex et ordre des gardes relus dans le code, pas mesurés à chaque instruction. |
| T014 | Consolidation groupe US1 R2, correction worktree et quatre processus R3 | Validée pour le groupe prévu, avec historique des replays conservé. |
| T017/T019/T020 | Dix oracles permissions PASS R2 ; lecture parent_proof et recheck avant RELEASE ; preuve distante corrélée à confirmer avec l’owner | Restent ouverts à ce checkpoint : les tests de primitives ne sont pas présentés comme un oracle complet de révocation sur le raccordement worker parent_proof. |
| T021 | Réservation de slot/identité stable dans reprise managed et pause/quota ; recovery5 PASS ; slot initial exigé par R3 | Validée ; les refus de dispatch conservent identité et preuve. |
| T022/T025 | Quatre unitaires wrapper R2 ; blob immutable et relais/collecte R2 ; worker arrêté, blob publié, wrapper mort, reprise et ACK perdu R3 | Validées : la frontière réelle qui manquait auparavant est maintenant exercée. |
| T023/T024 | Capture/transaction/rejeu/quatre dispositions/quota différé ; refus exact des deux appelants Send R2 ; notification nominale R3 | Validées ; injection d’échec transactionnel et rejeu vérifient les frontières avant/après commit. |
| T026 | recovery5 PASS ; reprise managed/AgentInfo R1 ; Fleet sans Desired et exécution obsolète R2 ; worker restart R3 | Validée ; placement/délais fixes, slot requis, inconnus conservés. |
| T027 | worker162_collecte_exige_disposition_producteur_et_cloture, budget/orphelins et slot caduc R2 | Validée ; Deferred ne permet pas la collecte, compact conservé puis rétention testée. |
| T028 | Trois primitives historiques managed_process R2 ; reprise_frontieres_durables R2 ; store/capture/collecte R2 ; pannes réelles R3 | Validée comme combinaison des modes d’essai prévus. Pas de crash OS injecté à chaque instruction CPU ou fsync. Les garanties d’ordre fsync reposent aussi sur lecture des primitives d’écriture atomique ; aucune simulation de panne électrique matérielle revendiquée. |
| T029/T032 | Journal distant froid/gardes et live/gap R2 ; résultat retenu et parent absent R2 ; régressions155 ; cancel/restart R3 | Validées en isolé par le parcours natif Bridget. Overrun réemploie restart_from_seq et le lecteur froid, pas le RPC WorkerJournalPage. |
| T031 | R3 cancel : naissance divergente puis arrêt attesté ; epoch/unknown R2 | Validée ; pas de kill par nom, arrêt du worker distinct de l’arrêt de mission. |
| T034 | worker_auth_epoch/contact, versions/contact/fraîcheur, projection list/show R2 ; transport figé | Validée ; transitions de contact, pas mutation métier à chaque heartbeat. |
| T037 | Orchestrator.bridget149.test.ts:1759, groupe UI67 PASS R1 ; sources réattestées identiques par owner T3 | Validée : placement indépendant de available maître, activeProviderThreadId nul, replay sans fournisseur ni outbox. |
| T038 | ThreadRelationshipsControl tests355/375 ; BridgetTaskJournal tests671/708, groupe UI67 PASS R1 | Validée : machine/contact/date et v1 absent/v2null/unknown distincts ; aucun timer ajouté. |
| T039 | Contrats+Reader102 PASS R2 ; service149 dans125 PASS R1 ; UI/orchestrateur67 PASS R1 | Validée : v1 stricte, v2, repli unique/page2, refus/timeout, journal/cancel sans retry, masquage et reprise. Pas navigateur ni runtime T3 relancé. |

<a id="extrait-24"></a>

## Extrait 24

T015/T016/T018, T030, T033/T035/T036 étaient déjà cochées. Les tests UI restent des tests React et stores isolés. Les dix erreurs types web étrangères restent consignées ; aucune baseline compilée ni GREEN web inventé. Les numéros de lignes T3 sont ceux transmis par l’owner avec sources identiques aux captures.

<a id="extrait-25"></a>

## Extrait 25

| Tâche | Preuve exécutée et relecture | Décision |
|---|---|---|
| T017 | worker162_parent_proof_handle_correle_hote_tuple_revision_et_epoch PASS dans R2 ; appel réel handle, reçu Released synthétique, parent/SHA, refus tuple/révision/cwd/epoch et reçu inchangé sans processus | Validée ; remplace la limite antérieure de simple lecture du raccordement. |
| T019 | remote162_release_relit_droits_parent_apres_start_et_refuse_preuve_t3_indisponible et remote162_remise_refuse_fait_natif_change_sans_exiger_parent_absent_ou_t3 PASS | Validée ; parent courant relu après Start puis à la libération, pas auto-comparaison d’une ancienne capture. |
| T020 | Dix oracles de permissions PASS, nouveau ParentProof et nouvelle révocation après Start PASS, quatre E2E PASS | Groupe US2 validé en isolé ; la limite de confiance entre processus du même UID demeure. |
| T040 | FULL unique conservée,122 échecs rejoués PASS, replays des correctifs R2 et contrôles ciblés Rust/T3 déjà consignés | Validée avec limites explicites, sans GREEN global ni GREEN web. |
| T042 | Lecture indépendante ciblée, quatre défauts corrigés et oracles R2 PASS ; mapping 30 FR/12 SC transmis par l’agent de lecture et rapproché par ROOT | Validée comme relecture préparant Converge ; aucun passage Converge encore exécuté dans ce checkpoint. |

<a id="extrait-26"></a>

## Extrait 26

- **Deadline bootstrap 300 s** : worker162_reprise_frontieres_durables_ne_reemet_ni_bootstrap_ni_release vérifie Released puis un second slot après la deadline. La durée bootstrap ne devient plus une annulation métier après cinq minutes. Lecture indépendante : remote_worker.rs:2331 et:522.
- **Machine désactivée et reprise** : le même oracle vérifie la reprise avec configuration désactivée et le refus de nouvelles admissions. L’authentification de récupération ne réactive pas une route. Lecture : registry.rs:135, initialisation worker remote_worker.rs:1516 et chargement:1561.
- **Parent courant avant RELEASE** : les deux oracles de T019 vérifient le fait courant, y compris indisponibilité de la preuve T3 et changement natif. Lecture : native_delegation.rs:1721 et:1750.
- **Posture omise** : remote162_prepare_posture_omise_conserve_heritage_et_refuse_null PASS. Option absente conserve l’héritage ; la valeur nulle invalide ne devient pas un défaut permissif. Lecture : native_delegation.rs:1100.

<a id="extrait-27"></a>

## Extrait 27

T3 reste validé par tests ciblés de contrats/reader/orchestrateur/composants. Aucun navigateur ni redémarrage de l’application. Les dix diagnostics types web étrangers restent consignés avec comparaison de sources, sans compilation baseline. Types serveur/contrats et lint ciblé sont réussis avec les cinq warnings historiques documentés. La lecture indépendante T042 ne revendique pas une relecture intégrale des grands fichiers historiques ; le code T3 est couvert par ses lectures précédentes et sera relu dans le passage Converge préparé.

<a id="extrait-28"></a>

## Extrait 28

T043 est cochée après GO ROOT. Développement/préparation Implemented, 43/43. Converge1 reste non convergé dans l’historique ; Converge2 ciblé est le prochain passage. G8/G9 et les limites T3/recettes isolées restent ouverts. Aucun Git, build release, service ou activation par ce checkpoint.

<a id="extrait-29"></a>

## Extrait 29

La correction T043 est relue sur daemon.rs SHA-256 71c5b7065eb8e1133edbbc6a56a14165a0768fd7bd89a1ae6317e3b9c389abe4 : trace après drop(st), machine/worker et deux époques, sans credential. L’oracle existant auth/epoch a 1 PASS, RC0 en 48,95 s. Le contenu du logger est prouvé par lecture, pas par une assertion de capture ajoutée. Le mapping 30 FR/12 SC et la lecture effective T3 du passage précédent sont conservés sur code inchangé. Aucun test, build, Git ou service exécuté pendant ce passage.

<a id="extrait-30"></a>

## Extrait 30

La lecture fraîche porte sur le delta T044 : une sonde starting par cadence hors mutex puis contrôle complet de l’identité au handler ; Terminal/Refused avant DesiredStopped ; conservation du Child sur les erreurs EOF/StartupFailed ou de marqueur ; clôture des slots après preuve d’arrêt ; inventaire groupé en maintenance. Le mapping 30 FR/12 SC et les lectures T3 des passages précédents sont réutilisés explicitement sur les parties inchangées. Le raccord T044 couvre FR-009/012/014/015, avec FR-013 pour les bornes, et SC-004/005/007.

<a id="extrait-31"></a>

## Extrait 31

La couverture de lecture est de **7 898 lignes uniques sur 186 764 lignes du périmètre, soit 4,228866 %**. Il s’agit d’un échantillon des sources Bridget et T3 : les lectures antérieures ne sont reprises que sur les lignes inchangées, avec lecture fraîche des deltas D1. Cette note ne certifie pas tout le produit, ne remplace pas les résultats de tests et ne constitue pas un GREEN global.

<a id="extrait-32"></a>

## Extrait 32

L’activation est attestée par /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3/progress.json : phase activate réussie ; daemon Mac 71469 né à 17:53:53 CEST, pont 71567 né à 17:53:55 ; worker Linux 471321, naissance 142199350, exécutable du préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d. Les SHA des binaires actifs sont d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44 sur Mac et 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352 sur Linux. DB quick_check ok, zéro erreur de clé étrangère, schémas 7/12 et Fleet 5. T3 704/969 et le tunnel 1384 restent inchangés par rapport au préflight après le redémarrage externe.
