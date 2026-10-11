# Déléguer avec Bridget

Bridget possède son moteur de délégation. T3 peut être absent. Le connecteur T3
atteste seulement la session qui appelle les outils. Il ne crée pas les tâches.

Ce moteur n'est pas l'outil `delegate_task` de T3. L'exécution ne dépend pas de T3.
T3 sert seulement à attester la session MCP.

Le parent consulte `bridget_capabilities`. Il choisit une entrée réellement
disponible. Le catalogue provient du registre natif Bridget. Les noms de modèle
et d'effort doivent correspondre exactement aux valeurs déclarées.

Un seul appel `bridget_delegate` crée l'enfant et remet sa mission. Exemple
de forme d'appel, à adapter au catalogue de la session :

```json
{
  "request_id": "revue-contrat-round1",
  "agent_type": "glm",
  "model": "glm-5.3",
  "task": "Lis le contrat indiqué et rends les défauts avec leur preuve.",
  "cwd": "/Users/moi/Nextcloud/10.Scripts/64.bridget"
}
```

`posture` est facultatif. Son absence demande l'héritage de la politique prouvée
du parent (voir plus bas).

`glm` est le nom du profil Bridget observé dans le registre de cette session.
Le nom d'instance T3 `claude_glm` n'est pas un type du registre Bridget.
Cet exemple ne prouve pas que ce profil existe dans tous les registres.
Un profil absent, un modèle différent ou un effort non déclaré produit un refus.
Bridget ne remplace jamais la cible demandée par une autre.

Le reçu conserve `task_id`, les références de l'enfant et de la mission.
Si la réponse se perd, reprendre les mêmes arguments avec le même `request_id`.
Le rejeu retrouve la tâche existante. Modifier son enveloppe produit
`envelope_mismatch`, sans nouvelle mission. Une clé neuve crée une nouvelle mission.
Ne pas changer de clé après une issue inconnue.

Le parent reçoit automatiquement un résultat corrélé à la mission.
`bridget_task_status` permet aussi de lire l'état durable avec `task_id`.
Un enfant actif retarde la publication du résultat de son parent.
Le résultat publié ne change plus. Une fin de tour ne suffit pas à le publier.
`bridget_task_cancel` annule sa tâche et ses descendants actifs.
Le catalogue annonce `mission_reply_limit_secs` (3600 secondes, le défaut),
`mission_reply_min_secs` (60) et `mission_reply_max_secs` (86 400).
`reply_timeout` est facultatif : il fixe le délai de réponse de cette mission,
en secondes entières. Sans lui, le délai reste 3600 secondes.
Le reçu donne `mission_deadline_at` après admission de la remise. Cette date brute
reste identique après rejeu ou reprise. Le temps de pause est exclu du délai métier.
L'expiration de ce délai produit `mission_reply_timeout`, puis le nettoyage.
Un délai de transport ou une attente du parent n'annule rien.

Après une panne du daemon, Bridget peut reprendre une mission `starting`,
`mission_pending` ou `working`. Il vérifie l'identité de flotte, les droits encore
valables et le temps restant. La tâche et la mission gardent leurs identifiants.
Deux reprises après redémarrage sont possibles ; la troisième échoue avec
`native_resume_exhausted`. Un résultat déjà capturé reste durable et attend la fin
des descendants avant sa remise. Rejouer la demande retrouve la même tâche.

La vue courante donne `resume_attempts`, le nombre de décisions de reprise après panne.
Ce compteur est sauvegardé avant la reprise. Un crash peut donc consommer une
tentative avant son effet ; il ne permet jamais de dépasser la borne.

## Attendre un résultat ou une réponse - contrat 151

Le parent peut attendre dans le même tour, sans relire le statut en boucle.
Ce contrat exige un daemon et un serveur MCP qui prennent en charge la session 151.
Les fichiers installés seuls ne prouvent pas que ces processus ont été rechargés.

`bridget_delegate` et `bridget_send` gardent leurs reçus immédiats. Conserver le
`task_id` de la délégation, ou l'`id` de l'envoi avec `reply: true`. Appeler ensuite
l'outil d'attente correspondant. Ces outils et `bridget_task_status` sont des lectures
pures : ils ne marquent aucune livraison, ne suppriment aucun message et ne provoquent
aucun nettoyage ni aucune purge. La remise du résultat suit toujours le chemin normal.

