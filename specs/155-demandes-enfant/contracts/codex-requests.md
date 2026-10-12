# Contrat fournisseur T002 — Codex app-server 0.162.1

Date : 2026-10-12. Source primaire : schémas JSON générés par le CLI installé,
avec `--experimental`, car Bridget annonce `capabilities.experimentalApi=true`.
Ce contrat décrit les formes lues. Il ne prouve aucune exécution de modèle.

Archive primaire :
/Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1

Commande, RC0, durée0,12s et empreintes des447 fichiers :
/Users/moi/.cache/bridget-delivery155.um4xccv4/schema155-generation.json

La version0.161.0 des documents préparatoires est historique. La référence
effective de T002 est0.162.1. Les délais Bridget600/3600s ne sont jamais la durée
d'un octroi Codex.

## Enveloppe et fermeture

Les cinq demandes sont des requêtes `{id,method,params}`. `RequestId` vaut une
chaîne ou un entier signé64bits, repris sans conversion. Sans identifiant,
aucune réponse n'est inventée. `serverRequest/resolved` est une notification,
pas une demande. Les schémas de paramètres ne sont pas tous des objets fermés :
la validation155 doit refuser tout champ d'effet hors de sa liste blanche.

Les erreurs utilisent le schéma `JSONRPCError` :
`{"id":<id>,"error":{"code":-32000,"message":"child_request_refused"}}`.
Aucun `result` ne l'accompagne. Le motif interne reste distinct :
`approval_method_unsupported`, `duration_unsupported`, `details_incomplete`,
`secret_not_forwardable` ou `url_mode_unsupported`, selon la garde déclenchée.
Une erreur JSON-RPC ne prouve pas que le fournisseur a clos son attente.

| Méthode | Décision155 | Réponse fournisseur et limite |
|---|---|---|
| `item/commandExecution/requestApproval` | Support conditionnel, Human seulement après gardes155 | `{decision:"accept"|"decline"|"cancel"}`, chaque valeur explicitement offerte ; accept courant seulement |
| `item/fileChange/requestApproval` | Refusée en v1 : approval_method_unsupported ; grantRoot non nul : duration_unsupported | Refus JSON-RPC ; aucune liste offerte ni effets ajoutés pour obtenir un accord |
| `item/permissions/requestApproval` | Refus `approval_method_unsupported` | Réponse native `{permissions,scope?,strictAutoReview?}` ; aucune décision decline/cancel |
| `mcpServer/elicitation/request` | TOUT formulaire MCP tiers refusé en v1 : approval_method_unsupported ; exception vide Bridget historique distincte | `{action:"accept"|"decline"|"cancel",content?,_meta?}` est la forme native ; aucune preuve structurée non-secret/non-approval, donc aucun accept Parent pour un tiers |
| `item/tool/requestUserInput` | Parent pour questions complètes, non secrètes | `{answers:{<qid>:{answers:[<string>,...]}}}` ; aucun decline/cancel natif, refus par JSON-RPC |
| `serverRequest/resolved` | Notification de règlement, jamais Parent/Human | `{threadId,requestId}` ; aucun turnId, aucun résultat ni preuve d'acceptation |

## Commande : paramètres, effets, décisions et durée

Type primaire `CommandExecutionRequestApprovalParams`. Champs requis :
`threadId`, `turnId`, `itemId` (chaînes), `startedAtMs` (int64). Champs optionnels
et nullables : `approvalId`, `availableDecisions`, `command`, `commandActions`,
`cwd`, `environmentId`, `additionalPermissions`, `networkApprovalContext`,
`proposedExecpolicyAmendment`, `proposedNetworkPolicyAmendments`, `reason`.
`kind` est optionnel, défaut `command`, valeurs `command` et `writeStdin`.
`approvalId` peut distinguer plusieurs callbacks d'un même item, notamment
zsh-exec-bridge ou stdin ; il ne remplace pas l'id JSON-RPC.

Liste blanche de champs d'effet et formes :

- `command`, `cwd` : commande et dossier cités. Les deux chaînes ont chacune
  une borne de 1024 octets UTF-8. `cwd` est chaîne ou `null` ; absence signifie
  `null`. Toute autre forme ou longueur hors borne donne `details_incomplete`.
  Le rendu `facts.cwd` est Debug entier avec contrôles visibles, ou `null` ;
  `facts.cwd_verified=false`. Aucun fallback ou preuve de confinement.
  `commandActions` est un affichage best-effort (`read`, `listFiles`, `search`,
  `unknown`), pas une preuve d’effets. Le budget des faits reste 4096 octets.
