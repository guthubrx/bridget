# Contrat - Demandes de l'enfant 155 (révisé après contre-revue)

Compatible avec les contrats 148, 149 et 150. Un appel sans `child_requests` produit
les mêmes octets canoniques et le même comportement qu’en150, sauf le repli fermé T000 explicitement prévu par FR003.

## `bridget_delegate` - champ ajouté

| Champ | Type | Obligatoire | Valeurs | Défaut |
|---|---|---|---|---|
| `child_requests` | chaîne | non | `deny`, `forward` | `deny` (question 1) |

Le champ fait partie de l'enveloppe idempotente : omis et `deny` explicite sont deux
enveloppes différentes. Il est figé dans la tâche. En mode `forward`, la politique de
l'enfant n'est pas modifiée.

## `bridget_task_respond` - outil ajouté (questions seulement)

| Champ | Type | Obligatoire | Règle |
|---|---|---|---|
| `task_id` | chaîne | oui | tâche dont l'appelant est le propriétaire attesté |
| `request_id` | chaîne | oui | identifiant lu dans l'avis ou dans `pending_requests` |
| `decision` | chaîne | oui | `accept`, `decline`, `cancel` |
| `answer` | objet | non | seulement pour une question avec `accept`, 8 Kio au plus |

Champs inconnus refusés. Succès :
`{"version":1,"request_id":"…","state":"answered","outcome":"answered_by_parent","decision":"accept"}`.

