# Tests155 — suite unique et replays ciblés

Checkpoint courant R2 : In Progress, 42/42 acquises, T040 validée sur le reçu
réel antérieur au correctif cwd. Ce correctif a maintenant 71 PASS ciblés sur le code courant. Converge P3 est historique ; relecture du delta après
audit attendue. Les anciens statuts et PASS ci-dessous gardent leur snapshot.
État courant : implémentation réalisée à 100 %, 42/42 tâches cochées par le
principal, T040 validée après accord explicite de l’utilisateur et quatre cas
PASS. Le pipeline reste In Progress : Converge P3, audit final, revue code R2
et livraison restent à faire. Aucun GREEN workspace global n’est revendiqué.
Les checkpoints 41/42, Human PENDING et corrections en cours ci-dessous sont
historiques et supersédés par les preuves finales ajoutées en fin de document.
État historique après le cycle C1 : In Progress, 41/42 ; T040 seule ouverte et
validation explicite de l’utilisateur attendue. Corrections H1/H2/H3 en cours.
Aucune résolution n’est attestée avant les preuves ciblées. Converge P2 reste
un checkpoint historique avant ces correctifs ; P3 reste à faire. Les résultats
antérieurs décrivent leurs snapshots, sans GREEN global courant.
État courant : In Progress, 41/42 ; T039 et T041 documentaire acquises.
T040 seule reste ouverte. T041 ne change aucun code ni résultat de test.

Le présent état distingue la suite workspace unique avant correctifs et les
replays ciblés ultérieurs. Il ne revendique aucun GREEN global. Aucune case de tâche n’est
modifiée par l’owner documentaire.

## Suite workspace unique avant correctifs

Le run workspace155-final-r1 se termine avec 2259 PASS, 15 FAIL et 72 ignored,
RC101 en 554,43 s. Les 94 cibles sont complètes. Les trois résumés de helpers
imbriqués sont exclus du total. Les 214 empreintes source avant/après sont égales.
La suite a commencé le 2026-10-12 à 01:47:46 UTC. Elle précède les correctifs
ciblés décrits ci-dessous ; aucune seconde suite complète n’a été lancée.

Les 180/180 tests de la baseline T001 sont PASS dans ce run avant correctifs :
167 daemon/lib, 9 transport/lib et 4 Claude. Aucun nom ne manque. La comparaison
retrouve les 8 échecs historiques152 et distingue 7 nouveaux échecs. La fixture
UTC corrigée154 est PASS et n’est jamais classée parmi les échecs historiques.

Les sept nouveaux noms sont :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/155-demandes-enfant/validation/tests-155.md#extrait-1).

Copies exactes de la comparaison root, conservées sans réécrire son diagnostic
antérieur aux correctifs :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace155-final-r1-comparison.json.
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/workspace155-final-r1-comparison.md.
Log primaire : /Users/moi/.cache/bridget-delivery155.um4xccv4/workspace155-final-r1.log.
SHA256 : decd7bf7246e62d12cb47a98caa6af9bf96627ef06a238e11350ba4eab8d9183.
Reçu : /Users/moi/.cache/bridget-delivery155.um4xccv4/workspace155-final-r1.json.
SHA256 : 2e77e91dce76b0819fe16a45dfee7c2b2baa36331509b7de371534834e811785.

## Correctifs et replays ciblés après la suite

La mutation anticipée des descendants dans cancel_tree a été supprimée. Leur
transition cancelling reste effectuée après classification de tous les membres.
Ainsi, un arrêt non vérifiable ne change plus prématurément les descendants.
La garde centrale linked_task contrôle au plus sept ancêtres et interdit un
accord si un ancêtre annule. L’extension du scénario chaîne existant couvre
ce chemin ; aucun nouveau scénario n’est ajouté. Lecture source ciblée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs,
fonction cancel_tree ;
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs,
fonction linked_task.

