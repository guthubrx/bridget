# Session155 - Remontée des demandes de l'enfant

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 155-demandes-enfant
Titre: Remontée des demandes de l'enfant au parent ou à l'humain
Statut: Livré le 2026-10-12 (42/42 ; trois Converge et contrôle cwd ciblé ; installé/actif ; WT/branche retirés)
Priorité: P1
Tâches: 42/42 (cochées par root ; T040 validée avec accord explicite utilisateur)
Tests: 71 PASS ciblés post-cwd, RC0, 214 sources stables ; quatre régressions canal087/STDIO antérieures distinctes ; recette réelle quatre cas PASS avant cwd ; suite unique antérieure 2259 PASS / 15 FAIL / 72 ignored, aucun GREEN global

Résumé:
- Contexte: Un enfant natif qui demande une approbation ou pose une question reçoit aujourd'hui une réponse automatique (Codex) ou un refus (élicitation). Personne ne décide. Le parent et l'humain ne voient rien.
- Objectif: Les questions de l'enfant remontent au parent. Les approbations remontent à l'humain, dans la limite absolue des droits figés du parent. L'enfant attend une décision tracée. Rien ne donne plus de droits que ceux du parent, humain compris.
- Dépendances: SPEC-148, SPEC-149, SPEC-150 (délai, pause, purge), SPEC-151 (attente et boîte de fin), ADR 027, ADR 149