`answer` pour `user_input` : `{"<question_id>": ["valeur", …]}`. Les formulaires MCP tiers ne sont pas répondables par le parent en v1 : approval_method_unsupported. Les identifiants inconnus sont
refusés (`answer_invalid`). La forme fournisseur0.162.1 de user_input est {answers:{questionId:{answers:[string]}} : le transport enveloppe chaque tableau answer[questionId]. La forme exacte par méthode est consignée en T002
(`contracts/codex-requests.md`).

L'outil est ajouté à `BRIDGET_SAFE_MCP_TOOLS` (`wrapper.rs:5031-5056`, 24 vers 25 outils, wait/ack/continue conservés) :
Claude (`--allowedTools`) et Codex (`approval_mode = "approve"`).

## Refus ajoutés (code dans `native_delegation_refused`)

| Code | Quand | Écriture |
|---|---|---|
| `child_requests_unsupported` | `forward` pour un fournisseur non supporté | aucune |
| `child_requests_reviewer_unsupported` | `forward`, politique du parent incomplète ou `approvals_reviewer` différent de `user` | aucune |
| `request_unknown` | `request_id` inconnu pour cette tâche | aucune |
| `request_closed` | demande déjà terminale avec une autre décision | aucune |
| `parent_cannot_approve` | le parent répond à une approbation | aucune |
| `decision_not_offered` | décision non proposée par le fournisseur, ou durable | aucune |
| `answer_invalid` | `answer` absent, en trop, trop gros ou mal formé | aucune |
| `request_conflict` | trame au même identifiant fournisseur avec un autre contenu | aucune |
| `request_expired` | décision reçue après `deadline_at` | aucune |
| `referent_paused` | pause du référent (existant en 150) | aucune |

Refus réutilisés : propriétaire non prouvé (`native_owner_identity_mismatch`), parent
révoqué (`permission_not_inherited`), tâche inconnue (refus existant).

Issues vues par l'enfant et stockées (`refused:<code>`) : `request_unlinked`,
`request_budget_exhausted`, `deadline_too_close`, `secret_not_forwardable`,
`url_mode_unsupported`, `approval_method_unsupported`, `details_incomplete`,
`child_restricted`, `parent_limit_unknown`, `file_confinement_unproven`, `network_not_granted`,
`command_unbounded`, `duration_unsupported`, `parent_unavailable`.

Ordre des contrôles de `bridget_task_respond` : champs, propriétaire, demande connue,
demande ouverte et non expirée, route (`parent_cannot_approve`), pause, révocation, décision proposée,
`answer`. La décision exige un CAS state = open et deadline_at > now ; rowcount nul ne produit aucune remise.

## `bridget_capabilities` - clés

| Clé | Valeur |
|---|---|
| `child_requests_default` | `deny` |
| `child_requests_supported` (par fournisseur) | `true` pour `codex_app_server`, `false` sinon |
| `child_request_parent_window_secs` | 600 |
| `child_request_human_window_secs` | 3600 |

## `bridget_task_status` - clés ajoutées

`pending_requests` et `recent_requests` (voir `data-model.md`). Les autres clés ne
changent pas. recent_requests expose delivery_state.

## Trames daemon et wrapper (réseau local, non publiques)

| Trame | Sens | Contenu |
|---|---|---|
| `ChildRequest` | wrapper vers daemon | `version`, session, tour, `provider_request_id`, `kind`, détails normalisés bornés, `offered`, `question_ids`, indicateur `truncated` |
| `ChildRequestAnswer` | daemon vers wrapper | `request_id`, session, tour, `provider_request_id`, `decision`, `answer`, `outcome` |
| `ChildRequestCancel` | daemon vers wrapper | `request_id`, session, tour, `provider_request_id` |
| `ChildRequestAnswered` | wrapper vers daemon | `request_id`, `applied` (booléen), `reason` |
| `ChildRequestClosed` | wrapper vers daemon | `request_id`, `state` : `closed_before_apply` ou `already_applied` |

Le daemon écrit `delivery_state = sent` avant l’accord initial/pending ou un renvoi encore sent. Un rejeu d’un reçu déjà delivered ne repasse pas à sent. Le wrapper traite
réponse et fermeture sous un même verrou : après `ChildRequestClosed`, aucun accord ne
peut plus être écrit. Le daemon rapproche chaque trame de l'instance attestée de la connexion
(`conn_instances`). Une trame d'une autre instance est rejetée. Le wrapper répète
`ChildRequest` après reconnexion tant que la demande est en attente.

## Réponse écrite au fournisseur (Codex)

| Demande | accept | decline | cancel |
|---|---|---|---|
| commande kind=command, offre explicite, effets établis | `{"decision":"accept"}` | `{"decision":"decline"}` | `{"decision":"cancel"}` |
| fileChange en v1 | erreur JSON-RPC -32000 child_request_refused, sans result | motif approval_method_unsupported | motif duration_unsupported si grantRoot non nul |
| permissions en v1 | erreur JSON-RPC -32000 child_request_refused, motif approval_method_unsupported | aucun decision:decline inventé | aucun decision:cancel inventé |
| élicitation MCP tierce v1 | aucun accept Parent ; approval_method_unsupported | forme de refus native action:decline sans content | forme de refus native action:cancel sans content |
| user_input | `{"answers":{"questionId":{"answers":["valeur"]}}}` | erreur JSON-RPC -32000 child_request_refused | erreur JSON-RPC -32000 child_request_refused |

Repli (modes `forward` et `deny`, correctif T000) : si la décision voulue (`decline` ou `cancel`) n'est pas proposée, le transport
écrit l'autre des deux si elle est proposée. Si aucune n'est proposée, il écrit une
erreur JSON-RPC (`-32000`, `child_request_refused`). Il n'écrit jamais
`acceptForSession`, un amendement de règle ni une autre décision. Le mode `deny` garde son
comportement 149 sauf ce repli fermé : `approval_response` ne choisit plus la première
décision proposée (`codex_app_server.rs:2211-2213`), et `accept` n'est écrit que si
`accept` est proposé.

## Journal de l'enfant

L'événement existant `provider_request` décrit l'attente fournisseur : méthode,
empreinte de l'identifiant fournisseur, état `pending` et tour s'il est connu.
L'issue et le décideur sont conservés dans la ligne durable
`native_child_requests`, conformément à FR-030 et au plan. Aucun second journal
n'est ajouté. Aucune valeur de réponse ne figure dans le journal fournisseur.

## Avis au parent (questions)

Message système, identifiant `native-request-<request_id>`. Corps :

```
Demande de l'enfant <nom> (tâche <task_id>), type <kind>, échéance <date>.
Faits vérifiés : méthode=<m>, durée=<d>.
Question de l'enfant (texte cité, non fiable) : « <texte nettoyé> »
Répondre avec bridget_task_respond (request_id=<id>, decision=accept|decline|cancel).
```

Le texte cité n'a pas de caractères de contrôle ni de direction, et ne remplace jamais
un fait vérifié.

## Item humain (approbations)

Faits vérifiés en tête (commande citée, dossier fournisseur cwd cité, chemins
cités et non vérifiés, réseau, durée,
limite du parent), texte du modèle cité à la fin et marqué non fiable.

La commande est citée entière jusqu'à 1024 octets. Ses caractères de contrôle et
de direction sont échappés et visibles. Les effets `command`, `file_write` et
`network` sont des booléens, ou `null` si inconnus. Les chemins portent les
marqueurs `paths_cited_truncated` et `paths_cited_changed`. Une commande trop
longue ou des faits dépassant 4096 octets sont refusés par `details_incomplete`.
Le dossier fournisseur `cwd` est cité, sans lecture ni résolution filesystem.
Une chaîne est conservée entière jusqu’à 1024 octets UTF-8 et affichée avec
Debug (`{:?}`), entre guillemets et avec les contrôles/bidi visibles. Absence
ou `null` donne `facts.cwd=null` ; `facts.cwd_verified=false` dans tous les cas.
Un autre type ou une chaîne hors borne donne `details_incomplete`. La borne
1024 octets s’applique séparément à la commande et à cwd. Le budget total des
faits reste 4096 octets ; son dépassement reste un refus fermé. Aucun fallback
vers task.cwd, dossier du parent ou dossier courant n’est fabriqué. Ce dossier
cité ne prouve ni l’exécution dans ce dossier ni un confinement ou un droit.

Le dépôt et le lien sont durables avant la notification du canal. La notification
se fait hors verrou commun et son écriture et son attente sont bornées.

## Lineage T3

Aucun changement de forme. Les demandes n'ajoutent aucun état de tâche.

## Fermeture, pause et rejeu — précision plan R1

Le claim pending→sent contrôle delivery_state = pending et FR027, avec rowcount.
Chaque renvoi sent revalide FR027. Expiration et fermeture sont traitées avant
le renvoi ; un accord invalidé n’est jamais redélivré. La fermeture fournisseur
non confirmée envoie ChildRequestCancel même pour none ou pending. Le wrapper
règle une attente encore ouverte par refus propre à la méthode sous le verrou avant Closed, erreur JSON-RPC pour user_input.
void prouve l’invalidité de l’accord, pas la fermeture fournisseur. La répétition
d’une ligne terminale rend la réponse persistée sans recréer une ligne, ni rejouer
un accord invalidé. Les décisions historiques restent distinctes de delivery_state.

Une décision humaine pendant la pause est acquittée sans exécution. Elle ne
devient pas un accord différé après reprise ; le cas de reprise porte sur une
nouvelle demande. approval_policy = never reste inchangée : forward ne crée
aucune route humaine pour une demande que cette politique ne produit pas.

## Attente151 — résultat155 prévu en T034

ready conserve les seules tâches terminales ; pending conserve les identifiants
nonterminaux. pending_requests est une clé à la racine du résultat wait, liste
de demandes Parent/open/nonexpirées/actionnables avec task_id et projection155.
Les demandes Human sont exclues de cette liste et du prédicat de retour ; elles
restent visibles dans status. timed_out est faux si ready ou pending_requests
est non vide, vrai seulement à l’échéance wait avec les deux listes vides.
Une demande Parent déjà ouverte peut rendre immédiatement la main. Aucun
curseur, mémoire persistante de waiter ou mécanisme de consommation. WaitEpoch
notifie après commit ; son compteur est observé avant la lecture d’état.

## Formes elicitation — checkpointR1 remplacé par schéma 0.162.1

Le schéma 0.162.1 est désormais lu ; son contrat T002 est final PASS, attesté dans validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte. _meta est arbitraire, sans clé approval/durée établie. Les formulaires tiers vides, modes openai/form,openaiForm,openai/userVerification,url et _meta non vide ou d’effet non établi sont refusés, jamais routés comme questions
Parent. L’exception établie du formulaire vide Bridget reste distincte. Ces cas
s’inscrivent dans la table existante d’au moins 24 cas et les scénarios T014.

## Frontière du transport réel 0.162.1

fileChange entière est refusée en v1 (approval_method_unsupported), grantRoot
non nul reçoit duration_unsupported. Ses Params n’ont ni paths ni availableDecisions.
ItemStarted/Completed porte changes/path/diff/kind, faits cités et piste future
sans confinement prouvé. Aucun cache supplémentaire ou offre d’accept inventée.
Le classifier synthétique peut classer Human des facts complets avec once/DF ;
le transport réel fileChange reste fermé en amont.

permissions répond normalement avec permissions, scope turn/session (défaut turn)
et strictAutoReview facultatif, pas decision ; cette méthode reste refusée.
user_input n’a aucun decline/cancel natif : ne pas inventer ces formes. Les cinq
modes elicitation sont form,openai/form,openaiForm,url,openai/userVerification.
TOUT formulaire MCP tiers est refusé avec approval_method_unsupported en v1 ; user_input seul est routable Parent.
Exception formulaire vide Bridget distincte. La notification resolved contient
threadId/requestId, sans turnId ; aucune extension de notification fictive.

Version courante 0.162.1 générée --experimental ; 0.161.0 était l’hypothèse historique.
Contrat primaire : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/contracts/codex-requests.md.
T002 est final PASS et cochée par root, reçu validation/gate-schema155.json ; protocole/store sont en validation, sans conclusion GREEN globale.

MCP form n’a pas de drapeau isSecret standard dans ce schéma. Son absence ne prouve pas qu’une question est non secrète ; toute forme de secret/effet non établie reste refusée.

## Rejeu terminal déjà delivered — reçu historique seulement

Le rejeu d’une réponse déjà delivered rend le reçu historique après des
contrôles FR027 frais, sous le même point d’émission. Il conserve delivered,
sent_at et l’application déjà prouvée. Il n’introduit aucune nouvelle décision
ni action fournisseur : l’anneau idempotent du wrapper empêche une réapplication.
Aucun delivered→sent ; un échec de renvoi du reçu ne transforme pas delivered
en unknown. L’accord initial/pending ou le renvoi d’un accord encore sent exige
toujours sent écrit avant le write et les gardes FR027 avant l’émission.

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