Les attentes de fixtures de schéma/listes sont alignées sur les clés de vue et
l’outil ajoutés155. Le replay099 avec diagnostic constate O_NONBLOCK=true sur
le socket. Il passe, mais le diagnostic temporaire a ensuite été retiré et le seul test rejoué
sur sa forme finale avec set_nonblocking(false). Le reçu http099-final-r2
confirme RC0, 1 PASS / 0 FAIL, 8,27 s et 214 sources stables.
Ce checkpoint ne qualifie pas l’échec initial de fluctuation aléatoire.

| Replay distinct | RC | Durée | Résultat | SHA256 du log |
|---|---:|---:|---|---|
| regressions155-replay-r1 | 0 | 10,04 s | 1 PASS, diagnostic temporaire099, 0 FAIL | 5da85eb8b649cbfe65350a8562f4d33e34d8e708b76a1ad1d414255419981db1 |
| native155-postfix-r1 | 0 | 42,64 s | 71 PASS : daemon63, E2E1, transport7, 0 FAIL | a6b5c7261e64e87ea0be1fa38e23d7c993b461e9a84dffbbc0ecc2cb6be895a3 |
| annulation152-postfix-r1 | 0 | 4,64 s | 2 PASS, 0 FAIL | ff7cf7a01970f9eebe72010870609cd723d06d342ec08f3618f908dfa678c797 |
| annulation148-postfix-r1 | 0 | 4,76 s | 1 PASS, 0 FAIL | e3f13f62b9d2f305895069d53606d0273874877e131677a322ee80d297ee9445 |
| mcp155-matrix-postfix-r1 | 0 | 4,63 s | 1 PASS, 0 FAIL | 7773f131a339c9488872b988f1cfeb14538292c729a117f957dc9dbc1a2dd610 |
| mcp151-schema-postfix-r1 | 0 | 4,6 s | 1 PASS, 0 FAIL | 4d659031417a57bbdc631d05a79f2a76b62c6dee4949cb7cbc2eadc3812b85f6 |
| workspace154-schema-postfix-r1 | 0 | 4,93 s | 1 PASS, 0 FAIL | e4d3496a06d155915e9f37d362e2f4381b3c09a5239480b022d86174a5a8605b |
| skill089-postfix-r1 | 0 | 6,35 s | 1 PASS, 0 FAIL | 5fe14050bd619cef0e7bb9bba5272d150ea0080245fbfd10ad2201003b2c0b88 |

Chaque replay a 214 sources stables, propres à son reçu. Les JSON et logs portent
le nom du replay sous /Users/moi/.cache/bridget-delivery155.um4xccv4/ et sous
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/.
Ces replays ne sont pas une seconde suite et leurs résultats ne sont pas
additionnés au total historique 2259/15/72. Les sept nouveaux échecs ont un
replay PASS ; la confirmation099 après retrait du diagnostic est acquise.
T039 est acquise ; seule T040 reste ouverte. Aucun GREEN final ou global n’est revendiqué.

Les quatre tests transport prévus sont PASS dans native155-postfix-r1 :

- `codex_app_server::tests::native155_transport_mcp_tiers_et_exception_bridget`.
- `codex_app_server::tests::native155_transport_methodes_et_offres_refusees`.
- `codex_app_server::tests::native155_transport_command_forward_borne_et_fin_tour`.
- `codex_app_server::tests::native155_transport_user_input_secret_et_reponses`.

## Résultats ciblés historiques

| Exécution | RC | Durée totale | Résultats |
|---|---:|---:|---|
| requests155-green-r2 | 101 | 32,14 s | Lib 39 PASS / 2 FAIL ; intégration Claude 1 PASS, donc 40 PASS / 2 FAIL |
| requests155-fixtures-green-r3 | 0 | 6,55 s | Les deux tests de fixture encore rouges en R2 : 2 PASS / 0 FAIL |
| e2e155-green-r1 | 101 | 2,88 s | Erreur de compilation E0282, type Option de tuple non inféré ; aucun test exécuté |
| e2e155-green-r2 | 0 | 21,99 s | native155_e2e : 1 PASS / 0 FAIL |