### Attendre une ou plusieurs tâches

```json
{"name":"bridget_task_wait","arguments":{"task_ids":["<task_id>"],"wait_timeout":600}}
```

`task_ids` contient 1 à 16 identifiants distincts de tâches du parent. `wait_timeout`
est un entier en secondes, de 1 à 3600 ; son défaut est 600. Tout champ inconnu ou
type incorrect est refusé. Une tâche inconnue, purgée ou détenue par un autre parent
fait refuser tout l'appel avec `task_unavailable`.

La réponse porte `version`, `timed_out`, `waited_secs`, `ready` et `pending`.
Elle revient dès qu'au moins une tâche est `result_available`, `failed` ou `cancelled`.
`ready` regroupe toutes les tâches terminées à cet instant ; `pending` contient les
autres identifiants. Les deux listes suivent l'ordre de `task_ids`. Une tâche déjà
terminée permet un retour immédiat. Si aucune tâche ne finit avant le délai,
`timed_out` vaut `true`. Les tâches continuent : l'expiration ne les annule pas.

Les entrées de `ready` omettent le texte complet `result`. Le parent le lit avec
`bridget_task_status` et le même `task_id`, ou par le message de résultat.
Le statut conserve toutes ses clés existantes, dont `result` et `error`.
Il ajoute les champs suivants :

| Champ | Sens |
|---|---|
| `work_state` | `working`, `waiting_for_children`, `result_available`, `failed` ou `cancelled`. |
| `summary` | Début du résultat publié, 2048 octets au plus, avec une coupe sur une frontière UTF-8. `null` hors `result_available`. |
| `summary_truncated` | `true` si le résumé omet une partie du résultat. |
| `result_bytes` | Taille du résultat en octets, ou `null` si aucun résultat n'est conservé. |
| `detail` | `{"tool":"bridget_task_status","task_id":"…"}` pour `result_available` et `failed` ; sinon `null`. |
| `mailbox` | État calculé de la boîte de fin, ou `null` pour une tâche non terminée ou annulée. |

`queued`, `starting`, `mission_pending`, `working` et `cancelling` donnent le même
`work_state: "working"`. Les quatre autres états gardent leur nom. Le résultat
publié est figé : deux lectures donnent le même résumé.

### Lire la boîte de fin et déclarer la lecture

| `mailbox.state` | Fait représenté |
|---|---|
| `pending` | Résultat ou avis d'échec prêt, encore sans envoi engagé. |
| `claimed` | Envoi engagé, sans accusé de remise prouvé. Une phase indéterminée, orpheline ou absente reste dans cet état. |
| `delivered` | Remise accusée : phase `acked` ou preuve durable `delivery_acked`. |
| `acknowledged` | Le parent a déclaré la lecture par `bridget_task_ack`. Ce fait ne prouve pas la remise du message. |

Une tâche déjà purgée par la rétention 150 n'a plus de boîte consultable.
L'outil rend `task_unavailable`, sans entrée `mailbox`.

Pour déclarer explicitement la lecture après avoir reçu le contenu :

```json
{"name":"bridget_task_ack","arguments":{"task_ids":["<task_id>"]}}
```

Cet outil accepte 1 à 16 identifiants distincts de tâches du parent, en
`result_available` ou `failed`. Il écrit `acknowledged_at` une seule fois.
La réponse sépare `acknowledged` (nouveaux accusés) et `already` (déjà déclarés),
avec `version: 1`. Une tâche non terminée ou annulée donne `task_not_ready` pour
tout l'appel, sans accusé partiel. Un identifiant inconnu ou étranger donne
`task_unavailable`. Cet accusé de lecture est informatif : aucun effet sur la
livraison, les messages, le nettoyage, la purge ou la projection Lineage.

Un résultat peut apparaître deux fois : résumé rendu par l'attente, puis message de
résultat. Le message reste envoyé même si l'attente a rendu le résumé ou a expiré.
L'accusé de lecture ne supprime pas cette copie. La rétention 150 continue à exiger
l'accusé de remise du message pour purger un arbre.

### Attendre la réponse d'un envoi suivi

```json
{"name":"bridget_request_wait","arguments":{"request_id":"<id_rendu_par_bridget_send>","wait_timeout":600}}
```

