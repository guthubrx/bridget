# Implémentation155 — livrée et activée le 2026-10-12

Statut courant : Livré, 42/42. Source 59da3968c7d14dbd7acd1370875dd9d135ba7dcb
intégrée/poussée, binaire signé installé et runtime actif. Trois Converge et
contrôle cwd ciblé acquis ; audit final 2 LOW, couverture 27,46 %, validateur 0/0.
WT/branche retirés ; clôture documentaire sans changement du binaire.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/livraison155.md.
Les checkpoints ci-dessous sont historiques, sans GREEN workspace global.
Checkpoint historique R2 : In Progress, 42/42 acquises, T040 validée sur le reçu
réel antérieur au correctif cwd. Ce correctif a maintenant 71 PASS ciblés sur le code courant. Converge P3 est historique ; relecture du delta après
audit attendue. Les anciens statuts et PASS ci-dessous gardent leur snapshot.
État historique : implémentation réalisée à 100 %, 42/42 tâches cochées par le
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
État : 41/42 tâches cochées par le principal ; T039 et T041 documentaire acquises.
T039 est validée et cochée par le principal sur les preuves cumulées vérifiées.
Seule T040 (recette avec validation explicite utilisateur) reste ouverte.
Ce rapport ne vaut ni convergence finale, audit de code final ou livraison155.
Aucune case n’est modifiée par son rédacteur.

## Résultat et périmètre

Le parcours155 ajoute les questions user_input au parent et les approbations
de commande admissibles à l’humain, sur option child_requests=forward Codex.
Le défaut reste deny. Les droits figés et la politique never restent inchangés.
Le parent n’approuve aucune action. FileChange, permissions, writeStdin, tout
formulaire MCP tiers et les effets non établis restent refusés en v1, conformément
au contrat T002. L’exception historique du formulaire vide Bridget est distincte.

La décision historique est séparée de sa remise. CAS, échéance et FR027 sont
recontrôlés avant émission. Fermeture et réponse sont sérialisées ; les replays
n’inventent pas une nouvelle décision. delivered reste livré après un échec de
renvoi du reçu. Les décisions humaines pendant pause sont acquittées sans
exécution ; la reprise porte sur une nouvelle demande. L’attente151 reste
fondée sur l’état, avec pending_requests racine Parent seulement.

## Limite explicite de l’avis Parent — FR018

Le libellé initial « Une question ouvre un avis » garantissait implicitement
un succès que le parcours ne garantit pas. L’avis est tenté par SendAs avec
les gardes149 et leur disjoncteur ; le refus du neuvième avis a été observé
par root dans la fixture de budget (8 échanges sur 180 s). FR018 est clarifiée
pour décrire cette tentative, sans contourner les gardes ni ajouter un renvoi.

La demande est persistée avant la tentative. Un refus d’avis ne la perd pas :
elle reste consultable par le propriétaire dans Status et dans Wait si elle
est Parent, ouverte, non expirée et actionnable. La publication de l’avis et
la conservation de la demande sont deux faits distincts. Cette limite est
consignée, sans revendiquer une notification réussie pour chaque question.

## Relecture T038 et capture acquises

Le principal indique avoir lu le core complet, le magasin, le classifier et le
formatter, les blocs critiques du transport (normalize/answer/forward/resolved)
et le diff daemon réel. La revue native T038 a examiné les deux helpers et les
bornes de présentation décrites ci-dessous. Le commentaire du writer a été
corrigé par root pour refléter le verrou state conservé jusqu’au write afin de
figer les faits FR027 ; root atteste zéro changement sémantique de cette correction.

Root a reçu la seconde lecture complète de 21 fichiers Rust suivis et de 7 fichiers
Rust nouveaux par l’owner audit_existant151. Aucun autre problème de sécurité
ou de course n’est confirmé dans ce périmètre ; la réserve FR018 ci-dessus est
explicite. Le principal a clôturé et coché T038 après cette lecture et la capture
du patch complet, le 2026-10-12. git diff --check est RC0 selon son reçu. La
seconde revue source ne vaut pas audit final ni couverture runtime générale.

Capture vérifiée par lecture des octets : 2 300 233 octets, SHA256
667e0fab1cdee0dafd5fe2ca86bbe4c2db74ff5fbc2a327bbbdd815a5618f88f.
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/diff-155.patch.
Cette empreinte décrit la capture root antérieure à la présente actualisation
documentaire ; elle ne prétend pas être le hash d’un patch recapturé ensuite.