Chaque reçu contient 214 empreintes avant/après égales. Les 42 cas de la sélection
requests sont réussis cumulativement : 40 PASS distincts en R2, puis les deux
anciens FAIL devenus PASS en R3. Aucun run n’a 42 PASS seul. L’E2E 1 PASS reste
une preuve distincte de cette sélection, avec ses propres sources historiques.

Les trois tests unitaires de liste sûre et l’intégration Claude passent en R2 :

- `wrapper::reconnect_tests::native148_claude_allows_native_delegation_without_global_mcp_approval`.
- `wrapper::reconnect_tests::spec091_politique_mcp_reste_octet_pour_octet_stable`.
- `wrapper::reconnect_tests::spec094_codex_autorise_exactement_les_outils_de_communication_surs`.
- `spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises`.

Les vues et réponses parent, leurs clés fermées, la pause et l’attente Parent
sont prouvées par leurs tests exacts dans la liste du gate. Ces preuves
permettent au principal de relire T025/T026 et les trois guides T037 ; elles
ne prouvent aucune installation ou activation de155.

## Deux corrections de montage entre R2 et R3

Le reçu R2 et le log montrent deux FAIL :

- `daemon::native_delegation::native_delegation_permissions149_tests::native_child_requests155_tests::human155::native155_human_decisions_obsoletes_expiration_annulation_purge`.
- `daemon::native_delegation::native_delegation_permissions149_tests::native_child_requests155_tests::native155_request_budgets_ouverte_et_totale`.

Budget : l’ID fournisseur 8 est refusé par le disjoncteur149, après huit envois
sur sa fenêtre 180 s. Le diagnostic runtime de git154, lecture SQLite mode=ro,
constate huit ledger/deliveries pour 0–7, puis aucun ledger/delivery pour 8 ;
la raison est exactement « disjoncteur: limite 8 échanges / 180s ».
La fixture total32 monte uniquement son disjoncteur existant à 64 et draine
les avis du parent. Le budget155 reste 4 ouvertes / 32 totales, les assertions et
le refus wire restent inchangés ; aucune correction de production pour cette cause.
DB synthétique observée :
/private/tmp/b.z08cvf1_/bridget-n149-tot-77ae673e-62468.db.

Humanpurge : R2 observe cancelling au lieu de cancelled. La fixture installe
le backend synthétique de confirmation STOP avant le premier tick. Une
confirmation arrivée après expiration du receiver ne règle pas ce montage.
Les assertions d’arrêt confirmé, fermeture humaine et purge sont conservées.
R3 vérifie les deux cas avec 2 PASS, sans créer de scénario supplémentaire.

Les empreintes des runs prouvent que seules ces deux fixtures ont changé
entre requests R2 et R3. Les autres sources de ces deux snapshots sont identiques.
Entre E2ER1 et E2ER2, seule la fixture native_delegation_e2e.rs a changé ;
root identifie une annotation du tuple Option, sans changement de scénario.

## Limite FR018 observée — avis tenté, pas garanti

Dans la fixture R2, le neuvième avis (ID fournisseur 8) est refusé par SendAs
avec ses gardes149/disjoncteur : aucun ledger/delivery d’avis, mais une demande
open déjà persistée. Le libellé initial « Une question ouvre un avis » promettait
un succès garanti ; FR018 est clarifiée en tentative avec gardes conservées.
Un refus ne perd pas la demande : Status la conserve pour son propriétaire et
Wait peut la rendre lorsqu’elle est Parent ouverte/non expirée/actionnable.
Les 42 PASS cumulés ne prouvent donc pas que chaque avis est émis en production.
Aucune modification de source, garde, scénario ou compte de tests pour cette
clarification documentaire.

## Limites des empreintes et environnement

Les hashes ci-dessous sont ceux des runs archivés. Root a arrêté le formatage
large et fait relire les changements de format seuls avant retrait. Ces
preuves ne décrivent pas les sources actuelles après ce retrait. La suite
workspace et les replays ci-dessus ont chacun leur snapshot source distinct.