Fichiers:
- spec.md: ✓
- plan.md: ✓
- tasks.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-155-demandes-enfant` (worktree dédié `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant`, exécution séquentielle, sans commit automatique)
**Created**: 2026-10-11
**Status**: Livré
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-150, SPEC-151

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles »
(`docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md`). Critère visé :
« Demandes de l'enfant », note 1 sur 5 aujourd'hui, 4 sur 5 visé dans cette session
(5 sur 5 si l'utilisateur répond « oui » aux questions 2 et 3).

## Besoin et périmètre

Un enfant natif travaille sans humain devant lui. Son fournisseur lui pose parfois
une question ou lui demande une approbation. Quatre défauts rendent ce cas
inutilisable.

**Défaut 1 - Les approbations sont tranchées par un réglage fixe.** Le transport
Codex répond `accept` ou `decline` selon le réglage `permissions` de l'enfant, sans
regarder la demande (`crates/bridget-transport/src/codex_app_server.rs:2195-2226`
et `:3344-3366`). Les enfants natifs ont `permissions = deny` : toute demande est
refusée en silence.

**Défaut 2 - Les questions sont refusées ou inconnues.** Une élicitation MCP est
déclinée, sauf le formulaire vide de Bridget (`codex_app_server.rs:2317-2353`).
Une demande `user_input` tombe dans la réponse « requête non prise en charge »
(`:3469-3500`). L'enfant ne peut donc jamais poser une vraie question.

**Défaut 3 - Personne n'est prévenu.** La boîte humaine a neuf types et aucun ne
concerne une délégation (`crates/bridget-transport/src/protocol.rs:1267-1277`).
L'état `waiting_approval` existe dans le suivi des exécutions, mais aucun pilote ne
l'émet pour un enfant natif. Le signal `permission_required` n'est qu'une observation
après coup (`crates/bridget-daemon/src/wrapper.rs:5741`).

**Défaut 4 - Pas de réponse possible.** Aucun outil ne permet au parent de répondre à
son enfant (`crates/bridget-daemon/src/delegation_mcp.rs:11-17`).

**Comparaison T3 (lecture seule, `origin/main`).** T3 remonte seulement les questions
`user_input` au parent. Les approbations restent à l'humain par alerte
(`research-t3-bridget.md`, A11). Bridget fait la même répartition, avec en plus un
plafond de droits vérifié par le daemon, une issue explicite et une trace durable.

**Répartition retenue (réduction de périmètre après contre-revue).**

| Demande | Décideur | Pourquoi |
|---|---|---|
| Question user_input avec isSecret=false (défaut normatif) ; formulaires MCP tiers refusés | le parent | c'est une information, pas un droit |
| Approbation de commande kind=command, offre explicite accept/decline/cancel, aux effets/durée établis ; fileChange reste refusée en v1 selon le schéma 0.162.1 | l'humain | un parent modèle pourrait être induit en erreur par le texte de la demande ; un enfant hérité a la même politique que son parent, donc le parent ne peut rien couvrir de plus |
| Tout ce qui dépasse la limite absolue | personne : refus automatique | aucune décision ne donne plus de droits que le bac à sable figé du parent |

**Limite absolue.** La limite absolue est la politique de bac à sable attestée et
figée du parent (`permission_snapshot.parent`), réduite par la politique figée de
l'enfant. Une approbation « humaine requise » reste possible seulement dans cette
limite. Une demande hors de cette limite reçoit un refus automatique, même si
l'humain l'aurait acceptée.

**Changement d'une décision de la session 149.** La spec 149 écrit : « Aucun dialogue
de validation de permission n'est introduit » (`specs/149-sous-agents-lineage/spec.md:86`
et FR-008) et l'ADR 149 : « Les permissions non couvertes restent refusées sans
dialogue humain Bridget ». La session 155 change cette décision de façon limitée :

- Le but de 149 reste valable : pas de grant humain par mission, pas de gain de
  droits, pas de dialogue pour une action déjà permise au parent.
- Ce qui change : une demande que le fournisseur pose lui-même, et qui reste dans la
  limite absolue, ne se termine plus par un refus automatique. Elle devient une
  demande tracée avec un décideur identifié.
- Facultatif par tâche (`child_requests: "forward"`). Sans ce paramètre, le
  comportement 149 reste identique octet pour octet.
- La limite ne bouge pas. Un accord humain ne peut pas la dépasser.
- Une ADR 155 consigne ce changement (tâche T036).

**Hors périmètre** :

- Réponse du parent à une approbation (question 2, défaut : non).
- Enfants Claude (flux `claude_stream_json`, y compris GLM) : leurs invites sont
  désactivées au lancement (`--permission-prompts none`,
  `crates/bridget-daemon/src/native_permissions.rs:313`). Session suivante
  (question 3).
- Enfants ACP (Cursor) et agents hors moteur natif.
- Réponse humaine en texte libre. La boîte humaine ne porte que des choix fermés.
- Interface graphique des demandes dans T3 ou dans le panneau Bridget.
- Décisions durables (« toujours autoriser », « pour la session », amendements de
  règles) : jamais proposées, jamais relayées.
- Méthode item/permissions/requestApproval (octroi additionnel à scope turn/session, hors périmètre v1) et toute autre méthode `item/*/requestApproval` inconnue :
  refus automatique `approval_method_unsupported`.
- Questions à secret (`isSecret`) : refus `secret_not_forwardable`. Élicitations en
  mode URL : refus `url_mode_unsupported`.
- Approbations d'écriture de fichier sous un parent `workspaceWrite` : refusées en v1
  (`file_confinement_unproven`). Elles reviendront avec une preuve de confinement à
  l'exécution, dans une session future.
- Parent dont `approvals_reviewer` n'est pas `user` (relecteur automatique Codex) :
  refus à l'admission en mode `forward`.
- Transfert d'une demande au grand-parent quand le parent se tait : un seul saut.
- Gel des délais pendant l'attente (session152) et budgets (session158).
- Réutilisation d'un enfant (session153).

## Scénarios utilisateur

### US1 - Une question remonte au parent et la réponse revient à l'enfant (P1)

Un parent délègue avec `child_requests: "forward"`. L'enfant Codex pose une question
(user_input avec isSecret=false, défaut normatif ; aucune élicitation MCP tierce). Le tour de l'enfant
attend. Le parent reçoit un avis avec l'identifiant de la demande, la question et ses
choix. Il répond avec `bridget_task_respond`. L'enfant reçoit la réponse et continue.
Si le parent répond `decline`, l'enfant reçoit un refus.

### US2 - Une approbation dans la limite va à l'humain (P1)

L'enfant demande d'exécuter une commande ou d'écrire un fichier alors que le parent a
le plein accès, ou de sortir sur le réseau alors que le parent a le réseau. Bridget
vérifie la limite sur des faits structurés. L'item arrive dans la boîte humaine avec
les faits vérifiés d'abord. L'humain choisit `accept` ou `decline`. Le parent ne peut
pas répondre à cette demande (`parent_cannot_approve`).

### US3 - Une demande hors de la limite est refusée sans humain (P1)

Une demande réseau alors que le parent n'a pas le réseau, une commande alors que le
parent est confiné, toute écriture de fichier alors que le parent est confiné
(`workspaceWrite`), un enfant en lecture seule,
un détail incomplet ou tronqué, une méthode inconnue : Bridget refuse
automatiquement. L'enfant reçoit un refus. L'issue `refused:<code>` est visible. Aucun
item n'est déposé.

### US4 - Délai, silence et fin de tâche donnent toujours une issue explicite (P1)

Le parent ou l'humain ne répond pas à temps : la demande expire et l'enfant reçoit un
refus (jamais un accord). Même chose si la tâche est annulée, si le tour est interrompu
(pause), si l'enfant s'arrête, si la délégation est révoquée ou si le daemon redémarre.
Un accord déjà enregistré mais non appliqué devient sans effet. Aucune demande ne
reste ouverte sans fin.

### US5 - Aucune boucle et aucune élévation (P1)

Une demande monte d'un seul niveau : de l'enfant vers le propriétaire de sa tâche. Un
agent ne répond jamais à sa propre demande. Une chaîne parent, enfant, petit-enfant
garde chaque demande à son niveau. Un parent révoqué, en pause ou sans preuve
d'identité ne peut pas répondre. Les décisions durables ne sont jamais relayées.

### US6 - Voir et suivre les demandes (P2)

Le parent lit dans `bridget_task_status` les demandes ouvertes et les dix dernières
issues. L’attente151 rendra la main après T034 pour une demande Parent ouverte et actionnable, y compris déjà présente. Une demande Human seule ne provoque pas ce retour. ready reste réservé aux tâches terminales et pending aux tâches nonterminales. pending_requests est une clé à la racine du résultat wait, réservée aux demandes Parent actionnables avec task_id. timed_out est faux si ready ou pending_requests est non vide ; il est vrai seulement à l’échéance wait avec ces deux listes vides.

### US7 - Aucun changement sans option (P1)

Sans `child_requests: "forward"`, tout reste comme en 149-150 : réponse automatique
selon `permissions`, élicitations refusées, mêmes octets canoniques, mêmes tests verts.

## Exigences fonctionnelles

**Option et compatibilité**

- **FR-001** : `bridget_delegate` accepte `child_requests` (`"deny"` ou `"forward"`).
  Sans valeur, le défaut est `deny` (question 1). Le champ fait partie de l'enveloppe
  idempotente et il est figé dans la tâche.
- **FR-002** : `forward` pour un `agent_type` qui ne le supporte pas (tout sauf
  `codex_app_server`) rend `child_requests_unsupported`. `forward` avec un parent dont
  la politique n'est pas complète ou dont `approvals_reviewer` n'est pas `user` rend
  `child_requests_reviewer_unsupported`. Aucune tâche n'est créée. Le catalogue annonce
  `child_requests_default` et, par fournisseur, `child_requests_supported`.
- **FR-003** : En mode `deny`, le comportement est identique à 149 : réponse selon
  `permissions`, élicitation refusée sauf formulaire vide de Bridget. Une seule
  exception, volontaire et livrée seule (T000) : le repli fermé de FR-012 s'applique
  aussi au mode `deny`, car le code actuel choisit la première décision proposée quand
  celle voulue manque, ce qui peut être `acceptForSession`
  (`crates/bridget-transport/src/codex_app_server.rs:2211-2213`).
- **FR-004** : En mode `forward`, la politique figée de l'enfant n'est pas modifiée.
  Bridget ne change ni son `approval_policy`, ni son bac à sable, ni son relecteur.

**Interception**

- **FR-005** : En mode `forward`, le transport ne répond pas automatiquement aux
  demandes item/commandExecution/requestApproval aux effets/durée établis, aux demandes user_input avec isSecret=false (défaut normatif) ; aucune élicitation MCP tierce ne remonte.
  Il les enregistre, les signale au wrapper et attend. fileChange et permissions sont refusées avec approval_method_unsupported selon les formes 0.162.1 (duration_unsupported pour grantRoot non nul) : les effets/durée de fileChange ne sont pas établis. TOUT formulaire MCP tiers, même form typé, non vide et sans _meta, reste refusé en v1 avec approval_method_unsupported : aucune preuve structurée de non-secret ou de non-approbation. L’exception historique vide Bridget reste distincte ; URL reste refusé avec url_mode_unsupported. T002 fige le contrat avant T003. Toute autre méthode
  `item/*/requestApproval` reçoit un refus `approval_method_unsupported`.
- **FR-006** : Attendre ne bloque pas la lecture du fournisseur : fins de tour et
  notifications continuent d'être traitées.
- **FR-007** : Au plus 4 demandes ouvertes en même temps par enfant, 32 en tout par
  tâche. Au-delà : `refused:request_budget_exhausted`.
- **FR-008** : Une demande n'est acceptée que pendant le tour actif de la mission
  autorisée de l'enfant, sur l'instance attestée de cet enfant. Sinon : refus
  `request_unlinked`.

**Verdict de route**

- **FR-009** : Le daemon calcule le verdict. Il ne se fie ni au texte de la demande ni
  à un champ déclaré par l'enfant. Verdicts : `Parent` (question valide, parent vivant,
  prouvé, non révoqué), `Human` (approbation dans la limite absolue), `Forbidden(code)`
  (tout le reste). L'incertitude donne `Forbidden`.
- **FR-010** : Limite absolue d'une approbation, vérifiée sur des faits structurés
  complets (jamais sur le texte libre `reason`) :
  - l'enfant n'est pas en lecture seule (`child_restricted` sinon) ;
  - la politique figée du parent est complète, son bac à sable est connu et n'est ni
    `readOnly` ni `externalSandbox` ;
  - écriture de fichier : le parent a le plein accès (`dangerFullAccess`). Sous un bac
    à sable `workspaceWrite`, toute approbation d'écriture est refusée
    (`file_confinement_unproven`), quel que soit le chemin, tant que le confinement à
    l'exécution n'est pas prouvé (FR-011) ;
  - réseau : le parent a le réseau, ou le plein accès ;
  - commande : le parent a le plein accès (les effets d'une commande ne sont pas bornés) ;
  - détails complets : aucun champ d'effet inconnu, aucune troncature.
- **FR-011** : Un dossier vérifié peut devenir un lien symbolique entre le dernier
  contrôle et l'écriture faite par Codex ; aucune seconde résolution ne ferme cette
  course. Bridget ne décide donc jamais d'une écriture à partir d'un chemin. Les chemins
  de la demande sont seulement cités à l'humain (bornés, nettoyés, marqués non
  vérifiés). La résolution de chemins et la preuve de confinement à l'exécution sont
  reportées à une session future. Les commandes et le réseau n'utilisent aucun chemin.
- **FR-012** : Décisions internes : accept (une fois), decline, cancel. Leur réponse fournisseur suit la forme propre à chaque méthode établie en T002 ; user_input n’a pas de decline/cancel natif et reçoit une erreur JSON-RPC fermée pour ces refus. Aucune
  décision durable. Si le fournisseur ne propose ni `decline` ni `cancel`, le
  transport écrit une erreur JSON-RPC fermée, jamais une autre décision. Cette règle
  vaut pour les modes `forward` et `deny` (correctif T000).
- **FR-013** : Le parent ne peut jamais répondre à une approbation
  (`parent_cannot_approve`). L'humain ne peut jamais répondre à une question.

**Réponse du parent (questions)**

- **FR-014** : Nouvel outil `bridget_task_respond` (`task_id`, `request_id`,
  `decision`, `answer` facultatif). `decision` vaut `accept`, `decline` ou `cancel`.
  `answer` n'existe que pour une question avec `accept`. Champs inconnus refusés.
- **FR-015** : L'outil est ajouté à la liste fermée des outils du parent natif
  (`BRIDGET_SAFE_MCP_TOOLS`, 24 outils existants plus `bridget_task_respond`, soit 25), pour Claude et pour Codex.
- **FR-016** : Au moment de la réponse, le daemon vérifie : propriétaire attesté ;
  demande ouverte et non expirée ; route `Parent` ; révocation ; pause. Une réponse
  identique rejouée rend l'état. Une réponse différente sur une demande close rend
  `request_closed`.
- **FR-017** : Pendant la pause du référent, la réponse d'un parent rend
  `referent_paused`. La décision humaine par la boîte reste possible, mais elle est acquittée sans exécution pendant la pause. Elle ne produit aucun accord différé après reprise ; une réponse après reprise concerne une nouvelle demande.

**Avis, statut, boîte humaine**

- **FR-018** : Une question tente un avis système idempotent vers le parent par
  SendAs, avec les gardes149 et leur disjoncteur. S’il est émis, l’avis donne
  l’identifiant, le type, un résumé borné et l’échéance. Son succès n’est pas
  garanti. Un refus ne supprime pas la demande persistée : le propriétaire
  peut la consulter par Status et par Wait lorsqu’elle est actionnable.
- **FR-019** : Présentation : les faits vérifiés par Bridget (commande citée,
  dossier fournisseur cwd cité, chemins
  cités et normalisés lexicalement, sans résolution filesystem ni valeur de preuve de confinement, réseau oui ou non, méthode, durée) viennent d'abord.
  Le dossier provider cwd est une chaîne citée entière, rendue avec Debug et
  contrôles visibles, ou null si absent/null ; cwd_verified reste false. Un autre
  type ou plus de 1024 octets UTF-8 impose details_incomplete. Commande et cwd
  ont cette borne séparément ; les faits restent bornés à 4096 octets. Aucun
  fallback vers task.cwd ni résolution, confinement ou droit n’est inventé.
  La commande reste entière, avec ses contrôles visibles ; toute perte de contenu
  décisif impose un refus. Les chemins signalent toute coupe ou modification.
  Le texte du modèle (`reason`, question) est cité à part, borné, nettoyé des caractères de contrôle et de
  direction, et marqué non fiable.
- **FR-020** : `bridget_task_status` ajoute `pending_requests` et `recent_requests`.
  Les clés actuelles ne changent pas. recent_requests expose aussi delivery_state.
- **FR-021** : Une approbation `Human` crée un item `intervention_required` (type
  existant) avec les options `accept` et `decline`. Le dépôt humain existant rend
  l'identifiant de l'item. Avant tout dépôt, le daemon cherche un item de même clé
  exacte et le réutilise.
- **FR-022** : Le daemon lit l'état de chaque item par son identifiant exact (au plus
  512 demandes ouvertes), jamais par balayage de la file. Une décision humaine arrivée
  après la fin de la demande est acquittée sans exécution (abandon durable).

**Délai et issues**

- **FR-023** : Délai : 600 s pour le parent, 3600 s pour l'humain, bornés par le
  temps restant de la tâche moins 30 s. S'il reste moins de 30 s : refus immédiat
  `deadline_too_close`.
- **FR-024** : À l’expiration, l’enfant reçoit le refus propre à sa méthode : decline/cancel pour approbation ou élicitation, erreur JSON-RPC -32000 child_request_refused pour user_input. Jamais accept. L'issue `expired` est stockée.
- **FR-025** : Chaque demande a une issue terminale visible : `answered_by_parent`,
  `answered_by_human`, `expired`, `cancelled` ou `refused:<code>`.
- **FR-026** : Annulation, fin de tâche, interruption du tour (pause), arrêt de
  l'enfant, révocation, changement d'instance, échéance ou redémarrage : les demandes
  ouvertes passent à `cancelled` ou `expired`, l'item humain est fermé ou acquitté sans
  exécution, et tout accord encore `pending` (jamais envoyé) devient `void`.
- **FR-027** : Un accord n'est envoyé que si, à l'instant de l'envoi, la tâche est
  active, l'instance de l'enfant est celle de la ligne, aucune pause n'est active,
  aucune révocation n'existe et l'échéance n'est pas passée. Ce contrôle est refait à
  chaque envoi et au moment de la décision (`request_expired` sinon). Le daemon écrit
  `delivery_state = sent` AVANT d'envoyer. Un accord envoyé sans accusé reste d'issue
  inconnue (`unknown`) : il n'est jamais déclaré annulé.
- **FR-027b** : Le wrapper sérialise la fermeture et la réponse d'une demande sous un
  même verrou. Une fermeture demandée par le daemon (`ChildRequestCancel`) retire la
  demande ou constate qu'elle est déjà réglée, puis répond par l'état réel
  (`closed_before_apply` ou `already_applied`). Une fois la fermeture confirmée,
  aucun accord ne peut plus être écrit. Le wrapper garde les 64 dernières demandes
  réglées pour répondre exactement à un renvoi. La fermeture non confirmée utilise ChildRequestCancel→ChildRequestClosed même pour une demande pending ou none. Une demande inconnue donne
  `applied = false`. Expiration et fermeture sont traitées avant chaque renvoi sent, qui recontrôle FR027.

**Corrélation et rejeu**

- **FR-028** : L'identifiant d'une demande dérive de la tâche, de l'instance attestée,
  de la session, du tour et de l'identifiant fournisseur. Le contenu est figé par une
  empreinte. Une trame répétée au même contenu rend la même ligne. Un contenu différent
  rend `request_conflict`. Le replay d’une ligne terminale rend la réponse persistée, sans recréer une demande ni rejouer un accord invalidé. Un accusé de remise n'est accepté que de la connexion de
  l'instance de la ligne.

**Pas de boucle**

- **FR-029** : Une demande n'a qu'un destinataire : le propriétaire de la tâche de
  l'enfant. Pas de transfert. Un agent ne répond pas à sa propre demande. Rien ne
  descend d'un parent vers l'enfant par ce mécanisme. Le plafond d'un petit-enfant se
  calcule avec la politique figée de son propre parent.

**Traçabilité, rétention, documentation**

- **FR-030** : Chaque demande laisse une ligne durable : tâche, instance, tour, type,
  résumé borné (400 caractères), faits vérifiés, route, décideur, décision, validité de
  remise, dates, issue. Détail brut borné à 4 Kio. Aucune réponse secrète n'est stockée.
- **FR-031** : La purge d'un arbre (session 150) supprime les lignes de demandes de cet
  arbre dans la même transaction.
- **FR-032** : Les refus, l'outil et le paramètre sont documentés dans
  `docs/delegation-native.md` et la skill Bridget. Une ADR 155 consigne le changement
  de la décision 149.

## Critères de succès

- **SC-001** : En mode `forward`, 0 réponse `accept` n'est écrite vers le fournisseur
  sans une décision enregistrée de l'humain, ni sans validité de remise.
- **SC-002** : Sur la table de verdicts (au moins 24 cas : chemins, réseau, commande,
  lecture seule, méthodes, détails incomplets, texte hostile), 100 % des cas hors
  limite donnent `Forbidden` sans item de boîte.
- **SC-003** : 0 décision hors de `accept` / `decline` / `cancel` atteint le
  fournisseur, y compris quand l'offre ne contient que `acceptForSession` ou des
  amendements de règles.
- **SC-004** : 100 % des demandes atteignent une issue terminale. Une demande échue est
  close au plus tard au tick suivant (1 s).
- **SC-005** : Sans option, les tests 148, 149 et 150 restent verts et les octets
  canoniques d'un appel sans `child_requests` sont inchangés. Seule exception : le repli
  fermé de T000 (FR-003).
- **SC-006** : Sur une chaîne de 3 niveaux avec de vrais parents natifs, 100 % des
  questions sont résolues à leur niveau ; 0 demande est répondue par son auteur.
- **SC-007** : Pendant qu'une demande est ouverte, le transport traite une fin de tour
  et une notification reçues ensuite.
- **SC-008** : Un arbre purgé ne laisse aucune ligne de demande.
- **SC-009** : Test accord, annulation, reconnexion : 0 accord appliqué après une
  fermeture confirmée. Test envoi, annulation du daemon, réception du wrapper : soit
  l'accord est écrit avant la fermeture et l'issue est `already_applied`, soit il n'est
  jamais écrit ; jamais les deux. Un envoi sans accusé reste `unknown`.
- **SC-010** : 250 décisions humaines obsolètes ne bloquent ni ne déclenchent aucune
  exécution.
- **SC-012** : Un accord `pending` ou `sent` dont l'échéance est passée n'est jamais
  écrit ; l'échéance est contrôlée à la décision et à chaque envoi.
- **SC-013** : En mode `deny` comme en `forward`, une offre qui ne contient que
  `acceptForSession` ou des amendements de règles ne produit jamais un accord (T000).
- **SC-014** : Toute approbation d'écriture de fichier sous un parent `workspaceWrite`
  est refusée, quel que soit le chemin (`..`, lien, fichier neuf, dans la zone).
- **SC-011** : Recette de bout en bout avec un faux serveur Codex : question, réponse,
  reprise ; approbation humaine ; expiration ; sans service réel relancé.

## Entités et hypothèses

**Entités**

- **Demande de l'enfant** : question ou approbation posée par le fournisseur d'un
  enfant natif, corrélée à sa tâche, son instance et son tour.
- **Verdict** : `Parent`, `Human` ou `Forbidden(code)`, calculé par le daemon.
- **Limite absolue** : bac à sable figé du parent, réduit par la politique de l'enfant.
- **Décision** : `accept`, `decline` ou `cancel`, avec une réponse facultative.
- **Validité de remise** : `none`, `pending`, `sent`, `delivered`, `void` ou `unknown`.
  Distincte de la décision. `void` n'est posé que si l'accord n'a jamais été envoyé ou
  si le wrapper confirme qu'il n'a pas été écrit.
- **Avis de demande** : message système idempotent vers le parent (questions).
- **Item humain** : item `intervention_required` de la boîte existante (approbations).

**Formes fournisseur** (schéma courant 0.162.1 lu ; 0.161.0 était l’hypothèse initiale, T002 final PASS attestée dans validation/gate-schema155.json)

- Le serveur Codex envoie une requête JSON-RPC par approbation, élicitation et saisie,
  et attend la réponse tant que le tour dure.
- Les formes générées 0.162.1 sont lues. Le contrat T002 consigne la réponse user_input {answers:{questionId:{answers:[string]}} et les refus par méthode. FileChangeRequestApprovalParams n’a ni paths ni availableDecisions ; grantRoot vise une portée session à l’effet incertain : fileChange entière reste approval_method_unsupported en v1, duration_unsupported si grantRoot non nul. Les notifications changes/path/diff/kind existent mais ne créent aucune route Human réelle ni nouveau cache. permissions répond avec permissions/scope/strictAutoReview, jamais decision. T002 est revue et figée, PASS attesté dans validation/gate-schema155.json ; protocole/store prouvés par les tests ciblés ; recette utilisateur T040 ouverte. Un
  champ d'effet absent de la liste blanche donne `Forbidden`.
- Les délais 600 s et 3600 s sont des valeurs de départ.
- Le défaut du paramètre est `deny` pour garder le contrat 149 (question 1).
- La livraison suit les sessions 147 à 150 : fichiers sur disque, aucune relance de
  service.

## Questions pour l'utilisateur

1. **Défaut de `child_requests`.** `deny` (défaut proposé) garde 149 intact : un
   parent qui ne sait pas répondre ne bloque jamais son enfant. `forward` donne un
   meilleur service par défaut, mais un parent qui ne surveille pas ferait attendre son
   enfant jusqu'à 600 s par question. Réponse attendue : `deny` ou `forward`.
2. **Le parent peut-il approuver une action de son enfant ?** Défaut : non, seul
   l'humain approuve. Raison : un enfant hérité a la même politique que son parent,
   donc le parent ne couvre rien de plus, et un parent modèle peut être induit en
   erreur par le texte de la demande. Seul cas utile : un enfant en lecture seule qui
   demande d'écrire dans la zone du parent, ce que son propre parent avait exclu.
   Réponse attendue : `non` ou `oui, pour l'enfant discovery seulement`.
