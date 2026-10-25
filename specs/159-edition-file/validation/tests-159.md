# Tests — session 159, lot B


## Clôture partielle B — T014 à T017

État au présent reçu : preuves ciblées acquises, B en cours. Aucune case de
tâche n'est modifiée par ce préparateur. T018 E2E cancel est en cours chez
ROOT et ne reçoit aucun crédit ici. Aucune suite complète, Converge, audit
ou livraison B n'est annoncée.

Trois exécutions GREEN distinctes donnent 24 noms uniques : T014/T015 7 PASS
(5 reçus et 2 droits), T016 2 PASS protocole, T017 15 PASS (7 SQL cancel et
8 daemon cancel). Ce total cumulé n'est pas le résultat d'un run unique.

| Exécution | Résultat réel | Nature de la preuve |
|---|---|---|
| T014-T015-foundations-GREEN | RC0, 7 PASS, 20,99 s | Fondations schéma/reçu et droits. |
| T016-protocol-GREEN | RC0, 2 PASS, 16,19 s | Contrat protocole ciblé. |
| T017-cancel-RED | RC101, 4,78 s, aucune assertion | Deux erreurs d'API : arité et type ; compilation seulement. |
| T017-cancel-RED-api-corrected | RC101, 2 PASS/13 FAIL, 15,87 s | Huit montages daemon échouent à la renégociation ClientHello, attribués à AlreadyNegotiated par ROOT ; ils ne prouvent pas le refus métier. Les cinq autres échecs concernent les assertions SQL cancel. |
| T017-daemon-T022-facades-RED | RC101, 1 PASS/12 FAIL, 27,84 s | Huit cancel daemon atteignent bien Nack unsupported_operation. Les quatre autres échecs concernent trois façades T022 et un oracle catalogue native148 ; ce résultat ne les clôt pas. |
| T017-cancel-GREEN | RC101, 19,17 s, aucune assertion | Deux erreurs de compilation : qualification BridgetMessage et comparaison du Result d'encodage. Corrigées par ROOT ; le label GREEN n'est pas un succès. |
| T017-cancel-GREEN-compile-corrected | RC0, 15 PASS/0 FAIL, 25,33 s | Les quinze cancel prévus réussissent ; 222 sources avant/après identiques. |

Le dernier GREEN T017 est lié à daemon
1b09e270fe6350877f579984e85bb48e315de12731a19cbf02efe5b6dfc5763e.
Les cinq empreintes ciblées ont aussi été comparées aux fichiers actuels.
La stabilité s'applique à chaque run ; elle ne rend pas les sources des runs
antérieurs identiques à celles du dernier. Aucun échec de compilation ne compte
comme RED métier. Aucun test T018, reorder, promote ou façade n'est ajouté au bilan.

Reçus GREEN canoniques privés :
/Users/moi/.cache/bridget-tests159B-20261012/T014-T015-foundations-GREEN.json
/Users/moi/.cache/bridget-tests159B-20261012/T016-protocol-GREEN.json
/Users/moi/.cache/bridget-tests159B-20261012/T017-cancel-GREEN-compile-corrected.json
Log final T017 SHA256 :
9b765613d54527780efcf35c4a4007ad823738c3c9fec43f23126663f47527aa
La chronologie, les 24 noms et les SHA de tous les reçus/logs figurent dans :
/Users/moi/.cache/bridget-analysis159B-20261012/documentary-proof-preparation/T014-T017-progress/proof-T014-T017.json

Référence de nettoyage antérieur 152–159A, distincte de la livraison B :
/Users/moi/.cache/bridget-cleanup152-159B-preparation-20261012/final-cleanup152-159A.md
/Users/moi/.cache/bridget-cleanup152-159B-preparation-20261012/final-cleanup152-159A.json
ROOT a retiré 23 dossiers possédés, soit 1 647 339 986 octets logiques ; aucun
gain physique APFS revendiqué. Le reçu conserve le WT B, les archives,
les sauvegardes et le cache utile. Il ne prouve aucun nettoyage final B.


## Point ciblé B — T018 à T022

