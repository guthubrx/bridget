# Vérifications156 en cours

T001 : baseline774 PASS/0FAIL/7ignored, filtres recouvrants (pas774 tests uniques).
T002 : six axes levés par lecture ; aucune recette fournisseur déduite.
Analyze après autofix : PASS documentaire,0finding actif, preuve cache e3176427d5fee611d625a9f36df587fae33412cd91d9836c6f1ec23b9a81e38a.
Toutes les commandes depuis le contrôle de charge sont séquentielles : CARGO_BUILD_JOBS=2, test-threads=2, nice10, environnement et sockets privés.

T003 : premier essai compileE0599 dû à une assertion sur une méthode inexistante côtérequête. Assertion supprimée ; aucune nouvelleAPI. RED métier ensuite1PASS/2FAIL (valeur inconnue et flagperdu). GREEN3PASS/0FAIL, anciennes trames byte-identiques.
T004–T006 RED : API absentes, compilationRC101, zéro assertion métier exécutée. Aucun stub d'autorisation.
T004–T012 : première compilation a révélé deux anciens constructeursCancelDelivery nonadaptés ; ajout interrupt_active=None. RED regroupé14PASS/14FAIL, dont deux erreurs de fixtures socketsDarwin héritant nonblocking (pas des défauts produit). Les sockets acceptées des deux nouveaux helpers passent en mode bloquant.
GREEN regroupé28PASS/0FAIL/0ignored : 8fondations, 6daemoncanon/rejeu/intention, 8MCP/CLI, 6pontT3. T003 classification exhaustive et purgeT006 raccordées ; QueueList métier reste fermé provisoirement avantT025.

Reçus et journaux : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T003-protocol-green.json et /Users/moi/.cache/bridget-delivery156.iuy6stuo/T004-T012-phase3-green.json.
T014–T031 restent ouverts. Aucun GREEN global, audit final ou succès de livraison156 annoncé.

ComplémentT006/FR012 : même oracle de rejeu étendu, aucune nouvelle fonction de test. REDAPIabsente/RC101 puis quatre tests magasin PASS. La transaction de remise porte admitted avant le mêmeCOMMIT ; un refusSQLite réel annule remise, ledger, suivi, lien et compteur. Rejeu sans double événement. Le raccord daemon de cette API reste en préparationT015/T018.
T022 transport : RED réel sur retraitfile absent, puis ACP1PASS ; retrait et transition versactif sous mêmeverrou, sansinterrompreactif. Nom de fonction corrigé vers guidance156_acp_cancel_queued_atomique, nomexact prévuT022.
T013 CodexH : RED réel interruptsurdeadlineagent, puis1PASS ; aucuneinterruptionagent avecdeadlinefourni ; repli humainevalidéaprèsdeadline conservé. G/I et deux anciens témoinsACP : quatrePASS ciblés. Aucun modèle réel exécuté ; fauxfournisseurs privés existants.
Les reçusT006-atomic-green, T022-acp-green, T013-codex-H-green, T013-codex-G, T013-codex-I, T022-acp-existing-cancel et T022-acp-existing-queue (JSON+logs) sont dans /Users/moi/.cache/bridget-delivery156.iuy6stuo.

T013 chaîne réelle : trois scénarios PASS, 29.10 s compilation comprise (5.50 s tests). Parent attesté, daemon, wrapper natif et fournisseurs privés ; aucun modèle réel. Codex actif : steer corrélé au tour initial, ACK sans consommation, deadline agent sans interruption. Claude : mission courte réellement achevée puis tour ordinaire ; steer refusé, interruption et ordre FIFO prouvés. Codex idle : refus sans nouveau tour ni steer/interrupt. Le RED F1 reste celui du témoin Codex H, pas un échec inventé de ces scénarios déjà pris en charge.
Reçu : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T013-chain-initial.json ; journal SHA256 c066d0cc3dead8d935731b5f3b77ce4c3a4a35aabee539c059a3c507483ad5f9. Ces preuves précèdent le raccord des gardes T014–T020 ; la suite finale prévue les revalidera après ce raccord.
## Phase IV — gardes156 et corrections de fixtures (2026-10-12)

