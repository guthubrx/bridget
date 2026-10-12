# Preuve T002 — formes fournisseur lues

Date : 2026-10-12. CLI installé : codex-cli0.162.1.
Génération exécutée par le principal : RC0,0,12s,447 fichiers, --experimental.
Justification : initialize.capabilities.experimentalApi=true dans le pilote.
Ce propriétaire a seulement lu les fichiers et écrit les deux documentsT002.

Archive primaire :
/Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1

Commande et empreintes primaires :
/Users/moi/.cache/bridget-delivery155.um4xccv4/schema155-generation.json

Contrat figé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/contracts/codex-requests.md

| Surface | Lecture | Décision |
|---|---|---|
| CommandExecution Params/Response + toutes leurs définitions | LU | accept par commande seulement ; session/amendements interdits ; variantes d'effet non établies refusées |
| FileChange Params/Response + FileChangeThreadItem/FileUpdateChange/PatchChangeKind dans ItemStarted/Completed | LU | Refus v1 approval_method_unsupported ; grantRoot non nul duration_unsupported ; corrélation seulement piste future |
| Permissions Params/Response + profils/grantScope | LU | turn/session, réponse sans decision : méthode refusée |
| MCP Elicitation Params/Response + toutes les formes primitives/enums | LU | cinq modes ; TOUT formulaire tiers refusé v1 approval_method_unsupported, absence de preuve structurée non-secret/non-approval ; exception vide Bridget distincte |
| ToolUserInput Params/Response + Question/Option/Answer | LU | enveloppe answers[qid].answers ; secret refusé ; refus JSON-RPC |
| ServerRequestResolvedNotification + dispatch ServerNotification | LU | threadId/requestId seuls, notification sans réponse |
| ServerRequest, RequestId, JSONRPCError | LU | id string/int64 ; erreur -32000 sans result |
| Sémantique exacte de clés approval dans _meta | NON ÉTABLIE | Le schéma est true/arbitraire : aucune clé typée ; ne pas prétendre à une forme vérifiée |
| Runtime hors capture locale ci-dessous, délai fournisseur, effet MCP tiers, compat CLI0.161.0 | NON LU/NON TESTÉ | Correctif local GREEN ciblé ; demande Human réelle ouverte ci-dessous ; décision utilisateur et validation runtime générale non acquises |

Reproduction lecture : ouvrir les schémas primaires et comparer properties,
required, descriptions et definitions aux entrées du contrat. Les SHA256 de
chaque fichier sont dans le manifeste de génération. Le contrat ne modifie pas
ce manifeste ni les schémas. T000 reste une preuve distincte du parcours forward.

Contradictions transmises au principal et au propriétaire des autres docs155 :
fileChange sans liste offerte/effets dans Params, permissions sans decision,
userInput sans decline/cancel et avec enveloppe imbriquée, elicitation cinq
modes/_meta arbitraire, notification resolved sans turnId, version réelle0.162.1.

T002 a été acceptée et cochée par le principal, reçu gate-schema155.json.
Les fondations et le parcours ont leurs preuves ciblées. Le rédacteur ne
modifie aucune case et n’exécute aucune commande Cargo/Git/service/modèle.

## Arbitrage primaire final T002 — formulaire MCP tiers fermé en v1

Aucune forme MCP ne fournit une preuve structurée de non-secret et de
non-approbation. Même bool/enum peut accepter un outil. FR013 et le gate T002
imposent donc approval_method_unsupported pour TOUT formulaire MCP tiers en v1,
même form typé, complet, non vide et sans _meta. Aucun heuristique textuel,
marqueur inventé ou route Parent. L’exception historique vide Bridget reste
distincte. user_input seul remonte au parent, avec isSecret normatif défaut false ;
isSecret=true reste secret_not_forwardable. Les modes URL restent refusés.

Les mêmes scénarios T014 sont des refus MCP et des réponses user_input ; T015
applique cette frontière, sans nouvel identifiant ni nombre de tests/scénarios.
Checkpoint final : T002 est PASS et cochée par root, attestée dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/gate-schema155.json.
Les 41 tâches sont conservées ; leur état courant reste celui de tasks.md.
Les mentions antérieures de formulaire candidat Parent sont supersédées.
Aucun code, helper/API, test exécuté ou commande Cargo/Git/service.

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

La suite unique commencée à 01:47:46 UTC est terminée avant ce correctif :
2259 PASS / 15 FAIL / 72 ignored. Ses 8 échecs historiques et 7 nouveaux
sont séparés des replays ciblés ultérieurs. Aucune seconde suite complète
ni aucun GREEN global n’est revendiqué. T038 et 40/41 restent acquis ; T039 est acquise ; seule T040 reste ouverte. Aucun nouveau
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