Ce point complète le reçu T014–T017, sans le remplacer. Aucun cochage, aucune
clôture B ni suite globale verte. Les sommes ci-dessous portent sur des noms
ciblés ; elles ne décrivent pas un run global.

| Groupe | Preuve au présent point | Limite |
|---|---|---|
| T019 reorder | 14 noms PASS cumulés : 13 puis 1 ciblé | Le FAIL WouldBlock historique reste dans le log ; la fixture de drainage corrigée réussit ensuite. Pas un run unique 14/14. |
| T020 transports | 9 noms PASS : 1 wrapper et 8 transport | Deux cibles Cargo dans un lot mixte RC101 dont les quatre FAIL sont reorder ; aucun run autonome 9/9 annoncé. |
| T022 façades | 8 noms PASS cumulés : 7 puis 1 gel de schéma ciblé | Premier lot RC101, 7 PASS/1 FAIL ; guidance156_mcp_queue_schema réussit ensuite. Sources MCP distinctes, pas un run unique 8/8. |
| T021 SQL | RED 3 PASS/4 FAIL, puis GREEN 7 PASS/0 FAIL, RC0/11,39 s | Les sept tests daemon promote ne sont pas encore exécutés. |
| T018 cancel E2E | Deux lots historiques 0 PASS/3 FAIL, puis R2 3 PASS/0 FAIL, RC0/37,71 s | Reprise explicitement pausée ; aucune preuve de retry gagné sans pause. |

Chronologie historique des lots mixtes : T019-four-daemon-RED-T020-nine-GREEN,
RC101/42,82 s : daemon 1 PASS/4 FAIL, transport 8 PASS. Le seul PASS daemon
est le wrapper T020. T022-facades-metadata-GREEN, RC101/18,26 s : 7 PASS/1 FAIL.
T019-fourteen-GREEN-T021-seven-SQL-RED-T022-freeze, RC101/21,39 s : 17 PASS/5 FAIL,
soit reorder 13/1, SQL promote 3/4, gel MCP 1/0. Ces échecs restent dans les logs.
Les cartes sources avant/après et les SHA des huit logs ont été vérifiés.
Chaque source reste liée à son run ; aucune couverture sur un hash futur.

Replays finaux ciblés : T019-order-fixture-drain-GREEN, 1 PASS/0 FAIL,
RC0/7,22 s, log SHA256
3a626cdd7fce33b12b8f44ed93a71fbdc7faac13283efc690947b74e0c3d0ee5.
T021-seven-SQL-GREEN, 7 PASS/0 FAIL, RC0/11,39 s, log SHA256
017601e7a76c8f447c1fd576f6eab12269f957a62eef41a57fa24d649400bfbd.
T018-cancel-three-boundaries-paused-recovery-r2, 3 PASS/0 FAIL, RC0/37,71 s,
log SHA256 6fedfcfc91364699067a718d3c12a8fd03423f152ac622538a5116eeac02055f.
Chaque run reste stable sur ses sources ; aucun FULL ni verdict global B.

La fixture T018 R2 impose une pause pendant la reprise privée : réinscription
réelle de l’auteur, retrait/rejeu avant reprise autonome, puis levée par le
socket de contrôle humain existant. Elle ne prouve pas un retry gagné contre
la libération automatique sans pause. Cette course peut rendre not_held.
Avant BEGIN ou après rollback, aucun retrait n’est validé : SC007 « jamais
livré » commence seulement au commit Applied. Le patch test-only est
301a6cfd1359ed86907e5f1d5e6171bf6f1da55a5ee89067bbee46564dc2d6d2,
candidat 1fd3ee6d9f2fa99328f7b69a62d115e32ec366133a4251ca59a5312e295d45a6.
Le précédent lot avec reconnexion échouait 0/3 en 34,89 s ; cette correction
ultérieure ne transforme pas ce run historique en succès.

Reçus et noms exacts, empreintes par cible, patch SQL et limites R2 :
/Users/moi/.cache/bridget-analysis159B-20261012/documentary-proof-preparation/T018-T022-progress/proof-T018-T022.json
Receipts mixtes ROOT :
/Users/moi/.cache/bridget-tests159B-20261012/T019-four-daemon-RED-T020-nine-GREEN.json
/Users/moi/.cache/bridget-tests159B-20261012/T022-facades-metadata-GREEN.json
/Users/moi/.cache/bridget-tests159B-20261012/T019-fourteen-GREEN-T021-seven-SQL-RED-T022-freeze.json