- `additionalPermissions.network.enabled` : booléen nullable, overlay par
  commande. `fileSystem` peut contenir `read`/`write` (listes de chemins cités),
  `entries` et `globScanMaxDepth` (entier>=1). Les formes nouvelles ne sont pas
  aplaties silencieusement en une simple racine autorisée.
- Une entrée `fileSystem.entries[]` est `{access,path}`, avec
  `access=read|write|deny` et `path={type:"path",path:string}` ou
  `{type:"glob_pattern",pattern:string}` ou `{type:"special",value:{kind,...}}`.
  `kind` peut être `root`, `minimal`, `project_roots` (+subpath?), `tmpdir`,
  `slash_tmp`, `unknown` (+path et subpath?). Toute portée non établie demeure
  refusée ; aucun chemin cité ne prouve son confinement.
- `networkApprovalContext={host:string,protocol:http|https|socks5Tcp|socks5Udp}`.
- `proposedExecpolicyAmendment` est une liste de chaînes pour les commandes
  futures. `proposedNetworkPolicyAmendments[]={action:allow|deny,host:string}`
  propose des règles futures. Ce sont des propositions, jamais des accords155.
- Correction runtime prouvée par les tests ciblés : environmentId absent/null ou
  chaîne exacte "local" correspond au lancement local attesté ; les autres valeurs
  non nulles restent refusées. kind=writeStdin reste refusé ; aucune portée
  étrangère n’est inventée. La recette Human complète reste ouverte.

`CommandExecutionApprovalDecision` accepte nativement :

1. `"accept"` : commande courante approuvée ; portée opérationnelle unique,
   par opposition explicite à la variante session et à l'overlay par commande.
2. `"acceptForSession"` : prompts futurs du même cache de session, **interdit155**.
3. `{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":[string,...]}}` :
   règles futures, **interdit155**.
4. `{"applyNetworkPolicyAmendment":{"network_policy_amendment":{"action":"allow"|"deny","host":string}}}` :
   règle réseau persistante, **interdit155**, même avec action deny.
5. `"decline"` : commande refusée, tour poursuivi.
6. `"cancel"` : commande refusée, tour interrompu.

T000 traite seulement les chaînes offertes avec as_array/filter_map(as_str).
Une offre absente, null, vide ou contenant seulement des décisions durables
n'autorise aucun choix inventé. Une offre mixte accept+objet d'amendement peut
choisir accept en mode allow ; l'objet est ignoré, jamais appliqué. En forward,
la normalisation de la demande conserve les informations nécessaires aux gardes
d'effet ; filtrer les décisions ne suffit pas à classifier une demande sûre.

## Fichiers : information manquante dans la demande

`FileChangeRequestApprovalParams` exige `threadId`, `turnId`, `itemId`,
`startedAtMs`. Il contient seulement `reason?` et `grantRoot?`, nullables.
Il ne contient **ni availableDecisions ni changes/paths**.
`grantRoot` non nul demande des écritures sous une racine pour le reste de la
session ; sa description précise que son application effective est incertaine.
Cette portée n'est pas once : duration_unsupported, jamais une autorisation Human implicite.

`FileChangeApprovalDecision` définit `accept`, `acceptForSession`, `decline`,
`cancel`. acceptForSession autorise de futurs changements des mêmes fichiers.
accept concerne les changements courants ; cette distinction ne fournit pas
les chemins ni tous les effets absents de Params. Arbitrage du principal :
**fileChange reste entièrement refusée dans la v1 de155** avec
`approval_method_unsupported`, ou `duration_unsupported` si grantRoot est non nul.
Le refus fournisseur est l'erreur JSON-RPC child_request_refused. Ne fabriquer
ni offre decline/cancel ni réponse accept à partir de l'énumération de réponse.

