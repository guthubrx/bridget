# Recherche - Session155 remontée des demandes de l'enfant

Date : 2026-10-11. Source : lecture du checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`), lecture seule, et du
code T3 `origin/main` par `git show`. Ce checkpoint historique supposait150 livrée ; les acquis150–154 sont désormais livrés. Le passageR16 ci-dessous remplace les hypothèses fournisseur. Les
lignes citées sont celles du checkout principal.

## R1 - Aujourd'hui, une approbation Codex est tranchée par un réglage

- **Constat** : `approval_response` choisit `accept` si `permissions == "allow"`,
  sinon `decline`, parmi `availableDecisions`
  (`crates/bridget-transport/src/codex_app_server.rs:2195-2226`). L'appel est dans la
  boucle de lecture (`:3344-3366`), et la réponse est écrite sur place.
- **Constat de sécurité (ronde 2)** : si la décision voulue manque dans
  `availableDecisions`, `approval_response` prend la première décision proposée
  (`codex_app_server.rs:2209-2213`). Avec une offre `["acceptForSession"]`, le mode `deny`
  actuel écrit un accord durable. Défaut préexistant en production, hors du mode
  `forward`. Correctif T000, livrable seul.
- **Constat** : les enfants natifs ont `permissions = deny`
  (`crates/bridget-daemon/src/daemon/native_delegation.rs:105-108`). Donc toute
  approbation est refusée.
- **Conséquence** : pour attendre, il faut sortir la réponse de la boucle de lecture.
  Une attente dans la boucle bloquerait aussi `turn/completed` et les notifications
  (R6).

## R2 - Aujourd'hui, une question est refusée ou inconnue

- **Constat** : `mcp_elicitation_response` accepte seulement le serveur `bridget` en
  formulaire sans champ requis ; tout le reste est décliné (`codex_app_server.rs:2317-2353`).
- **Constat** : `is_approval_request` accepte toute méthode `item/<x>/requestApproval`
  (`codex_app_server.rs:2176-2181`), donc aussi `item/permissions/requestApproval` (faux
  serveur `:4864`, test `:4194`). Hypothèse historique : commandExecution et fileChange seraient transmises. Le passageR16 refuse entièrement fileChange réelle 0.162.1 en v1 ; commandExecution reste soumise au contrat et aux limites.
- **Constat** : aucune trace de `requestUserInput` dans le code, les specs ou les
  docs (recherche `grep -rn requestUserInput`). Une telle requête finit dans la
  branche « non prise en charge » (`codex_app_server.rs:3469-3500`).
- **Décision** : l'élicitation vide de Bridget reste acceptée automatiquement : elle
  sert à publier des artefacts. Le passageR16 restreint la remontée aux formes de question classiques établies, sans secret ni _meta non vide.
- **Hypothèse historique supersédée** : la forme exacte de `item/tool/requestUserInput`
  et de sa réponse est établie par la lecture0.162.1 du passageR16 ; T002 est désormais final PASS, reçu validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte.

## R3 - La politique de l'enfant n'est PAS modifiée (révisé après contre-revue)

- **Constat** : la politique de l'enfant copie `approval_policy` et `approvals_reviewer`
  du parent (`crates/bridget-daemon/src/native_permissions.rs:206-215`). Elle part à
  chaque tour (`codex_app_server.rs:1372-1374`).
- **Ancienne idée (rejetée)** : passer `never` vers `on-request` pour que le fournisseur
  demande. Une contre-revue (objection 4) montre que cela peut activer un relecteur
  automatique concurrent (`auto_review`, `guardian_subagent`) et que le seul effet
  utile était de refuser (le parent `never` ne peut rien accorder de plus).
- **Décision (FR-004)** : aucune modification de la politique de l'enfant. En mode
  `forward`, un parent dont `approvals_reviewer` n'est pas `user` est refusé à
  l'admission (`child_requests_reviewer_unsupported`). Un parent `never` donne un enfant
  `never` : le fournisseur ne pose pas d'approbation ; seules les questions remontent.

## R4 - Limite absolue : le bac à sable figé du parent, aussi pour l'humain (révisé)

- **Constat** : chaque tâche fige `permission_snapshot` avec `parent: ProviderPermissions`
  et `child_policy` (`crates/bridget-transport/src/native_permissions.rs:23-31`,
  `crates/bridget-daemon/src/delegation.rs:42`). Un changement de mode UI ne l'élargit pas.
- **Constat** : pour Codex vers Codex, `child_policy` copie la politique du parent et
  ne la réduit que pour la posture `discovery` (`readOnly`)
  (`native_permissions.rs:206-215`). Le parent ne « couvre » donc rien de plus que
  son enfant, sauf l'enfant `discovery`. Un parent modèle qui approuverait pour un
  enfant `discovery` lèverait une réduction qu'il a lui-même demandée (objection 3).
- **Décision** : les approbations vont à l'humain seul ; les questions vont au parent.
  Une approbation est acceptée par Bridget seulement dans la limite absolue, y
  compris pour l'humain (objection 1) :
  - enfant `readOnly`, parent `readOnly`, `externalSandbox` ou politique incomplète :
    rien ;
  - écriture de fichier : seulement si le parent a le plein accès (sous `workspaceWrite` :
    refusée, voir R13) ;
  - réseau : seulement si le parent l'a ; commande : seulement si le parent a le plein
    accès (les effets d'une commande ne sont pas bornés) ;
  - détail incomplet ou tronqué, champ d'effet hors liste blanche : rien.
- **Pourquoi pas l'humain sans limite** : l'ADR 149 interdit tout gain de droits. Un
  accord humain hors du bac à sable du parent en serait un.

## R5 - Canal vers le parent et vers l'enfant

- **Vers le parent** : l'avis suit le patron de l'avis d'échec (message système de
  propriétaire à propriétaire, identifiant déterministe, envoyé par `send_as`)
  (`native_delegation.rs:1040-1066`). L'identifiant `native-request-<id>` rend
  l'envoi idempotent.
- **Vers l'enfant** : le daemon cherche la route de l'agent puis la connexion, et
  pousse une trame de contrôle (patron de `ControlExecutionDispatch`,
  `crates/bridget-daemon/src/daemon.rs:9195-9240` et `wrapper.rs:4485-4495`). Le
  wrapper répond par une trame d'accusé. Le daemon réessaie à chaque tick (1 s,
  `daemon.rs:4700`) tant que l'accusé manque.
- **Vers le suivi d'exécution** : les états `waiting_approval` et `waiting_user_input`
  existent (`execution_store.rs:954`, `daemon.rs:3584`). Le wrapper les publie par
  `publish_execution_transition` (`wrapper.rs:5506`). On n'utilise pas
  `publish_approval_wait` : il coupe l'exécution après 3 demandes
  (`wrapper.rs:5700`, `approval_loop_detected`), ce qui contredit le budget de
  FR-007.
- **À vérifier en T015** : la transition `waiting_approval` vers `running` est
  acceptée par `transition_if_current`.

## R6 - Ne pas bloquer la boucle de lecture

- **Décision** : le transport garde une table des demandes en attente
  (identifiant JSON-RPC, session, tour, type, décisions proposées). Il émet un événement
  `ProviderRequest` et continue de lire. La réponse arrive plus tard par
  `ManagedSession::answer_provider_request`, qui écrit la réponse JSON-RPC.
- **Nettoyage** : fin de tour, interruption, arrêt, erreur d'écriture : la table est
  vidée. Un `cancel` est écrit si le canal vit.
- **Test** : SC-007.

## R7 - Boîte humaine : type existant, lecture par identifiant exact (révisé)

- **Constat** : la colonne `kind` a un `CHECK` fermé (`crates/bridget-daemon/src/human_inbox.rs:55-70`).
  Ajouter un type exigerait une migration. Décision : `intervention_required`, sujet
  `delegation_id` + `agent_id` (`protocol.rs:1344-1357`), clé `native-child-request:<request_id>`.
- **Constat** : `pending_decisions` rend au plus 200 items (`clamp(1, 200)`, `:281`),
  triés par `resolved_at` croissant (`:275-277`). Aucun code du producteur `Daemon`
  n'acquitte ses décisions (seules celles du `Guichet` sont servies,
  `daemon.rs:9836-9850`). Une relève par préfixe de clé resterait affamée par 200
  anciens items (objection 9).
- **Constat** : `close_self` ne ferme que les items `open` (`:342-346`). Un item déjà
  résolu par l'humain avant l'expiration ne se ferme pas.
- **Décision** : pas de relève. Le daemon lit chaque item lié par son identifiant exact
  (`get`, `human_inbox.rs:189`), pour au plus 512 demandes. Une décision arrivée après la
  fin de la demande est acquittée sans exécution (abandon durable). `inbox_settled`
  marque la fin du traitement.
- **Constat** : `daemon_deposit_human_inbox` rend `()` (`daemon.rs:9412-9440`) : le daemon
  ne connaît pas l'identifiant de l'item. Décision : elle rend le résultat. Une recherche
  par clé exacte avant dépôt réconcilie une panne entre dépôt et liaison (objection 10).
- **Limite connue** : la boîte ne porte que des choix fermés. D'où : aucune question ne
  va à l'humain.

## R8 - Sécurité du contenu (révisé)

- Le texte d'une demande vient du fournisseur de l'enfant, donc d'un modèle. Il peut
  contenir une injection. Un simple avertissement « non fiable » ne l'arrête pas
  (objection 8). Parades : (1) le parent ne répond plus aux approbations, donc aucun
  modèle n'approuve une action ; (2) l'humain voit d'abord les faits calculés par
  Bridget (commande citée, chemins cités et normalisés lexicalement sans résolution filesystem, réseau, durée), puis le texte du modèle
  cité à part, borné et nettoyé des caractères de contrôle et de direction ; (3) la
  demande doit venir du tour de la mission autorisée (`request_unlinked` sinon) ;
  (4) un cas ambigu est refusé ; (5) la décision est un champ structuré, jamais lue
  dans le texte.
- Les questions à secret (`isSecret`) et l'élicitation en mode URL sont refusées en v1.
  Une réponse de parent à une question est bornée (8 Kio) et validée contre les
  identifiants de question proposés.

## R9 - Pas de boucle

- Un seul saut : la route vise `task.owner` (instance attestée comme dans
  `recover_owner`, `native_delegation.rs:172`). L'identité de l'appelant de
  `bridget_task_respond` doit égaler le propriétaire. L'enfant ne peut donc pas
  répondre à sa demande.
- Une chaîne de trois niveaux : la tâche du petit-enfant a pour propriétaire l'enfant,
  dont le snapshot est celui du petit-enfant (`parent_fact` lit le snapshot de
  l'enfant pour un parent natif, `native_delegation.rs:38-66`). FR-027 en découle.

## R10 - Pause (ADR 027)

- La pause interrompt les tours en cours (amendement 2026-09-03). Les demandes
  ouvertes de ces tours passent à `cancelled`. Une réponse de parent pendant la pause
  est un effet autonome : refus `referent_paused` via la garde de 150
  (`AutonomousEffect::NativeDelegation`). L'humain n'est pas bloqué.
- La reprise d'un tour interrompu relève de la session152. Ici, la demande annulée
  n'est pas recréée par le daemon ; le fournisseur la reposera à la reprise.

## R11 - Délais

- Fenêtre parent 600 s, humain 3600 s (durée de rappel de la boîte,
  `human_inbox.rs:26`). Plafond : `mission_deadline_at - now - 30` (champ de 150,
  `delegation.rs:41`). Pas de gel des horloges : c'est la session152.

## R12 - Rétention (session 150)

- Les lignes de demandes sont supprimées par la purge d'arbre, dans la même
  transaction (FR-029). Une tâche terminale n'a plus de demande ouverte (FR-025). Donc
  les demandes ne bloquent pas la purge.

## R13 - Chemins : ne pas décider d'une écriture à partir d'un chemin (ronde 2, objection 2)

- Une résolution avec base attestée (le `cwd` figé, `native_delegation.rs:540-545`) réduit
  la fenêtre sans la fermer : un dossier vérifié peut devenir un lien symbolique après
  le dernier contrôle et avant l'écriture faite par Codex. Une seconde résolution juste
  avant l'envoi ne change pas ce point : Bridget ne contrôle pas le moment de l'écriture.
- **Décision (refus fermé)** : sous un parent `workspaceWrite`, toute approbation
  d'écriture de fichier est refusée (`file_confinement_unproven`), quel que soit le
  chemin. Sous `dangerFullAccess`, le parent peut écrire partout : le chemin n'est pas une
  limite, l'humain décide. Les chemins sont cités à l'humain, jamais utilisés pour
  décider. Le réseau et les commandes n'utilisent aucun chemin.
- **Retiré** : `resolve_request_paths` et ses tests. La tâche T009 ne garde que la mise
  en forme des chemins cités.
- **Pour rouvrir** : une session future devra prouver le confinement à l'exécution
  (le fournisseur applique lui-même les racines inscriptibles et rejette les liens), par
  une mesure sur Codex et non par une lecture de ce dépôt.

## R14 - Corrélation, validité de remise, fermeture sérialisée (objections 6 et 7, ronde 2)

- Les identifiants JSON-RPC sont courts et peuvent se répéter entre tours. L'identifiant
  de demande dérive de la tâche, de l'instance attestée (`conn_instances`, comme
  `capture_reply`, `native_delegation.rs:771-808`), de la session, du tour et de
  l'identifiant fournisseur ; une empreinte du contenu détecte un doublon trompeur.
- La décision est un fait historique ; la validité de remise (`delivery_state`) est un
  état séparé : `none`, `pending`, `sent`, `delivered`, `void`, `unknown`.
- **Constat de la ronde 2** : `void` posé côté daemon ne retire pas un accord déjà envoyé.
  Séquence : envoi, annulation du daemon, réception du wrapper, écriture de l'accord.
  Parades : (1) le daemon écrit `sent` avant d'envoyer ; (2) le wrapper sérialise la
  réponse et la fermeture sous un seul verrou ; (3) l'annulation passe par une trame
  `ChildRequestCancel` et n'est confirmée qu'après `ChildRequestClosed` ; (4) un accord
  `sent` sans accusé reste d'issue inconnue (`unknown`), jamais « annulé » ; (5) un
  anneau des 64 dernières demandes réglées permet de répondre exactement à un renvoi.
- **Échéance** (ronde 2, objection nouvelle) : `deadline_at` est contrôlée à la décision
  et avant chaque envoi, indépendamment du tick de 1 s. Un accord `pending` à
  l'échéance devient `void` ; un accord `sent` suit la fermeture confirmée.
- Le chemin de relais reste le patron `ControlExecutionDispatch`
  (`daemon.rs:9195-9240`).

## R15 - Liste fermée des outils du parent natif (objection 11)

- `BRIDGET_SAFE_MCP_TOOLS` (24 outils existants, `wrapper.rs:5031-5056`) alimente
  `--allowedTools` de Claude (`:5061`) et `approval_mode="approve"` de Codex
  (`:5265-5270`). Sans l'ajout de `bridget_task_respond`, un parent natif ne pourrait
  pas répondre. Tests de liste : `wrapper.rs` autour de `:8116-8172` et
  `crates/bridget-daemon/tests/claude_native_permissions_test.rs:400`.

## Comparaison T3 (point A11 de `research-t3-bridget.md`)

- T3 : `t3_pending_request_list/read/respond` ne couvre que `user_input`. Les
  approbations restent à l'humain.
- Bridget 155 : questions au parent, approbations à l'humain dans une limite absolue
  vérifiée, issue explicite. Un point T3 reste meilleur : son affichage dans une
  interface. Il est hors périmètre ici.

## Alternatives rejetées

| Alternative | Raison du rejet |
|---|---|
| Défaut `forward` | Casse le contrat 149 et fait attendre les parents qui ne surveillent pas |
| Parent qui approuve les approbations | Aucun gain utile pour Codex vers Codex (même politique), risque d'injection ; question 2 |
| Accord humain sans limite | Gain de droits contraire à l'ADR 149 (objection 1) |
| Passage `never` vers `on-request` | Peut activer un relecteur automatique concurrent (objection 4) |
| Relève des décisions par préfixe | Affamée par 200 anciens items (objection 9) |
| Nouveau type de boîte humaine | Migration de table pour un gain nul |
| Laisser le parent accorder `acceptForSession` | Droit durable, hors plafond figé |
| Attente dans la boucle de lecture | Bloque fin de tour et notifications |
| Escalade parent puis humain après silence | Deux états de route, accord tardif après reprise de l'enfant |
| Réponse humaine en texte libre | La boîte ne le porte pas ; hors périmètre |
| Enfants Claude dans cette session | Rouvre le lancement 149 ; question 3 |

## R16 — Formes primaires0.162.1 lues avant T003

Source : /Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1,
génération --experimental et initialize.experimentalApi=true par root. Ce passage
a lu Params/Response fileChange, permissions, userInput, elicitation, la notification
resolved et les branches fileChange des notifications ItemStarted/Completed.
Le contrat détaillé et les 447 hashes sont consignés par l’owner de codex-requests.md
et schema155.md ; aucun outil générateur n’a été exécuté par l’owner documentaire.

| Forme | Constat primaire | Conséquence conservative155 |
|---|---|---|
| FileChangeRequestApprovalParams | ni paths ni availableDecisions ; grantRoot vise le reste de session avec effet indiqué incertain | refus entier en v1 approval_method_unsupported ; grantRoot non nul duration_unsupported |
| FileChangeResponse | décision accept/acceptForSession/decline/cancel | aucune offre ni portée d’effet inventée ; pas de route Human réelle v1 |
| ItemStarted/Completed | item fileChange corrélé threadId/turnId/id ; changes path/diff/kind et move_path possible | faits cités, piste future, aucune preuve de confinement ni cache supplémentaire |
| PermissionsResponse | permissions, scope turn/session avec défaut turn, strictAutoReview facultatif | méthode refusée par JSON-RPC -32000 child_request_refused, motif approval_method_unsupported ; jamais faux decision:decline |
| ToolRequestUserInputResponse | answers{questionId:{answers:[string]}} ; aucune alternative decline/cancel | adapter answer interne qid→tableau ; refus/expiration par erreur JSON-RPC fermée |
| ElicitationParams | form,openai/form,openaiForm,url,openai/userVerification ; _meta JSONarbitraire, pas clé approval/durée établie | TOUT formulaire tiers fermé approval_method_unsupported, sans preuve structurée non-secret/non-approval ; user_input seul Parent ; exception vide Bridget historique distincte |
| ServerRequestResolvedNotification | threadId/requestId seulement, pas turnId | corréler à la table existante ; aucune donnée fournisseur inventée |

Arbitrage root : fileChange entière refusée ; aucun cache de corrélation nouveau.
Le classifier pur conserve ses tests synthétiques complets once/DF→Human, sans
déduire le support du transport réel. commandExecution exige kind=command et
offre explicite accept/decline/cancel, plus les limites figées de155.
T001 reçue root 180 PASS/0 FAIL,207sources stables ; T002 est final PASS et cochée par root, reçu validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte.

## R17 — Delta runtime environmentId local, GREEN ciblé, recette ouverte

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