BRIDGET_BUILD_CACHE_GIB=40 est une variable du processus dans ces reçus.
Elle ne prouve aucune modification d’une configuration durable ou du budget de 20 Gio
historique. Aucun build, test, Git ou service n’a été lancé par le rédacteur.
Les deux warnings historiques du run (due_child_requests,format_cited_paths)
restent visibles dans les logs ; aucun succès de lint n’est déduit.

## Reçus et logs

- /Users/moi/.cache/bridget-delivery155.um4xccv4/requests155-green-r2.json.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/requests155-green-r2.log. SHA256 `5d2c0a879b09a8cec09c6aa87a8f5479ca80b370a91dfd0112fbf1908c4b9385`.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/requests155-fixtures-green-r3.json.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/requests155-fixtures-green-r3.log. SHA256 `0a7a292663a55dfcd2189232e79d31b678e1923473905a4c9b8835fc4ab16223`.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/e2e155-green-r1.json.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/e2e155-green-r1.log. SHA256 `6748a319278f4190c502b2da5b06bd76a4a533753a3b3ebb10939a10c4b15442`.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/e2e155-green-r2.json.
- /Users/moi/.cache/bridget-delivery155.um4xccv4/e2e155-green-r2.log. SHA256 `c3f55aaaafb700ebaf6cf9f0b0e46d90397c0a3287f5a22375714914300d6a2b`.
- Gate cumulatif avec 42 noms exacts et provenance : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/requests155-cumulative-gate.json.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests155_tests.rs : R2 `99bfea8ed0f020cff854223cb472f4cff103deaf00ec33dded1223fc9ba557cd`, R3 `96be315b4fc6ad8f4586972ac83b0e0c4a996f5a3956d98c675c5d6e958d8c31`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests_human155_tests.rs : R2 `d20bbf4f572d30a93c5738fb3010bfc34f01a40800c6b61f2f506c50a0b710b2`, R3 `e04b0b97814e31782bd530d019704fe9abd726edd2a92b14fc20615eee7ffb44`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/tests/native_delegation_e2e.rs : E2ER2 `7d6ff9420938b4766fe90eae6ebb352e8c84ba23cb44713b04cffd92fb070401`.

## Delta runtime environmentId local — GREEN ciblé, recette ouverte

Une capture réelle Codex 0.162.1 montre item/commandExecution/requestApproval
avec kind=command, environmentId="local", startedAtMs et offre explicite
[accept,cancel], sans amendement. Preuve durable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/human-approval-runtime155.json.
SHA256 : fc69404af625cc3ce5cd8830c9caa4cf225fbd2e8fd8efb5c42cb34d98523b96.
Capture primaire : /private/tmp/b155.2s2_gkz3/human-approval-runtime.json.

Le snapshot antérieur refusait toute valeur environmentId non nulle avec
approval_method_unsupported. Le correctif accepte seulement le champ
absent, null ou la chaîne EXACTE "local". Toute autre valeur non nulle,
notamment un environnement étranger, reste fermée. "local" n’est pas un droit
ni une permission d’approuver. kind=command, décisions explicitement offertes,
effets établis et droits figés requis restent contrôlés.

La revue de confiance native transmise par root porte sur le lancement local
Command::new, ses pipes STDIO, le mode forward non interactif, session/tour et
identité enfant attestés, avec dangerFullAccess figé requis. Cette frontière
ne repose pas sur une simple déclaration de l’enfant dans un texte libre.
Le correctif transport a un GREEN ciblé dans native155-postfix-r1 : 71 PASS
(63 daemon, 1 E2E, 7 transport), dont les quatre tests transport prévus.
La capture initiale sans item reste historique ; la recette réelle finale
ci-dessous constate désormais une demande Human ouverte. T040 attend la
validation explicite de l’utilisateur ; aucun accord accepté n’est revendiqué.

La suite unique commencée à 01:47:46 UTC est terminée avant ce correctif.
Son résultat 2259/15/72 ne décrit donc pas le code corrigé. Les preuves ciblées
ultérieures sont séparées ; aucune seconde suite complète n’est revendiquée. T038 et 40/41 restent acquis ; T039 est acquise ; seule T040 reste ouverte. Aucun nouveau
scénario, API/helper, cochage, test ou changement de source par ce rédacteur.