Les types `ItemStartedNotification` et `ItemCompletedNotification` décrivent
`FileChangeThreadItem{type:"fileChange",id,status,changes}`. `changes[]` contient
`{path,diff,kind}` ; `kind` vaut `{type:"add"}`, `{type:"delete"}` ou
`{type:"update",move_path?:string|null}`. Une corrélation exacte
threadId/turnId/item.id peut fournir des chemins cités, y compris move_path.
Elle ne prouve pas l'existence ni le confinement de ces chemins. Cette piste
concerne une évolution future ; la v1 n'ajoute aucun cache ni corrélation pour
autoriser fileChange. Ce refus ne décrit pas une limitation définitive de Codex.
La table pure de politiques peut toujours tester un effet synthétique d'écriture
établi : sous workspaceWrite, FR011 impose `file_confinement_unproven` ; sous
dangerFullAccess, le verdict générique Human ne rend pas cette méthode supportée.

Compatibilité T000 : l'absence normale de availableDecisions déclenche l'erreur
fermée T000. Ne pas fabriquer une liste offerte à partir de l'énumération de
réponse. Les fixtures T000 ont une liste explicite ; cela n'atteste pas que le
serveur réel la transmet pour fileChange.

## Permissions : réponse différente, octroi de tour ou de session

`PermissionsRequestApprovalParams` exige `threadId`, `turnId`, `itemId`,
`startedAtMs`, `cwd`, `permissions`. `environmentId?` et `reason?` sont nullables.
`permissions` est un objet fermé `RequestPermissionProfile{fileSystem?,network?}`
utilisant les formes d'effet ci-dessus. Il ne comporte pas availableDecisions.

Réponse : `{"permissions":<GrantedPermissionProfile>,"scope":"turn"|"session","strictAutoReview":bool|null}`.
Seul permissions est requis ; scope défaut turn. strictAutoReview demande une
revue de chaque commande suivante de ce tour avant exécution normale.
Les scopes turn/session sont hors once. Aucun `{decision:"decline"}` ou cancel
n'existe dans le schéma de réponse. Même un profil vide ne remplace pas une
réponse de refus prouvée.155 refuse par erreur JSON-RPC et conserve le motif
`approval_method_unsupported`. T000 ne peut pas approuver cette demande sans
une offre explicite ; ne pas étendre sa réponse générique à cette méthode.

## Élicitation MCP : cinq modes, métadonnées non établies

Champs communs requis : `serverName`, `threadId`. `turnId?` est nullable ; sa
description explique qu'une élicitation MCP peut être autonome, sans tour.
Sans tour exactement corrélé à l'enfant,155 refuse `request_unlinked`.

| Mode | Champs propres requis | Politique155 |
|---|---|---|
| `form` | message, requestedSchema typé | Refus approval_method_unsupported, même typé/non vide/sans _meta : aucune preuve structurée non-secret/non-approval |
| `openai/form` | message, requestedSchema JSON quelconque | Effets/forme non établis : approval_method_unsupported |
| `openaiForm` | message, requestedSchema JSON quelconque | Même refus, alias distinct constaté |
| `url` | message, elicitationId, url | url_mode_unsupported ; aucune ouverture de navigateur |
| `openai/userVerification` | challenge, description, title | Approbation authentifiée par appareil, preuve dans content ; approval_method_unsupported, jamais Parent |

Dans chacune des cinq variantes, `_meta` est optionnel et son schéma vaut
exactement `true` : **n'importe quelle valeur JSON**. Le schéma ne définit
aucune clé `_meta.approval`, `_meta["openai/approval"]` ni structure d'octroi.
Ces exemples ne sont pas des formats établis.155 n'invente pas de marqueur :
metadata absente/null/objet vide n'ajoute pas d'effet connu ; toute metadata
non vide ou autre forme dont l'effet n'est pas établi demeure Forbidden.
TOUT formulaire MCP tiers ne devient jamais une question Parent en v1, y compris un formulaire typé complet sans métadonnée. Aucune heuristique textuelle ou de bool/enum ne remplace une preuve structurée non-secret/non-approbation. L'exception historique du formulaire vide du serveur Bridget reste
séparée ; elle n'autorise pas les autres serveurs ou un octroi non établi.

`McpElicitationSchema` est fermé : `{type:"object",properties:{...},required?:[string]|null,$schema?:string|null}`.
Ses propriétés typées acceptent chaîne (format email/uri/date/date-time), nombre
ou entier, booléen, sélection unique ou multiple. Les enums peuvent être
`enum:[string]`, `oneOf:[{const,title}]` ou le format historique enum+enumNames ;
les multi-sélections sont des arrays avec items enum/anyOf et minItems/maxItems.
Les champs inconnus et formes de secret/effet non établies restent refusés.
Aucun indicateur isSecret n'est défini pour ces propriétés MCP : ne pas conclure
qu'un texte demande une donnée non secrète sur la seule absence d'un marqueur.