La demande provient d'un `bridget_send` avec `reply: true`. Seul son émetteur peut
l'attendre. `wait_timeout` a les mêmes bornes de 1 à 3600 secondes et le même défaut
de 600. Il ne remplace pas le `reply_timeout` de l'envoi.

La réponse porte `version`, `state`, `timed_out_wait`, `waited_secs`, `reply` et
`reply_reason` :

| `state` | Sens |
|---|---|
| `answered` | La demande est répondue. `reply` contient le résumé conservé, s'il existe. |
| `open` | Le délai de cet appel d'attente a expiré. `timed_out_wait` vaut `true` ; la demande reste ouverte. |
| `timed_out` | Le délai de réponse de la demande est passé. Cet état peut être calculé sur une ligne encore `open`, sans écriture. |
| `cancelled` | L'émetteur a annulé la demande par `bridget_cancel`. |

`reply`, lorsqu'il existe, contient `id`, `from`, `summary` et `summary_truncated`.
Son résumé garde 1024 octets de corps au plus, sur une frontière UTF-8. Il est figé
avec le premier passage à `answered`, dans la même transaction. Il ne dépend pas
du journal des messages (ledger), dont une ligne peut être remplacée.

Une demande `answered` sans résumé conservé rend `reply: null` et
`reply_reason: "reply_unavailable"`. Cela couvre le guichet, les lignes antérieures
à 151 et une enveloppe indisponible. Si le guichet clôture d'abord, une réponse de
message tardive ne remplace pas ce `null`. Hors de ce cas, `reply_reason` vaut `null`.

La réponse arrive aussi par le chemin normal d'injection. Elle peut donc être visible
deux fois : cette limite est FR-030. Après une coupure de l'hôte, rappeler
`bridget_request_wait` avec le même `request_id`. Cet appel n'envoie aucun message.

### Délais, disponibilité et refus

Les attentes se réveillent sur les changements. Un filet interne de 5 secondes
reste borné par le temps restant ; pour une demande, par son échéance aussi.
La pause du référent laisse les attentes possibles. Le client MCP garde sa connexion
pendant `wait_timeout + 10` secondes.

Il peut y avoir 4 attentes par processus MCP et 64 dans le daemon, tâches et demandes
réunies. Les autres créneaux MCP restent disponibles pour les appels de contrôle.
Une annulation MCP ferme le socket de l'appel, y compris pendant la sonde de support.
La fermeture de l'entrée MCP (EOF) ferme les attentes avant de joindre leurs fils.
Fermeture du client, annulation MCP ou arrêt du daemon libèrent l'attente en 6 secondes
au plus. Si le daemon redémarre, l'appel échoue avec `daemon_unreachable` côté client ;
le parent peut rappeler l'attente, sans renvoyer la mission ou la demande.

Avant une attente ou un accusé de lecture, la façade sonde `DaemonIdentityRequest`
avec un budget de 3 secondes. Le daemon annonce `features: ["wait_v1"]`.
Sans `wait_v1`, un ancien daemon donne `wait_unsupported` et ne reçoit aucune trame
d'attente ou d'accusé nouvelle. Un outil absent du catalogue peut aussi signaler
un serveur MCP ancien ; une installation de fichiers ne recharge pas ce processus.

Le catalogue annonce `wait_timeout_default_secs: 600` et
`wait_timeout_max_secs: 3600`. Le réglage `tool_timeout_sec = 3700` s'applique au seul
montage Codex de Bridget. Dans un autre hôte ou montage, choisir un `wait_timeout`
inférieur au délai d'outil de cet hôte. Une coupure reste récupérable par un nouvel
appel d'attente en lecture pure.

| Refus | Cause |
|---|---|
| `InvalidParams` | Champ inconnu ou type incorrect dans les arguments MCP des trois outils. |
| `invalid_request` | Identifiants de tâche invalides, liste vide, plus de 16 éléments ou doublon ; demande invalide. |
| `wait_timeout_out_of_range` | `wait_timeout` hors de 1 à 3600 secondes. |
| `wait_unsupported` | Le daemon n'annonce pas `wait_v1`. |
| `task_unavailable` | Tâche inconnue, purgée ou détenue par un autre parent. |
| `request_unavailable` | Demande inconnue ou émise par un autre agent. |
| `wait_capacity` | Les 4 attentes du processus MCP ou les 64 attentes du daemon sont occupées. |
| `task_not_ready` | Accusé de lecture demandé pour une tâche non terminée ou annulée. |