## Recette réelle finale — Human ouverte, validation utilisateur attendue

Le principal constate dans le namespace isolé
/private/tmp/b155.2s2_gkz3/final-ns une demande Human open.
Binaire de cette recette :
fb48b0420c8e305ef4cff9621f67a5fa84c85ac6d78da62deb3e0581d1b3d2ec.
Tâche : 37356718-0f26-4f4b-b23f-bbc703d5af32.
Demande : 7f4868101610d6bd45b22cf7836336ab.
Item humain : eeeccded-e8bb-45e4-9d85-b7a526eaa115.
Respond du parent est refusé avec parent_cannot_approve. La remontée réelle
est observée ; aucune décision humaine acceptée ni exécution approuvée
n’est attestée. T040 attend la validation explicite de l’utilisateur.
La session reste In Progress, sans livraison ni GREEN global.

## Dernier replay099 sur sa forme finale

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/155-demandes-enfant/validation/tests-155.md#extrait-2).

Le refus réel workspaceWrite est PASS dans la même recette : tâche
8512b027-5afc-4596-bfb9-a8aa5b6ecf7d, demande
7e0ea0ce245919d60b52e93632199e2d, verdict forbidden, outcome
refused:command_unbounded, décision decline remise (delivered), sans item
humain ; parent_sandbox=workspaceWrite. Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-final-runtime-open.json.
SHA256 : 8dee7fc2f7776e5ebb45dec7aa14d1917f093fd14500dade240e92e69e535dcd.
T039 est acquise sur preuve cumulative ; seule T040 reste ouverte.

Checkpoint historique après T041 : In Progress, 41/42, T039 et T041 documentaire
acquises ; T040 seule ouverte. Les checkpoints historiques 40/41 restent
historiques et sont remplacés par cet état courant. T041 corrige uniquement
le paragraphe Journal du contrat, sans changement de code ou de tests.
Preuve et lectures ciblées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/implementation.md,
section T041 (transport codex_app_server.rs:165–178,4266–4310 ;
table delegation.rs:307–327).

## Checkpoint historique C1 — avant preuves finales

Le cycle C1 en lecture seule est clos : 0 CRITICAL, 3 HIGH, 1 MEDIUM, 2 LOW.
Archive originale conservée :
/Users/moi/.cache/bridget-delivery155.um4xccv4/audit155-cycle1-original.
Agrégat : /Users/moi/.cache/bridget-delivery155.um4xccv4/audit155-cycle1-original/cycle-1/aggregated-findings.json.
SHA256 : c831e7a10f706e5ef23b10125643ec9938a7b7abb63772010a09333eef5384ab.

Les trois HIGH sont confirmés par lecture statique, sans reproduction runtime :
H1/SEC-001 (faits Human transformés ou raccourcis), H2/REL-001 (écriture STDIO
privée non bornée sous les tables), H3/PERF-081 (notification externe Human
retenant le verrou DaemonState). Leurs correctifs sont en cours. Aucun nouveau
PASS, hash final de source, recette réussie ou finding résolu n’est attribué
à ces correctifs dans ce checkpoint.

La correction du signalement Claude MEDIUM sur emit a été écrite et relue par
le principal : l’erreur ne produit unknown que si le permis d’émission avait
été acquis. Sans permis, aucun octet ne peut avoir été écrit et sent reste
suivi, avec FR027 refait avant un prochain envoi. Aucun champ SQL, marqueur
persistant ou helper nouveau. Les tests ciblés de cette correction restent
attendus. PERF-041 corrige seulement l’annotation de coût en q², sans modifier
l’algorithme. PERF-082 et DUP-001 restent des limites acceptées : budget du tick
renouvelé par émission et duplication locale de reconnexion, sans incident
runtime mesuré ni refactor de confort.

Converge P2 est historique, avant les corrections C1 :
/Users/moi/.cache/bridget-delivery155.um4xccv4/converge155-p2.json.
SHA256 : 9c77dc98c3f40a3f9da51d4a29f703aea0f6aabd962419a61f0914e0c092ecb0.
P3 reste à faire après validation des changements. Le statut reste In Progress,
41/42, avec T040 seule ouverte ; aucune case n’est modifiée par le rédacteur.