| Exécution distincte | PASS | FAIL | ignored | RC | Durée totale |
|---|---:|---:|---:|---:|---:|
| T014-T023-B-green | 61 | 10 | 0 | 101 | 16,85 s |
| T014-T023-B-fixtures-green | 10 | 1 | 0 | 101 | 8,35 s |
| T016-CLI-green | 1 | 0 | 0 | 0 | 9,95 s |

Les noms PASS lus dans les trois logs donnent **71 tests distincts réussis**.
Le deuxième passage contient un PASS déjà acquis :61 +9 nouveaux +1 final.
Ce cumul n’est pas un run final71PASS, une suite complète ou un GREEN global.
Les gardes pures, parenté, pause, mission et rythme ont des preuves ciblées.
La production B et la preuve de source ont été appliquées ensemble ; aucun
RED isolé de l’alias UI n’est revendiqué.

Les échecs Cancel de montage auxiliaire ClientRole précédaient la garde métier :
ils ne constituent pas un RED de celle-ci. La rétrovalidation des sept oracles
Cancel/combo sur la production A avec fixtures corrigées est terminée :
0 PASS / 7 FAIL, RC101, 14,34 s, puis restauration de B attestée ci-dessous.
C’est une preuve rétrospective, pas un RED chronologique avant B.
La fixture NativeCancel attendait
à tort CancelDelivery. La source réelle remet un avis CLI passif ; le témoin
corrigé vérifie zéro interruption et la tâche toujours working.
La fixture CLI attendait à tort un envoi idempotent et un type de présence.
Les données runtime montrent Register CLI sans instance, Send de human et
live_connection_identity=None ; seule cette attente a été corrigée avant le PASS.

Chaque commande utilise nice10, Cargo jobs2 et test-threads2.
Reçus JSON et logs complets dans :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T014-T023-B-green.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T014-T023-B-fixtures-green.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T016-CLI-green.json
SHA-256 des logs, dans le même ordre :
`c5d71bb0602f1f7d334db2dfa0d80a0ca50f649e73296e45107cdb60679d014f`
`3bc232acdf6fd43683b60baffb2e2cecb7c374e09fa5153bfbf32387a4b892ea`
`edb621ad9b4ddf4d352489b2040305ecba8935adbf5b07989467631947ab8097`

Aucun cochage, nouveau test, Analyze ou audit découle de cet append.
Régression distincte T017-spec087-green :11 PASS/0 FAIL/0 ignored, RC0,5,65 s.
Reçu : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T017-spec087-green.json
Log SHA-256 : `6efc91c841cf976a299ac211c145098740fc94c0ec3e6e5599cfacd181c67d39`.
Ces onze tests sont distincts du cumul guidance156 de71, sans nouvelle suite complète.

Rétrovalidation de la référence A avec fixtures Cancel corrigées : T022-T023-corrected-fixtures-baseline-red, 0 PASS / 7 FAIL, RC101, 14,34 s. Les assertions échouent sur les flags interrupt_active réels. Production B restaurée byte-identique SHA256 3bb681d8e8aa3ce8e43ecb13a1a727a6449ff5e159345e7f0c6c7b6cadfb281d. Cette vérification rétrospective ne change pas la chronologie. Log : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T022-T023-corrected-fixtures-baseline-red.log ; SHA256 be5510c5d611d2f99b4079ef640a35db80a9fef1baa4ec9680eca0d3dd4c3ef5.

## Phase V — lecture de file et façades (2026-10-12)

| Exécution distincte | PASS | FAIL | ignored | RC | Durée totale |
|---|---:|---:|---:|---:|---:|
| T024-queue-green-T026-facades-red | 6 | 3 | 0 | 101 | 20,80 s |
| T024-T026-last-green | 3 | 0 | 0 | 0 | 13,15 s |