Pour l'attente de tâches, l'ordre daemon est `invalid_request`,
`wait_timeout_out_of_range`, `wait_unsupported`, `task_unavailable`, `wait_capacity`.
La façade peut refuser sa capacité avant de valider les arguments. Les refus
n'écrivent aucun état. Les erreurs d'identité restent celles de la session 149.
La session 151 n'ajoute aucune commande CLI humaine.

## Droits du parent et de l'enfant

Le parent doit être inscrit dans Bridget avec une identité et un projet attestés.
Il ne choisit pas son identité dans les arguments des outils. L'enfant reçoit
la mission demandée et les faits natifs d'identité et de Git utiles à sa reprise.
Ces faits sont joints au premier tour. Il ne reçoit ni la conversation du parent
ni ses credentials T3.

Le catalogue indique le périmètre du dossier avec `cwd_scope` :

- `project` : le parent a une preuve de droits. La mission doit rester dans
  `cwd_root`, c'est-à-dire le dossier effectif du parent. Un dossier hors de cette
  racine est refusé (`cwd_outside_parent_project`).
- `root` : un grant humain explicite fixe la racine.
- `same_project` : parent externe avec projet attesté, sans grant. `cwd_root` vaut
  `null`. Ce parent peut inspecter ce dépôt et ses worktrees.

Un dépôt étranger reste refusé. Le dossier `.git` ne sert jamais de racine de droits.

**Posture (facultative) :**

- **Absente (cas normal) :** l'enfant hérite de la politique que le parent a prouvée.
  Bridget n'ajoute aucun droit et ne demande aucun grant. Si la cible ne peut pas
  garder cette politique, le motif figure dans `inherit_refusal` du catalogue.
- **`discovery` :** avec un parent attesté, la mission reste en lecture seule. Sans
  preuve, cette valeur suit le chemin 148 : un grant humain existant est requis,
  sinon `delegation_grant_required`.
- **`development` :** avec un parent attesté, l'écriture n'est accordée que si la
  politique du parent et la cible le permettent. Sans preuve, la demande est refusée
  avec `permission_attestation_unavailable`. Le motif éventuel figure dans
  `development_refusal`.

**Ce qui passe d'un parent à une cible (149) :**

- **Codex vers Codex :** Bridget garde la politique d'approbation, le bac à sable et le
  réseau du parent. Un parent en lecture seule ne donne pas `development`
  (`permission_not_inherited`). Un bac à sable externe est refusé
  (`provider_confinement_unavailable`).
- **Codex vers Claude ou GLM avec écriture :** seulement si le parent est en `dangerFullAccess` avec
  approbation `never`. L'enfant passe en `bypassPermissions`, dans la seule définition
  de cette mission. Un parent en `workspaceWrite` est refusé
  (`provider_confinement_unavailable`). Un parent en lecture seule donne une mission en
  mode plan, avec `Read`, `Glob` et `Grep`.
- **Claude ou GLM vers Claude ou GLM :** la mission réutilise les mêmes entrées : même
  lanceur, même profil, mêmes sources et mêmes empreintes. Le dossier de l'enfant doit
  être celui du parent. Les règles `allow` et `deny` du parent restent en vigueur.
  Bridget n'ajoute aucun bypass universel. Si une entrée change, la demande est refusée
  avec `settings_revision_changed`. Une source opaque, par exemple une politique gérée
  non observable, donne `permission_source_unavailable`.
- **Claude ou GLM vers Codex :** seulement si la politique est un bypass complet sans
  règle, ou une lecture seule limitée à `Read`, `Glob` et `Grep`.

Dans tous les autres cas, la réponse est `permission_mapping_unavailable`. Tous ces
refus arrivent avant la création de la tâche : aucun enfant n'est lancé.

Un humain peut accorder ou retirer des droits natifs à une instance avec
`bridget delegate-grant`, dans un terminal humain. Le grant porte sur l'instance,
une racine absolue et une posture maximale. Le MCP ne crée jamais ce grant.

Un grant n'est requis que pour la voie 148 : `discovery` sans preuve de droits du
parent. Une délégation héritée d'un parent attesté n'en demande aucun.