La suite workspace unique conserve 2259 PASS / 15 FAIL / 72 ignored, avec
180/180 baseline PASS. Ses 8 échecs historiques et 7 nouveaux sont distincts ;
les 7 nouveaux ont ensuite leurs replays PASS sur le patch antérieur à C1.
Les 71 PASS155 antérieurs ne valident pas les nouveaux correctifs C1. Aucune
seconde suite complète, aucun total final GREEN ou test supplémentaire n’est
inventé. Les reçus de question, expiration et workspaceWrite restent historiques ;
la demande Human attend toujours la validation explicite de l’utilisateur.

## Checkpoint historique avant Converge P3 — 42/42

Le principal a validé et coché T040 après l’accord explicite « Oui, accepter et
valider si les quatre cas réussissent ». Le reçu conserve cet accord à
2026-10-12T03:02:02.400752+00:00. Les quatre cas sont PASS. L’implémentation
est réalisée à 100 %, 42/42 ; le pipeline reste In Progress. Converge P2 est
historique avant les correctifs C1 ; P3, audit final, revue R2 et livraison
restent à faire.

Reçu actuel :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-postfix-final.json.
SHA256 : 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3.
H1/H2/H3 et le permis d’émission MEDIUM ont leurs correctifs et preuves ciblées
fraîches. PERF-041 change seulement le commentaire de coût. PERF-082 et DUP-001
restent les deux LOW acceptés dans le périmètre. Cela ne vaut pas audit final.
La suite complète unique reste 2259 PASS / 15 FAIL / 72 ignored avant correctifs,
avec 180/180 baseline PASS et les sept nouveaux échecs ensuite rejoués PASS.
Aucune seconde suite complète n’a été lancée.

## Tests ciblés frais après les correctifs C1

| Run distinct | PASS / FAIL | RC | Durée | SHA256 du log |
|---|---:|---:|---:|---|
| native155-auditfix-r1 | 71 / 0 | 0 | 60,68 s | 5428eb4ed0e3ffb17091f1be18683094003eaaa490a8a58898004bf8f46b113d |
| channel087-auditfix-r1 | 1 / 0 | 0 | 7,57 s | ec912932bc2bf13fac58c68d15ca050d868a13fe557e1746a7134a0a1225283d |
| stdio-sequence-auditfix-r1 | 1 / 0 | 0 | 13,39 s | ee18028c445d3febaa8c26c0ba042897ea979ab8fa9fa5f328b2ba4f4d990812 |
| stdio-approval-auditfix-r1 | 1 / 0 | 0 | 0,75 s | 6c9a6bca566595647921c0eaa6a326d6543aee0e238e348ca60c82b7c30cb767 |
| stdio-permissions-auditfix-r1 | 1 / 0 | 0 | 0,7 s | caab0ce817c3e7b61c9a6583b0fbbf277f1370d1ac8c4c538ab81086122f7225 |

Chaque reçu a 214 sources stables avant/après. native155-auditfix-r1 contient
63 PASS daemon, 1 PASS E2E et 7 PASS transport, soit 71 noms PASS vérifiés.
Les quatre autres runs sont distincts : canal087 et trois régressions STDIO,
4 PASS cumulés. Aucun compteur n’est ajouté au résultat de la suite complète.
Les reçus JSON et logs sont conservés sous :
/Users/moi/.cache/bridget-delivery155.um4xccv4/.
Leurs copies sont sous :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/.

H1 : commande complète citée avec contrôles visibles, indicateurs explicites
paths_cited_changed/truncated et effets command/file_write/network. Une commande
hors borne ou des faits hors 4096 octets sont refusés. H2 : les écritures STDIO
privées et leur verrou ont une échéance, avec refus sûr sur incertitude.
H3 : dépôt et lien restent durables ; notification Human hors DaemonState.
Le permis d’émission MEDIUM distingue l’échec avant toute écriture possible.
Le principal atteste la relecture et les scénarios étendus prévus ; les runs
ci-dessus en donnent la preuve ciblée. Aucune garantie générale de terminaison
d’un programme hostile, benchmark ou couverture globale n’est ajoutée.