Réponse native : `{action:"accept"|"decline"|"cancel",content?:JSON,_meta?:JSON}`.
Elle ne prouve pas qu’un formulaire tiers est une simple question : même bool/enum
peut autoriser un outil. TOUT formulaire MCP tiers reste refusé en v1, sans contenu
d’accord, avec motif approval_method_unsupported. Ne pas recopier une
metadata d'octroi. Ces actions règlent une sollicitation de données ; elles ne
constituent pas une preuve générale des effets du serveur MCP tiers.

## Saisie utilisateur : enveloppe exacte des réponses

`ToolRequestUserInputParams` exige `threadId`, `turnId`, `itemId`, `isBlocking`
et `questions` (array). `autoResolutionMs?` est uint64 nullable, déprécié ;
il ne définit ni l'échéance Bridget ni une durée de permission.
Chaque question exige `id`, `header`, `question`. `isOther` et `isSecret`
défaut false ; `options?` est une liste nullable de `{label,description}`.
isSecret=true donne `secret_not_forwardable` ; question incomplète ou
identifiants non corrélés restent refusés. isBlocking false n'est pas un droit.

Réponse exacte :
`{"id":<id>,"result":{"answers":{"q1":{"answers":["valeur"]}}}}`.
Chaque valeur de answers est un **objet** `{answers:[string,...]}`, pas un array
direct. L'API Bridget `answer={"q1":["valeur"]}` reste distincte et doit être
enveloppée à la frontière fournisseur, après validation des ids et de la borne8Kio.
Le schéma n'impose pas minItems, ni nombre exact d'answers, ni la sélection dans
options ; les règles155 ne doivent pas être présentées comme contraintes Codex.

Aucune décision decline/cancel n'est définie par cette réponse. Pour annulation,
expiration, refus ou réponse invalide : erreur JSON-RPC child_request_refused,
jamais `{decision:"cancel"}`, `{answers:{}}` ou une réponse vide inventée pour
simuler un refus. La confirmation fournisseur de fermeture reste distincte.

## Notification de règlement

`ServerRequestResolvedNotification` exige `threadId` et `requestId` (RequestId).
Pas de turnId, itemId, verdict, durée ou contenu de réponse. La corrélation
retrouve la requête ouverte déjà attestée pour cette session/thread et cet id ;
elle ne devine pas un tour à partir du texte et ne prouve pas un accord exécuté.
La notification ne reçoit aucune réponse JSON-RPC.

## Corrections documentaires nécessaires avant T003

- Remplacer la référence générée0.161.0 par0.162.1 ; conserver l'historique T000.
- Ne pas supposer availableDecisions pour fileChange ou permissions.
- Ne pas coder un refus permissions avec decision : utiliser l'erreur JSON-RPC.
- Distinguer answer Bridget et l'enveloppe answers[qid].answers fournisseur.
- À l'expiration, utiliser le refus propre à la méthode : userInput n'a pas cancel.
- _meta approval n'est pas établi ; modes enrichis/authentifiés et tiers vide
  restent refusés. Les formulaires MCP n'ont pas de drapeau secret standard ici.
- Les champs requis startedAtMs/isBlocking et la notification sans turnId doivent
  remplacer les fixtures simplifiées quand celles-ci prétendent au schéma réel.

T002 n'est pas cochée par ce propriétaire. Aucun code, test, service ou modèle
n'a été exécuté pour rédiger ce contrat.

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
modèle, attente réelle600s, suite complète ou permission utilisateur n’est
exigée ou annoncée dans ce passage documentaire.

Converge P3 historique :
/Users/moi/.cache/bridget-delivery155.um4xccv4/converge155-p3.json.
SHA256 : 74a0df9585f1d32b0811c54ed289a6e5d857370b1eaf65a195827d6ad9dce129.
Les 71 PASS C1 et quatre régressions distinctes conservent leur propre snapshot
avant cwd. La suite unique2259/15/72 n’est pas rejouée ni transformée en GREEN.

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