La révocation reste prioritaire. Elle ferme les nouvelles demandes du même agent,
même après changement d'instance, héritage compris (`delegation_grant_required`).
Seul un nouveau grant humain explicite lève ce refus durable. Elle ne transforme pas
le rejeu d'une demande déjà acceptée en une nouvelle mission.

## Identité dans T3

Deux conversations peuvent partager un processus fournisseur. Chaque montage
MCP Bridget reçoit une preuve privée distincte. Bridget la vérifie auprès du
serveur T3 à chaque opération, puis vérifie son inscription vivante au daemon.
Une preuve partielle, périmée ou révoquée produit un refus explicite.
Bridget n'emprunte jamais l'identité d'une conversation voisine.

Un credential retiré, révoqué ou tourné ferme tous les outils Bridget de cette
session avec `t3_session_unavailable`. Ce refus n'a ni repli par PID ni repli en
lecture seule. Une session qui n'a jamais eu de fait de droits garde l'identité seule (version 1)
pour les lectures et le rejeu d'une demande déjà admise. Une admission héritée ou
`development` exige la version 2 : sans elle, la réponse est
`permission_attestation_unavailable`, sans grant ni repli vers `discovery`.

Les options MCP humaines existantes et les désactivations restent prioritaires.
Les outils148 deviennent disponibles au chargement des nouveaux processus.
Une installation de fichiers seule ne recharge pas une session déjà ouverte.
Les comportements 149 exigent le daemon et le binaire 149 chargés. Ce guide ne
prouve pas leur activation.

Au redémarrage, Bridget réconcilie la commande, l'instance et le lien natifs depuis
les faits attestés. Une incohérence native non réconciliable fait échouer la tâche
avec `native_restart_identity_mismatch` ; le daemon poursuit son démarrage.
Cette garantie couvre un marqueur lisible. Un marqueur corrompu ou illisible garde
le refus sûr de la flotte ; sa réparation générale ne fait pas partie du contrat152.
Une incohérence de cible d'exécution ou une erreur de stockage peut encore
bloquer le démarrage. Bridget conserve ses contrôles de propriété et de clôture.
Les erreurs `parent_task_unavailable`, `native_owner_identity_mismatch`,
`native_execution_changed` et `native_execution_mismatch` exigent aussi un diagnostic
des données conservées. Ne supprimez pas les marqueurs pour forcer une reprise.
Si un wrapper natif reste vivant après le délai commun de 8 secondes, le daemon
refuse aussi de démarrer. Il conserve le marqueur et ne force pas cet arrêt.

Si le suivi durable d'une mission admise manque, Bridget ferme son exécution
avec les contrôles existants avant de rendre `request_unavailable`. Une erreur
de lecture ou de clôture reporte le traitement. Elle ne crée pas de date supposée.
Cette fermeture termine aussi la pause liée. Elle ne crée aucune reprise.

## Suivi humain et panneau Lineage

Le terminal humain lit une tâche avec `bridget lineage inspect`, `watch` et `cancel`.
Ces commandes demandent `--t3-thread` (fil T3 du parent) et `--project-root` (chemin
absolu). Elles n'utilisent pas le MCP. Le fil T3 sert à la lecture : il ne crée pas la
tâche.

Dans T3, le panneau Lineage affiche l'arbre des tâches du fil parent : la tâche
racine, puis les enfants imbriqués. Une tâche se lit dans son journal et son résultat.
Ce panneau n'est pas un fil de conversation. Il n'a pas de zone de saisie, et l'enfant
ne reçoit aucun tour. Le bouton Arrêter demande l'annulation native de la tâche et de
ses descendants.

Statuts natifs : `queued`, `starting`, `mission_pending`, `working`,
`waiting_for_children`, `cancelling`, `result_available`, `failed`, `cancelled`. Un
statut inconnu ferme le contrat.

Preuve : l'interface web a été testée avec un daemon simulé. Aucune preuve ne couvre
le daemon réel, l'application de bureau ou une installation. Ne pas annoncer cette
fonction comme livrée.

## Délai de réponse choisi

`reply_timeout` fait partie de l'enveloppe de la demande. Rejouer la même clé avec un
autre délai produit `envelope_mismatch`. Omettre le champ et envoyer `3600` sont deux
enveloppes différentes.