## Six réponses finales XIX/XX

| Question constitutionnelle | Réponse consolidée de la revue finale source |
|---|---|
| Pourquoi nécessaire ? | Corréler les demandes fournisseur à l’enfant et remettre une décision sans perdre la clôture, inventer un droit ou exécuter un accord devenu invalide. |
| Pourquoi plus simple ou maintenable ? | Réutiliser le magasin natif, les gardes149, SendAs, la boîte humaine et WaitEpoch ; un parcours corrélé, sans service/framework supplémentaire. La duplication légère des présentations est bornée et justifiée ci-dessous. |
| Quelles hypothèses ? | Formes primaires Codex 0.162.1, identité/session/tour attestés, refus des effets non établis ; isSecret normatif user_input ; avis tenté et non garanti ; attente fondée sur l’état. |
| Qu’est-ce qui a été vérifié ? | Lecture complète de 21 fichiers Rust suivis et 7 nouveaux et blocs critiques root ; CAS, ordre writer/state jusqu’au write, invalidations et règlement après STOP. Tests ciblés/logs et preuves cumulées ci-dessous ; aucun accord invalidé ne crée une nouvelle exécution dans les chemins relus. |
| Qu’est-ce qui n’a pas été vérifié ? | Recette utilisateur T040, activation réelle du parcours155, audit/Converge finaux et performance runtime générale. |
| Qu’aurait-on pu supprimer ? | Potentiel connu : 0 ligne suppressible à comportement constant dans le périmètre relu. due_child_requests est l’API T005 testée ; format_cited_paths est le helper pur T009 testé. Les garder ne revendique pas un lint vert. |

Ces réponses rapportent la revue audit_existant151 transmise par root. Elles
portent sur les sources lues, sans déclarer des effets runtime non observés.


## Réutilisation et helpers conservés

| Surface | Justification |
|---|---|
| due_child_requests | API prévue par T005 pour les échéances, avec index et sortie bornée à 512 ; le test du magasin vérifie 199 vide puis 200 dû |
| child_request_followups dans le tick | Doit aussi reprendre sent et les clôtures none/void non confirmées ; prédicat OR sur l’index existant, coût documenté O(n + B). LIMIT 512 borne la projection, pas le scan potentiel complet ; la requête des seules open échues ne couvre pas cette responsabilité |
| format_cited_paths | Formatter pur T009 testé, 64 valeurs maximum et sortie 4096 octets UTF-8 ; aucune lecture filesystem ni autorité de droits |
| Présentation des chemins dans le core | Normalisation lexicale de 64 valeurs, budget local 1024 octets et faits complets plafonnés à 4096 octets ; contrôles remplacés et paths_verified=false |

Le formatter produit une phrase d’affichage avec séparateurs et signal de coupe.
Le core produit un tableau borné à l’intérieur des faits JSON. Leurs budgets et
leurs formes diffèrent. Raccorder le formatter à cette structure demanderait une
adaptation plus large sans supprimer la normalisation lexicale ni le contrôle
de taille du JSON. La légère duplication de présentation reste explicitement
justifiée ; aucun helper, cache, service ou framework nouveau n’est ajouté.
Les chemins cités ne servent jamais de preuve de confinement.

## Minimisation du formatage

Le reçu du principal atteste 47/47 comparaisons canoniques exactes avant/après,
26 restaurations HEAD et 21 fichiers mixtes minimisés par Git --theirs. Les sources
étaient stables après l’écriture. Le diff Rust suivi passe de +17985/−3517 lignes
à +4664/−71. Ces chiffres décrivent le retrait du formatage historique ; ils
ne sont ni une mesure de couverture ni un nouveau résultat de test.

Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/format155-minimization.json.
SHA256 : 75ddc20ac700eadcd4f22ca670536d9e67a866aa9044acf4a584f8d0b9261b64.
La lecture de ce reçu ne constitue pas un lancement Git/rustfmt par le rédacteur.

## Tests acquis et limites