Les neuf noms réussis sont distincts : sept oracles daemon de lecture de file,
un oracle MCP et un oracle CLI. Le premier passage conserve trois échecs
historiques : réaffectation de file, schéma MCP et analyse CLI. Leur reprise
ciblée est réussie ; le reçu initial RC101 n’est pas réécrit en succès.
L’oracle de preuve enfant appartient à la vraie façade MCP de T026 : QueueList
sur le fil ne porte pas ChildProof. Aucun scénario supplémentaire n’est déduit.

Les noms cumulés avec Phase IV donnent **80 tests unitaires guidance156 distincts
réussis** (71 + 9). Ce nombre n’est pas le résultat d’une exécution unique,
ni celui de la suite complète. Les onze tests spec087 restent une preuve séparée.
Les commandes utilisent nice10, Cargo jobs2 et test-threads2.

Reçus :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T024-queue-green-T026-facades-red.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T024-T026-last-green.json
SHA-256 des logs, dans cet ordre :
`9521f3d79d1328329eaa143420b3a40883e72fc350a7e879878b550ff9f184bb`
`0b97f7eb39c35de967a0e7e0e55752b6659177788393ac7c599f834dd1497101`

Le complément de rétrovalidation Cancel ci-dessus est terminé : 0 PASS/7 FAIL.
Reçu : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T022-T023-corrected-fixtures-baseline-red.json
La restauration byte-identique de B est attestée par le principal dans ce
complément, SHA-256 `3bb681d8e8aa3ce8e43ecb13a1a727a6449ff5e159345e7f0c6c7b6cadfb281d`.
Aucune nouvelle vérification de sources n’est créditée à cet append documentaire.

**État de la suite complète/T029 à ce checkpoint : en cours.** Aucun résultat
final, GREEN global, audit ou cochage n’est annoncé par cet append.

## Clôture T029/T030 — une suite complète, puis reprises ciblées

La suite complète unique est terminée : 95 cibles Cargo, 700,10 s, RC101,
2350 PASS / 11 FAIL / 73 ignored. Trois PASS de processus Rust enfants sont
séparés ; le brut contient 2353 PASS. Aucun compteur de contratbuild distinct.
Les 216 sources sont stables pendant le FULL. Les 80 unitaires guidance156
et 3 E2E passent dans cette exécution : ils sont inclus, jamais additionnés.

Les 11 FAIL demeurent dans ce reçu. Huit modes sont hérités155 : deux contract,
un crash, trois identity, un reply, un pont098. Noms, messages, cibles et sites
d’assertion correspondent aux traces historiques. Le pont098 garde absence
d’agent au site 159 et refus HTTP404. Les causes fines non établies restent
non établies ; aucune qualification flaky ni correction hors périmètre.

Trois nouvelles fixtures ont des replays exacts réussis :

- deux témoins lib avec parent/connexion/lien attestés : 2 PASS, RC0, 8,27 s ;
- spec138 client nu : refus durable attendu, 1 PASS, RC0, 7,67 s. Ce témoin ne
  prouve pas une admission de parent ; celle-ci relève des oracles156/lib.

Les trois FAIL du FULL ne sont pas réécrits, ni les trois replays ajoutés au FULL.
Le gate T029 final est 763/763 PASS distincts cumulés : 762 FULL + 1 control rejoué,
zéro manquant/doublon/cible incorrecte ; 7 ignored baseline séparés. Les 774 PASS
bruts initiaux comprenaient 2 enfants imbriqués et 9 recouvrements de filtres.
Aucune relance des neuf filtres ni seconde suite complète n’a été effectuée.

ROOT atteste 214/216 sources inchangées après FULL : seuls daemon.rs et
spec138_project_test.rs portent les deltas de fixtures relus. Les replays
ne contiennent pas de snapshots sources avant/après ; leur stabilité n’est
pas inventée. Les checkpoints précédents « FULL en cours » sont supersédés ici.
T029/T030 sont validées dans leur périmètre par ROOT, sans GREEN global.