Une valeur hors de 60 à 86 400 est refusée. Bridget ne la corrige pas.

Un enfant qui fixe un `reply_timeout` ne peut pas dépasser le temps restant effectif
de sa tâche parente. Ce temps exclut les pauses. Sans délai explicite, ce contrôle
reste celui du contrat149. Le suivi de la demande compte depuis l'envoi de la mission,
avec un délai explicite ou omis. Il ne compte pas le retard du lancement.
Les tâches déjà actives sans délai choisi gardent leur délai de 3600 secondes.

La date brute `mission_deadline_at` reste la date durable. Elle ne donne pas à elle
seule le temps restant après une pause. `bridget_request_wait` utilise le délai métier
effectif d'une demande native, sans changer sa date ni son état.
Le budget `wait_timeout` de l'appelant continue pendant la pause. Une demande ordinaire
garde le comportement151.

## Refus

Le refus revient dans `native_delegation_refused`, avec son code. Aucun refus ne crée
de tâche. Les cinq refus définitifs ci-dessous engagent la clé de rejeu pendant
30 jours. Tout autre refus reste sans reçu enregistré par le contrat152.

| Code | Cause |
|---|---|
| `referent_paused` | Le référent est en pause et la clé est nouvelle. |
| `control_state_unavailable` | L'état de pause est illisible : Bridget refuse plutôt que de déléguer pendant une pause possible. |
| `request_expired` | La tâche de cette clé a été purgée depuis moins de 30 jours. |
| `retention_saturated` | 4096 tâches sont conservées et aucun arbre n'est purgeable. |
| `reply_timeout_out_of_range` | `reply_timeout` est hors de 60 à 86 400. |
| `reply_timeout_exceeds_parent` | Le délai explicite d'un enfant dépasse le temps restant du parent. Sans délai explicite, aucun contrôle (comportement 149). |