## Clôture ciblée T021 et SC014

Ce point remplace l’attente des sept tests daemon promote au checkpoint
précédent. Le lot T014-five-T021-fourteen-SC014-GREEN réussit 19/19 tests,
RC0, 18,04 s : cinq reçus T014, sept promote SQL et sept promote daemon.
Les deux oracles SC014 appartiennent aux sept SQL ; ils ne sont pas ajoutés
au total. Les 223 sources avant/après sont identiques pendant ce run.

Les deux oracles SQL existants ont d’abord échoué : 0 PASS/2 FAIL,
RC101/7,59 s ; la remise restait held au lieu de cancelled après demande
close ou échéance. Le correctif conserve la clôture et sa notice existante
avec le reçu not_held, dans le même commit. Un refus ordinaire continue à
annuler les écritures du callback. Rejeu : reçu seul, sans effet ni notice fresh.
Ce résultat ne prouve pas une injection ni une consommation par le modèle.

Chronologie technique bornée : le premier filtre daemon sélectionne zéro
test (RC0/12,93 s), donc aucune preuve métier. Le filtre corrigé échoue 0/7
(RC101/4,75 s) sur une réinscription d’agent déjà connecté, avant l’action.
Après correction de fixture, le vrai RED 0/7 (RC101/9,02 s) atteint
unsupported_operation. Ces runs restent distincts du GREEN 19/19.

Reçu final :
/Users/moi/.cache/bridget-tests159B-20261012/T014-five-T021-fourteen-SC014-GREEN.json
Log SHA256 156443d79eb6d65891a47f35681198f8b753f08bac4ea3d1d57a9fb116ba6c38
Noms exacts, cinq reçus/logs, classification des non-preuves et empreintes :
/Users/moi/.cache/bridget-analysis159B-20261012/documentary-proof-preparation/T021-SC014-progress/proof-T021-SC014.json

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/validation/tests-159.md#extrait-1).


## T026 final — FULL unique et disposition ROOT

FULL unique RC101, 856,66 s, 97 cibles Cargo : 2629 PASS, 1 FAIL, 76 ignored.
Sept PASS de processus enfants sont exclus de ces totaux. Par package : core
45 PASS ; daemon 2212 PASS/1 FAIL/74 ignored ; transport 372 PASS/2 ignored
(11 cibles). Les 223 sources avant/après et celles des replays sont identiques.
La comparaison transport couvre maintenant le package via ce FULL ; les
337 PASS du run --lib ne constituaient pas cette comparaison complète.

Le seul FAIL est claude_gere_recoit_mcp_identite_et_path : EINVAL22 Darwin lors
de set_read_timeout(Some3s) dans le nettoyage du mock, puis échec du join.
Le test et ses fonctions sont inchangés, mais cet échec n’était pas parmi les
neuf du FULL A : ne pas le présenter comme un échec A déjà observé. Rejeu exact
1 PASS/0 FAIL, RC0/6,36 s ; quatre cas existants, deux threads : 4 PASS/0 FAIL,
RC0/6,33 s. Aucun changement de code, aucune seconde suite complète. ROOT
retient le FAIL et les deux replays : défaut de production non reproduit,
aucune cause précise du noyau affirmée, aucun GREEN global inventé.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/validation/tests-159.md#extrait-2).

FULL :
/Users/moi/.cache/bridget-tests159B-20261012/T026-FULL159B-once-final-source.json
Log SHA256 14962562377e6f696daef443540d6d04ca3cb575444d2dffff8c263e7917294e
Disposition finale ROOT et reçus des deux replays :
/Users/moi/.cache/bridget-tests159B-20261012/T026-disposition-final.json
SHA256 bc96203086eb32f5890fbd59bb57280c21426f53e0fb3b9682b0438b85aa71b2
La validation technique/cochage dépend du Converge et des gates ROOT ;
le présent append ne coche aucune tâche et ne déclare pas la livraison.