Preuves :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-full156-once.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T029-full156-once-comparison.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-two-old-fixtures-green.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T030-spec138-refusal-green.json
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/156-guidage-file/specs/156-guidage-file/validation/full156-source-delta-final.json
Log FULL : `fdb7deaff78eb9fa82a7af2879f59c3a0dd5858300450e8e198d80bd891e0274`.
Comparaison finale : `218af3b449aab1af086d29ebf1b5168a12adb1b6a3a1832e96823026db03f27c`.
Logs ciblés 2/1 : `3e848047dcac5c540d3d0a7d32767a6fc39ee912407abc7f4233d27a22b39aac`,
`deeda4675380e9fe704b61f4b980cd5fdd3344aa0789d8d6d43c45c03d2a4b35`.

## T032 — RED réel, GREEN ciblé et reprise E2E

| Exécution distincte | PASS | FAIL | ignored | RC | Durée totale |
|---|---:|---:|---:|---:|---:|
| T032-fr016-red | 0 | 1 | 0 | 101 | 13,75 s |
| T032-fr016-green-regressions | 93 | 0 | 0 | 0 | 21,91 s |
| T032-fr016-e2e-green | 3 | 0 | 0 | 0 | 17,87 s |

Le RED exécuté échoue sur le premier refus projet : count=(0,0), attendu(1,0).
Ce refus devait donner une trace et zéro admission. Après production, le même
oracle étendu passe, avec les 80 guidance156, 11 spec087 et 2 témoins historiques.
Les 3 E2E passent séparément : Codex actif/steer corrélé, Claude/interrupt FIFO,
Codex inactif/refus sans effet. Aucun nouvel oracle nommé ni modèle réel.

Les 216 sources sont stables avant/après chaque commande. Les deux GREEN ont
les mêmes sources ; daemon final SHA-256 :
`a11310b737d1ac186309a57f306babe3146ec8af7f57a921bc6adf6b398c3c3c`.
L’oracle étendu prouve pair/projet/persistance idempotents, faits/date réels,
trace unique sans admission/quota/remise, puis rejeu projet identique sans
nouvelle trace. Les autres puits sont relus et raccordés, sans prétendre un
nouvel oracle pour chaque branche. Nice10, Cargo jobs2, test-threads2 conservés.

Ces preuves portent sur T032 après changement de production. Le FULL historique
reste 2350 PASS Cargo + 3 enfants/11 FAIL/73 ignored, RC101, 700,10 s : huit hérités
confirmés, trois nouvelles fixtures corrigées 2+1. Les nombres 93 et 3 sont deux
runs ciblés ; aucune seconde suite complète ni GREEN global sur ce hash.
Le checkpoint antérieur « production inchangée après FULL » est supersédé ici.

Après les deux GREEN, ROOT a restauré les blobs de départ exacts de cinq
sous-modules suivis par rustfmt. La reproduction de cette mise en forme est
byte-identique pour chacun : aucun changement métier. Les 216 sources étaient
stables pendant les runs ; cinq hashes de forme ont ensuite changé par cette
restauration. Daemon production/test reste au SHA a113 ci-dessus. Aucun rejeu
pour cette seule mise en forme, ni couverture globale actuelle inventée.
Reçu : /Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-format-only-restored.json

Reçus et logs :
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-red.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-green-regressions.json
/Users/moi/.cache/bridget-delivery156.iuy6stuo/T032-fr016-e2e-green.json
Log RED : `32e28cb3c23be03e185ea2f51ac011304bbd5d8571eb652d0bd0183e001b58a2`.
Log GREEN93 : `0d54add94b0be1ad48dcbff28ab6e4df3b0c0e37989a928b51c92338af455c2e`.
Log E2E3 : `bb077b291e7d78ddb3483a8440dd0685c0ff292ba5b2c0f33e8c2e264c1d5723`.