3. **Enfants Claude et GLM.** Défaut : Codex seul. Les enfants Claude demandent de
   changer leur lancement (`--permission-prompts`), donc de rouvrir le contrat 149.
   Réponse attendue : `Codex seul` ou `inclure Claude` (ajoute une sonde et environ 6
   tâches).

Checkpoint historique après T041 : In Progress, 41/42, T039 et T041 documentaire
acquises ; T040 seule ouverte. Les checkpoints historiques 40/41 restent
historiques et sont remplacés par cet état courant. T041 corrige uniquement
le paragraphe Journal du contrat, sans changement de code ou de tests.
Preuve et lectures ciblées :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/implementation.md,
section T041 (transport codex_app_server.rs:165–178,4266–4310 ;
table delegation.rs:307–327).

## Checkpoint final avant Converge P3 — 42/42

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

## Statut de livraison155

Livré et activé le 2026-10-12 : source 59da3968c7d14dbd7acd1370875dd9d135ba7dcb,
42/42 ; trois Converge et contrôle cwd ciblé acquis. Audit final 2 LOW,
couverture 27,46 %, validateur 0/0. Build/install/activation et nettoyage attestés.
La clôture documentaire conserve le commit source du binaire ; les métadonnées ne changent pas ce build.
Les checkpoints In Progress et les chemins WT ci-dessus restent historiques.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/livraison155.md.