La sélection requests réussit cumulativement 42 cas : 40 PASS du runR2, puis les
2 FAIL de fixture devenus 2 PASS en R3. Les deux corrections concernent les fixtures,
avec production inchangée entre ces snapshots. E2ER1 a échoué à compiler ;
E2ER2 apporte 1 PASS distinct après annotation de type du tuple. Les trois
unitaires de listes sûres et l’intégration Claude sont PASS enR2. T037 est
cochée par le principal après correspondance des guides aux outils et vues.

Ces résultats restent des preuves ciblées sur leurs sources historiques. Ils
ne prouvent ni une seconde suite ni un GREEN workspace global. La capture T038
est acquise. La suite unique lancée le 2026-10-12 à 01:47:46 UTC est terminée :
2259 PASS / 15 FAIL / 72 ignored, RC101 en 554,43 s, 94 cibles complètes et
214 sources stables. Les 180/180 tests de baseline sont PASS avant correctifs.
Les 15 échecs se répartissent en 8 historiques et 7 nouveaux. Les comparaisons
root sont copiées à l’identique dans la validation ; le rapport de tests donne
leurs chemins, les noms et les empreintes.

Les replays ciblés apportent 71 PASS natifs (63 daemon, 1 E2E, 7 transport),
2 PASS annulation152, puis les confirmations unitaires annulation148, matrices
MCP155/151, vue154, skill089 et099. Chaque run a ses 214 sources stables. Les
sept nouveaux échecs ont un replay PASS. Le dernier099 sans diagnostic
temporaire est confirmé : RC0, 1 PASS, 8,27 s, 214 sources stables.
T039 est acquise après validation des preuves cumulées par le principal. Les résultats
ciblés ne changent pas le résultat de la suite antérieure aux correctifs.

Le correctif cancel_tree retire la mutation anticipée des descendants ; leur
transition est conservée après classification de tous les membres. linked_task
contrôle au plus sept ancêtres et refuse un accord si un ancêtre annule.
L’extension du scénario chaîne existant couvre ce chemin, sans nouveau scénario.
Ces corrections ont une preuve ciblée ; elles ne valent pas GREEN global.
Le budget de cache 40 Gio était une variable de processus, pas une configuration
durable. T040 exige toujours la validation explicite de l’utilisateur.

Deux warnings restent non bloquants : due_child_requests et format_cited_paths
sont signalés inutilisés dans la cible de production compilée, tout en étant
prévus et testés. Aucun succès de lint n’est revendiqué. Les motifs de maintien
sont consignés dans reuse-audit et la notice dédiée.

Preuves ciblées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/tests-155.md.
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/requests155-cumulative-gate.json.

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

Le refus réel workspaceWrite est PASS dans la même recette : tâche
8512b027-5afc-4596-bfb9-a8aa5b6ecf7d, demande
7e0ea0ce245919d60b52e93632199e2d, verdict forbidden, outcome
refused:command_unbounded, décision decline remise (delivered), sans item
humain ; parent_sandbox=workspaceWrite. Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/recette155-final-runtime-open.json.
SHA256 : 8dee7fc2f7776e5ebb45dec7aa14d1917f093fd14500dade240e92e69e535dcd.
T039 est acquise sur preuve cumulative ; seule T040 reste ouverte.

## T041 — correction documentaire du journal, acquise

Le principal a ajouté, réalisé et coché T041. La correction porte uniquement
sur le paragraphe Journal du contrat : provider_request décrit l’attente
pending et l’empreinte de l’identifiant fournisseur. L’issue et le décideur
restent dans la table SQLite native_child_requests, conformément à FR030.
Aucun code ou test n’a changé pour T041.

Lecture ciblée du transport :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-transport/src/codex_app_server.rs,
lignes 165–178 : PendingProviderRequest.payload écrit state=pending ;
lignes 4266–4310 : record_provider_request projette l’identifiant fournisseur,
empreinte le tour et enregistre l’événement provider_request.
Lecture de la table :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/delegation.rs,
lignes 307–327 : native_child_requests conserve state, outcome, decision,
answer_json et decided_by. Le journal fournisseur ne reçoit aucune valeur
nouvelle de réponse ou de décision.

Checkpoint historique : In Progress, 41/42 ; T041 documentaire et T039 acquises,
T040 seule ouverte. Ce checkpoint remplace les statuts historiques 40/41
conservés dans ce reçu. Les résultats de tests antérieurs restent inchangés.

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

État historique : In Progress, 42/42 tâches acquises, T040 validée. L’objection
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