Ordre des contrôles : identité, forme de la demande, bornes du délai, rejeu
(`envelope_mismatch`, `request_expired`), pause (`referent_paused`, ou
`control_state_unavailable` si l'état de pause est illisible), droits, `task_limit`,
`retention_saturated`, lignée du parent, délai de l'enfant.

`task_limit` reste pour 128 tâches actives, ou 16 enfants actifs par instance.
L'historique cumulé ne le provoque plus.

### Reçus des refus définitifs

| Code conservé | Cause |
|---|---|
| `cwd_outside_parent_project` | Le dossier demandé sort du projet autorisé du parent. |
| `cwd_unavailable` | Le dossier demandé ne peut pas être résolu. |
| `model_unavailable` | Le modèle demandé est absent du catalogue autorisé. |
| `effort_unavailable` | L'effort demandé est absent du catalogue autorisé. |
| `agent_type_unknown` | Le type demandé est inconnu du registre. |

Le rejeu de la même clé avec les mêmes arguments rend le premier refus, même si
le dossier ou le catalogue a changé. Une autre enveloppe rend `envelope_mismatch`.
Une tâche déjà admise prime sur un reçu de refus. L'entretien retire les reçus
âgés de plus de 30 jours ; un reçu reste présent à 30 jours exactement.

La pause, la saturation, un défaut d'identité ou de droits ne sont pas enregistrés
comme refus définitifs. Le registre conserve seulement `agent_type_unknown`,
jamais son texte dynamique. Ne pas changer automatiquement de clé après une panne
de transport ou une issue inconnue.

## Pause du référent - contrat152

Pendant la pause, une nouvelle clé reçoit `referent_paused`. Rejouer une clé déjà
admise rend la tâche existante. Un reçu de refus définitif garde sa réponse.
La lecture du catalogue, la lecture de statut et les attentes restent disponibles.
Une admission refusée seulement pour la pause peut être rejouée après sa levée.

Une tâche `queued`, `starting`, `mission_pending` ou `working` garde son état
et rend `suspended: true` dans sa vue courante. Lineage garde ses statuts existants.
Bridget ne lance pas un enfant retenu en file et ne remet pas une nouvelle mission.
Une interruption due à la pause ne fait pas échouer la tâche.

Le délai métier de mission et le suivi de sa demande excluent les pauses depuis
leur admission durable. Une pause avant l'envoi ne retire pas du temps au suivi.
Ce calcul survit à un redémarrage, même avant le premier passage du moteur.

À la levée, une exécution encore `interrupting` attend de devenir `interrupted`.
La reprise garde le message et la mission d'origine, puis suit leur dernière
exécution. La seule levée de pause ne consomme pas le compteur de reprises après panne.

Un lancement déjà soumis continue pendant la pause. Son délai de 300 secondes
continue aussi ; il peut donc expirer. Son enveloppe reste identique sur rejeu.
Un lancement retenu avant sa première soumission reçoit sa date d'émission à la levée.

Les résultats et avis d'échec déjà acquis peuvent encore être remis. Le nettoyage
et l'annulation restent possibles. Annuler pendant la pause empêche la mission
de reprendre à la levée.

Ces comportements demandent un daemon chargé en version152. Une installation de
fichiers seule ne prouve pas l'activation. Un agent ne redémarre pas le daemon
ou le fournisseur pour obtenir ce comportement.

## Annulation vérifiée avant les arrêts

`bridget_task_cancel` demande l'annulation de la tâche et de ses descendants.
Bridget vérifie d'abord que tous les membres sont gérés et arrêtables.
Si un membre ne l'est pas, aucun arrêt n'est exécuté et aucun descendant n'est
marqué pour annulation. La racine reste `cancelling` avec
`native_cancel_unverifiable:<enfant>`. Le contrôle sera refait depuis cette racine.

Après un contrôle complet, les descendants sont arrêtés avant le parent.
Chaque arrêt confirmé est conservé. Un nouvel essai ou une reprise après panne
ne rejoue que les arrêts sans confirmation durable.
Un arrêt peut échouer après le contrôle initial ; les confirmations déjà acquises
restent conservées et la racine reste `cancelling`.
Une erreur ou un résultat terminal déjà acquis reste conservé pendant le nettoyage.

| Erreur de reprise ou d'annulation | Lecture |
|---|---|
| `native_resume_exhausted` | Deux reprises après panne ont déjà été décidées. |
| `mission_reply_timeout` | Le délai métier hors pause est dépassé. |
| `unreachable` | Les conditions de reprise ne sont pas réunies. |
| `native_restart_identity_mismatch` | L'identité native ne peut pas être réconciliée depuis les faits attestés. |
| `native_cancel_unverifiable:<enfant>` | Annulation bloquée ; racine `cancelling`. Au précontrôle refusé, aucun arrêt. Après un échec d'arrêt, les confirmations précédentes restent conservées. |

## Rétention des tâches

Bridget garde au plus 4096 tâches. Il purge par arbre : une tâche sans parent et tous ses
descendants. L'entretien horaire purge les arbres finis depuis plus de 7 jours, 128 au
plus par passage. Si 4096 tâches sont conservées, les arbres purgeables les plus anciens
partent d'abord, même avant 7 jours. L'entretien le fait dans le même quota de 128 arbres
par passage, partagé avec la purge d'âge. Une nouvelle délégation le fait aussi : elle
retire seulement les arbres nécessaires pour repasser sous 4096, sans limite de 128.

Un arbre n'est jamais purgé dans ces cas :

- une de ses tâches est active ou son nettoyage n'est pas fini ;
- un résultat ou un avis d'échec n'a pas été accusé par le destinataire ;
- un de ses sous-agents est connecté, routé, en cours de lancement, en cours de
  décommission ou désiré `Running`.

Avant la purge, chaque sous-agent est retiré de la flotte. Une remise indéterminée n'a
pas d'accusé : elle bloque donc son arbre. Si ces arbres remplissent la borne, la
délégation est refusée avec `retention_saturated`.

Après une purge, `bridget_task_status` sur cette tâche rend le refus existant de tâche
inconnue (`task_unavailable`). La vue Lineage fait une lecture complète sans tâche
fantôme. Le rejeu de la clé rend `request_expired` pendant 30 jours. La clé redevient
libre au passage d'entretien qui suit ces 30 jours (1 h au plus de décalage).

## Ce qui ne change pas

- Le lancement ordinaire (`bridget spawn`), ses postures et ses protections du
  terminal humain ne changent pas. Ce guide ne remplace pas leur tutoriel.
- `bridget delegate-grant` reste une commande humaine de terminal.