## Checkpoint R2 — correctif cwd prouvé par les ciblés

État courant : In Progress, 42/42 tâches acquises, T040 validée. L’objection
MEDIUM cwd de la revue R2 est retenue ; le correctif minimal a maintenant 71 PASS ciblés, RC0, sources 214 stables.
Le reçu post-cwd ci-dessous remplace l’attente des tests ; aucun hash futur
de source ou résultat global n’est inventé. Converge P3 est un
checkpoint historique antérieur au correctif cwd (03:23 selon le principal).
La vérification du delta après audit reste attendue. Aucun audit final,
livraison ou GREEN workspace global n’est déduit.

La recette réelle canonique reste immuable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-postfix-final.json.
SHA256 : 153b70cbbbaa94315b87f192a8fa67fc217f607eb1cc70c5e9ea5b047b6cc9f3.
Elle atteste quatre cas PASS et executed=true. Son Human frais était réalisé
après H1/H2/H3, mais avant le correctif cwd. Les trois autres cas gardent leurs
captures et ancien binaire. Ces quatre cas ne sont pas déclarés sur le dernier
binaire incluant cwd. Le delta cwd est validé uniquement par les ciblés avec
fournisseur synthétique lorsqu’ils sont reçus ; aucune nouvelle recette de
modèle, attente réelle de 600 s, suite complète ou permission utilisateur n’est
exigée ou annoncée dans ce passage documentaire.

Converge P3 historique :
/Users/moi/.cache/bridget-delivery155.um4xccv4/converge155-p3.json.
SHA256 : 74a0df9585f1d32b0811c54ed289a6e5d857370b1eaf65a195827d6ad9dce129.
Les 71 PASS C1 et quatre régressions distinctes conservent leur propre snapshot
avant cwd. La suite unique 2259/15/72 n’est pas rejouée ni transformée en GREEN.

Projection cwd retenue pour le delta :

Le dossier fournisseur `cwd` est cité, sans lecture ni résolution filesystem.
Une chaîne est conservée entière jusqu’à 1024 octets UTF-8 et affichée avec
Debug (`{:?}`), entre guillemets et avec les contrôles/bidi visibles. Absence
ou `null` donne `facts.cwd=null` ; `facts.cwd_verified=false` dans tous les cas.
Un autre type ou une chaîne hors borne donne `details_incomplete`. La borne
1024 octets s’applique séparément à la commande et à cwd. Le budget total des
faits reste 4096 octets ; son dépassement reste un refus fermé. Aucun fallback
vers task.cwd, dossier du parent ou dossier courant n’est fabriqué. Ce dossier
cité ne prouve ni l’exécution dans ce dossier ni un confinement ou un droit.

## Preuve ciblée post-cwd — 71 PASS courants

native155-cwd-r1 : RC0, 64,81 s, 71 PASS / 0 FAIL (63 daemon, 1 E2E,
7 transport). Les 214 empreintes avant/après sont strictement égales.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/native155-cwd-r1.json.
Log : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/native155-cwd-r1.log.
SHA256 du log : 2be93808d3211bfe126877fec072bae9eba5c2d8652f575ac3c353a531d7ba16.
Le correctif cwd a maintenant sa preuve synthétique sur le code courant.
La capture et la relecture du delta après P3 appartiennent au principal.

Les quatre régressions canal087/STDIO restent des preuves distinctes avant cwd ;
aucun total de 75 PASS sur le dernier snapshot n’est revendiqué. La recette réelle
canonique 153b70 reste validée sur ses provenances antérieures à cwd, avec Human
après H1/H2/H3. Aucun nouveau runtime de modèle, second délai réel de 600 s ou second
FULL n’est lancé ou déduit. Statut In Progress, 42/42 et T040 acquise ; audit
final, relecture du delta, revue finale et livraison restent distincts.
